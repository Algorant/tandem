//! Typed CLI-to-application conversion and dispatch.
use super::model::*;
use crate::{app, CliError};

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
                    serde_json::json!({"ok":true,"data":{"id":outcome.id},"warnings":[]})
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
            println!(
                "{}",
                serde_json::json!({"ok":true,"data":{"id":doc.id(),"type":doc.doc_type(),"title":doc.title()},"warnings":[]})
            );
        } else {
            println!(
                "ID: {}\nType: {}\nTitle: {}",
                doc.id(),
                doc.doc_type(),
                doc.title()
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
            clear: args.clear,
            ..Default::default()
        },
    )?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"changes":outcome.changes.iter().map(|c| &c.field).collect::<Vec<_>>()},"warnings":outcome.warnings})
        );
    } else {
        println!("Updated {}", outcome.id);
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
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"status":outcome.status,"event":outcome.event_name},"warnings":[]})
        );
    } else {
        println!("Accord {}: {}", outcome.id, outcome.status);
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
            serde_json::json!({"ok":true,"data":{"id":outcome.id,"state":outcome.state},"warnings":[]})
        );
    } else {
        println!("Validation requested for {}", outcome.id);
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
            serde_json::json!({"ok":true,"data":{"id":outcome.id},"warnings":outcome.warnings})
        );
    } else {
        println!("Completed {}", outcome.id);
    }
    Ok(super::StartupRequest::Exit)
}

fn cancel(args: CancelArgs, json: bool) -> Result<super::StartupRequest, CliError> {
    let project = app::project::open()?;
    let outcome = app::tasks::cancel(&project, &args.id, &args.note)?;
    if json {
        println!(
            "{}",
            serde_json::json!({"ok":true,"data":{"id":outcome.id},"warnings":[]})
        );
    } else {
        println!("Canceled {}", outcome.id);
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
            if json {
                println!(
                    "{}",
                    serde_json::json!({"ok":true,"data":values.iter().map(|r| serde_json::json!({"id":r.id,"category":r.category,"rule":r.text,"source":r.source})).collect::<Vec<_>>(),"warnings":[]})
                );
            } else {
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
