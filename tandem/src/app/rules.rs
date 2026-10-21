//! Shared per-file Rules mutations (protocol 0.3.0, D46/D49).
//!
//! Each Rule is one Markdown file in `.tandem/rules/<category>-<id>.md` with a
//! composite id, the category stored in the record, optional source, and the
//! rule text as the body.

use std::collections::BTreeMap;

use crate::app::support::{current_timestamp as now_timestamp, document_exists, prepare_event};
use crate::app::Error;
use crate::project::rules::{
    delete_rule_file, next_rule_id, parse_rule_id, patch_rule_file, read_rule_files,
    write_rule_file, RuleRecord,
};
use crate::project::write::HierarchyLock;
use crate::project::yaml_double_quote;
use crate::project::TandemProject;
use crate::protocol::config::RULE_CATEGORIES;

#[derive(Debug)]
pub(crate) struct MutationOutcome {
    pub(crate) category: String,
    /// Rule number; 0 for a provisional rule that is not yet published.
    pub(crate) id: usize,
    /// Complete rule ID such as `always-12` or `always-new-3f9a2c1d`.
    pub(crate) rule_id: String,
    pub(crate) rule: String,
    pub(crate) warning: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DeleteOutcome {
    pub(crate) category: String,
    pub(crate) id: usize,
    pub(crate) rule_id: String,
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
    let uid = crate::app::support::new_uid();
    let id = if crate::project::sync::has_remote(project) {
        crate::protocol::ids::provisional_id(category, &uid)
    } else {
        next_rule_id(&project.rules_dir(), category).map_err(|e| Error::user(e.message))?
    };
    let now = now_timestamp();
    let record = RuleRecord {
        id: id.clone(),
        uid: Some(uid),
        category: category.to_string(),
        enabled: true,
        source,
        created_at: Some(now.clone()),
        updated_at: Some(now),
        text: rule.to_string(),
        path: project.rules_dir().join(format!("{id}.md")),
    };
    let prepared_events = prepare_event(
        project,
        "rules.updated",
        serde_json::json!({"action": "add", "ruleId": id}),
    )?;
    write_rule_file(&project.rules_dir(), &record).map_err(|e| Error::user(e.message))?;
    prepared_events.append_for_id("rules")?;
    Ok(MutationOutcome {
        category: category.to_string(),
        id: parse_rule_id(&record.id).map(|(_, n)| n).unwrap_or(0),
        rule_id: record.id.clone(),
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
    let _lock = HierarchyLock::acquire(project)?;
    let record = rule_for_write(project, id)?;
    let id = record.id.as_str();
    let (category, number) = parse_rule_id(id).expect("validated rule identity");
    let rule = require_rule_text(rule, "rules edit requires rule text")?;
    let source = if clear_source {
        None
    } else {
        normalized_source(source.or(record.source.clone()))
    };
    let warning = missing_source_warning(project, source.as_deref())?;
    let mut updates =
        BTreeMap::from([("updatedAt".to_string(), yaml_double_quote(&now_timestamp()))]);
    let removes = if let Some(source) = source {
        updates.insert("source".to_string(), yaml_double_quote(&source));
        Vec::new()
    } else {
        vec!["source"]
    };
    let prepared_events = prepare_event(
        project,
        "rules.updated",
        serde_json::json!({"action": "edit", "ruleId": id}),
    )?;
    patch_rule_file(project, &record.path, &updates, &removes, Some(rule))?;
    prepared_events.append_for_id("rules")?;
    Ok(MutationOutcome {
        category,
        id: number,
        rule_id: id.to_string(),
        rule: rule.to_string(),
        warning,
    })
}

/// All rules remain available to management; consumers opt into enabled selection.
pub(crate) fn list(
    project: &TandemProject,
    category: Option<&str>,
    all: bool,
) -> Result<Vec<RuleRecord>, Error> {
    Ok(read_rule_files(&project.rules_dir())?
        .into_iter()
        .filter(|rule| {
            (all || rule.enabled) && category.is_none_or(|category| category == rule.category)
        })
        .collect())
}

pub(crate) fn current_rule_id(project: &TandemProject, id: &str) -> Result<String, Error> {
    let Some((_, hex)) = crate::protocol::ids::provisional_parts(id) else {
        return Ok(id.to_string());
    };
    let matches: Vec<_> = read_rule_files(&project.rules_dir())?
        .into_iter()
        .filter(|rule| {
            rule.uid
                .as_deref()
                .is_some_and(|uid| uid.replace('-', "").starts_with(hex))
        })
        .collect();
    match matches.as_slice() {
        [] => Ok(id.to_string()),
        [rule] => Ok(rule.id.clone()),
        _ => Err(Error::user(format!("ambiguous provisional rule id: {id}"))),
    }
}

fn rule_for_write(project: &TandemProject, id: &str) -> Result<RuleRecord, Error> {
    if let Some(message) = crate::app::project::historical(project) {
        return Err(Error::user(format!("cannot change rules: {message}")));
    }
    let id = current_rule_id(project, id)?;
    project.refuse_held(&id)?;
    if let Some(conflict) = crate::project::sync::conflict_for(project, &id) {
        return Err(Error::user(format!(
            "{id} has an unresolved sync conflict: {}",
            conflict.reason
        )));
    }
    read_rule_files(&project.rules_dir())?
        .into_iter()
        .find(|rule| rule.id == id)
        .ok_or_else(|| Error::user(format!("rule not found: {id}")))
}

#[derive(Debug)]
pub(crate) struct StateOutcome {
    pub(crate) rule_id: String,
    pub(crate) enabled: bool,
    pub(crate) changed: bool,
}

pub(crate) fn set_enabled(
    project: &TandemProject,
    id: &str,
    enabled: bool,
) -> Result<StateOutcome, Error> {
    let _lock = HierarchyLock::acquire(project)?;
    let record = rule_for_write(project, id)?;
    let changed = record.enabled != enabled;
    if changed {
        let updates = BTreeMap::from([
            ("enabled".to_string(), enabled.to_string()),
            ("updatedAt".to_string(), yaml_double_quote(&now_timestamp())),
        ]);
        let events = prepare_event(
            project,
            "rules.updated",
            serde_json::json!({
                "action": if enabled { "enable" } else { "disable" }, "ruleId": record.id,
            }),
        )?;
        patch_rule_file(project, &record.path, &updates, &[], None)?;
        events.append_for_id("rules")?;
    }
    Ok(StateOutcome {
        rule_id: record.id,
        enabled,
        changed,
    })
}

pub(crate) fn delete(project: &TandemProject, id: &str) -> Result<DeleteOutcome, Error> {
    let _lock = HierarchyLock::acquire(project)?;
    let record = rule_for_write(project, id)?;
    let id = record.id.as_str();
    let (category, number) = parse_rule_id(id).ok_or_else(|| {
        Error::usage(format!(
            "invalid rule id `{id}`; expected <category>-<number>"
        ))
    })?;
    let rule_path = project.rules_dir().join(format!("{id}.md"));
    if !read_rule_files(&project.rules_dir())
        .map_err(|error| Error::user(error.message))?
        .iter()
        .any(|record| record.id == id)
    {
        return Err(Error::user(format!("rule not found: {id}")));
    }
    let prepared_events = prepare_event(
        project,
        "rules.updated",
        serde_json::json!({"action": "delete", "ruleId": id}),
    )?;
    delete_rule_file(&project.rules_dir(), id).map_err(|e| Error::user(e.message))?;
    crate::project::sync::note_removed(project, &rule_path)?;
    prepared_events.append_for_id("rules")?;
    Ok(DeleteOutcome {
        category,
        id: number,
        rule_id: id.to_string(),
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
            "---\nprotocolVersion: 0.5.0\nstates: [todo, in-progress, validation]\nunknown: retain\n---\nbody\n",
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
