//! Canonical resolved task hierarchy, relationship, and structural validation.
//!
//! This module never discovers, reads, locks, or writes a project. Callers
//! build an index from one coherent Board-and-Logs snapshot and use its queries
//! for CLI and TUI projection.

use std::collections::{BTreeSet, HashMap};

use crate::{display_path, CliError, Document};

use super::document::{validate_task_kind, EFFORTS, PRIORITIES};
use super::ids::{global_task_number, subtask_suffix};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskRole {
    Epic,
    Task,
    Subtask,
}

impl TaskRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Epic => "epic",
            Self::Task => "task",
            Self::Subtask => "subtask",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentRelationship {
    EpicTask,
    Subtask,
    Parent,
}

impl ParentRelationship {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::EpicTask => "epic-task",
            Self::Subtask => "subtask",
            Self::Parent => "parent",
        }
    }

    pub(crate) fn human_label(self) -> &'static str {
        match self {
            Self::EpicTask => "Task of Epic",
            Self::Subtask => "Subtask of",
            Self::Parent => "Parent",
        }
    }
}

/// Canonical Board-and-Logs graph for role, relationship, and validation queries.
#[derive(Debug, Clone)]
pub(crate) struct HierarchyIndex {
    pub(crate) documents: HashMap<String, Document>,
}

impl HierarchyIndex {
    pub(crate) fn from_documents(docs: Vec<Document>) -> Result<Self, CliError> {
        let mut documents = HashMap::new();
        for doc in docs {
            let id = doc.id().to_string();
            if id.trim().is_empty() {
                return Err(CliError::user(format!(
                    "Validation failed for {}: missing required field `id`",
                    display_path(&doc.path)
                )));
            }
            if let Some(existing) = documents.insert(id.clone(), doc.clone()) {
                return Err(CliError::user(format!(
                    "Validation failed: duplicate document ID `{id}` in {} and {}",
                    display_path(&existing.path),
                    display_path(&doc.path)
                )));
            }
        }
        Ok(Self { documents })
    }

    pub(crate) fn with_replacement(&self, doc: Document) -> Self {
        let mut documents = self.documents.clone();
        documents.insert(doc.id().to_string(), doc);
        Self { documents }
    }

    pub(crate) fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    pub(crate) fn task_role(&self, doc: &Document) -> Result<Option<TaskRole>, CliError> {
        if doc.doc_type() != "task" {
            return Ok(None);
        }
        let mut roles = HashMap::new();
        let mut stack = Vec::new();
        self.task_role_by_id(doc.id(), &mut roles, &mut stack)
            .map(Some)
    }

    fn task_role_by_id(
        &self,
        id: &str,
        roles: &mut HashMap<String, TaskRole>,
        stack: &mut Vec<String>,
    ) -> Result<TaskRole, CliError> {
        if let Some(role) = roles.get(id) {
            return Ok(*role);
        }
        if let Some(cycle_start) = stack.iter().position(|entry| entry == id) {
            let mut cycle = stack[cycle_start..].to_vec();
            cycle.push(id.to_string());
            return Err(CliError::user(format!(
                "Validation failed: task hierarchy cycle: {}",
                cycle.join(" -> ")
            )));
        }
        let doc = self.document(id).ok_or_else(|| {
            CliError::user(format!(
                "Validation failed: parent document not found: {id}"
            ))
        })?;
        if doc.doc_type() != "task" {
            return Err(CliError::user(format!(
                "Validation failed: {id} is type {}, not task",
                doc.doc_type()
            )));
        }
        if let Some(kind) = doc.field("kind") {
            validate_task_kind(kind).map_err(|message| {
                CliError::user(format!(
                    "Validation failed for {}: {message}",
                    display_path(&doc.path)
                ))
            })?;
        }
        if doc.kind() == Some("epic") {
            roles.insert(id.to_string(), TaskRole::Epic);
            return Ok(TaskRole::Epic);
        }

        stack.push(id.to_string());
        let role = match doc.field("parentId") {
            None => TaskRole::Task,
            Some(parent_id) => {
                let parent = self.document(parent_id).ok_or_else(|| {
                    CliError::user(format!(
                        "Validation failed for {}: unresolved parentId `{parent_id}`",
                        display_path(&doc.path)
                    ))
                })?;
                if parent.doc_type() != "task" {
                    TaskRole::Task
                } else {
                    match self.task_role_by_id(parent_id, roles, stack)? {
                        TaskRole::Epic => TaskRole::Task,
                        TaskRole::Task => TaskRole::Subtask,
                        TaskRole::Subtask => return Err(CliError::user(format!(
                            "Validation failed for {}: task {} cannot be a child of Subtask {parent_id}", display_path(&doc.path), doc.id()
                        ))),
                    }
                }
            }
        };
        stack.pop();
        roles.insert(id.to_string(), role);
        Ok(role)
    }

    pub(crate) fn relationship(
        &self,
        doc: &Document,
    ) -> Result<Option<ParentRelationship>, CliError> {
        let Some(parent_id) = doc.field("parentId") else {
            return Ok(None);
        };
        let parent = self.document(parent_id).ok_or_else(|| {
            CliError::user(format!(
                "Validation failed for {}: unresolved parentId `{parent_id}`",
                display_path(&doc.path)
            ))
        })?;
        if doc.doc_type() != "task" || parent.doc_type() != "task" {
            return Ok(Some(ParentRelationship::Parent));
        }
        Ok(Some(match self.task_role(parent)? {
            Some(TaskRole::Epic) => ParentRelationship::EpicTask,
            Some(TaskRole::Task) => ParentRelationship::Subtask,
            Some(TaskRole::Subtask) => {
                return Err(CliError::user(format!(
                    "Validation failed for {}: task {} cannot be a child of Subtask {parent_id}",
                    display_path(&doc.path),
                    doc.id()
                )))
            }
            None => ParentRelationship::Parent,
        }))
    }

    pub(crate) fn validate_task_hierarchy(&self, doc: &Document) -> Result<TaskRole, CliError> {
        let role = self.task_role(doc)?.ok_or_else(|| {
            CliError::user(format!("Validation failed: {} is not a task", doc.id()))
        })?;
        if role == TaskRole::Epic && doc.field("parentId").is_some() {
            return Err(CliError::user(format!(
                "Validation failed for {}: Epic {} cannot have parentId",
                display_path(&doc.path),
                doc.id()
            )));
        }
        let valid_id = match role {
            TaskRole::Epic | TaskRole::Task => global_task_number(doc.id()).is_some(),
            TaskRole::Subtask => doc
                .field("parentId")
                .is_some_and(|parent_id| subtask_suffix(doc.id(), parent_id).is_some()),
        };
        if !valid_id {
            let expected = match role {
                TaskRole::Epic | TaskRole::Task => "global `task-N`".to_string(),
                TaskRole::Subtask => format!(
                    "`{}-M` with a positive M",
                    doc.field("parentId").unwrap_or("task-N")
                ),
            };
            return Err(CliError::user(format!(
                "Validation failed for {}: {} {} has invalid ID `{}`; expected {expected}",
                display_path(&doc.path),
                role.as_str(),
                doc.title(),
                doc.id()
            )));
        }
        if role == TaskRole::Subtask
            && self
                .documents
                .values()
                .any(|child| child.field("parentId") == Some(doc.id()))
        {
            return Err(CliError::user(format!(
                "Validation failed for {}: Subtask {} cannot have children",
                display_path(&doc.path),
                doc.id()
            )));
        }
        Ok(role)
    }

    pub(crate) fn task_hierarchy_errors(&self) -> Vec<String> {
        let mut ids = self
            .documents
            .values()
            .filter(|doc| doc.doc_type() == "task")
            .map(|doc| doc.id().to_string())
            .collect::<Vec<_>>();
        ids.sort();
        let mut errors = BTreeSet::new();
        for id in ids {
            if let Err(error) =
                self.validate_task_hierarchy(self.document(&id).expect("indexed task"))
            {
                errors.insert(error.message);
            }
        }
        errors.into_iter().collect()
    }

    pub(crate) fn validate_document_metadata(&self) -> Result<(), CliError> {
        for doc in self
            .documents
            .values()
            .filter(|doc| doc.doc_type() == "task")
        {
            for (field, allowed) in [("priority", PRIORITIES), ("effort", EFFORTS)] {
                if let Some(value) = doc.field(field) {
                    if !allowed.contains(&value) {
                        return Err(CliError::user(format!(
                            "Validation failed for {}: invalid {field} `{value}`; expected one of: {}", display_path(&doc.path), allowed.join(", ")
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_all_task_hierarchies(&self) -> Result<(), CliError> {
        let errors = self.task_hierarchy_errors();
        match errors.len() {
            0 => Ok(()),
            1 => Err(CliError::user(
                errors.into_iter().next().expect("one hierarchy error"),
            )),
            count => Err(CliError::user(format!(
                "Validation failed: hierarchy contains {count} structural errors:\n- {}",
                errors.join("\n- ")
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::DocumentLocation;

    fn document(id: &str, parent_id: Option<&str>, kind: Option<&str>) -> Document {
        let mut fields = HashMap::from([
            ("id".to_string(), id.to_string()),
            ("type".to_string(), "task".to_string()),
            ("title".to_string(), id.to_string()),
        ]);
        if let Some(parent_id) = parent_id {
            fields.insert("parentId".to_string(), parent_id.to_string());
        }
        if let Some(kind) = kind {
            fields.insert("kind".to_string(), kind.to_string());
        }
        Document::new(
            PathBuf::from(format!("{id}.md")),
            DocumentLocation::Board,
            fields,
            String::new(),
        )
    }

    #[test]
    fn decision_seven_roles_and_id_forms_are_strict() {
        let valid = HierarchyIndex::from_documents(vec![
            document("task-1", None, Some("epic")),
            document("task-2", Some("task-1"), None),
            document("task-2-1", Some("task-2"), None),
        ])
        .unwrap();
        assert_eq!(
            valid.task_role(valid.document("task-2").unwrap()).unwrap(),
            Some(TaskRole::Task)
        );
        assert_eq!(
            valid
                .relationship(valid.document("task-2").unwrap())
                .unwrap(),
            Some(ParentRelationship::EpicTask)
        );
        assert_eq!(
            valid
                .task_role(valid.document("task-2-1").unwrap())
                .unwrap(),
            Some(TaskRole::Subtask)
        );

        let invalid = HierarchyIndex::from_documents(vec![
            document("task-1", None, Some("epic")),
            document("task-1-1", Some("task-1"), None),
        ])
        .unwrap();
        assert!(invalid
            .validate_all_task_hierarchies()
            .unwrap_err()
            .message
            .contains("expected global `task-N`"));
    }
}
