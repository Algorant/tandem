//! Board state, canonical hierarchy-backed projection, row rendering helpers, and details.

use super::*;

mod render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoardArrangement {
    State,
    Epic,
}

impl BoardArrangement {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::State => "State",
            Self::Epic => "Epic",
        }
    }

    pub(super) fn toggled(self) -> Self {
        match self {
            Self::State => Self::Epic,
            Self::Epic => Self::State,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct BoardFilters {
    pub(super) tag: Option<String>,
    pub(super) priority: Option<String>,
}

impl BoardFilters {
    pub(super) fn is_active(&self) -> bool {
        self.tag.is_some() || self.priority.is_some()
    }

    pub(super) fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(tag) = self.tag.as_deref() {
            parts.push(format!("#{}", tag));
        }
        if let Some(priority) = self.priority.as_deref() {
            parts.push(format!("priority {}", priority));
        }
        if parts.is_empty() {
            "no Board filters".to_string()
        } else {
            format!("filter {}", parts.join(" · "))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct TuiHierarchySnapshot {
    pub(super) index: Option<HierarchyIndex>,
    pub(super) errors: Vec<String>,
}

impl TuiHierarchySnapshot {
    pub(super) fn from_documents(active_docs: &[Document], completed_logs: &[Document]) -> Self {
        match hierarchy_index_for(active_docs, completed_logs) {
            Ok(index) => Self {
                errors: index.task_hierarchy_errors(),
                index: Some(index),
            },
            Err(error) => Self {
                index: None,
                errors: vec![error.message],
            },
        }
    }

    pub(super) fn valid_index(&self) -> Option<&HierarchyIndex> {
        self.errors
            .is_empty()
            .then_some(self.index.as_ref())
            .flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BoardSubviewTab {
    pub(super) state: String,
    pub(super) count: usize,
}

pub(super) fn board_subview_tabs(
    states: &[String],
    docs: &[Document],
    filters: &BoardFilters,
) -> Vec<BoardSubviewTab> {
    states
        .iter()
        .map(|state| BoardSubviewTab {
            state: state.clone(),
            count: docs
                .iter()
                .filter(|doc| is_board_visible_doc(doc))
                .filter(|doc| document_state_label(doc) == state.as_str())
                .filter(|doc| board_filters_match(doc, filters))
                .count(),
        })
        .collect()
}

pub(super) fn state_tab_title(state: &str, count: usize) -> String {
    format!(" {} {} ", display_state_label(state), count)
}

pub(super) fn plural_suffix(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

pub(super) fn display_state_label(state: &str) -> String {
    state.trim().replace(['-', '_'], " ").to_uppercase()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct BoardRelationshipHints {
    pub(super) active_children: usize,
    pub(super) completed_children: usize,
}

impl BoardRelationshipHints {
    fn total_children(self) -> usize {
        self.active_children + self.completed_children
    }

    fn has_children(self) -> bool {
        self.total_children() > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BoardRelatedChild {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) state: String,
    pub(super) completed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct BoardRelationshipContext {
    pub(super) task_role: Option<TaskRole>,
    pub(super) parent_relationship: Option<ParentRelationship>,
    pub(super) parent_id: Option<String>,
    pub(super) parent_title: Option<String>,
    pub(super) parent_missing: bool,
    pub(super) hierarchy_error: Option<String>,
    pub(super) active_children: Vec<BoardRelatedChild>,
    pub(super) completed_children: Vec<BoardRelatedChild>,
}

impl BoardRelationshipContext {
    pub(super) fn hints(&self) -> BoardRelationshipHints {
        BoardRelationshipHints {
            active_children: self.active_children.len(),
            completed_children: self.completed_children.len(),
        }
    }

    fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }

    fn has_children(&self) -> bool {
        self.hints().has_children()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StateBoardEntryRole {
    Root,
    Child,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StateBoardEntry<'a> {
    pub(super) doc: &'a Document,
    pub(super) role: StateBoardEntryRole,
    pub(super) task_role: Option<TaskRole>,
    pub(super) depth: usize,
    pub(super) active_descendants: usize,
    pub(super) completed_descendants: usize,
    pub(super) has_active_children: bool,
    pub(super) expanded: bool,
    pub(super) last_sibling: bool,
}

#[cfg(test)]
pub(super) fn state_board_entries<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    state: &str,
    filters: &BoardFilters,
    expanded_ids: &BTreeSet<String>,
) -> Vec<StateBoardEntry<'a>> {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    let Some(hierarchy) = snapshot.valid_index() else {
        return Vec::new();
    };
    state_board_entries_with_hierarchy(
        active_docs,
        completed_logs,
        state,
        filters,
        expanded_ids,
        hierarchy,
    )
}

pub(super) fn state_board_entries_with_hierarchy<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    state: &str,
    filters: &BoardFilters,
    expanded_ids: &BTreeSet<String>,
    hierarchy: &HierarchyIndex,
) -> Vec<StateBoardEntry<'a>> {
    let mut entries = Vec::new();
    for root in active_docs.iter().filter(|doc| {
        is_board_visible_doc(doc) && is_state_board_root(doc, active_docs, completed_logs)
    }) {
        let mut visited = BTreeSet::from([root.id().to_string()]);
        let root_matches_state = document_state_label(root) == state;
        let descendant_matches_state = is_task_doc(root)
            && task_subtree_matches_filters(
                root.id(),
                active_docs,
                completed_logs,
                state,
                filters,
                &mut visited,
            );
        // A state pane must expose every task counted by its tab, even when that task
        // sits below an ancestor in another workflow state. Keep the ancestor path as
        // context and automatically open the matching branch.
        let subtree_matches =
            (root_matches_state && board_filters_match(root, filters)) || descendant_matches_state;
        if !subtree_matches {
            continue;
        }
        let (active_descendants, completed_descendants) = if is_task_doc(root) {
            count_task_descendants(
                root.id(),
                active_docs,
                completed_logs,
                &mut BTreeSet::from([root.id().to_string()]),
            )
        } else {
            (0, 0)
        };
        let has_active_children = active_descendants > 0;
        // Auto-expand only an ancestor whose own state differs from this pane. A
        // same-state hierarchy remains collapsed by default and user-controlled.
        let expanded =
            expanded_ids.contains(root.id()) || (descendant_matches_state && !root_matches_state);
        let role = if normalized_parent_id(root).is_some_and(|parent_id| {
            completed_logs
                .iter()
                .any(|parent| parent.id() == parent_id && is_task_doc(parent))
        }) {
            StateBoardEntryRole::Child
        } else {
            StateBoardEntryRole::Root
        };
        entries.push(StateBoardEntry {
            doc: root,
            role,
            task_role: hierarchy.task_role(root).ok().flatten(),
            depth: 0,
            active_descendants,
            completed_descendants,
            has_active_children,
            expanded,
            last_sibling: false,
        });
        if is_task_doc(root) {
            collect_visible_state_descendants(
                root.id(),
                1,
                (
                    active_docs,
                    completed_logs,
                    state,
                    filters,
                    expanded_ids,
                    expanded || filters.is_active(),
                    hierarchy,
                ),
                &mut BTreeSet::from([root.id().to_string()]),
                &mut entries,
            );
        }
    }
    mark_state_board_last_siblings(&mut entries);
    entries
}

pub(super) fn mark_state_board_last_siblings(entries: &mut [StateBoardEntry<'_>]) {
    for index in 0..entries.len() {
        let depth = entries[index].depth;
        if depth == 0 {
            continue;
        }
        entries[index].last_sibling = !entries[index + 1..]
            .iter()
            .take_while(|candidate| candidate.depth >= depth)
            .any(|candidate| candidate.depth == depth);
    }
}

pub(super) fn collect_visible_state_descendants<'a>(
    parent_id: &str,
    depth: usize,
    projection: (
        &'a [Document],
        &[Document],
        &str,
        &BoardFilters,
        &BTreeSet<String>,
        bool,
        &HierarchyIndex,
    ),
    visited: &mut BTreeSet<String>,
    entries: &mut Vec<StateBoardEntry<'a>>,
) {
    let (active_docs, completed_logs, target_state, filters, expanded_ids, parent_open, hierarchy) =
        projection;
    for child in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(child.id().to_string()) {
            continue;
        }
        let mut match_visited = visited.clone();
        let subtree_matches = !filters.is_active()
            || (document_state_label(child) == target_state && board_filters_match(child, filters))
            || task_subtree_matches_filters(
                child.id(),
                active_docs,
                completed_logs,
                target_state,
                filters,
                &mut match_visited,
            );
        if !parent_open || !subtree_matches {
            continue;
        }
        let (active_descendants, completed_descendants) = count_task_descendants(
            child.id(),
            active_docs,
            completed_logs,
            &mut BTreeSet::from([child.id().to_string()]),
        );
        let has_active_children = active_descendants > 0;
        let descendant_matches_state = task_subtree_matches_filters(
            child.id(),
            active_docs,
            completed_logs,
            target_state,
            filters,
            &mut BTreeSet::from([child.id().to_string()]),
        );
        let child_matches_state = document_state_label(child) == target_state;
        let expanded =
            expanded_ids.contains(child.id()) || (descendant_matches_state && !child_matches_state);
        let task_role = hierarchy.task_role(child).ok().flatten();
        entries.push(StateBoardEntry {
            doc: child,
            role: StateBoardEntryRole::Child,
            task_role,
            depth,
            active_descendants,
            completed_descendants,
            has_active_children,
            expanded,
            last_sibling: false,
        });
        collect_visible_state_descendants(
            child.id(),
            depth + 1,
            (
                active_docs,
                completed_logs,
                target_state,
                filters,
                expanded_ids,
                expanded || filters.is_active(),
                hierarchy,
            ),
            visited,
            entries,
        );
    }

    for completed in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(completed.id().to_string()) && parent_open {
            collect_visible_state_descendants(
                completed.id(),
                depth + 1,
                (
                    active_docs,
                    completed_logs,
                    target_state,
                    filters,
                    expanded_ids,
                    true,
                    hierarchy,
                ),
                visited,
                entries,
            );
        }
    }
}

pub(super) fn is_state_board_root(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
) -> bool {
    if !is_task_doc(doc) {
        return true;
    }
    let mut current = doc;
    let mut visited = vec![doc.id().to_string()];
    let mut saw_active_ancestor = false;
    loop {
        let Some(parent_id) = normalized_parent_id(current) else {
            return !saw_active_ancestor;
        };
        if let Some(cycle_start) = visited.iter().position(|id| id == &parent_id) {
            let cycle_root = visited[cycle_start..]
                .iter()
                .filter(|id| {
                    active_docs
                        .iter()
                        .any(|candidate| candidate.id() == id.as_str())
                })
                .min();
            return cycle_root.is_some_and(|id| id == doc.id());
        }
        visited.push(parent_id.clone());
        if let Some(parent) = active_docs
            .iter()
            .find(|candidate| candidate.id() == parent_id && is_task_doc(candidate))
        {
            saw_active_ancestor = true;
            current = parent;
            continue;
        }
        if let Some(parent) = completed_logs
            .iter()
            .find(|candidate| candidate.id() == parent_id && is_task_doc(candidate))
        {
            current = parent;
            continue;
        }
        return !saw_active_ancestor;
    }
}

pub(super) fn task_subtree_matches_filters(
    parent_id: &str,
    active_docs: &[Document],
    completed_logs: &[Document],
    target_state: &str,
    filters: &BoardFilters,
    visited: &mut BTreeSet<String>,
) -> bool {
    let active_match = active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
        .any(|child| {
            visited.insert(child.id().to_string())
                && ((document_state_label(child) == target_state
                    && board_filters_match(child, filters))
                    || task_subtree_matches_filters(
                        child.id(),
                        active_docs,
                        completed_logs,
                        target_state,
                        filters,
                        visited,
                    ))
        });
    active_match
        || completed_logs
            .iter()
            .filter(|doc| is_task_doc(doc))
            .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
            .any(|completed| {
                visited.insert(completed.id().to_string())
                    && task_subtree_matches_filters(
                        completed.id(),
                        active_docs,
                        completed_logs,
                        target_state,
                        filters,
                        visited,
                    )
            })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EpicBoardEntryRole {
    Epic,
    Task,
    Subtask,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct EpicBoardEntry<'a> {
    pub(super) doc: &'a Document,
    pub(super) role: EpicBoardEntryRole,
    pub(super) depth: usize,
    pub(super) active_descendants: usize,
    pub(super) completed_descendants: usize,
}

#[cfg(test)]
pub(super) fn epic_board_entries<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    filters: &BoardFilters,
) -> Vec<EpicBoardEntry<'a>> {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    let Some(hierarchy) = snapshot.valid_index() else {
        return Vec::new();
    };
    epic_board_entries_with_hierarchy(active_docs, completed_logs, filters, hierarchy)
}

pub(super) fn epic_board_entries_with_hierarchy<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    filters: &BoardFilters,
    hierarchy: &HierarchyIndex,
) -> Vec<EpicBoardEntry<'a>> {
    let mut entries = Vec::new();

    for epic in active_docs.iter().filter(|doc| {
        is_board_visible_doc(doc) && matches!(hierarchy.task_role(doc), Ok(Some(TaskRole::Epic)))
    }) {
        let mut visited = BTreeSet::from([epic.id().to_string()]);
        let mut descendants = Vec::new();
        collect_visible_epic_descendants(
            epic.id(),
            1,
            (active_docs, completed_logs, filters, hierarchy),
            &mut visited,
            &mut descendants,
        );
        if !board_filters_match(epic, filters) && descendants.is_empty() {
            continue;
        }
        let (active_descendants, completed_descendants) = count_task_descendants(
            epic.id(),
            active_docs,
            completed_logs,
            &mut BTreeSet::from([epic.id().to_string()]),
        );
        entries.push(EpicBoardEntry {
            doc: epic,
            role: EpicBoardEntryRole::Epic,
            depth: 0,
            active_descendants,
            completed_descendants,
        });
        entries.extend(
            descendants
                .into_iter()
                .map(|(doc, depth, role)| EpicBoardEntry {
                    doc,
                    role: match role {
                        TaskRole::Task => EpicBoardEntryRole::Task,
                        TaskRole::Subtask => EpicBoardEntryRole::Subtask,
                        TaskRole::Epic => EpicBoardEntryRole::Epic,
                    },
                    depth,
                    active_descendants: 0,
                    completed_descendants: 0,
                }),
        );
    }
    entries
}

pub(super) fn collect_visible_epic_descendants<'a>(
    parent_id: &str,
    depth: usize,
    projection: (&'a [Document], &[Document], &BoardFilters, &HierarchyIndex),
    visited: &mut BTreeSet<String>,
    entries: &mut Vec<(&'a Document, usize, TaskRole)>,
) -> bool {
    let (active_docs, completed_logs, filters, hierarchy) = projection;
    let mut any_visible = false;

    for child in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(child.id().to_string()) {
            continue;
        }
        let Ok(Some(role @ (TaskRole::Task | TaskRole::Subtask))) = hierarchy.task_role(child)
        else {
            continue;
        };
        let insert_at = entries.len();
        entries.push((child, depth, role));
        let descendant_visible = collect_visible_epic_descendants(
            child.id(),
            depth + 1,
            (active_docs, completed_logs, filters, hierarchy),
            visited,
            entries,
        );
        if board_filters_match(child, filters) || descendant_visible {
            any_visible = true;
        } else {
            entries.remove(insert_at);
        }
    }

    for completed in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(completed.id().to_string()) {
            continue;
        }
        if collect_visible_epic_descendants(
            completed.id(),
            depth + 1,
            (active_docs, completed_logs, filters, hierarchy),
            visited,
            entries,
        ) {
            any_visible = true;
        }
    }

    any_visible
}

pub(super) fn count_task_descendants(
    parent_id: &str,
    active_docs: &[Document],
    completed_logs: &[Document],
    visited: &mut BTreeSet<String>,
) -> (usize, usize) {
    let mut active = 0;
    let mut completed = 0;

    for doc in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(doc.id().to_string()) {
            active += 1;
            let nested = count_task_descendants(doc.id(), active_docs, completed_logs, visited);
            active += nested.0;
            completed += nested.1;
        }
    }

    for doc in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(doc.id().to_string()) {
            if completion_outcome(doc) == COMPLETION_OUTCOME_COMPLETED {
                completed += 1;
            }
            let nested = count_task_descendants(doc.id(), active_docs, completed_logs, visited);
            active += nested.0;
            completed += nested.1;
        }
    }

    (active, completed)
}

pub(super) fn hierarchy_index_for(
    active_docs: &[Document],
    completed_logs: &[Document],
) -> Result<HierarchyIndex, CliError> {
    Ok(HierarchyIndex::from_documents(
        active_docs
            .iter()
            .chain(completed_logs.iter())
            .cloned()
            .collect(),
    )?)
}

#[cfg(test)]
pub(super) fn relationship_context_for_doc(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
) -> BoardRelationshipContext {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    relationship_context_for_doc_with_hierarchy(
        doc,
        active_docs,
        completed_logs,
        snapshot.index.as_ref(),
    )
}

pub(super) fn relationship_context_for_doc_with_hierarchy(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
    hierarchy: Option<&HierarchyIndex>,
) -> BoardRelationshipContext {
    let (task_role, parent_relationship, hierarchy_error) = match hierarchy {
        Some(hierarchy) => {
            let task_role = hierarchy.task_role(doc);
            let relationship = hierarchy.relationship(doc);
            let validation = if doc.doc_type() == "task" {
                hierarchy.validate_task_hierarchy(doc).map(|_| ())
            } else {
                Ok(())
            };
            match (task_role, relationship, validation) {
                (Ok(role), Ok(relationship), Ok(())) => (role, relationship, None),
                (role, relationship, Err(error)) => (
                    role.ok().flatten(),
                    relationship.ok().flatten(),
                    Some(error.message.clone()),
                ),
                (Err(error), _, _) | (_, Err(error), _) => {
                    (None, None, Some(error.message.clone()))
                }
            }
        }
        None => (
            None,
            None,
            Some("Validation failed: hierarchy snapshot is unavailable".to_string()),
        ),
    };
    let parent_id = normalized_parent_id(doc).filter(|parent_id| parent_id.as_str() != doc.id());
    let parent_doc = parent_id.as_deref().and_then(|parent_id| {
        active_docs
            .iter()
            .chain(completed_logs.iter())
            .find(|candidate| candidate.id() == parent_id)
    });
    let parent_title = parent_doc.map(|parent| parent.title().to_string());
    let parent_missing = parent_id.is_some() && parent_doc.is_none();
    let active_children = if is_task_doc(doc) {
        active_docs
            .iter()
            .filter(|child| is_task_doc(child))
            .filter(|child| normalized_parent_id(child).as_deref() == Some(doc.id()))
            .filter(|child| child.id() != doc.id())
            .map(|child| related_child_summary(child, false))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let completed_children = if is_task_doc(doc) {
        completed_logs
            .iter()
            .filter(|child| is_task_doc(child))
            .filter(|child| completion_outcome(child) == COMPLETION_OUTCOME_COMPLETED)
            .filter(|child| normalized_parent_id(child).as_deref() == Some(doc.id()))
            .filter(|child| child.id() != doc.id())
            .map(|child| related_child_summary(child, true))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    BoardRelationshipContext {
        task_role,
        parent_relationship,
        parent_id,
        parent_title,
        parent_missing,
        hierarchy_error,
        active_children,
        completed_children,
    }
}

pub(super) fn related_child_summary(doc: &Document, completed: bool) -> BoardRelatedChild {
    BoardRelatedChild {
        id: doc.id().to_string(),
        title: doc.title().to_string(),
        state: if completed {
            "log".to_string()
        } else {
            document_state_label(doc)
        },
        completed,
    }
}

pub(super) fn normalized_parent_id(doc: &Document) -> Option<String> {
    doc.field("parentId")
        .map(str::trim)
        .filter(|parent_id| !parent_id.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
pub(super) fn board_item_lines_for_doc(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    board_item_lines_for_doc_with_context(
        doc,
        theme,
        content_width,
        show_doc_type,
        &BoardRelationshipContext::default(),
        expanded,
        selected,
    )
}

#[cfg(test)]
pub(super) fn board_item_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    relationship_context: &BoardRelationshipContext,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    board_item_lines_for_doc_with_context_and_limit(
        doc,
        theme,
        relationship_context,
        (
            content_width,
            show_doc_type,
            INLINE_PREVIEW_MAX_LINES,
            expanded,
            selected,
        ),
    )
}

#[cfg(test)]
pub(super) fn board_item_lines_for_doc_with_context_and_limit(
    doc: &Document,
    theme: &TuiTheme,
    relationship_context: &BoardRelationshipContext,
    layout: (usize, bool, usize, bool, bool),
) -> Vec<Line<'static>> {
    let (content_width, show_doc_type, preview_line_limit, expanded, selected) = layout;
    // Board rows are intentionally sparse. The Board is for scanning and choosing work;
    // details belong in expanded rows and the detail pane. Relationship context is shown
    // as nesting/expanded-row content, not noisy parent-id chips.
    let chips = board_scan_chips(doc, relationship_context.task_role, theme);

    let mut lines = vec![board_row_line(
        doc,
        theme,
        chips,
        (
            content_width,
            show_doc_type,
            doc.id().to_string(),
            0,
            selected,
        ),
    )];
    if expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

pub(super) fn board_scan_chips(
    doc: &Document,
    task_role: Option<TaskRole>,
    theme: &TuiTheme,
) -> Vec<(String, Style)> {
    let priority = doc.field("priority").unwrap_or("-");
    let mut chips = Vec::new();
    if let Some(priority_chip) = priority_chip(priority, theme) {
        chips.push((priority_chip, theme.priority_chip_style(priority)));
    }
    if task_role == Some(TaskRole::Epic) {
        chips.push((
            chip_text("EPIC", theme),
            theme.progress_chip_style(StatusTone::Accent),
        ));
    }
    for (kind_chip, tone) in work_type_tag_chips(doc, theme) {
        chips.push((kind_chip, theme.progress_chip_style(tone)));
    }
    if let Some((visual_chip, tone)) = validation_visual_chip(doc, theme) {
        chips.push((visual_chip, theme.progress_chip_style(tone)));
    }
    for (tag_chip, tone) in configured_tag_chips(doc, theme) {
        chips.push((tag_chip, theme.progress_chip_style(tone)));
    }
    if let Some(accord) =
        accord_status(doc).filter(|status| board_should_surface_accord_status(doc, status, theme))
    {
        chips.push((status_chip(accord, theme), theme.accord_chip_style(accord)));
    }
    if let Some(review) =
        review_status(doc).filter(|status| board_should_surface_review_status(status, theme))
    {
        chips.push((status_chip(review, theme), theme.review_chip_style(review)));
    }
    if let Some((completed, total)) = subtask_progress(doc).filter(|(completed, total)| {
        !theme.badge_disabled("subtasks")
            && !theme.badge_disabled("subtask-progress")
            && (*completed > 0 || completed == total)
    }) {
        let tone = if completed == total {
            StatusTone::Success
        } else {
            StatusTone::Warning
        };
        chips.push((
            chip_text(&format!("{completed}/{total}"), theme),
            theme.progress_chip_style(tone),
        ));
    }
    chips
}

pub(super) fn state_list_item_for_entry(
    entry: &StateBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    layout: (usize, bool, usize, bool, bool),
) -> ListItem<'static> {
    let (content_width, show_doc_type, preview_line_limit, preview_expanded, selected) = layout;
    ListItem::new(state_lines_for_entry(
        entry,
        relationship_context,
        theme,
        (
            content_width,
            show_doc_type,
            preview_line_limit,
            preview_expanded,
            selected,
        ),
    ))
}

pub(super) fn state_lines_for_entry(
    entry: &StateBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    layout: (usize, bool, usize, bool, bool),
) -> Vec<Line<'static>> {
    let (content_width, show_doc_type, preview_line_limit, preview_expanded, selected) = layout;
    let doc = entry.doc;
    debug_assert!(entry.task_role.is_none() || is_task_doc(doc));
    let meta_width = content_width.saturating_div(2).min(32);
    let right_meta = if entry.active_descendants + entry.completed_descendants > 0 {
        truncate(
            &descendant_rollup(entry.active_descendants, entry.completed_descendants),
            meta_width,
        )
    } else {
        String::new()
    };
    let mut chips = vec![(state_hierarchy_prefix(entry), theme.muted_style())];
    chips.push(board_id_chip(doc, theme));
    let state = compact_epic_state(&document_state_label(doc));
    chips.push((
        chip_text(&format!("{state:<4}"), theme),
        theme.state_chip_style(&document_state_label(doc)),
    ));
    match entry.role {
        StateBoardEntryRole::Root => chips.extend(board_scan_chips(doc, entry.task_role, theme)),
        StateBoardEntryRole::Child if entry.depth == 0 => {
            chips.extend(board_scan_chips(doc, entry.task_role, theme));
        }
        StateBoardEntryRole::Child => {}
    }
    let mut lines = vec![board_row_line(
        doc,
        theme,
        chips,
        (
            content_width,
            show_doc_type && entry.depth == 0,
            right_meta,
            0,
            selected,
        ),
    )];
    if preview_expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

pub(super) fn state_hierarchy_prefix(entry: &StateBoardEntry<'_>) -> String {
    if entry.depth == 0 {
        return if entry.has_active_children {
            if entry.expanded {
                "▾"
            } else {
                "▸"
            }
        } else {
            " "
        }
        .to_string();
    }
    let branch = if entry.last_sibling { "└" } else { "├" };
    let disclosure = if entry.has_active_children {
        if entry.expanded {
            "▾"
        } else {
            "▸"
        }
    } else {
        "─"
    };
    format!(
        "{}{}{}",
        "│  ".repeat(entry.depth.saturating_sub(1)),
        branch,
        disclosure
    )
}

pub(super) fn epic_list_item_for_entry(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    preview_line_limit: usize,
    expanded: bool,
    selected: bool,
) -> ListItem<'static> {
    ListItem::new(epic_lines_for_entry(
        entry,
        relationship_context,
        theme,
        content_width,
        preview_line_limit,
        expanded,
        selected,
    ))
}

pub(super) fn epic_lines_for_entry(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    preview_line_limit: usize,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    let doc = entry.doc;
    let mut lines = vec![epic_row_line(
        entry,
        relationship_context,
        theme,
        content_width,
        selected,
    )];
    if expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

pub(super) const EPIC_META_COLUMN_WIDTH: usize = 32;
pub(super) const EPIC_MIN_TITLE_WIDTH: usize = 8;

pub(super) fn epic_row_line(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    selected: bool,
) -> Line<'static> {
    let doc = entry.doc;
    let indent = "  ".repeat(entry.depth);
    let title_style = if selected {
        theme.board_selected_title_style()
    } else {
        theme.text_style().add_modifier(Modifier::BOLD)
    };
    let mut prefix = vec![Span::styled(indent.clone(), theme.muted_style())];
    let mut prefix_width = match entry.role {
        EpicBoardEntryRole::Epic => {
            prefix.push(Span::styled(
                chip_text("EPIC", theme),
                theme.progress_chip_style(StatusTone::Accent),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent) + text_width(&chip_text("EPIC", theme)) + 1
        }
        EpicBoardEntryRole::Task => {
            let state = compact_epic_state(&document_state_label(doc));
            prefix.push(Span::styled(
                chip_text(&format!("{state:<4}"), theme),
                theme.state_chip_style(&document_state_label(doc)),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent) + text_width(&chip_text(&format!("{state:<4}"), theme)) + 1
        }
        EpicBoardEntryRole::Subtask => {
            let state = compact_epic_state(&document_state_label(doc));
            prefix.push(Span::styled(
                chip_text("SUB", theme),
                theme.progress_chip_style(StatusTone::Accent),
            ));
            prefix.push(Span::raw(" "));
            prefix.push(Span::styled(
                chip_text(&format!("{state:<4}"), theme),
                theme.state_chip_style(&document_state_label(doc)),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent)
                + text_width(&chip_text("SUB", theme))
                + text_width(&chip_text(&format!("{state:<4}"), theme))
                + 2
        }
    };
    let (id_chip, id_style) = board_id_chip(doc, theme);
    prefix_width += text_width(&id_chip) + 1;
    prefix.push(Span::styled(id_chip, id_style));
    prefix.push(Span::raw(" "));

    let meta_column_width = EPIC_META_COLUMN_WIDTH
        .min(content_width.saturating_sub(prefix_width + EPIC_MIN_TITLE_WIDTH + 1));
    let show_meta = meta_column_width >= 7;
    let meta_column_width = if show_meta { meta_column_width } else { 0 };
    let raw_meta = match entry.role {
        EpicBoardEntryRole::Epic => {
            descendant_rollup(entry.active_descendants, entry.completed_descendants)
        }
        EpicBoardEntryRole::Task | EpicBoardEntryRole::Subtask => compact_relationship_meta(
            relationship_context.parent_id.as_deref().unwrap_or("?"),
            doc.id(),
            meta_column_width,
        ),
    };
    let meta = if show_meta {
        truncate(&raw_meta, meta_column_width)
    } else {
        String::new()
    };
    let title_width =
        content_width.saturating_sub(prefix_width + meta_column_width + usize::from(show_meta));
    let title = truncate(doc.title(), title_width);
    let spacer_width = content_width
        .saturating_sub(prefix_width + text_width(&title) + meta_column_width)
        .max(usize::from(show_meta));

    prefix.push(Span::styled(title, title_style));
    prefix.push(Span::raw(" ".repeat(spacer_width)));
    if show_meta {
        prefix.push(Span::styled(
            format!("{meta:<meta_column_width$}"),
            theme.muted_style(),
        ));
    }
    Line::from(prefix)
}

pub(super) fn board_id_chip(doc: &Document, theme: &TuiTheme) -> (String, Style) {
    let id = doc
        .id()
        .strip_prefix("task-")
        .map(|suffix| format!("#{suffix}"))
        .unwrap_or_else(|| doc.id().to_string());
    (chip_text(&id, theme), theme.muted_style())
}

pub(super) fn compact_epic_state(state: &str) -> String {
    match normalize_filter_value(state).as_str() {
        "todo" => "TODO".to_string(),
        "in-progress" | "inprogress" | "doing" => "WIP".to_string(),
        "validation" | "review" => "VAL".to_string(),
        "blocked" => "BLK".to_string(),
        "ready" => "RDY".to_string(),
        "backlog" => "BACK".to_string(),
        "unfiled" => "UNFD".to_string(),
        "" => "UNFD".to_string(),
        other => truncate(&other.to_ascii_uppercase().replace('_', "-"), 4),
    }
}

pub(super) fn compact_relationship_meta(parent_id: &str, child_id: &str, width: usize) -> String {
    let full = format!("{parent_id} → {child_id}");
    if text_width(&full) <= width {
        return full;
    }
    if width < 7 {
        return String::new();
    }
    let id_width = width.saturating_sub(3);
    let parent_width = id_width / 2;
    let child_width = id_width.saturating_sub(parent_width);
    format!(
        "{} → {}",
        truncate(parent_id, parent_width),
        truncate(child_id, child_width)
    )
}

pub(super) fn board_row_line(
    doc: &Document,
    theme: &TuiTheme,
    chips: Vec<(String, Style)>,
    layout: (usize, bool, String, usize, bool),
) -> Line<'static> {
    let (content_width, show_doc_type, right_meta, depth, selected) = layout;
    let doc_type = doc_type_badge(doc, show_doc_type);
    let indent_width = depth.saturating_mul(3);
    let chip_width = chips
        .iter()
        .map(|(chip, _)| text_width(chip))
        .sum::<usize>()
        + chips.len();
    let doc_type_width = doc_type
        .as_ref()
        .map(|badge| text_width(badge) + 1)
        .unwrap_or(0);
    let title_separator_width = if chip_width > 0 || doc_type_width > 0 || indent_width > 0 {
        1
    } else {
        0
    };
    let base_width = indent_width + doc_type_width + chip_width + title_separator_width;
    let max_meta_width = content_width.saturating_sub(base_width).saturating_sub(1);
    let right_meta = truncate(&right_meta, max_meta_width);
    let meta_width = text_width(&right_meta);
    let spacer_min_width = if right_meta.is_empty() { 0 } else { 1 };
    let fixed_width = base_width + spacer_min_width + meta_width;
    let title_width = content_width.saturating_sub(fixed_width);
    let title = truncate(doc.title(), title_width);
    let used_before_meta =
        indent_width + doc_type_width + chip_width + title_separator_width + text_width(&title);
    let spacer_width = content_width
        .saturating_sub(used_before_meta + meta_width)
        .max(spacer_min_width);

    let mut spans = Vec::new();
    if indent_width > 0 {
        spans.push(Span::styled("  └".to_string(), theme.muted_style()));
        if indent_width > 3 {
            spans.push(Span::raw(" ".repeat(indent_width - 3)));
        }
    }
    if let Some(doc_type) = doc_type {
        spans.push(Span::styled(doc_type, theme.board_doc_type_style()));
        spans.push(Span::raw(" "));
    }
    for (index, (chip, style)) in chips.into_iter().enumerate() {
        if index > 0 || doc_type_width > 0 || indent_width > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(chip, style));
    }
    if chip_width > 0 || doc_type_width > 0 || indent_width > 0 {
        spans.push(Span::raw(" "));
    }
    let title_style = if selected {
        theme.board_selected_title_style()
    } else {
        theme.text_style().add_modifier(Modifier::BOLD)
    };
    spans.push(Span::styled(title, title_style));
    spans.push(Span::raw(" ".repeat(spacer_width)));
    spans.push(Span::styled(right_meta, theme.muted_style()));
    Line::from(spans)
}

pub(super) fn descendant_rollup(active: usize, completed: usize) -> String {
    match (active, completed) {
        (0, 0) => "no descendants".to_string(),
        (active, 0) => format!("{active} active"),
        (0, completed) => format!("{completed} logged"),
        (active, completed) => format!("{active} active · {completed} logged"),
    }
}

pub(super) fn relationship_detail_summary(context: &BoardRelationshipContext) -> String {
    let hints = context.hints();
    let mut parts = Vec::new();
    if hints.active_children > 0 {
        parts.push(format!(
            "{} active {}",
            hints.active_children,
            if hints.active_children == 1 {
                "child"
            } else {
                "children"
            }
        ));
    }
    if hints.completed_children > 0 {
        parts.push(format!(
            "{} completed {} in Logs",
            hints.completed_children,
            if hints.completed_children == 1 {
                "child"
            } else {
                "children"
            }
        ));
    }
    if parts.len() > 1 {
        format!("{} ({} total)", parts.join(", "), hints.total_children())
    } else if parts.is_empty() {
        "no linked children".to_string()
    } else {
        parts.join(", ")
    }
}

pub(super) fn doc_type_badge(doc: &Document, show_doc_type: bool) -> Option<String> {
    let doc_type = doc.doc_type().trim();
    if doc_type.is_empty() || (!show_doc_type && doc_type == "task") {
        None
    } else {
        Some(doc_type.to_string())
    }
}

pub(super) fn is_task_doc(doc: &Document) -> bool {
    doc.doc_type() == "task"
}

pub(super) fn priority_chip(priority: &str, theme: &TuiTheme) -> Option<String> {
    let normalized = priority.trim().to_ascii_lowercase();
    let (label, badge_id) = match normalized.as_str() {
        "critical" | "urgent" => ("CRIT".to_string(), "priority:critical"),
        "high" => ("HIGH".to_string(), "priority:high"),
        "medium" | "med" => ("MED".to_string(), "priority:medium"),
        "low" => ("LOW".to_string(), "priority:low"),
        "" | "-" | "none" => return None,
        other => (
            other.chars().take(4).collect::<String>().to_uppercase(),
            "priority:other",
        ),
    };
    if theme.badge_disabled("priority") || theme.badge_disabled(badge_id) {
        return None;
    }
    Some(chip_text(&label, theme))
}

pub(super) fn work_type_tag_chips(doc: &Document, theme: &TuiTheme) -> Vec<(String, StatusTone)> {
    let tags = document_tags(doc);
    let mut chips = Vec::new();
    for (tag, default_label) in [
        ("research", "RESEARCH"),
        ("spike", "SPIKE"),
        ("deliverable", "DELIVERABLE"),
    ] {
        if tags.iter().any(|candidate| tag_matches(candidate, tag))
            && !theme.badge_disabled(tag)
            && !theme.badge_disabled(&format!("tag:{tag}"))
        {
            chips.push(configured_or_default_tag_chip(tag, default_label, theme));
        }
    }
    chips
}

pub(super) fn configured_tag_chips(doc: &Document, theme: &TuiTheme) -> Vec<(String, StatusTone)> {
    document_tags(doc)
        .into_iter()
        .filter(|tag| !is_builtin_work_type_tag(tag))
        .filter_map(|tag| {
            theme
                .tag_badge(&tag)
                .map(|config| (chip_text(&config.label_for(&tag), theme), config.tone()))
        })
        .collect()
}

pub(super) fn configured_or_default_tag_chip(
    tag: &str,
    default_label: &str,
    theme: &TuiTheme,
) -> (String, StatusTone) {
    if let Some(config) = theme.tag_badge(tag) {
        (chip_text(&config.label_for(tag), theme), config.tone())
    } else {
        (chip_text(default_label, theme), StatusTone::Accent)
    }
}

pub(super) fn is_builtin_work_type_tag(tag: &str) -> bool {
    ["research", "spike", "deliverable"]
        .iter()
        .any(|candidate| tag_matches(tag, candidate))
}

pub(super) fn tag_matches(candidate: &str, expected: &str) -> bool {
    candidate.trim().eq_ignore_ascii_case(expected)
}

pub(super) fn board_filter_bar_line(filters: &BoardFilters, theme: &TuiTheme) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            " FILTERS ",
            theme
                .status_style(StatusTone::Warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if let Some(tag) = filters.tag.as_deref() {
        spans.push(Span::styled(
            chip_text(&format!("#{}", tag), theme),
            theme.progress_chip_style(StatusTone::Accent),
        ));
        spans.push(Span::raw(" "));
    }
    if let Some(priority) = filters.priority.as_deref() {
        spans.push(Span::styled(" priority ", theme.muted_style()));
        spans.push(Span::styled(
            chip_text(priority, theme),
            theme.priority_chip_style(priority),
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(" t/p cycle · F clear ", theme.muted_style()));
    Line::from(spans)
}

pub(super) fn board_filters_match(doc: &Document, filters: &BoardFilters) -> bool {
    if let Some(tag) = filters.tag.as_deref() {
        if !document_tags(doc)
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(tag))
        {
            return false;
        }
    }
    if let Some(priority) = filters.priority.as_deref() {
        if normalize_filter_value(doc.field("priority").unwrap_or("")) != priority {
            return false;
        }
    }
    true
}

pub(super) fn board_filter_tags(docs: &[Document]) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for doc in docs.iter().filter(|doc| is_board_visible_doc(doc)) {
        for tag in document_tags(doc) {
            tags.insert(tag);
        }
    }
    tags.into_iter().collect()
}

pub(super) fn board_filter_priorities(docs: &[Document]) -> Vec<String> {
    let mut priorities = docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc))
        .filter_map(|doc| {
            let priority = normalize_filter_value(doc.field("priority").unwrap_or(""));
            if priority.is_empty() || priority == "-" || priority == "none" {
                None
            } else {
                Some(priority)
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    priorities.sort_by_key(|priority| priority_filter_sort_key(priority));
    priorities
}

pub(super) fn next_filter_value(current: Option<&str>, values: &[String]) -> Option<String> {
    let next_index = current
        .and_then(|current| values.iter().position(|value| value == current))
        .map(|index| index + 1)
        .unwrap_or(0);
    values.get(next_index).cloned()
}

pub(super) fn priority_filter_sort_key(priority: &str) -> (usize, String) {
    let rank = match priority {
        "critical" | "urgent" => 0,
        "high" => 1,
        "medium" | "med" => 2,
        "low" => 3,
        _ => 4,
    };
    (rank, priority.to_string())
}

pub(super) fn document_tags(doc: &Document) -> Vec<String> {
    doc.field("tags")
        .map(|tags| format_inline_list(tags, ""))
        .unwrap_or_default()
        .into_iter()
        .map(|tag| normalize_filter_value(&tag))
        .filter(|tag| !tag.is_empty())
        .collect()
}

pub(super) fn normalize_filter_value(value: &str) -> String {
    value.trim().trim_start_matches('#').to_ascii_lowercase()
}

pub(super) fn status_chip(status: &str, theme: &TuiTheme) -> String {
    chip_text(&status.trim().replace('_', "-").to_uppercase(), theme)
}

pub(super) fn chip_text(label: &str, theme: &TuiTheme) -> String {
    theme.badge_label(label)
}

pub(super) const INLINE_PREVIEW_MAX_LINES: usize = 25;
pub(super) const BOARD_LIST_HIGHLIGHT_SYMBOL_WIDTH: u16 = 2;

pub(super) fn inline_preview_line_limit_for_area(area: Rect) -> usize {
    area.height.saturating_sub(2).saturating_sub(1) as usize
}

pub(super) fn inline_preview_height_with_context(
    doc: &Document,
    relationship_context: &BoardRelationshipContext,
    content_width: usize,
    preview_line_limit: usize,
) -> u16 {
    inline_preview_lines_for_doc_with_context(
        doc,
        &TuiTheme::default_dark(),
        relationship_context,
        content_width,
        preview_line_limit,
    )
    .len() as u16
}

pub(super) fn inline_preview_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    relationship_context: &BoardRelationshipContext,
    content_width: usize,
    preview_line_limit: usize,
) -> Vec<Line<'static>> {
    let max_lines = preview_line_limit.min(INLINE_PREVIEW_MAX_LINES);
    if max_lines == 0 {
        return Vec::new();
    }

    let footer_lines = max_lines.min(2);
    let content_limit = max_lines.saturating_sub(footer_lines);
    let files = doc
        .field("relatedFiles")
        .map(|files| format_inline_list(files, ""))
        .unwrap_or_default();
    let subtasks = board_subtasks(doc);
    let checklist_progress = subtask_progress(doc);
    let relationship_lines = inline_relationship_preview_lines(relationship_context, theme);
    let mut trailing_sections = validation_inline_preview_sections(doc, theme, content_width);
    trailing_sections.extend(inline_preview_list_section(
        "Files",
        files,
        theme,
        content_width,
    ));
    trailing_sections.extend(inline_preview_subtasks_section(
        subtasks,
        checklist_progress,
        theme,
    ));

    let mut content_lines = Vec::new();
    if let Some(tags) = doc.field("tags") {
        let tags = format_hash_list(tags);
        content_lines.extend(inline_preview_key_value(
            "Tags",
            &tags,
            theme,
            content_width,
        ));
        content_lines.push(Line::from(""));
    }

    if !relationship_lines.is_empty() {
        content_lines.extend(relationship_lines);
        content_lines.push(Line::from(""));
    }

    if content_lines.len() < content_limit {
        let (summary_label, summary_text) = inline_preview_summary(doc);
        let reserved = content_lines
            .len()
            .saturating_add(1)
            .saturating_add(trailing_sections.len());
        let summary_capacity = content_limit
            .saturating_sub(reserved)
            .clamp(1, INLINE_PREVIEW_MAX_LINES);
        content_lines.push(inline_preview_heading(summary_label, theme));
        content_lines.extend(inline_preview_markdownish(
            &summary_text,
            theme,
            content_width,
            summary_capacity,
        ));
    }

    content_lines.extend(trailing_sections);
    let overflow = content_lines.len() > content_limit;
    if overflow {
        content_lines.truncate(content_limit);
        if let Some(last) = content_lines.last_mut() {
            *last = Line::from(Span::styled("   …", theme.muted_style()));
        }
    }

    let mut lines = content_lines;
    if footer_lines == 2 {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "   Space close preview · Tab detail pane · e edit",
        theme.muted_style(),
    )));
    lines
}

pub(super) fn inline_preview_summary(doc: &Document) -> (&'static str, String) {
    if document_state_label(doc) == "validation" {
        if let Some(summary) = doc
            .field("accord.summary")
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
        {
            return ("Delivery summary", summary.to_string());
        }
    }
    ("Summary", doc.body.clone())
}

pub(super) fn validation_inline_preview_sections(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    if document_state_label(doc) != "validation" {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.extend(inline_preview_list_section(
        "Validation",
        first_accord_list(
            doc,
            &[
                "accord.validation.commands",
                "accord.validation",
                "accord.validations",
            ],
        ),
        theme,
        content_width,
    ));
    lines.extend(inline_preview_list_section(
        "Evidence",
        first_accord_list(doc, &["accord.evidence"]),
        theme,
        content_width,
    ));
    lines.extend(inline_preview_list_section(
        "Files changed",
        first_accord_list(doc, &["accord.filesChanged"]),
        theme,
        content_width,
    ));
    lines
}

pub(super) fn inline_relationship_preview_lines(
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if relationship_context.task_role == Some(TaskRole::Epic) || relationship_context.has_children()
    {
        let (heading, children_label) = match relationship_context.task_role {
            Some(TaskRole::Epic) => ("Epic", "Tasks"),
            Some(TaskRole::Task) => ("Task", "Subtasks"),
            Some(TaskRole::Subtask) => ("Subtask", "Children"),
            None => ("Relationships", "Children"),
        };
        lines.push(inline_preview_heading(heading, theme));
        if relationship_context.has_children() {
            lines.push(Line::from(vec![
                Span::styled(format!("   {children_label}: "), theme.label_style()),
                Span::styled(
                    relationship_detail_summary(relationship_context),
                    theme.text_style(),
                ),
            ]));
            for child in relationship_context
                .active_children
                .iter()
                .chain(relationship_context.completed_children.iter())
                .take(8)
            {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("   • {} ", display_state_label(&child.state)),
                        theme.muted_style(),
                    ),
                    Span::styled(child.title.clone(), theme.text_style()),
                    Span::styled(format!(" ({})", child.id), theme.muted_style()),
                ]));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "   No linked children yet. Set parentId on child tasks to attach them.",
                theme.muted_style(),
            )));
        }
    }

    if relationship_context.has_parent() {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(inline_preview_heading("Relationship", theme));
        if relationship_context.parent_missing {
            let parent_id = relationship_context
                .parent_id
                .as_deref()
                .unwrap_or("unknown");
            lines.push(Line::from(vec![
                Span::styled("   Missing parent: ", theme.label_style()),
                Span::styled(
                    parent_id.to_string(),
                    theme.status_style(StatusTone::Warning),
                ),
            ]));
        } else if let Some(parent_id) = relationship_context.parent_id.as_deref() {
            let parent_title = relationship_context
                .parent_title
                .as_deref()
                .unwrap_or("untitled parent");
            let label = match relationship_context.parent_relationship {
                Some(ParentRelationship::EpicTask) => "   Task of Epic: ",
                Some(ParentRelationship::Subtask) => "   Subtask of: ",
                Some(ParentRelationship::Parent) | None => "   Parent: ",
            };
            lines.push(Line::from(vec![
                Span::styled(label, theme.label_style()),
                Span::styled(parent_title.to_string(), theme.text_style()),
                Span::styled(format!(" ({parent_id})"), theme.muted_style()),
            ]));
        }
    }
    lines
}

pub(super) fn inline_preview_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(format!("   {label}"), theme.label_style()))
}

pub(super) fn inline_preview_key_value(
    label: &str,
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    let prefix = format!("   {label}: ");
    let value_width = content_width.saturating_sub(text_width(&prefix)).max(12);
    let wrapped = wrap_words(value, value_width);
    wrapped
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            if index == 0 {
                Line::from(vec![
                    Span::styled(prefix.clone(), theme.label_style()),
                    Span::styled(chunk, theme.text_style()),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" ".repeat(text_width(&prefix))),
                    Span::styled(chunk, theme.text_style()),
                ])
            }
        })
        .collect()
}

pub(super) fn inline_preview_markdownish(
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
    max_lines: usize,
) -> Vec<Line<'static>> {
    if max_lines == 0 {
        return Vec::new();
    }

    let indent = "   ";
    let value_width = content_width.saturating_sub(text_width(indent)).max(24);
    let mut raw_lines = preview_markdownish_wrapped_lines(value, value_width);
    if raw_lines.len() > max_lines {
        raw_lines.truncate(max_lines);
        if let Some(last) = raw_lines.last_mut() {
            *last = append_preview_ellipsis(last, value_width);
        }
    }

    raw_lines
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                Line::from("")
            } else {
                markdownish_line(&format!("{indent}{line}"), theme)
            }
        })
        .collect()
}

pub(super) fn preview_markdownish_wrapped_lines(value: &str, width: usize) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return vec!["(no body text)".to_string()];
    }

    let mut lines = Vec::new();
    for raw_line in trimmed.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(wrap_preview_markdownish_line(line, width));
        }
    }
    lines
}

pub(super) fn wrap_preview_markdownish_line(line: &str, width: usize) -> Vec<String> {
    let width = width.max(12);
    let trimmed = line.trim_start();
    let leading = &line[..line.len() - trimmed.len()];

    if markdown_table_like_line(trimmed) {
        return vec![truncate(line, width)];
    }

    if let Some((marker, content)) = markdown_list_source_parts(trimmed) {
        let prefix = format!("{leading}{marker}");
        let value_width = width.saturating_sub(text_width(&prefix)).max(12);
        return wrap_words(content, value_width)
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| {
                if index == 0 {
                    format!("{prefix}{chunk}")
                } else {
                    format!("{}{chunk}", " ".repeat(text_width(&prefix)))
                }
            })
            .collect();
    }

    if let Some(content) = trimmed
        .strip_prefix("> ")
        .or_else(|| trimmed.strip_prefix('>'))
    {
        let prefix = format!("{leading}> ");
        let value_width = width.saturating_sub(text_width(&prefix)).max(12);
        return wrap_words(content, value_width)
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| {
                if index == 0 {
                    format!("{prefix}{chunk}")
                } else {
                    format!("{}{chunk}", " ".repeat(text_width(&prefix)))
                }
            })
            .collect();
    }

    wrap_words(line, width)
}

pub(super) fn markdown_list_source_parts(trimmed: &str) -> Option<(&str, &str)> {
    for marker in [
        "- [ ] ", "* [ ] ", "+ [ ] ", "- [x] ", "* [x] ", "+ [x] ", "- [X] ", "* [X] ", "+ [X] ",
        "- ", "* ", "+ ",
    ] {
        if let Some(content) = trimmed.strip_prefix(marker) {
            return Some((&trimmed[..marker.len()], content));
        }
    }

    let digit_count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 || digit_count + 2 > trimmed.len() {
        return None;
    }
    let suffix = &trimmed[digit_count..];
    if suffix.starts_with(". ") || suffix.starts_with(") ") {
        Some((&trimmed[..digit_count + 2], &trimmed[digit_count + 2..]))
    } else {
        None
    }
}

pub(super) fn markdown_table_like_line(trimmed: &str) -> bool {
    let pipe_count = trimmed.chars().filter(|ch| *ch == '|').count();
    pipe_count >= 2 || trimmed.starts_with('|') || trimmed.ends_with('|')
}

pub(super) fn append_preview_ellipsis(line: &str, width: usize) -> String {
    let width = width.max(1);
    let char_count = line.chars().count();
    if char_count + 2 <= width {
        format!("{line} …")
    } else {
        let mut truncated = line
            .chars()
            .take(width.saturating_sub(1))
            .collect::<String>();
        truncated.push('…');
        truncated
    }
}

pub(super) fn inline_preview_list_section(
    label: &str,
    values: Vec<String>,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    if values.is_empty() {
        return Vec::new();
    }

    let total = values.len();
    let max_items = 4;
    let mut lines = vec![Line::from(""), inline_preview_heading(label, theme)];
    for value in values.into_iter().take(max_items) {
        lines.extend(inline_preview_bullet_value(&value, theme, content_width));
    }
    if total > max_items {
        lines.push(Line::from(Span::styled(
            format!("   … {} more", total - max_items),
            theme.muted_style(),
        )));
    }
    lines
}

pub(super) fn inline_preview_bullet_value(
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    let prefix = "   • ";
    let value_width = content_width.saturating_sub(text_width(prefix)).max(12);
    wrap_words(value, value_width)
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            if index == 0 {
                Line::from(vec![
                    Span::styled(prefix.to_string(), theme.muted_style()),
                    Span::styled(chunk, theme.text_style()),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" ".repeat(text_width(prefix))),
                    Span::styled(chunk, theme.text_style()),
                ])
            }
        })
        .collect()
}

pub(super) fn inline_preview_subtasks_section(
    subtasks: Vec<BoardSubtask>,
    checklist_progress: Option<(usize, usize)>,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    if subtasks.is_empty() {
        return Vec::new();
    }

    let (completed, total) = checklist_progress.unwrap_or((0, subtasks.len()));
    let mut lines = vec![
        Line::from(""),
        inline_preview_heading(&format!("Checklist {completed}/{total}"), theme),
    ];
    for subtask in subtasks {
        let marker = if subtask.completed { "[x]" } else { "[ ]" };
        lines.push(Line::from(vec![
            Span::styled(format!("   {marker} "), theme.muted_style()),
            Span::styled(subtask.title, theme.text_style()),
        ]));
    }
    lines
}

pub(super) fn format_hash_list(value: &str) -> String {
    format_inline_list(value, "#").join(" ")
}

pub(super) fn format_inline_list(value: &str, prefix: &str) -> Vec<String> {
    let trimmed = value.trim();
    let values = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(clean_inline_list_item)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    } else {
        trimmed
            .split(',')
            .map(clean_inline_list_item)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    };

    if values.is_empty() {
        vec![trimmed.to_string()]
    } else {
        values
            .into_iter()
            .map(|item| format!("{prefix}{item}"))
            .collect()
    }
}

pub(super) fn clean_inline_list_item(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

pub(super) fn wrap_words(value: &str, width: usize) -> Vec<String> {
    let width = width.max(12);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let separator = usize::from(!current.is_empty());
        if !current.is_empty() && text_width(&current) + separator + text_width(word) > width {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }
    lines
}

pub(super) fn board_should_surface_accord_status(
    doc: &Document,
    status: &str,
    theme: &TuiTheme,
) -> bool {
    let normalized = normalized_accord_status(status);
    if document_state_label(doc) == "validation" && normalized == "delivered" {
        return false;
    }
    matches!(
        normalized.as_str(),
        "delivered" | "accepted" | "rework" | "blocked" | "failed"
    ) && !theme.badge_disabled("accord")
        && !theme.badge_disabled(&normalized)
        && !theme.badge_disabled(&format!("accord:{normalized}"))
}

pub(super) fn validation_visual_chip(
    doc: &Document,
    theme: &TuiTheme,
) -> Option<(String, StatusTone)> {
    if document_state_label(doc) != "validation"
        || theme.badge_disabled("visual")
        || theme.badge_disabled("tag:visual")
        || theme.badge_disabled("validation:visual")
    {
        return None;
    }
    let tags = document_tags(doc);
    tags.iter()
        .any(|tag| {
            ["visual", "ui", "ux"]
                .iter()
                .any(|expected| tag_matches(tag, expected))
        })
        .then(|| configured_or_default_tag_chip("visual", "VISUAL", theme))
}

pub(super) fn board_should_surface_review_status(status: &str, theme: &TuiTheme) -> bool {
    let normalized = status.trim().replace('_', "-").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "pending" | "changes-requested" | "rejected" | "failed"
    ) && !theme.badge_disabled("review")
        && !theme.badge_disabled(&normalized)
        && !theme.badge_disabled(&format!("review:{normalized}"))
}

#[derive(Debug, Clone)]
pub(super) struct BoardSubtask {
    pub(super) title: String,
    pub(super) completed: bool,
}

pub(super) fn board_subtasks(doc: &Document) -> Vec<BoardSubtask> {
    let mut by_index: BTreeMap<usize, BoardSubtask> = BTreeMap::new();
    for (key, value) in &doc.fields {
        let Some(rest) = key.strip_prefix("subtasks.") else {
            continue;
        };
        let Some((index, field)) = rest.split_once('.') else {
            continue;
        };
        let Ok(index) = index.parse::<usize>() else {
            continue;
        };
        let entry = by_index.entry(index).or_insert_with(|| BoardSubtask {
            title: format!("subtask {}", index + 1),
            completed: false,
        });
        match field {
            "title" => entry.title = value.to_string(),
            "completed" => entry.completed = is_completed_value(value),
            _ => {}
        }
    }
    by_index.into_values().collect()
}

pub(super) fn is_completed_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "done" | "1"
    )
}

pub(super) fn subtask_progress(doc: &Document) -> Option<(usize, usize)> {
    let subtasks = board_subtasks(doc);
    let total = subtasks.len();
    let completed = subtasks.iter().filter(|subtask| subtask.completed).count();
    (total > 0).then_some((completed, total))
}

pub(super) fn text_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
pub(super) fn detail_lines_for_doc(doc: &Document, theme: &TuiTheme) -> Vec<Line<'static>> {
    detail_lines_for_doc_with_context(doc, theme, &BoardRelationshipContext::default())
}

pub(super) fn detail_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    relationship_context: &BoardRelationshipContext,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme.label_style()),
        Span::styled(doc.title().to_string(), theme.text_style()),
    ]));
    lines.push(detail_field_line("ID", doc.id(), theme));
    lines.push(detail_field_line("Type", doc.doc_type(), theme));
    push_optional_detail_line(&mut lines, "Kind", doc.field("kind"), theme);
    if let Some(role) = relationship_context.task_role {
        lines.push(detail_field_line("Role", role.as_str(), theme));
    }
    push_optional_detail_line(&mut lines, "State", doc.field("state"), theme);
    push_optional_detail_line(&mut lines, "Priority", doc.field("priority"), theme);
    push_optional_detail_line(&mut lines, "Effort", doc.field("effort"), theme);
    push_optional_detail_line(&mut lines, "Assignee", doc.field("assignee"), theme);
    push_optional_detail_line(&mut lines, "Due", doc.field("dueDate"), theme);
    push_optional_detail_line(&mut lines, "Tags", doc.field("tags"), theme);
    if let Some(parent_id) = relationship_context.parent_id.as_deref() {
        let parent = relationship_context
            .parent_title
            .as_deref()
            .map(|title| format!("{title} ({parent_id})"))
            .unwrap_or_else(|| format!("missing parent {parent_id}"));
        let label = relationship_context
            .parent_relationship
            .map(ParentRelationship::human_label)
            .unwrap_or("Parent");
        lines.push(detail_field_line(label, &parent, theme));
    }
    if let Some(error) = relationship_context.hierarchy_error.as_deref() {
        lines.push(detail_field_line("Hierarchy error", error, theme));
    }
    if relationship_context.has_children() {
        let label = match relationship_context.task_role {
            Some(TaskRole::Epic) => "Tasks",
            Some(TaskRole::Task) => "Subtasks",
            _ => "Children",
        };
        lines.push(detail_field_line(
            label,
            &relationship_detail_summary(relationship_context),
            theme,
        ));
    }
    push_optional_detail_line(&mut lines, "Accord", accord_status(doc), theme);
    push_optional_detail_line(&mut lines, "Review", review_status(doc), theme);
    push_optional_detail_line(&mut lines, "Updated", doc.field("updatedAt"), theme);
    lines.push(detail_field_line("Path", &display_path(&doc.path), theme));
    push_board_accord_detail_section(&mut lines, doc, theme);
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Body", theme));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&doc.body, theme));
    }
    lines
}

pub(super) fn push_board_accord_detail_section(
    lines: &mut Vec<Line<'static>>,
    doc: &Document,
    theme: &TuiTheme,
) {
    if doc.doc_type() != "task" {
        return;
    }

    let status = accord_status(doc).unwrap_or("missing").trim();
    let status = if status.is_empty() { "missing" } else { status };

    lines.push(Line::from(""));
    lines.push(detail_section_heading("Accord", theme));
    lines.push(detail_status_line(
        "Status",
        status,
        accord_detail_status_style(status, theme),
        theme,
    ));
    if let Some(warning) = accord::state_divergence_warning(doc) {
        let warning_style = theme.status_style(StatusTone::Warning);
        lines.push(Line::from(vec![
            Span::styled("Warning: ", warning_style.add_modifier(Modifier::BOLD)),
            Span::styled(warning, warning_style),
        ]));
    }
    lines.push(detail_field_line(
        "Signal",
        accord_state_signal(status),
        theme,
    ));
    push_optional_detail_line(
        lines,
        "Accord assignee",
        doc.field("accord.assignee"),
        theme,
    );
    push_optional_detail_line(lines, "Claimed", doc.field("accord.claimedAt"), theme);
    push_optional_detail_line(lines, "Delivered", doc.field("accord.deliveredAt"), theme);
    push_optional_detail_list_line(
        lines,
        "Deliverables",
        first_accord_list(doc, &["accord.deliverables"]),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Validation",
        first_accord_list(
            doc,
            &[
                "accord.validation.commands",
                "accord.validation",
                "accord.validations",
            ],
        ),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Constraints",
        first_accord_list(doc, &["accord.constraints"]),
        theme,
    );
    push_optional_detail_line(lines, "Summary", doc.field("accord.summary"), theme);
    push_optional_detail_list_line(
        lines,
        "Evidence",
        first_accord_list(doc, &["accord.evidence"]),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Files changed",
        first_accord_list(doc, &["accord.filesChanged"]),
        theme,
    );
    push_optional_detail_line(lines, "Reviewer", doc.field("accord.reviewer"), theme);
    push_optional_detail_line(lines, "Note", doc.field("accord.note"), theme);
    push_optional_detail_line(lines, "Reason", doc.field("accord.reason"), theme);
    push_optional_detail_line(
        lines,
        "Accord updated",
        doc.field("accord.updatedAt"),
        theme,
    );
    lines.push(detail_field_line("Next", accord_next_action(status), theme));
    lines.push(Line::from(vec![
        Span::styled("CLI hint: ", theme.label_style()),
        Span::styled(accord_cli_hint(doc.id(), status), theme.text_style()),
    ]));
    if document_state_label(doc) == "validation" {
        lines.push(Line::from(Span::styled(
            "Board Validation: A opens accept sign-off confirmation, R opens feedback/rework, e opens the task; completion is intentionally separate.",
            theme.muted_style(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Board accord mutations beyond movement are CLI-guided from this detail pane.",
            theme.muted_style(),
        )));
    }
}

pub(super) fn detail_field_line(label: &str, value: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), theme.text_style()),
    ])
}

pub(super) fn detail_status_line(
    label: &str,
    value: &str,
    style: Style,
    theme: &TuiTheme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), style),
    ])
}

pub(super) fn detail_section_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    ))
}

pub(super) fn push_optional_detail_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(detail_field_line(label, value, theme));
    }
}

pub(super) fn push_optional_detail_list_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    values: Vec<String>,
    theme: &TuiTheme,
) {
    if !values.is_empty() {
        lines.push(detail_field_line(label, &values.join(", "), theme));
    }
}

pub(super) fn first_accord_list(doc: &Document, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .filter_map(|key| doc.field(key).map(parse_field_values))
        .find(|values| !values.is_empty())
        .unwrap_or_default()
}

pub(super) fn accord_detail_status_style(status: &str, theme: &TuiTheme) -> Style {
    if normalized_accord_status(status) == "missing" {
        theme.muted_style()
    } else {
        theme.accord_style(status).add_modifier(Modifier::BOLD)
    }
}

pub(super) fn accord_state_signal(status: &str) -> &'static str {
    match normalized_accord_status(status).as_str() {
        "ready" => "Legacy ready: treat as unclaimed and claim when an owner is known.",
        "claimed" => "Claimed: an owner is actively working the accord.",
        "delivered" => "Delivered: inspect summary/evidence, then accept or request rework.",
        "accepted" => "Accepted: accord review passed; completion/logging is still separate.",
        "rework" => "Rework: changes were requested before the accord can be accepted.",
        "blocked" => "Blocked: work cannot proceed until the recorded reason is resolved.",
        "failed" => "Failed: the accord attempt ended unsuccessfully and needs review.",
        "missing" | "" => "Missing: no accord metadata is recorded for this task yet.",
        _ => "Unknown: inspect the raw task before changing accord state.",
    }
}

pub(super) fn accord_next_action(status: &str) -> &'static str {
    match normalized_accord_status(status).as_str() {
        "ready" => "Legacy status: claim the accord when an owner is known.",
        "claimed" => "Deliver when complete, or block/fail with a reason if work cannot proceed.",
        "delivered" => "Inspect the delivery, then accept it or request rework.",
        "accepted" => "Complete/archive the task when it is ready to leave the Board.",
        "rework" => "Apply requested changes, then deliver again with a fresh summary.",
        "blocked" => "Resolve the blocker, then claim/deliver; fail only if unrecoverable.",
        "failed" => "Review the failure and claim again if retrying the work.",
        "missing" | "" => "Claim the accord when an owner is known.",
        _ => "Inspect current metadata before choosing the next accord action.",
    }
}

pub(super) fn accord_cli_hint(id: &str, status: &str) -> String {
    match normalized_accord_status(status).as_str() {
        "ready" => format!("tandem accord claim {id} --assignee <name>"),
        "claimed" => format!(
            "tandem accord deliver {id} --summary <text> [--evidence <text>] [--file-changed <path>]"
        ),
        "delivered" => format!(
            "tandem accord accept {id} [--reviewer <name>] [--note <text>] OR tandem accord rework {id} --note <text>"
        ),
        "accepted" => format!(
            "tandem complete {id} --summary <text> [--validation <text>] [--reviewer <name>]"
        ),
        "rework" => format!("tandem accord deliver {id} --summary <text> [--evidence <text>]"),
        "blocked" => format!(
            "tandem accord claim {id} --assignee <name> OR tandem accord fail {id} --reason <text>"
        ),
        "failed" => format!("tandem accord claim {id} --assignee <name>"),
        "missing" | "" => format!(
            "tandem accord claim {id} --assignee <name> [--deliverable <spec>] [--validation <command>]"
        ),
        _ => format!("tandem show {id}  # inspect accord metadata before mutating"),
    }
}

pub(super) fn normalized_accord_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace('_', "-")
}

pub(super) fn markdownish_lines(markdown: &str, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_fence = false;

    for line in markdown.lines() {
        if markdown_fence_marker(line).is_some() {
            lines.push(markdown_code_fence_line(line, theme));
            in_code_fence = !in_code_fence;
        } else if in_code_fence {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.markdown_code_style(),
            )));
        } else {
            lines.push(markdownish_line(line, theme));
        }
    }

    lines
}

pub(super) fn markdownish_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return Line::from("");
    }

    let indent = &line[..line.len() - trimmed.len()];
    if let Some(heading) = markdown_heading_text(trimmed) {
        return Line::from(with_indent(
            indent,
            vec![Span::styled(
                heading.to_string(),
                theme.markdown_heading_style().add_modifier(Modifier::BOLD),
            )],
            theme,
        ));
    }

    if let Some(quote) = markdown_blockquote_text(trimmed) {
        let mut spans = with_indent(indent, vec![Span::styled("│ ", theme.muted_style())], theme);
        spans.extend(markdown_inline_spans(
            quote,
            theme,
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
        return Line::from(spans);
    }

    if let Some((marker, content)) = markdown_list_parts(trimmed) {
        let mut spans = with_indent(
            indent,
            vec![Span::styled(marker, theme.markdown_list_style())],
            theme,
        );
        spans.extend(markdown_inline_spans(content, theme, theme.text_style()));
        return Line::from(spans);
    }

    Line::from(markdown_inline_spans(line, theme, theme.text_style()))
}

pub(super) fn markdown_fence_marker(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

pub(super) fn markdown_code_fence_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let marker = markdown_fence_marker(line).unwrap_or("```");
    let info = trimmed[marker.len()..].trim();
    let mut spans = with_indent(
        indent,
        vec![Span::styled(
            marker.to_string(),
            theme.markdown_code_style().add_modifier(Modifier::BOLD),
        )],
        theme,
    );
    if !info.is_empty() {
        spans.push(Span::styled(" ", theme.markdown_code_style()));
        spans.push(Span::styled(info.to_string(), theme.muted_style()));
    }
    Line::from(spans)
}

pub(super) fn markdown_heading_text(trimmed: &str) -> Option<&str> {
    let marker_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }

    let rest = &trimmed[marker_count..];
    if rest.is_empty() || rest.chars().next().is_some_and(|ch| ch.is_whitespace()) {
        let heading = rest.trim_start();
        Some(if heading.is_empty() { trimmed } else { heading })
    } else {
        None
    }
}

pub(super) fn markdown_blockquote_text(trimmed: &str) -> Option<&str> {
    let quote = trimmed.strip_prefix('>')?;
    Some(quote.strip_prefix(' ').unwrap_or(quote))
}

pub(super) fn markdown_list_parts(trimmed: &str) -> Option<(String, &str)> {
    for bullet in ["-", "*", "+"] {
        let unchecked = format!("{bullet} [ ] ");
        if let Some(content) = trimmed.strip_prefix(&unchecked) {
            return Some(("☐ ".to_string(), content));
        }
        let checked_lower = format!("{bullet} [x] ");
        if let Some(content) = trimmed.strip_prefix(&checked_lower) {
            return Some(("☑ ".to_string(), content));
        }
        let checked_upper = format!("{bullet} [X] ");
        if let Some(content) = trimmed.strip_prefix(&checked_upper) {
            return Some(("☑ ".to_string(), content));
        }
        let marker = format!("{bullet} ");
        if let Some(content) = trimmed.strip_prefix(&marker) {
            return Some(("• ".to_string(), content));
        }
    }

    let digit_count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 || digit_count + 2 > trimmed.len() {
        return None;
    }
    let suffix = &trimmed[digit_count..];
    if suffix.starts_with(". ") || suffix.starts_with(") ") {
        Some((
            trimmed[..digit_count + 2].to_string(),
            &trimmed[digit_count + 2..],
        ))
    } else {
        None
    }
}

pub(super) fn markdown_inline_spans(
    text: &str,
    theme: &TuiTheme,
    base_style: Style,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(strong) = rest.strip_prefix("**") {
            if let Some(end) = strong.find("**") {
                push_span(
                    &mut spans,
                    &strong[..end],
                    base_style.add_modifier(Modifier::BOLD),
                );
                rest = &strong[end + 2..];
                continue;
            }
        }

        if let Some(code) = rest.strip_prefix('`') {
            if let Some(end) = code.find('`') {
                push_span(&mut spans, &code[..end], theme.markdown_code_style());
                rest = &code[end + 1..];
                continue;
            }
        }

        if let Some((consumed, label, url)) = markdown_link_parts(rest) {
            let link_label = if label.is_empty() { url } else { label };
            push_span(
                &mut spans,
                link_label,
                theme
                    .status_style(StatusTone::Accent)
                    .add_modifier(Modifier::UNDERLINED),
            );
            if !url.is_empty() && url != label {
                push_span(&mut spans, " (", theme.muted_style());
                push_span(&mut spans, url, theme.muted_style());
                push_span(&mut spans, ")", theme.muted_style());
            }
            rest = &rest[consumed..];
            continue;
        }

        let next_special = next_markdown_inline_special(rest).unwrap_or(rest.len());
        if next_special > 0 {
            push_span(&mut spans, &rest[..next_special], base_style);
            rest = &rest[next_special..];
        } else {
            let ch = rest.chars().next().expect("rest is not empty");
            push_span(&mut spans, &rest[..ch.len_utf8()], base_style);
            rest = &rest[ch.len_utf8()..];
        }
    }

    spans
}

pub(super) fn markdown_link_parts(text: &str) -> Option<(usize, &str, &str)> {
    let label_rest = text.strip_prefix('[')?;
    let label_end = label_rest.find(']')?;
    let after_label = &label_rest[label_end + 1..];
    let url_rest = after_label.strip_prefix('(')?;
    let url_end = url_rest.find(')')?;
    let consumed = 1 + label_end + 1 + 1 + url_end + 1;
    Some((consumed, &label_rest[..label_end], &url_rest[..url_end]))
}

pub(super) fn next_markdown_inline_special(text: &str) -> Option<usize> {
    ['`', '[', '*']
        .iter()
        .filter_map(|needle| text.find(*needle))
        .min()
}

pub(super) fn with_indent(
    indent: &str,
    mut spans: Vec<Span<'static>>,
    theme: &TuiTheme,
) -> Vec<Span<'static>> {
    if !indent.is_empty() {
        spans.insert(0, Span::styled(indent.to_string(), theme.text_style()));
    }
    spans
}

pub(super) fn push_span(spans: &mut Vec<Span<'static>>, content: &str, style: Style) {
    if !content.is_empty() {
        spans.push(Span::styled(content.to_string(), style));
    }
}
