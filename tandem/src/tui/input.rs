//! Keyboard and mouse event translation for the TUI aggregate.
//!
//! This adapter translates terminal events into UI operations and contains no
//! project filesystem or application mutation calls.

use super::*;

impl TuiApp {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<KeyAction, CliError> {
        // Fixed precedence: emergency quit, text input, modal layers, close/quit,
        // universal actions, view actions, then pane navigation.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(KeyAction::Quit);
        }

        if self.quick_add.is_some() {
            self.handle_quick_add_key(key);
            return Ok(KeyAction::Continue);
        }
        if matches!(
            self.validation_prompt,
            Some(ValidationPrompt::Rework { .. })
        ) {
            self.handle_validation_prompt_key(key);
            return Ok(KeyAction::Continue);
        }
        if self.log_search_input.is_some() {
            self.handle_log_search_key(key);
            return Ok(KeyAction::Continue);
        }
        if self.rules_text_prompt_active() {
            self.handle_rules_prompt_key(key);
            return Ok(KeyAction::Continue);
        }
        if self.decision_prompt_active() {
            self.handle_decision_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.show_help {
            match key.code {
                KeyCode::Char('q') => return Ok(KeyAction::Quit),
                KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll = self.help_scroll.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll = self.help_scroll.saturating_add(1)
                }
                KeyCode::PageUp => self.help_scroll = self.help_scroll.saturating_sub(8),
                KeyCode::PageDown => self.help_scroll = self.help_scroll.saturating_add(8),
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.help_scroll = self.help_scroll.saturating_sub(8)
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.help_scroll = self.help_scroll.saturating_add(8)
                }
                KeyCode::Home | KeyCode::Char('g') => self.help_scroll = 0,
                KeyCode::End | KeyCode::Char('G') => self.help_scroll = u16::MAX,
                KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                    self.select_help_section(-1)
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => self.select_help_section(1),
                _ => {}
            }
            return Ok(KeyAction::Continue);
        }

        if self.board_picker.is_some() {
            match key.code {
                KeyCode::Char('q') => return Ok(KeyAction::Quit),
                KeyCode::Char('?') => self.open_help(),
                KeyCode::Char(ch) if TuiView::from_digit(ch).is_some() => {
                    self.board_picker = None;
                    self.switch_view(TuiView::from_digit(ch).unwrap());
                }
                _ => self.handle_picker_key(key),
            }
            return Ok(KeyAction::Continue);
        }

        if self.validation_prompt.is_some() || self.rules_prompt_active() {
            match key.code {
                KeyCode::Char('q') => return Ok(KeyAction::Quit),
                KeyCode::Char('?') => self.open_help(),
                _ if self.validation_prompt.is_some() => self.handle_validation_prompt_key(key),
                _ => self.handle_rules_prompt_key(key),
            }
            return Ok(KeyAction::Continue);
        }

        if self.papercuts_open() {
            match key.code {
                KeyCode::Char('q') => return Ok(KeyAction::Quit),
                KeyCode::Char('?') => self.open_help(),
                KeyCode::Char('i') | KeyCode::Esc => self.close_papercuts(),
                KeyCode::Char(ch) if TuiView::from_digit(ch).is_some() => {
                    self.close_papercuts();
                    self.switch_view(TuiView::from_digit(ch).unwrap());
                }
                _ => self.handle_papercuts_key(key),
            }
            return Ok(KeyAction::Continue);
        }

        match key.code {
            KeyCode::Esc => match self.view {
                TuiView::Board if self.focus == FocusPane::Detail => self.focus = FocusPane::Board,
                TuiView::Logs => self.clear_log_filter_or_focus(),
                TuiView::Decisions if self.focus == FocusPane::Detail => {
                    self.focus = FocusPane::Board
                }
                _ => {}
            },
            KeyCode::Char('q') => return Ok(KeyAction::Quit),
            KeyCode::Char('?') => self.open_help(),
            KeyCode::Char('r') => {
                self.reload();
            }
            KeyCode::Char('i') => self.toggle_papercuts(),
            KeyCode::Char(ch) if TuiView::from_digit(ch).is_some() => {
                self.switch_view(TuiView::from_digit(ch).unwrap())
            }
            KeyCode::Char('a') if self.view == TuiView::Board => self.start_quick_add(),
            KeyCode::Char('a') if self.view == TuiView::Rules => self.start_rule_add_prompt(),
            KeyCode::Char('a') if self.view == TuiView::Decisions => {
                self.start_decision_add_prompt()
            }
            KeyCode::Char('e') if self.view == TuiView::Board => return Ok(KeyAction::OpenEditor),
            KeyCode::Char('e') if self.view == TuiView::Logs => {
                self.status = "Completed logs are read-only in the TUI.".into()
            }
            KeyCode::Char('e') if self.view == TuiView::Decisions => {
                self.status = "Decision editing remains available through the CLI.".into()
            }
            KeyCode::Char('b') if self.view == TuiView::Board => self.toggle_board_arrangement(),
            KeyCode::Char('f') if self.view == TuiView::Board => self.start_filter_picker(),
            KeyCode::Char('m') if self.view == TuiView::Board => self.start_move_picker(),
            KeyCode::Char('v') if self.view == TuiView::Board => self.start_validation_picker(),
            KeyCode::Char('/') if self.view == TuiView::Logs => self.start_log_search(),
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_previous(),
            KeyCode::Enter if self.view == TuiView::Board => self.toggle_board_expansion(),
            KeyCode::Char(' ') if self.view == TuiView::Board => self.toggle_board_preview(),
            KeyCode::Enter if self.view == TuiView::Logs => self.activate_logs_selection(),
            _ => match self.view {
                TuiView::Board => match self.focus {
                    FocusPane::Board => self.handle_board_key(key),
                    FocusPane::Detail => self.handle_detail_key(key),
                },
                TuiView::Logs => self.handle_logs_key(key),
                TuiView::Rules => self.handle_rules_key(key),
                TuiView::Decisions => self.handle_decisions_key(key),
            },
        }
        Ok(KeyAction::Continue)
    }

    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) -> KeyAction {
        if self.show_help {
            match mouse.kind {
                MouseEventKind::ScrollUp => self.help_scroll = self.help_scroll.saturating_sub(3),
                MouseEventKind::ScrollDown => self.help_scroll = self.help_scroll.saturating_add(3),
                MouseEventKind::Down(MouseButton::Left) => {
                    match self.mouse_hit_action(mouse.column, mouse.row) {
                        Some(HitAction::CloseHelp) => self.show_help = false,
                        Some(HitAction::HelpSection(index)) => {
                            self.help_section = index;
                            self.help_scroll = 0;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            return KeyAction::Continue;
        }
        if self.board_picker.is_some()
            || self.validation_prompt.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
            || self.quick_add.is_some()
            || self.log_search_input.is_some()
        {
            if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                match self.mouse_hit_action(mouse.column, mouse.row) {
                    Some(HitAction::SelectPickerOption(index)) => self.select_picker_option(index),
                    Some(HitAction::ActivatePicker) => self.activate_picker_selection(),
                    Some(HitAction::CancelPicker) => self.cancel_top_modal(),
                    Some(HitAction::ConfirmModal) => self.activate_confirmation(),
                    Some(HitAction::CancelModal) => self.cancel_top_modal(),
                    Some(HitAction::ShowHelp) => self.open_help(),
                    _ => {}
                }
            }
            return KeyAction::Continue;
        }
        if self.papercuts_open() {
            self.handle_papercuts_mouse(mouse);
            return KeyAction::Continue;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let hit = self
                    .hits
                    .iter()
                    .rev()
                    .find(|hit| rect_contains(hit.rect, mouse.column, mouse.row))
                    .cloned();
                if let Some(hit) = hit {
                    match hit.action {
                        HitAction::SwitchView(view) => self.switch_view(view),
                        HitAction::SelectState(index) if self.view == TuiView::Board => {
                            self.selected_state = index.min(self.states.len().saturating_sub(1));
                            self.selected_item = 0;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::SelectState(_) => {}
                        HitAction::SelectBoardItem(state_index, item_index)
                            if self.view == TuiView::Board =>
                        {
                            let state_index = state_index.min(self.states.len().saturating_sub(1));
                            let was_selected = self.selected_state == state_index
                                && self.selected_item == item_index
                                && self.focus == FocusPane::Board;
                            self.selected_state = state_index;
                            self.selected_item = item_index;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                            if was_selected {
                                self.toggle_board_expansion();
                            }
                        }
                        HitAction::SelectBoardItem(_, _) => {}
                        HitAction::ToggleBoardExpansion if self.view == TuiView::Board => {
                            self.toggle_board_expansion()
                        }
                        HitAction::ToggleBoardExpansion => {}
                        HitAction::ToggleBoardDetail if self.view == TuiView::Board => {
                            self.toggle_board_detail()
                        }
                        HitAction::ToggleBoardDetail => {}
                        HitAction::ToggleBoardArrangement if self.view == TuiView::Board => {
                            self.toggle_board_arrangement()
                        }
                        HitAction::ToggleBoardArrangement => {}
                        HitAction::StartQuickAdd if self.view == TuiView::Board => {
                            self.start_quick_add()
                        }
                        HitAction::StartQuickAdd => {}
                        HitAction::OpenFilterPicker if self.view == TuiView::Board => {
                            self.start_filter_picker()
                        }
                        HitAction::OpenMovePicker if self.view == TuiView::Board => {
                            self.start_move_picker()
                        }
                        HitAction::OpenValidationPicker if self.view == TuiView::Board => {
                            self.start_validation_picker()
                        }
                        HitAction::OpenFilterPicker
                        | HitAction::OpenMovePicker
                        | HitAction::OpenValidationPicker => {}
                        HitAction::SelectPickerOption(_)
                        | HitAction::ActivatePicker
                        | HitAction::CancelPicker => {}
                        HitAction::OpenEditor if self.view == TuiView::Board => {
                            return KeyAction::OpenEditor
                        }
                        HitAction::OpenEditor => {}
                        HitAction::ShowHelp => self.show_help = true,
                        HitAction::FocusDetail if self.view == TuiView::Board => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusDetail => {}
                        HitAction::FocusReviewList => {}
                        HitAction::SelectReviewItem(_) => {}
                        HitAction::FocusReviewDetail => {}
                        HitAction::SelectLog(index) if self.view == TuiView::Logs => {
                            self.selected_log = index;
                            self.log_detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::SelectLog(_) => {}
                        HitAction::SelectRuleCategory(index) if self.view == TuiView::Rules => {
                            self.rules_view.selected_category = index;
                            self.rules_view.selected_item = 0;
                            self.rules_view.list_offset = 0;
                            self.clamp_rules_state();
                        }
                        HitAction::SelectRuleCategory(_) => {}
                        HitAction::SelectRuleItem(index) if self.view == TuiView::Rules => {
                            self.rules_view.selected_item = index;
                            self.rules_view.preview_scroll = 0;
                            self.focus_rule_list();
                            self.clamp_rules_state();
                        }
                        HitAction::SelectRuleItem(_) => {}
                        HitAction::ToggleRulePreview if self.view == TuiView::Rules => {
                            self.handle_rules_key(KeyEvent::from(KeyCode::Enter))
                        }
                        HitAction::FocusRuleList if self.view == TuiView::Rules => {
                            self.focus_rule_list()
                        }
                        HitAction::FocusRulePreview if self.view == TuiView::Rules => {
                            self.focus_rule_preview()
                        }
                        HitAction::FocusRuleList
                        | HitAction::FocusRulePreview
                        | HitAction::ToggleRulePreview => {}
                        HitAction::FocusLogList if self.view == TuiView::Logs => {
                            self.focus = FocusPane::Board
                        }
                        HitAction::FocusLogList => {}
                        HitAction::FocusLogDetail if self.view == TuiView::Logs => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusLogDetail => {}
                        HitAction::StartLogSearch if self.view == TuiView::Logs => {
                            self.start_log_search()
                        }
                        HitAction::StartLogSearch => {}
                        HitAction::ToggleFocus => self.toggle_focus(),
                        HitAction::SelectDecision(index) if self.view == TuiView::Decisions => {
                            self.select_decision(index)
                        }
                        HitAction::SelectDecision(_) => {}
                        HitAction::FocusDecisionList if self.view == TuiView::Decisions => {
                            self.focus = FocusPane::Board
                        }
                        HitAction::FocusDecisionDetail if self.view == TuiView::Decisions => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusDecisionList | HitAction::FocusDecisionDetail => {}
                        HitAction::TogglePapercuts => self.toggle_papercuts(),
                        HitAction::ConfirmModal
                        | HitAction::CancelModal
                        | HitAction::HelpSection(_)
                        | HitAction::CloseHelp => {}
                        HitAction::FocusPapercutList
                        | HitAction::SelectPapercut(_)
                        | HitAction::FocusPapercutDetail => {}
                    }
                }
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Board => {
                self.scroll_board_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Board => {
                self.scroll_board_at(mouse, -3)
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Logs => {
                self.scroll_logs_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Logs => {
                self.scroll_logs_at(mouse, -3)
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Rules => {
                self.scroll_rules_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Rules => {
                self.scroll_rules_at(mouse, -3)
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Decisions => {
                self.scroll_decisions_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Decisions => {
                self.scroll_decisions_at(mouse, -3)
            }
            _ => {}
        }
        KeyAction::Continue
    }

    pub(super) fn mouse_hit_action(&self, column: u16, row: u16) -> Option<HitAction> {
        self.hits
            .iter()
            .rev()
            .find(|hit| rect_contains(hit.rect, column, row))
            .map(|hit| hit.action.clone())
    }

    pub(super) fn scroll_board_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusDetail) => {
                self.focus = FocusPane::Detail;
                if amount > 0 {
                    self.scroll_detail_down(amount as u16);
                } else {
                    self.scroll_detail_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::SelectState(_))
            | Some(HitAction::SelectBoardItem(_, _))
            | Some(HitAction::ToggleBoardExpansion)
            | None => {
                self.focus = FocusPane::Board;
                if amount > 0 {
                    self.next_item();
                } else {
                    self.previous_item();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scroll_rules_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusRulePreview) | Some(HitAction::ToggleRulePreview) => {
                self.focus_rule_preview();
                if amount > 0 {
                    self.scroll_rule_preview_down(amount as u16);
                } else {
                    self.scroll_rule_preview_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::FocusRuleList) | Some(HitAction::SelectRuleItem(_)) | None => {
                if amount > 0 {
                    self.next_rule_selection();
                } else {
                    self.previous_rule_selection();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scroll_decisions_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusDecisionDetail) => {
                self.focus = FocusPane::Detail;
                if amount > 0 {
                    self.scroll_decision_detail_down(amount as u16);
                } else {
                    self.scroll_decision_detail_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::FocusDecisionList) | Some(HitAction::SelectDecision(_)) | None => {
                self.focus = FocusPane::Board;
                if amount > 0 {
                    self.next_decision_selection();
                } else {
                    self.previous_decision_selection();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scroll_logs_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusLogDetail) => {
                self.focus = FocusPane::Detail;
                if amount > 0 {
                    self.scroll_log_detail_down(amount as u16);
                } else {
                    self.scroll_log_detail_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::FocusLogList) | Some(HitAction::SelectLog(_)) | None => {
                self.focus = FocusPane::Board;
                if amount > 0 {
                    self.next_log();
                } else {
                    self.previous_log();
                }
            }
            _ => {}
        }
    }
}
