//! Shared Papercut inbox use cases.
//!
//! This module coordinates protocol validation with project-local storage.
//! Papercuts remain outside the general document and workflow machinery.

use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::path::PathBuf;

use crate::app::support::{append_event, current_timestamp, require_nonempty};
use crate::project::write::{
    create_new_sequential_file_after, ensure_file_unchanged, read_file_snapshot, HierarchyLock,
};
use crate::project::{
    patch_frontmatter_content, patch_papercut_resolution_content, write_atomic, yaml_double_quote,
    StoredPapercut, TandemProject,
};
use crate::protocol::papercut::{papercut_number, STATUSES};
use crate::CliError;

#[derive(Debug, Default)]
pub(crate) struct AddOptions {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) references: Vec<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct ListOptions<'a> {
    pub(crate) status: Option<&'a str>,
    pub(crate) all: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ResolveOptions {
    pub(crate) id: String,
    pub(crate) note: Option<String>,
    pub(crate) references: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct MutationOutcome {
    pub(crate) papercut: StoredPapercut,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct InboxLoad {
    pub(crate) items: Vec<StoredPapercut>,
    pub(crate) warnings: Vec<String>,
}

/// Loads the read-only open inbox without allowing one malformed record to
/// hide valid Papercuts or affect unrelated project reads.
pub(crate) fn load_open_inbox(project: &TandemProject) -> InboxLoad {
    let mut warnings = Vec::new();
    let mut items = project
        .read_papercuts_tolerant(&mut warnings)
        .into_iter()
        .filter(|item| item.status() == "open")
        .collect::<Vec<_>>();
    items.sort_by_key(|item| papercut_number(item.id()).unwrap_or(usize::MAX));
    match warnings_for_items(project, &items) {
        Ok(reference_warnings) => warnings.extend(reference_warnings),
        Err(error) => warnings.push(format!(
            "Papercut reference warnings unavailable: {}",
            error.message
        )),
    }
    warnings.sort();
    warnings.dedup();
    InboxLoad { items, warnings }
}

pub(crate) fn add(
    project: &TandemProject,
    options: AddOptions,
) -> Result<MutationOutcome, CliError> {
    let title = require_nonempty(
        options.title.as_deref(),
        "papercut add requires --title <text>",
    )?
    .to_string();
    validate_values("papercut add --reference", &options.references)?;
    validate_values("papercut add --tag", &options.tags)?;
    let warnings = reference_warnings(project, &options.references)?;
    let now = current_timestamp();
    let _lock = HierarchyLock::acquire(project)?;
    let existing = project.read_papercuts()?;
    let last = existing
        .iter()
        .filter_map(|item| papercut_number(item.id()))
        .max()
        .unwrap_or(0);
    let created =
        create_new_sequential_file_after(&project.papercuts_dir(), "papercut", last, |id| {
            let mut lines = vec![
                "---".to_string(),
                format!("id: {id}"),
                format!("title: {}", yaml_double_quote(&title)),
                "status: open".to_string(),
                format!("createdAt: {}", yaml_double_quote(&now)),
                format!("updatedAt: {}", yaml_double_quote(&now)),
            ];
            push_array(&mut lines, "references", &options.references);
            push_array(&mut lines, "tags", &options.tags);
            lines.push("---".to_string());
            if let Some(body) = options.body.as_deref() {
                lines.push(body.to_string());
            }
            lines.push(String::new());
            lines.join("\n")
        })?;
    let papercut = crate::project::read_papercut(&created.path)?;
    append_event(
        project,
        "papercut.created",
        &created.id,
        &format!("Created Papercut: {title}"),
    )?;
    Ok(MutationOutcome { papercut, warnings })
}

pub(crate) fn list(
    project: &TandemProject,
    options: ListOptions<'_>,
) -> Result<(Vec<StoredPapercut>, Vec<String>), CliError> {
    if options.all && options.status.is_some() {
        return Err(CliError::usage(
            "papercut list cannot combine --all and --status",
        ));
    }
    if let Some(status) = options.status {
        validate_status(status)?;
    }
    let wanted = if options.all {
        None
    } else {
        Some(options.status.unwrap_or("open"))
    };
    let mut items = project
        .read_papercuts()?
        .into_iter()
        .filter(|item| wanted.is_none_or(|status| item.status() == status))
        .collect::<Vec<_>>();
    items.sort_by_key(|item| papercut_number(item.id()).unwrap_or(usize::MAX));
    let warnings = warnings_for_items(project, &items)?;
    Ok((items, warnings))
}

pub(crate) fn show(
    project: &TandemProject,
    id: &str,
) -> Result<(StoredPapercut, Vec<String>), CliError> {
    validate_id(id)?;
    let item = project
        .find_papercut(id)?
        .ok_or_else(|| CliError::user(format!("Papercut not found: {id}")))?;
    let warnings = warnings_for_items(project, std::slice::from_ref(&item))?;
    Ok((item, warnings))
}

pub(crate) fn resolve(
    project: &TandemProject,
    options: ResolveOptions,
) -> Result<MutationOutcome, CliError> {
    validate_id(&options.id)?;
    let note = require_nonempty(
        options.note.as_deref(),
        "papercut resolve requires --note <text>",
    )?
    .to_string();
    validate_values("papercut resolve --reference", &options.references)?;
    let warnings = reference_warnings(project, &options.references)?;
    let _lock = HierarchyLock::acquire(project)?;
    let item = project
        .find_papercut(&options.id)?
        .ok_or_else(|| CliError::user(format!("Papercut not found: {}", options.id)))?;
    if item.status() == "resolved" {
        return Err(CliError::user(format!(
            "Papercut {} is already resolved",
            item.id()
        )));
    }
    let (content, signature) = read_file_snapshot(&item.path)?;
    let now = current_timestamp();
    let mut references = item.values("references");
    for reference in &options.references {
        if !references.contains(reference) {
            references.push(reference.clone());
        }
    }
    let mut updates = BTreeMap::from([
        ("status".to_string(), "resolved".to_string()),
        ("updatedAt".to_string(), now.clone()),
    ]);
    if !references.is_empty() {
        updates.insert("references".to_string(), inline_array(&references));
    }
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    let patched = patch_papercut_resolution_content(&patched, &note, &now)?;
    ensure_file_unchanged(&item.path, &signature)?;
    write_atomic(&item.path, &patched)?;
    let papercut = crate::project::read_papercut(&item.path)?;
    append_event(
        project,
        "papercut.resolved",
        papercut.id(),
        &format!("Resolved Papercut: {note}"),
    )?;
    Ok(MutationOutcome { papercut, warnings })
}

pub(crate) fn warnings_for_items(
    project: &TandemProject,
    items: &[StoredPapercut],
) -> Result<Vec<String>, CliError> {
    let mut warnings = Vec::new();
    for item in items {
        for reference in item.values("references") {
            if !project.reference_target_exists(&reference)? {
                warnings.push(format!(
                    "{} references missing target {reference}.",
                    item.id()
                ));
            }
        }
    }
    warnings.sort();
    warnings.dedup();
    Ok(warnings)
}

fn reference_warnings(
    project: &TandemProject,
    references: &[String],
) -> Result<Vec<String>, CliError> {
    let mut warnings = Vec::new();
    for reference in references {
        if !project.reference_target_exists(reference)? {
            warnings.push(format!("reference not found: {reference}"));
        }
    }
    Ok(warnings)
}

fn validate_status(status: &str) -> Result<(), CliError> {
    if STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: invalid Papercut status `{status}`; expected one of: {}",
            STATUSES.join(", ")
        )))
    }
}

fn validate_id(id: &str) -> Result<(), CliError> {
    if papercut_number(id).is_some() {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Validation failed: invalid Papercut ID `{id}`; expected `papercut-N`"
        )))
    }
}

fn validate_values(label: &str, values: &[String]) -> Result<(), CliError> {
    if values.iter().any(|value| value.trim().is_empty()) {
        Err(CliError::usage(format!("{label} must not be empty")))
    } else {
        Ok(())
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

fn push_array(lines: &mut Vec<String>, key: &str, values: &[String]) {
    let unique = values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if !unique.is_empty() {
        lines.push(format!("{key}: {}", inline_array(&unique)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn project() -> (TandemProject, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "tandem-papercuts-{}-{}",
            std::process::id(),
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
        (project, root)
    }

    #[test]
    fn lazy_add_filter_show_resolve_allocation_and_events() {
        let (project, root) = project();
        assert!(!project.papercuts_dir().exists());
        let first = add(
            &project,
            AddOptions {
                title: Some("First friction".into()),
                body: Some("Evidence here".into()),
                references: vec!["missing-task".into()],
                tags: vec!["tooling".into()],
            },
        )
        .unwrap();
        assert_eq!(first.papercut.id(), "papercut-1");
        assert_eq!(first.warnings, ["reference not found: missing-task"]);
        assert!(project.papercuts_dir().is_dir());
        let second = add(
            &project,
            AddOptions {
                title: Some("First friction".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(second.papercut.id(), "papercut-2");
        let first_path = project.papercuts_dir().join("papercut-1.md");
        let source = fs::read_to_string(&first_path).unwrap();
        fs::write(
            &first_path,
            source.replace("status: open\n", "status: open\ncustom: keep\n"),
        )
        .unwrap();
        let resolved = resolve(
            &project,
            ResolveOptions {
                id: "papercut-1".into(),
                note: Some("Added helper".into()),
                references: vec!["papercut-2".into()],
            },
        )
        .unwrap();
        assert_eq!(resolved.papercut.status(), "resolved");
        assert_eq!(
            resolved.papercut.field("resolution.note"),
            Some("Added helper")
        );
        assert_eq!(resolved.papercut.body, "Evidence here\n");
        assert_eq!(resolved.papercut.field("custom"), Some("keep"));
        let search = crate::app::queries::search_papercuts(
            project.read_papercuts().unwrap(),
            "Added helper",
        );
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].papercut.id(), "papercut-1");
        assert_eq!(list(&project, ListOptions::default()).unwrap().0.len(), 1);
        assert_eq!(
            list(
                &project,
                ListOptions {
                    all: true,
                    status: None
                }
            )
            .unwrap()
            .0
            .len(),
            2
        );
        let shown = show(&project, "papercut-1").unwrap().0;
        assert_eq!(shown.values("references"), ["missing-task", "papercut-2"]);
        let events = project.read_events_tolerant(&mut Vec::new());
        assert!(events.iter().any(|event| event.event == "papercut.created"));
        assert!(events
            .iter()
            .any(|event| event.event == "papercut.resolved"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_papercut_does_not_affect_board_reads() {
        let (project, root) = project();
        fs::create_dir_all(project.papercuts_dir()).unwrap();
        fs::write(
            project.papercuts_dir().join("papercut-1.md"),
            "---\nid: papercut-1\nstatus: bad\n---\n",
        )
        .unwrap();
        assert!(project.read_board_documents().is_ok());
        assert!(project
            .read_papercuts()
            .unwrap_err()
            .message
            .contains("Papercut validation failed"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn open_inbox_load_is_empty_when_missing_and_isolates_malformed_records() {
        let (project, root) = project();
        let empty = load_open_inbox(&project);
        assert!(empty.items.is_empty());
        assert!(empty.warnings.is_empty());

        let open = add(
            &project,
            AddOptions {
                title: Some("Open friction".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let resolved = add(
            &project,
            AddOptions {
                title: Some("Fixed friction".into()),
                ..Default::default()
            },
        )
        .unwrap();
        resolve(
            &project,
            ResolveOptions {
                id: resolved.papercut.id().to_string(),
                note: Some("fixed".into()),
                ..Default::default()
            },
        )
        .unwrap();
        fs::write(
            project.papercuts_dir().join("papercut-3.md"),
            "not frontmatter",
        )
        .unwrap();

        let inbox = load_open_inbox(&project);
        assert_eq!(inbox.items.len(), 1);
        assert_eq!(inbox.items[0].id(), open.papercut.id());
        assert_eq!(inbox.warnings.len(), 1);
        assert!(inbox.warnings[0].contains("papercut-3.md"));
        fs::remove_dir_all(root).unwrap();
    }
}
