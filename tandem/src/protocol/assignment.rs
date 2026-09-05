//! Native assignment-definition semantics.
//!
//! An assignment is a normal Task and its direct Subtasks are milestones. This
//! module owns the stable, opaque definition token used by consumers to reject
//! stale scope without treating workflow progress as a definition change.

use super::document::Document;
use super::hierarchy::DocumentLocation;
use super::ids::compare_ids;
use super::workflow::resolution_outcome;

const DEFINITION_FIELDS: &[&str] = &[
    "id",
    "type",
    "kind",
    "title",
    "parentId",
    "relatedFiles",
    "blockers",
    "accord.acceptance",
    "accord.constraints",
    "accord.validation",
];

/// The native meaning of one blocker while deriving assignment dependency
/// status. This is deliberately not persisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DependencyResolution {
    pub(crate) status: &'static str,
    pub(crate) clear: bool,
    pub(crate) reason: &'static str,
}

/// Resolve one blocker using native Board/Logs semantics. Every archived
/// outcome unblocks a dependency, matching task completion's existing
/// `unresolved_blockers_in_hierarchy` policy.
pub(crate) fn dependency_resolution(
    document: Option<(&Document, DocumentLocation)>,
) -> DependencyResolution {
    let Some((document, location)) = document else {
        return DependencyResolution {
            status: "missing",
            clear: false,
            reason: "missing blocker",
        };
    };
    if location == DocumentLocation::Board {
        return DependencyResolution {
            status: "active",
            clear: false,
            reason: "active blocker",
        };
    }
    match resolution_outcome(document) {
        "completed" => DependencyResolution {
            status: "completed",
            clear: true,
            reason: "archived blocker: completed",
        },
        "canceled" => DependencyResolution {
            status: "canceled",
            clear: true,
            reason: "archived blocker: canceled",
        },
        "failed" => DependencyResolution {
            status: "failed",
            clear: true,
            reason: "archived blocker: failed",
        },
        _ => DependencyResolution {
            status: "archived",
            clear: true,
            reason: "archived blocker",
        },
    }
}

/// Return an opaque token for the current assignment definition.
///
/// The caller supplies the root and its direct milestones from one coherent
/// hierarchy snapshot. Members are canonically sorted by native ID so progress
/// changes cannot reorder the definition before hashing. Bodies and the
/// explicit definition fields are included. State, Accord status/evidence,
/// assignee, timestamps, resolution, references, tags, and unrelated
/// documents are intentionally excluded.
pub(crate) fn definition_token(documents: &[&Document]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn add(hash: &mut u64, value: &[u8]) {
        for byte in value {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(PRIME);
        }
        *hash ^= 0xff;
        *hash = hash.wrapping_mul(PRIME);
    }

    let mut hash = OFFSET;
    add(&mut hash, b"tandem-assignment-definition-v1");
    let mut documents = documents.to_vec();
    documents.sort_by(|left, right| compare_ids(left.id(), right.id()));
    for document in documents {
        add(&mut hash, document.id().as_bytes());
        for field in DEFINITION_FIELDS {
            add(&mut hash, field.as_bytes());
            add(
                &mut hash,
                document.field(field).unwrap_or("<absent>").as_bytes(),
            );
        }
        add(&mut hash, document.body.as_bytes());
    }
    format!("ad1-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn document(fields: &[(&str, &str)], body: &str) -> Document {
        Document::new(
            fields
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect::<HashMap<_, _>>(),
            body.to_string(),
        )
    }

    #[test]
    fn token_changes_for_every_definition_field_and_body() {
        let base = document(
            &[
                ("id", "task-1"),
                ("type", "task"),
                ("title", "Outcome"),
                ("relatedFiles", "[src/main.rs]"),
                ("blockers", "[task-9]"),
                ("accord.acceptance", "[accept]"),
                ("accord.constraints", "[constraint]"),
                ("accord.validation", "[cargo test]"),
            ],
            "description",
        );
        let fields = [
            ("type", "decision"),
            ("kind", "epic"),
            ("title", "Changed"),
            ("parentId", "task-2"),
            ("relatedFiles", "[src/other.rs]"),
            ("blockers", "[task-8]"),
            ("accord.acceptance", "[other]"),
            ("accord.constraints", "[other]"),
            ("accord.validation", "[other]"),
        ];
        for (key, value) in fields {
            let mut changed = base.fields.clone();
            changed.insert(key.to_string(), value.to_string());
            let changed = Document::new(changed, base.body.clone());
            assert_ne!(
                definition_token(&[&base]),
                definition_token(&[&changed]),
                "definition field {key} must affect the token"
            );
        }
        let changed = document(
            &[("id", "task-1"), ("title", "Outcome")],
            "changed description",
        );
        assert_ne!(definition_token(&[&base]), definition_token(&[&changed]));
    }

    #[test]
    fn token_member_order_is_not_definition_scope() {
        let root = document(&[("id", "task-1"), ("title", "Root")], "root");
        let first = document(
            &[
                ("id", "task-1-1"),
                ("parentId", "task-1"),
                ("title", "First"),
            ],
            "first",
        );
        let second = document(
            &[
                ("id", "task-1-2"),
                ("parentId", "task-1"),
                ("title", "Second"),
            ],
            "second",
        );
        assert_eq!(
            definition_token(&[&root, &first, &second]),
            definition_token(&[&root, &second, &first])
        );
    }

    #[test]
    fn dependency_resolution_matches_native_archived_blocker_policy() {
        let active = document(&[("id", "task-1")], "");
        assert_eq!(
            dependency_resolution(Some((&active, DocumentLocation::Board))),
            DependencyResolution {
                status: "active",
                clear: false,
                reason: "active blocker"
            }
        );
        assert_eq!(
            dependency_resolution(None),
            DependencyResolution {
                status: "missing",
                clear: false,
                reason: "missing blocker"
            }
        );
        for outcome in ["completed", "canceled", "failed"] {
            let archived = document(&[("id", "task-1"), ("resolution.outcome", outcome)], "");
            let resolution = dependency_resolution(Some((&archived, DocumentLocation::Logs)));
            assert!(resolution.clear, "archived {outcome} must unblock");
            assert_eq!(resolution.status, outcome);
            assert_eq!(resolution.reason, format!("archived blocker: {outcome}"));
        }
    }

    #[test]
    fn token_ignores_progress_fields_and_unrelated_documents() {
        let base = document(
            &[
                ("id", "task-1"),
                ("title", "Outcome"),
                ("state", "todo"),
                ("accord.status", "ready"),
                ("assignee", "worker-a"),
                ("updatedAt", "one"),
                ("accord.evidence", "[evidence]"),
                ("tags", "[tag]"),
                ("references", "[task-9]"),
            ],
            "description",
        );
        let mut progressed = base.fields.clone();
        progressed.insert("state".to_string(), "in-progress".to_string());
        progressed.insert("accord.status".to_string(), "delivered".to_string());
        progressed.insert("updatedAt".to_string(), "two".to_string());
        progressed.insert("accord.evidence".to_string(), "[new evidence]".to_string());
        progressed.insert("tags".to_string(), "[other-tag]".to_string());
        let progressed = Document::new(progressed, base.body.clone());
        let unrelated = document(&[("id", "task-2"), ("title", "Other")], "other");
        assert_eq!(definition_token(&[&base]), definition_token(&[&progressed]));
        assert_ne!(
            definition_token(&[&base]),
            definition_token(&[&base, &unrelated])
        );
    }
}
