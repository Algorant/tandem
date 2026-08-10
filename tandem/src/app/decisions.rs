//! Shared Decision creation and diagnostic orchestration.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::support::{
    append_event, create_new_sequential_document, current_timestamp, date_from_timestamp,
    reference_target_exists,
};
use crate::project::write::{ensure_file_unchanged, read_file_snapshot};
use crate::project::{
    patch_frontmatter_content, replace_markdown_body, write_atomic, yaml_double_quote,
    TandemProject,
};
use crate::protocol::config::DECISION_STATUSES;
use crate::CliError;

#[derive(Debug, Default)]
pub(crate) struct AddOptions {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) deciders: Vec<String>,
    pub(crate) context: Option<String>,
    pub(crate) consequences: Vec<String>,
    pub(crate) alternatives: Vec<String>,
    pub(crate) supersedes: Vec<String>,
    pub(crate) superseded_by: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct AddOutcome {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) date: String,
    pub(crate) path: PathBuf,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct UpdateOptions {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) body: Option<String>,
}

#[derive(Debug)]
pub(crate) struct UpdateOutcome {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct WithdrawOutcome {
    pub(crate) id: String,
    pub(crate) reason: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn add(project: &TandemProject, options: AddOptions) -> Result<AddOutcome, CliError> {
    let title = require_nonempty(
        options.title.as_deref(),
        "decision add requires --title <title>",
    )?
    .to_string();
    let status = options.status.as_deref().unwrap_or("proposed");
    validate_status(status)?;
    validate_options(&options)?;
    let warnings = diagnostics(project, &options)?;
    let now = current_timestamp();
    let date = match options.date.as_deref() {
        Some(date) => {
            require_nonempty(Some(date), "decision add --date must not be empty")?.to_string()
        }
        None => date_from_timestamp(&now),
    };
    let created = create_new_sequential_document(project, "decision", |decision_id| {
        let mut lines = vec![
            "---".to_string(),
            format!("id: {decision_id}"),
            "type: decision".to_string(),
            format!("title: {}", yaml_double_quote(&title)),
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
    append_event(project, "decision.created", &created.id, &title)?;
    Ok(AddOutcome {
        id: created.id,
        title,
        status: status.to_string(),
        date,
        path: created.path,
        warnings,
    })
}

pub(crate) fn update(
    project: &TandemProject,
    options: UpdateOptions,
) -> Result<UpdateOutcome, CliError> {
    let doc = active_decision(project, &options.id)?;
    if let Some(status) = options.status.as_deref() {
        validate_status(status)?;
    }
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let mut updates = BTreeMap::new();
    if let Some(title) = options.title {
        updates.insert("title".to_string(), title);
    }
    if let Some(status) = options.status {
        updates.insert("status".to_string(), status);
    }
    updates.insert("updatedAt".to_string(), current_timestamp());
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    let patched = if let Some(body) = options.body.as_deref() {
        replace_markdown_body(&patched, body)?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        project,
        "decision.updated",
        doc.id(),
        &format!("Updated decision {}", doc.id()),
    )?;
    Ok(UpdateOutcome {
        id: doc.id().to_string(),
        path: doc.path,
    })
}

pub(crate) fn withdraw(
    project: &TandemProject,
    id: &str,
    reason: String,
) -> Result<WithdrawOutcome, CliError> {
    let doc = active_decision(project, id)?;
    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let updates = BTreeMap::from([
        ("status".to_string(), "withdrawn".to_string()),
        ("withdrawnAt".to_string(), now.clone()),
        ("withdrawalReason".to_string(), reason.clone()),
        ("updatedAt".to_string(), now),
    ]);
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        project,
        "decision.withdrawn",
        doc.id(),
        &format!("Withdrew decision {}: {reason}", doc.id()),
    )?;
    Ok(WithdrawOutcome {
        id: doc.id().to_string(),
        reason,
        path: doc.path,
    })
}

fn active_decision(
    project: &TandemProject,
    id: &str,
) -> Result<crate::project::StoredDocument, CliError> {
    project
        .read_board_document(id)?
        .filter(|doc| doc.doc_type() == "decision")
        .ok_or_else(|| CliError::user(format!("active decision not found: {id}")))
}

fn validate_options(options: &AddOptions) -> Result<(), CliError> {
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

pub(crate) fn validate_status(status: &str) -> Result<(), CliError> {
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

pub(crate) fn diagnostics(
    project: &TandemProject,
    options: &AddOptions,
) -> Result<Vec<String>, CliError> {
    let mut warnings = Vec::new();
    for reference in &options.references {
        if !reference_target_exists(project, reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }
    for target in &options.supersedes {
        push_reference_warning(project, &mut warnings, "supersedes", target)?;
    }
    for target in &options.superseded_by {
        push_reference_warning(project, &mut warnings, "supersededBy", target)?;
    }
    Ok(warnings)
}

fn push_reference_warning(
    project: &TandemProject,
    warnings: &mut Vec<String>,
    field: &str,
    id: &str,
) -> Result<(), CliError> {
    match project.find_document(id)? {
        Some(doc) if doc.doc_type() == "decision" => {}
        Some(doc) => warnings.push(format!(
            "{field} target {id} is type {}, not decision",
            doc.doc_type()
        )),
        None => warnings.push(format!("{field} decision not found: {id}")),
    }
    Ok(())
}

fn require_nonempty<'a>(value: Option<&'a str>, message: &str) -> Result<&'a str, CliError> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::usage(message))?;
    Ok(value)
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
fn push_optional_line(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("{key}: {}", yaml_double_quote(value.trim())));
    }
}
fn push_array_line(lines: &mut Vec<String>, key: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("{key}: {}", inline_array(values)));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn creation_preserves_adr_input_and_returns_reference_diagnostics() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-decision-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        let papercut_id = crate::app::papercuts::add(
            &project,
            crate::app::papercuts::AddOptions {
                title: Some("Decision friction".to_string()),
                ..Default::default()
            },
        )
        .unwrap()
        .papercut
        .id()
        .to_string();
        let outcome = add(
            &project,
            AddOptions {
                title: Some("Choose seam".to_string()),
                body: Some("## Decision\nKeep bytes.  ".to_string()),
                deciders: vec!["A".to_string()],
                references: vec![papercut_id.clone(), "missing-task".to_string()],
                supersedes: vec!["missing-decision".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(outcome.id, "decision-1");
        assert_eq!(
            outcome.warnings,
            vec![
                "reference not found: missing-task".to_string(),
                "supersedes decision not found: missing-decision".to_string()
            ]
        );
        let source = fs::read_to_string(outcome.path).unwrap();
        assert!(source.contains("deciders: [\"A\"]"));
        assert!(source.contains(&format!(
            "references: [\"{papercut_id}\", \"missing-task\"]"
        )));
        assert!(source.contains("## Decision\nKeep bytes.  \n"));
        fs::remove_dir_all(root).unwrap();
    }
}
