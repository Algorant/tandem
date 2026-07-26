//! Concrete filesystem boundary for one discovered Tandem project.
//!
//! This module resolves project-local paths and reads source documents without
//! interpreting their protocol meaning. The executable `protocol` module owns
//! validation, hierarchy, workflow, accord, review, and event semantics.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use yaml_rust2::{Yaml, YamlLoader};

use crate::protocol::document::Document as ProtocolDocument;
use crate::CliError;

#[derive(Debug, Clone)]
pub(crate) struct TandemProject {
    pub(crate) board_dir: PathBuf,
    pub(crate) logs_dir: PathBuf,
    pub(crate) config_path: PathBuf,
    pub(crate) events_path: PathBuf,
}

impl TandemProject {
    pub(crate) fn discover() -> Result<Self, CliError> {
        Self::discover_from(&env::current_dir()?)
    }

    pub(crate) fn discover_from(start: &Path) -> Result<Self, CliError> {
        let mut dir = start.to_path_buf();
        loop {
            let tandem_dir = dir.join(".tandem");
            let config_path = tandem_dir.join("tandem.md");
            if config_path.is_file() {
                return Ok(Self::with_paths(dir, tandem_dir, config_path));
            }

            // The normative compatibility path is deliberately checked after
            // `.tandem/tandem.md` and only within the repository boundary.
            let compatibility_config = dir.join("tandem.md");
            if compatibility_config.is_file() {
                return Ok(Self::with_paths(
                    dir.clone(),
                    tandem_dir,
                    compatibility_config,
                ));
            }
            if dir.join(".git").exists() || !dir.pop() {
                break;
            }
        }
        Err(CliError::user(
            "No Tandem workspace found. Run `tandem init` first.",
        ))
    }

    fn with_paths(_root: PathBuf, data_dir: PathBuf, config_path: PathBuf) -> Self {
        Self {
            board_dir: data_dir.join("board"),
            logs_dir: data_dir.join("logs"),
            events_path: data_dir.join("events.jsonl"),
            config_path,
        }
    }

    /// The resolved project root for standard and compatibility discovery.
    pub(crate) fn root(&self) -> PathBuf {
        let parent = self.config_path.parent().unwrap_or_else(|| Path::new("."));
        if parent.file_name().is_some_and(|name| name == ".tandem") {
            parent.parent().unwrap_or(parent).to_path_buf()
        } else {
            parent.to_path_buf()
        }
    }

    /// The resolved project-local data directory. Compatibility projects keep
    /// their config at the root but retain this conventional data location.
    pub(crate) fn data_dir(&self) -> PathBuf {
        self.board_dir
            .parent()
            .unwrap_or_else(|| Path::new(".tandem"))
            .to_path_buf()
    }

    pub(crate) fn read_board_documents(&self) -> Result<Vec<StoredDocument>, CliError> {
        read_documents(&self.board_dir, DocumentLocation::Board)
    }

    pub(crate) fn read_log_documents(&self) -> Result<Vec<StoredDocument>, CliError> {
        read_documents(&self.logs_dir, DocumentLocation::Logs)
    }

    pub(crate) fn read_documents(&self) -> Result<Vec<StoredDocument>, CliError> {
        let mut docs = self.read_board_documents()?;
        docs.extend(self.read_log_documents()?);
        Ok(docs)
    }

    pub(crate) fn find_document(&self, id: &str) -> Result<Option<StoredDocument>, CliError> {
        Ok(self
            .read_documents()?
            .into_iter()
            .find(|document| document.id() == id))
    }

    pub(crate) fn read_board_document(&self, id: &str) -> Result<Option<StoredDocument>, CliError> {
        Ok(self
            .read_board_documents()?
            .into_iter()
            .find(|document| document.id() == id))
    }

    pub(crate) fn read_config_yaml(&self) -> Result<Option<Yaml>, CliError> {
        read_frontmatter_yaml_file(&self.config_path)
    }

    pub(crate) fn read_board_documents_tolerant(
        &self,
        warnings: &mut Vec<String>,
    ) -> Vec<StoredDocument> {
        read_documents_tolerant(&self.board_dir, DocumentLocation::Board, "Board", warnings)
    }

    pub(crate) fn read_log_documents_tolerant(
        &self,
        warnings: &mut Vec<String>,
    ) -> Vec<StoredDocument> {
        read_documents_tolerant(&self.logs_dir, DocumentLocation::Logs, "Logs", warnings)
    }

    pub(crate) fn read_events_tolerant(&self, warnings: &mut Vec<String>) -> Vec<ProjectEvent> {
        if !self.events_path.exists() {
            return Vec::new();
        }
        let content = match fs::read_to_string(&self.events_path) {
            Ok(content) => content,
            Err(error) => {
                warnings.push(format!(
                    "Events load warning: could not read {}: {error}",
                    display_path(&self.events_path)
                ));
                return Vec::new();
            }
        };
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(ProjectEvent::parse)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectEvent {
    pub(crate) id: String,
    pub(crate) ts: String,
    pub(crate) event: String,
    pub(crate) summary: String,
}

impl ProjectEvent {
    fn parse(line: &str) -> Option<Self> {
        Some(Self {
            id: extract_json_string(line, "id")?,
            event: extract_json_string(line, "event").unwrap_or_else(|| "event".to_string()),
            ts: extract_json_string(line, "ts").unwrap_or_default(),
            summary: extract_json_string(line, "summary").unwrap_or_default(),
        })
    }
}

pub(crate) fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{key}\"");
    let after_key = line.find(&key_pattern)? + key_pattern.len();
    let colon_offset = line[after_key..].find(':')?;
    let mut cursor = after_key + colon_offset + 1;
    while let Some(ch) = line[cursor..].chars().next() {
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }
    if line[cursor..].chars().next()? != '"' {
        return None;
    }
    cursor += 1;
    let mut value = String::new();
    let mut escaped = false;
    for ch in line[cursor..].chars() {
        if escaped {
            value.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

/// Raw project source paired with its protocol-level document value.
#[derive(Debug, Clone)]
pub(crate) struct StoredDocument {
    pub(crate) path: PathBuf,
    pub(crate) location: DocumentLocation,
    document: ProtocolDocument,
}

impl StoredDocument {
    pub(crate) fn id(&self) -> &str {
        self.document.id()
    }

    pub(crate) fn title(&self) -> &str {
        self.document.title()
    }

    pub(crate) fn new(
        path: PathBuf,
        location: DocumentLocation,
        fields: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            path,
            location,
            document: ProtocolDocument::new(fields, body),
        }
    }

    pub(crate) fn diagnostic_source_label(&self) -> String {
        display_path(&self.path)
    }
}

impl Deref for StoredDocument {
    type Target = ProtocolDocument;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}

impl DerefMut for StoredDocument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.document
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentLocation {
    Board,
    Logs,
}

impl DocumentLocation {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Board => "board",
            Self::Logs => "logs",
        }
    }
}

pub(crate) fn read_documents(
    dir: &Path,
    location: DocumentLocation,
) -> Result<Vec<StoredDocument>, CliError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| read_document(&path, location))
        .collect()
}

pub(crate) fn read_document(
    path: &Path,
    location: DocumentLocation,
) -> Result<StoredDocument, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::user(format!("failed to read {}: {error}", display_path(path)))
    })?;
    let (frontmatter, body) = split_frontmatter(&content).map_err(|message| {
        CliError::user(format!("Parse failure: {}: {message}", display_path(path)))
    })?;
    let fields = parse_frontmatter_fields(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(path)
        ))
    })?;
    Ok(StoredDocument::new(
        path.to_path_buf(),
        location,
        fields,
        body,
    ))
}

pub(crate) fn read_documents_tolerant(
    dir: &Path,
    location: DocumentLocation,
    label: &str,
    warnings: &mut Vec<String>,
) -> Vec<StoredDocument> {
    if !dir.exists() {
        return Vec::new();
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!(
                "{label} load failed: could not read {}: {error}",
                display_path(dir)
            ));
            return Vec::new();
        }
    };
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry)
                if entry.path().extension().and_then(|value| value.to_str()) == Some("md") =>
            {
                paths.push(entry.path())
            }
            Ok(_) => {}
            Err(error) => warnings.push(format!(
                "{label} load warning: could not inspect entry in {}: {error}",
                display_path(dir)
            )),
        }
    }
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| match read_document(&path, location) {
            Ok(document) => Some(document),
            Err(error) => {
                warnings.push(format!("{label} load warning: {}", error.message));
                None
            }
        })
        .collect()
}

pub(crate) fn split_frontmatter(content: &str) -> Result<(String, String), &'static str> {
    let first_line_end = content.find('\n').ok_or("missing frontmatter delimiter")?;
    if content[..first_line_end].trim_end_matches('\r') != "---" {
        return Err("missing opening frontmatter delimiter");
    }
    let frontmatter_start = first_line_end + 1;
    let mut cursor = frontmatter_start;
    while cursor <= content.len() {
        let line_start = cursor;
        let Some(relative_newline) = content[cursor..].find('\n') else {
            break;
        };
        let line_end = cursor + relative_newline;
        if content[line_start..line_end].trim_end_matches('\r').trim() == "---" {
            return Ok((
                content[frontmatter_start..line_start].to_string(),
                content[line_end + 1..].to_string(),
            ));
        }
        cursor = line_end + 1;
    }
    Err("missing closing frontmatter delimiter")
}

pub(crate) fn parse_frontmatter_fields(
    frontmatter: &str,
) -> Result<HashMap<String, String>, String> {
    let Some(root) = parse_frontmatter_yaml(frontmatter)? else {
        return Ok(HashMap::new());
    };
    let hash = root
        .as_hash()
        .ok_or_else(|| "frontmatter root must be a YAML mapping".to_string())?;
    let mut fields = HashMap::new();
    flatten_yaml_hash(hash, "", &mut fields);
    add_status_aliases(&mut fields);
    Ok(fields)
}

pub(crate) fn parse_frontmatter_yaml(frontmatter: &str) -> Result<Option<Yaml>, String> {
    if frontmatter.trim().is_empty() {
        return Ok(None);
    }
    let docs = YamlLoader::load_from_str(frontmatter).map_err(|error| error.to_string())?;
    if docs.is_empty() {
        return Ok(None);
    }
    if docs.len() > 1 {
        return Err("frontmatter must contain exactly one YAML document".to_string());
    }
    let root = docs
        .into_iter()
        .next()
        .expect("checked non-empty YAML documents");
    if root.is_badvalue() {
        return Err("frontmatter root must be a YAML mapping".to_string());
    }
    Ok(Some(root))
}

pub(crate) fn read_frontmatter_yaml_file(path: &Path) -> Result<Option<Yaml>, CliError> {
    let content = fs::read_to_string(path)?;
    let (frontmatter, _) = split_frontmatter(&content).map_err(|message| {
        CliError::user(format!("Parse failure: {}: {message}", display_path(path)))
    })?;
    parse_frontmatter_yaml(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(path)
        ))
    })
}

fn flatten_yaml_hash(
    hash: &yaml_rust2::yaml::Hash,
    prefix: &str,
    fields: &mut HashMap<String, String>,
) {
    for (key, value) in hash {
        let Some(key) = yaml_scalar_to_string(key) else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        let field_key = if prefix.is_empty() {
            key
        } else {
            format!("{prefix}.{key}")
        };
        flatten_yaml_value(&field_key, value, fields);
    }
}
fn flatten_yaml_value(prefix: &str, value: &Yaml, fields: &mut HashMap<String, String>) {
    match value {
        Yaml::Hash(hash) => flatten_yaml_hash(hash, prefix, fields),
        Yaml::Array(values) => {
            if let Some(inline) = yaml_array_field_value(values) {
                fields.insert(prefix.to_string(), inline);
            } else {
                for (index, item) in values.iter().enumerate() {
                    flatten_yaml_value(&format!("{prefix}.{index}"), item, fields);
                }
            }
        }
        _ => {
            if let Some(value) = yaml_scalar_to_string(value) {
                if !value.is_empty() {
                    fields.insert(prefix.to_string(), value);
                }
            }
        }
    }
}
fn yaml_array_field_value(values: &[Yaml]) -> Option<String> {
    Some(format!(
        "[{}]",
        values
            .iter()
            .map(yaml_scalar_to_string)
            .collect::<Option<Vec<_>>>()?
            .iter()
            .map(|value| yaml_double_quote(value))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}
pub(crate) fn yaml_scalar_to_string(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(value) | Yaml::Real(value) => Some(value.clone()),
        Yaml::Integer(value) => Some(value.to_string()),
        Yaml::Boolean(value) => Some(value.to_string()),
        Yaml::Null | Yaml::BadValue | Yaml::Array(_) | Yaml::Hash(_) | Yaml::Alias(_) => None,
    }
}
pub(crate) fn yaml_mapping_value<'a>(root: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    root.as_hash()?.iter().find_map(|(candidate, value)| {
        (yaml_scalar_to_string(candidate).as_deref() == Some(key)).then_some(value)
    })
}
fn yaml_double_quote(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}
fn add_status_aliases(fields: &mut HashMap<String, String>) {
    copy_first_alias(fields, "accordStatus", &["accord.status"]);
    copy_first_alias(fields, "reviewStatus", &["review.status"]);
    copy_first_alias(fields, "completionSummary", &["completion.summary"]);
    copy_first_alias(
        fields,
        "completionValidation",
        &[
            "completion.validation",
            "completion.validation.summary",
            "completion.validation.status",
        ],
    );
    copy_first_alias(fields, "completionReviewer", &["completion.reviewer"]);
    copy_first_alias(fields, "filesChanged", &["completion.filesChanged"]);
}
fn copy_first_alias(fields: &mut HashMap<String, String>, alias: &str, sources: &[&str]) {
    if fields.contains_key(alias) {
        return;
    }
    for source in sources {
        if let Some(value) = fields.get(*source).cloned() {
            fields.insert(alias.to_string(), value);
            return;
        }
    }
}

pub(crate) fn display_path(path: &Path) -> String {
    match env::current_dir() {
        Ok(current_dir) => path
            .strip_prefix(&current_dir)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| path.display().to_string()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovers_dot_tandem_before_root_compatibility_file() {
        let root = env::temp_dir().join(format!("tandem-project-{}", std::process::id()));
        let data = root.join(".tandem");
        fs::create_dir_all(&data).unwrap();
        fs::write(root.join("tandem.md"), "root").unwrap();
        fs::write(data.join("tandem.md"), "nested").unwrap();
        let project = TandemProject::discover_from(&root).unwrap();
        assert_eq!(project.config_path, data.join("tandem.md"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn preserves_unknown_fields_and_body_with_source_location() {
        let document = read_document_from(
            "---\nid: task-1\nunknown: keep\n---\n# Body\n",
            DocumentLocation::Board,
        );
        assert_eq!(document.field("unknown"), Some("keep"));
        assert_eq!(document.body, "# Body\n");
        assert_eq!(document.location, DocumentLocation::Board);
    }
    fn read_document_from(content: &str, location: DocumentLocation) -> StoredDocument {
        let (frontmatter, body) = split_frontmatter(content).unwrap();
        StoredDocument::new(
            PathBuf::from("source.md"),
            location,
            parse_frontmatter_fields(&frontmatter).unwrap(),
            body,
        )
    }
}
