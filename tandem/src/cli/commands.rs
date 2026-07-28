//! Thin command adapters over shared application and project queries.

use super::args::*;
use super::output::*;
use crate::app::accord::AccordOptions;
use crate::app::tasks::{AddOptions, CancelOptions, CompleteOptions, MoveOptions, UpdateOptions};
use crate::project::rules::read_rules;
use crate::project::TandemProject;
use crate::protocol::accord;
use crate::protocol::config::{LEGACY_PROTOCOL_VERSION, PROTOCOL_VERSION};
use crate::protocol::hierarchy::ParentRelationship;
use crate::{app, CliError};

type DecisionAddOptions = app::decisions::AddOptions;

pub(super) fn cmd_init(options: InitOptions) -> Result<(), CliError> {
    let outcome = app::project::initialize(app::project::InitOptions {
        title: options.title,
        force: options.force,
    })?;
    println!("Created Tandem workspace");
    println!("Title: {}", outcome.title);
    println!("Config: {}", display_path(&outcome.project.config_path));
    println!("Board:  {}", display_path(&outcome.project.board_dir));
    println!("Logs:   {}", display_path(&outcome.project.logs_dir));
    println!("Events: {}", display_path(&outcome.project.events_dir()));
    println!("States: todo, in-progress, validation");
    Ok(())
}

pub(super) fn cmd_list(options: ListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let mut filtered = snapshot.board_documents(&app::queries::ListFilter {
        state: options.state.as_deref(),
        doc_type: options.doc_type.as_deref(),
        priority: options.priority.as_deref(),
        tag: options.tag.as_deref(),
        assignee: options.assignee.as_deref(),
        parent: options.parent.as_deref(),
        accord: options.accord.as_deref(),
        review: options.review.as_deref(),
    });
    sort_documents(&mut filtered);
    let relationships = snapshot.relationships_for(&filtered)?;

    if options.json {
        println!("{}", list_json(&filtered, &relationships)?);
    } else {
        print_workspace_deprecation_warnings(&workspace)?;
        print_list_table(&filtered, &relationships)?;
        print_document_warnings(&filtered);
    }

    Ok(())
}

pub(super) fn cmd_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let hierarchy = &snapshot.hierarchy;
    let doc = snapshot
        .document(&options.id)
        .ok_or_else(|| CliError::user(format!("document not found: {}", options.id)))?;
    let relationship = hierarchy.relationship(&doc)?;
    let children = snapshot.children(&doc)?;
    let role = hierarchy.task_role(&doc)?;

    if options.json {
        println!("{}", show_json(&doc, &children, role, relationship));
    } else {
        print_workspace_deprecation_warnings(&workspace)?;
        print_show(&doc, &children, role, relationship);
        print_document_warnings(&[doc]);
    }

    Ok(())
}

pub(super) fn cmd_add(options: AddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let json = options.json;
    let outcome = app::tasks::add(&workspace, options)?;

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

pub(super) fn cmd_move(options: MoveOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let state = options
        .state
        .as_deref()
        .ok_or_else(|| CliError::usage("move requires --state <state>"))?;
    let outcome = app::tasks::move_to_state(&workspace, &options.id, state)?;

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

pub(super) fn cmd_update(options: UpdateOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::tasks::update(&workspace, options)?;

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
                app::tasks::display_change_field(&change.field, outcome.parent_relationship),
                app::tasks::display_change_value(&change.old),
                app::tasks::display_change_value(&change.new)
            );
        }
    }
    println!("Path: {}", display_path(&outcome.path));
    Ok(())
}

pub(super) fn cmd_complete(options: CompleteOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::tasks::complete(&workspace, options)?;
    for warning in &outcome.warnings {
        println!("Warning: {warning}");
    }
    if outcome.has_completion_warnings {
        println!("Completing anyway under the canonical protocol policy.\n");
    }
    println!("Completed {}", outcome.id);
    println!(
        "Moved: {} -> {}",
        display_path(&outcome.board_path),
        display_path(&outcome.log_path)
    );
    println!("Event: task.completed");
    Ok(())
}

pub(super) fn cmd_cancel(options: CancelOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let reason = require_nonempty(options.reason.as_deref(), "cancel requires --reason <text>")?;
    let outcome = app::tasks::cancel(&workspace, &options.id, reason)?;

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

pub(super) fn cmd_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let docs = snapshot.all_documents();
    let results = app::queries::search_documents(
        docs,
        &app::queries::SearchFilter {
            query: &options.query,
            state: options.state.as_deref(),
            doc_type: options.doc_type.as_deref(),
            parent: options.parent.as_deref(),
        },
    );
    let relationships = snapshot.relationships_for(
        &results
            .iter()
            .map(|result| result.doc.clone())
            .collect::<Vec<_>>(),
    )?;

    if options.json {
        println!("{}", search_json(&options.query, &results, &relationships)?);
    } else {
        print_workspace_deprecation_warnings(&workspace)?;
        print_search_table(&results, &relationships)?;
        print_document_warnings(
            &results
                .iter()
                .map(|result| result.doc.clone())
                .collect::<Vec<_>>(),
        );
    }
    Ok(())
}

pub(super) fn cmd_log(args: &[String]) -> Result<(), CliError> {
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

pub(super) fn cmd_log_list(options: LogListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let mut docs = snapshot.log_documents();
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

pub(super) fn cmd_log_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let hierarchy = &snapshot.hierarchy;
    let doc = snapshot
        .log_document(&options.id)
        .ok_or_else(|| CliError::user(format!("log document not found: {}", options.id)))?;
    let relationship = hierarchy.relationship(&doc)?;
    if options.json {
        println!("{}", log_show_json(&doc, relationship));
    } else {
        print_log_show(&doc, relationship);
    }
    Ok(())
}

pub(super) fn cmd_log_search(options: SearchOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let mut results = snapshot
        .log_documents()
        .into_iter()
        .filter_map(|doc| app::queries::search_match(doc, &options.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));
    let relationships = snapshot.relationships_for(
        &results
            .iter()
            .map(|result| result.doc.clone())
            .collect::<Vec<_>>(),
    )?;
    if options.json {
        println!("{}", search_json(&options.query, &results, &relationships)?);
    } else {
        print_search_table(&results, &relationships)?;
    }
    Ok(())
}

pub(super) fn accord_actions_help() -> String {
    accord::ACTIONS.join("|")
}

pub(super) fn accord_actions_usage() -> String {
    let (last, leading) = accord::ACTIONS
        .split_last()
        .expect("accord actions must not be empty");
    format!("{}, or {last}", leading.join(", "))
}

pub(super) fn cmd_accord(args: &[String]) -> Result<(), CliError> {
    let Some((action, rest)) = args.split_first() else {
        return Err(CliError::usage(format!(
            "tandem accord requires {}",
            accord_actions_usage()
        )));
    };
    let status = accord::status_for_action(action).ok_or_else(|| {
        CliError::usage(format!(
            "unknown accord subcommand `{action}`; use {}",
            accord_actions_usage()
        ))
    })?;
    let options = parse_accord_args(action, rest)?;
    cmd_accord_update(action, status, options)
}

pub(super) fn cmd_accord_update(
    action: &str,
    _status: &str,
    options: AccordOptions,
) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::accord::transition(&workspace, action, options)?;
    print_accord_update(
        &outcome.id,
        &outcome.previous_status,
        &outcome.status,
        &outcome.event_name,
        &outcome.path,
    );
    if let Some(state) = outcome.synced_state.as_deref() {
        println!("State:  {} -> {state}", outcome.previous_state);
    }
    Ok(())
}

pub(super) fn cmd_rules(args: &[String]) -> Result<(), CliError> {
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

pub(super) fn cmd_rules_list(options: CategoryListOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    if let Some(category) = options.category.as_deref() {
        app::rules::validate_rule_category(category)?;
    }
    let rules = read_rules(&workspace.config_path)?;
    if options.json {
        println!("{}", rules_json(&rules, options.category.as_deref()));
    } else {
        print_rules(&rules, options.category.as_deref());
    }
    Ok(())
}

pub(super) fn cmd_rules_add(options: RuleAddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let rule = require_nonempty(options.rule.as_deref(), "rules add requires --rule <text>")?;
    let outcome = app::rules::add(&workspace, category, rule, options.source)?;
    if let Some(warning) = outcome.warning {
        println!("Warning: {warning}");
    }
    println!("Added rule");
    println!("Category: {}", outcome.category);
    println!("ID:       {}", outcome.id);
    println!("Rule:     {}", outcome.rule);
    Ok(())
}

pub(super) fn cmd_rules_edit(options: RuleEditOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let id = options
        .id
        .ok_or_else(|| CliError::usage("rules edit requires --id <rule-id>"))?;
    let rule = require_nonempty(options.rule.as_deref(), "rules edit requires --rule <text>")?;
    let outcome = app::rules::edit(&workspace, category, id, rule, options.source)?;
    if let Some(warning) = outcome.warning {
        println!("Warning: {warning}");
    }
    println!("Edited rule");
    println!("Category: {}", outcome.category);
    println!("ID:       {}", outcome.id);
    println!("Rule:     {}", outcome.rule);
    Ok(())
}

pub(super) fn cmd_rules_delete(options: RuleDeleteOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let category = require_rule_category(options.category.as_deref())?;
    let id = options
        .id
        .ok_or_else(|| CliError::usage("rules delete requires --id <rule-id>"))?;

    let outcome = app::rules::delete(&workspace, category, id)?;
    println!("Deleted rule");
    println!("Category: {}", outcome.category);
    println!("ID:       {}", outcome.id);
    Ok(())
}

pub(super) fn cmd_decision(args: &[String]) -> Result<(), CliError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(CliError::usage(
            "tandem decision requires list, show, add, update, or withdraw",
        ));
    };
    match subcommand.as_str() {
        "list" => cmd_decision_list(parse_json_only_args(rest, "decision list")?),
        "show" => cmd_decision_show(parse_show_args(rest)?),
        "add" => cmd_decision_add(parse_decision_add_args(rest)?),
        "update" => cmd_decision_update(parse_decision_update_args(rest)?),
        "withdraw" => cmd_decision_withdraw(parse_decision_withdraw_args(rest)?),
        other => Err(CliError::usage(format!(
            "unknown decision subcommand `{other}`; use list, show, add, update, or withdraw"
        ))),
    }
}

pub(super) fn cmd_decision_list(json: bool) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut docs = workspace
        .read_board_documents()?
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

pub(super) fn cmd_decision_show(options: ShowOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let snapshot = app::queries::load(&workspace)?;
    let hierarchy = &snapshot.hierarchy;
    let doc = snapshot
        .document(&options.id)
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

pub(super) fn cmd_decision_update(options: DecisionUpdateOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::decisions::update(
        &workspace,
        app::decisions::UpdateOptions {
            id: options.id,
            title: options.title,
            status: options.status,
            body: options.body,
        },
    )?;
    println!(
        "Updated decision\nID:    {}\nPath:  {}",
        outcome.id,
        display_path(&outcome.path)
    );
    Ok(())
}

pub(super) fn cmd_decision_withdraw(options: DecisionWithdrawOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::decisions::withdraw(&workspace, &options.id, options.reason)?;
    println!(
        "Withdrew decision\nID:      {}\nReason:  {}\nPath:    {}",
        outcome.id,
        outcome.reason,
        display_path(&outcome.path)
    );
    Ok(())
}

pub(super) fn cmd_decision_add(options: DecisionAddOptions) -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let outcome = app::decisions::add(&workspace, options)?;
    for warning in outcome.warnings {
        println!("Warning: {warning}");
    }
    println!("Created decision");
    println!("ID:     {}", outcome.id);
    println!("Status: {}", outcome.status);
    println!("Date:   {}", outcome.date);
    println!("Title:  {}", outcome.title);
    println!("Path:   {}", display_path(&outcome.path));
    Ok(())
}

pub(super) fn discover_workspace() -> Result<TandemProject, CliError> {
    app::project::open()
}

pub(super) fn cmd_upgrade(args: &[String]) -> Result<(), CliError> {
    if !args.is_empty() {
        return Err(CliError::usage("tandem upgrade does not accept options"));
    }
    match app::project::upgrade()? {
        app::project::UpgradeOutcome::AlreadyCurrent => {
            println!("Tandem project is already at protocol {PROTOCOL_VERSION}.");
        }
        app::project::UpgradeOutcome::Upgraded => {
            println!(
                "Upgraded Tandem project protocol: {LEGACY_PROTOCOL_VERSION} -> {PROTOCOL_VERSION}"
            );
            println!(
                "Preserved existing documents, configuration, events, and logs without conversion."
            );
        }
    }
    Ok(())
}

fn print_workspace_deprecation_warnings(workspace: &TandemProject) -> Result<(), CliError> {
    for warning in app::project::warnings(workspace)? {
        println!("Warning: {warning}");
    }
    Ok(())
}
