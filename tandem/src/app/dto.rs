//! UI-neutral read projections over resolved documents.
//!
//! These types are the single serialized shape of a Tandem document for every
//! machine consumer. The CLI JSON envelope and the web read API both build
//! them from the same `ReadSnapshot`; neither owns a second projection.

use serde::Serialize;

use crate::app::queries::ReadSnapshot;
use crate::app::Error;
use crate::project::StoredDocument as Document;
use crate::protocol::accord::{status as accord_status, AccordRecord};
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::workflow::{
    resolution_files_changed, resolution_note, resolution_outcome, resolution_reviewer,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentSummaryDto {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) kind: Option<String>,
    pub(crate) role: Option<&'static str>,
    pub(crate) title: String,
    pub(crate) location: &'static str,
    pub(crate) state: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) effort: Option<String>,
    pub(crate) assignee: Option<String>,
    pub(crate) parent_id: Option<String>,
    pub(crate) parent_relationship: Option<&'static str>,
    pub(crate) tags: Vec<String>,
    pub(crate) accord_status: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentDetailDto {
    #[serde(flatten)]
    pub(crate) summary: DocumentSummaryDto,
    pub(crate) body: String,
    pub(crate) due_date: Option<String>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
    pub(crate) completed_at: Option<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) references: Vec<String>,
    pub(crate) related_files: Vec<String>,
    pub(crate) parent: Option<Box<DocumentSummaryDto>>,
    pub(crate) children: Vec<DocumentSummaryDto>,
    pub(crate) accord: Option<AccordDto>,
    #[serde(flatten)]
    pub(crate) accord_counts: Option<crate::app::accord::AccordCounts>,
    pub(crate) validation: Option<ValidationDto>,
    pub(crate) resolution: Option<ResolutionDto>,
    pub(crate) decision: Option<DecisionDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccordDto {
    pub(crate) status: String,
    pub(crate) acceptance: Vec<String>,
    pub(crate) claimed_at: Option<String>,
    pub(crate) delivered_at: Option<String>,
    pub(crate) deliverables: Vec<String>,
    pub(crate) validations: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) summary: Option<String>,
    pub(crate) evidence: Vec<String>,
    pub(crate) files_changed: Vec<String>,
    pub(crate) reviewer: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) reason: Option<String>,
}

impl From<AccordRecord> for AccordDto {
    fn from(record: AccordRecord) -> Self {
        Self {
            status: record.status,
            acceptance: record.acceptance,
            claimed_at: record.claimed_at,
            delivered_at: record.delivered_at,
            deliverables: record.deliverables,
            validations: record.validations,
            constraints: record.constraints,
            summary: record.summary,
            evidence: record.evidence,
            files_changed: record.files_changed,
            reviewer: record.reviewer,
            note: record.note,
            reason: record.reason,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValidationDto {
    pub(crate) state: String,
    pub(crate) criterion: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) reviewer: Option<String>,
    pub(crate) requested_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResolutionDto {
    pub(crate) outcome: String,
    pub(crate) note: Option<String>,
    pub(crate) files_changed: Vec<String>,
    pub(crate) reviewer: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecisionDto {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) deciders: Vec<String>,
    pub(crate) context: Option<String>,
    pub(crate) consequences: Vec<String>,
    pub(crate) alternatives: Vec<String>,
    pub(crate) supersedes: Vec<String>,
    pub(crate) superseded_by: Vec<String>,
    pub(crate) summary: Option<String>,
}

/// Builds the summary projection for one resolved document.
pub(crate) fn summary(
    read: &ReadSnapshot,
    document: &Document,
) -> Result<DocumentSummaryDto, Error> {
    let role = read.snapshot.hierarchy.task_role(document)?;
    let relationship = read.snapshot.hierarchy.relationship(document)?;
    Ok(DocumentSummaryDto {
        id: document.id().to_string(),
        document_type: document.doc_type().to_string(),
        kind: document.kind().map(str::to_string),
        role: role.map(TaskRole::as_str),
        title: document.title().to_string(),
        location: document.location.as_str(),
        state: document.field("state").map(str::to_string),
        priority: document.field("priority").map(str::to_string),
        effort: document.field("effort").map(str::to_string),
        assignee: document.field("assignee").map(str::to_string),
        parent_id: document.field("parentId").map(str::to_string),
        parent_relationship: relationship.map(ParentRelationship::as_str),
        tags: values(document, "tags"),
        accord_status: accord_status(document).map(str::to_string),
    })
}

/// Builds the full record projection for one resolved document.
pub(crate) fn detail(read: &ReadSnapshot, document: &Document) -> Result<DocumentDetailDto, Error> {
    let summary = summary(read, document)?;
    let parent = document
        .field("parentId")
        .and_then(|id| read.snapshot.document(id))
        .map(|parent| self::summary(read, &parent))
        .transpose()?;
    let children = read
        .snapshot
        .children(document)?
        .iter()
        .map(|child| self::summary(read, child))
        .collect::<Result<Vec<_>, _>>()?;
    let resolution = (document.location == DocumentLocation::Logs && document.doc_type() == "task")
        .then(|| ResolutionDto {
            outcome: resolution_outcome(document).to_string(),
            note: resolution_note(document).map(str::to_string),
            files_changed: resolution_files_changed(document),
            reviewer: resolution_reviewer(document).map(str::to_string),
        });
    let accord = accord_status(document).map(|_| {
        AccordDto::from(AccordRecord::from_document(
            document,
            document.field("updatedAt").unwrap_or(""),
        ))
    });
    let validation = (document.field("state") == Some("validation")).then(|| ValidationDto {
        state: "validation".to_string(),
        criterion: document.field("validation.criterion").map(str::to_string),
        note: document.field("validation.note").map(str::to_string),
        reviewer: document.field("validation.reviewer").map(str::to_string),
        requested_at: document.field("validation.requestedAt").map(str::to_string),
    });
    Ok(DocumentDetailDto {
        summary,
        body: document.body.clone(),
        due_date: document.field("dueDate").map(str::to_string),
        created_at: document.field("createdAt").map(str::to_string),
        updated_at: document.field("updatedAt").map(str::to_string),
        completed_at: document.field("completedAt").map(str::to_string),
        blockers: values(document, "blockers"),
        references: values(document, "references"),
        related_files: values(document, "relatedFiles"),
        parent: parent.map(Box::new),
        children,
        accord,
        accord_counts: (document.doc_type() == "task")
            .then(|| crate::app::accord::counts(&read.events, document.id())),
        validation,
        resolution,
        decision: (document.doc_type() == "decision").then(|| decision(document)),
    })
}

/// Builds the ADR projection for one decision document.
pub(crate) fn decision(document: &Document) -> DecisionDto {
    DecisionDto {
        id: document.id().to_string(),
        title: document.title().to_string(),
        status: document.field("status").map(str::to_string),
        date: document.field("date").map(str::to_string),
        deciders: values(document, "deciders"),
        context: document.field("context").map(str::to_string),
        consequences: values(document, "consequences"),
        alternatives: values(document, "alternatives"),
        supersedes: values(document, "supersedes"),
        superseded_by: values(document, "supersededBy"),
        summary: document
            .body
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string),
    }
}

fn values(document: &Document, key: &str) -> Vec<String> {
    document
        .field(key)
        .map(parse_field_values)
        .unwrap_or_default()
}
