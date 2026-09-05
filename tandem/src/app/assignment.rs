//! Complete current-assignment reads for native coordination clients.
//!
//! An assignment is a Task plus its direct Subtasks. This projection is
//! intentionally complete for definition fields and does not create a second
//! task store or persist readiness state.

use serde::Serialize;

use crate::app::queries::{self, ReadSnapshot};
use crate::app::Error;
use crate::project::{ProjectHierarchy, StoredDocument as Document, TandemProject};
use crate::protocol::accord::{status as accord_status, AccordRecord};
use crate::protocol::assignment::{definition_token, dependency_resolution};
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, TaskRole};
use crate::protocol::workflow::resolution_outcome;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssignmentOutcome {
    pub(crate) data: AssignmentDto,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssignmentDto {
    pub(crate) definition_token: String,
    pub(crate) root: AssignmentNodeDto,
    pub(crate) milestones: Vec<AssignmentNodeDto>,
    pub(crate) dependency_readiness: DependencyReadinessDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssignmentNodeDto {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) document_type: String,
    pub(crate) role: &'static str,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) location: &'static str,
    pub(crate) state: Option<String>,
    pub(crate) accord_status: Option<String>,
    pub(crate) resolution_outcome: Option<String>,
    pub(crate) acceptance: Vec<String>,
    pub(crate) constraints: Vec<String>,
    pub(crate) planned_validation: Vec<String>,
    pub(crate) owned_scope: Vec<String>,
    pub(crate) dependencies: Vec<DependencyDto>,
    pub(crate) ready: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DependencyDto {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) ready: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DependencyReadinessDto {
    pub(crate) all_clear: bool,
    pub(crate) issues: Vec<ReadinessIssueDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadinessIssueDto {
    pub(crate) owner_id: String,
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) reason: String,
}

/// Read one Task assignment from one coherent native project snapshot.
pub(crate) fn read(project: &TandemProject, id: &str) -> Result<AssignmentOutcome, Error> {
    let read = queries::load_read(project)?;
    let root = read.snapshot.document(id).ok_or_else(|| {
        Error::new(
            crate::app::ErrorKind::NotFound,
            format!("document not found: {id}"),
        )
    })?;
    let role = read.snapshot.hierarchy.task_role(&root)?;
    if role != Some(TaskRole::Task) {
        return Err(Error::user(format!(
            "assignment root must be a Task: {id} is {}",
            role.map(TaskRole::as_str).unwrap_or("not a task")
        )));
    }

    let milestones = read.snapshot.children(&root)?;
    let mut token_documents = Vec::with_capacity(milestones.len() + 1);
    token_documents.push(&*root);
    token_documents.extend(milestones.iter().map(|milestone| &**milestone));
    let definition_token = definition_token(&token_documents);

    let root_node = node(&read, &read.snapshot.hierarchy, &root)?;
    let milestone_nodes = milestones
        .iter()
        .map(|milestone| node(&read, &read.snapshot.hierarchy, milestone))
        .collect::<Result<Vec<_>, _>>()?;

    let mut issues = Vec::new();
    for item in [&root_node].into_iter().chain(milestone_nodes.iter()) {
        for dependency in &item.dependencies {
            if !dependency.ready {
                issues.push(ReadinessIssueDto {
                    owner_id: item.id.clone(),
                    id: dependency.id.clone(),
                    status: dependency.status.clone(),
                    reason: dependency.reason.clone(),
                });
            }
        }
    }
    Ok(AssignmentOutcome {
        data: AssignmentDto {
            definition_token,
            root: root_node,
            milestones: milestone_nodes,
            dependency_readiness: DependencyReadinessDto {
                all_clear: issues.is_empty(),
                issues,
            },
        },
        warnings: read.warnings,
    })
}

fn node(
    read: &ReadSnapshot,
    hierarchy: &ProjectHierarchy,
    document: &Document,
) -> Result<AssignmentNodeDto, Error> {
    let role = hierarchy.task_role(document)?.ok_or_else(|| {
        Error::user(format!(
            "assignment document is not a task: {}",
            document.id()
        ))
    })?;
    let dependencies = document
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default()
        .into_iter()
        .map(|id| dependency(read, &id))
        .collect::<Vec<_>>();
    let ready = dependencies.iter().all(|dependency| dependency.ready);
    let accord = accord_status(document)
        .map(|_| AccordRecord::from_document(document, document.field("updatedAt").unwrap_or("")));
    Ok(AssignmentNodeDto {
        id: document.id().to_string(),
        document_type: document.doc_type().to_string(),
        role: role.as_str(),
        title: document.title().to_string(),
        body: document.body.clone(),
        location: document.location.as_str(),
        state: document.field("state").map(str::to_string),
        accord_status: accord.as_ref().map(|record| record.status.clone()),
        resolution_outcome: (document.location == DocumentLocation::Logs)
            .then(|| resolution_outcome(document).to_string()),
        acceptance: accord
            .as_ref()
            .map(|record| record.acceptance.clone())
            .unwrap_or_default(),
        constraints: accord
            .as_ref()
            .map(|record| record.constraints.clone())
            .unwrap_or_default(),
        planned_validation: accord
            .as_ref()
            .map(|record| record.validations.clone())
            .unwrap_or_default(),
        owned_scope: document
            .field("relatedFiles")
            .map(parse_field_values)
            .unwrap_or_default(),
        dependencies,
        ready,
    })
}

fn dependency(read: &ReadSnapshot, id: &str) -> DependencyDto {
    let document = read
        .snapshot
        .hierarchy
        .document(id)
        .map(|document| (&**document, document.location));
    let resolution = dependency_resolution(document);
    DependencyDto {
        id: id.to_string(),
        status: resolution.status.to_string(),
        ready: resolution.clear,
        reason: resolution.reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tasks::{add, AddOptions};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn project(name: &str) -> TandemProject {
        let root = std::env::temp_dir().join(format!(
            "tandem-assignment-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.3.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap()
    }

    #[test]
    fn reads_complete_root_and_milestone_definition_without_truncation() {
        let project = project("complete");
        let root = add(
            &project,
            AddOptions {
                title: Some("Root".to_string()),
                description: Some("root body with tail ROOT-TAIL".to_string()),
                acceptance: vec!["root acceptance".to_string()],
                constraints: vec!["root constraint".to_string()],
                validations: vec!["root validation".to_string()],
                related_files: vec!["src/root.rs".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        for index in 1..=10 {
            add(
                &project,
                AddOptions {
                    title: Some(format!("Milestone {index}")),
                    description: Some(format!("milestone body {index} MILESTONE-TAIL-{index}")),
                    acceptance: vec![format!("acceptance {index}")],
                    constraints: vec![format!("constraint {index}")],
                    validations: vec![format!("validation {index}")],
                    related_files: vec![format!("src/milestone-{index}.rs")],
                    parent: Some(root.id.clone()),
                    ..Default::default()
                },
            )
            .unwrap();
        }
        let outcome = read(&project, &root.id).unwrap();
        assert!(outcome
            .data
            .root
            .body
            .contains("root body with tail ROOT-TAIL"));
        assert_eq!(outcome.data.milestones.len(), 10);
        let last = outcome.data.milestones.last().unwrap();
        assert_eq!(last.id, "task-1-10");
        assert!(last.body.contains("milestone body 10 MILESTONE-TAIL-10"));
        assert_eq!(last.acceptance, ["acceptance 10"]);
        assert_eq!(last.constraints, ["constraint 10"]);
        assert_eq!(last.planned_validation, ["validation 10"]);
        assert_eq!(last.owned_scope, ["src/milestone-10.rs"]);
        assert!(outcome.data.dependency_readiness.all_clear);
        fs::remove_dir_all(project.root()).unwrap();
    }
}
