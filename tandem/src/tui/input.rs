//! Keyboard and mouse event translation for the TUI aggregate.
//!
//! This adapter translates terminal events into UI operations and contains no
//! project filesystem or application mutation calls.

use super::*;

impl TuiApp {
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<KeyAction, CliError> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(KeyAction::Quit);
        }

        if self.quick_add.is_some() {
            self.handle_quick_add_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.validation_prompt.is_some() {
            self.handle_validation_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.log_search_input.is_some() {
            self.handle_log_search_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.rules_prompt_active() {
            self.handle_rules_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.decision_prompt_active() {
            self.handle_decision_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.show_help = false,
                _ => {}
            }
            return Ok(KeyAction::Continue);
        }

        if let KeyCode::Char(ch) = key.code {
            if let Some(view) = TuiView::from_digit(ch) {
                self.switch_view(view);
                return Ok(KeyAction::Continue);
            }
        }

        match key.code {
            KeyCode::Char('q') => return Ok(KeyAction::Quit),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('r') => {
                self.reload();
            }
            KeyCode::Char('a') if self.view == TuiView::Board => self.start_quick_add(),
            KeyCode::Char('a') if self.view == TuiView::Rules => self.start_rule_add_prompt(),
            KeyCode::Char('a') if self.view == TuiView::Decisions => {
                self.start_decision_add_prompt()
            }
            KeyCode::Char('a') => {
                self.status = "Add is available in Board, Rules, and Decisions views.".to_string()
            }
            KeyCode::Char('A') if self.view == TuiView::Board => self.start_validation_accept(),
            KeyCode::Char('R') if self.view == TuiView::Board => self.start_validation_rework(),
            KeyCode::Char('C') if self.view == TuiView::Board => self.start_validation_apply_accepted(),
            KeyCode::Char('H') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(-1)
            }
            KeyCode::Char('L') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(1)
            }
            KeyCode::Char('b') if self.view == TuiView::Board => self.toggle_board_arrangement(),
            KeyCode::Char('t') if self.view == TuiView::Board => self.cycle_board_tag_filter(),
            KeyCode::Char('p') if self.view == TuiView::Board => self.cycle_board_priority_filter(),
            KeyCode::Char('F') if self.view == TuiView::Board => self.clear_board_filters(),
            KeyCode::Char('H') | KeyCode::Char('L') => {
                self.status = "Task move is available in Board view; press 1 for Board.".to_string()
            }
            KeyCode::Char('/') if self.view == TuiView::Logs => self.start_log_search(),
            KeyCode::Char('/') => {
                self.status = "Search is available in Logs view; press 2 for Logs.".to_string()
            }
            KeyCode::Char('e') if self.view == TuiView::Board => {
                return Ok(KeyAction::OpenEditor)
            }
            KeyCode::Char('e') if self.view == TuiView::Logs => {
                self.status = "Completed logs are read-only in the TUI; $EDITOR is intentionally disabled for generated history.".to_string()
            }
            KeyCode::Char('e') if self.view == TuiView::Decisions => {
                self.status = "Use `tandem decision update <id> …` or `tandem decision withdraw <id> --reason …`; editor-based decision actions are deferred.".to_string()
            }
            KeyCode::Tab | KeyCode::BackTab => self.cycle_focus_or_hint(),
            KeyCode::Enter if self.view == TuiView::Board => self.toggle_board_expansion(),
            KeyCode::Char(' ') if self.view == TuiView::Board => self.toggle_board_preview(),
            KeyCode::Enter if self.view == TuiView::Logs => self.toggle_focus(),
            KeyCode::Esc => match self.view {
                TuiView::Board if self.focus == FocusPane::Detail => {
                    self.focus = FocusPane::Board
                }
                TuiView::Logs => self.clear_log_filter_or_focus(),
                TuiView::Decisions if self.focus == FocusPane::Detail => {
                    self.focus = FocusPane::Board
                }
                _ => {}
            },
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
        if self.input_overlay_active() {
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
                        HitAction::CycleBoardTagFilter if self.view == TuiView::Board => {
                            self.cycle_board_tag_filter()
                        }
                        HitAction::CycleBoardTagFilter => {}
                        HitAction::CycleBoardPriorityFilter if self.view == TuiView::Board => {
                            self.cycle_board_priority_filter()
                        }
                        HitAction::CycleBoardPriorityFilter => {}
                        HitAction::ClearBoardFilters if self.view == TuiView::Board => {
                            self.clear_board_filters()
                        }
                        HitAction::ClearBoardFilters => {}
                        HitAction::MoveSelectedTask(delta) if self.view == TuiView::Board => {
                            self.move_selected_task_by_delta(delta)
                        }
                        HitAction::MoveSelectedTask(_) => {}
                        HitAction::ShowValidationAction(action) if self.view == TuiView::Board => {
                            self.show_validation_action_hint(action)
                        }
                        HitAction::ShowValidationAction(_) => {}
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
                            self.clamp_rules_state();
                        }
                        HitAction::SelectRuleItem(_) => {}
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
            MouseEventKind::ScrollDown if self.view == TuiView::Rules => self.next_rule_selection(),
            MouseEventKind::ScrollUp if self.view == TuiView::Rules => {
                self.previous_rule_selection()
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Decisions => {
                self.next_decision_selection()
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Decisions => {
                self.previous_decision_selection()
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
