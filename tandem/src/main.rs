use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use yaml_rust2::{Yaml, YamlLoader};

mod tui;

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
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
const DECISION_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "deprecated",
    "superseded",
];
const PRIORITIES: &[&str] = &["critical", "high", "medium", "low"];
const TASK_KINDS: &[&str] = &["epic"];
const COMPLETION_OUTCOME_COMPLETED: &str = "completed";
const COMPLETION_OUTCOME_CANCELED: &str = "canceled";
const COMPLETION_OUTCOMES: &[&str] = &[COMPLETION_OUTCOME_COMPLETED, COMPLETION_OUTCOME_CANCELED];
const DEFAULT_WORKSPACE_TITLE: &str = "Tandem Workspace";
const MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS: usize = 1000;

static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Exit code categories: 0 success, 1 runtime/data/write failure, 2 usage/argument failure.
#[derive(Debug)]
pub(crate) struct CliError {
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
pub(crate) struct Workspace {
    board_dir: PathBuf,
    logs_dir: PathBuf,
    config_path: PathBuf,
    events_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct Document {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskRole {
    Epic,
    Task,
    Subtask,
}

impl TaskRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            TaskRole::Epic => "epic",
            TaskRole::Task => "task",
            TaskRole::Subtask => "subtask",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentRelationship {
    EpicTask,
    Subtask,
    Parent,
}

impl ParentRelationship {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ParentRelationship::EpicTask => "epic-task",
            ParentRelationship::Subtask => "subtask",
            ParentRelationship::Parent => "parent",
        }
    }

    pub(crate) fn human_label(self) -> &'static str {
        match self {
            ParentRelationship::EpicTask => "Task of Epic",
            ParentRelationship::Subtask => "Subtask of",
            ParentRelationship::Parent => "Parent",
        }
    }
}

/// Canonical board+logs graph used by CLI reads, allocation, and mutation validation.
/// Kept crate-visible so the TUI can adopt the same decision-7 seam.
#[derive(Debug, Clone)]
pub(crate) struct HierarchyIndex {
    pub(crate) documents: HashMap<String, Document>,
}

impl HierarchyIndex {
    pub(crate) fn from_documents(docs: Vec<Document>) -> Result<Self, CliError> {
        let mut documents = HashMap::new();
        for doc in docs {
            let id = doc.id().to_string();
            if id.trim().is_empty() {
                return Err(CliError::user(format!(
                    "Validation failed for {}: missing required field `id`",
                    display_path(&doc.path)
                )));
            }
            if let Some(existing) = documents.insert(id.clone(), doc.clone()) {
                return Err(CliError::user(format!(
                    "Validation failed: duplicate document ID `{id}` in {} and {}",
                    display_path(&existing.path),
                    display_path(&doc.path)
                )));
            }
        }
        Ok(Self { documents })
    }

    pub(crate) fn from_workspace(workspace: &Workspace) -> Result<Self, CliError> {
        Self::from_documents(read_workspace_documents(workspace)?)
    }

    fn with_replacement(&self, doc: Document) -> Self {
        let mut documents = self.documents.clone();
        documents.insert(doc.id().to_string(), doc);
        Self { documents }
    }

    pub(crate) fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    pub(crate) fn task_role(&self, doc: &Document) -> Result<Option<TaskRole>, CliError> {
        if doc.doc_type() != "task" {
            return Ok(None);
        }
        let mut roles = HashMap::new();
        let mut stack = Vec::new();
        self.task_role_by_id(doc.id(), &mut roles, &mut stack)
            .map(Some)
    }

    fn task_role_by_id(
        &self,
        id: &str,
        roles: &mut HashMap<String, TaskRole>,
        stack: &mut Vec<String>,
    ) -> Result<TaskRole, CliError> {
        if let Some(role) = roles.get(id) {
            return Ok(*role);
        }
        if let Some(cycle_start) = stack.iter().position(|entry| entry == id) {
            let mut cycle = stack[cycle_start..].to_vec();
            cycle.push(id.to_string());
            return Err(CliError::user(format!(
                "Validation failed: task hierarchy cycle: {}",
                cycle.join(" -> ")
            )));
        }
        let doc = self.document(id).ok_or_else(|| {
            CliError::user(format!(
                "Validation failed: parent document not found: {id}"
            ))
        })?;
        if doc.doc_type() != "task" {
            return Err(CliError::user(format!(
                "Validation failed: {id} is type {}, not task",
                doc.doc_type()
            )));
        }
        if let Some(kind) = doc.field("kind") {
            validate_task_kind_value(kind).map_err(|message| {
                CliError::user(format!(
                    "Validation failed for {}: {message}",
                    display_path(&doc.path)
                ))
            })?;
        }
        if doc.kind() == Some("epic") {
            roles.insert(id.to_string(), TaskRole::Epic);
            return Ok(TaskRole::Epic);
        }

        stack.push(id.to_string());
        let role = match doc.field("parentId") {
            None => TaskRole::Task,
            Some(parent_id) => {
                let parent = self.document(parent_id).ok_or_else(|| {
                    CliError::user(format!(
                        "Validation failed for {}: unresolved parentId `{parent_id}`",
                        display_path(&doc.path)
                    ))
                })?;
                if parent.doc_type() != "task" {
                    TaskRole::Task
                } else {
                    match self.task_role_by_id(parent_id, roles, stack)? {
                        TaskRole::Epic => TaskRole::Task,
                        TaskRole::Task => TaskRole::Subtask,
                        TaskRole::Subtask => {
                            return Err(CliError::user(format!(
                                "Validation failed for {}: task {} cannot be a child of Subtask {parent_id}",
                                display_path(&doc.path),
                                doc.id()
                            )))
                        }
                    }
                }
            }
        };
        stack.pop();
        roles.insert(id.to_string(), role);
        Ok(role)
    }

    pub(crate) fn relationship(
        &self,
        doc: &Document,
    ) -> Result<Option<ParentRelationship>, CliError> {
        let Some(parent_id) = doc.field("parentId") else {
            return Ok(None);
        };
        let parent = self.document(parent_id).ok_or_else(|| {
            CliError::user(format!(
                "Validation failed for {}: unresolved parentId `{parent_id}`",
                display_path(&doc.path)
            ))
        })?;
        if doc.doc_type() != "task" || parent.doc_type() != "task" {
            return Ok(Some(ParentRelationship::Parent));
        }
        Ok(Some(match self.task_role(parent)? {
            Some(TaskRole::Epic) => ParentRelationship::EpicTask,
            Some(TaskRole::Task) => ParentRelationship::Subtask,
            Some(TaskRole::Subtask) => {
                return Err(CliError::user(format!(
                    "Validation failed for {}: task {} cannot be a child of Subtask {parent_id}",
                    display_path(&doc.path),
                    doc.id()
                )))
            }
            None => ParentRelationship::Parent,
        }))
    }

    pub(crate) fn validate_task_hierarchy(&self, doc: &Document) -> Result<TaskRole, CliError> {
        let role = self.task_role(doc)?.ok_or_else(|| {
            CliError::user(format!("Validation failed: {} is not a task", doc.id()))
        })?;
        if role == TaskRole::Epic && doc.field("parentId").is_some() {
            return Err(CliError::user(format!(
                "Validation failed for {}: Epic {} cannot have parentId",
                display_path(&doc.path),
                doc.id()
            )));
        }
        let valid_id = match role {
            TaskRole::Epic | TaskRole::Task => global_task_number(doc.id()).is_some(),
            TaskRole::Subtask => doc
                .field("parentId")
                .is_some_and(|parent_id| subtask_suffix(doc.id(), parent_id).is_some()),
        };
        if !valid_id {
            let expected = match role {
                TaskRole::Epic | TaskRole::Task => "global `task-N`".to_string(),
                TaskRole::Subtask => format!(
                    "`{}-M` with a positive M",
                    doc.field("parentId").unwrap_or("task-N")
                ),
            };
            return Err(CliError::user(format!(
                "Validation failed for {}: {} {} has invalid ID `{}`; expected {expected}",
                display_path(&doc.path),
                role.as_str(),
                doc.title(),
                doc.id()
            )));
        }
        if role == TaskRole::Subtask
            && self
                .documents
                .values()
                .any(|child| child.field("parentId") == Some(doc.id()))
        {
            return Err(CliError::user(format!(
                "Validation failed for {}: Subtask {} cannot have children",
                display_path(&doc.path),
                doc.id()
            )));
        }
        Ok(role)
    }

    pub(crate) fn task_hierarchy_errors(&self) -> Vec<String> {
        let mut ids = self
            .documents
            .values()
            .filter(|doc| doc.doc_type() == "task")
            .map(|doc| doc.id().to_string())
            .collect::<Vec<_>>();
        ids.sort();
        let mut errors = BTreeSet::new();
        for id in ids {
            if let Err(error) =
                self.validate_task_hierarchy(self.document(&id).expect("indexed task"))
            {
                errors.insert(error.message);
            }
        }
        errors.into_iter().collect()
    }

    pub(crate) fn validate_all_task_hierarchies(&self) -> Result<(), CliError> {
        let errors = self.task_hierarchy_errors();
        if errors.is_empty() {
            return Ok(());
        }
        if errors.len() == 1 {
            return Err(CliError::user(errors.into_iter().next().unwrap()));
        }
        let count = errors.len();
        Err(CliError::user(format!(
            "Validation failed: hierarchy contains {count} structural errors:\n- {}",
            errors.join("\n- ")
        )))
    }
}

/// Serializes cooperative CLI hierarchy snapshots and mutations on the workspace config inode.
pub(crate) struct HierarchyLock {
    file: File,
}

impl HierarchyLock {
    pub(crate) fn acquire(workspace: &Workspace) -> Result<Self, CliError> {
        let path = workspace.config_path.clone();
        let file = OpenOptions::new().read(true).open(&path).map_err(|error| {
            CliError::user(format!(
                "Write failure: could not open hierarchy lock {}: {error}",
                display_path(&path)
            ))
        })?;
        file.lock().map_err(|error| {
            CliError::user(format!(
                "Write failure: could not lock hierarchy snapshot {}: {error}",
                display_path(&path)
            ))
        })?;
        Ok(Self { file })
    }
}

impl Drop for HierarchyLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
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

    fn kind(&self) -> Option<&str> {
        self.field("kind")
            .map(str::trim)
            .filter(|value| !value.is_empty())
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
    parent: Option<String>,
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
    json: bool,
    description: Option<String>,
    kind: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    assignee: Option<String>,
    due_date: Option<String>,
    parent: Option<String>,
    blockers: Vec<String>,
    references: Vec<String>,
    related_files: Vec<String>,
}

#[derive(Debug)]
struct AddOutcome {
    id: String,
    state: String,
    title: String,
    kind: Option<String>,
    parent: Option<String>,
    parent_relationship: Option<ParentRelationship>,
    path: PathBuf,
    warnings: Vec<String>,
}

#[derive(Debug, Default)]
struct MoveOptions {
    id: String,
    state: Option<String>,
}

#[derive(Debug, Default)]
struct UpdateOptions {
    id: String,
    title: Option<String>,
    body: Option<String>,
    kind: Option<String>,
    priority: Option<String>,
    assignee: Option<String>,
    due_date: Option<String>,
    parent: Option<String>,
    tags: Vec<String>,
    blockers: Vec<String>,
    references: Vec<String>,
    related_files: Vec<String>,
}

#[derive(Debug, Clone)]
struct UpdateChange {
    field: String,
    old: String,
    new: String,
}

#[derive(Debug)]
struct UpdateOutcome {
    id: String,
    path: PathBuf,
    changes: Vec<UpdateChange>,
    warnings: Vec<String>,
    parent_relationship: Option<ParentRelationship>,
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
struct CancelOptions {
    id: String,
    reason: Option<String>,
}

#[derive(Debug)]
struct CancelOutcome {
    id: String,
    reason: String,
    board_path: PathBuf,
    log_path: PathBuf,
}

#[derive(Debug, Default)]
struct SearchOptions {
    query: String,
    state: Option<String>,
    doc_type: Option<String>,
    parent: Option<String>,
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
        "update" => cmd_update(parse_update_args(&args)?)?,
        "complete" => cmd_complete(parse_complete_args(&args)?)?,
        "cancel" => cmd_cancel(parse_cancel_args(&args)?)?,
        "search" => cmd_search(parse_search_args(&args)?)?,
        "log" => cmd_log(&args)?,
        "accord" => cmd_accord(&args)?,
        "rules" => cmd_rules(&args)?,
        "decision" => cmd_decision(&args)?,
        "tui" => cmd_tui(&args)?,
        "version" | "--version" => print_version(),
        "help" | "--help" => print_help(),
        other => {
            return Err(CliError::usage(format!(
                "unknown command `{other}`. Supported commands: init, list, show, add, move, update, complete, cancel, search, log, accord, rules, decision, tui, version"
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
    println!("  tandem list [--state <state>] [--type <type>] [--parent <id>] [--json]");
    println!("  tandem show <id> [--json]");
    println!("  tandem add --title <title> [--state <state>] [--kind epic] [--parent <id>] [--description <text>] [--json]");
    println!("  tandem move <id> --state <state>");
    println!("  tandem update <id> [--title <title>] [--body <markdown>] [--kind epic] [--parent <id>] [--priority <priority>] ...");
    println!("  tandem complete <id> --summary <text>");
    println!("  tandem cancel <id> --reason <text>");
    println!("  tandem search <query> [--state <state>] [--type <type>] [--parent <id>] [--json]");
    println!("  tandem log list|show|search ...");
    println!("  tandem accord ready|claim|deliver|accept|rework|block|fail ...");
    println!("  tandem rules list|add|edit|delete ...");
    println!("  tandem decision list|show|add ... [--status <status>] [--date <date>]");
    println!("  tandem tui");
    println!("  tandem version");
    println!("  tandem --version");
}

fn print_version() {
    println!("{}", version_text());
}

fn version_text() -> String {
    format!("tandem {PACKAGE_VERSION}")
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
            "--parent" => {
                index += 1;
                options.parent = Some(required_value(args, index, "--parent")?.to_string());
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
            "--json" => options.json = true,
            "--description" => {
                index += 1;
                options.description =
                    Some(required_value(args, index, "--description")?.to_string());
            }
            "--kind" => {
                index += 1;
                options.kind = Some(required_value(args, index, "--kind")?.to_string());
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
                return Err(inline_subtask_authoring_error("add"));
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

fn parse_update_args(args: &[String]) -> Result<UpdateOptions, CliError> {
    let mut options = UpdateOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--title" => {
                index += 1;
                options.title = Some(required_value(args, index, "--title")?.to_string());
            }
            "--body" => {
                index += 1;
                options.body = Some(required_raw_value(args, index, "--body")?.to_string());
            }
            "--kind" => {
                index += 1;
                options.kind = Some(required_value(args, index, "--kind")?.to_string());
            }
            "--priority" => {
                index += 1;
                options.priority = Some(required_value(args, index, "--priority")?.to_string());
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
            "--tag" => {
                index += 1;
                options
                    .tags
                    .push(required_value(args, index, "--tag")?.to_string());
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
            "--state" => {
                return Err(CliError::usage(
                    "update does not support --state; use `tandem move <id> --state <state>`",
                ))
            }
            "--parent-id" | "--parentId" => {
                return Err(CliError::usage(
                    "update uses the canonical --parent <id> flag",
                ))
            }
            "--subtask" => return Err(inline_subtask_authoring_error("update")),
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown update flag `{flag}`")))
            }
            value => set_single_positional(&mut options.id, value, "update")?,
        }
        index += 1;
    }
    if options.id.is_empty() {
        return Err(CliError::usage("update requires an <id>"));
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

fn parse_cancel_args(args: &[String]) -> Result<CancelOptions, CliError> {
    let mut options = CancelOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--reason" => {
                index += 1;
                options.reason = Some(required_raw_value(args, index, "--reason")?.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::usage(format!("unknown cancel flag `{flag}`")))
            }
            value => set_single_positional(&mut options.id, value, "cancel")?,
        }
        index += 1;
    }
    if options.id.is_empty() {
        return Err(CliError::usage("cancel requires an <id>"));
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
            "--parent" => {
                index += 1;
                options.parent = Some(required_value(args, index, "--parent")?.to_string());
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

fn required_raw_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn inline_subtask_authoring_error(command: &str) -> CliError {
    CliError::usage(format!(
        "{command} --subtask is deprecated; create a tracked subtask with `tandem add --title <title> --parent <task-id>`"
    ))
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
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let docs = hierarchy
        .documents
        .values()
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .collect::<Vec<_>>();
    let mut filtered = filter_documents(docs, &options);
    sort_documents(&mut filtered);

    if options.json {
        println!("{}", list_json(&filtered, &hierarchy)?);
    } else {
        print_list_table(&filtered, &hierarchy)?;
        print_document_warnings(&filtered);
    }

    Ok(())
}

fn cmd_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(&options.id)
        .cloned()
        .ok_or_else(|| CliError::user(format!("document not found: {}", options.id)))?;
    let relationship = hierarchy.relationship(&doc)?;
    let children = find_hierarchy_children(&hierarchy, &doc)?;
    let role = hierarchy.task_role(&doc)?;

    if options.json {
        println!("{}", show_json(&doc, &children, role, relationship));
    } else {
        print_show(&doc, &children, role, relationship);
    }

    Ok(())
}

fn cmd_add(options: AddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let json = options.json;
    let outcome = add_task(&workspace, options)?;

    if json {
        println!("{}", add_outcome_json(&outcome));
        return Ok(());
    }

    for warning in &outcome.warnings {
        println!("Warning: {warning}");
    }
    if outcome.parent_relationship == Some(ParentRelationship::Subtask) {
        println!("Created subtask");
    } else {
        println!("Created task");
    }
    println!("ID:    {}", outcome.id);
    println!("State: {}", outcome.state);
    if let Some(kind) = outcome.kind.as_deref() {
        println!("Kind:  {kind}");
    }
    if let Some(parent) = outcome.parent.as_deref() {
        let label = outcome
            .parent_relationship
            .unwrap_or(ParentRelationship::Parent)
            .human_label();
        println!("{label}: {parent}");
    }
    println!("Title: {}", outcome.title);
    println!("Path:  {}", display_path(&outcome.path));
    Ok(())
}

fn add_task(workspace: &Workspace, options: AddOptions) -> Result<AddOutcome, CliError> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    let title =
        require_nonempty(options.title.as_deref(), "add requires --title <title>")?.to_string();
    let state = options.state.as_deref().unwrap_or("todo").to_string();
    validate_state(workspace, &state)?;
    validate_task_kind_option(options.kind.as_deref(), "add --kind")?;
    let kind = options
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if kind.as_deref() == Some("epic") && options.parent.is_some() {
        return Err(CliError::user(
            "Validation failed: an Epic cannot have parentId; remove --parent or --kind epic",
        ));
    }
    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let parent_relationship = options
        .parent
        .as_deref()
        .map(|parent| resolve_parent_relationship(&hierarchy, "task", parent))
        .transpose()?;
    for blocker in &options.blockers {
        if hierarchy.document(blocker).is_none() {
            return Err(CliError::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }

    let mut warnings = Vec::new();
    for reference in &options.references {
        if hierarchy.document(reference).is_none() {
            warnings.push(format!("reference not found: {reference}"));
        }
    }

    let allocation_prefix = match (parent_relationship, options.parent.as_deref()) {
        (Some(ParentRelationship::Subtask), Some(parent)) => parent,
        _ => "task",
    };
    let now = current_timestamp();
    let last_allocated = next_sequential_number_in_hierarchy(&hierarchy, allocation_prefix);
    let created = create_new_sequential_document_after(
        workspace,
        allocation_prefix,
        last_allocated,
        |task_id| {
            let mut lines = vec![
                "---".to_string(),
                format!("id: {task_id}"),
                "type: task".to_string(),
            ];
            push_optional_line(&mut lines, "kind", kind.as_deref());
            lines.push(format!("title: {}", yaml_double_quote(&title)));
            lines.push(format!("state: {state}"));
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
            lines.push("---".to_string());
            lines.push(String::new());
            if let Some(description) = options.description.as_deref() {
                lines.push("## Description".to_string());
                lines.push(String::new());
                lines.push(description.to_string());
            }
            lines.push(String::new());
            lines.join("\n")
        },
    )?;
    append_event(workspace, "task.created", &created.id, &title)?;

    Ok(AddOutcome {
        id: created.id,
        state,
        title,
        kind,
        parent: options.parent,
        parent_relationship,
        path: created.path,
        warnings,
    })
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

fn cmd_update(options: UpdateOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = update_task_metadata(&workspace, options)?;

    for warning in outcome.warnings {
        println!("Warning: {warning}");
    }
    if outcome.changes.is_empty() {
        println!("No changes for {}", outcome.id);
        println!("Path: {}", display_path(&outcome.path));
        return Ok(());
    }

    println!("Updated {}", outcome.id);
    for change in outcome.changes {
        if change.field == "body" {
            println!("body: changed");
        } else {
            println!(
                "{}: {} -> {}",
                display_change_field(&change.field, outcome.parent_relationship),
                display_change_value(&change.old),
                display_change_value(&change.new)
            );
        }
    }
    println!("Path: {}", display_path(&outcome.path));
    Ok(())
}

fn cmd_complete(options: CompleteOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let summary = require_nonempty(
        options.summary.as_deref(),
        "complete requires --summary <text>",
    )?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be completed in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(&workspace, &doc, &hierarchy)?;
    let unresolved = unresolved_blockers_in_hierarchy(&hierarchy, doc.field("blockers"));
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
        None,
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

fn cmd_cancel(options: CancelOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let reason = require_nonempty(options.reason.as_deref(), "cancel requires --reason <text>")?;
    let outcome = cancel_task(&workspace, &options.id, reason)?;

    println!("Canceled {}", outcome.id);
    println!("Reason: {}", outcome.reason);
    println!(
        "Moved: {} -> {}",
        display_path(&outcome.board_path),
        display_path(&outcome.log_path)
    );
    println!("Event: task.canceled");
    Ok(())
}

fn cancel_task(workspace: &Workspace, id: &str, reason: &str) -> Result<CancelOutcome, CliError> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    let reason = require_nonempty(Some(reason), "cancel requires --reason <text>")?.to_string();
    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only active task documents can be canceled: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

    let active_descendants = active_task_descendant_ids(&hierarchy, doc.id());
    if !active_descendants.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: cannot cancel {} while it has active descendants: {}",
            doc.id(),
            active_descendants.join(", ")
        )));
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
    let summary = format!("Canceled: {reason}");
    let patched = patch_completion_content(
        &patched,
        &summary,
        Some(COMPLETION_OUTCOME_CANCELED),
        &[],
        None,
        None,
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
            "Write failure: could not remove active document {} after writing canceled log {}: {error}",
            display_path(&doc.path),
            display_path(&log_path)
        ))
    })?;
    append_event(workspace, "task.canceled", doc.id(), &summary)?;

    Ok(CancelOutcome {
        id: doc.id().to_string(),
        reason,
        board_path: doc.path,
        log_path,
    })
}

fn cmd_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let docs = hierarchy.documents.values().cloned().collect::<Vec<_>>();
    let results = search_documents(docs, &options);

    if options.json {
        println!("{}", search_json(&options.query, &results, &hierarchy)?);
    } else {
        print_search_table(&results, &hierarchy)?;
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
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let mut docs = hierarchy
        .documents
        .values()
        .filter(|doc| doc.location == DocumentLocation::Logs)
        .cloned()
        .collect::<Vec<_>>();
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
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Logs)
        .cloned()
        .ok_or_else(|| CliError::user(format!("log document not found: {}", options.id)))?;
    let relationship = hierarchy.relationship(&doc)?;
    if options.json {
        println!("{}", log_show_json(&doc, relationship));
    } else {
        print_log_show(&doc, relationship);
    }
    Ok(())
}

fn cmd_log_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let mut results = hierarchy
        .documents
        .values()
        .filter(|doc| doc.location == DocumentLocation::Logs)
        .cloned()
        .filter_map(|doc| search_match(doc, &options.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));
    if options.json {
        println!("{}", search_json(&options.query, &results, &hierarchy)?);
    } else {
        print_search_table(&results, &hierarchy)?;
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
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can have accord actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(&workspace, &doc, &hierarchy)?;

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
    status: Option<String>,
    date: Option<String>,
    deciders: Vec<String>,
    context: Option<String>,
    consequences: Vec<String>,
    alternatives: Vec<String>,
    supersedes: Vec<String>,
    superseded_by: Vec<String>,
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
            "--status" => {
                index += 1;
                options.status = Some(required_value(args, index, "--status")?.to_string());
            }
            "--date" => {
                index += 1;
                options.date = Some(required_value(args, index, "--date")?.to_string());
            }
            "--decider" => {
                index += 1;
                options
                    .deciders
                    .push(required_value(args, index, "--decider")?.to_string());
            }
            "--context" => {
                index += 1;
                options.context = Some(required_value(args, index, "--context")?.to_string());
            }
            "--consequence" => {
                index += 1;
                options
                    .consequences
                    .push(required_value(args, index, "--consequence")?.to_string());
            }
            "--alternative" => {
                index += 1;
                options
                    .alternatives
                    .push(required_value(args, index, "--alternative")?.to_string());
            }
            "--supersedes" => {
                index += 1;
                options
                    .supersedes
                    .push(required_value(args, index, "--supersedes")?.to_string());
            }
            "--superseded-by" => {
                index += 1;
                options
                    .superseded_by
                    .push(required_value(args, index, "--superseded-by")?.to_string());
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
    let _hierarchy_lock = HierarchyLock::acquire(&workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(&workspace)?;
    hierarchy.validate_all_task_hierarchies()?;
    let doc = hierarchy
        .document(&options.id)
        .cloned()
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
        print_show(&doc, &[], None, hierarchy.relationship(&doc)?);
    }
    Ok(())
}

fn cmd_decision_add(options: DecisionAddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let title = require_nonempty(
        options.title.as_deref(),
        "decision add requires --title <title>",
    )?;
    let status = options.status.as_deref().unwrap_or("proposed");
    validate_decision_status(status)?;
    validate_decision_add_options(&options)?;
    let warnings = decision_add_warnings(&workspace, &options)?;

    let now = current_timestamp();
    let date = match options.date.as_deref() {
        Some(date) => {
            require_nonempty(Some(date), "decision add --date must not be empty")?.to_string()
        }
        None => date_from_timestamp(&now),
    };
    let created = create_new_sequential_document(&workspace, "decision", |decision_id| {
        let mut lines = vec![
            "---".to_string(),
            format!("id: {decision_id}"),
            "type: decision".to_string(),
            format!("title: {}", yaml_double_quote(title)),
            format!("status: {}", yaml_double_quote(status)),
            format!("date: {}", yaml_double_quote(&date)),
        ];
        push_array_line(&mut lines, "deciders", &options.deciders);
        push_optional_line(&mut lines, "context", options.context.as_deref());
        push_array_line(&mut lines, "consequences", &options.consequences);
        push_array_line(&mut lines, "alternatives", &options.alternatives);
        push_array_line(&mut lines, "supersedes", &options.supersedes);
        push_array_line(&mut lines, "supersededBy", &options.superseded_by);
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
        lines.join("\n")
    })?;
    append_event(&workspace, "decision.created", &created.id, title)?;

    for warning in warnings {
        println!("Warning: {warning}");
    }
    println!("Created decision");
    println!("ID:     {}", created.id);
    println!("Status: {status}");
    println!("Date:   {date}");
    println!("Title:  {title}");
    println!("Path:   {}", display_path(&created.path));
    Ok(())
}

fn validate_decision_add_options(options: &DecisionAddOptions) -> Result<(), CliError> {
    if let Some(context) = options.context.as_deref() {
        require_nonempty(Some(context), "decision add --context must not be empty")?;
    }
    for (flag, values) in [
        ("--decider", &options.deciders),
        ("--consequence", &options.consequences),
        ("--alternative", &options.alternatives),
        ("--supersedes", &options.supersedes),
        ("--superseded-by", &options.superseded_by),
        ("--reference", &options.references),
        ("--tag", &options.tags),
    ] {
        for value in values {
            require_nonempty(
                Some(value),
                &format!("decision add {flag} must not be empty"),
            )?;
        }
    }
    Ok(())
}

fn validate_decision_status(status: &str) -> Result<(), CliError> {
    let status = require_nonempty(Some(status), "decision add --status must not be empty")?;
    if DECISION_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: invalid decision status `{status}`; expected one of: {}",
            DECISION_STATUSES.join(", ")
        )))
    }
}

fn decision_add_warnings(
    workspace: &Workspace,
    options: &DecisionAddOptions,
) -> Result<Vec<String>, CliError> {
    let mut warnings = Vec::new();
    for reference in &options.references {
        if !document_exists(workspace, reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }
    for target in &options.supersedes {
        push_decision_reference_warning(workspace, &mut warnings, "supersedes", target)?;
    }
    for target in &options.superseded_by {
        push_decision_reference_warning(workspace, &mut warnings, "supersededBy", target)?;
    }
    Ok(warnings)
}

fn push_decision_reference_warning(
    workspace: &Workspace,
    warnings: &mut Vec<String>,
    field: &str,
    id: &str,
) -> Result<(), CliError> {
    match find_document(workspace, id)? {
        Some(doc) if doc.doc_type() == "decision" => {}
        Some(doc) => warnings.push(format!(
            "{field} target {id} is type {}, not decision",
            doc.doc_type()
        )),
        None => warnings.push(format!("{field} decision not found: {id}")),
    }
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

fn read_workspace_documents(workspace: &Workspace) -> Result<Vec<Document>, CliError> {
    let mut docs = read_documents(&workspace.board_dir, DocumentLocation::Board)?;
    docs.extend(read_documents(&workspace.logs_dir, DocumentLocation::Logs)?);
    Ok(docs)
}

fn find_hierarchy_children(
    hierarchy: &HierarchyIndex,
    parent: &Document,
) -> Result<Vec<Document>, CliError> {
    let Some(parent_role) = hierarchy.task_role(parent)? else {
        return Ok(Vec::new());
    };
    if parent_role == TaskRole::Subtask {
        return Ok(Vec::new());
    }
    let expected_child_role = match parent_role {
        TaskRole::Epic => TaskRole::Task,
        TaskRole::Task => TaskRole::Subtask,
        TaskRole::Subtask => unreachable!(),
    };
    let mut children = hierarchy
        .documents
        .values()
        .filter(|doc| doc.doc_type() == "task" && doc.field("parentId") == Some(parent.id()))
        .filter_map(|doc| match hierarchy.task_role(doc) {
            Ok(Some(role)) if role == expected_child_role => Some(Ok(doc.clone())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort_by(|a, b| {
        a.location
            .as_str()
            .cmp(b.location.as_str())
            .then_with(|| {
                a.field("state")
                    .unwrap_or("")
                    .cmp(b.field("state").unwrap_or(""))
            })
            .then_with(|| a.id().cmp(b.id()))
    });
    Ok(children)
}

fn active_task_descendant_ids(hierarchy: &HierarchyIndex, root_id: &str) -> Vec<String> {
    let mut visited = BTreeSet::from([root_id.to_string()]);
    let mut pending = vec![root_id.to_string()];
    let mut active = BTreeSet::new();

    while let Some(parent_id) = pending.pop() {
        for doc in hierarchy
            .documents
            .values()
            .filter(|doc| doc.doc_type() == "task")
            .filter(|doc| doc.field("parentId") == Some(parent_id.as_str()))
        {
            if !visited.insert(doc.id().to_string()) {
                continue;
            }
            if doc.location == DocumentLocation::Board {
                active.insert(doc.id().to_string());
            }
            pending.push(doc.id().to_string());
        }
    }

    active.into_iter().collect()
}

fn find_board_document(workspace: &Workspace, id: &str) -> Result<Option<Document>, CliError> {
    Ok(
        read_documents(&workspace.board_dir, DocumentLocation::Board)?
            .into_iter()
            .find(|doc| doc.id() == id),
    )
}

fn document_exists(workspace: &Workspace, id: &str) -> Result<bool, CliError> {
    Ok(find_document(workspace, id)?.is_some())
}

fn resolve_parent_relationship(
    hierarchy: &HierarchyIndex,
    child_type: &str,
    parent_id: &str,
) -> Result<ParentRelationship, CliError> {
    let parent = hierarchy.document(parent_id).ok_or_else(|| {
        CliError::user(format!(
            "Validation failed: parent document not found: {parent_id}"
        ))
    })?;
    if child_type != "task" || parent.doc_type() != "task" {
        return Ok(ParentRelationship::Parent);
    }
    match hierarchy.task_role(parent)? {
        Some(TaskRole::Epic) => Ok(ParentRelationship::EpicTask),
        Some(TaskRole::Task) => Ok(ParentRelationship::Subtask),
        Some(TaskRole::Subtask) => Err(CliError::user(format!(
            "Validation failed: cannot attach a child beneath Subtask {parent_id}"
        ))),
        None => Ok(ParentRelationship::Parent),
    }
}

fn positive_canonical_number(value: &str) -> Option<usize> {
    let number = value.parse::<usize>().ok()?;
    (number > 0 && number.to_string() == value).then_some(number)
}

fn global_task_number(id: &str) -> Option<usize> {
    let suffix = id.strip_prefix("task-")?;
    if suffix.contains('-') {
        return None;
    }
    positive_canonical_number(suffix)
}

fn subtask_suffix(id: &str, parent_id: &str) -> Option<usize> {
    global_task_number(parent_id)?;
    let suffix = id.strip_prefix(&format!("{parent_id}-"))?;
    if suffix.contains('-') {
        return None;
    }
    positive_canonical_number(suffix)
}

fn unresolved_blockers(
    workspace: &Workspace,
    blockers: Option<&str>,
) -> Result<Vec<String>, CliError> {
    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    Ok(unresolved_blockers_in_hierarchy(&hierarchy, blockers))
}

fn unresolved_blockers_in_hierarchy(
    hierarchy: &HierarchyIndex,
    blockers: Option<&str>,
) -> Vec<String> {
    let mut unresolved = Vec::new();
    for blocker in blockers.map(parse_field_values).unwrap_or_default() {
        match hierarchy.document(&blocker) {
            Some(doc) if doc.location == DocumentLocation::Board => unresolved.push(blocker),
            Some(_) => {}
            None => unresolved.push(format!("{blocker} (missing)")),
        }
    }
    unresolved
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
                .parent
                .as_deref()
                .is_none_or(|parent| doc.field("parentId") == Some(parent))
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
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    validate_state(workspace, state)?;

    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    let doc = hierarchy
        .document(id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be moved in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;

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

fn update_task_metadata(
    workspace: &Workspace,
    options: UpdateOptions,
) -> Result<UpdateOutcome, CliError> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    let doc = hierarchy
        .document(&options.id)
        .filter(|doc| doc.location == DocumentLocation::Board)
        .cloned()
        .ok_or_else(|| CliError::user(format!("active task not found: {}", options.id)))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be updated in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_against_hierarchy(workspace, &doc, &hierarchy)?;
    validate_update_options(&options, &hierarchy)?;

    hierarchy.validate_all_task_hierarchies()?;
    let old_role = hierarchy
        .task_role(&doc)?
        .expect("active task has a task role");
    let mut prospective = doc.clone();
    if let Some(kind) = options.kind.as_deref() {
        prospective
            .fields
            .insert("kind".to_string(), kind.to_string());
    }
    if let Some(parent) = options.parent.as_deref() {
        prospective
            .fields
            .insert("parentId".to_string(), parent.to_string());
    }
    let prospective_hierarchy = hierarchy.with_replacement(prospective.clone());
    let prospective_role = prospective_hierarchy
        .task_role(&prospective)?
        .expect("prospective task has a task role");
    if options.parent.is_some() && old_role != prospective_role {
        return Err(CliError::user(format!(
            "Validation failed: reparenting {} would change its canonical role from {} to {}; IDs are immutable",
            doc.id(),
            old_role.as_str(),
            prospective_role.as_str()
        )));
    }
    prospective_hierarchy.validate_all_task_hierarchies()?;
    let parent_relationship = if options.parent.is_some() {
        prospective_hierarchy.relationship(&prospective)?
    } else {
        None
    };

    let mut warnings = Vec::new();
    for reference in &options.references {
        if hierarchy.document(reference).is_none() {
            warnings.push(format!("reference not found: {reference}"));
        }
    }

    let mut updates = BTreeMap::new();
    let mut changes = Vec::new();
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "title",
        options.title.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "kind",
        options.kind.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "priority",
        options.priority.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "assignee",
        options.assignee.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "dueDate",
        options.due_date.as_deref(),
    )?;
    apply_scalar_update(
        &mut updates,
        &mut changes,
        &doc,
        "parentId",
        options.parent.as_deref(),
    )?;
    apply_list_append_update(&mut updates, &mut changes, &doc, "tags", &options.tags);
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "blockers",
        &options.blockers,
    );
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "references",
        &options.references,
    );
    apply_list_append_update(
        &mut updates,
        &mut changes,
        &doc,
        "relatedFiles",
        &options.related_files,
    );
    let replacement_body = options
        .body
        .as_deref()
        .filter(|body| doc.body.as_str() != *body);
    if replacement_body.is_some() {
        changes.push(UpdateChange {
            field: "body".to_string(),
            old: "<body>".to_string(),
            new: "<body>".to_string(),
        });
    }

    let doc_id = doc.id().to_string();
    let path = doc.path.clone();
    if changes.is_empty() {
        return Ok(UpdateOutcome {
            id: doc_id,
            path,
            changes,
            warnings,
            parent_relationship,
        });
    }

    updates.insert("updatedAt".to_string(), current_timestamp());
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    let patched = if let Some(body) = replacement_body {
        replace_markdown_body(&patched, body)?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "task.updated",
        &doc_id,
        &format!(
            "Updated {} metadata: {}",
            doc_id,
            changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )?;

    Ok(UpdateOutcome {
        id: doc_id,
        path,
        changes,
        warnings,
        parent_relationship,
    })
}

fn validate_update_options(
    options: &UpdateOptions,
    hierarchy: &HierarchyIndex,
) -> Result<Option<ParentRelationship>, CliError> {
    if let Some(title) = options.title.as_deref() {
        require_nonempty(Some(title), "update --title must not be empty")?;
    }
    validate_task_kind_option(options.kind.as_deref(), "update --kind")?;
    if let Some(priority) = options.priority.as_deref() {
        let priority = require_nonempty(Some(priority), "update --priority must not be empty")?;
        if !PRIORITIES.contains(&priority) {
            return Err(CliError::user(format!(
                "Validation failed: invalid priority `{priority}`; expected one of: {}",
                PRIORITIES.join(", ")
            )));
        }
    }
    if let Some(assignee) = options.assignee.as_deref() {
        require_nonempty(Some(assignee), "update --assignee must not be empty")?;
    }
    if let Some(due_date) = options.due_date.as_deref() {
        require_nonempty(Some(due_date), "update --due-date must not be empty")?;
    }
    let parent_relationship = if let Some(parent) = options.parent.as_deref() {
        let parent = require_nonempty(Some(parent), "update --parent must not be empty")?;
        if parent == options.id {
            return Err(CliError::user(format!(
                "Validation failed: task {} cannot be its own parent",
                options.id
            )));
        }
        hierarchy.validate_all_task_hierarchies()?;
        Some(resolve_parent_relationship(hierarchy, "task", parent)?)
    } else {
        None
    };
    for (field, values) in [
        ("--tag", &options.tags),
        ("--blocker", &options.blockers),
        ("--reference", &options.references),
        ("--related-file", &options.related_files),
    ] {
        for value in values {
            require_nonempty(Some(value), &format!("update {field} must not be empty"))?;
        }
    }
    for blocker in &options.blockers {
        if hierarchy.document(blocker).is_none() {
            return Err(CliError::user(format!(
                "Validation failed: blocker document not found: {blocker}"
            )));
        }
    }
    Ok(parent_relationship)
}

fn validate_task_kind_option(kind: Option<&str>, flag: &str) -> Result<(), CliError> {
    let Some(kind) = kind else {
        return Ok(());
    };
    let kind = require_nonempty(Some(kind), &format!("{flag} must not be empty"))?;
    validate_task_kind_value(kind)
        .map_err(|message| CliError::user(format!("Validation failed: {message}")))
}

fn validate_task_kind_value(kind: &str) -> Result<(), String> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err("kind must not be empty when present".to_string());
    }
    if !TASK_KINDS.contains(&kind) {
        return Err(format!(
            "invalid kind `{kind}`; expected one of: {}",
            TASK_KINDS.join(", ")
        ));
    }
    Ok(())
}

fn apply_scalar_update(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &Document,
    key: &str,
    value: Option<&str>,
) -> Result<(), CliError> {
    let Some(value) = value else {
        return Ok(());
    };
    let value = require_nonempty(Some(value), &format!("update --{key} must not be empty"))?;
    let old = doc.field(key).unwrap_or("");
    if old != value {
        updates.insert(key.to_string(), value.to_string());
        changes.push(UpdateChange {
            field: key.to_string(),
            old: old.to_string(),
            new: value.to_string(),
        });
    }
    Ok(())
}

fn apply_list_append_update(
    updates: &mut BTreeMap<String, String>,
    changes: &mut Vec<UpdateChange>,
    doc: &Document,
    key: &str,
    additions: &[String],
) {
    if additions.is_empty() {
        return;
    }
    let old_values = doc.field(key).map(parse_field_values).unwrap_or_default();
    let mut new_values = old_values.clone();
    for addition in additions {
        if !new_values.iter().any(|value| value == addition) {
            new_values.push(addition.to_string());
        }
    }
    if new_values != old_values {
        updates.insert(key.to_string(), inline_array(&new_values));
        changes.push(UpdateChange {
            field: key.to_string(),
            old: display_list_value(&old_values),
            new: display_list_value(&new_values),
        });
    }
}

fn display_list_value(values: &[String]) -> String {
    if values.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}

fn display_change_field(field: &str, relationship: Option<ParentRelationship>) -> &str {
    match field {
        "parentId" => relationship
            .unwrap_or(ParentRelationship::Parent)
            .human_label(),
        _ => field,
    }
}

fn display_change_value(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
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

fn completion_outcome(doc: &Document) -> &str {
    doc.field("completion.outcome")
        .unwrap_or(COMPLETION_OUTCOME_COMPLETED)
}

fn is_canceled_log(doc: &Document) -> bool {
    doc.location == DocumentLocation::Logs && completion_outcome(doc) == COMPLETION_OUTCOME_CANCELED
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

fn decision_status(doc: &Document) -> Option<&str> {
    doc.field("status")
}

fn decision_date(doc: &Document) -> Option<&str> {
    doc.field("date")
}

fn decision_context(doc: &Document) -> Option<&str> {
    doc.field("context")
}

fn decision_deciders(doc: &Document) -> Vec<String> {
    decision_values(doc, "deciders")
}

fn decision_consequences(doc: &Document) -> Vec<String> {
    decision_values(doc, "consequences")
}

fn decision_alternatives(doc: &Document) -> Vec<String> {
    decision_values(doc, "alternatives")
}

fn decision_supersedes(doc: &Document) -> Vec<String> {
    decision_values(doc, "supersedes")
}

fn decision_superseded_by(doc: &Document) -> Vec<String> {
    decision_values(doc, "supersededBy")
}

fn decision_values(doc: &Document, key: &str) -> Vec<String> {
    doc.field(key).map(parse_field_values).unwrap_or_default()
}

fn validate_task_document_for_mutation(
    workspace: &Workspace,
    doc: &Document,
) -> Result<(), CliError> {
    let hierarchy = HierarchyIndex::from_workspace(workspace)?.with_replacement(doc.clone());
    validate_task_document_against_hierarchy(workspace, doc, &hierarchy)
}

fn validate_task_document_against_hierarchy(
    workspace: &Workspace,
    doc: &Document,
    hierarchy: &HierarchyIndex,
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
    if let Some(kind) = doc.field("kind") {
        if let Err(message) = validate_task_kind_value(kind) {
            errors.push(message);
        }
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
        if hierarchy.document(parent).is_none() {
            errors.push(format!("unresolved parentId `{parent}`"));
        }
    }
    for blocker in doc
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default()
    {
        if hierarchy.document(&blocker).is_none() {
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

    if !errors.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed for {}: {}",
            display_path(&doc.path),
            errors.join("; ")
        )));
    }

    hierarchy.validate_all_task_hierarchies()
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

fn parent_table_values<'a>(
    doc: &'a Document,
    hierarchy: &HierarchyIndex,
) -> Result<(&'static str, &'a str), CliError> {
    let Some(parent_id) = doc.field("parentId") else {
        return Ok(("-", "-"));
    };
    let relationship = hierarchy
        .relationship(doc)?
        .unwrap_or(ParentRelationship::Parent)
        .as_str();
    Ok((relationship, parent_id))
}

fn print_list_table(docs: &[Document], hierarchy: &HierarchyIndex) -> Result<(), CliError> {
    if docs.is_empty() {
        println!("No active Tandem documents found.");
        return Ok(());
    }

    println!(
        "{:<12} {:<12} {:<8} {:<8} {:<9} {:<12} {:<32} {:<12}",
        "ID", "STATE", "TYPE", "KIND", "RELATION", "PARENT", "TITLE", "ASSIGNEE"
    );
    for doc in docs {
        let (relationship, parent_id) = parent_table_values(doc, hierarchy)?;
        println!(
            "{:<12} {:<12} {:<8} {:<8} {:<9} {:<12} {:<32} {:<12}",
            truncate(doc.id(), 12),
            truncate(doc.field("state").unwrap_or("-"), 12),
            truncate(doc.doc_type(), 8),
            truncate(doc.kind().unwrap_or("-"), 8),
            relationship,
            truncate(parent_id, 12),
            truncate(doc.title(), 32),
            truncate(doc.field("assignee").unwrap_or("-"), 12)
        );
    }
    Ok(())
}

fn print_document_warnings(docs: &[Document]) {
    for warning in docs.iter().flat_map(document_warnings) {
        println!("Warning: {warning}");
    }
}

fn print_decision_metadata(doc: &Document) {
    if let Some(status) = decision_status(doc) {
        println!("Status:    {status}");
    }
    if let Some(date) = decision_date(doc) {
        println!("Date:      {date}");
    }
    print_metadata_values("Deciders", decision_deciders(doc));
    if let Some(context) = decision_context(doc) {
        println!("Context:   {context}");
    }
    print_metadata_values("Consequences", decision_consequences(doc));
    print_metadata_values("Alternatives", decision_alternatives(doc));
    print_metadata_values("Supersedes", decision_supersedes(doc));
    print_metadata_values("Superseded by", decision_superseded_by(doc));
    print_metadata_values(
        "References",
        doc.field("references")
            .map(parse_field_values)
            .unwrap_or_default(),
    );
    print_metadata_values(
        "Tags",
        doc.field("tags")
            .map(parse_field_values)
            .unwrap_or_default(),
    );
}

fn print_metadata_values(label: &str, values: Vec<String>) {
    if !values.is_empty() {
        println!("{label}: {}", values.join(", "));
    }
}

fn print_show(
    doc: &Document,
    children: &[Document],
    role: Option<TaskRole>,
    relationship: Option<ParentRelationship>,
) {
    println!("ID:        {}", doc.id());
    println!("Type:      {}", doc.doc_type());
    if let Some(kind) = doc.kind() {
        println!("Kind:      {kind}");
    }
    println!("Title:     {}", doc.title());
    if doc.doc_type() == "decision" {
        print_decision_metadata(doc);
    }
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
    if let Some(parent_id) = doc.field("parentId") {
        let label = relationship
            .unwrap_or(ParentRelationship::Parent)
            .human_label();
        println!("{label}: {parent_id}");
    }
    if !children.is_empty() {
        let label = if role == Some(TaskRole::Epic) {
            "Tasks"
        } else {
            "Subtasks"
        };
        println!("{label}:   {}", children.len());
        for child in children {
            let status = child
                .field("state")
                .or_else(|| {
                    (child.location == DocumentLocation::Logs).then(|| completion_outcome(child))
                })
                .unwrap_or(child.location.as_str());
            println!("  {} [{}] {}", child.id(), status, child.title());
        }
    }
    if let Some(created_at) = doc.field("createdAt") {
        println!("Created:   {created_at}");
    }
    if let Some(updated_at) = doc.field("updatedAt") {
        println!("Updated:   {updated_at}");
    }
    if let Some(completed_at) = doc.field("completedAt") {
        if is_canceled_log(doc) {
            println!("Canceled:  {completed_at}");
        } else {
            println!("Completed: {completed_at}");
        }
    }
    if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
        println!("Outcome:   {}", completion_outcome(doc));
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

fn search_documents(docs: Vec<Document>, options: &SearchOptions) -> Vec<SearchResult> {
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
        .filter(|doc| {
            options
                .parent
                .as_deref()
                .is_none_or(|parent| doc.field("parentId") == Some(parent))
        })
        .filter_map(|doc| search_match(doc, &options.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));
    results
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

fn print_search_table(
    results: &[SearchResult],
    hierarchy: &HierarchyIndex,
) -> Result<(), CliError> {
    if results.is_empty() {
        println!("No matching Tandem documents found.");
        return Ok(());
    }
    println!(
        "{:<12} {:<8} {:<12} {:<8} {:<8} {:<9} {:<12} {:<24} MATCH",
        "ID", "WHERE", "STATE", "TYPE", "KIND", "RELATION", "PARENT", "TITLE"
    );
    for result in results {
        let doc = &result.doc;
        let (relationship, parent_id) = parent_table_values(doc, hierarchy)?;
        println!(
            "{:<12} {:<8} {:<12} {:<8} {:<8} {:<9} {:<12} {:<24} {}",
            truncate(doc.id(), 12),
            doc.location.as_str(),
            truncate(
                doc.field("state")
                    .or_else(|| {
                        (doc.location == DocumentLocation::Logs).then(|| completion_outcome(doc))
                    })
                    .unwrap_or("-"),
                12,
            ),
            truncate(doc.doc_type(), 8),
            truncate(doc.kind().unwrap_or("-"), 8),
            relationship,
            truncate(parent_id, 12),
            truncate(doc.title(), 24),
            truncate(&result.snippet, 80)
        );
    }
    Ok(())
}

fn print_log_table(docs: &[Document]) {
    if docs.is_empty() {
        println!("No archived Tandem logs found.");
        return;
    }
    println!(
        "{:<12} {:<20} {:<10} {:<36} SUMMARY",
        "ID", "ARCHIVED", "OUTCOME", "TITLE"
    );
    for doc in docs {
        println!(
            "{:<12} {:<20} {:<10} {:<36} {}",
            truncate(doc.id(), 12),
            truncate(doc.field("completedAt").unwrap_or("-"), 20),
            truncate(completion_outcome(doc), 10),
            truncate(doc.title(), 36),
            truncate(completion_summary(doc).unwrap_or("-"), 80)
        );
    }
}

fn print_log_show(doc: &Document, relationship: Option<ParentRelationship>) {
    println!("Log document");
    print_show(doc, &[], None, relationship);
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
    println!(
        "{:<14} {:<12} {:<10} {:<34} {:<20} SUMMARY",
        "ID", "STATUS", "DATE", "TITLE", "REFERENCES"
    );
    for doc in docs {
        println!(
            "{:<14} {:<12} {:<10} {:<34} {:<20} {}",
            truncate(doc.id(), 14),
            truncate(decision_status(doc).unwrap_or("-"), 12),
            truncate(decision_date(doc).unwrap_or("-"), 10),
            truncate(doc.title(), 34),
            truncate(doc.field("references").unwrap_or("-"), 20),
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
    outcome: Option<&str>,
    files_changed: &[String],
    validation: Option<&str>,
    reviewer: Option<&str>,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let completion_block =
        render_completion_block(summary, outcome, files_changed, validation, reviewer);
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
    outcome: Option<&str>,
    files_changed: &[String],
    validation: Option<&str>,
    reviewer: Option<&str>,
) -> String {
    let mut lines = Vec::new();
    lines.push("completion:".to_string());
    if let Some(outcome) = outcome {
        lines.push(format!("  outcome: {}", yaml_double_quote(outcome)));
    }
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

fn add_outcome_json(outcome: &AddOutcome) -> String {
    let mut fields = vec![
        format!("\"id\":{}", json_string(&outcome.id)),
        "\"type\":\"task\"".to_string(),
        format!("\"state\":{}", json_string(&outcome.state)),
        format!("\"title\":{}", json_string(&outcome.title)),
    ];
    if let Some(kind) = outcome.kind.as_deref() {
        fields.push(format!("\"kind\":{}", json_string(kind)));
    }
    if let Some(parent) = outcome.parent.as_deref() {
        fields.push(format!("\"parentId\":{}", json_string(parent)));
    }
    if let Some(relationship) = outcome.parent_relationship {
        fields.push(format!(
            "\"parentRelationship\":{}",
            json_string(relationship.as_str())
        ));
    }
    fields.push(format!(
        "\"path\":{}",
        json_string(&display_path(&outcome.path))
    ));
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{{{}}}}},\"warnings\":{}}}",
        fields.join(","),
        json_array_strings(&outcome.warnings)
    )
}

fn list_json(docs: &[Document], hierarchy: &HierarchyIndex) -> Result<String, CliError> {
    let mut by_state = BTreeMap::<String, usize>::new();
    for doc in docs {
        let state = doc.field("state").unwrap_or("unknown").to_string();
        *by_state.entry(state).or_insert(0) += 1;
    }

    let items = docs
        .iter()
        .map(|doc| Ok(document_summary_json(doc, hierarchy.relationship(doc)?)))
        .collect::<Result<Vec<_>, CliError>>()?;
    let states = by_state
        .iter()
        .map(|(state, count)| format!("{}:{count}", json_string(state)))
        .collect::<Vec<_>>()
        .join(",");
    let warnings = docs.iter().flat_map(document_warnings).collect::<Vec<_>>();

    Ok(format!(
        "{{\"ok\":true,\"data\":{{\"items\":[{}],\"counts\":{{\"total\":{},\"byState\":{{{}}}}}}},\"warnings\":{}}}",
        items.join(","),
        docs.len(),
        states,
        json_array_strings(&warnings)
    ))
}

fn show_json(
    doc: &Document,
    children: &[Document],
    role: Option<TaskRole>,
    relationship: Option<ParentRelationship>,
) -> String {
    let warnings = document_warnings(doc);
    let mut data_fields = vec![format!("\"document\":{}", document_detail_json(doc))];
    if let Some(relationship) = relationship {
        data_fields.push(format!(
            "\"parentRelationship\":{}",
            json_string(relationship.as_str())
        ));
    }
    if matches!(role, Some(TaskRole::Epic | TaskRole::Task)) {
        let children = children
            .iter()
            .map(child_task_summary_json)
            .collect::<Vec<_>>()
            .join(",");
        let key = if role == Some(TaskRole::Epic) {
            "tasks"
        } else {
            "subtasks"
        };
        data_fields.push(format!("\"{key}\":[{children}]"));
    }
    data_fields.push(format!("\"body\":{}", json_string(&doc.body)));
    data_fields.push(format!(
        "\"path\":{}",
        json_string(&display_path(&doc.path))
    ));
    data_fields.push(format!(
        "\"location\":{}",
        json_string(doc.location.as_str())
    ));
    format!(
        "{{\"ok\":true,\"data\":{{{}}},\"warnings\":{}}}",
        data_fields.join(","),
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

fn log_show_json(doc: &Document, relationship: Option<ParentRelationship>) -> String {
    let files = completion_files_changed(doc);
    let relationship_field = relationship
        .map(|relationship| {
            format!(
                ",\"parentRelationship\":{}",
                json_string(relationship.as_str())
            )
        })
        .unwrap_or_default();
    format!(
        "{{\"ok\":true,\"data\":{{\"document\":{}{},\"completion\":{{\"outcome\":{},\"summary\":{},\"filesChanged\":{},\"validation\":{},\"reviewer\":{}}},\"body\":{},\"path\":{}}},\"warnings\":[]}}",
        document_detail_json(doc),
        relationship_field,
        json_string(completion_outcome(doc)),
        json_string(completion_summary(doc).unwrap_or("")),
        json_array_strings(&files),
        json_string(completion_validation(doc).unwrap_or("")),
        json_string(completion_reviewer(doc).unwrap_or("")),
        json_string(&doc.body),
        json_string(&display_path(&doc.path))
    )
}

fn search_json(
    query: &str,
    results: &[SearchResult],
    hierarchy: &HierarchyIndex,
) -> Result<String, CliError> {
    let items = results
        .iter()
        .map(|result| {
            let doc = &result.doc;
            let mut fields = Vec::new();
            push_json_field(&mut fields, "id", doc.id());
            push_json_field(&mut fields, "type", doc.doc_type());
            push_optional_json_field(&mut fields, "kind", doc.kind());
            push_json_field(&mut fields, "title", doc.title());
            push_json_field(&mut fields, "location", doc.location.as_str());
            push_optional_json_field(&mut fields, "state", doc.field("state"));
            push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
            if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
                push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
            }
            push_optional_json_field(&mut fields, "parentId", doc.field("parentId"));
            push_parent_relationship_json_field(&mut fields, hierarchy.relationship(doc)?);
            push_json_field(&mut fields, "snippet", &result.snippet);
            Ok(format!("{{{}}}", fields.join(",")))
        })
        .collect::<Result<Vec<_>, CliError>>()?;
    Ok(format!(
        "{{\"ok\":true,\"data\":{{\"query\":{},\"results\":[{}]}},\"warnings\":[]}}",
        json_string(query),
        items.join(",")
    ))
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
            push_decision_metadata_json_fields(&mut fields, doc);
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
    push_decision_metadata_json_fields(&mut fields, doc);
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

fn document_summary_json(doc: &Document, relationship: Option<ParentRelationship>) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "priority", doc.field("priority"));
    push_optional_json_field(&mut fields, "assignee", doc.field("assignee"));
    push_optional_json_field(&mut fields, "parentId", doc.field("parentId"));
    push_parent_relationship_json_field(&mut fields, relationship);
    if let Some(tags) = doc.field("tags") {
        fields.push(format!(
            "\"tags\":{}",
            json_array_strings(&parse_field_values(tags))
        ));
    }
    if doc.doc_type() == "decision" {
        push_decision_metadata_json_fields(&mut fields, doc);
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
    format!("{{{}}}", fields.join(","))
}

fn document_detail_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    for key in [
        "state",
        "priority",
        "assignee",
        "dueDate",
        "parentId",
        "createdAt",
        "updatedAt",
        "completedAt",
    ] {
        push_optional_json_field(&mut fields, key, doc.field(key));
    }
    push_optional_json_field(&mut fields, "completionSummary", completion_summary(doc));
    if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
        push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
    }
    for key in ["tags", "blockers", "references", "relatedFiles"] {
        if let Some(value) = doc.field(key) {
            fields.push(format!(
                "{}:{}",
                json_string(key),
                json_array_strings(&parse_field_values(value))
            ));
        }
    }
    if doc.doc_type() == "decision" {
        push_decision_metadata_json_fields(&mut fields, doc);
    }
    push_status_object_json(&mut fields, "accord", accord_status(doc));
    push_status_object_json(&mut fields, "review", review_status(doc));
    format!("{{{}}}", fields.join(","))
}

fn child_task_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "state", doc.field("state"));
    push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
    if doc.location == DocumentLocation::Logs {
        push_json_field(&mut fields, "completionOutcome", completion_outcome(doc));
    }
    push_json_field(&mut fields, "location", doc.location.as_str());
    format!("{{{}}}", fields.join(","))
}

fn log_summary_json(doc: &Document) -> String {
    let mut fields = Vec::new();
    push_json_field(&mut fields, "id", doc.id());
    push_json_field(&mut fields, "type", doc.doc_type());
    push_optional_json_field(&mut fields, "kind", doc.kind());
    push_json_field(&mut fields, "title", doc.title());
    push_optional_json_field(&mut fields, "completedAt", doc.field("completedAt"));
    push_json_field(&mut fields, "outcome", completion_outcome(doc));
    push_optional_json_field(&mut fields, "summary", completion_summary(doc));
    push_optional_json_field(&mut fields, "validationStatus", completion_validation(doc));
    format!("{{{}}}", fields.join(","))
}

fn push_json_field(fields: &mut Vec<String>, key: &str, value: &str) {
    fields.push(format!("{}:{}", json_string(key), json_string(value)));
}

fn push_parent_relationship_json_field(
    fields: &mut Vec<String>,
    relationship: Option<ParentRelationship>,
) {
    if let Some(relationship) = relationship {
        push_json_field(fields, "parentRelationship", relationship.as_str());
    }
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

fn push_optional_json_array_field(fields: &mut Vec<String>, key: &str, values: Vec<String>) {
    if !values.is_empty() {
        fields.push(format!(
            "{}:{}",
            json_string(key),
            json_array_strings(&values)
        ));
    }
}

fn push_decision_metadata_json_fields(fields: &mut Vec<String>, doc: &Document) {
    push_optional_json_field(fields, "status", decision_status(doc));
    push_optional_json_field(fields, "date", decision_date(doc));
    push_optional_json_field(fields, "context", decision_context(doc));
    push_optional_json_array_field(fields, "deciders", decision_deciders(doc));
    push_optional_json_array_field(fields, "consequences", decision_consequences(doc));
    push_optional_json_array_field(fields, "alternatives", decision_alternatives(doc));
    push_optional_json_array_field(fields, "supersedes", decision_supersedes(doc));
    push_optional_json_array_field(fields, "supersededBy", decision_superseded_by(doc));
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

#[derive(Debug)]
struct CreatedDocument {
    id: String,
    path: PathBuf,
}

fn create_new_sequential_document<F>(
    workspace: &Workspace,
    prefix: &str,
    content_for_id: F,
) -> Result<CreatedDocument, CliError>
where
    F: FnMut(&str) -> String,
{
    let last_allocated = next_sequential_number(workspace, prefix)?;
    create_new_sequential_document_after(workspace, prefix, last_allocated, content_for_id)
}

fn create_new_sequential_document_after<F>(
    workspace: &Workspace,
    prefix: &str,
    last_allocated: usize,
    mut content_for_id: F,
) -> Result<CreatedDocument, CliError>
where
    F: FnMut(&str) -> String,
{
    let mut next_number = last_allocated.checked_add(1).ok_or_else(|| {
        CliError::user(format!("ID allocation failure: {prefix} sequence overflow"))
    })?;

    for _ in 0..MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS {
        let id = format!("{prefix}-{next_number}");
        let path = workspace.board_dir.join(format!("{id}.md"));
        let content = content_for_id(&id);
        if write_new_atomic(&path, &content)? {
            return Ok(CreatedDocument { id, path });
        }
        next_number = next_number.checked_add(1).ok_or_else(|| {
            CliError::user(format!("ID allocation failure: {prefix} sequence overflow"))
        })?;
    }

    Err(CliError::user(format!(
        "ID allocation failure: could not reserve a new {prefix} document after {MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS} attempts; concurrent writers may be too active, rerun the command"
    )))
}

fn next_sequential_number(workspace: &Workspace, prefix: &str) -> Result<usize, CliError> {
    let hierarchy = HierarchyIndex::from_workspace(workspace)?;
    Ok(next_sequential_number_in_hierarchy(&hierarchy, prefix))
}

fn next_sequential_number_in_hierarchy(hierarchy: &HierarchyIndex, prefix: &str) -> usize {
    let needle = format!("{prefix}-");
    hierarchy
        .documents
        .values()
        .filter_map(|doc| doc.id().strip_prefix(&needle))
        .filter_map(positive_canonical_number)
        .max()
        .unwrap_or(0)
}

fn replace_markdown_body(content: &str, body: &str) -> Result<String, CliError> {
    let (frontmatter, _) = split_frontmatter(content).map_err(CliError::user)?;
    Ok(format!("---\n{}---\n{}", frontmatter, body))
}

fn patch_frontmatter_content(
    content: &str,
    updates: &BTreeMap<String, String>,
    removes: &[&str],
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let mut seen = BTreeMap::<String, bool>::new();
    let mut output_frontmatter = String::new();
    let lines = frontmatter.split_inclusive('\n').collect::<Vec<_>>();
    let mut index = 0;

    while index < lines.len() {
        let raw_line = lines[index];
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(key) = frontmatter_line_key(line) {
            if removes.iter().any(|remove| *remove == key) {
                index += 1;
                while index < lines.len() {
                    let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                    if is_top_level_frontmatter_boundary(next) {
                        break;
                    }
                    index += 1;
                }
                continue;
            }
            if let Some(value) = updates.get(key) {
                output_frontmatter.push_str(&format!("{key}: {}\n", yaml_value_for_update(value)));
                seen.insert(key.to_string(), true);
                index += 1;
                while index < lines.len() {
                    let next = lines[index].trim_end_matches('\n').trim_end_matches('\r');
                    if is_top_level_frontmatter_boundary(next) {
                        break;
                    }
                    index += 1;
                }
                continue;
            }
        }
        output_frontmatter.push_str(raw_line);
        index += 1;
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
    let temp_path = write_temp_file_for(path, content)?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(CliError::user(format!(
            "Write failure: could not replace {}: {error}",
            display_path(path)
        )));
    }
    Ok(())
}

fn write_new_atomic(path: &Path, content: &str) -> Result<bool, CliError> {
    // Hard-linking the fully written temp file reserves `path` without replacing
    // an existing document, letting concurrent adders retry with the next ID.
    let temp_path = write_temp_file_for(path, content)?;
    match fs::hard_link(&temp_path, path) {
        Ok(()) => {
            let _ = fs::remove_file(&temp_path);
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temp_path);
            Ok(false)
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(CliError::user(format!(
                "Write failure: could not create {}: {error}",
                display_path(path)
            )))
        }
    }
}

fn write_temp_file_for(path: &Path, content: &str) -> Result<PathBuf, CliError> {
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
    drop(file);
    Ok(temp_path)
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.md");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(
        ".{file_name}.tmp.{}.{}.{}",
        std::process::id(),
        nanos,
        counter
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

fn date_from_timestamp(timestamp: &str) -> String {
    timestamp.chars().take(10).collect()
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

    fn test_workspace(root: &Path) -> Workspace {
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
        workspace
    }

    #[test]
    fn cli_version_uses_cargo_package_version() {
        assert_eq!(
            version_text(),
            format!("tandem {}", env!("CARGO_PKG_VERSION"))
        );
    }

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
    fn concurrent_task_adds_allocate_unique_ids_without_overwrite() {
        let root = std::env::temp_dir().join(format!(
            "tandem-concurrent-add-{}-{}",
            std::process::id(),
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

        let thread_count = 8;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(thread_count));
        let handles = (0..thread_count)
            .map(|index| {
                let workspace = workspace.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    add_task(
                        &workspace,
                        AddOptions {
                            title: Some(format!("Concurrent task {index}")),
                            ..AddOptions::default()
                        },
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();

        let outcomes = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        let mut ids = outcomes
            .iter()
            .map(|outcome| outcome.id.clone())
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.strip_prefix("task-").unwrap().parse::<usize>().unwrap());
        assert_eq!(
            ids,
            (1..=thread_count)
                .map(|number| format!("task-{number}"))
                .collect::<Vec<_>>()
        );

        let docs = read_documents(&workspace.board_dir, DocumentLocation::Board).unwrap();
        assert_eq!(docs.len(), thread_count);
        for outcome in outcomes {
            let content = fs::read_to_string(outcome.path).unwrap();
            assert!(content.contains(&format!("id: {}", outcome.id)));
            assert!(content.contains(&outcome.title));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_relationships_show_collections_and_task_delegation_metadata() {
        let root = std::env::temp_dir().join(format!(
            "tandem-canonical-relationships-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Parent decision\nstatus: accepted\n---\n",
        )
        .unwrap();
        let epic = add_task(
            &workspace,
            AddOptions {
                title: Some("Epic".to_string()),
                kind: Some("epic".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let task = add_task(
            &workspace,
            AddOptions {
                title: Some("Task of epic".to_string()),
                parent: Some(epic.id.clone()),
                assignee: Some("worker-a".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Task checklist item".to_string()),
                parent: Some(task.id.clone()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic = add_task(
            &workspace,
            AddOptions {
                title: Some("Decision-parented Task".to_string()),
                parent: Some("decision-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic_subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Generic Task checklist item".to_string()),
                parent: Some(generic.id.clone()),
                ..AddOptions::default()
            },
        )
        .unwrap();

        assert_eq!(epic.id, "task-1");
        assert_eq!(task.id, "task-2");
        assert_eq!(subtask.id, "task-2-1");
        assert_eq!(generic.id, "task-3");
        assert_eq!(generic_subtask.id, "task-3-1");
        assert_eq!(task.parent_relationship, Some(ParentRelationship::EpicTask));
        assert_eq!(
            subtask.parent_relationship,
            Some(ParentRelationship::Subtask)
        );
        assert_eq!(
            generic.parent_relationship,
            Some(ParentRelationship::Parent)
        );

        let hierarchy = HierarchyIndex::from_workspace(&workspace).unwrap();
        hierarchy.validate_all_task_hierarchies().unwrap();
        let task_doc = hierarchy.document(&task.id).unwrap();
        assert_eq!(hierarchy.task_role(task_doc).unwrap(), Some(TaskRole::Task));
        assert_eq!(task_doc.field("assignee"), Some("worker-a"));
        let epic_doc = hierarchy.document(&epic.id).unwrap();
        let epic_children = find_hierarchy_children(&hierarchy, epic_doc).unwrap();
        let task_children = find_hierarchy_children(&hierarchy, task_doc).unwrap();
        assert!(
            show_json(epic_doc, &epic_children, Some(TaskRole::Epic), None)
                .contains("\"tasks\":[{\"id\":\"task-2\"")
        );
        assert!(show_json(
            task_doc,
            &task_children,
            Some(TaskRole::Task),
            Some(ParentRelationship::EpicTask)
        )
        .contains("\"subtasks\":[{\"id\":\"task-2-1\""));
        let docs = hierarchy.documents.values().cloned().collect::<Vec<_>>();
        assert!(list_json(&docs, &hierarchy)
            .unwrap()
            .contains("\"parentRelationship\":\"epic-task\""));
        let filtered = filter_documents(
            docs.clone(),
            &ListOptions {
                parent: Some(epic.id.clone()),
                ..ListOptions::default()
            },
        );
        assert_eq!(
            filtered.iter().map(Document::id).collect::<Vec<_>>(),
            ["task-2"]
        );
        let results = search_documents(
            docs,
            &SearchOptions {
                query: "Task of epic".to_string(),
                parent: Some(epic.id.clone()),
                ..SearchOptions::default()
            },
        );
        assert!(search_json("Task of epic", &results, &hierarchy)
            .unwrap()
            .contains("\"parentRelationship\":\"epic-task\""));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_allocation_scans_logs_for_global_and_subtask_sequences() {
        let root = std::env::temp_dir().join(format!(
            "tandem-canonical-allocation-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        fs::write(
            workspace.logs_dir.join("task-103.md"),
            "---\nid: task-103\ntype: task\ntitle: Logged Task\ncompletedAt: now\ncompletion:\n  summary: done\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.logs_dir.join("task-103-1.md"),
            "---\nid: task-103-1\ntype: task\ntitle: Logged Subtask\nparentId: task-103\ncompletedAt: now\ncompletion:\n  summary: done\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Decision\nstatus: accepted\n---\n",
        )
        .unwrap();

        let second_subtask = add_task(
            &workspace,
            AddOptions {
                title: Some("Second Subtask".to_string()),
                parent: Some("task-103".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        let generic_task = add_task(
            &workspace,
            AddOptions {
                title: Some("Generic-parent Task".to_string()),
                parent: Some("decision-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        assert_eq!(second_subtask.id, "task-103-2");
        assert_eq!(generic_task.id, "task-104");
        assert_eq!(
            second_subtask.parent_relationship,
            Some(ParentRelationship::Subtask)
        );
        let hierarchy = HierarchyIndex::from_workspace(&workspace).unwrap();
        hierarchy.validate_all_task_hierarchies().unwrap();
        let logged_parent = hierarchy.document("task-103").unwrap();
        assert_eq!(
            hierarchy.task_role(logged_parent).unwrap(),
            Some(TaskRole::Task)
        );
        let logged_child = hierarchy.document("task-103-1").unwrap();
        assert!(
            log_show_json(logged_child, hierarchy.relationship(logged_child).unwrap())
                .contains("\"parentRelationship\":\"subtask\"")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolved_graph_rejects_duplicates_unresolved_parents_cycles_and_invalid_depth() {
        let make_doc = |path: &str, frontmatter: &str| Document {
            path: PathBuf::from(path),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(frontmatter).unwrap(),
            body: String::new(),
        };

        let duplicate = HierarchyIndex::from_documents(vec![
            make_doc("a.md", "id: task-1\ntype: task\ntitle: A\nstate: todo\n"),
            make_doc("b.md", "id: task-1\ntype: task\ntitle: B\nstate: todo\n"),
        ])
        .unwrap_err();
        assert!(duplicate.message.contains("duplicate document ID `task-1`"));

        let unsupported_kind = HierarchyIndex::from_documents(vec![make_doc(
            "task-1.md",
            "id: task-1\ntype: task\nkind: Epic\ntitle: Wrong kind casing\nstate: todo\n",
        )])
        .unwrap();
        let unsupported_doc = unsupported_kind.document("task-1").unwrap();
        assert!(unsupported_kind
            .task_role(unsupported_doc)
            .unwrap_err()
            .message
            .contains("invalid kind `Epic`"));
        let error = unsupported_kind
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(error.message.contains("invalid kind `Epic`"));
        assert!(error.message.contains("expected one of: epic"));

        let aggregate = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: First global child\nstate: todo\nparentId: task-1\n",
            ),
            make_doc(
                "task-3.md",
                "id: task-3\ntype: task\ntitle: Second global child\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        let error = aggregate.validate_all_task_hierarchies().unwrap_err();
        assert!(error
            .message
            .contains("hierarchy contains 2 structural errors"));
        assert!(error.message.contains("task-2"));
        assert!(error.message.contains("task-3"));

        let unresolved = HierarchyIndex::from_documents(vec![make_doc(
            "task-1.md",
            "id: task-1\ntype: task\ntitle: Missing parent\nstate: todo\nparentId: task-9\n",
        )])
        .unwrap();
        assert!(unresolved
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("unresolved parentId `task-9`"));

        let cycle = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: A\nstate: todo\nparentId: task-2\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: B\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(cycle
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("task hierarchy cycle"));

        let direct_epic_hierarchical = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\nkind: epic\ntitle: Epic\nstate: todo\n",
            ),
            make_doc(
                "task-1-1.md",
                "id: task-1-1\ntype: task\ntitle: Wrong Task ID\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        let error = direct_epic_hierarchical
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(error.message.contains("expected global `task-N`"));

        let parented_epic = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\nkind: epic\ntitle: Nested Epic\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(parented_epic
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("Epic task-2 cannot have parentId"));

        let global_subtask = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-2.md",
                "id: task-2\ntype: task\ntitle: Wrong Subtask ID\nstate: todo\nparentId: task-1\n",
            ),
        ])
        .unwrap();
        assert!(global_subtask
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("expected `task-1-M`"));

        let child_beneath_subtask = HierarchyIndex::from_documents(vec![
            make_doc(
                "task-1.md",
                "id: task-1\ntype: task\ntitle: Task\nstate: todo\n",
            ),
            make_doc(
                "task-1-1.md",
                "id: task-1-1\ntype: task\ntitle: Subtask\nstate: todo\nparentId: task-1\n",
            ),
            make_doc(
                "task-1-1-1.md",
                "id: task-1-1-1\ntype: task\ntitle: Invalid depth\nstate: todo\nparentId: task-1-1\n",
            ),
        ])
        .unwrap();
        let error = child_beneath_subtask
            .validate_all_task_hierarchies()
            .unwrap_err();
        assert!(
            error
                .message
                .contains("Subtask task-1-1 cannot have children")
                || error.message.contains("child of Subtask task-1-1")
        );
    }

    #[test]
    fn prospective_updates_reject_role_changes_id_mismatches_and_invalid_descendants() {
        let root = std::env::temp_dir().join(format!(
            "tandem-prospective-hierarchy-{}-{}",
            std::process::id(),
            current_timestamp()
        ));
        let workspace = test_workspace(&root);
        for (name, content) in [
            (
                "task-1.md",
                "---\nid: task-1\ntype: task\nkind: epic\ntitle: Epic one\nstate: todo\n---\n",
            ),
            (
                "task-2.md",
                "---\nid: task-2\ntype: task\ntitle: Task\nstate: todo\nparentId: task-1\n---\n",
            ),
            (
                "task-2-1.md",
                "---\nid: task-2-1\ntype: task\ntitle: Subtask\nstate: todo\nparentId: task-2\n---\n",
            ),
            (
                "task-3.md",
                "---\nid: task-3\ntype: task\nkind: epic\ntitle: Epic two\nstate: todo\n---\n",
            ),
            (
                "task-4.md",
                "---\nid: task-4\ntype: task\ntitle: Other Task\nstate: todo\nparentId: task-3\n---\n",
            ),
            (
                "task-5.md",
                "---\nid: task-5\ntype: task\ntitle: Root with child\nstate: todo\n---\n",
            ),
            (
                "task-5-1.md",
                "---\nid: task-5-1\ntype: task\ntitle: Child\nstate: todo\nparentId: task-5\n---\n",
            ),
        ] {
            fs::write(workspace.board_dir.join(name), content).unwrap();
        }

        let valid = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-2".to_string(),
                parent: Some("task-3".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(
            valid.parent_relationship,
            Some(ParentRelationship::EpicTask)
        );

        let mismatch = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-2-1".to_string(),
                parent: Some("task-4".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap_err();
        assert!(mismatch.message.contains("expected `task-4-M`"));
        assert_eq!(
            read_document(
                &workspace.board_dir.join("task-2-1.md"),
                DocumentLocation::Board
            )
            .unwrap()
            .field("parentId"),
            Some("task-2")
        );

        let descendant = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-5".to_string(),
                kind: Some("epic".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap_err();
        assert!(
            descendant.message.contains("task-5-1")
                && descendant.message.contains("expected global `task-N`")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_subtask_adds_allocate_unique_parent_derived_ids() {
        let root = std::env::temp_dir().join(format!(
            "tandem-concurrent-subtask-add-{}-{}",
            std::process::id(),
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
        fs::write(
            workspace.board_dir.join("task-103.md"),
            "---\nid: task-103\ntype: task\ntitle: Parent\nstate: todo\n---\n",
        )
        .unwrap();

        let thread_count = 8;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(thread_count));
        let handles = (0..thread_count)
            .map(|index| {
                let workspace = workspace.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    add_task(
                        &workspace,
                        AddOptions {
                            title: Some(format!("Concurrent child {index}")),
                            parent: Some("task-103".to_string()),
                            ..AddOptions::default()
                        },
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        let mut ids = handles
            .into_iter()
            .map(|handle| handle.join().unwrap().id)
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| {
            id.strip_prefix("task-103-")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        });
        assert_eq!(
            ids,
            (1..=thread_count)
                .map(|number| format!("task-103-{number}"))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            read_documents(&workspace.board_dir, DocumentLocation::Board)
                .unwrap()
                .len(),
            thread_count + 1
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hierarchy_change_labels_follow_resolved_parent_type() {
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::EpicTask)),
            "Task of Epic"
        );
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::Subtask)),
            "Subtask of"
        );
        assert_eq!(
            display_change_field("parentId", Some(ParentRelationship::Parent)),
            "Parent"
        );
        assert_eq!(display_change_field("priority", None), "priority");
    }

    #[test]
    fn inline_subtask_authoring_flags_point_to_parent_linked_tasks() {
        let add_error = parse_add_args(&[
            "--title".to_string(),
            "Parent".to_string(),
            "--subtask".to_string(),
            "Checklist".to_string(),
        ])
        .unwrap_err();
        assert!(add_error.message.contains("add --subtask is deprecated"));
        assert!(add_error.message.contains("--parent <task-id>"));

        let update_error = parse_update_args(&[
            "task-1".to_string(),
            "--subtask".to_string(),
            "Checklist".to_string(),
        ])
        .unwrap_err();
        assert!(update_error
            .message
            .contains("update --subtask is deprecated"));
    }

    #[test]
    fn update_body_parser_accepts_empty_and_flag_looking_markdown() {
        let leading_dash = parse_update_args(&[
            "task-1".to_string(),
            "--body".to_string(),
            "- first item\n\nBody".to_string(),
        ])
        .unwrap();
        assert_eq!(leading_dash.body.as_deref(), Some("- first item\n\nBody"));

        let empty = parse_update_args(&["task-1".to_string(), "--body".to_string(), String::new()])
            .unwrap();
        assert_eq!(empty.body.as_deref(), Some(""));
    }

    #[test]
    fn cancel_parser_accepts_reason_and_requires_an_id() {
        let parsed = parse_cancel_args(&[
            "task-1".to_string(),
            "--reason".to_string(),
            "- no longer needed".to_string(),
        ])
        .unwrap();
        assert_eq!(parsed.id, "task-1");
        assert_eq!(parsed.reason.as_deref(), Some("- no longer needed"));
        assert!(parse_cancel_args(&["--reason".to_string(), "why".to_string()]).is_err());
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

        let hierarchy = HierarchyIndex::from_documents(vec![doc.clone()]).unwrap();
        assert!(show_json(&doc, &[], Some(TaskRole::Task), None)
            .contains("accord.status `claimed` suggests `in-progress`"));
        assert!(list_json(&[doc], &hierarchy)
            .unwrap()
            .contains("accord.status `claimed` suggests `in-progress`"));
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
            None,
            &["src/main.rs".to_string()],
            Some("cargo test passed"),
            Some("Algorant"),
        )
        .unwrap();
        assert!(!output.contains("completionSummary:"));
        assert!(!output.contains("filesChanged: [old.rs]"));
        assert!(output.contains("completion:\n  summary: \"Done\"\n"));
        assert!(output.contains("  filesChanged: [\"src/main.rs\"]\n"));
        assert!(output.contains("  validation: \"cargo test passed\"\n"));
        assert!(output.contains("  reviewer: \"Algorant\"\n"));
        assert!(output.ends_with("\nBody\n"));
    }

    #[test]
    fn completion_helpers_read_nested_and_legacy_flat_metadata() {
        let nested = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "completion:\n  summary: Done\n  validation: passed\n  reviewer: Algorant\n  filesChanged: [src/main.rs]\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert_eq!(completion_summary(&nested), Some("Done"));
        assert_eq!(completion_validation(&nested), Some("passed"));
        assert_eq!(completion_reviewer(&nested), Some("Algorant"));
        assert_eq!(completion_files_changed(&nested), vec!["src/main.rs"]);

        let legacy = Document {
            path: PathBuf::from("task-2.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "completionSummary: Done\ncompletionValidation: passed\ncompletionReviewer: Algorant\nfilesChanged: [src/lib.rs]\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert_eq!(completion_summary(&legacy), Some("Done"));
        assert_eq!(completion_validation(&legacy), Some("passed"));
        assert_eq!(completion_reviewer(&legacy), Some("Algorant"));
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
    fn validation_reports_invalid_task_kind() {
        let doc = Document {
            path: PathBuf::from(".tandem/logs/task-1.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\nkind: feature\ntitle: Demo\nstate: todo\n",
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
        assert!(error.message.contains("invalid kind `feature`"));
    }

    #[test]
    fn update_task_metadata_changes_scalars_and_appends_lists() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-{}",
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
        fs::write(
            workspace.board_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\nkind: epic\ntitle: Blocker Epic\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("decision-1.md"),
            "---\nid: decision-1\ntype: decision\ntitle: Parent decision\nstatus: accepted\n---\n",
        )
        .unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Old\nstate: todo\npriority: low\ntags: [cli]\ncustom: keep\ncreatedAt: \"2026-06-26T00:00:00Z\"\nupdatedAt: \"2026-06-26T00:00:00Z\"\n---\n\nBody\n",
        )
        .unwrap();

        let outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                title: Some("New".to_string()),
                priority: Some("high".to_string()),
                parent: Some("decision-1".to_string()),
                tags: vec!["cli".to_string(), "metadata".to_string()],
                blockers: vec!["task-2".to_string()],
                references: vec!["missing-decision".to_string()],
                related_files: vec!["src/main.rs".to_string()],
                ..UpdateOptions::default()
            },
        )
        .unwrap();

        let output = fs::read_to_string(&task_path).unwrap();
        assert_eq!(outcome.changes.len(), 7);
        assert_eq!(
            outcome.parent_relationship,
            Some(ParentRelationship::Parent)
        );
        assert_eq!(
            outcome.warnings,
            vec!["reference not found: missing-decision"]
        );
        assert!(output.contains("title: \"New\"\n"));
        assert!(output.contains("priority: \"high\"\n"));
        assert!(output.contains("parentId: \"decision-1\"\n"));
        assert!(output.contains("tags: [\"cli\", \"metadata\"]\n"));
        assert!(output.contains("blockers: [\"task-2\"]\n"));
        assert!(output.contains("references: [\"missing-decision\"]\n"));
        assert!(output.contains("relatedFiles: [\"src/main.rs\"]\n"));
        assert!(output.contains("custom: keep\n"));
        assert!(output.ends_with("\nBody\n"));
        assert!(fs::read_to_string(&workspace.events_path)
            .unwrap()
            .contains("task.updated"));

        let epic_parent_outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                parent: Some("task-2".to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(epic_parent_outcome.changes.len(), 1);
        assert_eq!(
            epic_parent_outcome.parent_relationship,
            Some(ParentRelationship::EpicTask)
        );
        assert!(fs::read_to_string(&task_path)
            .unwrap()
            .contains("parentId: \"task-2\"\n"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_task_metadata_noops_existing_list_entries_without_touching_file() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-noop-{}",
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
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        let before = "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\ntags: [cli]\nupdatedAt: \"old\"\n---\n\nBody\n";
        fs::write(&task_path, before).unwrap();

        let outcome = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                tags: vec!["cli".to_string()],
                ..UpdateOptions::default()
            },
        )
        .unwrap();

        assert!(outcome.changes.is_empty());
        assert_eq!(fs::read_to_string(&task_path).unwrap(), before);
        assert_eq!(fs::read_to_string(&workspace.events_path).unwrap(), "");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn update_task_body_replaces_clears_and_noops_exactly() {
        let root = std::env::temp_dir().join(format!(
            "tandem-update-body-{}",
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
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        let task_path = workspace.board_dir.join("task-1.md");
        fs::write(
            &task_path,
            "---\nid: task-1\ntype: task\ntitle: Demo\nstate: todo\ncustom: keep\nupdatedAt: \"old\"\n---\n\nOld body\n",
        )
        .unwrap();

        let replacement = "- first item\n\nUnicode: café 🦀\n";
        let changed = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(replacement.to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(changed.changes.len(), 1);
        assert_eq!(changed.changes[0].field, "body");
        let after_change = fs::read_to_string(&task_path).unwrap();
        assert!(after_change.contains("custom: keep\n"));
        assert_eq!(split_frontmatter(&after_change).unwrap().1, replacement);
        let events_after_change = fs::read_to_string(&workspace.events_path).unwrap();
        assert!(events_after_change.contains("task.updated"));
        assert!(!events_after_change.contains("Unicode"));
        assert!(!events_after_change.contains("first item"));

        let noop = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(replacement.to_string()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert!(noop.changes.is_empty());
        assert_eq!(fs::read_to_string(&task_path).unwrap(), after_change);
        assert_eq!(
            fs::read_to_string(&workspace.events_path).unwrap(),
            events_after_change
        );

        let cleared = update_task_metadata(
            &workspace,
            UpdateOptions {
                id: "task-1".to_string(),
                body: Some(String::new()),
                ..UpdateOptions::default()
            },
        )
        .unwrap();
        assert_eq!(cleared.changes.len(), 1);
        let after_clear = fs::read_to_string(&task_path).unwrap();
        assert!(after_clear.contains("custom: keep\n"));
        assert_eq!(split_frontmatter(&after_clear).unwrap().1, "");
        assert_eq!(
            fs::read_to_string(&workspace.events_path)
                .unwrap()
                .matches("task.updated")
                .count(),
            2
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancel_task_archives_auditable_outcome_and_rejects_active_descendants() {
        let root = std::env::temp_dir().join(format!(
            "tandem-cancel-{}",
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
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Parent\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\ntitle: Blocker\nstate: in-progress\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-3.md"),
            "---\nid: task-3\ntype: task\ntitle: Dependent\nstate: todo\nblockers: [task-2]\n---\n",
        )
        .unwrap();
        let child_path = workspace.board_dir.join("task-1-1.md");
        fs::write(
            &child_path,
            "---\nid: task-1-1\ntype: task\ntitle: Child\nstate: validation\nparentId: task-1\nblockers: [task-2]\nreview:\n  status: pending\naccord:\n  status: delivered\ncustom: keep\n---\n\nCanceled body\n",
        )
        .unwrap();

        let parent_error = cancel_task(&workspace, "task-1", "Parent canceled").unwrap_err();
        assert!(parent_error
            .message
            .contains("active descendants: task-1-1"));
        assert!(workspace.board_dir.join("task-1.md").exists());

        let child = cancel_task(&workspace, "task-1-1", "Created by mistake").unwrap();
        assert_eq!(child.id, "task-1-1");
        assert!(!child_path.exists());
        let canceled_content = fs::read_to_string(&child.log_path).unwrap();
        let canceled_doc = read_document(&child.log_path, DocumentLocation::Logs).unwrap();
        assert_eq!(
            completion_outcome(&canceled_doc),
            COMPLETION_OUTCOME_CANCELED
        );
        assert_eq!(
            completion_summary(&canceled_doc),
            Some("Canceled: Created by mistake")
        );
        assert!(canceled_doc.field("state").is_none());
        assert!(canceled_doc.field("completedAt").is_some());
        assert!(canceled_doc.field("updatedAt").is_some());
        assert!(canceled_content.contains("custom: keep\n"));
        assert_eq!(canceled_doc.body, "\nCanceled body\n");
        assert!(canceled_content.contains("review:\n  status: pending\n"));
        assert!(canceled_content.contains("accord:\n  status: delivered\n"));
        assert!(log_summary_json(&canceled_doc).contains("\"outcome\":\"canceled\""));
        assert!(document_detail_json(&canceled_doc).contains("\"completionOutcome\":\"canceled\""));
        assert!(fs::read_to_string(&workspace.events_path)
            .unwrap()
            .contains("task.canceled"));

        let next_child = add_task(
            &workspace,
            AddOptions {
                title: Some("Fresh child".to_string()),
                parent: Some("task-1".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap();
        assert_eq!(next_child.id, "task-1-2");

        cancel_task(&workspace, "task-2", "Dependency intentionally waived").unwrap();
        assert!(unresolved_blockers(&workspace, Some("[task-2]"))
            .unwrap()
            .is_empty());

        let legacy_completed = Document {
            path: PathBuf::from(".tandem/logs/task-99.md"),
            location: DocumentLocation::Logs,
            fields: parse_frontmatter_fields(
                "id: task-99\ntype: task\ntitle: Legacy\ncompletedAt: now\ncompletion:\n  summary: Done\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert_eq!(
            completion_outcome(&legacy_completed),
            COMPLETION_OUTCOME_COMPLETED
        );

        fs::write(
            workspace.board_dir.join("task-4.md"),
            "---\nid: task-4\ntype: task\ntitle: Duplicate board\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.logs_dir.join("task-4.md"),
            "---\nid: task-4\ntype: task\ntitle: Duplicate log\ncompletedAt: now\ncompletion:\n  summary: Existing\n---\n",
        )
        .unwrap();
        let duplicate_error = cancel_task(&workspace, "task-4", "Should fail").unwrap_err();
        assert!(duplicate_error
            .message
            .contains("duplicate document ID `task-4`"));
        assert!(workspace.board_dir.join("task-4.md").exists());
        assert!(workspace.logs_dir.join("task-4.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn show_json_includes_parent_id_only_when_present() {
        let child = Document {
            path: PathBuf::from("task-1-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: task-1-1\ntype: task\ntitle: Child\nstate: todo\nparentId: task-1\n",
            )
            .unwrap(),
            body: String::new(),
        };
        let parent = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\ntitle: Parent\nstate: todo\n",
            )
            .unwrap(),
            body: String::new(),
        };

        let parent_json = show_json(
            &parent,
            std::slice::from_ref(&child),
            Some(TaskRole::Task),
            None,
        );
        assert!(show_json(
            &child,
            &[],
            Some(TaskRole::Subtask),
            Some(ParentRelationship::Subtask)
        )
        .contains("\"parentId\":\"task-1\""));
        assert!(parent_json.contains("\"subtasks\":[{\"id\":\"task-1-1\""));
        assert!(!document_detail_json(&parent).contains("\"parentId\""));
    }

    #[test]
    fn update_rejects_invalid_priority_and_kind_while_json_exposes_metadata() {
        let doc = Document {
            path: PathBuf::from("task-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: task-1\ntype: task\nkind: epic\ntitle: Demo\nstate: todo\npriority: high\nblockers: [task-2]\nreferences: [decision-1]\nrelatedFiles: [src/main.rs]\n",
            )
            .unwrap(),
            body: String::new(),
        };
        assert!(document_summary_json(&doc, None).contains("\"kind\":\"epic\""));
        assert!(document_detail_json(&doc).contains("\"kind\":\"epic\""));
        let hierarchy = HierarchyIndex::from_documents(vec![doc.clone()]).unwrap();
        let search = search_json(
            "epic",
            &[SearchResult {
                doc: doc.clone(),
                snippet: "epic".to_string(),
            }],
            &hierarchy,
        )
        .unwrap();
        assert!(search.contains("\"kind\":\"epic\""));
        assert!(document_detail_json(&doc).contains("\"blockers\":[\"task-2\"]"));
        assert!(document_detail_json(&doc).contains("\"references\":[\"decision-1\"]"));
        assert!(document_detail_json(&doc).contains("\"relatedFiles\":[\"src/main.rs\"]"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                priority: Some("urgent".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("invalid priority `urgent`"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                kind: Some("feature".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("invalid kind `feature`"));

        let error = validate_update_options(
            &UpdateOptions {
                id: "task-1".to_string(),
                parent: Some("task-1".to_string()),
                ..UpdateOptions::default()
            },
            &hierarchy,
        )
        .unwrap_err();
        assert!(error.message.contains("cannot be its own parent"));
    }

    #[test]
    fn decision_metadata_is_status_not_workflow_state() {
        let doc = Document {
            path: PathBuf::from("decision-1.md"),
            location: DocumentLocation::Board,
            fields: parse_frontmatter_fields(
                "id: decision-1\ntype: decision\ntitle: Choose cache\nstatus: accepted\ndate: 2026-07-01\ndeciders: [Algorant, pi]\ncontext: Need a cache policy\nconsequences: [Faster reads]\nalternatives: [No cache]\nsupersedes: [decision-0]\nsupersededBy: [decision-2]\n",
            )
            .unwrap(),
            body: "## Decision\nUse the small cache.\n".to_string(),
        };

        let detail = document_detail_json(&doc);
        assert!(detail.contains("\"status\":\"accepted\""));
        assert!(detail.contains("\"date\":\"2026-07-01\""));
        assert!(detail.contains("\"deciders\":[\"Algorant\",\"pi\"]"));
        assert!(detail.contains("\"supersededBy\":[\"decision-2\"]"));
        assert!(!detail.contains("\"state\""));

        let filtered = filter_documents(
            vec![doc],
            &ListOptions {
                state: Some("accepted".to_string()),
                ..ListOptions::default()
            },
        );
        assert!(filtered.is_empty());
    }

    #[test]
    fn decision_status_validation_rejects_workflow_states() {
        assert!(validate_decision_status("accepted").is_ok());
        let error = validate_decision_status("todo").unwrap_err();
        assert!(error.message.contains("invalid decision status `todo`"));
        assert!(error.message.contains("proposed, accepted, rejected"));
    }

    #[test]
    fn decision_add_reference_and_supersession_diagnostics_are_warnings() {
        let root = std::env::temp_dir().join(format!(
            "tandem-decision-warnings-{}",
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
        fs::write(&workspace.config_path, "---\nstates: [todo]\n---\n").unwrap();
        fs::write(&workspace.events_path, "").unwrap();
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Task\nstate: todo\n---\n",
        )
        .unwrap();

        let warnings = decision_add_warnings(
            &workspace,
            &DecisionAddOptions {
                references: vec!["missing-ref".to_string()],
                supersedes: vec!["task-1".to_string()],
                superseded_by: vec!["missing-decision".to_string()],
                ..DecisionAddOptions::default()
            },
        )
        .unwrap();

        assert_eq!(
            warnings,
            vec![
                "reference not found: missing-ref".to_string(),
                "supersedes target task-1 is type task, not decision".to_string(),
                "supersededBy decision not found: missing-decision".to_string(),
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn date_from_timestamp_uses_utc_calendar_date() {
        assert_eq!(date_from_timestamp("2026-07-01T18:05:47Z"), "2026-07-01");
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
