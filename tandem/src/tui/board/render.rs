//! Board layout, Ratatui rendering, and frame-local hit geometry.

use super::*;

impl TuiApp {
    pub(in crate::tui) fn draw_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let content_area = if self.board_filters.is_active() && area.height >= 7 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(4)])
                .split(area);
            self.draw_board_filter_bar(frame, chunks[0]);
            chunks[1]
        } else {
            area
        };

        if !self.hierarchy.errors.is_empty() {
            self.draw_hierarchy_errors(frame, content_area);
            return;
        }

        if self.show_board_detail {
            let detail_height = (content_area.height / 3)
                .clamp(5, 12)
                .min(content_area.height.saturating_sub(4));
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(detail_height)])
                .split(content_area);
            if self.board_arrangement == BoardArrangement::Epic {
                self.draw_epic_board(frame, chunks[0]);
            } else {
                self.draw_state_tabs(frame, chunks[0]);
            }
            self.draw_detail(frame, chunks[1]);
        } else if self.board_arrangement == BoardArrangement::Epic {
            self.draw_epic_board(frame, content_area);
        } else {
            self.draw_state_tabs(frame, content_area);
        }
    }

    pub(in crate::tui) fn draw_hierarchy_errors(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = vec![
            Line::from(Span::styled(
                "Board hierarchy is invalid; task rows and graph-sensitive mutations are disabled.",
                self.theme.status_style(StatusTone::Error),
            )),
            Line::from(Span::styled(
                "Fix the referenced documents, then reload. Canonical shape: Epic → Task → Subtask.",
                self.theme.muted_style(),
            )),
            Line::from(""),
        ];
        for error in &self.hierarchy.errors {
            lines.push(Line::from(vec![
                Span::styled("• ", self.theme.status_style(StatusTone::Error)),
                Span::styled(error.clone(), self.theme.text_style()),
            ]));
        }
        let panel = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Hierarchy errors ({}) ",
                        self.hierarchy.errors.len()
                    ))
                    .border_style(self.theme.status_style(StatusTone::Error))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, area);
    }

    pub(in crate::tui) fn draw_board_filter_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let filter_bar = Paragraph::new(board_filter_bar_line(&self.board_filters, &self.theme))
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Active Board filters ")
                    .border_style(self.theme.status_style(StatusTone::Warning))
                    .style(self.theme.panel_style()),
            );
        frame.render_widget(filter_bar, area);
    }

    pub(in crate::tui) fn draw_epic_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let mode_line = Line::from(vec![
            Span::styled(" Board arrangement ", self.theme.muted_style()),
            Span::styled(" State ", self.theme.tab_style()),
            Span::raw(" "),
            Span::styled(" Epic ", self.theme.state_tab_selected_style()),
            Span::styled("  b switch ", self.theme.muted_style()),
        ]);
        frame.render_widget(Paragraph::new(mode_line), chunks[0]);
        self.draw_epic_board_list(frame, chunks[1]);
    }

    pub(in crate::tui) fn draw_epic_board_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::SelectState(self.selected_state),
        });

        let entries = self.epic_board_entries();
        let count = entries.len();
        let content_width = area
            .width
            .saturating_sub(4)
            .saturating_sub(BOARD_LIST_HIGHLIGHT_SYMBOL_WIDTH) as usize;
        let preview_line_limit = inline_preview_line_limit_for_area(area);
        let items = if entries.is_empty() {
            let empty_text = if self.board_filters.is_active() {
                "No Epic Board rows match the active filters. Press F to clear filters."
            } else {
                "No epic groups are available. Press b for State Board."
            };
            vec![ListItem::new(Line::from(Span::styled(
                empty_text,
                self.theme.muted_style(),
            )))]
        } else {
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let context = self.relationship_context(entry.doc);
                    epic_list_item_for_entry(
                        entry,
                        &context,
                        &self.theme,
                        content_width,
                        preview_line_limit,
                        self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()),
                        index == self.selected_item,
                    )
                })
                .collect::<Vec<_>>()
        };

        let title = format!(
            " Epic Board · {} row{} ",
            count,
            if count == 1 { "" } else { "s" }
        );
        let list = List::new(items)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(self.theme.border_style(self.focus == FocusPane::Board))
                    .style(self.theme.panel_style()),
            )
            .highlight_style(self.theme.board_selected_style())
            .highlight_symbol("▸ ");

        if count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_item.min(count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            let row_heights = entries
                .iter()
                .map(|entry| {
                    if self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()) {
                        let context = self.relationship_context(entry.doc);
                        1 + inline_preview_height_with_context(
                            entry.doc,
                            &context,
                            content_width,
                            preview_line_limit,
                        )
                    } else {
                        1
                    }
                })
                .collect::<Vec<_>>();
            self.register_board_row_hits(area, self.selected_state, state.offset(), &row_heights);
        } else {
            frame.render_widget(list, area);
        }
    }

    pub(in crate::tui) fn draw_state_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let subviews = board_subview_tabs(&self.states, &self.docs, &self.board_filters);
        let titles = subviews
            .iter()
            .map(|tab| Line::from(state_tab_title(&tab.state, tab.count)))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.selected_state)
            .style(self.theme.tab_style())
            .highlight_style(self.theme.state_tab_selected_style());
        frame.render_widget(tabs, chunks[0]);
        self.register_state_tab_hits(chunks[0], &subviews);
        self.draw_state_list(frame, chunks[1], self.selected_state);
    }

    pub(in crate::tui) fn register_state_tab_hits(
        &mut self,
        area: Rect,
        subviews: &[BoardSubviewTab],
    ) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let mut x = area.x;
        let right = area.x.saturating_add(area.width);
        for (index, tab) in subviews.iter().enumerate() {
            let width =
                (state_tab_title(&tab.state, tab.count).chars().count() as u16).saturating_add(1);
            if x >= right {
                break;
            }
            let clamped_width = width.min(right.saturating_sub(x));
            if clamped_width > 0 {
                self.hits.push(HitRegion {
                    rect: Rect {
                        x,
                        y: area.y,
                        width: clamped_width,
                        height: 1,
                    },
                    action: HitAction::SelectState(index),
                });
            }
            x = x.saturating_add(width);
        }
    }

    pub(in crate::tui) fn draw_state_list(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        state_index: usize,
    ) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::SelectState(state_index),
        });

        let Some(state_name) = self.states.get(state_index) else {
            return;
        };
        let entries = self.state_board_entries(state_name);
        let row_count = entries.len();
        let state_task_count = self
            .docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .filter(|doc| document_state_label(doc) == state_name.as_str())
            .filter(|doc| board_filters_match(doc, &self.board_filters))
            .count();
        let content_width = area.width.saturating_sub(4) as usize;
        let preview_line_limit = inline_preview_line_limit_for_area(area);
        let items = if entries.is_empty() {
            let empty_text = if self.board_filters.is_active() {
                "No hierarchy matches the active Board filters. Press F to clear filters."
            } else if state_task_count > 0 {
                "Tasks in this state are nested under parents in other state tabs."
            } else {
                "No active items in this state. Press a to quick-add here."
            };
            vec![ListItem::new(Line::from(Span::styled(
                empty_text,
                self.theme.muted_style(),
            )))]
        } else {
            let show_doc_type = entries.iter().any(|entry| entry.doc.doc_type() != "task");
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let context = self.relationship_context(entry.doc);
                    state_list_item_for_entry(
                        entry,
                        &context,
                        &self.theme,
                        (
                            content_width,
                            show_doc_type,
                            preview_line_limit,
                            self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()),
                            index == self.selected_item,
                        ),
                    )
                })
                .collect::<Vec<_>>()
        };

        let title = if row_count == state_task_count {
            format!(
                " {} · {} row{} ",
                display_state_label(state_name),
                row_count,
                if row_count == 1 { "" } else { "s" }
            )
        } else {
            format!(
                " {} · {} task{} · {} visible row{} ",
                display_state_label(state_name),
                state_task_count,
                if state_task_count == 1 { "" } else { "s" },
                row_count,
                if row_count == 1 { "" } else { "s" }
            )
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
            .highlight_style(self.theme.board_selected_style())
            .highlight_symbol("› ");

        if row_count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_item.min(row_count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            let row_heights = entries
                .iter()
                .map(|entry| {
                    if self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()) {
                        let context = self.relationship_context(entry.doc);
                        1 + inline_preview_height_with_context(
                            entry.doc,
                            &context,
                            content_width,
                            preview_line_limit,
                        )
                    } else {
                        1
                    }
                })
                .collect::<Vec<_>>();
            self.register_board_row_hits(area, state_index, state.offset(), &row_heights);
        } else {
            frame.render_widget(list, area);
        }
    }

    pub(in crate::tui) fn register_board_row_hits(
        &mut self,
        area: Rect,
        state_index: usize,
        first_visible_index: usize,
        row_heights: &[u16],
    ) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let mut y = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let bottom = area.y.saturating_add(area.height).saturating_sub(1);
        for (index, height) in row_heights
            .iter()
            .copied()
            .enumerate()
            .skip(first_visible_index)
        {
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: height.min(bottom.saturating_sub(y)),
                },
                action: HitAction::SelectBoardItem(state_index, index),
            });
            y = y.saturating_add(height);
        }
    }

    pub(in crate::tui) fn draw_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusDetail,
        });

        let focused = self.focus == FocusPane::Detail;
        let (title, lines) = match self.selected_doc() {
            Some(doc) => (
                format!(" Detail {} ", doc.id()),
                detail_lines_for_doc_with_context(
                    doc,
                    &self.theme,
                    &relationship_context_for_doc_with_hierarchy(
                        doc,
                        &self.docs,
                        &self.logs,
                        self.hierarchy.index.as_ref(),
                    ),
                ),
            ),
            None => (
                " Detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No item selected in this state.",
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
            .scroll((self.detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }
}
