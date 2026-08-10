//! Board action pickers. Filter, move, and Validation use one interaction grammar.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PickerKind {
    Filter,
    Move,
    Validation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PickerAction {
    SetTag(Option<String>),
    SetPriority(Option<String>),
    ClearAll,
    Move(String),
    Validation(&'static str),
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

impl TuiApp {
    pub(super) fn start_filter_picker(&mut self) {
        let mut options = Vec::new();
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
        options.extend([
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

    pub(super) fn start_move_picker(&mut self) {
        let Some(doc) = self.selected_doc() else {
            self.status = "No selected task to move.".into();
            return;
        };
        let id = doc.id().to_string();
        let current = document_state_label(doc);
        let options = self
            .configured_states
            .iter()
            .map(|state| PickerOption {
                label: display_state_label(state),
                detail: if *state == current {
                    "Disabled: current state".into()
                } else {
                    format!("Move to `{state}`")
                },
                enabled: *state != current,
                action: PickerAction::Move(state.clone()),
            })
            .collect();
        let mut picker = BoardPicker {
            kind: PickerKind::Move,
            selected: 0,
            title: "Move task".into(),
            context: format!(
                "{id} — {} · current state: {}",
                doc.title(),
                display_state_label(&current)
            ),
            options,
        };
        picker.select_first_enabled();
        self.board_picker = Some(picker);
        self.status =
            "Move picker: select a configured target state; Enter confirms, Esc cancels.".into();
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
        let (context, delivered) = match selected.as_ref() {
            Some((id, title, state, status)) => (
                format!(
                    "{id} — {title} · state {} · accord {status}",
                    display_state_label(state)
                ),
                state == "validation" && normalized_accord_status(status) == "delivered",
            ),
            None => ("No selected Board task".into(), false),
        };
        let reason = match selected.as_ref() {
            None => "Disabled: no selected task".to_string(),
            Some((_, _, state, _)) if state != "validation" => {
                format!("Disabled: state {}", display_state_label(state))
            }
            Some((_, _, _, status)) if normalized_accord_status(status) != "delivered" => {
                format!("Disabled: accord {status}")
            }
            _ => "Available for delivered Validation work".into(),
        };
        let apply_count = app::accord::accepted_validation_candidates(&self.docs).len();
        let mut picker = BoardPicker {
            kind: PickerKind::Validation,
            selected: 0,
            title: "Validation actions".into(),
            context,
            options: vec![
                PickerOption {
                    label: "Accept delivery".into(),
                    detail: reason.clone(),
                    enabled: delivered,
                    action: PickerAction::Validation("accept"),
                },
                PickerOption {
                    label: "Request rework".into(),
                    detail: reason,
                    enabled: delivered,
                    action: PickerAction::Validation("rework"),
                },
                PickerOption {
                    label: "Apply / archive accepted".into(),
                    detail: if apply_count == 0 {
                        "Disabled: no accepted tasks".into()
                    } else {
                        format!(
                            "Archive {apply_count} accepted Validation task{}",
                            plural_suffix(apply_count)
                        )
                    },
                    enabled: apply_count > 0 && self.hierarchy.errors.is_empty(),
                    action: PickerAction::Validation("apply"),
                },
            ],
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
            PickerAction::SetTag(value) => {
                self.board_filters.tag = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::SetPriority(value) => {
                self.board_filters.priority = value;
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::ClearAll => {
                self.board_filters = BoardFilters::default();
                self.restore_filtered_selection(selected_id.as_deref());
            }
            PickerAction::Move(state) => {
                if let Some(id) = selected_id {
                    self.move_selected_task_to_state(&id, &state);
                }
            }
            PickerAction::Validation(action) => self.show_validation_action_hint(action),
        }
    }

    fn restore_filtered_selection(&mut self, id: Option<&str>) {
        if !id.is_some_and(|id| self.select_document_by_id_preserving_scroll(id)) {
            self.clamp_selection();
        }
        self.status = format!("Board {}.", self.board_filters.summary());
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
