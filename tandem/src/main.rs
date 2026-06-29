use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use yaml_rust2::{Yaml, YamlLoader};

mod tui;

const PROTOCOL_VERSION: &str = "0.1.0";
const DEFAULT_STATES: &[&str] = &["todo", "in-progress", "validation"];
const LEGACY_REVIEW_STATE: &str = "review";
const VALIDATION_STATE: &str = "validation";
const ACCORD_STATUSES: &[&str] = &[
    "ready",
    "claimed",
    "delivered",
    "accepted",
    "rework",
    "failed",
    "blocked",
];
const REVIEW_STATUSES: &[&str] = &[
    "not-ready",
    "pending",
    "accepted",
    "changes-requested",
    "rejected",
];
const DEFAULT_WORKSPACE_TITLE: &str = "Tandem Workspace";

// Exit code categories: 0 success, 1 runtime/data/write failure, 2 usage/argument failure.
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
    config_path: PathBuf,
    events_path: PathBuf,
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
struct InitOptions {
    title: Option<String>,
    force: bool,
}

#[derive(Debug, Default)]
struct ListOptions {
    state: Option<String>,
    doc_type: Option<String>,
    priority: Option<String>,
    tag: Option<String>,
    assignee: Option<String>,
    accord: Option<String>,
    review: Option<String>,
    json: bool,
}

#[derive(Debug, Default)]
struct ShowOptions {
    id: String,
    json: bool,
}

#[derive(Debug, Default)]
struct AddOptions {
    title: Option<String>,
    state: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    assignee: Option<String>,
    due_date: Option<String>,
    parent: Option<String>,
    blockers: Vec<String>,
    references: Vec<String>,
    related_files: Vec<String>,
    subtasks: Vec<String>,
}

#[derive(Debug, Default)]
struct MoveOptions {
    id: String,
    state: Option<String>,
}

#[derive(Debug, Default)]
struct CompleteOptions {
    id: String,
    summary: Option<String>,
    files_changed: Vec<String>,
    validation: Option<String>,
    reviewer: Option<String>,
}

#[derive(Debug, Default)]
struct SearchOptions {
    query: String,
    state: Option<String>,
    doc_type: Option<String>,
    json: bool,
}

#[derive(Debug, Default)]
struct LogListOptions {
    limit: Option<usize>,
    json: bool,
}

#[derive(Debug, Default)]
struct CategoryListOptions {
    category: Option<String>,
    json: bool,
}

#[derive(Debug, Default)]
struct RuleAddOptions {
    category: Option<String>,
    rule: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Default)]
struct RuleEditOptions {
    category: Option<String>,
    id: Option<usize>,
    rule: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Default)]
struct RuleDeleteOptions {
    category: Option<String>,
    id: Option<usize>,
}

#[derive(Debug, Default)]
struct AccordOptions {
    id: String,
    assignee: Option<String>,
    summary: Option<String>,
    reviewer: Option<String>,
    note: Option<String>,
    reason: Option<String>,
    deliverables: Vec<String>,
    validations: Vec<String>,
    constraints: Vec<String>,
    evidence: Vec<String>,
    files_changed: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct AccordRecord {
    status: String,
    assignee: Option<String>,
    claimed_at: Option<String>,
    delivered_at: Option<String>,
    deliverables: Vec<String>,
    validations: Vec<String>,
    constraints: Vec<String>,
    summary: Option<String>,
    evidence: Vec<String>,
    files_changed: Vec<String>,
    reviewer: Option<String>,
    note: Option<String>,
    reason: Option<String>,
    updated_at: String,
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
        "add" => cmd_add(parse_add_args(&args)?)?,
        "move" => cmd_move(parse_move_args(&args)?)?,
        "complete" => cmd_complete(parse_complete_args(&args)?)?,
        "search" => cmd_search(parse_search_args(&args)?)?,
        "log" => cmd_log(&args)?,
        "accord" => cmd_accord(&args)?,
        "rules" => cmd_rules(&args)?,
        "decision" => cmd_decision(&args)?,
        "tui" => cmd_tui(&args)?,
        "help" | "--help" => print_help(),
        other => {
            return Err(CliError::usage(format!(
                "unknown command `{other}`. Supported commands: init, list, show, add, move, complete, search, log, accord, rules, decision, tui"
            )))
        }
    }

    Ok(())
}

fn print_help() {
    println!("tandem - Tandem CLI");
    println!();
    println!("Usage:");
    println!("  tandem init [--title <title>]");
    println!("  tandem list [--state <state>] [--type <type>] [--json]");
    println!("  tandem show <id> [--json]");
    println!("  tandem add --title <title> [--state <state>] [--description <text>]");
    println!("  tandem move <id> --state <state>");
    println!("  tandem complete <id> --summary <text>");
    println!("  tandem search <query> [--state <state>] [--type <type>] [--json]");
    println!("  tandem log list|show|search ...");
    println!("  tandem accord ready|claim|deliver|accept|rework|block|fail ...");
    println!("  tandem rules list|add|edit|delete ...");
    println!("  tandem decision list|show|add ...");
    println!("  tandem tui");
}

fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
    let mut options = InitOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--title" => {
                index += 1;
                options.title = Some(required_value(args, index, "--title")?.to_string());
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
            "--priority" => {
                index += 1;
                options.priority = Some(required_value(args, index, "--priority")?.to_string());
            }
            "--tag" => {
                index += 1;
                options.tag = Some(required_value(args, index, "--tag")?.to_string());
            }
            "--assignee" => {
                index += 1;
                options.assignee = Some(required_value(args, index, "--assignee")?.to_string());
            }
            "--accord" => {
                index += 1;
                options.accord = Some(required_value(args, index, "--accord")?.to_string());
            }
            "--review" => {
                index += 1;
                options.review = Some(required_value(args, index, "--review")?.to_string());
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
            value => set_single_positional(&mut options.id, value, "show")?,
        }
        index += 1;
    }

    if options.id.is_empty() {
        return Err(CliError::usage("show requires an <id>"));
    }

    Ok(options)
}

fn parse_add_args(args: &[String]) -> Result<AddOptions, CliError> {
    let mut options = AddOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--title" => {
                index += 1;
                options.title = Some(required_value(args, index, "--title")?.to_string());
            }
            "--state" => {
                index += 1;
                options.state = Some(required_value(args, index, "--state")?.to_string());
            }
            "--description" => {
                index += 1;
                options.description =
                    Some(required_value(args, index, "--description")?.to_string());
            }
            "--priority" => {
                index += 1;
                options.priority = Some(required_value(args, index, "--priority")?.to_string());
            }
            "--tag" => {
                index += 1;
                options
                    .tags
                    .push(required_value(args, index, "--tag")?.to_string());
            }
            "--assignee" => {
                index += 1;
                options.assignee = Some(required_value(args, index, "--assignee")?.to_string());
            }
            "--due-date" => {
                index += 1;
                options.due_date = Some(required_value(args, index, "--due-date")?.to_string());
            }
            "--parent" => {
                index += 1;
                options.parent = Some(required_value(args, index, "--parent")?.to_string());
            }
            "--blocker" => {
                index += 1;
                options
                    .blockers
                    .push(required_value(args, index, "--blocker")?.to_string());
            }
            "--reference" => {
                index += 1;
                options
                    .references
                    .push(required_value(args, index, "--reference")?.to_string());
            }
            "--related-file" => {
                index += 1;
                options
                    .related_files
                    .push(required_value(args, index, "--related-file")?.to_string());
            }
            "--subtask" => {
                index += 1;
                options
                    .subtasks
                    .push(required_value(args, index, "--subtask")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown add flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected add argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_move_args(args: &[String]) -> Result<MoveOptions, CliError> {
    let mut options = MoveOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" => {
                index += 1;
                options.state = Some(required_value(args, index, "--state")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown move flag `{flag}`")))
            }
            value => set_single_positional(&mut options.id, value, "move")?,
        }
        index += 1;
    }
    if options.id.is_empty() {
        return Err(CliError::usage("move requires an <id>"));
    }
    Ok(options)
}

fn parse_complete_args(args: &[String]) -> Result<CompleteOptions, CliError> {
    let mut options = CompleteOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--summary" => {
                index += 1;
                options.summary = Some(required_value(args, index, "--summary")?.to_string());
            }
            "--file-changed" => {
                index += 1;
                options
                    .files_changed
                    .push(required_value(args, index, "--file-changed")?.to_string());
            }
            "--validation" => {
                index += 1;
                options.validation = Some(required_value(args, index, "--validation")?.to_string());
            }
            "--reviewer" => {
                index += 1;
                options.reviewer = Some(required_value(args, index, "--reviewer")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown complete flag `{flag}`")))
            }
            value => set_single_positional(&mut options.id, value, "complete")?,
        }
        index += 1;
    }
    if options.id.is_empty() {
        return Err(CliError::usage("complete requires an <id>"));
    }
    Ok(options)
}

fn parse_search_args(args: &[String]) -> Result<SearchOptions, CliError> {
    let mut options = SearchOptions::default();
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
                return Err(CliError::usage(format!("unknown search flag `{flag}`")))
            }
            value => set_single_positional(&mut options.query, value, "search")?,
        }
        index += 1;
    }
    if options.query.is_empty() {
        return Err(CliError::usage("search requires a <query>"));
    }
    Ok(options)
}

fn parse_log_search_args(args: &[String]) -> Result<SearchOptions, CliError> {
    let mut options = SearchOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => options.json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown log search flag `{flag}`")))
            }
            value => set_single_positional(&mut options.query, value, "log search")?,
        }
        index += 1;
    }
    if options.query.is_empty() {
        return Err(CliError::usage("log search requires a <query>"));
    }
    Ok(options)
}

fn parse_log_list_args(args: &[String]) -> Result<LogListOptions, CliError> {
    let mut options = LogListOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--limit" => {
                index += 1;
                let value = required_value(args, index, "--limit")?;
                options.limit = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| CliError::usage("--limit must be a positive integer"))?,
                );
            }
            "--json" => options.json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown log list flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected log list argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_category_list_args(
    args: &[String],
    command: &str,
) -> Result<CategoryListOptions, CliError> {
    let mut options = CategoryListOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--category" => {
                index += 1;
                options.category = Some(required_value(args, index, "--category")?.to_string());
            }
            "--json" => options.json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown {command} flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected {command} argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_rule_add_args(args: &[String]) -> Result<RuleAddOptions, CliError> {
    let mut options = RuleAddOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--category" => {
                index += 1;
                options.category = Some(required_value(args, index, "--category")?.to_string());
            }
            "--rule" => {
                index += 1;
                options.rule = Some(required_value(args, index, "--rule")?.to_string());
            }
            "--source" => {
                index += 1;
                options.source = Some(required_value(args, index, "--source")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown rules add flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected rules add argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_rule_edit_args(args: &[String]) -> Result<RuleEditOptions, CliError> {
    let mut options = RuleEditOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--category" => {
                index += 1;
                options.category = Some(required_value(args, index, "--category")?.to_string());
            }
            "--id" => {
                index += 1;
                let value = required_value(args, index, "--id")?;
                options.id = Some(parse_rule_id(value)?);
            }
            "--rule" => {
                index += 1;
                options.rule = Some(required_value(args, index, "--rule")?.to_string());
            }
            "--source" => {
                index += 1;
                options.source = Some(required_value(args, index, "--source")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown rules edit flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected rules edit argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_rule_delete_args(args: &[String]) -> Result<RuleDeleteOptions, CliError> {
    let mut options = RuleDeleteOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--category" => {
                index += 1;
                options.category = Some(required_value(args, index, "--category")?.to_string());
            }
            "--id" => {
                index += 1;
                let value = required_value(args, index, "--id")?;
                options.id = Some(parse_rule_id(value)?);
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown rules delete flag `{flag}`"
                )))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected rules delete argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_rule_id(value: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| CliError::usage("--id must be a positive integer"))
}

fn parse_accord_args(action: &str, args: &[String]) -> Result<AccordOptions, CliError> {
    let mut options = AccordOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--assignee" => {
                index += 1;
                options.assignee = Some(required_value(args, index, "--assignee")?.to_string());
            }
            "--summary" => {
                index += 1;
                options.summary = Some(required_value(args, index, "--summary")?.to_string());
            }
            "--reviewer" => {
                index += 1;
                options.reviewer = Some(required_value(args, index, "--reviewer")?.to_string());
            }
            "--note" => {
                index += 1;
                options.note = Some(required_value(args, index, "--note")?.to_string());
            }
            "--reason" => {
                index += 1;
                options.reason = Some(required_value(args, index, "--reason")?.to_string());
            }
            "--deliverable" => {
                index += 1;
                options
                    .deliverables
                    .push(required_value(args, index, "--deliverable")?.to_string());
            }
            "--validation" => {
                index += 1;
                options
                    .validations
                    .push(required_value(args, index, "--validation")?.to_string());
            }
            "--constraint" => {
                index += 1;
                options
                    .constraints
                    .push(required_value(args, index, "--constraint")?.to_string());
            }
            "--evidence" => {
                index += 1;
                options
                    .evidence
                    .push(required_value(args, index, "--evidence")?.to_string());
            }
            "--file-changed" => {
                index += 1;
                options
                    .files_changed
                    .push(required_value(args, index, "--file-changed")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown accord {action} flag `{flag}`"
                )))
            }
            value => set_single_positional(&mut options.id, value, &format!("accord {action}"))?,
        }
        index += 1;
    }
    if options.id.is_empty() {
        return Err(CliError::usage(format!("accord {action} requires an <id>")));
    }
    Ok(options)
}

fn parse_json_only_args(args: &[String], command: &str) -> Result<bool, CliError> {
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown {command} flag `{flag}`")))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected {command} argument `{value}`"
                )))
            }
        }
    }
    Ok(json)
}

fn required_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn set_single_positional(target: &mut String, value: &str, command: &str) -> Result<(), CliError> {
    if target.is_empty() {
        *target = value.to_string();
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "unexpected extra {command} argument `{value}`"
        )))
    }
}

fn cmd_init(options: InitOptions) -> Result<(), CliError> {
    let root = env::current_dir()?;
    let title = match options.title.as_deref() {
        Some(title) => {
            let title = title.trim();
            if title.is_empty() {
                return Err(CliError::usage("--title must not be empty"));
            }
            title.to_string()
        }
        None => derive_workspace_title(&root),
    };
    let tandem_dir = root.join(".tandem");
    let config_path = tandem_dir.join("tandem.md");

    if tandem_dir.exists() || config_path.exists() {
        let hint = if options.force {
            " --force overwrite is not implemented yet."
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
        "---\nprotocolVersion: {PROTOCOL_VERSION}\ntype: {workspace_type}\ntitle: {}\nstates:\n  - id: todo\n    title: To Do\n  - id: in-progress\n    title: In Progress\n  - id: validation\n    title: Validation\nrules:\n  always: []\n  never: []\n  prefer: []\n  context: []\n---\n\n# {}\n",
        yaml_double_quote(&title),
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
    println!("States: todo, in-progress, validation");

    Ok(())
}

fn derive_workspace_title(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_WORKSPACE_TITLE.to_string())
}

fn cmd_list(options: ListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?;
    let mut filtered = filter_documents(docs, &options);
    sort_documents(&mut filtered);

    if options.json {
        println!("{}", list_json(&filtered));
    } else {
        print_list_table(&filtered);
        print_document_warnings(&filtered);
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

fn cmd_add(options: AddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let title = require_nonempty(options.title.as_deref(), "add requires --title <title>")?;
    let state = options.state.as_deref().unwrap_or("todo");
    validate_state(&workspace, state)?;

    if let Some(parent) = options.parent.as_deref() {
        require_existing_document(&workspace, parent, "parent")?;
    }
    for blocker in &options.blockers {
        require_existing_document(&workspace, blocker, "blocker")?;
    }

    let mut warnings = Vec::new();
    for reference in &options.references {
        if !document_exists(&workspace, reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }

    let task_id = next_sequential_id(&workspace, "task")?;
    let now = current_timestamp();
    let task_path = workspace.board_dir.join(format!("{task_id}.md"));
    let mut lines = vec![
        "---".to_string(),
        format!("id: {task_id}"),
        "type: task".to_string(),
        format!("title: {}", yaml_double_quote(title)),
        format!("state: {state}"),
    ];
    push_optional_line(&mut lines, "priority", options.priority.as_deref());
    push_optional_line(&mut lines, "assignee", options.assignee.as_deref());
    push_optional_line(&mut lines, "dueDate", options.due_date.as_deref());
    push_optional_line(&mut lines, "parentId", options.parent.as_deref());
    push_array_line(&mut lines, "blockers", &options.blockers);
    push_array_line(&mut lines, "references", &options.references);
    push_array_line(&mut lines, "relatedFiles", &options.related_files);
    push_array_line(&mut lines, "tags", &options.tags);
    lines.push(format!("createdAt: {}", yaml_double_quote(&now)));
    lines.push(format!("updatedAt: {}", yaml_double_quote(&now)));
    if !options.subtasks.is_empty() {
        lines.push("subtasks:".to_string());
        for (index, subtask) in options.subtasks.iter().enumerate() {
            let subtask_id = format!("{task_id}-{}", index + 1);
            lines.push(format!("  - id: {subtask_id}"));
            lines.push(format!("    title: {}", yaml_double_quote(subtask)));
            lines.push("    completed: false".to_string());
        }
    }
    lines.push("---".to_string());
    lines.push(String::new());
    if let Some(description) = options.description.as_deref() {
        lines.push("## Description".to_string());
        lines.push(String::new());
        lines.push(description.to_string());
    }
    lines.push(String::new());
    write_atomic(&task_path, &lines.join("\n"))?;
    append_event(&workspace, "task.created", &task_id, title)?;

    for warning in warnings {
        println!("Warning: {warning}");
    }
    println!("Created task");
    println!("ID:    {task_id}");
    println!("State: {state}");
    println!("Title: {title}");
    println!("Path:  {}", display_path(&task_path));
    Ok(())
}

fn cmd_move(options: MoveOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let state = options
        .state
        .as_deref()
        .ok_or_else(|| CliError::usage("move requires --state <state>"))?;
    let outcome = move_task_to_state(&workspace, &options.id, state)?;

    if !outcome.changed {
        println!("{} is already in state {state}", outcome.id);
        return Ok(());
    }

    println!("Moved {}", outcome.id);
    println!("From: {}", outcome.from);
    println!("To:   {}", outcome.to);
    if let Some(sync) = outcome.accord_sync.as_deref() {
        println!("Accord: {sync}");
    }
    println!("Path: {}", display_path(&outcome.path));
    Ok(())
}

fn cmd_complete(options: CompleteOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let summary = require_nonempty(
        options.summary.as_deref(),
        "complete requires --summary <text>",
    )?;
    let doc = find_board_document(&workspace, &options.id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be completed in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_for_mutation(&workspace, &doc)?;
    let unresolved = unresolved_blockers(&workspace, doc.field("blockers"))?;
    if !unresolved.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: {} has unresolved blockers: {}",
            doc.id(),
            unresolved.join(", ")
        )));
    }

    let review_status = review_status(&doc).unwrap_or("missing");
    let accord_status = accord_status(&doc).unwrap_or("missing");
    if review_status != "accepted" {
        println!("Warning: {} has review.status={review_status}.", doc.id());
    }
    if accord_status != "accepted" {
        println!(
            "Warning: {} has accord.status={accord_status}, not accepted.",
            doc.id()
        );
    }
    if review_status != "accepted" || accord_status != "accepted" {
        println!("Completing anyway in v0.");
        println!();
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("completedAt".to_string(), now.clone());
    updates.insert("updatedAt".to_string(), now);
    let patched = patch_frontmatter_content(
        &content,
        &updates,
        &[
            "state",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let patched = patch_completion_content(
        &patched,
        summary,
        &options.files_changed,
        options.validation.as_deref(),
        options.reviewer.as_deref(),
    )?;
    let log_path = workspace.logs_dir.join(file_name_for_path(&doc.path)?);
    if log_path.exists() {
        return Err(CliError::user(format!(
            "Validation failed: log document already exists: {}",
            display_path(&log_path)
        )));
    }
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&log_path, &patched)?;
    fs::remove_file(&doc.path).map_err(|error| {
        CliError::user(format!(
            "Write failure: could not remove active document {} after writing log {}: {error}",
            display_path(&doc.path),
            display_path(&log_path)
        ))
    })?;
    append_event(&workspace, "task.completed", doc.id(), summary)?;

    println!("Completed {}", doc.id());
    println!(
        "Moved: {} -> {}",
        display_path(&doc.path),
        display_path(&log_path)
    );
    println!("Event: task.completed");
    Ok(())
}

fn cmd_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?;
    docs.extend(read_documents(&workspace.logs_dir, DocumentLocation::Logs)?);
    let mut results = docs
        .into_iter()
        .filter(|doc| {
            options
                .doc_type
                .as_deref()
                .map_or(true, |doc_type| doc.doc_type() == doc_type)
        })
        .filter(|doc| {
            if doc.location == DocumentLocation::Logs {
                options.state.is_none()
            } else {
                options.state.as_deref().map_or(true, |state| {
                    state_matches_filter(doc.field("state"), state)
                })
            }
        })
        .filter_map(|doc| search_match(doc, &options.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));

    if options.json {
        println!("{}", search_json(&options.query, &results));
    } else {
        print_search_table(&results);
    }
    Ok(())
}

fn cmd_log(args: &[String]) -> Result<(), CliError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CliError::usage("tandem log requires list, show, or search"));
    };
    match subcommand.as_str() {
        "list" => cmd_log_list(parse_log_list_args(rest)?),
        "show" => cmd_log_show(parse_show_args(rest)?),
        "search" => cmd_log_search(parse_log_search_args(rest)?),
        other => Err(CliError::usage(format!(
            "unknown log subcommand `{other}`; use list, show, or search"
        ))),
    }
}

fn cmd_log_list(options: LogListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut docs = read_documents(&workspace.logs_dir, DocumentLocation::Logs)?;
    docs.sort_by(|a, b| {
        b.field("completedAt")
            .unwrap_or("")
            .cmp(a.field("completedAt").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
    if let Some(limit) = options.limit {
        docs.truncate(limit);
    }

    if options.json {
        println!("{}", log_list_json(&docs));
    } else {
        print_log_table(&docs);
    }
    Ok(())
}

fn cmd_log_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let doc = find_log_document(&workspace, &options.id)?
        .ok_or_else(|| CliError::user(format!("log document not found: {}", options.id)))?;
    if options.json {
        println!("{}", log_show_json(&doc));
    } else {
        print_log_show(&doc);
    }
    Ok(())
}

fn cmd_log_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut results = read_documents(&workspace.logs_dir, DocumentLocation::Logs)?
        .into_iter()
        .filter_map(|doc| search_match(doc, &options.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));
    if options.json {
        println!("{}", search_json(&options.query, &results));
    } else {
        print_search_table(&results);
    }
    Ok(())
}

fn cmd_accord(args: &[String]) -> Result<(), CliError> {
    let Some((action, rest)) = args.split_first() else {
        return Err(CliError::usage(
            "tandem accord requires ready, claim, deliver, accept, rework, block, or fail",
        ));
    };
    let status = match action.as_str() {
        "ready" => "ready",
        "claim" => "claimed",
        "deliver" => "delivered",
        "accept" => "accepted",
        "rework" => "rework",
        "block" => "blocked",
        "fail" => "failed",
        other => {
            return Err(CliError::usage(format!(
                "unknown accord subcommand `{other}`; use ready, claim, deliver, accept, rework, block, or fail"
            )))
        }
    };
    let options = parse_accord_args(action, rest)?;
    cmd_accord_update(action, status, options)
}

fn cmd_accord_update(action: &str, status: &str, options: AccordOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let doc = find_board_document(&workspace, &options.id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can have accord actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_for_mutation(&workspace, &doc)?;

    validate_accord_inputs(action, &options)?;
    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    validate_accord_transition(action, &previous_status)?;

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    apply_accord_action(&mut accord, action, status, &options);
    let patched = patch_accord_content(&content, &accord)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now);
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    let synced_state = accord_state_sync_target(status, &previous_state);
    if let Some(state) = synced_state {
        validate_state(&workspace, state)?;
        updates.insert("state".to_string(), state.to_string());
    }
    let patched = patch_frontmatter_content(&patched, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    let event_name = accord_event_name(action);
    append_event(
        &workspace,
        event_name,
        doc.id(),
        &format!("Accord {action} for {}", doc.id()),
    )?;

    print_accord_update(doc.id(), &previous_status, status, event_name, &doc.path);
    if let Some(state) = synced_state {
        println!("State:  {previous_state} -> {state}");
    }
    Ok(())
}

fn cmd_rules(args: &[String]) -> Result<(), CliError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CliError::usage(
            "tandem rules requires list, add, edit, or delete",
        ));
    };
    match subcommand.as_str() {
        "list" => cmd_rules_list(parse_category_list_args(rest, "rules list")?),
        "add" => cmd_rules_add(parse_rule_add_args(rest)?),
        "edit" => cmd_rules_edit(parse_rule_edit_args(rest)?),
        "delete" => cmd_rules_delete(parse_rule_delete_args(rest)?),
        other => Err(CliError::usage(format!(
            "unknown rules subcommand `{other}`; use list, add, edit, or delete"
        ))),
    }
}

fn cmd_rules_list(options: CategoryListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    if let Some(category) = options.category.as_deref() {
        validate_rule_category(category)?;
    }
    let rules = read_rules(&workspace.config_path)?;
    if options.json {
        println!("{}", rules_json(&rules, options.category.as_deref()));
    } else {
        print_rules(&rules, options.category.as_deref());
    }
    Ok(())
}

fn cmd_rules_add(options: RuleAddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let rule = require_nonempty(options.rule.as_deref(), "rules add requires --rule <text>")?;
    warn_missing_rule_source(&workspace, options.source.as_deref())?;

    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let next_id = rules
        .get(category)
        .into_iter()
        .flatten()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    rules
        .entry(category.to_string())
        .or_default()
        .push(RuleItem {
            id: next_id,
            rule: rule.to_string(),
            source: options.source.filter(|source| !source.trim().is_empty()),
        });
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        &workspace,
        "rules.updated",
        "rules",
        &format!("Added rule {next_id} to {category}"),
    )?;

    println!("Added rule");
    println!("Category: {category}");
    println!("ID:       {next_id}");
    println!("Rule:     {rule}");
    Ok(())
}

fn cmd_rules_edit(options: RuleEditOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let id = options
        .id
        .ok_or_else(|| CliError::usage("rules edit requires --id <rule-id>"))?;
    let rule = require_nonempty(options.rule.as_deref(), "rules edit requires --rule <text>")?;
    warn_missing_rule_source(&workspace, options.source.as_deref())?;

    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let items = rules.entry(category.to_string()).or_default();
    let item = items
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| CliError::user(format!("rule not found: {category} #{id}")))?;
    item.rule = rule.to_string();
    if let Some(source) = options.source {
        item.source = (!source.trim().is_empty()).then_some(source);
    }
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        &workspace,
        "rules.updated",
        "rules",
        &format!("Edited rule {id} in {category}"),
    )?;

    println!("Edited rule");
    println!("Category: {category}");
    println!("ID:       {id}");
    println!("Rule:     {rule}");
    Ok(())
}

fn cmd_rules_delete(options: RuleDeleteOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let id = options
        .id
        .ok_or_else(|| CliError::usage("rules delete requires --id <rule-id>"))?;

    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let items = rules.entry(category.to_string()).or_default();
    let before_len = items.len();
    items.retain(|item| item.id != id);
    if items.len() == before_len {
        return Err(CliError::user(format!("rule not found: {category} #{id}")));
    }
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        &workspace,
        "rules.updated",
        "rules",
        &format!("Deleted rule {id} from {category}"),
    )?;

    println!("Deleted rule");
    println!("Category: {category}");
    println!("ID:       {id}");
    Ok(())
}

fn cmd_decision(args: &[String]) -> Result<(), CliError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CliError::usage(
            "tandem decision requires list, show, or add",
        ));
    };
    match subcommand.as_str() {
        "list" => cmd_decision_list(parse_json_only_args(rest, "decision list")?),
        "show" => cmd_decision_show(parse_show_args(rest)?),
        "add" => cmd_decision_add(parse_decision_add_args(rest)?),
        other => Err(CliError::usage(format!(
            "unknown decision subcommand `{other}`; use list, show, or add"
        ))),
    }
}

#[derive(Debug, Default)]
struct DecisionAddOptions {
    title: Option<String>,
    body: Option<String>,
    references: Vec<String>,
    tags: Vec<String>,
}

fn parse_decision_add_args(args: &[String]) -> Result<DecisionAddOptions, CliError> {
    let mut options = DecisionAddOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--title" => {
                index += 1;
                options.title = Some(required_value(args, index, "--title")?.to_string());
            }
            "--body" => {
                index += 1;
                options.body = Some(required_value(args, index, "--body")?.to_string());
            }
            "--reference" => {
                index += 1;
                options
                    .references
                    .push(required_value(args, index, "--reference")?.to_string());
            }
            "--tag" => {
                index += 1;
                options
                    .tags
                    .push(required_value(args, index, "--tag")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown decision add flag `{flag}`"
                )))
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected decision add argument `{value}`"
                )))
            }
        }
        index += 1;
    }
    Ok(options)
}

fn cmd_decision_list(json: bool) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?
        .into_iter()
        .filter(|doc| doc.doc_type() == "decision")
        .collect::<Vec<_>>();
    docs.sort_by(|a, b| a.id().cmp(b.id()));
    if json {
        println!("{}", decision_list_json(&docs));
    } else {
        print_decision_table(&docs);
    }
    Ok(())
}

fn cmd_decision_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let doc = find_document(&workspace, &options.id)?
        .ok_or_else(|| CliError::user(format!("decision not found: {}", options.id)))?;
    if doc.doc_type() != "decision" {
        return Err(CliError::user(format!(
            "{} is type {}, not decision",
            doc.id(),
            doc.doc_type()
        )));
    }
    if options.json {
        println!("{}", decision_show_json(&doc));
    } else {
        print_show(&doc);
    }
    Ok(())
}

fn cmd_decision_add(options: DecisionAddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let title = require_nonempty(
        options.title.as_deref(),
        "decision add requires --title <title>",
    )?;
    for reference in &options.references {
        require_existing_document(&workspace, reference, "reference")?;
    }

    let decision_id = next_sequential_id(&workspace, "decision")?;
    let now = current_timestamp();
    let decision_path = workspace.board_dir.join(format!("{decision_id}.md"));
    let mut lines = vec![
        "---".to_string(),
        format!("id: {decision_id}"),
        "type: decision".to_string(),
        format!("title: {}", yaml_double_quote(title)),
    ];
    push_array_line(&mut lines, "references", &options.references);
    push_array_line(&mut lines, "tags", &options.tags);
    lines.push(format!("createdAt: {}", yaml_double_quote(&now)));
    lines.push(format!("updatedAt: {}", yaml_double_quote(&now)));
    lines.push("---".to_string());
    lines.push(String::new());
    if let Some(body) = options.body.as_deref() {
        lines.push(body.to_string());
    }
    lines.push(String::new());
    write_atomic(&decision_path, &lines.join("\n"))?;
    append_event(&workspace, "decision.created", &decision_id, title)?;

    println!("Created decision");
    println!("ID:    {decision_id}");
    println!("Title: {title}");
    println!("Path:  {}", display_path(&decision_path));
    Ok(())
}

fn cmd_tui(args: &[String]) -> Result<(), CliError> {
    if !args.is_empty() {
        return Err(CliError::usage(
            "tandem tui does not accept options in this implementation slice",
        ));
    }

    tui::run_tui()
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
                events_path: tandem_dir.join("events.jsonl"),
                config_path,
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
        "No Tandem workspace found. Run `tandem init` first.",
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
        CliError::user(format!("Parse failure: {}: {message}", display_path(path)))
    })?;
    let fields = parse_frontmatter_fields(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(path)
        ))
    })?;

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

fn find_board_document(workspace: &Workspace, id: &str) -> Result<Option<Document>, CliError> {
    Ok(
        read_documents(&workspace.board_dir, DocumentLocation::Board)?
            .into_iter()
            .find(|doc| doc.id() == id),
    )
}

fn find_log_document(workspace: &Workspace, id: &str) -> Result<Option<Document>, CliError> {
    Ok(read_documents(&workspace.logs_dir, DocumentLocation::Logs)?
        .into_iter()
        .find(|doc| doc.id() == id))
}

fn document_exists(workspace: &Workspace, id: &str) -> Result<bool, CliError> {
    Ok(find_document(workspace, id)?.is_some())
}

fn require_existing_document(workspace: &Workspace, id: &str, kind: &str) -> Result<(), CliError> {
    if document_exists(workspace, id)? {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: {kind} document not found: {id}"
        )))
    }
}

fn unresolved_blockers(
    workspace: &Workspace,
    blockers: Option<&str>,
) -> Result<Vec<String>, CliError> {
    let mut unresolved = Vec::new();
    for blocker in blockers.map(parse_field_values).unwrap_or_default() {
        if find_board_document(workspace, &blocker)?.is_some() {
            unresolved.push(blocker);
        } else if find_log_document(workspace, &blocker)?.is_none() {
            unresolved.push(format!("{blocker} (missing)"));
        }
    }
    Ok(unresolved)
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

fn parse_frontmatter_fields(frontmatter: &str) -> Result<HashMap<String, String>, String> {
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

fn parse_frontmatter_yaml(frontmatter: &str) -> Result<Option<Yaml>, String> {
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
    let root = docs.into_iter().next().unwrap();
    if root.is_badvalue() {
        return Err("frontmatter root must be a YAML mapping".to_string());
    }
    Ok(Some(root))
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
    let mut scalars = Vec::new();
    for value in values {
        scalars.push(yaml_scalar_to_string(value)?);
    }
    Some(inline_array(&scalars))
}

fn yaml_scalar_to_string(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(value) | Yaml::Real(value) => Some(value.clone()),
        Yaml::Integer(value) => Some(value.to_string()),
        Yaml::Boolean(value) => Some(value.to_string()),
        Yaml::Null | Yaml::BadValue | Yaml::Array(_) | Yaml::Hash(_) | Yaml::Alias(_) => None,
    }
}

fn yaml_mapping_value<'a>(root: &'a Yaml, key: &str) -> Option<&'a Yaml> {
    root.as_hash()?.iter().find_map(|(candidate, value)| {
        (yaml_scalar_to_string(candidate).as_deref() == Some(key)).then_some(value)
    })
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

fn filter_documents(docs: Vec<Document>, options: &ListOptions) -> Vec<Document> {
    docs.into_iter()
        .filter(|doc| {
            options.state.as_deref().map_or(true, |state| {
                state_matches_filter(doc.field("state"), state)
            })
        })
        .filter(|doc| {
            options
                .doc_type
                .as_deref()
                .map_or(true, |doc_type| doc.doc_type() == doc_type)
        })
        .filter(|doc| {
            options
                .priority
                .as_deref()
                .map_or(true, |priority| doc.field("priority") == Some(priority))
        })
        .filter(|doc| {
            options
                .assignee
                .as_deref()
                .map_or(true, |assignee| doc.field("assignee") == Some(assignee))
        })
        .filter(|doc| {
            options
                .tag
                .as_deref()
                .map_or(true, |tag| field_values_contain(doc.field("tags"), tag))
        })
        .filter(|doc| {
            options
                .accord
                .as_deref()
                .map_or(true, |accord| accord_status(doc) == Some(accord))
        })
        .filter(|doc| {
            options
                .review
                .as_deref()
                .map_or(true, |review| review_status(doc) == Some(review))
        })
        .collect()
}

#[derive(Debug)]
struct MoveTaskOutcome {
    id: String,
    from: String,
    to: String,
    changed: bool,
    path: PathBuf,
    accord_sync: Option<String>,
}

fn move_task_to_state(
    workspace: &Workspace,
    id: &str,
    state: &str,
) -> Result<MoveTaskOutcome, CliError> {
    validate_state(workspace, state)?;

    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be moved in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;

    let doc_id = doc.id().to_string();
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    if previous_state == state {
        return Ok(MoveTaskOutcome {
            id: doc_id,
            from: previous_state,
            to: state.to_string(),
            changed: false,
            path: doc.path,
            accord_sync: None,
        });
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("state".to_string(), state.to_string());
    updates.insert("updatedAt".to_string(), now.clone());
    let mut patched = patch_frontmatter_content(&content, &updates, &[])?;
    let mut synced_accord_event = None;
    let mut accord_sync = None;
    if state == "in-progress" && accord_status(&doc) == Some("ready") {
        let mut accord = AccordRecord::from_document(&doc, &now);
        accord.status = "claimed".to_string();
        if accord.claimed_at.is_none() {
            accord.claimed_at = Some(now.clone());
        }
        patched = patch_accord_content(&patched, &accord)?;
        synced_accord_event = Some("accord.claimed");
        accord_sync = Some("ready -> claimed".to_string());
    }
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "task.moved",
        &doc_id,
        &format!("Moved {doc_id} from {previous_state} to {state}"),
    )?;
    if let Some(event_name) = synced_accord_event {
        append_event(
            workspace,
            event_name,
            &doc_id,
            &format!("Synchronized accord claim for {doc_id} after move"),
        )?;
    }

    Ok(MoveTaskOutcome {
        id: doc_id,
        from: previous_state,
        to: state.to_string(),
        changed: true,
        path: doc.path,
        accord_sync,
    })
}

fn accord_state_sync_target<'a>(accord_status: &str, current_state: &'a str) -> Option<&'a str> {
    match accord_status {
        "claimed" if current_state == "todo" => Some("in-progress"),
        "delivered" | "accepted"
            if matches!(current_state, "todo" | "in-progress" | LEGACY_REVIEW_STATE) =>
        {
            Some(VALIDATION_STATE)
        }
        "rework" if matches!(current_state, VALIDATION_STATE | LEGACY_REVIEW_STATE) => {
            Some("in-progress")
        }
        _ => None,
    }
}

fn accord_state_divergence_warning(doc: &Document) -> Option<String> {
    let status = accord_status(doc)?;
    let state = doc.field("state")?;
    let expected = accord_state_sync_target(status, state)?;
    Some(format!(
        "{} has workflow state `{state}` but accord.status `{status}` suggests `{expected}`; preserving recorded state until a mutation synchronizes it.",
        doc.id()
    ))
}

fn document_warnings(doc: &Document) -> Vec<String> {
    accord_state_divergence_warning(doc).into_iter().collect()
}

fn state_matches_filter(actual: Option<&str>, requested: &str) -> bool {
    actual == Some(requested)
        || (requested == VALIDATION_STATE && actual == Some(LEGACY_REVIEW_STATE))
        || (requested == LEGACY_REVIEW_STATE && actual == Some(VALIDATION_STATE))
}

fn is_known_or_legacy_state(states: &[String], state: &str) -> bool {
    states.iter().any(|known| known == state)
        || (state == LEGACY_REVIEW_STATE && states.iter().any(|known| known == VALIDATION_STATE))
        || (state == VALIDATION_STATE && states.iter().any(|known| known == LEGACY_REVIEW_STATE))
}

fn display_known_states(states: &[String]) -> String {
    let mut display = states.to_vec();
    if states.iter().any(|state| state == VALIDATION_STATE)
        && !states.iter().any(|state| state == LEGACY_REVIEW_STATE)
    {
        display.push(format!("{LEGACY_REVIEW_STATE} (legacy alias)"));
    } else if states.iter().any(|state| state == LEGACY_REVIEW_STATE)
        && !states.iter().any(|state| state == VALIDATION_STATE)
    {
        display.push(format!("{VALIDATION_STATE} (preferred alias)"));
    }
    display.join(", ")
}

fn accord_status(doc: &Document) -> Option<&str> {
    doc.field("accord.status")
        .or_else(|| doc.field("accordStatus"))
}

fn review_status(doc: &Document) -> Option<&str> {
    doc.field("review.status")
        .or_else(|| doc.field("reviewStatus"))
}

fn completion_summary(doc: &Document) -> Option<&str> {
    doc.field("completion.summary")
        .or_else(|| doc.field("completionSummary"))
}

fn completion_validation(doc: &Document) -> Option<&str> {
    doc.field("completion.validation")
        .or_else(|| doc.field("completion.validation.summary"))
        .or_else(|| doc.field("completion.validation.status"))
        .or_else(|| doc.field("completionValidation"))
}

fn completion_reviewer(doc: &Document) -> Option<&str> {
    doc.field("completion.reviewer")
        .or_else(|| doc.field("completionReviewer"))
}

fn completion_files_changed(doc: &Document) -> Vec<String> {
    doc.field("completion.filesChanged")
        .or_else(|| doc.field("filesChanged"))
        .map(parse_field_values)
        .unwrap_or_default()
}

fn validate_task_document_for_mutation(
    workspace: &Workspace,
    doc: &Document,
) -> Result<(), CliError> {
    let mut errors = Vec::new();
    if doc.id().trim().is_empty() {
        errors.push("missing required field `id`".to_string());
    }
    if doc.title().trim().is_empty() {
        errors.push("missing required field `title`".to_string());
    }
    match doc.field("type") {
        Some("task") => {}
        Some(other) => errors.push(format!("expected type `task`, found `{other}`")),
        None => errors.push("missing required field `type`".to_string()),
    }
    if doc.location == DocumentLocation::Board {
        match doc.field("state") {
            Some(state) if !state.trim().is_empty() => {
                let states = read_workspace_states(workspace)?;
                if !is_known_or_legacy_state(&states, state) {
                    errors.push(format!(
                        "unknown state `{state}`; known states: {}",
                        display_known_states(&states)
                    ));
                }
            }
            _ => errors.push("missing required field `state`".to_string()),
        }
    }
    if let Some(parent) = doc
        .field("parentId")
        .filter(|value| !value.trim().is_empty())
    {
        if !document_exists(workspace, parent)? {
            errors.push(format!("unresolved parentId `{parent}`"));
        }
    }
    for blocker in doc
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default()
    {
        if !document_exists(workspace, &blocker)? {
            errors.push(format!("unresolved blocker `{blocker}`"));
        }
    }
    if has_metadata(doc, "accord") || doc.field("accordStatus").is_some() {
        match accord_status(doc) {
            Some(status) if ACCORD_STATUSES.contains(&status) => {}
            Some(status) => errors.push(format!("invalid accord.status `{status}`")),
            None => {
                errors.push("accord.status is required when accord metadata is present".to_string())
            }
        }
    }
    if has_metadata(doc, "review") || doc.field("reviewStatus").is_some() {
        match review_status(doc) {
            Some(status) if REVIEW_STATUSES.contains(&status) => {}
            Some(status) => errors.push(format!("invalid review.status `{status}`")),
            None => {
                errors.push("review.status is required when review metadata is present".to_string())
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed for {}: {}",
            display_path(&doc.path),
            errors.join("; ")
        )))
    }
}

fn has_metadata(doc: &Document, prefix: &str) -> bool {
    let nested_prefix = format!("{prefix}.");
    doc.fields
        .keys()
        .any(|key| key == prefix || key.starts_with(&nested_prefix))
}

impl AccordRecord {
    fn from_document(doc: &Document, updated_at: &str) -> Self {
        Self {
            status: accord_status(doc).unwrap_or("missing").to_string(),
            assignee: doc.field("accord.assignee").map(str::to_string),
            claimed_at: doc.field("accord.claimedAt").map(str::to_string),
            delivered_at: doc.field("accord.deliveredAt").map(str::to_string),
            deliverables: doc
                .field("accord.deliverables")
                .map(parse_field_values)
                .unwrap_or_default(),
            validations: doc
                .field("accord.validation.commands")
                .or_else(|| doc.field("accord.validation"))
                .or_else(|| doc.field("accord.validations"))
                .map(parse_field_values)
                .unwrap_or_default(),
            constraints: doc
                .field("accord.constraints")
                .map(parse_field_values)
                .unwrap_or_default(),
            summary: doc.field("accord.summary").map(str::to_string),
            evidence: doc
                .field("accord.evidence")
                .map(parse_field_values)
                .unwrap_or_default(),
            files_changed: doc
                .field("accord.filesChanged")
                .map(parse_field_values)
                .unwrap_or_default(),
            reviewer: doc.field("accord.reviewer").map(str::to_string),
            note: doc.field("accord.note").map(str::to_string),
            reason: doc.field("accord.reason").map(str::to_string),
            updated_at: updated_at.to_string(),
        }
    }
}

fn validate_accord_inputs(action: &str, options: &AccordOptions) -> Result<(), CliError> {
    match action {
        "claim" => {
            require_nonempty(
                options.assignee.as_deref(),
                "accord claim requires --assignee <name>",
            )?;
        }
        "deliver" => {
            require_nonempty(
                options.summary.as_deref(),
                "accord deliver requires --summary <text>",
            )?;
        }
        "rework" => {
            require_nonempty(
                options.note.as_deref(),
                "accord rework requires --note <text>",
            )?;
        }
        "block" | "fail" => {
            require_nonempty(
                options.reason.as_deref(),
                &format!("accord {action} requires --reason <text>"),
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_accord_transition(action: &str, previous_status: &str) -> Result<(), CliError> {
    match action {
        "accept" if previous_status != "delivered" && previous_status != "accepted" => {
            Err(CliError::user(format!(
                "accord accept requires current accord.status=delivered; current status is {previous_status}"
            )))
        }
        "rework" if previous_status != "delivered" && previous_status != "rework" => {
            Err(CliError::user(format!(
                "accord rework requires current accord.status=delivered; current status is {previous_status}"
            )))
        }
        "ready" | "claim" | "deliver" | "block" | "fail"
            if previous_status == "accepted" && action != "ready" =>
        {
            Err(CliError::user(
                "accepted accord cannot transition without resetting with `tandem accord ready`".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn apply_accord_action(
    accord: &mut AccordRecord,
    action: &str,
    status: &str,
    options: &AccordOptions,
) {
    accord.status = status.to_string();
    match action {
        "ready" => {
            accord.claimed_at = None;
            accord.delivered_at = None;
            accord.summary = None;
            accord.evidence.clear();
            accord.files_changed.clear();
            accord.reviewer = None;
            accord.note = None;
            accord.reason = None;
        }
        "claim" => {
            accord.claimed_at = Some(accord.updated_at.clone());
            accord.delivered_at = None;
            accord.summary = None;
            accord.evidence.clear();
            accord.files_changed.clear();
            accord.reviewer = None;
            accord.note = None;
            accord.reason = None;
        }
        "deliver" => {
            accord.delivered_at = Some(accord.updated_at.clone());
            accord.reviewer = None;
            accord.note = None;
            accord.reason = None;
        }
        "accept" => {
            accord.reason = None;
        }
        "rework" => {
            accord.reviewer = None;
            accord.reason = None;
        }
        "block" | "fail" => {
            accord.reviewer = None;
            accord.note = None;
        }
        _ => {}
    }
    if let Some(assignee) = options
        .assignee
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        accord.assignee = Some(assignee.to_string());
    }
    if !options.deliverables.is_empty() {
        accord.deliverables = options.deliverables.clone();
    }
    if !options.validations.is_empty() {
        accord.validations = options.validations.clone();
    }
    if !options.constraints.is_empty() {
        accord.constraints = options.constraints.clone();
    }
    if let Some(summary) = options
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        accord.summary = Some(summary.to_string());
    }
    if !options.evidence.is_empty() {
        accord.evidence = options.evidence.clone();
    }
    if !options.files_changed.is_empty() {
        accord.files_changed = options.files_changed.clone();
    }
    if let Some(reviewer) = options
        .reviewer
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        accord.reviewer = Some(reviewer.to_string());
    }
    if let Some(note) = options
        .note
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        accord.note = Some(note.to_string());
    }
    if let Some(reason) = options
        .reason
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        accord.reason = Some(reason.to_string());
    }
}

fn accord_event_name(action: &str) -> &'static str {
    match action {
        "ready" => "accord.ready",
        "claim" => "accord.claimed",
        "deliver" => "accord.delivered",
        "accept" => "accord.accepted",
        "rework" => "accord.rework",
        "block" => "accord.blocked",
        "fail" => "accord.failed",
        _ => "accord.updated",
    }
}

fn print_accord_update(
    id: &str,
    previous_status: &str,
    status: &str,
    event_name: &str,
    path: &Path,
) {
    println!("Updated accord");
    println!("ID:     {id}");
    println!("From:   {previous_status}");
    println!("To:     {status}");
    println!("Path:   {}", display_path(path));
    println!("Event:  {event_name}");
}

fn sort_documents(docs: &mut [Document]) {
    docs.sort_by(|a, b| {
        a.field("state")
            .unwrap_or("")
            .cmp(b.field("state").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
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

fn print_document_warnings(docs: &[Document]) {
    for warning in docs.iter().flat_map(document_warnings) {
        println!("Warning: {warning}");
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
    if let Some(due_date) = doc.field("dueDate") {
        println!("Due:       {due_date}");
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
    if let Some(status) = accord_status(doc) {
        println!("Accord:    {status}");
    }
    if let Some(status) = review_status(doc) {
        println!("Review:    {status}");
    }
    for warning in document_warnings(doc) {
        println!("Warning:   {warning}");
    }
    if let Some(summary) = completion_summary(doc) {
        println!("Summary:   {summary}");
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

#[derive(Debug)]
struct SearchResult {
    doc: Document,
    snippet: String,
}

fn search_match(doc: Document, query: &str) -> Option<SearchResult> {
    let lowered_query = query.to_lowercase();
    let mut haystacks = vec![
        doc.id().to_string(),
        doc.title().to_string(),
        doc.body.clone(),
    ];
    haystacks.extend(doc.fields.values().cloned());
    for haystack in haystacks {
        if haystack.to_lowercase().contains(&lowered_query) {
            return Some(SearchResult {
                doc,
                snippet: snippet_for_match(&haystack, query),
            });
        }
    }
    None
}

fn snippet_for_match(value: &str, query: &str) -> String {
    let condensed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if condensed.chars().count() <= 80 {
        return condensed;
    }

    let lower = condensed.to_lowercase();
    let query_lower = query.to_lowercase();
    let byte_index = lower.find(&query_lower).unwrap_or(0);
    let char_index = condensed[..byte_index].chars().count();
    let start = char_index.saturating_sub(20);
    let end = (start + 80).min(condensed.chars().count());
    let chars = condensed.chars().collect::<Vec<_>>();
    let mut snippet = chars[start..end].iter().collect::<String>();
    if start > 0 {
        snippet.insert_str(0, "…");
    }
    if end < chars.len() {
        snippet.push('…');
    }
    snippet
}

fn print_search_table(results: &[SearchResult]) {
    if results.is_empty() {
        println!("No matching Tandem documents found.");
        return;
    }
    println!(
        "{:<12} {:<8} {:<12} {:<8} {:<32} MATCH",
        "ID", "WHERE", "STATE", "TYPE", "TITLE"
    );
    for result in results {
        let doc = &result.doc;
        println!(
            "{:<12} {:<8} {:<12} {:<8} {:<32} {}",
            truncate(doc.id(), 12),
            doc.location.as_str(),
            truncate(doc.field("state").unwrap_or("-"), 12),
            truncate(doc.doc_type(), 8),
            truncate(doc.title(), 32),
            truncate(&result.snippet, 80)
        );
    }
}

fn print_log_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No completed Tandem logs found.");
        return;
    }
    println!("{:<12} {:<20} {:<36} SUMMARY", "ID", "COMPLETED", "TITLE");
    for doc in docs {
        println!(
            "{:<12} {:<20} {:<36} {}",
            truncate(doc.id(), 12),
            truncate(doc.field("completedAt").unwrap_or("-"), 20),
            truncate(doc.title(), 36),
            truncate(completion_summary(doc).unwrap_or("-"), 80)
        );
    }
}

fn print_log_show(doc: &Document) {
    println!("Log document");
    print_show(doc);
    if let Some(validation) = completion_validation(doc) {
        println!();
        println!("Validation: {validation}");
    }
    let files = completion_files_changed(doc);
    if !files.is_empty() {
        println!("Files changed: {}", files.join(", "));
    }
    if let Some(reviewer) = completion_reviewer(doc) {
        println!("Reviewer: {reviewer}");
    }
}

fn print_decision_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No Tandem decisions found.");
        return;
    }
    println!("{:<14} {:<42} {:<24} SUMMARY", "ID", "TITLE", "REFERENCES");
    for doc in docs {
        println!(
            "{:<14} {:<42} {:<24} {}",
            truncate(doc.id(), 14),
            truncate(doc.title(), 42),
            truncate(doc.field("references").unwrap_or("-"), 24),
            truncate(&first_body_line(doc), 80)
        );
    }
}

#[derive(Debug, Clone)]
struct RuleItem {
    id: usize,
    rule: String,
    source: Option<String>,
}

type RulesByCategory = BTreeMap<String, Vec<RuleItem>>;

fn empty_rules() -> RulesByCategory {
    let mut rules = BTreeMap::new();
    for category in ["always", "never", "prefer", "context"] {
        rules.insert(category.to_string(), Vec::new());
    }
    rules
}

fn read_rules(config_path: &Path) -> Result<RulesByCategory, CliError> {
    let root = read_frontmatter_yaml_file(config_path)?;
    Ok(parse_rules_from_yaml(root.as_ref()))
}

fn parse_rules_from_content(content: &str, path: &Path) -> Result<RulesByCategory, CliError> {
    let (frontmatter, _) = split_frontmatter(content).map_err(|message| {
        CliError::user(format!("Parse failure: {}: {message}", display_path(path)))
    })?;
    let root = parse_frontmatter_yaml(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(path)
        ))
    })?;
    Ok(parse_rules_from_yaml(root.as_ref()))
}

fn read_frontmatter_yaml_file(path: &Path) -> Result<Option<Yaml>, CliError> {
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

fn parse_rules_from_yaml(root: Option<&Yaml>) -> RulesByCategory {
    let mut rules = empty_rules();
    let Some(rules_yaml) = root.and_then(|root| yaml_mapping_value(root, "rules")) else {
        return rules;
    };
    for category in ["always", "never", "prefer", "context"] {
        let Some(category_yaml) = yaml_mapping_value(rules_yaml, category) else {
            continue;
        };
        let parsed = parse_rule_category_items(category_yaml);
        rules.insert(category.to_string(), parsed);
    }
    rules
}

fn parse_rule_category_items(value: &Yaml) -> Vec<RuleItem> {
    match value {
        Yaml::Array(items) => items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| parse_rule_item(item, index + 1))
            .collect(),
        _ => parse_rule_item(value, 1).into_iter().collect(),
    }
}

fn parse_rule_item(value: &Yaml, fallback_id: usize) -> Option<RuleItem> {
    match value {
        Yaml::Hash(_) => {
            let id = yaml_mapping_value(value, "id")
                .and_then(yaml_scalar_to_string)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(fallback_id);
            let rule = yaml_mapping_value(value, "rule")
                .and_then(yaml_scalar_to_string)
                .unwrap_or_default();
            if rule.trim().is_empty() {
                return None;
            }
            let source = yaml_mapping_value(value, "source")
                .and_then(yaml_scalar_to_string)
                .filter(|source| !source.trim().is_empty());
            Some(RuleItem { id, rule, source })
        }
        _ => yaml_scalar_to_string(value)
            .filter(|rule| !rule.trim().is_empty())
            .map(|rule| RuleItem {
                id: fallback_id,
                rule,
                source: None,
            }),
    }
}

fn require_rule_category(category: Option<&str>) -> Result<&str, CliError> {
    let category =
        category.ok_or_else(|| CliError::usage("rules mutation requires --category <category>"))?;
    validate_rule_category(category)?;
    Ok(category)
}

fn validate_rule_category(category: &str) -> Result<(), CliError> {
    if ["always", "never", "prefer", "context"].contains(&category) {
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "unknown rule category `{category}`; use always, never, prefer, or context"
        )))
    }
}

fn warn_missing_rule_source(workspace: &Workspace, source: Option<&str>) -> Result<(), CliError> {
    if let Some(source) = source {
        if !source.trim().is_empty() && !document_exists(workspace, source)? {
            println!("Warning: rule source not found: {source}");
        }
    }
    Ok(())
}

fn patch_completion_content(
    content: &str,
    summary: &str,
    files_changed: &[String],
    validation: Option<&str>,
    reviewer: Option<&str>,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let completion_block = render_completion_block(summary, files_changed, validation, reviewer);
    let mut output_frontmatter = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0usize;
    let mut replaced = false;

    while index < lines.len() {
        let raw_line = lines[index];
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if matches!(
            frontmatter_line_key(line),
            Some("completionSummary")
                | Some("completionValidation")
                | Some("completionReviewer")
                | Some("filesChanged")
        ) {
            index += 1;
            continue;
        }
        if frontmatter_line_key(line) == Some("completion") {
            output_frontmatter.push_str(&completion_block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let skip_line = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(skip_line) {
                    break;
                }
                index += 1;
            }
            continue;
        }
        output_frontmatter.push_str(raw_line);
        index += 1;
    }

    if !replaced {
        if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
            output_frontmatter.push('\n');
        }
        output_frontmatter.push_str(&completion_block);
    }

    if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
        output_frontmatter.push('\n');
    }

    Ok(format!("---\n{}---\n{}", output_frontmatter, body))
}

fn render_completion_block(
    summary: &str,
    files_changed: &[String],
    validation: Option<&str>,
    reviewer: Option<&str>,
) -> String {
    let mut lines = Vec::new();
    lines.push("completion:".to_string());
    lines.push(format!("  summary: {}", yaml_double_quote(summary)));
    if !files_changed.is_empty() {
        lines.push(format!("  filesChanged: {}", inline_array(files_changed)));
    }
    if let Some(validation) = validation.filter(|value| !value.trim().is_empty()) {
        lines.push(format!(
            "  validation: {}",
            yaml_double_quote(validation.trim())
        ));
    }
    if let Some(reviewer) = reviewer.filter(|value| !value.trim().is_empty()) {
        lines.push(format!(
            "  reviewer: {}",
            yaml_double_quote(reviewer.trim())
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn patch_accord_content(content: &str, accord: &AccordRecord) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let accord_block = render_accord_block(accord);
    let mut output_frontmatter = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0usize;
    let mut replaced = false;

    while index < lines.len() {
        let raw_line = lines[index];
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if frontmatter_line_key(line) == Some("accord") {
            output_frontmatter.push_str(&accord_block);
            replaced = true;
            index += 1;
            while index < lines.len() {
                let skip_line = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(skip_line) {
                    break;
                }
                index += 1;
            }
            continue;
        }
        output_frontmatter.push_str(raw_line);
        index += 1;
    }

    if !replaced {
        if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
            output_frontmatter.push('\n');
        }
        output_frontmatter.push_str(&accord_block);
    }

    if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
        output_frontmatter.push('\n');
    }

    Ok(format!("---\n{}---\n{}", output_frontmatter, body))
}

fn render_accord_block(accord: &AccordRecord) -> String {
    let mut lines = Vec::new();
    lines.push("accord:".to_string());
    lines.push(format!("  status: {}", yaml_double_quote(&accord.status)));
    push_optional_nested_line(&mut lines, "assignee", accord.assignee.as_deref());
    push_optional_nested_line(&mut lines, "claimedAt", accord.claimed_at.as_deref());
    push_optional_nested_line(&mut lines, "deliveredAt", accord.delivered_at.as_deref());
    push_nested_array_line(&mut lines, "deliverables", &accord.deliverables);
    push_nested_validation_commands(&mut lines, &accord.validations);
    push_nested_array_line(&mut lines, "constraints", &accord.constraints);
    push_optional_nested_line(&mut lines, "summary", accord.summary.as_deref());
    push_nested_array_line(&mut lines, "evidence", &accord.evidence);
    push_nested_array_line(&mut lines, "filesChanged", &accord.files_changed);
    push_optional_nested_line(&mut lines, "reviewer", accord.reviewer.as_deref());
    push_optional_nested_line(&mut lines, "note", accord.note.as_deref());
    push_optional_nested_line(&mut lines, "reason", accord.reason.as_deref());
    lines.push(format!(
        "  updatedAt: {}",
        yaml_double_quote(&accord.updated_at)
    ));
    lines.push(String::new());
    lines.join("\n")
}

fn push_optional_nested_line(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        if !value.trim().is_empty() {
            lines.push(format!("  {key}: {}", yaml_double_quote(value.trim())));
        }
    }
}

fn push_nested_array_line(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("  {key}: {}", inline_array(values)));
    }
}

fn push_nested_validation_commands(lines: &mut Vec<String>, values: &[String]) {
    if !values.is_empty() {
        lines.push("  validation:".to_string());
        lines.push(format!("    commands: {}", inline_array(values)));
    }
}

fn patch_rules_category_content(
    content: &str,
    category: &str,
    rules: &RulesByCategory,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let category_block = render_rule_category_block(
        category,
        rules.get(category).map(Vec::as_slice).unwrap_or(&[]),
    );
    let mut output_frontmatter = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_rules = false;
    let mut saw_rules = false;
    let mut replaced_category = false;

    while index < lines.len() {
        let raw_line = lines[index];
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');

        if !in_rules {
            if frontmatter_line_key(line) == Some("rules") {
                let inline_value = line
                    .split_once(':')
                    .map(|(_, value)| value.trim())
                    .unwrap_or("");
                if inline_value.is_empty() {
                    output_frontmatter.push_str(raw_line);
                } else {
                    output_frontmatter.push_str("rules:\n");
                }
                in_rules = true;
                saw_rules = true;
            } else {
                output_frontmatter.push_str(raw_line);
            }
            index += 1;
            continue;
        }

        if is_top_level_frontmatter_boundary(line) {
            if !replaced_category {
                output_frontmatter.push_str(&category_block);
                replaced_category = true;
            }
            in_rules = false;
            output_frontmatter.push_str(raw_line);
            index += 1;
            continue;
        }

        if rule_category_key(line) == Some(category) {
            output_frontmatter.push_str(&category_block);
            replaced_category = true;
            index += 1;
            while index < lines.len() {
                let skip_line = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                if is_top_level_frontmatter_boundary(skip_line)
                    || rule_category_key(skip_line).is_some()
                {
                    break;
                }
                index += 1;
            }
            continue;
        }

        output_frontmatter.push_str(raw_line);
        index += 1;
    }

    if in_rules && !replaced_category {
        output_frontmatter.push_str(&category_block);
    }

    if !saw_rules {
        if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
            output_frontmatter.push('\n');
        }
        output_frontmatter.push_str(&render_rules_block(rules));
    }

    if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
        output_frontmatter.push('\n');
    }

    Ok(format!("---\n{}---\n{}", output_frontmatter, body))
}

fn render_rules_block(rules: &RulesByCategory) -> String {
    let mut output = String::from("rules:\n");
    for category in ["always", "never", "prefer", "context"] {
        output.push_str(&render_rule_category_block(
            category,
            rules.get(category).map(Vec::as_slice).unwrap_or(&[]),
        ));
    }
    output
}

fn render_rule_category_block(category: &str, items: &[RuleItem]) -> String {
    let mut lines = Vec::new();
    if items.is_empty() {
        lines.push(format!("  {category}: []"));
    } else {
        lines.push(format!("  {category}:"));
        for item in items {
            lines.push(format!("    - id: {}", item.id));
            lines.push(format!("      rule: {}", yaml_double_quote(&item.rule)));
            if let Some(source) = item.source.as_deref() {
                lines.push(format!("      source: {}", yaml_double_quote(source)));
            }
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn rule_category_key(line: &str) -> Option<&str> {
    let indentation = line.chars().take_while(|ch| *ch == ' ').count();
    if indentation != 2 || line.starts_with('\t') {
        return None;
    }
    let trimmed = line.trim();
    let (key, _) = trimmed.split_once(':')?;
    ["always", "never", "prefer", "context"]
        .contains(&key)
        .then_some(key)
}

fn print_rules(rules: &RulesByCategory, category_filter: Option<&str>) {
    let categories = ["always", "never", "prefer", "context"];
    let mut printed_any = false;
    for category in categories {
        if category_filter.is_some_and(|filter| filter != category) {
            continue;
        }
        println!("{category}:");
        let items = rules.get(category).map(Vec::as_slice).unwrap_or(&[]);
        if items.is_empty() {
            println!("  (none)");
        } else {
            printed_any = true;
            for item in items {
                match item.source.as_deref() {
                    Some(source) => println!("  {}. {} ({source})", item.id, item.rule),
                    None => println!("  {}. {}", item.id, item.rule),
                }
            }
        }
    }
    if !printed_any && category_filter.is_some() {
        // The category heading above is the intended empty-list output.
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
    let warnings = docs.iter().flat_map(document_warnings).collect::<Vec<_>>();

    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"counts\":{{\"total\":{},\"byState\":{{{}}}}}}},\"warnings\":{}}}",
        items.join(","),
        docs.len(),
        states,
        json_array_strings(&warnings)
    )
}

fn show_json(doc: &Document) -> String {
    let warnings = document_warnings(doc);
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{},\"body\":{},\"path\":{},\"location\":{}}},\"warnings\":{}}}",
        document_detail_json(doc),
        json_string(&doc.body),
        json_string(&display_path(&doc.path)),
        json_string(doc.location.as_str()),
        json_array_strings(&warnings)
    )
}

fn log_list_json(docs: &[Document]) -> String {
    let items = docs.iter().map(log_summary_json).collect::<Vec<_>>();
    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"count\":{}}},\"warnings\":[]}}",
        items.join(","),
        docs.len()
    )
}

fn log_show_json(doc: &Document) -> String {
    let files = completion_files_changed(doc);
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{},\"completion\":{{\"summary\":{},\"filesChanged\":{},\"validation\":{},\"reviewer\":{}}},\"body\":{},\"path\":{}}},\"warnings\":[]}}",
        document_detail_json(doc),
        json_string(completion_summary(doc).unwrap_or("")),
        json_array_strings(&files),
        json_string(completion_validation(doc).unwrap_or("")),
        json_string(completion_reviewer(doc).unwrap_or("")),
        json_string(&doc.body),
        json_string(&display_path(&doc.path))
    )
}

fn search_json(query: &str, results: &[SearchResult]) -> String {
    let items = results
        .iter()
        .map(|result| {
            let doc = &result.doc;
            let mut fields = Vec::new();
            push_json_field(&mut fields, "id", doc.id());
            push_json_field(&mut fields, "type", doc.doc_type());
            push_json_field(&mut fields, "title", doc.title());
            push_json_field(&mut fields, "location", doc.location.as_str());
            push_optional_json_field(&mut fields, "state", doc.field("state"));
            push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
            push_json_field(&mut fields, "snippet", &result.snippet);
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>();
    format!(
        "{{\"ok\":true,\"data\":{{\"query\":{},\"results\":[{}]}},\"warnings\":[]}}",
        json_string(query),
        items.join(",")
    )
}

fn rules_json(rules: &RulesByCategory, category_filter: Option<&str>) -> String {
    let categories = ["always", "never", "prefer", "context"];
    let mut category_fields = Vec::new();
    let mut count_fields = Vec::new();
    let mut total = 0usize;
    for category in categories {
        let items = rules.get(category).map(Vec::as_slice).unwrap_or(&[]);
        let included_items = if category_filter.is_some_and(|filter| filter != category) {
            Vec::new()
        } else {
            items.to_vec()
        };
        total += included_items.len();
        let json_items = included_items
            .iter()
            .map(|item| {
                let mut fields = Vec::new();
                fields.push(format!("\"id\":{}", item.id));
                push_json_field(&mut fields, "rule", &item.rule);
                push_optional_json_field(&mut fields, "source", item.source.as_deref());
                format!("{{{}}}", fields.join(","))
            })
            .collect::<Vec<_>>();
        category_fields.push(format!(
            "{}:[{}]",
            json_string(category),
            json_items.join(",")
        ));
        count_fields.push(format!(
            "{}:{}",
            json_string(category),
            included_items.len()
        ));
    }
    count_fields.push(format!("\"total\":{total}"));
    format!(
        "{{\"ok\":true,\"data\":{{\"rules\":{{{}}},\"counts\":{{{}}}}},\"warnings\":[]}}",
        category_fields.join(","),
        count_fields.join(",")
    )
}

fn decision_list_json(docs: &[Document]) -> String {
    let items = docs
        .iter()
        .map(|doc| {
            let references = doc
                .field("references")
                .map(parse_field_values)
                .unwrap_or_default();
            let mut fields = Vec::new();
            push_json_field(&mut fields, "id", doc.id());
            push_json_field(&mut fields, "type", doc.doc_type());
            push_json_field(&mut fields, "title", doc.title());
            fields.push(format!(
                "\"references\":{}",
                json_array_strings(&references)
            ));
            push_json_field(&mut fields, "summary", &first_body_line(doc));
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>();
    format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"count\":{}}},\"warnings\":[]}}",
        items.join(","),
        docs.len()
    )
}

fn decision_show_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    let references = doc
        .field("references")
        .map(parse_field_values)
        .unwrap_or_default();
    fields.push(format!(
        "\"references\":{}",
        json_array_strings(&references)
    ));
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    format!(
        "{{\"ok\":true,\"data\":{{\"decision\":{{{}}},\"body\":{},\"path\":{}}},\"warnings\":[]}}",
        fields.join(","),
        json_string(&doc.body),
        json_string(&display_path(&doc.path))
    )
}

fn first_body_line(doc: &Document) -> String {
    doc.body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

fn document_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "priority", doc.field("priority"));
    push_optional_json_field(&mut fields, "assignee", doc.field("assignee"));
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
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
        "dueDate",
        "createdAt",
        "updatedAt",
        "completedAt",
    ] {
        push_optional_json_field(&mut fields, key, doc.field(key));
    }
    push_optional_json_field(&mut fields, "completionSummary", completion_summary(doc));
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
    format!("{{{}}}", fields.join(","))
}

fn log_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
    push_optional_json_field(&mut fields, "summary", completion_summary(doc));
    push_optional_json_field(&mut fields, "validationStatus", completion_validation(doc));
    format!("{{{}}}", fields.join(","))
}

fn push_json_field(fields: &mut Vec<String>, key: &str, value: &str) {
    fields.push(format!("{}:{}", json_string(key), json_string(value)));
}

fn push_status_object_json(fields: &mut Vec<String>, key: &str, status: Option<&str>) {
    if let Some(status) = status {
        fields.push(format!(
            "{}:{{\"status\":{}}}",
            json_string(key),
            json_string(status)
        ));
    }
}

fn push_optional_json_field(fields: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_json_field(fields, key, value);
    }
}

fn json_array_strings(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
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

fn require_nonempty<'a>(value: Option<&'a str>, message: &str) -> Result<&'a str, CliError> {
    let value = value.ok_or_else(|| CliError::usage(message))?.trim();
    if value.is_empty() {
        Err(CliError::usage(message))
    } else {
        Ok(value)
    }
}

fn read_workspace_states(workspace: &Workspace) -> Result<Vec<String>, CliError> {
    let root = read_frontmatter_yaml_file(&workspace.config_path)?;
    let mut states = Vec::new();
    if let Some(states_yaml) = root
        .as_ref()
        .and_then(|root| yaml_mapping_value(root, "states"))
    {
        match states_yaml {
            Yaml::Array(items) => {
                for item in items {
                    if let Some(state) = yaml_scalar_to_string(item)
                        .or_else(|| yaml_mapping_value(item, "id").and_then(yaml_scalar_to_string))
                    {
                        if !state.trim().is_empty() {
                            states.push(state);
                        }
                    }
                }
            }
            _ => {
                if let Some(state) = yaml_scalar_to_string(states_yaml) {
                    if !state.trim().is_empty() {
                        states.push(state);
                    }
                }
            }
        }
    }
    if states.is_empty() {
        states.extend(DEFAULT_STATES.iter().map(|state| (*state).to_string()));
    }
    Ok(states)
}

fn validate_state(workspace: &Workspace, state: &str) -> Result<(), CliError> {
    if state.trim().is_empty() {
        return Err(CliError::usage("state must not be empty"));
    }
    let states = read_workspace_states(workspace)?;
    if is_known_or_legacy_state(&states, state) {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: unknown state `{state}`; known states: {}",
            display_known_states(&states)
        )))
    }
}

fn next_sequential_id(workspace: &Workspace, prefix: &str) -> Result<String, CliError> {
    let mut max_number = 0usize;
    let mut docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?;
    docs.extend(read_documents(&workspace.logs_dir, DocumentLocation::Logs)?);
    let needle = format!("{prefix}-");
    for doc in docs {
        if let Some(number) = doc.id().strip_prefix(&needle) {
            if let Ok(value) = number.parse::<usize>() {
                max_number = max_number.max(value);
            }
        }
    }
    Ok(format!("{prefix}-{}", max_number + 1))
}

fn patch_frontmatter_content(
    content: &str,
    updates: &BTreeMap<String, String>,
    removes: &[&str],
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let mut seen = BTreeMap::<String, bool>::new();
    let mut output_frontmatter = String::new();

    for raw_line in frontmatter.split_inclusive('\n') {
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(key) = frontmatter_line_key(line) {
            if removes.iter().any(|remove| *remove == key) {
                continue;
            }
            if let Some(value) = updates.get(key) {
                output_frontmatter.push_str(&format!("{key}: {}\n", yaml_value_for_update(value)));
                seen.insert(key.to_string(), true);
                continue;
            }
        }
        output_frontmatter.push_str(raw_line);
    }

    if !output_frontmatter.is_empty() && !output_frontmatter.ends_with('\n') {
        output_frontmatter.push('\n');
    }
    for (key, value) in updates {
        if !seen.contains_key(key) {
            output_frontmatter.push_str(&format!("{key}: {}\n", yaml_value_for_update(value)));
        }
    }

    Ok(format!("---\n{}---\n{}", output_frontmatter, body))
}

fn frontmatter_line_key(line: &str) -> Option<&str> {
    if line.starts_with(' ') || line.starts_with('\t') || line.trim_start().starts_with('-') {
        return None;
    }
    let (key, _) = line.split_once(':')?;
    let key = key.trim();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn is_top_level_frontmatter_boundary(line: &str) -> bool {
    !line.starts_with(' ')
        && !line.starts_with('\t')
        && !line.trim().is_empty()
        && (frontmatter_line_key(line).is_some() || line.trim_start().starts_with('#'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSignature {
    len: u64,
    modified: Option<SystemTime>,
}

fn read_file_snapshot(path: &Path) -> Result<(String, FileSignature), CliError> {
    let before = file_signature(path)?;
    let content = fs::read_to_string(path)?;
    let after = file_signature(path)?;
    if before != after {
        return Err(CliError::user(format!(
            "Write conflict: {} changed while the command was reading it. No files were updated; rerun the command.",
            display_path(path)
        )));
    }
    Ok((content, after))
}

fn ensure_file_unchanged(path: &Path, expected: &FileSignature) -> Result<(), CliError> {
    let current = file_signature(path)?;
    if &current == expected {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Write conflict: {} changed while the command was preparing its update. No files were updated; rerun the command.",
            display_path(path)
        )))
    }
}

fn file_signature(path: &Path) -> Result<FileSignature, CliError> {
    let metadata = fs::metadata(path)?;
    Ok(FileSignature {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn write_atomic(path: &Path, content: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = temporary_path_for(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|error| {
            CliError::user(format!(
                "Write failure: could not create temp file {} for {}: {error}",
                display_path(&temp_path),
                display_path(path)
            ))
        })?;
    if let Err(error) = file
        .write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
    {
        let _ = fs::remove_file(&temp_path);
        return Err(CliError::user(format!(
            "Write failure: could not write {}: {error}",
            display_path(path)
        )));
    }
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(CliError::user(format!(
            "Write failure: could not replace {}: {error}",
            display_path(path)
        )));
    }
    Ok(())
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.md");
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    path.with_file_name(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        millis
    ))
}

fn append_event(
    workspace: &Workspace,
    event_name: &str,
    id: &str,
    summary: &str,
) -> Result<(), CliError> {
    let ts = current_timestamp();
    let line = format!(
        "{{\"ts\":{},\"event\":{},\"id\":{},\"summary\":{}}}\n",
        json_string(&ts),
        json_string(event_name),
        json_string(id),
        json_string(summary)
    );
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&workspace.events_path)
        .map_err(|error| {
            CliError::user(format!(
                "Event append failure: could not open {} while recording `{event_name}` for `{id}`: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
                display_path(&workspace.events_path)
            ))
        })?;
    file.write_all(line.as_bytes()).map_err(|error| {
        CliError::user(format!(
            "Event append failure: could not append `{event_name}` for `{id}` to {}: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
            display_path(&workspace.events_path)
        ))
    })
}

fn file_name_for_path(path: &Path) -> Result<PathBuf, CliError> {
    path.file_name()
        .map(PathBuf::from)
        .ok_or_else(|| CliError::user(format!("cannot determine file name for {}", path.display())))
}

fn push_optional_line(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        if !value.trim().is_empty() {
            lines.push(format!("{key}: {}", yaml_double_quote(value.trim())));
        }
    }
}

fn push_array_line(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("{key}: {}", inline_array(values)));
    }
}

fn inline_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| yaml_double_quote(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn yaml_value_for_update(value: &str) -> String {
    if value.starts_with('[') && value.ends_with(']') {
        value.to_string()
    } else {
        yaml_double_quote(value)
    }
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

fn field_values_contain(value: Option<&str>, needle: &str) -> bool {
    value
        .map(parse_field_values)
        .map_or(false, |values| values.iter().any(|value| value == needle))
}

fn parse_field_values(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(docs) = YamlLoader::load_from_str(trimmed) {
            if let Some(Yaml::Array(values)) = docs.first() {
                return values
                    .iter()
                    .filter_map(yaml_scalar_to_string)
                    .filter(|item| !item.is_empty())
                    .collect();
            }
        }
        return trimmed[1..trimmed.len() - 1]
            .split(',')
            .map(|item| parse_scalar_value(item.trim()))
            .filter(|item| !item.is_empty())
            .collect();
    }
    vec![parse_scalar_value(trimmed)]
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect()
}

fn current_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_seconds(seconds)
}

fn format_unix_seconds(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
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
    fn derives_workspace_title_from_directory_basename() {
        assert_eq!(
            derive_workspace_title(Path::new("/tmp/Exact Project.Name")),
            "Exact Project.Name"
        );
        assert_eq!(
            derive_workspace_title(Path::new("/tmp/  spaced  ")),
            "  spaced  "
        );
        assert_eq!(
            derive_workspace_title(Path::new("/")),
            DEFAULT_WORKSPACE_TITLE
        );
    }

    #[test]
    fn parses_yaml_frontmatter_and_preserves_body() {
        let input = "---\nid: task-1\ntitle: \"Hello\"\nstate: todo\n---\n\nBody\n";
        let (frontmatter, body) = split_frontmatter(input).unwrap();
        let fields = parse_frontmatter_fields(&frontmatter).unwrap();
        assert_eq!(fields.get("id").map(String::as_str), Some("task-1"));
        assert_eq!(fields.get("title").map(String::as_str), Some("Hello"));
        assert_eq!(fields.get("state").map(String::as_str), Some("todo"));
        assert_eq!(body, "\nBody\n");
    }

    #[test]
    fn parses_nested_accord_and_review_statuses() {
        let frontmatter = r#"
id: task-1
accord:
  status: delivered
  assignee: pi
review:
  status: pending
tags: ["tui", "cli"]
"#;
        let fields = parse_frontmatter_fields(frontmatter).unwrap();
        assert_eq!(
            fields.get("accord.status").map(String::as_str),
            Some("delivered")
        );
        assert_eq!(
            fields.get("accordStatus").map(String::as_str),
            Some("delivered")
        );
        assert_eq!(
            fields.get("accord.assignee").map(String::as_str),
            Some("pi")
        );
        assert_eq!(
            fields.get("review.status").map(String::as_str),
            Some("pending")
        );
        assert_eq!(
            fields.get("reviewStatus").map(String::as_str),
            Some("pending")
        );
        assert_eq!(
            parse_field_values(fields.get("tags").unwrap()),
            vec!["tui", "cli"]
        );
    }

    #[test]
    fn parses_block_arrays_and_quoted_commas() {
        let frontmatter = r#"
tags:
  - "ui, polish"
  - cli
blockers: [task-1, "task-2"]
"#;
        let fields = parse_frontmatter_fields(frontmatter).unwrap();
        assert_eq!(
            parse_field_values(fields.get("tags").unwrap()),
            vec!["ui, polish", "cli"]
        );
        assert_eq!(
            parse_field_values(fields.get("blockers").unwrap()),
            vec!["task-1", "task-2"]
        );
    }

    #[test]
    fn parses_structured_rules_with_sources() {
        let root = parse_frontmatter_yaml(
            r#"
rules:
  always:
    - id: 3
      rule: "Run tests"
      source: decision-1
  prefer:
    - "Keep changes small"
"#,
        )
        .unwrap();
        let rules = parse_rules_from_yaml(root.as_ref());
        assert_eq!(rules["always"][0].id, 3);
        assert_eq!(rules["always"][0].rule, "Run tests");
        assert_eq!(rules["always"][0].source.as_deref(), Some("decision-1"));
        assert_eq!(rules["prefer"][0].id, 1);
        assert_eq!(rules["prefer"][0].rule, "Keep changes small");
    }

    #[test]
    fn patches_rules_category_without_touching_other_categories_or_body() {
        let input = "---\ntitle: Demo\nrules:\n  always: []\n  never:\n    - id: 9\n      rule: \"Keep me\"\nstate: ignored\n---\n\n# Body\n";
        let mut rules = empty_rules();
        rules.get_mut("always").unwrap().push(RuleItem {
            id: 1,
            rule: "Run tests".to_string(),
            source: Some("decision-1".to_string()),
        });
        let output = patch_rules_category_content(input, "always", &rules).unwrap();
        assert!(output.contains("rules:\n  always:\n    - id: 1\n"));
        assert!(output.contains("      source: \"decision-1\"\n"));
        assert!(output.contains("  never:\n    - id: 9\n      rule: \"Keep me\"\n"));
        assert!(output.contains("state: ignored\n"));
        assert!(output.ends_with("\n# Body\n"));
    }

    #[test]
    fn patches_accord_without_touching_body_or_other_fields() {
        let input = "---\nid: task-1\ntitle: Demo\naccord:\n  status: ready\n  assignee: pi\nreview:\n  status: pending\n---\n\nBody\n";
        let accord = AccordRecord {
            status: "delivered".to_string(),
            assignee: Some("pi".to_string()),
            delivered_at: Some("2026-06-26T00:00:00Z".to_string()),
            summary: Some("Done".to_string()),
            validations: vec!["cargo test".to_string()],
            evidence: vec!["cargo test passed".to_string()],
            updated_at: "2026-06-26T00:00:00Z".to_string(),
            ..AccordRecord::default()
        };
        let output = patch_accord_content(input, &accord).unwrap();
        assert!(output.contains("accord:\n  status: \"delivered\"\n"));
        assert!(output.contains("  assignee: \"pi\"\n"));
        assert!(output.contains("  deliveredAt: \"2026-06-26T00:00:00Z\"\n"));
        assert!(output.contains("  validation:\n    commands: [\"cargo test\"]\n"));
        assert!(output.contains("  summary: \"Done\"\n"));
        assert!(output.contains("  evidence: [\"cargo test passed\"]\n"));
        assert!(output.contains("review:\n  status: pending\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn divergence_warning_reports_sync_candidate_without_collapsing_state() {
        let doc = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: in-progress\naccord:\n  status: delivered\nreview:\n  status: pending\n",
            )
            .unwrap(),
            body: String::new(),
        };

        let warning = accord_state_divergence_warning(&doc).unwrap();
        assert!(warning.contains("workflow state `in-progress`"));
        assert!(warning.contains("accord.status `delivered` suggests `validation`"));
        assert_eq!(doc.field("state"), Some("in-progress"));
        assert_eq!(review_status(&doc), Some("pending"));
    }

    #[test]
    fn show_and_list_json_include_divergence_warnings() {
        let doc = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: todo\naccord:\n  status: claimed\n",
            )
            .unwrap(),
            body: String::new(),
        };

        assert!(show_json(&doc).contains("accord.status `claimed` suggests `in-progress`"));
        assert!(list_json(&[doc]).contains("accord.status `claimed` suggests `in-progress`"));
    }

    #[test]
    fn move_task_to_state_reuses_ready_to_claimed_sync() {
        let root = std::env::temp_dir().join(format!(
            "tandem-move-sync-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = Workspace {
            board_dir: root.join(".tandem/board"),
            logs_dir: root.join(".tandem/logs"),
            config_path: root.join(".tandem/tandem.md"),
            events_path: root.join(".tandem/events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\naccord:\n  status: ready\n---\n\nBody\n",
        )
        .unwrap();

        let outcome = move_task_to_state(&workspace, "task-1", "in-progress").unwrap();
        let output = fs::read_to_string(&task_path).unwrap();
        assert!(outcome.changed);
        assert_eq!(outcome.accord_sync.as_deref(), Some("ready -> claimed"));
        assert!(output.contains("state: \"in-progress\"\n"));
        assert!(output.contains("accord:\n  status: \"claimed\"\n"));
        assert!(output.contains("Body\n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn patches_completion_as_nested_metadata_and_preserves_body() {
        let input = "---\nid: task-1\ntype: task\ntitle: Demo\ncompletionSummary: old\nfilesChanged: [old.rs]\n---\n\nBody\n";
        let output = patch_completion_content(
            input,
            "Done",
            &["src/main.rs".to_string()],
            Some("cargo test passed"),
            Some("ivan"),
        )
        .unwrap();
        assert!(!output.contains("completionSummary:"));
        assert!(!output.contains("filesChanged: [old.rs]"));
        assert!(output.contains("completion:\n  summary: \"Done\"\n"));
        assert!(output.contains("  filesChanged: [\"src/main.rs\"]\n"));
        assert!(output.contains("  validation: \"cargo test passed\"\n"));
        assert!(output.contains("  reviewer: \"ivan\"\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn completion_helpers_read_nested_and_legacy_flat_metadata() {
        let nested = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "completion:\n  summary: Done\n  validation: passed\n  reviewer: ivan\n  filesChanged: [src/main.rs]\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert_eq!(completion_summary(&nested), Some("Done"));
        assert_eq!(completion_validation(&nested), Some("passed"));
        assert_eq!(completion_reviewer(&nested), Some("ivan"));
        assert_eq!(completion_files_changed(&nested), vec!["src/main.rs"]);

        let legacy = Document {
            path: PathBuf::from("task-2.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "completionSummary: Done\ncompletionValidation: passed\ncompletionReviewer: ivan\nfilesChanged: [src/lib.rs]\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert_eq!(completion_summary(&legacy), Some("Done"));
        assert_eq!(completion_validation(&legacy), Some("passed"));
        assert_eq!(completion_reviewer(&legacy), Some("ivan"));
        assert_eq!(completion_files_changed(&legacy), vec!["src/lib.rs"]);
    }

    #[test]
    fn validation_reports_invalid_review_status() {
        let doc = Document {
            path: PathBuf::from(".tandem/board/task-1.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Demo\nstate: todo\nreview:\n  status: maybe\n",
            )
            .unwrap(),
            body: String::new(),
        };
        let workspace = Workspace {
            board_dir: PathBuf::from(".tandem/board"),
            logs_dir: PathBuf::from(".tandem/logs"),
            config_path: PathBuf::from("missing-tandem.md"),
            events_path: PathBuf::from(".tandem/events.jsonl"),
        };
        let error = validate_task_document_for_mutation(&workspace, &doc).unwrap_err();
        assert!(error.message.contains("invalid review.status `maybe`"));
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }

    #[test]
    fn patches_frontmatter_without_touching_body() {
        let input = "---\nid: task-1\nstate: todo\ntitle: Old\n---\n\nBody\n";
        let mut updates = BTreeMap::new();
        updates.insert("state".to_string(), "validation".to_string());
        updates.insert("updatedAt".to_string(), "2026-06-26T00:00:00Z".to_string());
        let output = patch_frontmatter_content(input, &updates, &[]).unwrap();
        assert!(output.contains("state: \"validation\"\n"));
        assert!(output.contains("updatedAt: \"2026-06-26T00:00:00Z\"\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn validation_state_filter_accepts_legacy_review_alias() {
        assert!(state_matches_filter(Some("validation"), "validation"));
        assert!(state_matches_filter(Some("review"), "validation"));
        assert!(state_matches_filter(Some("validation"), "review"));
        assert!(!state_matches_filter(Some("todo"), "validation"));
    }

    #[test]
    fn configured_review_state_accepts_preferred_validation_writes() {
        let legacy_states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
        ];
        assert!(is_known_or_legacy_state(&legacy_states, "validation"));
        assert!(display_known_states(&legacy_states).contains("validation (preferred alias)"));

        let current_states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "validation".to_string(),
        ];
        assert!(is_known_or_legacy_state(&current_states, "review"));
        assert!(display_known_states(&current_states).contains("review (legacy alias)"));
    }

    #[test]
    fn parses_inline_arrays_for_filters() {
        let values = parse_field_values("[\"tui\", \"cli\"]");
        assert_eq!(values, vec!["tui".to_string(), "cli".to_string()]);
    }

    #[test]
    fn formats_unix_epoch_as_utc() {
        assert_eq!(format_unix_seconds(0), "1970-01-01T00:00:00Z");
    }
}
