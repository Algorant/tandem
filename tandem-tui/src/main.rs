use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

const PROTOCOL_VERSION: &str = "0.1.0";

#[derive(Debug)]
struct CliError {
    message: String,
    code: i32,
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
        }
    }

    fn user(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::user(error.to_string())
    }
}

#[derive(Debug, Clone)]
struct Workspace {
    board_dir: PathBuf,
    logs_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct Document {
    path: PathBuf,
    location: DocumentLocation,
    fields: HashMap<String, String>,
    body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentLocation {
    Board,
    Logs,
}

impl DocumentLocation {
    fn as_str(self) -> &'static str {
        match self {
            DocumentLocation::Board => "board",
            DocumentLocation::Logs => "logs",
        }
    }
}

impl Document {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    fn id(&self) -> &str {
        self.field("id").unwrap_or("")
    }

    fn doc_type(&self) -> &str {
        self.field("type").unwrap_or("task")
    }

    fn title(&self) -> &str {
        self.field("title").unwrap_or("")
    }
}

#[derive(Debug, Default)]
struct ListOptions {
    state: Option<String>,
    doc_type: Option<String>,
    json: bool,
}

#[derive(Debug, Default)]
struct ShowOptions {
    id: String,
    json: bool,
}

#[derive(Debug, Default)]
struct InitOptions {
    title: Option<String>,
    force: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error.message);
        std::process::exit(error.code);
    }
}

fn run() -> Result<(), CliError> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    let command = args.remove(0);
    match command.as_str() {
        "init" => cmd_init(parse_init_args(&args)?)?,
        "list" => cmd_list(parse_list_args(&args)?)?,
        "show" => cmd_show(parse_show_args(&args)?)?,
        "tui" => cmd_tui(&args)?,
        "help" | "--help" => print_help(),
        other => {
            return Err(CliError::usage(format!(
                "unknown command `{other}`. Supported first-slice commands: init, list, show, tui"
            )))
        }
    }

    Ok(())
}

fn print_help() {
    println!("tdm - Tandem CLI");
    println!();
    println!("Usage:");
    println!("  tdm init --title <title>");
    println!("  tdm list [--state <state>] [--type <type>] [--json]");
    println!("  tdm show <id> [--json]");
    println!("  tdm tui");
}

fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
    let mut options = InitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--title" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::usage("--title requires a value"))?;
                options.title = Some(value.clone());
            }
            "--force" => options.force = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown init flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected init argument `{value}`; use --title <title>"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_list_args(args: &[String]) -> Result<ListOptions, CliError> {
    let mut options = ListOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" => {
                index += 1;
                options.state = Some(required_value(args, index, "--state")?.to_string());
            }
            "--type" => {
                index += 1;
                options.doc_type = Some(required_value(args, index, "--type")?.to_string());
            }
            "--json" => options.json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown list flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected list argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_show_args(args: &[String]) -> Result<ShowOptions, CliError> {
    let mut options = ShowOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => options.json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown show flag `{flag}`")))
            }
            value => {
                if options.id.is_empty() {
                    options.id = value.to_string();
                } else {
                    return Err(CliError::usage(format!(
                        "unexpected extra show argument `{value}`"
                    )));
                }
            }
        }
        index += 1;
    }

    if options.id.is_empty() {
        return Err(CliError::usage("show requires an <id>"));
    }

    Ok(options)
}

fn required_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn cmd_init(options: InitOptions) -> Result<(), CliError> {
    let title = options
        .title
        .as_deref()
        .ok_or_else(|| CliError::usage("init requires --title <title>"))?
        .trim();

    if title.is_empty() {
        return Err(CliError::usage("--title must not be empty"));
    }

    let root = env::current_dir()?;
    let tandem_dir = root.join(".tandem");
    let config_path = tandem_dir.join("tandem.md");

    if tandem_dir.exists() || config_path.exists() {
        let hint = if options.force {
            " --force overwrite is not implemented in this first CLI slice."
        } else {
            ""
        };
        return Err(CliError::user(format!(
            "Tandem workspace already exists at {}.{hint}",
            tandem_dir.display()
        )));
    }

    fs::create_dir_all(tandem_dir.join("board"))?;
    fs::create_dir_all(tandem_dir.join("logs"))?;

    let workspace_type = "workspace";
    let config = format!(
        "---\nprotocolVersion: {PROTOCOL_VERSION}\ntype: {workspace_type}\ntitle: {}\nstates:\n  - id: todo\n    title: To Do\n  - id: in-progress\n    title: In Progress\n  - id: review\n    title: Review\nrules:\n  always: []\n  never: []\n  prefer: []\n  context: []\n---\n\n# {}\n",
        yaml_double_quote(title),
        title
    );
    fs::write(&config_path, config)?;
    File::create(tandem_dir.join("events.jsonl"))?;

    println!("Created Tandem workspace");
    println!("Title: {title}");
    println!("Config: {}", display_path(&config_path));
    println!("Board:  {}", display_path(&tandem_dir.join("board")));
    println!("Logs:   {}", display_path(&tandem_dir.join("logs")));
    println!("Events: {}", display_path(&tandem_dir.join("events.jsonl")));
    println!("States: todo, in-progress, review");

    Ok(())
}

fn cmd_list(options: ListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?;
    let mut filtered = docs
        .into_iter()
        .filter(|doc| {
            options
                .state
                .as_deref()
                .map_or(true, |state| doc.field("state") == Some(state))
        })
        .filter(|doc| {
            options
                .doc_type
                .as_deref()
                .map_or(true, |doc_type| doc.doc_type() == doc_type)
        })
        .collect::<Vec<_>>();

    filtered.sort_by(|a, b| {
        a.field("state")
            .unwrap_or("")
            .cmp(b.field("state").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });

    if options.json {
        println!("{}", list_json(&filtered));
    } else {
        print_list_table(&filtered);
    }

    Ok(())
}

fn cmd_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let doc = find_document(&workspace, &options.id)?
        .ok_or_else(|| CliError::user(format!("document not found: {}", options.id)))?;

    if options.json {
        println!("{}", show_json(&doc));
    } else {
        print_show(&doc);
    }

    Ok(())
}

fn cmd_tui(args: &[String]) -> Result<(), CliError> {
    if !args.is_empty() {
        return Err(CliError::usage(
            "tdm tui does not accept options in this first slice",
        ));
    }

    println!("tdm tui is planned but not implemented in this first CLI slice.");
    Ok(())
}

fn discover_workspace() -> Result<Workspace, CliError> {
    let mut dir = env::current_dir()?;

    loop {
        let tandem_dir = dir.join(".tandem");
        let config_path = tandem_dir.join("tandem.md");
        if config_path.is_file() {
            return Ok(Workspace {
                board_dir: tandem_dir.join("board"),
                logs_dir: tandem_dir.join("logs"),
            });
        }

        if dir.join(".git").exists() {
            break;
        }

        if !dir.pop() {
            break;
        }
    }

    Err(CliError::user(
        "No Tandem workspace found. Run `tdm init --title <title>` first.",
    ))
}

fn read_documents(dir: &Path, location: DocumentLocation) -> Result<Vec<Document>, CliError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut docs = Vec::new();
    for path in paths {
        docs.push(read_document(&path, location)?);
    }
    Ok(docs)
}

fn read_document(path: &Path, location: DocumentLocation) -> Result<Document, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::user(format!("failed to read {}: {error}", display_path(path)))
    })?;
    let (frontmatter, body) = split_frontmatter(&content).map_err(|message| {
        CliError::user(format!("failed to parse {}: {message}", display_path(path)))
    })?;
    let fields = parse_simple_frontmatter(&frontmatter);

    Ok(Document {
        path: path.to_path_buf(),
        location,
        fields,
        body,
    })
}

fn find_document(workspace: &Workspace, id: &str) -> Result<Option<Document>, CliError> {
    for location in [DocumentLocation::Board, DocumentLocation::Logs] {
        let dir = match location {
            DocumentLocation::Board => &workspace.board_dir,
            DocumentLocation::Logs => &workspace.logs_dir,
        };
        for doc in read_documents(dir, location)? {
            if doc.id() == id {
                return Ok(Some(doc));
            }
        }
    }
    Ok(None)
}

fn split_frontmatter(content: &str) -> Result<(String, String), &'static str> {
    let first_line_end = content.find('\n').ok_or("missing frontmatter delimiter")?;
    let first_line = content[..first_line_end].trim_end_matches('\r');
    if first_line != "---" {
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
        let line = content[line_start..line_end].trim_end_matches('\r');
        if line.trim() == "---" {
            let body_start = line_end + 1;
            return Ok((
                content[frontmatter_start..line_start].to_string(),
                content[body_start..].to_string(),
            ));
        }
        cursor = line_end + 1;
    }

    Err("missing closing frontmatter delimiter")
}

fn parse_simple_frontmatter(frontmatter: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('-')
            || line.starts_with(' ')
            || line.starts_with('\t')
        {
            continue;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = parse_scalar_value(value.trim());
        if !key.is_empty() && !value.is_empty() {
            fields.insert(key.to_string(), value);
        }
    }

    fields
}

fn parse_scalar_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let without_comment = if value.starts_with('"') || value.starts_with('\'') {
        value
    } else {
        value.split(" #").next().unwrap_or(value).trim_end()
    };

    if let Some(stripped) = without_comment
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return unescape_double_quoted(stripped);
    }

    if let Some(stripped) = without_comment
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return stripped.replace("''", "'");
    }

    without_comment.trim().to_string()
}

fn unescape_double_quoted(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('t') => output.push('\t'),
                Some('"') => output.push('"'),
                Some('\\') => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn print_list_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No active Tandem documents found.");
        return;
    }

    println!(
        "{:<12} {:<12} {:<8} {:<42} {:<12}",
        "ID", "STATE", "TYPE", "TITLE", "ASSIGNEE"
    );
    for doc in docs {
        println!(
            "{:<12} {:<12} {:<8} {:<42} {:<12}",
            truncate(doc.id(), 12),
            truncate(doc.field("state").unwrap_or("-"), 12),
            truncate(doc.doc_type(), 8),
            truncate(doc.title(), 42),
            truncate(doc.field("assignee").unwrap_or("-"), 12)
        );
    }
}

fn print_show(doc: &Document) {
    println!("ID:        {}", doc.id());
    println!("Type:      {}", doc.doc_type());
    println!("Title:     {}", doc.title());
    if let Some(state) = doc.field("state") {
        println!("State:     {state}");
    }
    if let Some(priority) = doc.field("priority") {
        println!("Priority:  {priority}");
    }
    if let Some(assignee) = doc.field("assignee") {
        println!("Assignee:  {assignee}");
    }
    if let Some(created_at) = doc.field("createdAt") {
        println!("Created:   {created_at}");
    }
    if let Some(updated_at) = doc.field("updatedAt") {
        println!("Updated:   {updated_at}");
    }
    if let Some(completed_at) = doc.field("completedAt") {
        println!("Completed: {completed_at}");
    }
    println!("Location:  {}", doc.location.as_str());
    println!("Path:      {}", display_path(&doc.path));
    println!();
    println!("Body:");
    if doc.body.trim().is_empty() {
        println!("(empty)");
    } else {
        print!("{}", doc.body);
        if !doc.body.ends_with('\n') {
            println!();
        }
    }
}

fn list_json(docs: &[Document]) -> String {
    let mut by_state = BTreeMap::<String, usize>::new();
    for doc in docs {
        let state = doc.field("state").unwrap_or("unknown").to_string();
        *by_state.entry(state).or_insert(0) += 1;
    }

    let items = docs.iter().map(document_summary_json).collect::<Vec<_>>();
    let states = by_state
        .iter()
        .map(|(state, count)| format!("{}:{count}", json_string(state)))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"counts\":{{\"total\":{},\"byState\":{{{}}}}}}},\"warnings\":[]}}",
        items.join(","),
        docs.len(),
        states
    )
}

fn show_json(doc: &Document) -> String {
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{},\"body\":{},\"path\":{},\"location\":{}}},\"warnings\":[]}}",
        document_detail_json(doc),
        json_string(&doc.body),
        json_string(&display_path(&doc.path)),
        json_string(doc.location.as_str())
    )
}

fn document_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "priority", doc.field("priority"));
    push_optional_json_field(&mut fields, "assignee", doc.field("assignee"));
    format!("{{{}}}", fields.join(","))
}

fn document_detail_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    for key in [
        "state",
        "priority",
        "assignee",
        "createdAt",
        "updatedAt",
        "completedAt",
    ] {
        push_optional_json_field(&mut fields, key, doc.field(key));
    }
    format!("{{{}}}", fields.join(","))
}

fn push_json_field(fields: &mut Vec<String>, key: &str, value: &str) {
    fields.push(format!("{}:{}", json_string(key), json_string(value)));
}

fn push_optional_json_field(fields: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_json_field(fields, key, value);
    }
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

fn yaml_double_quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn truncate(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return value.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut truncated = chars[..max_chars.saturating_sub(1)]
        .iter()
        .collect::<String>();
    truncated.push('…');
    truncated
}

fn display_path(path: &Path) -> String {
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
    fn parses_simple_frontmatter_and_body() {
        let input = "---\nid: task-1\ntitle: \"Hello\"\nstate: todo\n---\n\nBody\n";
        let (frontmatter, body) = split_frontmatter(input).unwrap();
        let fields = parse_simple_frontmatter(&frontmatter);
        assert_eq!(fields.get("id").map(String::as_str), Some("task-1"));
        assert_eq!(fields.get("title").map(String::as_str), Some("Hello"));
        assert_eq!(fields.get("state").map(String::as_str), Some("todo"));
        assert_eq!(body, "\nBody\n");
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }
}
