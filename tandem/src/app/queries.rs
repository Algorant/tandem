//! Canonical read/query composition shared by peer interfaces.

use crate::app::support::hierarchy_from_project;
use crate::project::rules::parse_rules_from_content;
use crate::project::write::HierarchyLock;
use crate::project::{ProjectHierarchy, StoredDocument as Document, StoredPapercut, TandemProject};
use crate::protocol::accord::{state_divergence_warning, status as accord_status};
use crate::protocol::config::RulesByCategory;
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::review::status as review_status;
use crate::protocol::workflow::{state_matches_filter, workflow_states};
use crate::CliError;

pub(crate) struct Snapshot {
    pub(crate) hierarchy: ProjectHierarchy,
}

/// One coherent, UI-neutral project read used by long-running peer interfaces.
pub(crate) struct ReadSnapshot {
    pub(crate) snapshot: Snapshot,
    pub(crate) revision: String,
    pub(crate) title: String,
    pub(crate) protocol_version: String,
    pub(crate) states: Vec<String>,
    pub(crate) rules: RulesByCategory,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct ListFilter<'a> {
    pub(crate) state: Option<&'a str>,
    pub(crate) doc_type: Option<&'a str>,
    pub(crate) priority: Option<&'a str>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) assignee: Option<&'a str>,
    pub(crate) parent: Option<&'a str>,
    pub(crate) accord: Option<&'a str>,
    pub(crate) review: Option<&'a str>,
}

#[derive(Debug)]
pub(crate) struct SearchResult {
    pub(crate) doc: Document,
    pub(crate) snippet: String,
}

#[derive(Debug)]
pub(crate) struct PapercutSearchResult {
    pub(crate) papercut: StoredPapercut,
    pub(crate) snippet: String,
}

#[derive(Debug)]
pub(crate) struct SearchFilter<'a> {
    pub(crate) query: &'a str,
    pub(crate) state: Option<&'a str>,
    pub(crate) doc_type: Option<&'a str>,
    pub(crate) parent: Option<&'a str>,
}

pub(crate) fn load(project: &TandemProject) -> Result<Snapshot, CliError> {
    let _lock = HierarchyLock::acquire(project)?;
    let hierarchy = hierarchy_from_project(project)?;
    hierarchy.validate_all_task_hierarchies()?;
    Ok(Snapshot { hierarchy })
}

pub(crate) fn load_read(project: &TandemProject) -> Result<ReadSnapshot, CliError> {
    let _lock = HierarchyLock::acquire(project)?;
    let config = project.read_config_raw()?;
    let documents = project.read_documents()?;
    let hierarchy = ProjectHierarchy::from_documents(documents.clone())?;
    hierarchy.validate_document_metadata()?;
    hierarchy.validate_all_task_hierarchies()?;
    let config_yaml = project.read_config_yaml()?;
    let title = config_yaml
        .as_ref()
        .and_then(|root| crate::project::yaml_mapping_value(root, "title"))
        .and_then(crate::project::yaml_scalar_to_string)
        .unwrap_or_else(|| crate::app::project::default_title(project.root()));
    let protocol_version = crate::app::project::protocol_version(project)?;
    let states = workflow_states(config_yaml.as_ref());
    let rules = parse_rules_from_content(&config, &project.config_path)?;
    let revision = crate::project::snapshot_revision(&config, &documents);
    let mut warnings = crate::app::project::warnings(project)?;
    for document in hierarchy.documents.values() {
        if let Some(warning) = state_divergence_warning(document) {
            warnings.push(warning);
        }
        for reference in document
            .field("references")
            .map(parse_field_values)
            .unwrap_or_default()
        {
            if !project.reference_target_exists(&reference)? {
                warnings.push(format!(
                    "{} references missing target {reference}.",
                    document.id()
                ));
            }
        }
    }
    for items in rules.values() {
        for item in items {
            if item
                .source
                .as_deref()
                .is_some_and(|source| hierarchy.document(source).is_none())
            {
                warnings.push(format!(
                    "Rule {} references missing source {}.",
                    item.id,
                    item.source.as_deref().expect("checked source")
                ));
            }
        }
    }
    warnings.sort();
    warnings.dedup();
    Ok(ReadSnapshot {
        snapshot: Snapshot { hierarchy },
        revision,
        title,
        protocol_version,
        states,
        rules,
        warnings,
    })
}

impl Snapshot {
    pub(crate) fn board_documents(&self, filter: &ListFilter<'_>) -> Vec<Document> {
        filter_documents(
            self.hierarchy
                .documents
                .values()
                .filter(|doc| doc.location == DocumentLocation::Board)
                .cloned()
                .collect(),
            filter,
        )
    }

    pub(crate) fn all_documents(&self) -> Vec<Document> {
        self.hierarchy.documents.values().cloned().collect()
    }

    pub(crate) fn log_documents(&self) -> Vec<Document> {
        self.hierarchy
            .documents
            .values()
            .filter(|doc| doc.location == DocumentLocation::Logs)
            .cloned()
            .collect()
    }

    pub(crate) fn document(&self, id: &str) -> Option<Document> {
        self.hierarchy.document(id).cloned()
    }

    pub(crate) fn log_document(&self, id: &str) -> Option<Document> {
        self.hierarchy
            .document(id)
            .filter(|doc| doc.location == DocumentLocation::Logs)
            .cloned()
    }

    pub(crate) fn children(&self, parent: &Document) -> Result<Vec<Document>, CliError> {
        children_for(&self.hierarchy, parent)
    }

    pub(crate) fn relationships_for(
        &self,
        documents: &[Document],
    ) -> Result<std::collections::BTreeMap<String, Option<ParentRelationship>>, CliError> {
        documents
            .iter()
            .map(|document| {
                Ok((
                    document.id().to_string(),
                    self.hierarchy.relationship(document)?,
                ))
            })
            .collect()
    }
}

pub(crate) fn children_for(
    hierarchy: &ProjectHierarchy,
    parent: &Document,
) -> Result<Vec<Document>, CliError> {
    let Some(parent_role) = hierarchy.task_role(parent)? else {
        return Ok(Vec::new());
    };
    if parent_role == TaskRole::Subtask {
        return Ok(Vec::new());
    }
    let expected = match parent_role {
        TaskRole::Epic => TaskRole::Task,
        TaskRole::Task => TaskRole::Subtask,
        TaskRole::Subtask => unreachable!(),
    };
    let mut children = hierarchy
        .documents
        .values()
        .filter(|doc| doc.doc_type() == "task" && doc.field("parentId") == Some(parent.id()))
        .filter_map(|doc| match hierarchy.task_role(doc) {
            Ok(Some(role)) if role == expected => Some(Ok(doc.clone())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort_by(|a, b| {
        a.location
            .as_str()
            .cmp(b.location.as_str())
            .then_with(|| {
                a.field("state")
                    .unwrap_or("")
                    .cmp(b.field("state").unwrap_or(""))
            })
            .then_with(|| a.id().cmp(b.id()))
    });
    Ok(children)
}

pub(crate) fn filter_documents(docs: Vec<Document>, filter: &ListFilter<'_>) -> Vec<Document> {
    docs.into_iter()
        .filter(|doc| matches_filter(doc, filter))
        .collect()
}

pub(crate) fn search_documents(
    docs: Vec<Document>,
    filter: &SearchFilter<'_>,
) -> Vec<SearchResult> {
    let mut results = docs
        .into_iter()
        .filter(|doc| filter.doc_type.is_none_or(|kind| doc.doc_type() == kind))
        .filter(|doc| {
            if doc.location == DocumentLocation::Logs {
                filter.state.is_none()
            } else {
                filter
                    .state
                    .is_none_or(|state| state_matches_filter(doc.field("state"), state))
            }
        })
        .filter(|doc| {
            filter
                .parent
                .is_none_or(|parent| doc.field("parentId") == Some(parent))
        })
        .filter_map(|doc| search_match(doc, filter.query))
        .collect::<Vec<_>>();
    results.sort_by(|a, b| a.doc.id().cmp(b.doc.id()));
    results
}

pub(crate) fn search_papercuts(
    items: Vec<StoredPapercut>,
    query: &str,
) -> Vec<PapercutSearchResult> {
    let lowered_query = query.to_lowercase();
    let mut results = items
        .into_iter()
        .filter_map(|papercut| {
            let mut haystacks = vec![
                papercut.id().to_string(),
                papercut.title().to_string(),
                papercut.body.clone(),
            ];
            for key in ["status", "tags", "references", "resolution.note"] {
                if let Some(value) = papercut.field(key) {
                    haystacks.push(value.to_string());
                }
            }
            haystacks
                .into_iter()
                .find(|value| value.to_lowercase().contains(&lowered_query))
                .map(|matched| PapercutSearchResult {
                    papercut,
                    snippet: snippet_for_match(&matched, query),
                })
        })
        .collect::<Vec<_>>();
    results.sort_by_key(|result| {
        crate::protocol::papercut::papercut_number(result.papercut.id()).unwrap_or(usize::MAX)
    });
    results
}

pub(crate) fn search_match(doc: Document, query: &str) -> Option<SearchResult> {
    let lowered_query = query.to_lowercase();
    let mut haystacks = vec![
        doc.id().to_string(),
        doc.title().to_string(),
        doc.body.clone(),
    ];
    haystacks.extend(doc.fields.values().cloned());
    for haystack in haystacks {
        if haystack.to_lowercase().contains(&lowered_query) {
            return Some(SearchResult {
                doc,
                snippet: snippet_for_match(&haystack, query),
            });
        }
    }
    None
}

fn snippet_for_match(value: &str, query: &str) -> String {
    let condensed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if condensed.chars().count() <= 80 {
        return condensed;
    }
    let lower = condensed.to_lowercase();
    let query_lower = query.to_lowercase();
    let byte_index = lower.find(&query_lower).unwrap_or(0);
    let char_index = condensed[..byte_index].chars().count();
    let start = char_index.saturating_sub(20);
    let end = (start + 80).min(condensed.chars().count());
    let chars = condensed.chars().collect::<Vec<_>>();
    let mut snippet = chars[start..end].iter().collect::<String>();
    if start > 0 {
        snippet.insert(0, '…');
    }
    if end < chars.len() {
        snippet.push('…');
    }
    snippet
}

fn matches_filter(doc: &Document, filter: &ListFilter<'_>) -> bool {
    filter
        .state
        .is_none_or(|state| state_matches_filter(doc.field("state"), state))
        && filter
            .doc_type
            .is_none_or(|doc_type| doc.doc_type() == doc_type)
        && filter
            .priority
            .is_none_or(|priority| doc.field("priority") == Some(priority))
        && filter
            .assignee
            .is_none_or(|assignee| doc.field("assignee") == Some(assignee))
        && filter
            .parent
            .is_none_or(|parent| doc.field("parentId") == Some(parent))
        && filter.tag.is_none_or(|tag| {
            parse_field_values(doc.field("tags").unwrap_or(""))
                .iter()
                .any(|value| value == tag)
        })
        && filter
            .accord
            .is_none_or(|status| accord_status(doc) == Some(status))
        && filter
            .review
            .is_none_or(|status| review_status(doc) == Some(status))
}
