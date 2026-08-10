//! Transient TUI state transitions, selection, navigation, and shared view projections.
//!
//! Durable mutations remain in `app`; this module owns only aggregate UI behavior.

use super::*;

impl TuiApp {
    pub(super) fn switch_view(&mut self, view: TuiView) {
        self.view = view;
        self.focus = FocusPane::Board;
        if view == TuiView::Logs {
            self.clamp_selection();
        }
        if view == TuiView::Rules {
            self.clamp_rules_state();
        }
        if view == TuiView::Decisions {
            self.clamp_decisions_state();
        }
        self.status = match view {
            TuiView::Board => {
                "Board view active. Use b for State/Epic Board, h/l for states, j/k for rows, and f/m/v for Board actions.".to_string()
            }
            TuiView::Logs => self.logs_status_message(),
            TuiView::Rules => format!(
                "Rules view active: {} project rule{} loaded. Use j/k select, h/l category, a/e/d add/edit/delete; Tab has no top-level fallback.",
                self.rules_total(),
                if self.rules_total() == 1 { "" } else { "s" }
            ),
            TuiView::Decisions => format!(
                "Decisions view active: {} decision{} loaded. Use j/k select, h/l or Tab for list/body focus, a add.",
                self.decision_docs().len(),
                if self.decision_docs().len() == 1 {
                    ""
                } else {
                    "s"
                }
            ),
        };
    }

    pub(super) fn open_help(&mut self) {
        self.show_help = true;
        self.help_scroll = 0;
        self.help_section = match self.view {
            TuiView::Board => 3,
            TuiView::Logs => 5,
            TuiView::Rules => 6,
            TuiView::Decisions => 7,
        };
        if self.papercuts_open() {
            self.help_section = 8;
        }
        if let Some(picker) = self.board_picker.as_ref() {
            self.help_section = if picker.kind == pickers::PickerKind::Validation {
                4
            } else {
                3
            };
        }
    }

    pub(super) fn select_help_section(&mut self, delta: isize) {
        self.help_section = (self.help_section as isize + delta)
            .clamp(0, BindingScope::ALL.len().saturating_sub(1) as isize)
            as usize;
        self.help_scroll = 0;
    }

    pub(super) fn focus_next(&mut self) {
        match self.view {
            TuiView::Board if self.show_board_detail => self.focus = FocusPane::Detail,
            TuiView::Logs | TuiView::Decisions => self.focus = FocusPane::Detail,
            TuiView::Rules => self.focus_rule_preview(),
            _ => {}
        }
    }

    pub(super) fn focus_previous(&mut self) {
        match self.view {
            TuiView::Board | TuiView::Logs | TuiView::Decisions => self.focus = FocusPane::Board,
            TuiView::Rules => self.focus_rule_list(),
        }
    }

    pub(super) fn activate_logs_selection(&mut self) {
        self.focus = FocusPane::Detail;
    }

    pub(super) fn focus_previous_pane(&mut self) {
        if matches!(self.view, TuiView::Logs | TuiView::Decisions) {
            self.focus = FocusPane::Board;
        }
    }

    pub(super) fn focus_next_pane(&mut self) {
        if matches!(self.view, TuiView::Logs | TuiView::Decisions) {
            self.focus = FocusPane::Detail;
        }
    }

    pub(super) fn handle_board_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.previous_state(),
            KeyCode::Right | KeyCode::Char('l') => self.next_state(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_item(),
            KeyCode::Home | KeyCode::Char('g') => self.selected_item = 0,
            KeyCode::End | KeyCode::Char('G') => self.last_item(),
            KeyCode::PageUp => self.selected_item = self.selected_item.saturating_sub(5),
            KeyCode::PageDown => {
                self.selected_item =
                    (self.selected_item + 5).min(self.selected_state_count().saturating_sub(1))
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected_item = self.selected_item.saturating_sub(5)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected_item =
                    (self.selected_item + 5).min(self.selected_state_count().saturating_sub(1))
            }
            _ => {}
        }
    }

    pub(super) fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.scroll_detail_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_detail_down(1),
            KeyCode::PageUp => self.scroll_detail_up(6),
            KeyCode::PageDown => self.scroll_detail_down(6),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_detail_up(6)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_detail_down(6)
            }
            KeyCode::Home | KeyCode::Char('g') => self.detail_scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.detail_scroll_to_end(),
            KeyCode::Left | KeyCode::Char('h') => self.previous_state(),
            KeyCode::Right | KeyCode::Char('l') => self.next_state(),
            _ => {}
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn handle_review_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.previous_review_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_review_item(),
            KeyCode::Home | KeyCode::Char('g') => self.selected_review_item = 0,
            KeyCode::End | KeyCode::Char('G') => self.last_review_item(),
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
        self.clamp_review_selection();
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn handle_review_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.scroll_review_detail_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_review_detail_down(1),
            KeyCode::PageUp | KeyCode::Char('u') => self.scroll_review_detail_up(6),
            KeyCode::PageDown | KeyCode::Char('d') => self.scroll_review_detail_down(6),
            KeyCode::Home | KeyCode::Char('g') => self.review_detail_scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.review_detail_scroll_to_end(),
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
    }

    pub(super) fn handle_logs_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                FocusPane::Board => self.previous_log(),
                FocusPane::Detail => self.scroll_log_detail_up(1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                FocusPane::Board => self.next_log(),
                FocusPane::Detail => self.scroll_log_detail_down(1),
            },
            KeyCode::PageUp => match self.focus {
                FocusPane::Board => self.previous_log_page(),
                FocusPane::Detail => self.scroll_log_detail_up(6),
            },
            KeyCode::PageDown => match self.focus {
                FocusPane::Board => self.next_log_page(),
                FocusPane::Detail => self.scroll_log_detail_down(6),
            },
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => match self.focus
            {
                FocusPane::Board => self.previous_log_page(),
                FocusPane::Detail => self.scroll_log_detail_up(6),
            },
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => match self.focus
            {
                FocusPane::Board => self.next_log_page(),
                FocusPane::Detail => self.scroll_log_detail_down(6),
            },
            KeyCode::Home | KeyCode::Char('g') => match self.focus {
                FocusPane::Board => {
                    self.selected_log = 0;
                    self.log_detail_scroll = 0;
                }
                FocusPane::Detail => self.log_detail_scroll = 0,
            },
            KeyCode::End | KeyCode::Char('G') => match self.focus {
                FocusPane::Board => self.last_log(),
                FocusPane::Detail => self.log_detail_scroll_to_end(),
            },
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
    }

    pub(super) fn handle_log_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.log_search_input = None;
                if self.log_search_filter.is_empty() {
                    self.status = "Log search canceled.".to_string();
                } else {
                    self.status = self.logs_status_message();
                }
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => self.finish_log_search(),
            KeyCode::Char('m') | KeyCode::Char('j')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.finish_log_search()
            }
            KeyCode::Backspace => {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.pop();
                }
                self.refresh_log_search_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.clear();
                }
                self.refresh_log_search_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.push(ch);
                }
                self.refresh_log_search_status();
            }
            _ => {}
        }
    }

    pub(super) fn handle_quick_add_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.quick_add = None;
                self.status = "Quick add canceled.".to_string();
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => self.finish_quick_add(),
            KeyCode::Char('m') | KeyCode::Char('j')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.finish_quick_add()
            }
            KeyCode::Backspace => {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.pop();
                }
                self.refresh_quick_add_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.clear();
                }
                self.refresh_quick_add_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.push(ch);
                }
                self.refresh_quick_add_status();
            }
            _ => {}
        }
    }

    pub(super) fn start_quick_add(&mut self) {
        if !self.hierarchy.errors.is_empty() {
            self.status =
                "Quick add disabled: fix the persistent Board hierarchy errors and reload first."
                    .to_string();
            return;
        }
        let (state, fallback_note) = quick_add_state_for_selection(
            &self.configured_states,
            &self.states,
            self.selected_state,
        );
        self.quick_add = Some(QuickAddInput {
            state,
            title: String::new(),
            fallback_note,
        });
        self.focus = FocusPane::Board;
        self.refresh_quick_add_status();
    }

    fn refresh_quick_add_status(&mut self) {
        if let Some(input) = self.quick_add.as_ref() {
            self.status = quick_add_status(input);
        }
    }

    fn finish_quick_add(&mut self) {
        let Some(input) = self.quick_add.as_ref() else {
            return;
        };
        let title = input.title.trim().to_string();
        if title.is_empty() {
            self.status = format!(
                "Quick add needs a title. Add task in {}: type title, Enter create, Esc cancel",
                input.state
            );
            return;
        }
        let state = input.state.clone();
        self.quick_add = None;

        match app::tasks::add(
            &self.workspace,
            AddOptions {
                title: Some(title.clone()),
                state: Some(state.clone()),
                ..AddOptions::default()
            },
        ) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = format!(
                    "Created {} in {}: {}{}",
                    outcome.id, outcome.state, outcome.title, reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Add error: {}{}", error.message, reload_note);
            }
        }
    }

    pub(super) fn start_log_search(&mut self) {
        self.log_search_input = Some(self.log_search_filter.clone());
        self.focus = FocusPane::Board;
        self.refresh_log_search_status();
    }

    fn refresh_log_search_status(&mut self) {
        let query = self.log_search_input.as_deref().unwrap_or("");
        self.status = format!(
            "Search logs: {} · type filter, Enter apply, Esc cancel",
            if query.is_empty() { "<query>" } else { query }
        );
    }

    fn finish_log_search(&mut self) {
        let query = self
            .log_search_input
            .take()
            .unwrap_or_default()
            .trim()
            .to_string();
        self.log_search_filter = query;
        self.selected_log = 0;
        self.log_detail_scroll = 0;
        self.clamp_selection();
        self.status = self.logs_status_message();
    }

    pub(super) fn clear_log_filter_or_focus(&mut self) {
        if !self.log_search_filter.is_empty() {
            self.log_search_filter.clear();
            self.selected_log = 0;
            self.log_detail_scroll = 0;
            self.status = "Cleared Logs search filter.".to_string();
            self.clamp_selection();
        } else if self.focus == FocusPane::Detail {
            self.focus = FocusPane::Board;
        }
    }

    pub(super) fn move_selected_task_to_state(&mut self, doc_id: &str, target_state: &str) {
        match app::tasks::move_to_state(&self.workspace, doc_id, target_state) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = if outcome.changed {
                    format!(
                        "Moved {}: {} -> {}{}{}",
                        outcome.id,
                        outcome.from,
                        outcome.to,
                        outcome
                            .accord_sync
                            .as_deref()
                            .map(|sync| format!("; accord {sync}"))
                            .unwrap_or_default(),
                        reload_note
                    )
                } else {
                    format!(
                        "{} is already in state {}{}",
                        outcome.id, outcome.to, reload_note
                    )
                };
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(doc_id);
                self.status = format!("Move error: {}{}", error.message, reload_note);
            }
        }
    }

    pub(super) fn open_selected_item_in_editor(
        &mut self,
        session: &mut TerminalSession,
    ) -> Result<(), CliError> {
        let target = match self.selected_editor_target() {
            Ok(target) => target,
            Err(message) => {
                self.status = message;
                return Ok(());
            }
        };
        let editor = match editor_command_from_env() {
            Ok(editor) => editor,
            Err(error) => {
                self.status = format!("Editor error: {}", error.message);
                return Ok(());
            }
        };

        self.status = format!(
            "Opening {} in {} from {}...",
            target.id,
            editor.display_label(),
            editor.source
        );
        session.terminal_mut().draw(|frame| self.draw(frame))?;

        session.suspend_for_editor()?;
        let editor_result = run_editor_command(&editor, &target.path);
        let resume_result = session.resume_after_editor();
        if let Err(error) = resume_result {
            return Err(CliError::user(format!(
                "failed to restore terminal after editor exit: {}",
                error.message
            )));
        }

        let reload_note = self.reload().warning_note();
        let selection_note = if self.select_document_by_id(&target.id) {
            String::new()
        } else {
            "; edited item is not currently loadable or no longer active".to_string()
        };
        let reload_note = format!("{reload_note}{selection_note}");

        self.status = match editor_result {
            Ok(status) if status.success() => format!(
                "Edited {} via {}{}",
                target.id,
                editor.display_label(),
                reload_note
            ),
            Ok(status) => format!(
                "Editor exited with {status} for {}{}",
                target.id, reload_note
            ),
            Err(error) => format!(
                "Editor launch failed for {} using {}: {error}{}",
                target.id,
                editor.display_label(),
                reload_note
            ),
        };
        Ok(())
    }

    pub(super) fn selected_editor_target(&self) -> Result<EditorTarget, String> {
        match self.view {
            TuiView::Board => self
                .selected_doc()
                .ok_or_else(|| "No active task selected to edit.".to_string())
                .and_then(editor_target_for_doc),
            TuiView::Logs => Err("Completed logs are read-only in the TUI; $EDITOR is intentionally disabled for generated history.".to_string()),
            TuiView::Rules => Err("Rules use the in-TUI a/e/d prompts; raw config-file editing is deferred.".to_string()),
            TuiView::Decisions => Err("Decision document editing in $EDITOR is deferred; active task documents are supported first.".to_string()),
        }
    }

    pub(super) fn select_document_by_id(&mut self, id: &str) -> bool {
        self.select_document_by_id_with_scroll(id, true)
    }

    pub(super) fn select_document_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        self.select_document_by_id_with_scroll(id, false)
    }

    fn select_document_by_id_with_scroll(&mut self, id: &str, reset_scroll: bool) -> bool {
        if self.board_arrangement == BoardArrangement::Epic {
            let epic_index = self
                .epic_board_entries()
                .iter()
                .position(|entry| entry.doc.id() == id);
            if let Some(index) = epic_index {
                self.selected_item = index;
                if reset_scroll {
                    self.detail_scroll = 0;
                }
                self.clamp_selection();
                return true;
            }
        }

        self.expand_active_task_ancestors(id);
        for state_index in 0..self.states.len() {
            let Some(state_name) = self.states.get(state_index) else {
                continue;
            };
            if let Some(item_index) = self
                .state_board_entries(state_name)
                .iter()
                .position(|entry| entry.doc.id() == id)
            {
                self.selected_state = state_index;
                self.selected_item = item_index;
                if reset_scroll {
                    self.detail_scroll = 0;
                }
                self.clamp_selection();
                return true;
            }
        }
        self.clamp_selection();
        false
    }

    pub(super) fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Board => FocusPane::Detail,
            FocusPane::Detail => FocusPane::Board,
        };
    }

    pub(super) fn toggle_board_detail(&mut self) {
        self.show_board_detail = !self.show_board_detail;
        self.focus = if self.show_board_detail {
            FocusPane::Detail
        } else {
            FocusPane::Board
        };
        self.status = if self.show_board_detail {
            "Board detail pane shown; Tab or Esc returns to the list.".to_string()
        } else {
            "Board detail pane hidden; Space toggles the selected row preview.".to_string()
        };
    }

    pub(super) fn toggle_board_arrangement(&mut self) {
        let selected_id = self.selected_doc().map(|doc| doc.id().to_string());
        self.board_arrangement = self.board_arrangement.toggled();
        self.selected_item = 0;
        self.detail_scroll = 0;
        if let Some(id) = selected_id.as_deref() {
            self.select_document_by_id_preserving_scroll(id);
        } else {
            self.clamp_selection();
        }
        self.status = format!(
            "Board arrangement: {}. Press b to switch State/Epic Board.",
            self.board_arrangement.label()
        );
    }

    pub(super) fn toggle_board_expansion(&mut self) {
        let Some((doc_id, role)) = self.selected_doc().map(|doc| {
            (
                doc.id().to_string(),
                self.hierarchy
                    .index
                    .as_ref()
                    .and_then(|hierarchy| hierarchy.task_role(doc).ok().flatten()),
            )
        }) else {
            self.status = "No selected Board item to expand or preview.".to_string();
            return;
        };
        if self.board_arrangement == BoardArrangement::Epic {
            self.toggle_board_preview();
            self.status = if self.expanded_board_doc_id.as_deref() == Some(doc_id.as_str()) {
                format!("Previewing {doc_id} inline; press Enter to close.")
            } else {
                format!("Closed preview for {doc_id}.")
            };
            return;
        }
        let has_active_descendants = role.is_some()
            && count_task_descendants(
                &doc_id,
                &self.docs,
                &self.logs,
                &mut BTreeSet::from([doc_id.clone()]),
            )
            .0 > 0;
        if has_active_descendants {
            let children = match role {
                Some(TaskRole::Epic) => "Tasks",
                Some(TaskRole::Task) => "Subtasks",
                _ => "children",
            };
            self.expanded_board_doc_id = None;
            if self.expanded_board_hierarchy_ids.remove(&doc_id) {
                self.status = format!("Collapsed {children} under {doc_id}.");
            } else {
                self.expanded_board_hierarchy_ids.insert(doc_id.clone());
                self.status =
                    format!("Expanded {children} under {doc_id}; press Enter to collapse.");
            }
            self.clamp_selection();
        } else {
            self.toggle_board_preview();
        }
    }

    pub(super) fn toggle_board_preview(&mut self) {
        let Some(doc_id) = self.selected_doc().map(|doc| doc.id().to_string()) else {
            self.status = "No selected Board item to preview.".to_string();
            return;
        };
        if self.expanded_board_doc_id.as_deref() == Some(doc_id.as_str()) {
            self.expanded_board_doc_id = None;
            self.status = format!("Closed preview for {doc_id}.");
        } else {
            self.expanded_board_doc_id = Some(doc_id.clone());
            self.status = format!("Previewing {doc_id} inline; press Space to close.");
        }
    }

    fn expand_active_task_ancestors(&mut self, id: &str) {
        let mut current = id.to_string();
        let mut visited = BTreeSet::from([current.clone()]);
        while let Some(parent_id) = self
            .docs
            .iter()
            .find(|doc| doc.id() == current)
            .and_then(normalized_parent_id)
        {
            if !visited.insert(parent_id.clone())
                || !self
                    .docs
                    .iter()
                    .any(|doc| doc.id() == parent_id && is_task_doc(doc))
            {
                break;
            }
            self.expanded_board_hierarchy_ids.insert(parent_id.clone());
            current = parent_id;
        }
    }

    fn previous_state(&mut self) {
        if self.board_arrangement == BoardArrangement::Epic {
            self.status =
                "Epic Board groups all workflow states; press b for State Board tabs.".to_string();
            return;
        }
        if self.selected_state > 0 {
            self.selected_state -= 1;
            self.selected_item = 0;
            self.detail_scroll = 0;
        }
        self.clamp_selection();
    }

    fn next_state(&mut self) {
        if self.board_arrangement == BoardArrangement::Epic {
            self.status =
                "Epic Board groups all workflow states; press b for State Board tabs.".to_string();
            return;
        }
        if self.selected_state + 1 < self.states.len() {
            self.selected_state += 1;
            self.selected_item = 0;
            self.detail_scroll = 0;
        }
        self.clamp_selection();
    }

    pub(super) fn previous_item(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.detail_scroll = 0;
        }
    }

    pub(super) fn next_item(&mut self) {
        let count = self.selected_state_count();
        if self.selected_item + 1 < count {
            self.selected_item += 1;
            self.detail_scroll = 0;
        }
    }

    fn last_item(&mut self) {
        let count = self.selected_state_count();
        if count > 0 {
            self.selected_item = count - 1;
            self.detail_scroll = 0;
        }
    }

    pub(super) fn previous_log(&mut self) {
        if self.selected_log > 0 {
            self.selected_log -= 1;
            self.log_detail_scroll = 0;
        }
    }

    pub(super) fn next_log(&mut self) {
        let count = self.filtered_logs().len();
        if self.selected_log + 1 < count {
            self.selected_log += 1;
            self.log_detail_scroll = 0;
        }
    }

    fn previous_log_page(&mut self) {
        self.selected_log = self.selected_log.saturating_sub(5);
        self.log_detail_scroll = 0;
    }

    fn next_log_page(&mut self) {
        let count = self.filtered_logs().len();
        if count > 0 {
            self.selected_log = (self.selected_log + 5).min(count - 1);
            self.log_detail_scroll = 0;
        }
    }

    fn last_log(&mut self) {
        let count = self.filtered_logs().len();
        if count > 0 {
            self.selected_log = count - 1;
            self.log_detail_scroll = 0;
        }
    }

    pub(super) fn scroll_detail_up(&mut self, amount: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(amount);
    }

    pub(super) fn scroll_detail_down(&mut self, amount: u16) {
        let max_scroll = self.detail_line_count().saturating_sub(1) as u16;
        self.detail_scroll = self.detail_scroll.saturating_add(amount).min(max_scroll);
    }

    fn detail_scroll_to_end(&mut self) {
        self.detail_scroll = self.detail_line_count().saturating_sub(1) as u16;
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn previous_review_item(&mut self) {
        if self.selected_review_item > 0 {
            self.selected_review_item -= 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn next_review_item(&mut self) {
        let count = self.review_items().len();
        if self.selected_review_item + 1 < count {
            self.selected_review_item += 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn last_review_item(&mut self) {
        let count = self.review_items().len();
        if count > 0 {
            self.selected_review_item = count - 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn scroll_review_detail_up(&mut self, amount: u16) {
        self.review_detail_scroll = self.review_detail_scroll.saturating_sub(amount);
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn scroll_review_detail_down(&mut self, amount: u16) {
        let max_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
        self.review_detail_scroll = self
            .review_detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn review_detail_scroll_to_end(&mut self) {
        self.review_detail_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
    }

    pub(super) fn scroll_log_detail_up(&mut self, amount: u16) {
        self.log_detail_scroll = self.log_detail_scroll.saturating_sub(amount);
    }

    pub(super) fn scroll_log_detail_down(&mut self, amount: u16) {
        let max_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
        self.log_detail_scroll = self
            .log_detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    fn log_detail_scroll_to_end(&mut self) {
        self.log_detail_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
    }

    pub(super) fn clamp_selection(&mut self) {
        if self.states.is_empty() {
            self.states.push("todo".to_string());
        }
        if self.selected_state >= self.states.len() {
            self.selected_state = self.states.len().saturating_sub(1);
        }
        let count = self.selected_state_count();
        if count == 0 {
            self.selected_item = 0;
        } else if self.selected_item >= count {
            self.selected_item = count - 1;
        }
        let max_scroll = self.detail_line_count().saturating_sub(1) as u16;
        self.detail_scroll = self.detail_scroll.min(max_scroll);
        self.clamp_review_selection();

        let log_count = self.filtered_logs().len();
        if log_count == 0 {
            self.selected_log = 0;
        } else if self.selected_log >= log_count {
            self.selected_log = log_count - 1;
        }
        let max_log_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
        self.log_detail_scroll = self.log_detail_scroll.min(max_log_scroll);
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn clamp_review_selection(&mut self) {
        let count = review::queue_len(&self.docs);
        if count == 0 {
            self.selected_review_item = 0;
        } else if self.selected_review_item >= count {
            self.selected_review_item = count - 1;
        }
        let max_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
        self.review_detail_scroll = self.review_detail_scroll.min(max_scroll);
    }

    pub(super) fn selected_state_count(&self) -> usize {
        if self.board_arrangement == BoardArrangement::Epic {
            return self.epic_board_entries().len();
        }
        self.states
            .get(self.selected_state)
            .map(|state| self.state_board_entries(state).len())
            .unwrap_or(0)
    }

    pub(super) fn selected_state_summary(&self) -> String {
        if self.board_arrangement == BoardArrangement::Epic {
            let count = self.selected_state_count();
            return format!("EPIC · {} row{}", count, if count == 1 { "" } else { "s" });
        }
        let Some(state) = self.states.get(self.selected_state) else {
            return "No state · 0 items".to_string();
        };
        let visible_rows = self.selected_state_count();
        let state_tasks = self
            .docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .filter(|doc| document_state_label(doc) == state.as_str())
            .filter(|doc| board_filters_match(doc, &self.board_filters))
            .count();
        if visible_rows == state_tasks {
            format!(
                "{} · {} row{}",
                display_state_label(state),
                visible_rows,
                if visible_rows == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} · {} task{} · {} row{}",
                display_state_label(state),
                state_tasks,
                if state_tasks == 1 { "" } else { "s" },
                visible_rows,
                if visible_rows == 1 { "" } else { "s" }
            )
        }
    }

    pub(super) fn selected_doc(&self) -> Option<&Document> {
        if self.board_arrangement == BoardArrangement::Epic {
            return self
                .epic_board_entries()
                .into_iter()
                .nth(self.selected_item)
                .map(|entry| entry.doc);
        }
        let state = self.states.get(self.selected_state)?;
        self.state_board_entries(state)
            .into_iter()
            .nth(self.selected_item)
            .map(|entry| entry.doc)
    }

    pub(super) fn state_board_entries(&self, state: &str) -> Vec<StateBoardEntry<'_>> {
        let Some(hierarchy) = self.hierarchy.valid_index() else {
            return Vec::new();
        };
        state_board_entries_with_hierarchy(
            &self.docs,
            &self.logs,
            state,
            &self.board_filters,
            &self.expanded_board_hierarchy_ids,
            hierarchy,
        )
    }

    pub(super) fn epic_board_entries(&self) -> Vec<EpicBoardEntry<'_>> {
        let Some(hierarchy) = self.hierarchy.valid_index() else {
            return Vec::new();
        };
        epic_board_entries_with_hierarchy(&self.docs, &self.logs, &self.board_filters, hierarchy)
    }

    pub(super) fn relationship_context(&self, doc: &Document) -> BoardRelationshipContext {
        relationship_context_for_doc_with_hierarchy(
            doc,
            &self.docs,
            &self.logs,
            self.hierarchy.index.as_ref(),
        )
    }

    fn detail_line_count(&self) -> usize {
        self.selected_doc()
            .map(|doc| {
                detail_lines_for_doc_with_context(
                    doc,
                    &self.theme,
                    &relationship_context_for_doc_with_hierarchy(
                        doc,
                        &self.docs,
                        &self.logs,
                        self.hierarchy.index.as_ref(),
                    ),
                )
            })
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn filtered_logs(&self) -> Vec<&Document> {
        logs::filter_logs(
            &self.logs,
            self.hierarchy.index.as_ref(),
            &self.log_search_filter,
        )
    }

    pub(super) fn selected_log(&self) -> Option<&Document> {
        self.filtered_logs().into_iter().nth(self.selected_log)
    }

    pub(super) fn select_log_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        let logs = self.filtered_logs();
        if let Some(index) = logs.iter().position(|doc| doc.id() == id) {
            self.selected_log = index;
            self.clamp_selection();
            true
        } else {
            self.clamp_selection();
            false
        }
    }

    fn log_events_for(&self, id: &str) -> &[logs::LogEvent] {
        self.log_events.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    fn log_detail_line_count(&self) -> usize {
        self.selected_log()
            .map(|doc| {
                logs::detail_lines_for_log(
                    doc,
                    self.hierarchy.index.as_ref(),
                    self.log_events_for(doc.id()),
                    &self.theme,
                )
            })
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn logs_status_message(&self) -> String {
        let visible = self.filtered_logs().len();
        if self.log_search_filter.is_empty() {
            format!(
                "Logs view active: {} archived item{} loaded. Press / to search, j/k to select, and h/l or Tab for list/detail focus.",
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Logs filter `{}` matched {} of {} archived item{}; Esc clears filter.",
                self.log_search_filter,
                visible,
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn review_items(&self) -> Vec<review::ReviewQueueItem> {
        review::queue_items_with_hierarchy(&self.docs, &self.logs, self.hierarchy.index.as_ref())
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn selected_review_item(&self) -> Option<review::ReviewQueueItem> {
        review::selected_item(&self.docs, &self.logs, self.selected_review_item)
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn select_review_item_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        let items = self.review_items();
        if let Some(index) = items.iter().position(|item| item.id() == id) {
            self.selected_review_item = index;
            self.clamp_review_selection();
            true
        } else {
            self.clamp_review_selection();
            false
        }
    }

    #[allow(
        dead_code,
        reason = "retained Review navigation remains compiled pending a separate product decision"
    )]
    fn review_detail_line_count(&self) -> usize {
        let item = self.selected_review_item();
        review::detail_line_count(item.as_ref(), &self.theme)
    }

    pub(super) fn board_docs(&self) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .collect()
    }

    pub(super) fn decision_docs(&self) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| is_decision_doc(doc))
            .collect()
    }

    pub(super) fn rules_total(&self) -> usize {
        self.rules.values().map(Vec::len).sum()
    }

    pub(super) fn draw_logs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if area.width >= 100 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(area);
            self.draw_log_list(frame, chunks[0]);
            self.draw_log_detail(frame, chunks[1]);
        } else {
            let detail_height = (area.height / 2).max(6);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(detail_height)])
                .split(area);
            self.draw_log_list(frame, chunks[0]);
            self.draw_log_detail(frame, chunks[1]);
        }
    }

    fn draw_log_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusLogList,
        });

        let filtered = self.filtered_logs();
        let count = filtered.len();
        let title = if self.log_search_filter.is_empty() {
            format!(" Logs ({count}/{}) ", self.logs.len())
        } else {
            format!(
                " Logs filter `{}` ({count}/{}) ",
                self.log_search_filter,
                self.logs.len()
            )
        };
        let items = if self.logs.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                format!(
                    "No completed logs found in {}.",
                    display_path(&self.workspace.logs_dir)
                ),
                self.theme.muted_style(),
            )))]
        } else if filtered.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                format!(
                    "No logs match `{}`. Press Esc to clear.",
                    self.log_search_filter
                ),
                self.theme.muted_style(),
            )))]
        } else {
            filtered
                .iter()
                .map(|doc| {
                    logs::list_item_for_log(
                        doc,
                        self.hierarchy.index.as_ref(),
                        &self.theme,
                        area.width.saturating_sub(4),
                    )
                })
                .collect::<Vec<_>>()
        };

        let list = List::new(items)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(self.theme.border_style(self.focus == FocusPane::Board))
                    .style(self.theme.panel_style()),
            )
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");

        if count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_log.min(count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            drop(filtered);
            self.register_log_row_hits(area, count);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn register_log_row_hits(&mut self, area: Rect, count: usize) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let top = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let bottom = area.y.saturating_add(area.height).saturating_sub(1);
        for index in 0..count {
            let y = top.saturating_add(index as u16);
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: 1,
                },
                action: HitAction::SelectLog(index),
            });
        }
    }

    fn draw_log_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusLogDetail,
        });

        let focused = self.focus == FocusPane::Detail;
        let (title, lines) = match self.selected_log() {
            Some(doc) => (
                format!(" Log detail {} ", doc.id()),
                logs::detail_lines_for_log(
                    doc,
                    self.hierarchy.index.as_ref(),
                    self.log_events_for(doc.id()),
                    &self.theme,
                ),
            ),
            None if self.logs.is_empty() => (
                " Log detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No completed logs are available yet. Complete a task to create one.",
                    self.theme.muted_style(),
                ))],
            ),
            None => (
                " Log detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No log matches the current filter.",
                    self.theme.muted_style(),
                ))],
            ),
        };
        let detail = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(self.theme.border_style(focused))
                    .style(self.theme.panel_style()),
            )
            .scroll((self.log_detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }
}

pub(super) fn workspace_title_from_root(root: Option<&Yaml>) -> Option<String> {
    root.and_then(|root| yaml_mapping_value(root, "title"))
        .and_then(yaml_scalar_to_string)
        .filter(|title| !title.trim().is_empty())
}

pub(super) fn workspace_states_from_root(root: Option<&Yaml>) -> Vec<String> {
    workflow_states(root)
}

pub(super) fn default_workspace_states() -> Vec<String> {
    workflow::DEFAULT_STATES
        .iter()
        .map(|state| (*state).to_string())
        .collect()
}

pub(super) fn states_with_board_docs(mut states: Vec<String>, docs: &[Document]) -> Vec<String> {
    for doc in docs.iter().filter(|doc| is_board_visible_doc(doc)) {
        let state = document_state_label(doc);
        if !states.iter().any(|known| known == &state) {
            states.push(state);
        }
    }
    if states.is_empty() {
        states.push("todo".to_string());
    }
    states
}

pub(super) fn document_state_label(doc: &Document) -> String {
    doc.field("state")
        .filter(|state| !state.trim().is_empty())
        .unwrap_or("unfiled")
        .to_string()
}

pub(super) fn is_decision_doc(doc: &Document) -> bool {
    doc.doc_type().eq_ignore_ascii_case("decision")
}

pub(super) fn is_board_visible_doc(doc: &Document) -> bool {
    doc.location == DocumentLocation::Board && !is_decision_doc(doc)
}

#[cfg(test)]
pub(super) fn validation_load_errors(
    docs: &[Document],
    logs: &[Document],
    configured_states: &[String],
) -> Vec<String> {
    let hierarchy = TuiHierarchySnapshot::from_documents(docs, logs);
    validation_load_errors_with_hierarchy(docs, logs, configured_states, &hierarchy)
}

pub(super) fn validation_load_errors_with_hierarchy(
    docs: &[Document],
    logs: &[Document],
    configured_states: &[String],
    hierarchy: &TuiHierarchySnapshot,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut ids = BTreeSet::new();
    let mut id_paths: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for doc in docs.iter().chain(logs.iter()) {
        let id = doc.id().trim();
        if !id.is_empty() {
            ids.insert(id.to_string());
            id_paths
                .entry(id.to_string())
                .or_default()
                .push(display_path(&doc.path));
        }
    }

    for (id, paths) in id_paths.iter().filter(|(_, paths)| paths.len() > 1) {
        warnings.push(format!(
            "Validation error: duplicate id `{id}` in {}",
            paths.join(", ")
        ));
    }

    warnings.extend(hierarchy.errors.iter().cloned());

    for doc in docs.iter().chain(logs.iter()) {
        let mut errors = crate::protocol::diagnostic::metadata_diagnostics(
            doc,
            doc.location == DocumentLocation::Logs,
        )
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == crate::protocol::diagnostic::Severity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
        if let Some(diagnostic) = crate::protocol::diagnostic::workflow_state_diagnostic(
            doc,
            doc.location == DocumentLocation::Board && doc.doc_type() == "task",
            configured_states,
        ) {
            errors.push(diagnostic.message);
        }

        if let Some(parent) = doc
            .field("parentId")
            .filter(|value| !value.trim().is_empty())
        {
            if !ids.contains(parent) {
                errors.push(format!("unresolved parentId `{parent}`"));
            }
        }
        for blocker in doc
            .field("blockers")
            .map(parse_field_values)
            .unwrap_or_default()
        {
            if !ids.contains(&blocker) {
                errors.push(format!("unresolved blocker `{blocker}`"));
            }
        }

        if !errors.is_empty() {
            warnings.push(format!(
                "Validation error: {}: {}",
                display_path(&doc.path),
                errors.join("; ")
            ));
        }
    }

    warnings
}

pub(super) fn runtime_warning_status_note(outcome: &ReloadOutcome) -> String {
    match outcome.warning_count {
        0 => String::new(),
        1 => format!(
            "; 1 runtime warning: {}",
            truncate(
                outcome.first_warning.as_deref().unwrap_or("inspect status"),
                120
            )
        ),
        count => format!(
            "; {count} runtime warnings; first: {}",
            truncate(
                outcome.first_warning.as_deref().unwrap_or("inspect status"),
                120
            )
        ),
    }
}

pub(super) fn collect_reload_fingerprint(workspace: &TandemProject) -> ReloadFingerprint {
    let mut files = BTreeMap::new();
    insert_optional_fingerprint(&mut files, workspace.config_path.clone());
    insert_optional_fingerprint(&mut files, workspace.events_path.clone());
    insert_optional_fingerprint(&mut files, theme::workspace_theme_path(workspace));
    insert_optional_fingerprint(&mut files, theme::workspace_config_path(workspace));
    if let Some(user_config_path) = theme::user_config_path_from_env() {
        insert_optional_fingerprint(&mut files, user_config_path);
    }
    insert_directory_fingerprints(&mut files, &workspace.board_dir, "md");
    insert_directory_fingerprints(&mut files, &workspace.logs_dir, "md");
    insert_directory_fingerprints(&mut files, &workspace.papercuts_dir(), "md");
    insert_directory_fingerprints(&mut files, &workspace.events_dir(), "jsonl");
    if let Some(user_theme_dir) = theme::user_theme_dir_from_env() {
        insert_directory_fingerprints(&mut files, &user_theme_dir, "toml");
    }
    ReloadFingerprint { files }
}

fn insert_optional_fingerprint(
    files: &mut BTreeMap<PathBuf, Option<FileSignature>>,
    path: PathBuf,
) {
    let signature = file_signature(&path).ok();
    files.insert(path, signature);
}

fn insert_directory_fingerprints(
    files: &mut BTreeMap<PathBuf, Option<FileSignature>>,
    dir: &Path,
    extension: &str,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            insert_optional_fingerprint(files, path);
        }
    }
}

#[cfg(test)]
pub(super) fn review_attention_reason(doc: &Document) -> Option<String> {
    match accord_status(doc) {
        Some("delivered") => return Some("accord delivered".to_string()),
        Some("blocked") => return Some("accord blocked".to_string()),
        Some("failed") => return Some("accord failed".to_string()),
        Some("rework") => return Some("accord in rework".to_string()),
        Some("accepted") => return Some("accord accepted; not completed".to_string()),
        _ => {}
    }

    match review_status(doc) {
        Some("pending") => Some("review pending".to_string()),
        Some("changes-requested") => Some("changes requested".to_string()),
        Some("rejected") => Some("review rejected".to_string()),
        Some("failed") => Some("review failed".to_string()),
        _ if doc
            .field("blockers")
            .map(parse_field_values)
            .map(|blockers| !blockers.is_empty())
            .unwrap_or(false) =>
        {
            Some("has blockers".to_string())
        }
        _ => None,
    }
}

pub(super) fn append_load_error_lines(lines: &mut Vec<Line<'static>>, load_errors: &[String]) {
    if load_errors.is_empty() {
        return;
    }
    lines.push(Line::from(Span::styled(
        "Load warnings:",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));
    for error in load_errors {
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(""));
}

pub(super) fn quick_add_state_for_selection(
    configured_states: &[String],
    visible_states: &[String],
    selected_state: usize,
) -> (String, Option<String>) {
    let fallback = configured_states
        .first()
        .cloned()
        .unwrap_or_else(|| "todo".to_string());
    let Some(selected) = visible_states.get(selected_state) else {
        return (fallback, Some("no selected state".to_string()));
    };
    if configured_states.iter().any(|state| state == selected) {
        (selected.clone(), None)
    } else {
        (
            fallback,
            Some(format!(
                "selected bucket `{selected}` is not a configured state"
            )),
        )
    }
}

pub(super) fn quick_add_status(input: &QuickAddInput) -> String {
    let fallback = input
        .fallback_note
        .as_ref()
        .map(|note| format!(" ({note})"))
        .unwrap_or_default();
    let title = if input.title.is_empty() {
        "<title>".to_string()
    } else {
        input.title.clone()
    };
    format!(
        "Add task in {}{}: {} · Enter create · Esc cancel",
        input.state, fallback, title
    )
}

pub(super) fn validation_prompt_lines(
    prompt: &ValidationPrompt,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Target: ", theme.label_style()),
            Span::styled(
                format!("{} — {}", prompt.id(), prompt.title()),
                theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];
    match prompt {
        ValidationPrompt::Accept { .. } => {
            lines.push(Line::from(Span::styled(
                "Accept this delivery as human sign-off?",
                theme.text_style(),
            )));
            lines.push(Line::from(Span::styled(
                "Enter/y accepts; Esc/n cancels. Completion/logging remains a separate later action.",
                theme.muted_style(),
            )));
        }
        ValidationPrompt::Rework { feedback, .. } => {
            lines.push(Line::from(Span::styled(
                "Feedback to append durably:",
                theme.label_style(),
            )));
            lines.push(Line::from(Span::styled(
                if feedback.is_empty() {
                    "<type feedback>".to_string()
                } else {
                    feedback.clone()
                },
                if feedback.is_empty() {
                    theme.muted_style()
                } else {
                    theme.text_style()
                },
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter requests rework and moves the item back to in-progress; Esc cancels without writing.",
                theme.muted_style(),
            )));
        }
        ValidationPrompt::ApplyAccepted { candidates } => {
            lines.push(Line::from(Span::styled(
                "These accepted Validation tasks will be completed and moved to logs:",
                theme.text_style(),
            )));
            for candidate in candidates.iter().take(8) {
                lines.push(Line::from(vec![
                    Span::styled("• ", theme.muted_style()),
                    Span::styled(candidate.id.clone(), theme.label_style()),
                    Span::styled(format!(" — {}", candidate.title), theme.text_style()),
                ]));
            }
            if candidates.len() > 8 {
                lines.push(Line::from(Span::styled(
                    format!("… and {} more", candidates.len() - 8),
                    theme.muted_style(),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter/y applies; Esc/n cancels without changing files. Delivered or rework items are excluded.",
                theme.muted_style(),
            )));
        }
    }
    lines
}
