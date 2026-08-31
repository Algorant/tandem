//! Shared per-file Rules mutations (protocol 0.3.0, D46/D49).
//!
//! Each Rule is one Markdown file in `.tandem/rules/<category>-<id>.md` with a
//! composite id, the category stored in the record, optional source, and the
//! rule text as the body.

use crate::app::support::{append_event, current_timestamp as now_timestamp, document_exists};
use crate::app::Error;
use crate::project::rules::{
    delete_rule_file, next_rule_id, parse_rule_id, read_rule_files, write_rule_file, RuleRecord,
};
use crate::project::TandemProject;
use crate::protocol::config::RULE_CATEGORIES;

#[derive(Debug)]
pub(crate) struct MutationOutcome {
    pub(crate) category: String,
    pub(crate) id: usize,
    pub(crate) rule: String,
    pub(crate) warning: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DeleteOutcome {
    pub(crate) category: String,
    pub(crate) id: usize,
}

pub(crate) fn add(
    project: &TandemProject,
    category: &str,
    rule: &str,
    source: Option<String>,
) -> Result<MutationOutcome, Error> {
    validate_rule_category(category)?;
    let rule = require_rule_text(rule, "rules add requires rule text")?;
    let source = normalized_source(source);
    let warning = missing_source_warning(project, source.as_deref())?;
    let id = next_rule_id(&project.rules_dir(), category).map_err(|e| Error::user(e.message))?;
    let now = now_timestamp();
    let record = RuleRecord {
        id: id.clone(),
        category: category.to_string(),
        source,
        created_at: Some(now.clone()),
        updated_at: Some(now),
        text: rule.to_string(),
        path: project.rules_dir().join(format!("{id}.md")),
    };
    write_rule_file(&project.rules_dir(), &record).map_err(|e| Error::user(e.message))?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Added rule {id}"),
    )?;
    Ok(MutationOutcome {
        category: category.to_string(),
        id: record.id.rsplit_once('-').unwrap().1.parse().unwrap(),
        rule: rule.to_string(),
        warning,
    })
}

pub(crate) fn edit(
    project: &TandemProject,
    id: &str,
    rule: &str,
    source: Option<String>,
    clear_source: bool,
) -> Result<MutationOutcome, Error> {
    let (category, number) = parse_rule_id(id).ok_or_else(|| {
        Error::usage(format!(
            "invalid rule id `{id}`; expected <category>-<number>"
        ))
    })?;
    let rule = require_rule_text(rule, "rules edit requires rule text")?;
    let dir = project.rules_dir();
    let mut record = read_rule_files(&dir)
        .map_err(|e| Error::user(e.message))?
        .into_iter()
        .find(|record| record.id == id)
        .ok_or_else(|| Error::user(format!("rule not found: {id}")))?;
    let source = if clear_source {
        None
    } else {
        normalized_source(source.or(record.source.clone()))
    };
    let warning = missing_source_warning(project, source.as_deref())?;
    record.text = rule.to_string();
    record.source = source;
    record.updated_at = Some(now_timestamp());
    write_rule_file(&dir, &record).map_err(|e| Error::user(e.message))?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Edited rule {id}"),
    )?;
    Ok(MutationOutcome {
        category,
        id: number,
        rule: rule.to_string(),
        warning,
    })
}

pub(crate) fn delete(project: &TandemProject, id: &str) -> Result<DeleteOutcome, Error> {
    let (category, number) = parse_rule_id(id).ok_or_else(|| {
        Error::usage(format!(
            "invalid rule id `{id}`; expected <category>-<number>"
        ))
    })?;
    delete_rule_file(&project.rules_dir(), id).map_err(|e| Error::user(e.message))?;
    append_event(
        project,
        "rules.updated",
        "rules",
        &format!("Deleted rule {id}"),
    )?;
    Ok(DeleteOutcome {
        category,
        id: number,
    })
}
pub(crate) fn validate_rule_category(category: &str) -> Result<(), Error> {
    if RULE_CATEGORIES.contains(&category) {
        Ok(())
    } else {
        Err(Error::usage(format!(
            "unknown rule category `{category}`; use always, never, prefer, or context"
        )))
    }
}

fn require_rule_text<'a>(value: &'a str, message: &str) -> Result<&'a str, Error> {
    let value = value.trim();
    if value.is_empty() {
        Err(Error::usage(message))
    } else {
        Ok(value)
    }
}

fn normalized_source(source: Option<String>) -> Option<String> {
    source.and_then(|value| (!value.trim().is_empty()).then_some(value))
}

fn missing_source_warning(
    project: &TandemProject,
    source: Option<&str>,
) -> Result<Option<String>, Error> {
    if let Some(source) = source {
        if !document_exists(project, source)? {
            return Ok(Some(format!("rule source not found: {source}")));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn add_edit_delete_round_trip_per_file_and_return_source_warning() {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-rules-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.2.0\nstates: [todo, in-progress, validation]\nunknown: retain\n---\nbody\n",
        )
        .unwrap();
        let papercut_id = "task-99".to_string();
        let added = add(&project, "always", "Keep it", Some(" missing ".to_string())).unwrap();
        assert_eq!(added.id, 1);
        assert_eq!(
            added.warning.as_deref(),
            Some("rule source not found:  missing ")
        );
        assert!(project.rules_dir().join("always-1.md").is_file());
        let file = fs::read_to_string(project.rules_dir().join("always-1.md")).unwrap();
        assert!(file.contains("id: always-1"));
        assert!(file.contains("category: always"));
        assert!(file.ends_with("Keep it\n"));

        edit(&project, "always-1", "Keep all", None, false).unwrap();
        assert!(!fs::read_to_string(project.rules_dir().join("always-1.md"))
            .unwrap()
            .contains("Keep it"));
        assert!(fs::read_to_string(project.rules_dir().join("always-1.md"))
            .unwrap()
            .contains("Keep all"));

        delete(&project, "always-1").unwrap();
        assert!(!project.rules_dir().join("always-1.md").exists());

        let missing_source = add(
            &project,
            "always",
            "Rule sources must reference existing documents",
            Some(papercut_id.clone()),
        )
        .unwrap();
        assert_eq!(
            missing_source.warning,
            Some(format!("rule source not found: {papercut_id}"))
        );
        let read = crate::app::queries::load_read(&project).unwrap();
        assert!(read.warnings.iter().any(|warning| {
            warning
                == &format!(
                    "Rule {} references missing source {papercut_id}.",
                    missing_source.id
                )
        }));
        fs::remove_dir_all(root).unwrap();
    }
}
