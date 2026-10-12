//! Board action pickers. Filter and Validation use one interaction grammar.

use super::*;

#[derive(Debug, Clone)]
pub(super) struct WorkspacePickerEntry {
    alias: String,
    root: Option<std::path::PathBuf>,
    summary: String,
    current: bool,
    valid: bool,
}

#[derive(Debug, Clone)]
pub(super) struct WorkspacePicker {
    entries: Vec<WorkspacePickerEntry>,
    filtered: Vec<usize>,
    query: String,
    selected: usize,
    add_path: Option<String>,
    feedback: Option<String>,
}

impl WorkspacePicker {
    fn apply_filter(&mut self) {
        let query = self.query.to_lowercase();
        self.filtered = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                (query.is_empty()
                    || entry.alias.to_lowercase().contains(&query)
                    || entry.root.as_ref().is_some_and(|root| {
                        root.display().to_string().to_lowercase().contains(&query)
                    })
                    || entry.summary.to_lowercase().contains(&query))
                .then_some(index)
            })
            .collect();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PickerKind {
    Filter,
    Actions,
    Validation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PickerAction {
    SetKind(Option<String>),
    SetTag(Option<String>),
    SetPriority(Option<String>),
    SetDeliveredUntriaged(bool),
    ClearAll,
    Validation(&'static str),
    Workflow(&'static str),
}

#[derive(Debug, Clone)]
struct PickerOption {
    label: String,
    detail: String,
    enabled: bool,
    action: PickerAction,
}

#[derive(Debug, Clone)]
pub(super) struct BoardPicker {
    pub(super) kind: PickerKind,
    pub(super) selected: usize,
    title: String,
    context: String,
    options: Vec<PickerOption>,
}

impl BoardPicker {
    fn select_first_enabled(&mut self) {
        self.selected = self
            .options
            .iter()
            .position(|option| option.enabled)
            .unwrap_or(0);
    }

    fn move_selection(&mut self, delta: isize) {
        if self.options.is_empty() {
            return;
        }
        self.selected =
            (self.selected as isize + delta).clamp(0, self.options.len() as isize - 1) as usize;
    }
}

fn path_suffix_label(path: &std::path::Path, width: u16) -> String {
    let value = path.display().to_string();
    let width = width as usize;
    if value.chars().count() <= width {
        return value;
    }
    let suffix = value
        .chars()
        .rev()
        .take(width.saturating_sub(1))
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("…{suffix}")
}

impl TuiApp {
    pub(super) fn start_workspace_picker(&mut self) {
        if self.text_input_active() || self.rules_prompt_active() || self.decision_prompt_active() {
            self.status = "Finish the current prompt or editor before switching workspaces.".into();
            return;
        }
        let feedback = self.workspace_feedback.take();
        self.populate_workspace_picker(feedback);
    }

    fn populate_workspace_picker(&mut self, feedback: Option<String>) {
        let current_root = self.workspace.root();
        let roster_entries = self
            .session_roster
            .lock()
            .expect("session roster lock")
            .entries()
            .to_vec();
        let mut entries = Vec::new();
        for entry in &roster_entries {
            let project = &entry.project;
            let current = project.root() == current_root;
            let summary = if let Some(message) = app::project::historical(project) {
                format!("Unavailable: {message}")
            } else {
                let mut errors = Vec::new();
                let docs = project.read_board_documents_tolerant(&mut errors);
                if !errors.is_empty() {
                    format!("Unavailable: {}", truncate(&errors.join("; "), 150))
                } else {
                    let attention = docs
                        .iter()
                        .filter(|doc| {
                            document_state_label(doc) == "validation"
                                || accord_status(doc) == Some("blocked")
                        })
                        .count();
                    let task_count = docs.iter().filter(|doc| is_task_doc(doc)).count();
                    format!(
                        "{}{} tasks · {} need attention",
                        if attention > 0 { "! " } else { "" },
                        task_count,
                        attention
                    )
                }
            };
            let valid = !summary.starts_with("Unavailable:");
            entries.push(WorkspacePickerEntry {
                alias: entry.alias.clone(),
                root: Some(project.root().to_path_buf()),
                summary,
                current,
                valid,
            });
        }
        let selected = entries.iter().position(|entry| entry.current).unwrap_or(0);
        let mut picker = WorkspacePicker {
            entries,
            filtered: Vec::new(),
            query: String::new(),
            selected,
            add_path: None,
            feedback,
        };
        picker.apply_filter();
        self.workspace_picker = Some(picker);
        self.status = "Session boards: type to search, Enter switch, Ctrl-A add path, Ctrl-D remove inactive, Esc close.".into();
    }

    pub(super) fn handle_workspace_picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.workspace_picker.as_mut() else {
            return;
        };
        if let Some(path) = picker.add_path.as_mut() {
            match key.code {
                KeyCode::Esc => picker.add_path = None,
                KeyCode::Enter => {
                    let path = std::path::PathBuf::from(path.trim());
                    picker.add_path = None;
                    let result = self
                        .session_roster
                        .lock()
                        .expect("session roster lock")
                        .add_path(&path);
                    match result {
                        Ok((entry, added)) => {
                            let message = if added {
                                format!("Added `{}` to this session.", entry.alias)
                            } else {
                                format!("Already in this session: `{}`.", entry.alias)
                            };
                            self.populate_workspace_picker(Some(message));
                        }
                        Err(error) => {
                            let message = format!("Cannot add board: {}", error.message);
                            self.populate_workspace_picker(Some(message));
                        }
                    }
                }
                KeyCode::Backspace => {
                    path.pop();
                }
                KeyCode::Char(ch)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    path.push(ch)
                }
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.workspace_picker = None;
                self.status = "Workspace switch canceled.".into();
            }
            KeyCode::Enter => {
                let target = picker
                    .filtered
                    .get(picker.selected)
                    .and_then(|index| picker.entries.get(*index))
                    .cloned();
                self.workspace_picker = None;
                if let Some(target) = target {
                    if target.valid {
                        self.switch_workspace(target);
                    } else {
                        let reason = target
                            .summary
                            .strip_prefix("Unavailable: ")
                            .unwrap_or(&target.summary);
                        self.status = format!("Workspace `{}` unavailable: {reason}", target.alias);
                    }
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                picker.add_path = Some(String::new())
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let target = picker
                    .filtered
                    .get(picker.selected)
                    .and_then(|index| picker.entries.get(*index))
                    .cloned();
                if let Some(target) = target {
                    if target.current {
                        self.populate_workspace_picker(Some(
                            "Cannot remove the current board; switch to another board first."
                                .into(),
                        ));
                    } else if let Some(root) = target.root {
                        self.session_roster
                            .lock()
                            .expect("session roster lock")
                            .remove(&root, self.workspace.root());
                        self.populate_workspace_picker(Some(format!(
                            "Removed `{}` from this session.",
                            target.alias
                        )));
                    }
                }
            }
            KeyCode::Up => picker.selected = picker.selected.saturating_sub(1),
            KeyCode::Down => {
                picker.selected = (picker.selected + 1).min(picker.filtered.len().saturating_sub(1))
            }
            KeyCode::Backspace => {
                picker.query.pop();
                picker.apply_filter();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                picker.query.clear();
                picker.apply_filter();
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                picker.query.push(ch);
                picker.apply_filter();
            }
            _ => {}
        }
    }

    fn switch_workspace(&mut self, target: WorkspacePickerEntry) {
        if target.current {
            self.status = format!("Already in workspace `{}`.", target.alias);
            return;
        }
        let Some(root) = target.root.as_deref() else {
            self.status = target.summary;
            return;
        };
        let candidate_workspace = {
            let roster = self.session_roster.lock().expect("session roster lock");
            let Some(board) = roster
                .entries()
                .iter()
                .find(|entry| entry.project.root() == root)
            else {
                self.status = format!("Session board `{}` is no longer available.", target.alias);
                return;
            };
            board.project.clone()
        };
        if let Some(message) = app::project::historical(&candidate_workspace) {
            self.status = format!(
                "Cannot switch to `{}`: cannot open the TUI: {message}",
                target.alias
            );
            return;
        }
        let mut candidate =
            match TuiApp::load_with(candidate_workspace, self.session_roster.clone()) {
                Ok(candidate) => candidate,
                Err(error) => {
                    self.status = format!(
                        "Cannot load workspace `{}`: {}",
                        target.alias, error.message
                    );
                    return;
                }
            };
        let diagnostics = candidate.load_errors.clone();
        if !diagnostics.is_empty() {
            self.status = format!(
                "Cannot switch to `{}`: {}",
                target.alias,
                truncate(&diagnostics.join("; "), 220)
            );
            return;
        }
        candidate.workspace_alias = target.alias.clone();
        candidate.status = format!(
            "Switched to workspace `{}` · {} tasks",
            target.alias,
            candidate.docs.iter().filter(|doc| is_task_doc(doc)).count()
        );
        *self = candidate;
    }

    pub(super) fn start_filter_picker(&mut self) {
        let mut options = Vec::new();
        let kinds = board_filter_kinds(&self.docs);
        if kinds.is_empty() {
            options.push(PickerOption {
                label: "Kind filter".into(),
                detail: "Unavailable: no kinds on active Board items".into(),
                enabled: false,
                action: PickerAction::SetKind(None),
            });
        } else {
            for kind in kinds {
                options.push(PickerOption {
                    label: format!("Kind  {kind}"),
                    detail: if self.board_filters.kind.as_deref() == Some(kind.as_str()) {
                        "Current filter".into()
                    } else {
                        "Show tasks of this kind".into()
                    },
                    enabled: self.board_filters.kind.as_deref() != Some(kind.as_str()),
                    action: PickerAction::SetKind(Some(kind)),
                });
            }
        }
        let tags = board_filter_tags(&self.docs);
        if tags.is_empty() {
            options.push(PickerOption {
                label: "Tag filter".into(),
                detail: "Unavailable: no tags on active Board items".into(),
                enabled: false,
                action: PickerAction::SetTag(None),
            });
        } else {
            for tag in tags {
                options.push(PickerOption {
                    label: format!("Tag  #{tag}"),
                    detail: if self.board_filters.tag.as_deref() == Some(tag.as_str()) {
                        "Current filter".into()
                    } else {
                        "Show tasks with this tag".into()
                    },
                    enabled: self.board_filters.tag.as_deref() != Some(tag.as_str()),
                    action: PickerAction::SetTag(Some(tag)),
                });
            }
        }
        let priorities = board_filter_priorities(&self.docs);
        if priorities.is_empty() {
            options.push(PickerOption {
                label: "Priority filter".into(),
                detail: "Unavailable: no priorities on active Board items".into(),
                enabled: false,
                action: PickerAction::SetPriority(None),
            });
        } else {
            for priority in priorities {
                options.push(PickerOption {
                    label: format!("Priority  {priority}"),
                    detail: if self.board_filters.priority.as_deref() == Some(priority.as_str()) {
                        "Current filter".into()
                    } else {
                        "Show tasks with this priority".into()
                    },
                    enabled: self.board_filters.priority.as_deref() != Some(priority.as_str()),
                    action: PickerAction::SetPriority(Some(priority)),
                });
            }
        }
        options.push(PickerOption {
            label: "Delivered · untriaged".into(),
            detail: if self.board_filters.delivered_untriaged {
                "Current filter".into()
            } else {
                "Show in-progress work with delivered accord".into()
            },
            enabled: !self.board_filters.delivered_untriaged,
            action: PickerAction::SetDeliveredUntriaged(true),
        });
        options.extend([
            PickerOption {
                label: "Clear kind".into(),
                detail: "Remove only the kind filter".into(),
                enabled: self.board_filters.kind.is_some(),
                action: PickerAction::SetKind(None),
            },
            PickerOption {
                label: "Clear tag".into(),
                detail: "Remove only the tag filter".into(),
                enabled: self.board_filters.tag.is_some(),
                action: PickerAction::SetTag(None),
            },
            PickerOption {
                label: "Clear priority".into(),
                detail: "Remove only the priority filter".into(),
                enabled: self.board_filters.priority.is_some(),
                action: PickerAction::SetPriority(None),
            },
            PickerOption {
                label: "Clear delivered · untriaged".into(),
                detail: "Remove only the delivered-work filter".into(),
                enabled: self.board_filters.delivered_untriaged,
                action: PickerAction::SetDeliveredUntriaged(false),
            },
            PickerOption {
                label: "Clear all filters".into(),
                detail: "Show the complete Board".into(),
                enabled: self.board_filters.is_active(),
                action: PickerAction::ClearAll,
            },
        ]);
        let mut picker = BoardPicker {
            kind: PickerKind::Filter,
            selected: 0,
            title: "Filter Board".into(),
            context: format!("Current: {}", self.board_filters.summary()),
            options,
        };
        picker.select_first_enabled();
        self.board_picker = Some(picker);
        self.status =
            "Filter picker: choose an available filter; Enter applies, Esc cancels.".into();
    }

    pub(super) fn start_workflow_picker(&mut self) {
        let Some(doc) = self.selected_doc() else {
            self.status = "No selected Board task for workflow action.".into();
            return;
        };
        let id = doc.id().to_string();
        let title = doc.title().to_string();
        let status = accord_status(doc).unwrap_or("missing").to_string();
        let task = is_task_doc(doc);
        let mut options = Vec::new();
        for (action, label, detail) in [
            (
                "claim",
                "Claim accord",
                "Set the assignee and move todo work in-progress",
            ),
            (
                "deliver",
                "Deliver accord",
                "Record a summary and required evidence",
            ),
            ("block", "Block accord", "Record why work cannot proceed"),
            (
                "resume",
                "Resume accord",
                "Clear the blocked state and continue claimed work",
            ),
        ] {
            let transition = accord::validate_transition(action, &status);
            let enabled = task && transition.is_ok();
            let detail = if !task {
                "Unavailable: only task documents have Accord actions".to_string()
            } else if let Err(error) = transition {
                error
            } else {
                detail.to_string()
            };
            options.push(PickerOption {
                label: label.to_string(),
                detail,
                enabled,
                action: PickerAction::Workflow(action),
            });
        }
        options.push(PickerOption {
            label: "Complete and archive".into(),
            detail: if task {
                "Run native completion checks and archive the Task".into()
            } else {
                "Unavailable: only task documents can be completed".into()
            },
            enabled: task,
            action: PickerAction::Workflow("complete"),
        });
        self.board_picker = Some(BoardPicker {
            kind: PickerKind::Actions,
            selected: 0,
            title: "Task actions".into(),
            context: format!(
                "{id} — {title} · state {} · accord {status}",
                display_state_label(&document_state_label(doc))
            ),
            options,
        });
        if let Some(picker) = self.board_picker.as_mut() {
            picker.select_first_enabled();
        }
        self.status =
            "Task actions: choose a native action; Enter opens required input, Esc cancels.".into();
    }

    pub(super) fn start_validation_picker(&mut self) {
        let selected = self.selected_doc().map(|doc| {
            (
                doc.id().to_string(),
                doc.title().to_string(),
                document_state_label(doc),
                accord_status(doc).unwrap_or("missing").to_string(),
            )
        });
        let (context, delivered, active_task) = match selected.as_ref() {
            Some((id, title, state, status)) => (
                format!(
                    "{id} — {title} · state {} · accord {status}",
                    display_state_label(state)
                ),
                state == "validation" && normalized_accord_status(status) == "delivered",
                state != "validation" && status != "missing",
            ),
            None => ("No selected Board task".into(), false, false),
        };
        let reason = match selected.as_ref() {
            None => "Disabled: no selected task".to_string(),
            Some((_, _, state, _)) if state != "validation" => {
                "Disabled: validation actions apply to delivered work".into()
            }
            Some((_, _, _, status)) if normalized_accord_status(status) != "delivered" => {
                format!("Disabled: accord {status}")
            }
            _ => "Available for delivered Validation work".into(),
        };
        let mut picker = BoardPicker {
            kind: PickerKind::Validation,
            selected: 0,
            title: "Validation actions".into(),
            context,
            options: if active_task {
                vec![PickerOption {
                    label: "Accept and archive".into(),
                    detail: reason,
                    enabled: false,
                    action: PickerAction::Validation("accept"),
                }]
            } else {
                vec![
                    PickerOption {
                        label: "Accept and archive".into(),
                        detail: reason.clone(),
                        enabled: delivered,
                        action: PickerAction::Validation("accept"),
                    },
                    PickerOption {
                        label: "Request changes".into(),
                        detail: reason,
                        enabled: delivered,
                        action: PickerAction::Validation("changes"),
                    },
                ]
            },
        };
        picker.select_first_enabled();
        self.board_picker = Some(picker);
        self.status = "Validation picker: choose an available action; Enter opens its next step, Esc cancels.".into();
    }

    pub(super) fn handle_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.board_picker = None;
                self.status = "Board action canceled.".into();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(p) = self.board_picker.as_mut() {
                    p.move_selection(-1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(p) = self.board_picker.as_mut() {
                    p.move_selection(1);
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if let Some(p) = self.board_picker.as_mut() {
                    p.selected = 0;
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                if let Some(p) = self.board_picker.as_mut() {
                    p.selected = p.options.len().saturating_sub(1);
                }
            }
            KeyCode::PageUp | KeyCode::Char('u')
                if key.code == KeyCode::PageUp || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if let Some(p) = self.board_picker.as_mut() {
                    p.move_selection(-5);
                }
            }
            KeyCode::PageDown | KeyCode::Char('d')
                if key.code == KeyCode::PageDown
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if let Some(p) = self.board_picker.as_mut() {
                    p.move_selection(5);
                }
            }
            KeyCode::Enter => self.activate_picker_selection(),
            _ => {}
        }
    }

    pub(super) fn select_picker_option(&mut self, index: usize) {
        if let Some(p) = self.board_picker.as_mut() {
            p.selected = index.min(p.options.len().saturating_sub(1));
        }
    }

    pub(super) fn activate_picker_selection(&mut self) {
        let Some(option) = self
            .board_picker
            .as_ref()
            .and_then(|p| p.options.get(p.selected))
            .cloned()
        else {
            return;
        };
        if !option.enabled {
            self.status = option.detail;
            return;
        }
        let selected_id = self.selected_doc().map(|doc| doc.id().to_string());
        self.board_picker = None;
        match option.action {
            PickerAction::SetKind(value) => {
                self.board_filters.kind = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::SetTag(value) => {
                self.board_filters.tag = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::SetPriority(value) => {
                self.board_filters.priority = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::SetDeliveredUntriaged(value) => {
                self.board_filters.delivered_untriaged = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::ClearAll => {
                self.board_filters = BoardFilters::default();
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::Validation(action) => self.show_validation_action_hint(action),
            PickerAction::Workflow(action) => self.start_workflow_action(action),
        }
    }

    fn restore_filtered_selection(&mut self, id: Option<&str>) {
        if !id.is_some_and(|id| self.select_document_by_id_preserving_scroll(id)) {
            self.clamp_selection();
        }
        self.status = format!("Board {}.", self.board_filters.summary());
    }

    pub(super) fn draw_workspace_picker(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let Some(picker) = self.workspace_picker.as_ref() else {
            return;
        };
        let popup = centered_rect(86, 72, area);
        frame.render_widget(Clear, popup);
        let inner = Block::default()
            .borders(Borders::ALL)
            .title(" Session boards ")
            .border_style(self.theme.border_style(true))
            .style(self.theme.panel_style())
            .inner(popup);
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(" Session boards ")
                .border_style(self.theme.border_style(true))
                .style(self.theme.panel_style()),
            popup,
        );
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        frame.render_widget(
            Paragraph::new(if let Some(path) = &picker.add_path {
                format!("Add board path: {path}_")
            } else {
                format!(
                    "Search: {}",
                    if picker.query.is_empty() {
                        "<type to filter>"
                    } else {
                        &picker.query
                    }
                )
            })
            .style(self.theme.text_style()),
            rows[0],
        );
        let selected_path = picker
            .filtered
            .get(picker.selected)
            .and_then(|index| picker.entries.get(*index))
            .and_then(|entry| entry.root.as_deref())
            .map(|path| path_suffix_label(path, rows[1].width))
            .unwrap_or_else(|| "<no selected board>".into());
        frame.render_widget(
            Paragraph::new(format!("Selected: {selected_path}")).style(self.theme.muted_style()),
            rows[1],
        );
        frame.render_widget(
            Paragraph::new(
                picker
                    .feedback
                    .as_deref()
                    .map(|message| truncate(message, rows[2].width as usize))
                    .unwrap_or_default(),
            )
            .style(
                picker
                    .feedback
                    .as_deref()
                    .map(|message| {
                        self.theme
                            .status_style(super::chrome::status_tone_for_message(message))
                    })
                    .unwrap_or_else(|| self.theme.muted_style()),
            ),
            rows[2],
        );
        let items = picker
            .filtered
            .iter()
            .enumerate()
            .filter_map(|(visible, index)| {
                picker.entries.get(*index).map(|entry| {
                    let marker = if entry.current { "●" } else { " " };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            if visible == picker.selected {
                                "▸ "
                            } else {
                                "  "
                            },
                            self.theme.status_style(StatusTone::Accent),
                        ),
                        Span::styled(
                            format!("{marker} {:<18}", truncate(&entry.alias, 18)),
                            if entry.current {
                                self.theme.status_style(StatusTone::Accent)
                            } else if entry.valid {
                                self.theme.text_style()
                            } else {
                                self.theme.status_style(StatusTone::Warning)
                            },
                        ),
                        Span::styled(
                            truncate(&entry.summary, rows[1].width.saturating_sub(28) as usize),
                            self.theme.muted_style(),
                        ),
                    ]))
                })
            })
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(Some(picker.selected));
        if items.is_empty() {
            frame.render_widget(
                Paragraph::new("No matching session boards").style(self.theme.muted_style()),
                rows[3],
            );
        } else {
            frame.render_stateful_widget(
                List::new(items).highlight_style(self.theme.selected_style()),
                rows[3],
                &mut state,
            );
        }
        frame.render_widget(
            Paragraph::new(
                "↑/↓ select · Enter switch · Ctrl-A add · Ctrl-D remove inactive · Esc close",
            )
            .style(self.theme.muted_style()),
            rows[4],
        );
    }

    pub(super) fn draw_board_picker(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let Some(picker) = self.board_picker.as_ref() else {
            return;
        };
        let popup = centered_rect(78, 68, area);
        frame.render_widget(Clear, popup);
        let inner = Block::default().borders(Borders::ALL).inner(popup);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    picker.context.clone(),
                    self.theme.muted_style(),
                )),
                Line::from(Span::styled(
                    "Available actions are bright; unavailable actions include a reason.",
                    self.theme.muted_style(),
                )),
            ]),
            rows[0],
        );
        let label_width: usize = if rows[1].width < 72 { 19 } else { 28 };
        let detail_width = rows[1].width.saturating_sub(label_width as u16 + 3) as usize;
        let items = picker
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        if index == picker.selected {
                            "▸ "
                        } else {
                            "  "
                        },
                        self.theme.status_style(StatusTone::Accent),
                    ),
                    Span::styled(
                        format!(
                            "{:<width$}",
                            truncate(&option.label, label_width),
                            width = label_width
                        ),
                        if option.enabled {
                            self.theme.text_style()
                        } else {
                            self.theme.muted_style()
                        },
                    ),
                    Span::styled(
                        truncate(&option.detail, detail_width),
                        self.theme.muted_style(),
                    ),
                ]))
            })
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(Some(picker.selected));
        frame.render_stateful_widget(
            List::new(items).highlight_style(self.theme.selected_style()),
            rows[1],
            &mut state,
        );
        let controls = if rows[2].width < 64 {
            "[ Apply ]  [ Cancel ]  [ Help ]"
        } else {
            "[ Apply selected ]    [ Cancel ]    [ Key reference ]"
        };
        frame.render_widget(
            Paragraph::new(controls).style(self.theme.muted_style()),
            rows[2],
        );
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", picker.title))
                .border_style(self.theme.border_style(true))
                .style(self.theme.panel_style()),
            popup,
        );
        let offset = state.offset();
        for index in offset
            ..picker
                .options
                .len()
                .min(offset.saturating_add(rows[1].height as usize))
        {
            self.hits.push(HitRegion {
                rect: Rect::new(
                    rows[1].x,
                    rows[1].y + index.saturating_sub(offset) as u16,
                    rows[1].width,
                    1,
                ),
                action: HitAction::SelectPickerOption(index),
            });
        }
        self.register_picker_control_hit(
            rows[2],
            controls,
            if rows[2].width < 64 {
                "[ Apply ]"
            } else {
                "[ Apply selected ]"
            },
            HitAction::ActivatePicker,
        );
        self.register_picker_control_hit(rows[2], controls, "[ Cancel ]", HitAction::CancelPicker);
        self.register_picker_control_hit(
            rows[2],
            controls,
            if rows[2].width < 64 {
                "[ Help ]"
            } else {
                "[ Key reference ]"
            },
            HitAction::ShowHelp,
        );
    }

    fn register_picker_control_hit(
        &mut self,
        area: Rect,
        text: &str,
        label: &str,
        action: HitAction,
    ) {
        let Some(start) = text.find(label) else {
            return;
        };
        let x = area.x.saturating_add(start as u16);
        self.hits.push(HitRegion {
            rect: Rect::new(
                x,
                area.y,
                (label.chars().count() as u16).min(area.right().saturating_sub(x)),
                1,
            ),
            action,
        });
    }
}
