//! Typed CLI-to-application conversion and dispatch.
use super::model::*;
use crate::project::{CheckpointOutcome, CheckpointStatus};
use crate::{app, CliError};

fn checkpoint_json(outcome: &CheckpointOutcome) -> serde_json::Value {
    match &outcome.status {
        CheckpointStatus::Batched => serde_json::json!({
            "status": "batched",
            "commit": serde_json::Value::Null,
            "amended": false,
        }),
        CheckpointStatus::Checkpointed => serde_json::json!({
            "status": "checkpointed",
            "commit": outcome.commit.as_deref(),
            "amended": false,
        }),
        CheckpointStatus::Clean => serde_json::json!({
            "status": "clean",
            "commit": serde_json::Value::Null,
            "amended": false,
        }),
        CheckpointStatus::Failed { message } => serde_json::json!({
            "status": "failed",
            "commit": serde_json::Value::Null,
            "amended": false,
            "error": message,
        }),
    }
}

fn checkpoint_text(outcome: &CheckpointOutcome) -> String {
    match &outcome.status {
        CheckpointStatus::Batched => "batched (awaiting assignment boundary)".to_string(),
        CheckpointStatus::Checkpointed => format!(
            "checkpointed{}",
            outcome
                .commit
                .as_deref()
                .map(|commit| format!(" ({commit})"))
                .unwrap_or_default()
        ),
        CheckpointStatus::Clean => "clean (no Tandem changes to checkpoint)".to_string(),
        CheckpointStatus::Failed { message } => format!("FAILED: {message}"),
    }
}

fn checkpoint_warning(outcome: &CheckpointOutcome) -> Option<String> {
    matches!(outcome.status, CheckpointStatus::Failed { .. })
        .then(|| format!("; Git checkpoint {}", checkpoint_text(outcome)))
}

pub(crate) fn dispatch(command: Command, json: bool) -> Result<super::StartupRequest, CliError> {
    match command {
        Command::Init(args) => {
            app::project::initialize(app::project::InitOptions {
                title: args.title,
                force: false,
            })?;
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
        Command::Rules(args) => rules(args, json),
        Command::Tui => Ok(super::StartupRequest::Tui),
        Command::Web(args) => Ok(super::StartupRequest::Web(crate::web::Options {
            port: args.port,
            no_open: args.no_open,
        })),
    }
}

fn add(args: AddArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
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
                    parent: task.parent,
                    blockers: task.blocker,
                    references: task.reference,
                    related_files: task.related_file,
                    constraints: task.constraint,
                    validations: task.validation,
                    ..Default::default()
                },
            )?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"id":outcome.id},"warnings":outcome.warnings})
                );
            } else {
                println!("Created task\nID: {}\nTitle: {}", outcome.id, outcome.title);
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
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":{"id":outcome.id},"warnings":outcome.warnings})
                );
            } else {
                println!(
                    "Created decision\nID: {}\nTitle: {}",
                    outcome.id, outcome.title
                );
            }
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn assignment(args: IdArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let outcome = app::assignment::read(&project, &args.id)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":outcome.data,"warnings":outcome.warnings})
        );
    } else {
        for warning in &outcome.warnings {
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

fn list(args: ListArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let docs = app::queries::documents_for_scope(
        &project,
        match args.scope {
            Scope::Active => app::queries::Scope::Active,
            Scope::Archived => app::queries::Scope::Archived,
            Scope::All => app::queries::Scope::All,
        },
    )?;
    let filter = app::queries::ListFilter {
        state: args.state.as_deref(),
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
    if let Some(limit) = args.limit {
        documents.truncate(limit);
    }
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":documents.iter().map(|d| serde_json::json!({"id":d.id(),"title":d.title()})).collect::<Vec<_>>(),"warnings":[]})
        );
    } else {
        for doc in documents {
            println!("{}\t{}", doc.id(), doc.title());
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn show(args: IdArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    if let Some(doc) = project.find_document(&args.id)? {
        if json {
            let read = app::queries::load_read(&project)?;
            let detail = app::dto::detail(&read, &doc)?;
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":detail,"warnings":read.warnings})
            );
        } else {
            print!("{}", show_text(&doc));
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
fn show_text(doc: &crate::project::StoredDocument) -> String {
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
    lines.push(String::new());
    lines.join("\n")
}

fn search(args: SearchArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let docs = app::queries::documents_for_scope(
        &project,
        match args.scope {
            Scope::Active => app::queries::Scope::Active,
            Scope::Archived => app::queries::Scope::Archived,
            Scope::All => app::queries::Scope::All,
        },
    )?;
    let filter = app::queries::SearchFilter {
        query: &args.query,
        state: args.state.as_deref(),
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
            serde_json::json!({"ok":true,"data":results.iter().map(|r| serde_json::json!({"id":r.doc.id(),"title":r.doc.title(),"snippet":r.snippet})).collect::<Vec<_>>(),"warnings":[]})
        );
    } else {
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

fn update(args: UpdateArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
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
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"changes":outcome.changes.iter().map(|c| &c.field).collect::<Vec<_>>()},"warnings":outcome.warnings})
        );
    } else if outcome.changes.is_empty() {
        println!("No changes for {}", outcome.id);
    } else {
        println!(
            "Updated {}: {}",
            outcome.id,
            outcome
                .changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(super::StartupRequest::Exit)
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
    let project = app::project::open()?;
    let outcome = app::accord::transition(
        &project,
        action,
        app::accord::AccordOptions { id, ..options },
    )?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"status":outcome.status,"event":outcome.event_name,"recordWritten":true,"checkpoint":checkpoint_json(&outcome.checkpoint)},"warnings":[]})
        );
    } else {
        println!(
            "Accord {}: {} (record written; Git {})",
            outcome.id,
            outcome.status,
            checkpoint_text(&outcome.checkpoint)
        );
        if let Some(warning) = checkpoint_warning(&outcome.checkpoint) {
            eprintln!("Warning: {warning}");
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn review(args: ReviewArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let outcome = app::review::transition(
        &project,
        "request",
        app::review::ReviewOptions {
            id: args.id,
            criterion: Some(args.criterion),
            note: Some(args.note),
            reviewer: args.reviewer,
            ..Default::default()
        },
    )?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"state":outcome.state,"recordWritten":true,"checkpoint":checkpoint_json(&outcome.checkpoint)},"warnings":[]})
        );
    } else {
        println!(
            "Validation requested for {} (record written; Git {})",
            outcome.id,
            checkpoint_text(&outcome.checkpoint)
        );
        if let Some(warning) = checkpoint_warning(&outcome.checkpoint) {
            eprintln!("Warning: {warning}");
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn complete(args: CompleteArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let outcome = app::tasks::complete(
        &project,
        app::tasks::CompleteOptions {
            id: args.id,
            reviewer: args.reviewer,
            ..Default::default()
        },
    )?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"recordWritten":true,"checkpoint":checkpoint_json(&outcome.checkpoint)},"warnings":outcome.warnings})
        );
    } else {
        println!(
            "Completed {} (record written; Git {})",
            outcome.id,
            checkpoint_text(&outcome.checkpoint)
        );
        if let Some(warning) = checkpoint_warning(&outcome.checkpoint) {
            eprintln!("Warning: {warning}");
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn cancel(args: CancelArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let outcome = app::tasks::cancel(&project, &args.id, &args.note)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"recordWritten":true,"checkpoint":checkpoint_json(&outcome.checkpoint)},"warnings":[]})
        );
    } else {
        println!(
            "Canceled {} (record written; Git {})",
            outcome.id,
            checkpoint_text(&outcome.checkpoint)
        );
        if let Some(warning) = checkpoint_warning(&outcome.checkpoint) {
            eprintln!("Warning: {warning}");
        }
    }
    Ok(super::StartupRequest::Exit)
}

fn rules(args: RulesArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    match args.command {
        RulesCommand::List { category } => {
            let mut values = crate::project::rules::read_rule_files(&project.rules_dir())?;
            if let Some(category) = category.as_deref() {
                values.retain(|rule| rule.category == category);
            }
            let warnings = app::project::warnings(&project)?;
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
        }
        RulesCommand::Add {
            category,
            text,
            source,
        } => {
            let o = app::rules::add(&project, &category, &text, source)?;
            println!(
                "{}",
                if json {
                    serde_json::json!({"ok":true,"data":{"id":format!("{}-{}",o.category,o.id)},"warnings":[]}).to_string()
                } else {
                    format!("Created rule {}-{}", o.category, o.id)
                }
            );
        }
        RulesCommand::Edit {
            id,
            text,
            source,
            clear,
        } => {
            let clear_source = clear.iter().any(|field| field == "source");
            let o = app::rules::edit(&project, &id, &text, source, clear_source)?;
            println!(
                "{}",
                if json {
                    serde_json::json!({"ok":true,"data":{"id":format!("{}-{}",o.category,o.id)},"warnings":[]}).to_string()
                } else {
                    format!("Updated rule {}-{}", o.category, o.id)
                }
            );
        }
        RulesCommand::Delete { id } => {
            let o = app::rules::delete(&project, &id)?;
            println!(
                "{}",
                if json {
                    serde_json::json!({"ok":true,"data":{"id":format!("{}-{}",o.category,o.id)},"warnings":[]}).to_string()
                } else {
                    format!("Deleted rule {}-{}", o.category, o.id)
                }
            );
        }
    }
    Ok(super::StartupRequest::Exit)
}
