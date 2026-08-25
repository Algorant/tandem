//! Canonical review metadata vocabulary and completion-policy semantics.

use super::document::Document;

pub(crate) const STATUSES: &[&str] = &[
    "not-ready",
    "pending",
    "accepted",
    "changes-requested",
    "rejected",
];

pub(crate) fn status(document: &Document) -> Option<&str> {
    document
        .field("review.status")
        .or_else(|| document.field("reviewStatus"))
}

pub(crate) fn is_known_status(status: &str) -> bool {
    STATUSES.contains(&status)
}
