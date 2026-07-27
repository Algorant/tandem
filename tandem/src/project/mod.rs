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
use crate::protocol::hierarchy::{
    DocumentLocation, HierarchyDocument, HierarchyIndex as ProtocolHierarchyIndex,
    ParentRelationship, TaskRole,
};
use crate::CliError;

pub(crate) mod events;
pub(crate) mod frontmatter;
pub(crate) mod write;
pub(crate) use frontmatter::{
    frontmatter_line_key, is_top_level_frontmatter_boundary, patch_frontmatter_content,
    replace_markdown_body,
};
pub(crate) use write::{read_file_snapshot, write_atomic};

#[derive(Debug, Clone)]
pub(crate) struct TandemProject {
    pub(crate) root: PathBuf,
    pub(crate) data_dir: PathBuf,
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

    fn with_paths(root: PathBuf, data_dir: PathBuf, config_path: PathBuf) -> Self {
        Self {
            root,
            data_dir: data_dir.clone(),
            board_dir: data_dir.join("board"),
            logs_dir: data_dir.join("logs"),
            events_path: data_dir.join("events.jsonl"),
            config_path,
        }
    }

    /// The resolved project root for standard and compatibility discovery.
    pub(crate) fn root(&self) -> &Path {
        if self.root.as_os_str().is_empty() {
            let parent = self.config_path.parent().unwrap_or_else(|| Path::new("."));
            if parent.file_name().is_some_and(|name| name == ".tandem") {
                parent.parent().unwrap_or(parent)
            } else {
                parent
            }
        } else {
            &self.root
        }
    }

    /// The resolved project-local data directory. Compatibility projects keep
    /// their config at the root but retain this conventional data location.
    pub(crate) fn data_dir(&self) -> &Path {
        if self.data_dir.as_os_str().is_empty() {
            self.board_dir
                .parent()
                .unwrap_or_else(|| Path::new(".tandem"))
        } else {
            &self.data_dir
        }
    }

    /// Materializes a newly validated project layout using caller-supplied
    /// config bytes; interpretation of those bytes remains outside project.
    pub(crate) fn initialize(root: &Path, config: &str) -> Result<Self, CliError> {
        let data_dir = root.join(".tandem");
        let created_data_dir = !data_dir.exists();
        let result = (|| {
            fs::create_dir_all(data_dir.join("board"))?;
            fs::create_dir_all(data_dir.join("logs"))?;
            fs::create_dir_all(data_dir.join("events"))?;
            let config_path = data_dir.join("tandem.md");
            fs::write(&config_path, config)?;
            Ok(Self::with_paths(
                root.to_path_buf(),
                data_dir.clone(),
                config_path,
            ))
        })();
        if result.is_err() && created_data_dir {
            let _ = fs::remove_dir_all(&data_dir);
        }
        result
    }

    pub(crate) fn events_dir(&self) -> PathBuf {
        self.data_dir().join("events")
    }

    pub(crate) fn actor_events_path(&self, actor: &str) -> PathBuf {
        self.events_dir().join(format!("{actor}.jsonl"))
    }

    pub(crate) fn read_config_raw(&self) -> Result<String, CliError> {
        fs::read_to_string(&self.config_path).map_err(Into::into)
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
        let mut events = read_event_file_tolerant(&self.events_path, warnings);
        let events_dir = self.events_dir();
        let entries = match fs::read_dir(&events_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return events,
            Err(error) => {
                warnings.push(format!(
                    "Events load warning: could not read {}: {error}",
                    display_path(&events_dir)
                ));
                return events;
            }
        };
        let mut paths = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry)
                    if entry.path().extension().and_then(|value| value.to_str())
                        == Some("jsonl") =>
                {
                    paths.push(entry.path());
                }
                Ok(_) => {}
                Err(error) => warnings.push(format!(
                    "Events load warning: could not inspect entry in {}: {error}",
                    display_path(&events_dir)
                )),
            }
        }
        paths.sort();
        for path in paths {
            events.extend(read_actor_event_file_tolerant(&path, warnings));
        }
        events
    }
}

/// Concrete project snapshot adapter over the protocol-only hierarchy index.
#[derive(Debug, Clone)]
pub(crate) struct ProjectHierarchy {
    pub(crate) documents: HashMap<String, StoredDocument>,
    logical: ProtocolHierarchyIndex,
}

impl ProjectHierarchy {
    pub(crate) fn from_documents(
        documents: Vec<StoredDocument>,
    ) -> Result<Self, crate::protocol::diagnostic::Diagnostic> {
        let logical = ProtocolHierarchyIndex::from_documents(documents.clone())?;
        let documents = documents
            .into_iter()
            .map(|document| (document.id().to_string(), document))
            .collect();
        Ok(Self { documents, logical })
    }

    pub(crate) fn with_replacement(&self, document: StoredDocument) -> Self {
        let mut documents = self.documents.clone();
        documents.insert(document.id().to_string(), document);
        Self::from_documents(documents.into_values().collect())
            .expect("replacement preserves indexed documents")
    }

    pub(crate) fn document(&self, id: &str) -> Option<&StoredDocument> {
        self.documents.get(id)
    }
    fn index_for(
        &self,
        document: &StoredDocument,
    ) -> Result<ProtocolHierarchyIndex, crate::protocol::diagnostic::Diagnostic> {
        if self
            .documents
            .get(document.id())
            .is_some_and(|indexed| indexed.path == document.path)
        {
            return Ok(self.logical.clone());
        }
        let mut documents = self.documents.clone();
        documents.insert(document.id().to_string(), document.clone());
        ProtocolHierarchyIndex::from_documents(documents.into_values().collect())
    }
    pub(crate) fn task_role(
        &self,
        document: &StoredDocument,
    ) -> Result<Option<TaskRole>, crate::protocol::diagnostic::Diagnostic> {
        let logical = self.index_for(document)?;
        logical.task_role(
            logical
                .document(document.id())
                .expect("fresh logical document"),
        )
    }
    pub(crate) fn relationship(
        &self,
        document: &StoredDocument,
    ) -> Result<Option<ParentRelationship>, crate::protocol::diagnostic::Diagnostic> {
        let logical = self.index_for(document)?;
        logical.relationship(
            logical
                .document(document.id())
                .expect("fresh logical document"),
        )
    }
    pub(crate) fn validate_task_hierarchy(
        &self,
        document: &StoredDocument,
    ) -> Result<TaskRole, crate::protocol::diagnostic::Diagnostic> {
        let logical = self.index_for(document)?;
        logical.validate_task_hierarchy(
            logical
                .document(document.id())
                .expect("fresh logical document"),
        )
    }
    pub(crate) fn task_hierarchy_errors(&self) -> Vec<String> {
        self.logical.task_hierarchy_errors()
    }
    pub(crate) fn validate_document_metadata(
        &self,
    ) -> Result<(), crate::protocol::diagnostic::Diagnostic> {
        self.logical.validate_document_metadata()
    }
    pub(crate) fn validate_all_task_hierarchies(
        &self,
    ) -> Result<(), crate::protocol::diagnostic::Diagnostic> {
        self.logical.validate_all_task_hierarchies()
    }
}

fn read_event_file_tolerant(path: &Path, warnings: &mut Vec<String>) -> Vec<ProjectEvent> {
    read_event_content(path, warnings)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(ProjectEvent::parse_legacy)
        .collect()
}

fn read_actor_event_file_tolerant(path: &Path, warnings: &mut Vec<String>) -> Vec<ProjectEvent> {
    let Some(actor) = path.file_stem().and_then(|stem| stem.to_str()) else {
        warnings.push(format!(
            "Events load warning: invalid actor event filename {}",
            display_path(path)
        ));
        return Vec::new();
    };
    if !events::is_safe_actor_id(actor) {
        warnings.push(format!(
            "Events load warning: unsafe actor event filename {}",
            display_path(path)
        ));
        return Vec::new();
    }
    let Some(content) = read_event_content(path, warnings) else {
        return Vec::new();
    };
    let mut events = Vec::new();
    let mut previous_seq = 0;
    for (line_number, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match ProjectEvent::parse_canonical(line, actor) {
            Ok(event) if event.seq.is_some_and(|seq| seq > previous_seq) => {
                previous_seq = event.seq.expect("checked sequence");
                events.push(event);
            }
            Ok(_) => warnings.push(format!(
                "Events load warning: non-monotonic or duplicate sequence in {} at line {}",
                display_path(path),
                line_number + 1
            )),
            Err(message) => warnings.push(format!(
                "Events load warning: malformed canonical event in {} at line {}: {message}",
                display_path(path),
                line_number + 1
            )),
        }
    }
    events
}

fn read_event_content(path: &Path, warnings: &mut Vec<String>) -> Option<String> {
    if !path.exists() {
        return None;
    }
    match fs::read_to_string(path) {
        Ok(content) => Some(content),
        Err(error) => {
            warnings.push(format!(
                "Events load warning: could not read {}: {error}",
                display_path(path)
            ));
            None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectEvent {
    pub(crate) id: String,
    pub(crate) ts: String,
    pub(crate) event: String,
    pub(crate) summary: String,
    pub(crate) actor: Option<String>,
    pub(crate) seq: Option<u64>,
}

impl ProjectEvent {
    fn parse_legacy(line: &str) -> Option<Self> {
        Some(Self {
            id: extract_json_string(line, "id")?,
            event: extract_json_string(line, "event").unwrap_or_else(|| "event".to_string()),
            ts: extract_json_string(line, "ts").unwrap_or_default(),
            summary: extract_json_string(line, "summary").unwrap_or_default(),
            actor: extract_json_string(line, "actor"),
            seq: extract_json_u64(line, "seq"),
        })
    }

    fn parse_canonical(line: &str, expected_actor: &str) -> Result<Self, &'static str> {
        let required = |key| extract_json_string(line, key).filter(|value| !value.is_empty());
        let id = required("id").ok_or("missing required id")?;
        let event = required("event").ok_or("missing required event")?;
        let ts = required("ts").ok_or("missing required ts")?;
        let summary = required("summary").ok_or("missing required summary")?;
        let actor = required("actor").ok_or("missing required actor")?;
        if actor != expected_actor {
            return Err("actor does not match filename");
        }
        let seq = extract_json_u64(line, "seq")
            .filter(|seq| *seq > 0)
            .ok_or("missing required seq")?;
        Ok(Self {
            id,
            event,
            ts,
            summary,
            actor: Some(actor),
            seq: Some(seq),
        })
    }
}

pub(crate) fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let start = top_level_value_start(line, key)?;
    parse_json_string(line, start).map(|(value, _)| value)
}

pub(crate) fn extract_json_u64(line: &str, key: &str) -> Option<u64> {
    let start = top_level_value_start(line, key)?;
    let digits = line[start..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .collect::<Vec<_>>();
    (!digits.is_empty())
        .then(|| std::str::from_utf8(&digits).ok()?.parse().ok())
        .flatten()
}

fn top_level_value_start(line: &str, wanted_key: &str) -> Option<usize> {
    let mut cursor = skip_json_whitespace(line, 0);
    if line.as_bytes().get(cursor)? != &b'{' {
        return None;
    }
    cursor += 1;
    loop {
        cursor = skip_json_whitespace(line, cursor);
        if line.as_bytes().get(cursor) == Some(&b'}') {
            return None;
        }
        let (key, after_key) = parse_json_string(line, cursor)?;
        cursor = skip_json_whitespace(line, after_key);
        if line.as_bytes().get(cursor)? != &b':' {
            return None;
        }
        let value_start = skip_json_whitespace(line, cursor + 1);
        if key == wanted_key {
            return Some(value_start);
        }
        cursor = skip_json_value(line, value_start)?;
        cursor = skip_json_whitespace(line, cursor);
        match line.as_bytes().get(cursor)? {
            b',' => cursor += 1,
            b'}' => return None,
            _ => return None,
        }
    }
}

fn parse_json_string(line: &str, start: usize) -> Option<(String, usize)> {
    if line.as_bytes().get(start)? != &b'"' {
        return None;
    }
    let mut cursor = start + 1;
    let mut value = String::new();
    let mut escaped = false;
    while let Some(&byte) = line.as_bytes().get(cursor) {
        cursor += 1;
        if escaped {
            value.push(match byte {
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'"' => '"',
                b'\\' => '\\',
                other => other as char,
            });
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some((value, cursor));
        } else {
            value.push(byte as char);
        }
    }
    None
}

fn skip_json_value(line: &str, start: usize) -> Option<usize> {
    if line.as_bytes().get(start) == Some(&b'"') {
        return parse_json_string(line, start).map(|(_, cursor)| cursor);
    }
    let mut cursor = start;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while let Some(&byte) = line.as_bytes().get(cursor) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' if depth > 0 => depth -= 1,
                b',' | b'}' if depth == 0 => return Some(cursor),
                _ => {}
            }
        }
        cursor += 1;
    }
    Some(cursor)
}

fn skip_json_whitespace(line: &str, mut cursor: usize) -> usize {
    while line
        .as_bytes()
        .get(cursor)
        .is_some_and(u8::is_ascii_whitespace)
    {
        cursor += 1;
    }
    cursor
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

    pub(crate) fn hierarchy_input(&self) -> HierarchyDocument {
        HierarchyDocument::new(self.document.clone(), display_path(&self.path))
    }
}

impl From<StoredDocument> for HierarchyDocument {
    fn from(document: StoredDocument) -> Self {
        document.hierarchy_input()
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

pub(crate) fn read_documents(
    dir: &Path,
    location: DocumentLocation,
) -> Result<Vec<StoredDocument>, CliError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
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
pub(crate) fn yaml_double_quote(value: &str) -> String {
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
    fn discovers_root_compatibility_with_resolved_data_paths() {
        let root = env::temp_dir().join(format!("tandem-project-compat-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("tandem.md"), "---\nprotocolVersion: 0.2.0\n---\n").unwrap();
        let data_dir = root.join(".tandem");
        let project = TandemProject::discover_from(&root).unwrap();
        assert_eq!(project.root(), root.as_path());
        assert_eq!(project.data_dir(), data_dir.as_path());
        assert_eq!(project.config_path, root.join("tandem.md"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn initialization_creates_actor_event_directory_not_a_new_legacy_log() {
        let root = env::temp_dir().join(format!("tandem-project-init-{}", std::process::id()));
        let project =
            TandemProject::initialize(&root, "---\nprotocolVersion: 0.2.0\n---\n").unwrap();
        assert!(project.events_dir().is_dir());
        assert!(!project.events_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn strict_reads_fail_while_tolerant_reads_warn() {
        let root = env::temp_dir().join(format!("tandem-project-read-{}", std::process::id()));
        let project = TandemProject::with_paths(
            root.clone(),
            root.join(".tandem"),
            root.join(".tandem/tandem.md"),
        );
        fs::create_dir_all(&project.board_dir).unwrap();
        fs::write(project.board_dir.join("task-1.md"), "not frontmatter").unwrap();
        assert!(project.read_board_documents().is_err());
        let mut warnings = Vec::new();
        assert!(project
            .read_board_documents_tolerant(&mut warnings)
            .is_empty());
        assert_eq!(warnings.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn aggregates_legacy_and_actor_event_logs_tolerantly() {
        let root =
            env::temp_dir().join(format!("tandem-project-events-read-{}", std::process::id()));
        let project = TandemProject::with_paths(
            root.clone(),
            root.join(".tandem"),
            root.join(".tandem/tandem.md"),
        );
        fs::create_dir_all(project.events_dir()).unwrap();
        fs::write(&project.events_path, "{\"ts\":\"old\",\"event\":\"task.created\",\"id\":\"task-1\",\"summary\":\"legacy\"}\n").unwrap();
        fs::write(project.actor_events_path("actor-1"), "{\"ts\":\"new\",\"event\":\"task.updated\",\"id\":\"task-1\",\"summary\":\"actor\",\"actor\":\"actor-1\",\"seq\":1}\n").unwrap();
        let mut warnings = Vec::new();
        let events = project.read_events_tolerant(&mut warnings);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].actor, None);
        assert_eq!(events[1].actor.as_deref(), Some("actor-1"));
        assert_eq!(events[1].seq, Some(1));
        assert!(warnings.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn warns_and_skips_corrupt_canonical_actor_events() {
        let root = env::temp_dir().join(format!(
            "tandem-project-events-corrupt-{}",
            std::process::id()
        ));
        let project = TandemProject::with_paths(
            root.clone(),
            root.join(".tandem"),
            root.join(".tandem/tandem.md"),
        );
        fs::create_dir_all(project.events_dir()).unwrap();
        fs::write(
            project.actor_events_path("actor-1"),
            "{\"ts\":\"now\",\"event\":\"task.updated\",\"id\":\"task-1\",\"summary\":\"ok\",\"actor\":\"actor-1\",\"seq\":1}\n{\"id\":\"task-1\",\"actor\":\"wrong\",\"seq\":1}\n{\"ts\":\"later\",\"event\":\"task.updated\",\"id\":\"task-1\",\"summary\":\"duplicate\",\"actor\":\"actor-1\",\"seq\":1}\n",
        )
        .unwrap();
        let mut warnings = Vec::new();
        let events = project.read_events_tolerant(&mut warnings);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].seq, Some(1));
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("malformed canonical event")));
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("non-monotonic or duplicate")));
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
