//! Private support shared by Task and accord application use cases.
//!
//! `CliError` remains a temporary crate-root exception pending task-159; this
//! module otherwise depends only on project/protocol ownership boundaries.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::project::{
    self, display_path, parse_frontmatter_fields, split_frontmatter, ProjectHierarchy,
    StoredDocument as Document, TandemProject,
};
use crate::protocol::diagnostic::{metadata_diagnostics, Severity};
use crate::protocol::document::{parse_field_values, validate_task_kind};
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::workflow::{display_known_states, is_known_or_legacy_state, workflow_states};
use crate::CliError;

pub(crate) fn require_nonempty<'a>(
    value: Option<&'a str>,
    message: &str,
) -> Result<&'a str, CliError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::usage(message))
}

pub(crate) fn date_from_timestamp(timestamp: &str) -> String {
    timestamp.chars().take(10).collect()
}

pub(crate) fn document_exists(project: &TandemProject, id: &str) -> Result<bool, CliError> {
    Ok(project.find_document(id)?.is_some())
}

pub(crate) fn current_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_unix_timestamp(seconds)
}

pub(crate) fn append_event(
    project: &TandemProject,
    event_name: &str,
    id: &str,
    summary: &str,
) -> Result<(), CliError> {
    project::events::append_event(project, event_name, id, summary, &current_timestamp())
}

fn format_unix_timestamp(seconds: u64) -> String {
    let seconds = seconds as i64;
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
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + i64::from(month <= 2), month, day)
}

pub(crate) fn create_new_sequential_document<F>(
    project: &TandemProject,
    prefix: &str,
    content_for_id: F,
) -> Result<project::write::CreatedDocument, CliError>
where
    F: FnMut(&str) -> String,
{
    let hierarchy = hierarchy_from_project(project)?;
    let last_allocated = crate::protocol::ids::next_sequential_number(
        hierarchy.documents.values().map(|document| document.id()),
        prefix,
    );
    project::write::create_new_sequential_document_after(
        project,
        prefix,
        last_allocated,
        content_for_id,
    )
}

pub(crate) fn hierarchy_from_project(
    project: &TandemProject,
) -> Result<ProjectHierarchy, CliError> {
    let index = ProjectHierarchy::from_documents(project.read_documents()?)?;
    index.validate_document_metadata()?;
    Ok(index)
}

pub(crate) fn active_task_descendant_ids(
    hierarchy: &ProjectHierarchy,
    root_id: &str,
) -> Vec<String> {
    let mut visited = std::collections::BTreeSet::from([root_id.to_string()]);
    let mut pending = vec![root_id.to_string()];
    let mut active = std::collections::BTreeSet::new();
    while let Some(parent_id) = pending.pop() {
        for document in hierarchy.documents.values().filter(|doc| {
            doc.doc_type() == "task" && doc.field("parentId") == Some(parent_id.as_str())
        }) {
            if !visited.insert(document.id().to_string()) {
                continue;
            }
            if document.location == DocumentLocation::Board {
                active.insert(document.id().to_string());
            }
            pending.push(document.id().to_string());
        }
    }
    active.into_iter().collect()
}

pub(crate) fn resolve_parent_relationship(
    hierarchy: &ProjectHierarchy,
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

pub(crate) fn unresolved_blockers_in_hierarchy(
    hierarchy: &ProjectHierarchy,
    blockers: Option<&str>,
) -> Vec<String> {
    blockers
        .map(parse_field_values)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|blocker| match hierarchy.document(&blocker) {
            Some(document) if document.location != DocumentLocation::Board => None,
            Some(_) => Some(blocker),
            None => Some(format!("{blocker} (missing)")),
        })
        .collect()
}

pub(crate) fn workspace_deprecation_warnings(
    workspace: &TandemProject,
) -> Result<Vec<String>, CliError> {
    let content = workspace.read_config_raw()?;
    let (frontmatter, _) = split_frontmatter(&content).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {}: {message}",
            display_path(&workspace.config_path)
        ))
    })?;
    let fields = parse_frontmatter_fields(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(&workspace.config_path)
        ))
    })?;
    let mut warnings = Vec::new();
    if fields
        .keys()
        .any(|key| key == "types" || key.starts_with("types."))
        || frontmatter.lines().any(|line| line.trim() == "types:")
    {
        warnings.push("custom type declarations are deprecated and read-only; Tandem preserves them but does not create or mutate custom-type documents.".to_string());
    }
    if fields
        .keys()
        .any(|key| key == "completion" || key.starts_with("completion."))
        || frontmatter.lines().any(|line| line.trim() == "completion:")
    {
        warnings.push("project completion-policy settings are deprecated and ignored; Tandem always warns for missing review or accord acceptance and completes unless structural validation fails.".to_string());
    }
    Ok(warnings)
}

pub(crate) fn validate_state(project: &TandemProject, state: &str) -> Result<(), CliError> {
    if state.trim().is_empty() {
        return Err(CliError::usage("state must not be empty"));
    }
    let states = workflow_states(project.read_config_yaml()?.as_ref());
    if is_known_or_legacy_state(&states, state) {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: unknown state `{state}`; known states: {}",
            display_known_states(&states)
        )))
    }
}

pub(crate) fn validate_task_document_against_hierarchy(
    project: &TandemProject,
    document: &Document,
    hierarchy: &ProjectHierarchy,
) -> Result<(), CliError> {
    let mut errors = metadata_diagnostics(document, document.location == DocumentLocation::Logs)
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    if document.doc_type() != "task" {
        errors.push(format!(
            "expected type `task`, found `{}`",
            document.doc_type()
        ));
    }
    if let Some(kind) = document.field("kind") {
        if let Err(error) = validate_task_kind(kind) {
            errors.push(error);
        }
    }
    if document.location == DocumentLocation::Board {
        match document.field("state") {
            Some(state) if !state.trim().is_empty() => {
                if let Err(error) = validate_state(project, state) {
                    errors.push(error.message);
                }
            }
            _ => errors.push("missing required field `state`".to_string()),
        }
    }
    if let Some(parent) = document
        .field("parentId")
        .filter(|value| !value.trim().is_empty())
    {
        if hierarchy.document(parent).is_none() {
            errors.push(format!("unresolved parentId `{parent}`"));
        }
    }
    for blocker in document
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default()
    {
        if hierarchy.document(&blocker).is_none() {
            errors.push(format!("unresolved blocker `{blocker}`"));
        }
    }
    if errors.is_empty() {
        hierarchy.validate_all_task_hierarchies()?;
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed for {}: {}",
            display_path(&document.path),
            errors.join("; ")
        )))
    }
}
