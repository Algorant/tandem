//! Typed links between records.
//!
//! See the normative [typed links specification](../../../protocol/links.md).
//! A link is stored on its source Task as `links.<type>: [target-id, ...]`;
//! only the outgoing side is stored and the inverse is derived here.

use std::collections::HashMap;

use crate::protocol::document::Document;
use crate::protocol::workflow::{resolution_outcome, RESOLUTION_OUTCOME_COMPLETED};

pub(crate) const LINKS_FIELD: &str = "links";
pub(crate) const FIXED_BY: &str = "fixed-by";

/// Stored link types with the label their derived inverse is shown under.
const LINK_TYPES: &[(&str, &str)] = &[
    ("relates-to", "relates-to"),
    ("duplicates", "duplicated-by"),
    (FIXED_BY, "fixes"),
    ("fixes", FIXED_BY),
    ("supersedes", "superseded-by"),
];

/// Stored types in vocabulary order.
pub(crate) fn link_types() -> impl Iterator<Item = &'static str> {
    LINK_TYPES.iter().map(|(link_type, _)| *link_type)
}

/// The label under which the target sees a link of `link_type`.
pub(crate) fn inverse(link_type: &str) -> Option<&'static str> {
    LINK_TYPES
        .iter()
        .find(|(candidate, _)| *candidate == link_type)
        .map(|(_, inverse)| *inverse)
}

/// Validates a stored link type against the fixed vocabulary.
pub(crate) fn validate_link_type(link_type: &str) -> Result<(), String> {
    if inverse(link_type).is_some() {
        Ok(())
    } else {
        Err(format!(
            "invalid link type `{link_type}`; expected one of: {}",
            link_types().collect::<Vec<_>>().join(", ")
        ))
    }
}

/// Validates a link-view filter label: a stored type or a derived inverse.
pub(crate) fn validate_link_label(label: &str) -> Result<(), String> {
    if LINK_TYPES
        .iter()
        .any(|(link_type, inverse)| *link_type == label || *inverse == label)
    {
        return Ok(());
    }
    let mut labels = Vec::new();
    for (link_type, inverse) in LINK_TYPES {
        for label in [*link_type, *inverse] {
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    Err(format!(
        "invalid link type `{label}`; expected one of: {}",
        labels.join(", ")
    ))
}

/// One stored outgoing link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) link_type: String,
    pub(crate) target: String,
}

/// Outgoing links as stored, in vocabulary order. Unknown types are ignored
/// here and reported by [`unknown_link_types`].
pub(crate) fn outgoing(document: &Document) -> Vec<Link> {
    let mut links = Vec::new();
    for link_type in link_types() {
        for target in document.values(&format!("{LINKS_FIELD}.{link_type}")) {
            links.push(Link {
                link_type: link_type.to_string(),
                target,
            });
        }
    }
    links
}

/// Stored link types that are not in the vocabulary.
pub(crate) fn unknown_link_types(document: &Document) -> Vec<String> {
    let prefix = format!("{LINKS_FIELD}.");
    let mut unknown = document
        .fields
        .keys()
        .filter_map(|key| key.strip_prefix(&prefix))
        .map(|rest| rest.split('.').next().unwrap_or(rest).to_string())
        .filter(|link_type| inverse(link_type).is_none())
        .collect::<Vec<_>>();
    unknown.sort();
    unknown.dedup();
    unknown
}

/// One link as seen from a record: its own link or a derived incoming one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkView {
    /// The label from this record's side (the inverse for an incoming link).
    pub(crate) label: String,
    /// The record on the other end.
    pub(crate) other: String,
    pub(crate) incoming: bool,
}

/// Every record's links, keyed by record ID: its stored outgoing links first,
/// then the derived inverse of every link stored on another document that
/// targets it.
pub(crate) fn index<'a>(
    documents: impl IntoIterator<Item = &'a Document>,
) -> HashMap<String, Vec<LinkView>> {
    let mut views: HashMap<String, Vec<LinkView>> = HashMap::new();
    for document in documents {
        for link in outgoing(document) {
            views
                .entry(link.target.clone())
                .or_default()
                .push(LinkView {
                    label: inverse(&link.link_type)
                        .expect("outgoing links use known types")
                        .to_string(),
                    other: document.id().to_string(),
                    incoming: true,
                });
            views
                .entry(document.id().to_string())
                .or_default()
                .push(LinkView {
                    label: link.link_type,
                    other: link.target,
                    incoming: false,
                });
        }
    }
    for entries in views.values_mut() {
        entries.sort_by_key(|entry| entry.incoming);
    }
    views
}

/// Checks a prospective link against its resolved source and target.
pub(crate) fn validate_link(
    link_type: &str,
    source: &Document,
    target: &Document,
    target_archived: bool,
) -> Result<(), String> {
    validate_link_type(link_type)?;
    if source.id() == target.id() {
        return Err(format!("{} cannot link to itself", source.id()));
    }
    if !target.is_first_class_type() {
        return Err(format!(
            "link target {} is type {}; only task and decision documents can be linked",
            target.id(),
            target.doc_type()
        ));
    }
    if link_type == FIXED_BY && target_archived {
        let outcome = resolution_outcome(target);
        if outcome != RESOLUTION_OUTCOME_COMPLETED {
            return Err(format!(
                "{} is archived with outcome {outcome}; only a completed record can fix {}",
                target.id(),
                source.id()
            ));
        }
    }
    Ok(())
}

/// Warnings for an active Board record's stored links.
pub(crate) fn diagnostics(
    document: &Document,
    target_exists: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut warnings = unknown_link_types(document)
        .into_iter()
        .map(|link_type| format!("{} has unknown link type {link_type}.", document.id()))
        .collect::<Vec<_>>();
    for link in outgoing(document) {
        if !target_exists(&link.target) {
            warnings.push(format!(
                "{} links missing target {} ({}).",
                document.id(),
                link.target,
                link.link_type
            ));
        }
    }
    warnings
}

/// Applies an add to a type-keyed link set, returning whether it changed.
pub(crate) fn insert(links: &mut Vec<Link>, link_type: &str, target: &str) -> bool {
    if links
        .iter()
        .any(|link| link.link_type == link_type && link.target == target)
    {
        return false;
    }
    links.push(Link {
        link_type: link_type.to_string(),
        target: target.to_string(),
    });
    true
}

/// Applies a removal to a link set, returning whether anything was removed.
pub(crate) fn remove(links: &mut Vec<Link>, link_type: &str, target: &str) -> bool {
    let before = links.len();
    links.retain(|link| !(link.link_type == link_type && link.target == target));
    links.len() != before
}

/// Targets of one link type as the field value list, for rendering.
pub(crate) fn targets_of<'a>(links: &'a [Link], link_type: &str) -> Vec<&'a str> {
    links
        .iter()
        .filter(|link| link.link_type == link_type)
        .map(|link| link.target.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn doc(fields: &[(&str, &str)]) -> Document {
        Document::new(
            fields
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect::<HashMap<_, _>>(),
            String::new(),
        )
    }

    #[test]
    fn vocabulary_has_a_derived_inverse_for_every_type() {
        assert_eq!(inverse("relates-to"), Some("relates-to"));
        assert_eq!(inverse("duplicates"), Some("duplicated-by"));
        assert_eq!(inverse("fixed-by"), Some("fixes"));
        assert_eq!(inverse("fixes"), Some("fixed-by"));
        assert_eq!(inverse("supersedes"), Some("superseded-by"));
        assert_eq!(inverse("duplicated-by"), None);
        assert!(validate_link_type("blocks")
            .unwrap_err()
            .contains("expected one of: relates-to, duplicates, fixed-by, fixes, supersedes"));
        assert!(validate_link_label("superseded-by").is_ok());
        assert!(validate_link_label("blocks").is_err());
    }

    #[test]
    fn outgoing_reads_stored_types_and_reports_unknown_ones() {
        let document = doc(&[
            ("id", "task-1"),
            ("links.fixed-by", "[\"task-2\"]"),
            ("links.relates-to", "[\"task-3\", \"decision-1\"]"),
            ("links.blocks", "[\"task-4\"]"),
        ]);
        assert_eq!(
            outgoing(&document)
                .iter()
                .map(|link| format!("{}>{}", link.link_type, link.target))
                .collect::<Vec<_>>(),
            vec![
                "relates-to>task-3",
                "relates-to>decision-1",
                "fixed-by>task-2"
            ]
        );
        assert_eq!(unknown_link_types(&document), vec!["blocks".to_string()]);
        assert_eq!(
            diagnostics(&document, |id| id != "decision-1"),
            vec![
                "task-1 has unknown link type blocks.".to_string(),
                "task-1 links missing target decision-1 (relates-to).".to_string()
            ]
        );
    }

    #[test]
    fn index_combines_outgoing_with_derived_inverses() {
        let source = doc(&[("id", "task-1"), ("links.fixed-by", "[\"task-2\"]")]);
        let fixer = doc(&[("id", "task-2"), ("links.relates-to", "[\"task-3\"]")]);
        let other = doc(&[("id", "task-3")]);
        let views = index([&source, &fixer, &other]);
        assert_eq!(
            views["task-2"]
                .iter()
                .map(|v| (v.label.as_str(), v.other.as_str(), v.incoming))
                .collect::<Vec<_>>(),
            vec![("relates-to", "task-3", false), ("fixes", "task-1", true)]
        );
        assert_eq!(views["task-3"].len(), 1);
        assert_eq!(views["task-3"][0].label, "relates-to");
        assert_eq!(views["task-3"][0].other, "task-2");
        assert_eq!(views["task-1"][0].label, "fixed-by");
    }

    #[test]
    fn validation_rejects_self_links_unsupported_targets_and_unfinished_fixers() {
        let source = doc(&[("id", "task-1"), ("type", "task")]);
        let target = doc(&[("id", "task-2"), ("type", "task")]);
        assert!(validate_link("relates-to", &source, &target, false).is_ok());
        assert_eq!(
            validate_link("relates-to", &source, &source, false).unwrap_err(),
            "task-1 cannot link to itself"
        );
        let custom = doc(&[("id", "note-1"), ("type", "note")]);
        assert!(validate_link("relates-to", &source, &custom, false)
            .unwrap_err()
            .contains("only task and decision documents"));
        let canceled = doc(&[
            ("id", "task-3"),
            ("type", "task"),
            ("resolution.outcome", "canceled"),
        ]);
        assert_eq!(
            validate_link("fixed-by", &source, &canceled, true).unwrap_err(),
            "task-3 is archived with outcome canceled; only a completed record can fix task-1"
        );
        assert!(validate_link("duplicates", &source, &canceled, true).is_ok());
        let completed = doc(&[
            ("id", "task-4"),
            ("type", "task"),
            ("resolution.outcome", "completed"),
        ]);
        assert!(validate_link("fixed-by", &source, &completed, true).is_ok());
    }

    #[test]
    fn insert_and_remove_report_whether_the_set_changed() {
        let mut links = Vec::new();
        assert!(insert(&mut links, "relates-to", "task-2"));
        assert!(!insert(&mut links, "relates-to", "task-2"));
        assert!(insert(&mut links, "fixes", "task-2"));
        assert_eq!(targets_of(&links, "relates-to"), vec!["task-2"]);
        assert!(remove(&mut links, "relates-to", "task-2"));
        assert!(!remove(&mut links, "relates-to", "task-2"));
    }
}
