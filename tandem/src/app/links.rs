//! Typed link operations: add, remove, read projections, and list filtering.
//!
//! Link meaning (vocabulary, inverses, validation) is owned by
//! [`crate::protocol::links`]; this module coordinates it with project I/O.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::app::support::{append_event_with_data, current_timestamp, require_nonempty};
use crate::app::Error;
use crate::project::frontmatter::patch_links_content;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot, HierarchyLock};
use crate::project::{
    patch_frontmatter_content, write_atomic, ProjectHierarchy, StoredDocument as Document,
    TandemProject,
};
use crate::protocol::event::{TASK_LINKED, TASK_UNLINKED};
use crate::protocol::hierarchy::DocumentLocation;
use crate::protocol::links::{self, Link};

#[derive(Debug)]
pub(crate) struct LinkOutcome {
    pub(crate) id: String,
    pub(crate) link_type: String,
    pub(crate) target: String,
    /// False when the request was already satisfied and nothing was written.
    pub(crate) changed: bool,
}

/// Adds `<id> <link_type> <target>` to an active Task.
pub(crate) fn add(
    project: &TandemProject,
    id: &str,
    link_type: &str,
    target: &str,
) -> Result<LinkOutcome, Error> {
    let (link_type, target) = require_inputs(link_type, target)?;
    let _lock = HierarchyLock::acquire(project)?;
    let hierarchy = crate::app::support::hierarchy_from_project(project)?;
    let source = active_task(&hierarchy, id)?;
    let mut stored = stored_links(source)?;
    validate_target(&hierarchy, source, link_type, target)?;
    let changed = links::insert(&mut stored, link_type, target);
    if changed {
        write_links(
            project,
            source,
            &stored,
            TASK_LINKED,
            link_type,
            target,
            "Linked",
        )?;
    }
    Ok(LinkOutcome {
        id: source.id().to_string(),
        link_type: link_type.to_string(),
        target: target.to_string(),
        changed,
    })
}

/// Removes a stored link from an active Task. A link that is not stored is an
/// error; the target need not still resolve.
pub(crate) fn remove(
    project: &TandemProject,
    id: &str,
    link_type: &str,
    target: &str,
) -> Result<LinkOutcome, Error> {
    let (link_type, target) = require_inputs(link_type, target)?;
    let _lock = HierarchyLock::acquire(project)?;
    let hierarchy = crate::app::support::hierarchy_from_project(project)?;
    let source = active_task(&hierarchy, id)?;
    let mut stored = stored_links(source)?;
    if !links::remove(&mut stored, link_type, target) {
        return Err(Error::user(format!(
            "Validation failed: {} has no {link_type} link to {target}",
            source.id()
        )));
    }
    write_links(
        project,
        source,
        &stored,
        TASK_UNLINKED,
        link_type,
        target,
        "Unlinked",
    )?;
    Ok(LinkOutcome {
        id: source.id().to_string(),
        link_type: link_type.to_string(),
        target: target.to_string(),
        changed: true,
    })
}

/// The source's stored links plus `fixed-by <target>`, validated exactly like
/// `add`. `complete --fixed-by` writes the result with the archive.
pub(crate) fn with_fixed_by(
    hierarchy: &ProjectHierarchy,
    source: &Document,
    target: &str,
) -> Result<Vec<Link>, Error> {
    let (_, target) = require_inputs(links::FIXED_BY, target)?;
    let mut stored = stored_links(source)?;
    validate_target(hierarchy, source, links::FIXED_BY, target)?;
    links::insert(&mut stored, links::FIXED_BY, target);
    Ok(stored)
}

fn require_inputs<'a>(link_type: &'a str, target: &'a str) -> Result<(&'a str, &'a str), Error> {
    let link_type = require_nonempty(Some(link_type), "link type must not be empty")?;
    let target = require_nonempty(Some(target), "link target must not be empty")?;
    links::validate_link_type(link_type)
        .map_err(|message| Error::user(format!("Validation failed: {message}")))?;
    Ok((link_type, target))
}

fn active_task<'a>(hierarchy: &'a ProjectHierarchy, id: &str) -> Result<&'a Document, Error> {
    let document = hierarchy
        .document(id)
        .ok_or_else(|| Error::user(format!("document not found: {id}")))?;
    if document.doc_type() != "task" || document.location != DocumentLocation::Board {
        return Err(Error::user(format!(
            "Validation failed: only active tasks own links: {id} is {}",
            if document.location == DocumentLocation::Logs {
                "archived".to_string()
            } else {
                format!("type {}", document.doc_type())
            }
        )));
    }
    Ok(document)
}

/// The stored links of a source. Rewriting the block would drop a link type
/// outside the vocabulary, so such a record is refused instead.
fn stored_links(source: &Document) -> Result<Vec<Link>, Error> {
    if let Some(unknown) = links::unknown_link_types(source).first() {
        return Err(Error::user(format!(
            "Validation failed: {} has unknown link type {unknown}; remove it from the record by hand before editing links",
            source.id()
        )));
    }
    Ok(links::outgoing(source))
}

fn validate_target(
    hierarchy: &ProjectHierarchy,
    source: &Document,
    link_type: &str,
    target: &str,
) -> Result<(), Error> {
    let target_document = hierarchy.document(target).ok_or_else(|| {
        Error::user(format!(
            "Validation failed: link target not found: {target}"
        ))
    })?;
    links::validate_link(
        link_type,
        source,
        target_document,
        target_document.location == DocumentLocation::Logs,
    )
    .map_err(|message| Error::user(format!("Validation failed: {message}")))
}

fn write_links(
    project: &TandemProject,
    source: &Document,
    stored: &[Link],
    event: &str,
    link_type: &str,
    target: &str,
    verb: &str,
) -> Result<(), Error> {
    let (content, signature) = read_file_snapshot(project, &source.path)?;
    let patched = patch_links_content(&content, stored)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), current_timestamp());
    let patched = patch_frontmatter_content(&patched, &updates, &[])?;
    ensure_file_unchanged(&source.path, &signature)?;
    write_atomic(&source.path, &patched)?;
    append_event_with_data(
        project,
        event,
        source.id(),
        &format!("{verb} {} {link_type} {target}", source.id()),
        Some(&serde_json::json!({"type": link_type, "target": target})),
    )
}

/// One stored outgoing link with the target's identity.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LinkDto {
    #[serde(rename = "type")]
    pub(crate) link_type: String,
    pub(crate) target: String,
    pub(crate) title: Option<String>,
    pub(crate) location: Option<&'static str>,
}

/// One derived incoming link, labelled with the inverse type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IncomingLinkDto {
    #[serde(rename = "type")]
    pub(crate) link_type: String,
    pub(crate) source: String,
    pub(crate) title: Option<String>,
    pub(crate) location: Option<&'static str>,
}

fn index(documents: &HashMap<String, Document>) -> HashMap<String, Vec<links::LinkView>> {
    links::index(documents.values().map(|document| &**document))
}

/// Outgoing and incoming link projections for one record.
pub(crate) fn dtos(
    documents: &HashMap<String, Document>,
    id: &str,
) -> (Vec<LinkDto>, Vec<IncomingLinkDto>) {
    let identity = |other: &str| {
        let document = documents.get(other);
        (
            document.map(|document| document.title().to_string()),
            document.map(|document| document.location.as_str()),
        )
    };
    let mut outgoing = Vec::new();
    let mut incoming = Vec::new();
    for view in index(documents).remove(id).unwrap_or_default() {
        let (title, location) = identity(&view.other);
        if view.incoming {
            incoming.push(IncomingLinkDto {
                link_type: view.label,
                source: view.other,
                title,
                location,
            });
        } else {
            outgoing.push(LinkDto {
                link_type: view.label,
                target: view.other,
                title,
                location,
            });
        }
    }
    (outgoing, incoming)
}

/// Human `show` lines for a record's links; empty when it has none.
pub(crate) fn text_lines(outgoing: &[LinkDto], incoming: &[IncomingLinkDto]) -> Vec<String> {
    let line = |label: &str, other: &str, title: &Option<String>, location: Option<&str>| {
        let mut line = format!("  {label} {other}");
        if let Some(title) = title {
            line.push_str(&format!(" - {title}"));
        }
        match location {
            Some("logs") => line.push_str(" [archived]"),
            None => line.push_str(" [missing]"),
            Some(_) => {}
        }
        line
    };
    let mut lines = Vec::new();
    if !outgoing.is_empty() || !incoming.is_empty() {
        lines.push("Links:".to_string());
    }
    for link in outgoing {
        lines.push(line(
            &link.link_type,
            &link.target,
            &link.title,
            link.location,
        ));
    }
    for link in incoming {
        lines.push(line(
            &link.link_type,
            &link.source,
            &link.title,
            link.location,
        ));
    }
    lines
}

/// Keeps documents with a link (outgoing or derived incoming) whose label is
/// `label` and whose other end is `linked_to`; an absent criterion matches any.
/// `all` is the whole workspace so links stored on out-of-scope records count.
pub(crate) fn filter_documents(
    documents: Vec<Document>,
    all: &[Document],
    label: Option<&str>,
    linked_to: Option<&str>,
) -> Result<Vec<Document>, Error> {
    if let Some(label) = label {
        links::validate_link_label(label)
            .map_err(|message| Error::user(format!("Validation failed: {message}")))?;
    }
    if label.is_none() && linked_to.is_none() {
        return Ok(documents);
    }
    let views = links::index(all.iter().map(|document| &**document));
    Ok(documents
        .into_iter()
        .filter(|document| {
            views.get(document.id()).is_some_and(|views| {
                views.iter().any(|view| {
                    label.is_none_or(|label| view.label == label)
                        && linked_to.is_none_or(|other| view.other == other)
                })
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::app::tasks::{self, AddOptions, CompleteOptions};

    fn project() -> TandemProject {
        let root = std::env::temp_dir().join(format!(
            "tandem-app-links-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.5.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap()
    }

    fn task(project: &TandemProject, title: &str) -> String {
        tasks::add(
            project,
            AddOptions {
                title: Some(title.to_string()),
                acceptance: vec!["done".to_string()],
                ..Default::default()
            },
        )
        .unwrap()
        .id
    }

    fn read(project: &TandemProject, id: &str) -> Document {
        project.find_document(id).unwrap().unwrap()
    }

    fn message(result: Result<LinkOutcome, Error>) -> String {
        result.unwrap_err().message
    }

    /// Archives `id` as completed (delivering first) or canceled.
    fn archive(project: &TandemProject, id: &str, completed: bool) {
        if completed {
            tasks::complete(
                project,
                CompleteOptions {
                    id: id.to_string(),
                    ..Default::default()
                },
            )
            .unwrap();
        } else {
            tasks::cancel(project, id, "not needed").unwrap();
        }
    }

    #[test]
    fn add_and_remove_round_trip_through_the_record_and_events() {
        let project = project();
        let (one, two) = (task(&project, "One"), task(&project, "Two"));

        let added = add(&project, &one, "relates-to", &two).unwrap();
        assert!(added.changed);
        let source = read(&project, &one);
        assert_eq!(
            source.field("links.relates-to"),
            Some(format!("[\"{two}\"]").as_str())
        );
        let content = fs::read_to_string(&source.path).unwrap();
        assert!(content.contains(&format!("links:\n  relates-to: [\"{two}\"]\n")));
        assert!(source.field("updatedAt").is_some());

        // The same add is an unchanged no-op.
        assert!(!add(&project, &one, "relates-to", &two).unwrap().changed);
        add(&project, &one, "supersedes", &two).unwrap();
        assert_eq!(links::outgoing(&read(&project, &one)).len(), 2);

        let removed = remove(&project, &one, "relates-to", &two).unwrap();
        assert!(removed.changed);
        let source = read(&project, &one);
        assert!(source.field("links.relates-to").is_none());
        assert!(source.field("links.supersedes").is_some());
        remove(&project, &one, "supersedes", &two).unwrap();
        let content = fs::read_to_string(&read(&project, &one).path).unwrap();
        assert!(!content.contains("links:"));

        let mut warnings = Vec::new();
        let events = project.read_events_tolerant(&mut warnings);
        let names = events
            .iter()
            .filter(|event| event.id == one)
            .map(|event| event.event.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"task.linked") && names.contains(&"task.unlinked"));
    }

    #[test]
    fn validation_rejects_bad_types_targets_self_links_and_missing_links() {
        let project = project();
        let (one, two) = (task(&project, "One"), task(&project, "Two"));

        assert_eq!(
            message(add(&project, &one, "blocks", &two)),
            "Validation failed: invalid link type `blocks`; expected one of: relates-to, duplicates, fixed-by, fixes, supersedes"
        );
        assert_eq!(
            message(add(&project, &one, "relates-to", &one)),
            format!("Validation failed: {one} cannot link to itself")
        );
        assert_eq!(
            message(add(&project, &one, "relates-to", "task-999")),
            "Validation failed: link target not found: task-999"
        );
        assert_eq!(
            message(add(&project, &one, "relates-to", " ")),
            "link target must not be empty"
        );
        assert_eq!(
            message(add(&project, "task-999", "relates-to", &two)),
            "document not found: task-999"
        );
        assert_eq!(
            message(remove(&project, &one, "relates-to", &two)),
            format!("Validation failed: {one} has no relates-to link to {two}")
        );
        // Nothing was written by any rejected call.
        assert!(read(&project, &one).field("links.relates-to").is_none());
        assert!(!fs::read_to_string(&read(&project, &one).path)
            .unwrap()
            .contains("links:"));
    }

    #[test]
    fn archived_targets_are_linkable_but_only_completed_ones_fix() {
        let project = project();
        let source = task(&project, "Source");
        let done = task(&project, "Done");
        let canceled = task(&project, "Canceled");
        archive(&project, &done, true);
        archive(&project, &canceled, false);

        add(&project, &source, "relates-to", &canceled).unwrap();
        add(&project, &source, "duplicates", &canceled).unwrap();
        add(&project, &source, "fixed-by", &done).unwrap();
        assert_eq!(
            message(add(&project, &source, "fixed-by", &canceled)),
            format!(
                "Validation failed: {canceled} is archived with outcome canceled; only a completed record can fix {source}"
            )
        );
    }

    #[test]
    fn only_active_tasks_own_links() {
        let project = project();
        let (active, archived) = (task(&project, "Active"), task(&project, "Archived"));
        archive(&project, &archived, true);
        assert_eq!(
            message(add(&project, &archived, "relates-to", &active)),
            format!("Validation failed: only active tasks own links: {archived} is archived")
        );

        crate::app::decisions::add(
            &project,
            crate::app::decisions::AddOptions {
                title: Some("A decision".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        let decision = project
            .read_documents()
            .unwrap()
            .into_iter()
            .find(|document| document.doc_type() == "decision")
            .unwrap()
            .id()
            .to_string();
        assert_eq!(
            message(add(&project, &decision, "relates-to", &active)),
            format!("Validation failed: only active tasks own links: {decision} is type decision")
        );
        // A decision is a valid target.
        add(&project, &active, "relates-to", &decision).unwrap();
    }

    #[test]
    fn unknown_stored_types_are_refused_not_dropped() {
        let project = project();
        let (one, two) = (task(&project, "One"), task(&project, "Two"));
        let path = read(&project, &one).path;
        let content = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            content.replacen("---\n", "---\nlinks:\n  blocks: [\"x\"]\n", 1),
        )
        .unwrap();
        assert!(message(add(&project, &one, "relates-to", &two))
            .contains("has unknown link type blocks"));
        assert!(fs::read_to_string(&path).unwrap().contains("blocks"));
    }

    #[test]
    fn complete_fixed_by_links_archives_completed_and_skips_the_delivery_warning() {
        let project = project();
        let papercut = task(&project, "Papercut");
        let fixer = task(&project, "Fixer");

        let outcome = tasks::complete(
            &project,
            CompleteOptions {
                id: papercut.clone(),
                fixed_by: Some(fixer.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        // The Accord was never delivered, but the linked record delivered it.
        assert!(outcome.warnings.is_empty() && !outcome.has_completion_warnings);
        let archived = read(&project, &papercut);
        assert_eq!(archived.location, DocumentLocation::Logs);
        assert_eq!(archived.field("resolution.outcome"), Some("completed"));
        assert_eq!(
            archived.field("resolution.note"),
            Some(format!("Fixed by {fixer}").as_str())
        );
        assert_eq!(
            archived.field("links.fixed-by"),
            Some(format!("[\"{fixer}\"]").as_str())
        );
        assert_eq!(archived.field("accord.status"), Some("ready"));

        // The archived papercut is visible from the fixer as `fixes`.
        let documents = project
            .read_documents()
            .unwrap()
            .into_iter()
            .map(|document| (document.id().to_string(), document))
            .collect::<HashMap<_, _>>();
        let (outgoing, incoming) = dtos(&documents, &fixer);
        assert!(outgoing.is_empty());
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].link_type, "fixes");
        assert_eq!(incoming[0].source, papercut);
        assert_eq!(incoming[0].location, Some("logs"));
    }

    #[test]
    fn complete_fixed_by_validates_before_writing_and_honors_an_explicit_note() {
        let project = project();
        let papercut = task(&project, "Papercut");
        let canceled = task(&project, "Canceled");
        archive(&project, &canceled, false);
        for (target, expected) in [
            (papercut.as_str(), format!("{papercut} cannot link to itself")),
            ("task-999", "link target not found: task-999".to_string()),
            (
                canceled.as_str(),
                format!("{canceled} is archived with outcome canceled; only a completed record can fix {papercut}"),
            ),
        ] {
            let error = tasks::complete(
                &project,
                CompleteOptions {
                    id: papercut.clone(),
                    fixed_by: Some(target.to_string()),
                    ..Default::default()
                },
            )
            .unwrap_err();
            assert_eq!(error.message, format!("Validation failed: {expected}"));
            assert_eq!(read(&project, &papercut).location, DocumentLocation::Board);
        }

        let fixer = task(&project, "Fixer");
        tasks::complete(
            &project,
            CompleteOptions {
                id: papercut.clone(),
                fixed_by: Some(fixer),
                note: Some("fixed in passing".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            read(&project, &papercut).field("resolution.note"),
            Some("fixed in passing")
        );
    }

    #[test]
    fn filter_matches_outgoing_and_derived_incoming_labels() {
        let project = project();
        let (one, two, three) = (
            task(&project, "One"),
            task(&project, "Two"),
            task(&project, "Three"),
        );
        add(&project, &one, "fixed-by", &two).unwrap();
        add(&project, &three, "relates-to", &one).unwrap();
        let all = project.read_documents().unwrap();
        let ids = |label: Option<&str>, to: Option<&str>| {
            let mut ids = filter_documents(all.clone(), &all, label, to)
                .unwrap()
                .iter()
                .map(|document| document.id().to_string())
                .collect::<Vec<_>>();
            ids.sort();
            ids
        };
        assert_eq!(ids(Some("fixed-by"), None), vec![one.clone()]);
        assert_eq!(ids(Some("fixes"), None), vec![two.clone()]);
        assert_eq!(
            ids(Some("relates-to"), None),
            vec![one.clone(), three.clone()]
        );
        assert_eq!(ids(None, Some(&one)), vec![two.clone(), three.clone()]);
        assert_eq!(ids(Some("fixes"), Some(&one)), vec![two.clone()]);
        assert_eq!(ids(Some("fixed-by"), Some(&three)), Vec::<String>::new());
        assert_eq!(ids(None, None).len(), 3);
        assert!(filter_documents(all.clone(), &all, Some("blocks"), None)
            .unwrap_err()
            .message
            .contains("invalid link type `blocks`"));
    }

    #[test]
    fn text_lines_label_archived_and_missing_ends() {
        let outgoing = vec![
            LinkDto {
                link_type: "fixed-by".into(),
                target: "task-2".into(),
                title: Some("Fixer".into()),
                location: Some("logs"),
            },
            LinkDto {
                link_type: "relates-to".into(),
                target: "task-9".into(),
                title: None,
                location: None,
            },
        ];
        let incoming = vec![IncomingLinkDto {
            link_type: "fixes".into(),
            source: "task-3".into(),
            title: Some("Other".into()),
            location: Some("board"),
        }];
        assert_eq!(
            text_lines(&outgoing, &incoming),
            vec![
                "Links:",
                "  fixed-by task-2 - Fixer [archived]",
                "  relates-to task-9 [missing]",
                "  fixes task-3 - Other"
            ]
        );
        assert!(text_lines(&[], &[]).is_empty());
    }
}
