//! Typed CLI-to-application conversion and dispatch.
use super::model::*;
use crate::project::sync::{self, Mode, Outcome, Report, Upgraded};
use crate::project::TandemProject;
use crate::{app, CliError};

/// How long a read trusts the local board before refreshing from the remote.
const READ_FRESHNESS: std::time::Duration = std::time::Duration::from_secs(60);

fn sync_json(report: &Report) -> serde_json::Value {
    let (status, message) = match &report.outcome {
        Outcome::Synced => ("synced", None),
        Outcome::Pending(reason) => ("pending", Some(reason.clone())),
        Outcome::LocalOnly => ("local-only", None),
        Outcome::NotGit => ("not-git", None),
    };
    let renamed: serde_json::Map<String, serde_json::Value> = report
        .renames
        .iter()
        .map(|(old, new)| (old.clone(), serde_json::Value::String(new.clone())))
        .collect();
    serde_json::json!({
        "status": status,
        "message": message,
        "renamed": renamed,
        "conflicts": report.conflicts.iter().map(|c| serde_json::json!({"id": c.id, "reason": c.reason})).collect::<Vec<_>>(),
        "held": report.held.iter().map(|h| serde_json::json!({"path": h.path, "reason": h.reason})).collect::<Vec<_>>(),
    })
}

fn sync_text(report: &Report) -> String {
    let mut text = match &report.outcome {
        Outcome::Synced => "synced".to_string(),
        Outcome::Pending(reason) => format!("saved locally; pending sync ({reason})"),
        Outcome::LocalOnly => "saved locally (no Git remote to sync with)".to_string(),
        Outcome::NotGit => "saved (board is not in a Git repository)".to_string(),
    };
    for (old, new) in &report.renames {
        text.push_str(&format!("\n  {old} is now {new}"));
    }
    for conflict in &report.conflicts {
        text.push_str(&format!(
            "\n  conflict: {}: {}",
            conflict.id, conflict.reason
        ));
    }
    for held in &report.held {
        text.push_str(&format!("\n  held: {}: {}", held.path, held.reason));
    }
    text
}

/// Publishes a mutation. A sync failure never undoes the saved change.
fn publish(project: &TandemProject) -> Report {
    sync::sync(project, Mode::Publish).unwrap_or_else(|error| Report::pending(error.message))
}

/// Refreshes the board before a read when it may be stale. Returns warnings.
fn refresh_for_read(project: &TandemProject) -> Vec<String> {
    if let Some(message) = app::project::historical(project) {
        return vec![message];
    }
    if !sync::is_stale(project, READ_FRESHNESS) {
        return Vec::new();
    }
    match sync::sync(project, Mode::Refresh) {
        Ok(report) => match report.outcome {
            Outcome::Pending(reason) => {
                vec![format!("the board may be out of date: {reason}")]
            }
            _ => Vec::new(),
        },
        Err(error) => vec![format!("the board may be out of date: {}", error.message)],
    }
}

fn open_read() -> Result<(TandemProject, Vec<String>), CliError> {
    let project = app::project::open()?;
    let mut warnings = refresh_for_read(&project);
    warnings.extend(project.held_edit_warnings());
    Ok((project, warnings))
}

fn print_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("Warning: {warning}");
    }
}

/// Opens the board for a change to `id`, mapping an outdated provisional ID
/// and refusing a record with an unresolved sync conflict.
fn open_write_for(id: &str) -> Result<(TandemProject, String), CliError> {
    let project = app::project::open_for_write()?;
    let id = project.current_id(id)?;
    refuse_conflicted(&project, &id)?;
    Ok((project, id))
}

fn refuse_conflicted(project: &TandemProject, id: &str) -> Result<(), CliError> {
    if let Some(conflict) = sync::conflict_for(project, id) {
        return Err(CliError::user(format!(
            "{id} has an unresolved sync conflict ({}). Resolve it with `tandem sync resolve {id} --keep local|remote|edited`.",
            conflict.reason
        )));
    }
    Ok(())
}

fn map_ids(project: &TandemProject, ids: Vec<String>) -> Result<Vec<String>, CliError> {
    ids.into_iter().map(|id| project.current_id(&id)).collect()
}

pub(crate) fn dispatch(command: Command, json: bool) -> Result<super::StartupRequest, CliError> {
    match command {
        Command::Init(args) => {
            let outcome = app::project::initialize(app::project::InitOptions {
                title: args.title,
                force: false,
            })?;
            let report = publish(&outcome.project);
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"title":outcome.title,"root":outcome.project.root().display().to_string(),"sync":sync_json(&report)},"warnings":[]})
                );
            } else if let Outcome::Pending(reason) = &report.outcome {
                eprintln!("Warning: the new board is not synced yet: {reason}");
            }
            Ok(super::StartupRequest::Exit)
        }
        Command::Add(args) => add(args, json),
        Command::Show(args) => show(args, json),
        Command::Assignment(args) => assignment(args, json),
        Command::List(args) => list(args, json),
        Command::Search(args) => search(args, json),
        Command::Update(args) => update(args, json),
        Command::Accord(args) => accord(args, json),
        Command::Review(args) => review(args, json),
        Command::Complete(args) => complete(args, json),
        Command::Cancel(args) => cancel(args, json),
        Command::Link(args) => link(args, json),
        Command::Sync(args) => sync_command(args, json),
        Command::Migrate(args) => migrate(args, json),
        Command::Rules(args) => rules(args, json),
        Command::Tui => Ok(super::StartupRequest::Tui),
        Command::Web(args) => Ok(super::StartupRequest::Web(crate::web::Options {
            port: args.port,
            no_open: args.no_open,
        })),
    }
}

fn add(args: AddArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open_for_write()?;
    match args.command {
        AddCommand::Task(task) => {
            let outcome = app::tasks::add(
                &project,
                app::tasks::AddOptions {
                    title: Some(task.title),
                    acceptance: task.acceptance,
                    description: task.body,
                    kind: task.kind,
                    priority: task.priority,
                    effort: task.effort,
                    tags: task.tag,
                    due_date: task.due_date,
                    parent: task
                        .parent
                        .map(|parent| project.current_id(&parent))
                        .transpose()?,
                    blockers: map_ids(&project, task.blocker)?,
                    references: task.reference,
                    related_files: task.related_file,
                    constraints: task.constraint,
                    validations: task.validation,
                    ..Default::default()
                },
            )?;
            let report = publish(&project);
            let id = report.renamed(&outcome.id);
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"id":id,"sync":sync_json(&report)},"warnings":outcome.warnings})
                );
            } else {
                print_warnings(&outcome.warnings);
                println!(
                    "Created task\nID: {id}\nTitle: {}\nSync: {}",
                    outcome.title,
                    sync_text(&report)
                );
            }
        }
        AddCommand::Decision(decision) => {
            let outcome = app::decisions::add(
                &project,
                app::decisions::AddOptions {
                    title: Some(decision.title),
                    body: decision.body,
                    deciders: decision.decider,
                    supersedes: decision.supersedes,
                    references: decision.reference,
                    tags: decision.tag,
                    ..Default::default()
                },
            )?;
            let report = publish(&project);
            let id = report.renamed(&outcome.id);
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"id":id,"sync":sync_json(&report)},"warnings":outcome.warnings})
                );
            } else {
                println!(
                    "Created decision\nID: {id}\nTitle: {}\nSync: {}",
                    outcome.title,
                    sync_text(&report)
                );
            }
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn assignment(args: IdArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, mut warnings) = open_read()?;
    let id = project.current_id(&args.id)?;
    let outcome = app::assignment::read(&project, &id)?;
    warnings.extend(outcome.warnings.iter().cloned());
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":outcome.data,"warnings":warnings})
        );
    } else {
        for warning in &warnings {
            eprintln!("Warning: {warning}");
        }
        println!(
            "Assignment {}\nDefinition token: {}\nDependencies clear: {}\nMilestones: {}",
            outcome.data.root.id,
            outcome.data.definition_token,
            outcome.data.dependency_readiness.all_clear,
            outcome.data.milestones.len()
        );
        for issue in outcome.data.dependency_readiness.issues {
            println!(
                "Blocked by {} ({}): {}",
                issue.id, issue.status, issue.reason
            );
        }
    }
    Ok(super::StartupRequest::Exit)
}

/// Rejects a `--kind` filter value outside the Task kind vocabulary.
fn validate_kind_filter(kind: Option<&str>) -> Result<(), CliError> {
    match kind {
        Some(kind) => crate::protocol::document::validate_task_kind(kind)
            .map_err(|message| CliError::user(format!("Validation failed: {message}"))),
        None => Ok(()),
    }
}

fn list(args: ListArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, warnings) = open_read()?;
    let docs = app::queries::documents_for_scope(
        &project,
        match args.scope {
            Scope::Active => app::queries::Scope::Active,
            Scope::Archived => app::queries::Scope::Archived,
            Scope::All => app::queries::Scope::All,
        },
    )?;
    validate_kind_filter(args.kind.as_deref())?;
    let filter = app::queries::ListFilter {
        state: args.state.as_deref(),
        kind: args.kind.as_deref(),
        doc_type: args.r#type.as_deref(),
        priority: args.priority.as_deref(),
        effort: args.effort.as_deref(),
        tags: &args.tag,
        assignee: args.assignee.as_deref(),
        parent: args.parent.as_deref(),
        accord: args.accord.as_deref(),
        decision_status: args.decision_status.as_deref(),
        resolution: args.resolution.as_deref(),
    };
    let mut documents = app::queries::filter_documents(docs, &filter);
    if args.link.is_some() || args.linked_to.is_some() {
        let linked_to = args
            .linked_to
            .as_deref()
            .map(|id| project.current_id(id))
            .transpose()?;
        documents = app::links::filter_documents(
            documents,
            &project.read_documents()?,
            args.link.as_deref(),
            linked_to.as_deref(),
        )?;
    }
    if let Some(limit) = args.limit {
        documents.truncate(limit);
    }
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":documents.iter().map(|d| serde_json::json!({"id":d.id(),"title":d.title()})).collect::<Vec<_>>(),"warnings":warnings})
        );
    } else {
        print_warnings(&warnings);
        for doc in documents {
            println!("{}\t{}", doc.id(), doc.title());
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn show(args: IdArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, sync_warnings) = open_read()?;
    if let Some(doc) = project.find_document(&args.id)? {
        if json {
            let read = app::queries::load_read(&project)?;
            let detail = app::dto::detail(&read, &doc)?;
            let mut warnings = sync_warnings;
            warnings.extend(read.warnings.iter().cloned());
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":detail,"warnings":warnings})
            );
        } else {
            print_warnings(&sync_warnings);
            let documents = project
                .read_documents()?
                .into_iter()
                .map(|document| (document.id().to_string(), document))
                .collect();
            let (outgoing, incoming) = app::links::dtos(&documents, doc.id());
            print!(
                "{}",
                show_text(&doc, app::links::text_lines(&outgoing, &incoming))
            );
        }
        return Ok(super::StartupRequest::Exit);
    }
    if let Some(rule) = app::queries::find_rule(&project, &args.id)? {
        if json {
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":{"id":rule.id,"category":rule.category,"text":rule.text},"warnings":[]})
            );
        } else {
            println!(
                "ID: {}\nCategory: {}\nRule: {}",
                rule.id, rule.category, rule.text
            );
        }
        return Ok(super::StartupRequest::Exit);
    }
    Err(CliError::user(format!("document not found: {}", args.id)))
}

/// Renders the short human identity-and-status block for `show`.
///
/// This is deliberately not a terminal rendering of the whole record. The TUI
/// is the human read surface; `--json` is the machine read surface.
fn show_text(doc: &crate::project::StoredDocument, link_lines: Vec<String>) -> String {
    let mut lines = vec![
        format!("ID: {}", doc.id()),
        format!("Type: {}", doc.doc_type()),
        format!("Title: {}", doc.title()),
        format!("Location: {}", doc.location.as_str()),
    ];
    if let Some(state) = doc.field("state") {
        lines.push(format!("State: {state}"));
    }
    if let Some(status) = crate::protocol::accord::status(doc) {
        lines.push(format!("Accord: {status}"));
    }
    if let Some(assignee) = doc.field("assignee") {
        lines.push(format!("Assignee: {assignee}"));
    }
    lines.extend(link_lines);
    lines.push(String::new());
    lines.join("\n")
}

fn search(args: SearchArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, warnings) = open_read()?;
    let docs = app::queries::documents_for_scope(
        &project,
        match args.scope {
            Scope::Active => app::queries::Scope::Active,
            Scope::Archived => app::queries::Scope::Archived,
            Scope::All => app::queries::Scope::All,
        },
    )?;
    validate_kind_filter(args.kind.as_deref())?;
    let filter = app::queries::SearchFilter {
        query: &args.query,
        state: args.state.as_deref(),
        kind: args.kind.as_deref(),
        doc_type: args.r#type.as_deref(),
        tags: &args.tag,
        parent: args.parent.as_deref(),
    };
    let mut results = app::queries::search_documents(docs, &filter);
    if let Some(limit) = args.limit {
        results.truncate(limit);
    }
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":results.iter().map(|r| serde_json::json!({"id":r.doc.id(),"title":r.doc.title(),"snippet":r.snippet})).collect::<Vec<_>>(),"warnings":warnings})
        );
    } else {
        print_warnings(&warnings);
        for result in results {
            println!(
                "{}\t{}\t{}",
                result.doc.id(),
                result.doc.title(),
                result.snippet
            );
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn update(mut args: UpdateArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, id) = open_write_for(&args.id)?;
    args.id = id;
    args.parent = args
        .parent
        .map(|parent| project.current_id(&parent))
        .transpose()?;
    args.blocker = map_ids(&project, args.blocker)?;
    let resolved_type = project
        .find_document(&args.id)?
        .map(|doc| doc.doc_type().to_string());
    match resolved_type.as_deref() {
        Some("task") => {
            reject_decision_only_flags(&args)?;
            let outcome = app::tasks::update(
                &project,
                app::tasks::UpdateOptions {
                    id: args.id,
                    title: args.title,
                    body: args.body,
                    kind: args.kind,
                    priority: args.priority,
                    effort: args.effort,
                    due_date: args.due_date,
                    parent: args.parent,
                    tags: args.tag,
                    blockers: args.blocker,
                    references: args.reference,
                    related_files: args.related_file,
                    acceptance: args.acceptance,
                    constraints: args.constraint,
                    validations: args.validation,
                    clear: args.clear,
                    ..Default::default()
                },
            )?;
            let report = publish(&project);
            print_update_outcome(
                json,
                &report.renamed(&outcome.id),
                &outcome.changes,
                &outcome.warnings,
                &report,
            );
        }
        Some("decision") => {
            reject_task_only_flags(&args)?;
            let outcome = app::decisions::update(
                &project,
                app::decisions::UpdateOptions {
                    id: args.id,
                    title: args.title,
                    body: args.body,
                    status: args.status,
                    deciders: args.decider,
                    supersedes: args.supersedes,
                    references: args.reference,
                    related_files: args.related_file,
                    tags: args.tag,
                    clear: args.clear,
                },
            )?;
            let report = publish(&project);
            print_update_outcome(
                json,
                &report.renamed(&outcome.id),
                &outcome.changes,
                &outcome.warnings,
                &report,
            );
        }
        Some(other) => {
            return Err(CliError::user(format!(
                "update does not support document type `{other}` in v0: {}",
                args.id
            )));
        }
        None => return Err(CliError::user(format!("document not found: {}", args.id))),
    }
    Ok(super::StartupRequest::Exit)
}

/// Rejects Decision-only flags on a resolved Task before any application write.
fn reject_decision_only_flags(args: &UpdateArgs) -> Result<(), CliError> {
    let mut invalid = Vec::new();
    if args.status.is_some() {
        invalid.push("--status");
    }
    if !args.decider.is_empty() {
        invalid.push("--decider");
    }
    if !args.supersedes.is_empty() {
        invalid.push("--supersedes");
    }
    reject_update_flags("task", &invalid)
}

/// Rejects Task-only flags on a resolved Decision before any application write.
fn reject_task_only_flags(args: &UpdateArgs) -> Result<(), CliError> {
    let mut invalid = Vec::new();
    if args.kind.is_some() {
        invalid.push("--kind");
    }
    if args.priority.is_some() {
        invalid.push("--priority");
    }
    if args.effort.is_some() {
        invalid.push("--effort");
    }
    if args.due_date.is_some() {
        invalid.push("--due-date");
    }
    if args.parent.is_some() {
        invalid.push("--parent");
    }
    if !args.blocker.is_empty() {
        invalid.push("--blocker");
    }
    if !args.acceptance.is_empty() {
        invalid.push("--acceptance");
    }
    if !args.constraint.is_empty() {
        invalid.push("--constraint");
    }
    if !args.validation.is_empty() {
        invalid.push("--validation");
    }
    reject_update_flags("decision", &invalid)
}

fn reject_update_flags(document_type: &str, flags: &[&str]) -> Result<(), CliError> {
    if flags.is_empty() {
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "update flags not valid for {document_type} documents: {}",
            flags.join(", ")
        )))
    }
}

/// Shared success output for the type-aware update command.
///
/// Warnings are returned in the JSON envelope and written to stderr in human
/// mode so they are never silently dropped.
fn print_update_outcome(
    json: bool,
    id: &str,
    changes: &[app::tasks::UpdateChange],
    warnings: &[String],
    report: &Report,
) {
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"changes":changes.iter().map(|c| &c.field).collect::<Vec<_>>(),"sync":sync_json(report)},"warnings":warnings})
        );
    } else {
        for warning in warnings {
            eprintln!("Warning: {warning}");
        }
        if changes.is_empty() {
            println!("No changes for {id}");
        } else {
            println!(
                "Updated {}: {}",
                id,
                changes
                    .iter()
                    .map(|change| change.field.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!("Sync: {}", sync_text(report));
    }
}

fn accord(args: AccordArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (action, id, options) = match args.command {
        AccordCommand::Claim(v) => (
            "claim",
            v.id,
            app::accord::AccordOptions {
                assignee: Some(v.assignee),
                ..Default::default()
            },
        ),
        AccordCommand::Deliver(v) => (
            "deliver",
            v.id,
            app::accord::AccordOptions {
                summary: Some(v.summary),
                evidence: v.evidence,
                files_changed: v.file_changed,
                ..Default::default()
            },
        ),
        AccordCommand::Rework(v) => (
            "rework",
            v.id,
            app::accord::AccordOptions {
                note: Some(v.note),
                ..Default::default()
            },
        ),
        AccordCommand::Block(v) => (
            "block",
            v.id,
            app::accord::AccordOptions {
                note: Some(v.note),
                ..Default::default()
            },
        ),
        AccordCommand::Resume(v) => ("resume", v.id, Default::default()),
        AccordCommand::Release(v) => (
            "release",
            v.id,
            app::accord::AccordOptions {
                note: Some(v.note),
                disposition: v.disposition,
                ..Default::default()
            },
        ),
        AccordCommand::Fail(v) => (
            "fail",
            v.id,
            app::accord::AccordOptions {
                note: Some(v.note),
                ..Default::default()
            },
        ),
    };
    let (project, id) = open_write_for(&id)?;
    let outcome = app::accord::transition(
        &project,
        action,
        app::accord::AccordOptions { id, ..options },
    )?;
    let report = publish(&project);
    let id = report.renamed(&outcome.id);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"status":outcome.status,"event":outcome.event_name,"recordWritten":true,"sync":sync_json(&report)},"warnings":[]})
        );
    } else {
        println!(
            "Accord {id}: {} (record written; {})",
            outcome.status,
            sync_text(&report)
        );
    }
    Ok(super::StartupRequest::Exit)
}

fn review(args: ReviewArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, id) = open_write_for(&args.id)?;
    let outcome = app::review::transition(
        &project,
        "request",
        app::review::ReviewOptions {
            id,
            criterion: Some(args.criterion),
            note: Some(args.note),
            reviewer: args.reviewer,
            ..Default::default()
        },
    )?;
    let report = publish(&project);
    let id = report.renamed(&outcome.id);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"state":outcome.state,"recordWritten":true,"sync":sync_json(&report)},"warnings":[]})
        );
    } else {
        println!(
            "Validation requested for {id} (record written; {})",
            sync_text(&report)
        );
    }
    Ok(super::StartupRequest::Exit)
}

fn complete(args: CompleteArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, id) = open_write_for(&args.id)?;
    let fixed_by = args
        .fixed_by
        .map(|target| project.current_id(&target))
        .transpose()?;
    let outcome = app::tasks::complete(
        &project,
        app::tasks::CompleteOptions {
            id,
            reviewer: args.reviewer,
            note: args.note,
            fixed_by,
        },
    )?;
    let report = publish(&project);
    let id = report.renamed(&outcome.id);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"recordWritten":true,"sync":sync_json(&report)},"warnings":outcome.warnings})
        );
    } else {
        for warning in &outcome.warnings {
            eprintln!("Warning: {warning}");
        }
        println!("Completed {id} (record written; {})", sync_text(&report));
    }
    Ok(super::StartupRequest::Exit)
}

fn link(args: LinkArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (adding, edit) = match args.command {
        LinkCommand::Add(edit) => (true, edit),
        LinkCommand::Remove(edit) => (false, edit),
    };
    let (project, id) = open_write_for(&edit.id)?;
    let target = project.current_id(&edit.target)?;
    let outcome = if adding {
        app::links::add(&project, &id, &edit.link_type, &target)?
    } else {
        app::links::remove(&project, &id, &edit.link_type, &target)?
    };
    let report = publish(&project);
    let id = report.renamed(&outcome.id);
    let target = report.renamed(&outcome.target);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"type":outcome.link_type,"target":target,"changed":outcome.changed,"recordWritten":outcome.changed,"sync":sync_json(&report)},"warnings":[]})
        );
    } else if !outcome.changed {
        println!("Unchanged: {id} already {} {target}", outcome.link_type);
    } else {
        println!(
            "{} {id} {} {target} (record written; {})",
            if adding { "Linked" } else { "Unlinked" },
            outcome.link_type,
            sync_text(&report)
        );
    }
    Ok(super::StartupRequest::Exit)
}

fn cancel(args: CancelArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let (project, id) = open_write_for(&args.id)?;
    let outcome = app::tasks::cancel(&project, &id, &args.note)?;
    let report = publish(&project);
    let id = report.renamed(&outcome.id);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":id,"recordWritten":true,"sync":sync_json(&report)},"warnings":[]})
        );
    } else {
        println!("Canceled {id} (record written; {})", sync_text(&report));
    }
    Ok(super::StartupRequest::Exit)
}

fn rules(args: RulesArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    if let RulesCommand::List { category } = args.command {
        let (project, mut warnings) = open_read()?;
        let mut values = crate::project::rules::read_rule_files(&project.rules_dir())?;
        if let Some(category) = category.as_deref() {
            values.retain(|rule| rule.category == category);
        }
        warnings.extend(app::project::warnings(&project)?);
        if json {
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":values.iter().map(|r| serde_json::json!({"id":r.id,"category":r.category,"rule":r.text,"source":r.source})).collect::<Vec<_>>(),"warnings":warnings})
            );
        } else {
            for warning in &warnings {
                println!("Warning: {warning}");
            }
            for r in values {
                println!("{}\t{}", r.id, r.text);
            }
        }
        return Ok(super::StartupRequest::Exit);
    }
    let (project, rule_id, verb) = match args.command {
        RulesCommand::List { .. } => unreachable!("handled above"),
        RulesCommand::Add {
            category,
            text,
            source,
        } => {
            let project = app::project::open_for_write()?;
            let o = app::rules::add(&project, &category, &text, source)?;
            (project, o.rule_id, "Created")
        }
        RulesCommand::Edit {
            id,
            text,
            source,
            clear,
        } => {
            let (project, id) = open_write_for(&id)?;
            let id = current_rule_id(&project, &id);
            let clear_source = clear.iter().any(|field| field == "source");
            let o = app::rules::edit(&project, &id, &text, source, clear_source)?;
            (project, o.rule_id, "Updated")
        }
        RulesCommand::Delete { id } => {
            let (project, id) = open_write_for(&id)?;
            let id = current_rule_id(&project, &id);
            let o = app::rules::delete(&project, &id)?;
            (project, o.rule_id, "Deleted")
        }
    };
    let report = publish(&project);
    let rule_id = report.renamed(&rule_id);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":rule_id,"sync":sync_json(&report)},"warnings":[]})
        );
    } else {
        println!("{verb} rule {rule_id} ({})", sync_text(&report));
    }
    Ok(super::StartupRequest::Exit)
}

/// Maps an outdated provisional rule ID to the rule's current ID.
fn current_rule_id(project: &TandemProject, id: &str) -> String {
    let Some((_, hex)) = crate::protocol::ids::provisional_parts(id) else {
        return id.to_string();
    };
    crate::project::rules::read_rule_files(&project.rules_dir())
        .ok()
        .and_then(|rules| {
            let matches: Vec<_> = rules
                .into_iter()
                .filter(|rule| {
                    rule.uid
                        .as_deref()
                        .is_some_and(|uid| uid.replace('-', "").starts_with(hex))
                })
                .collect();
            (matches.len() == 1).then(|| matches[0].id.clone())
        })
        .unwrap_or_else(|| id.to_string())
}

fn sync_command(args: SyncArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    if let Some(message) = app::project::historical(&project) {
        return Err(CliError::user(message));
    }
    match args.command {
        None => {
            let report = sync::sync(&project, Mode::Refresh)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"sync":sync_json(&report)},"warnings":[]})
                );
            } else {
                println!("Sync: {}", sync_text(&report));
            }
        }
        Some(SyncCommand::Status) => {
            let status = sync::status(&project)?;
            let last_fetch = status.last_fetch.map(crate::app::support::format_timestamp);
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{
                        "git": status.git,
                        "remote": status.remote,
                        "published": status.published,
                        "pending": status.pending,
                        "lastFetch": last_fetch,
                        "lastError": status.last_error,
                        "conflicts": status.conflicts.iter().map(|c| serde_json::json!({"id": c.id, "reason": c.reason})).collect::<Vec<_>>(),
                        "held": status.held.iter().map(|h| serde_json::json!({"path": h.path, "reason": h.reason})).collect::<Vec<_>>(),
                    },"warnings":[]})
                );
            } else if !status.git {
                println!("The board is not in a Git repository; it does not sync.");
            } else {
                match &status.remote {
                    Some(remote) => println!("Remote: {remote} (branch {})", sync::BRANCH),
                    None => println!("Remote: none (local-only board)"),
                }
                println!(
                    "Local changes: {}",
                    if status.pending {
                        "waiting to sync"
                    } else {
                        "none"
                    }
                );
                println!("Last fetch: {}", last_fetch.as_deref().unwrap_or("never"));
                if let Some(error) = &status.last_error {
                    println!("Last problem: {error}");
                }
                if status.conflicts.is_empty() && status.held.is_empty() {
                    println!("Conflicts: none");
                }
                for conflict in &status.conflicts {
                    println!(
                        "Conflict: {}: {}\n  resolve: tandem sync resolve {} --keep local|remote|edited",
                        conflict.id, conflict.reason, conflict.id
                    );
                }
                for held in &status.held {
                    println!("Held edit: {}: {}", held.path, held.reason);
                }
            }
        }
        Some(SyncCommand::Resolve(resolve)) => {
            let id = project.current_id(&resolve.id)?;
            let keep = match resolve.keep {
                KeepChoice::Local => sync::Keep::Local,
                KeepChoice::Remote => sync::Keep::Remote,
                KeepChoice::Edited => sync::Keep::Edited,
            };
            sync::resolve(&project, &id, keep)?;
            let report = sync::sync(&project, Mode::Refresh)
                .unwrap_or_else(|error| Report::pending(error.message));
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"id":report.renamed(&id),"sync":sync_json(&report)},"warnings":[]})
                );
            } else {
                println!("Resolved {id} ({})", sync_text(&report));
            }
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn migrate(args: MigrateArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let cwd = std::env::current_dir()?;
    if args.adopt {
        let report = crate::project::migrate::adopt(&cwd, args.dry_run)?;
        if json {
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":{
                    "dryRun": report.dry_run,
                    "remote": report.remote,
                    "newRecords": report.new_records,
                    "changedRecords": report.changed_records,
                    "renumbered": report.renumbered.iter().map(|(old, new)| serde_json::json!({"old": old, "new": new})).collect::<Vec<_>>(),
                    "unpushedCommits": report.unpushed_commits,
                    "needsPull": report.needs_pull,
                    "kindsSkipped": report.kinds_skipped.iter().map(|(id, reason)| serde_json::json!({"id": id, "reason": reason})).collect::<Vec<_>>(),
                    "sync": report.sync.as_ref().map(sync_json),
                },"warnings":[]})
            );
        } else {
            let prefix = if report.dry_run {
                "Would adopt"
            } else {
                "Adopted"
            };
            println!(
                "{prefix} {} new and {} changed record(s) from this machine into the shared board on {}.",
                report.new_records.len(),
                report.changed_records.len(),
                report.remote
            );
            for (old, new) in &report.renumbered {
                println!("  {old} is now {new}");
            }
            if let Some(sync_report) = &report.sync {
                println!("Sync: {}", sync_text(sync_report));
            }
            if !report.unpushed_commits.is_empty() {
                println!(
                    "These unpushed commits also changed .tandem/. Their board changes are adopted; when `git pull` reports conflicts in .tandem/, resolve them with `git rm -r --cached .tandem` and continue:"
                );
                for commit in &report.unpushed_commits {
                    println!("  {commit}");
                }
            }
            if !report.kinds_skipped.is_empty() {
                println!("Tags left as tags (not converted to kinds):");
                for (id, reason) in &report.kinds_skipped {
                    println!("  {id}: {reason}");
                }
            }
            if report.needs_pull {
                println!("Next: run `git pull`. The board returns automatically afterwards.");
            }
        }
    } else {
        let report = crate::project::migrate::migrate(&cwd, args.dry_run)?;
        let to_version = crate::protocol::config::PROTOCOL_VERSION;
        if json {
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":{
                    "dryRun": report.dry_run,
                    "fromVersion": report.from_version,
                    "toVersion": to_version,
                    "remote": report.remote,
                    "records": report.records,
                    "files": report.files,
                    "migratedFrom": report.migrated_from,
                    "sourceCommit": report.source_commit,
                    "upgraded": report.upgraded.map(|upgraded| match upgraded {
                        Upgraded::Published => "published",
                        Upgraded::Received => "received",
                        Upgraded::NotSynced => "notSynced",
                    }),
                    "kinds": {
                        "converted": report.kinds.converted.iter().map(|(id, kind)| serde_json::json!({"id": id, "kind": kind})).collect::<Vec<_>>(),
                        "skipped": report.kinds.skipped.iter().map(|(id, reason)| serde_json::json!({"id": id, "reason": reason})).collect::<Vec<_>>(),
                    },
                },"warnings":[]})
            );
        } else {
            let remote = report.remote.as_deref().unwrap_or("-");
            if report.from_version == "0.3.0" {
                if report.dry_run {
                    println!(
                        "Would move {} file(s) ({} record(s)) to the `tandem` branch on {remote} as protocol {to_version}, then create one source commit that stops tracking .tandem/.",
                        report.files, report.records
                    );
                } else {
                    println!(
                        "Moved {} record(s) to the `tandem` branch on {remote} as protocol {to_version}.\nCreated source commit {} that stops tracking .tandem/.\nNext: push it with `git push`, then run `git pull` (or `tandem migrate --adopt` if they have unpushed board changes) on your other machines.",
                        report.records,
                        report.source_commit.as_deref().unwrap_or("?")
                    );
                }
            } else if report.dry_run {
                println!(
                    "Would upgrade the board from protocol {} to {to_version} ({} file(s)).",
                    report.from_version, report.files
                );
            } else {
                match report.upgraded {
                    Some(Upgraded::Published) => println!(
                        "Upgraded the board from protocol {} to {to_version} and published it on the `tandem` branch of {remote}.\nNext: install this Tandem version on every other machine that shares this board, then run `tandem migrate` there.",
                        report.from_version
                    ),
                    Some(Upgraded::Received) => println!(
                        "Another machine already upgraded the shared board to protocol {to_version}; downloaded it."
                    ),
                    _ => println!(
                        "Upgraded the board from protocol {} to {to_version}. It is not synced to a remote, so nothing was published.",
                        report.from_version
                    ),
                }
            }
            if report.upgraded != Some(Upgraded::Received) {
                let verb = if report.dry_run {
                    "Would convert"
                } else {
                    "Converted"
                };
                println!(
                    "{verb} {} Task tag(s) to kinds.",
                    report.kinds.converted.len()
                );
                for (id, kind) in &report.kinds.converted {
                    println!("  {id} -> kind {kind}");
                }
                if !report.kinds.skipped.is_empty() {
                    println!("Left unchanged (nothing is guessed):");
                    for (id, reason) in &report.kinds.skipped {
                        println!("  {id}: {reason}");
                    }
                }
            }
        }
    }
    Ok(super::StartupRequest::Exit)
}
