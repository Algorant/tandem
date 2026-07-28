use super::output::require_nonempty;
use crate::app;
use crate::app::accord::AccordOptions;
use crate::app::decisions::AddOptions as DecisionAddOptions;
use crate::app::tasks::{AddOptions, CancelOptions, CompleteOptions, MoveOptions, UpdateOptions};
use crate::CliError;

#[derive(Debug, Default)]
pub(super) struct InitOptions {
    pub(super) title: Option<String>,
    pub(super) force: bool,
}

#[derive(Debug, Default)]
pub(super) struct ListOptions {
    pub(super) state: Option<String>,
    pub(super) doc_type: Option<String>,
    pub(super) priority: Option<String>,
    pub(super) tag: Option<String>,
    pub(super) assignee: Option<String>,
    pub(super) parent: Option<String>,
    pub(super) accord: Option<String>,
    pub(super) review: Option<String>,
    pub(super) json: bool,
}

#[derive(Debug, Default)]
pub(super) struct ShowOptions {
    pub(super) id: String,
    pub(super) json: bool,
}

#[derive(Debug, Default)]
pub(super) struct SearchOptions {
    pub(super) query: String,
    pub(super) state: Option<String>,
    pub(super) doc_type: Option<String>,
    pub(super) parent: Option<String>,
    pub(super) json: bool,
}

#[derive(Debug, Default)]
pub(super) struct LogListOptions {
    pub(super) limit: Option<usize>,
    pub(super) json: bool,
}

#[derive(Debug, Default)]
pub(super) struct CategoryListOptions {
    pub(super) category: Option<String>,
    pub(super) json: bool,
}

#[derive(Debug, Default)]
pub(super) struct RuleAddOptions {
    pub(super) category: Option<String>,
    pub(super) rule: Option<String>,
    pub(super) source: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct RuleEditOptions {
    pub(super) category: Option<String>,
    pub(super) id: Option<usize>,
    pub(super) rule: Option<String>,
    pub(super) source: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct RuleDeleteOptions {
    pub(super) category: Option<String>,
    pub(super) id: Option<usize>,
}

pub(super) fn parse_init_args(args: &[String]) -> Result<InitOptions, CliError> {
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

pub(super) fn parse_list_args(args: &[String]) -> Result<ListOptions, CliError> {
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

pub(super) fn parse_show_args(args: &[String]) -> Result<ShowOptions, CliError> {
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

pub(super) fn parse_add_args(args: &[String]) -> Result<AddOptions, CliError> {
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
            "--effort" => {
                index += 1;
                options.effort = Some(required_value(args, index, "--effort")?.to_string());
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

pub(super) fn parse_move_args(args: &[String]) -> Result<MoveOptions, CliError> {
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

pub(super) fn parse_update_args(args: &[String]) -> Result<UpdateOptions, CliError> {
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
            "--effort" => {
                index += 1;
                options.effort = Some(required_value(args, index, "--effort")?.to_string());
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

pub(super) fn parse_complete_args(args: &[String]) -> Result<CompleteOptions, CliError> {
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

pub(super) fn parse_cancel_args(args: &[String]) -> Result<CancelOptions, CliError> {
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

pub(super) fn parse_search_args(args: &[String]) -> Result<SearchOptions, CliError> {
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

pub(super) fn parse_log_search_args(args: &[String]) -> Result<SearchOptions, CliError> {
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

pub(super) fn parse_log_list_args(args: &[String]) -> Result<LogListOptions, CliError> {
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

pub(super) fn parse_category_list_args(
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

pub(super) fn parse_rule_add_args(args: &[String]) -> Result<RuleAddOptions, CliError> {
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

pub(super) fn parse_rule_edit_args(args: &[String]) -> Result<RuleEditOptions, CliError> {
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

pub(super) fn parse_rule_delete_args(args: &[String]) -> Result<RuleDeleteOptions, CliError> {
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

pub(super) fn parse_rule_id(value: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| CliError::usage("--id must be a positive integer"))
}

pub(super) fn parse_accord_args(action: &str, args: &[String]) -> Result<AccordOptions, CliError> {
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

pub(super) fn parse_json_only_args(args: &[String], command: &str) -> Result<bool, CliError> {
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

pub(super) fn required_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

pub(super) fn required_raw_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, CliError> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

pub(super) fn inline_subtask_authoring_error(command: &str) -> CliError {
    CliError::usage(format!(
        "{command} --subtask is deprecated; create a tracked subtask with `tandem add --title <title> --parent <task-id>`"
    ))
}

pub(super) fn set_single_positional(
    target: &mut String,
    value: &str,
    command: &str,
) -> Result<(), CliError> {
    if target.is_empty() {
        *target = value.to_string();
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "unexpected extra {command} argument `{value}`"
        )))
    }
}

#[derive(Debug, Default)]
pub(super) struct DecisionUpdateOptions {
    pub(super) id: String,
    pub(super) title: Option<String>,
    pub(super) body: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Debug)]
pub(super) struct DecisionWithdrawOptions {
    pub(super) id: String,
    pub(super) reason: String,
}

pub(super) fn parse_decision_add_args(args: &[String]) -> Result<DecisionAddOptions, CliError> {
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

pub(super) fn parse_decision_update_args(
    args: &[String],
) -> Result<DecisionUpdateOptions, CliError> {
    let Some((id, rest)) = args.split_first() else {
        return Err(CliError::usage("decision update requires <decision-id>"));
    };
    let mut options = DecisionUpdateOptions {
        id: id.clone(),
        ..Default::default()
    };
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--title" => {
                index += 1;
                options.title = Some(required_value(rest, index, "--title")?.to_string());
            }
            "--body" => {
                index += 1;
                options.body = Some(required_value(rest, index, "--body")?.to_string());
            }
            "--status" => {
                index += 1;
                options.status = Some(required_value(rest, index, "--status")?.to_string());
            }
            flag => {
                return Err(CliError::usage(format!(
                    "unknown decision update flag `{flag}`"
                )))
            }
        }
        index += 1;
    }
    if options.title.is_none() && options.body.is_none() && options.status.is_none() {
        return Err(CliError::usage(
            "decision update requires --title, --body, or --status",
        ));
    }
    if let Some(status) = options.status.as_deref() {
        app::decisions::validate_status(status)?;
    }
    Ok(options)
}

pub(super) fn parse_decision_withdraw_args(
    args: &[String],
) -> Result<DecisionWithdrawOptions, CliError> {
    let Some((id, rest)) = args.split_first() else {
        return Err(CliError::usage(
            "decision withdraw requires <decision-id> --reason <text>",
        ));
    };
    if rest.len() != 2 || rest[0] != "--reason" {
        return Err(CliError::usage(
            "decision withdraw requires <decision-id> --reason <text>",
        ));
    }
    Ok(DecisionWithdrawOptions {
        id: id.clone(),
        reason: require_nonempty(
            Some(&rest[1]),
            "decision withdraw --reason must not be empty",
        )?
        .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_parser_preserves_long_flags_and_rejects_unknown_flags() {
        let args = vec![
            "--state".into(),
            "validation".into(),
            "--parent".into(),
            "task-1".into(),
            "--json".into(),
        ];
        let options = parse_list_args(&args).unwrap();
        assert_eq!(options.state.as_deref(), Some("validation"));
        assert_eq!(options.parent.as_deref(), Some("task-1"));
        assert!(options.json);

        let error = parse_list_args(&["-j".into()]).unwrap_err();
        assert_eq!(error.code, 2);
        assert_eq!(error.message, "unknown list flag `-j`");
    }
}
