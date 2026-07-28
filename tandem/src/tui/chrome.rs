//! Shared top-level TUI chrome and frame-local hit geometry.

use super::*;

impl TuiApp {
    pub(super) fn draw_tiny(&self, frame: &mut Frame<'_>, area: Rect) {
        let message = Paragraph::new(vec![
            Line::from(Span::styled(
                "Tandem TUI needs a larger terminal",
                self.theme
                    .status_style(StatusTone::Warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "Current: {}x{} · minimum: 45x12",
                area.width, area.height
            )),
            Line::from("Press q to quit after resizing if needed."),
        ])
        .style(self.theme.panel_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" tandem tui ")
                .border_style(self.theme.border_style(true))
                .style(self.theme.panel_style()),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
    }

    fn board_header_context(&self) -> Line<'static> {
        let Some(doc) = self.selected_doc() else {
            return Line::from(Span::styled("No selected item", self.theme.muted_style()));
        };
        let mut spans = vec![Span::styled(
            format!("Selected {}", doc.id()),
            self.theme.muted_style(),
        )];
        let is_epic = self
            .hierarchy
            .valid_index()
            .is_some_and(|hierarchy| hierarchy.task_role(doc).ok() == Some(Some(TaskRole::Epic)));
        if is_epic {
            let (outstanding, completed) = count_task_descendants(
                doc.id(),
                &self.docs,
                &self.logs,
                &mut BTreeSet::from([doc.id().to_string()]),
            );
            let total = outstanding + completed;
            if let Some(filled) = completed
                .checked_mul(24usize)
                .and_then(|value| value.checked_div(total))
            {
                let width = 24usize;
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    "█".repeat(filled),
                    self.theme.progress_style(),
                ));
                spans.push(Span::styled(
                    "░".repeat(width - filled),
                    self.theme.muted_style(),
                ));
                spans.push(Span::styled(
                    format!(" {completed}/{total} complete"),
                    self.theme.muted_style(),
                ));
            }
        }
        Line::from(spans)
    }

    pub(super) fn draw_header(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let context = match self.view {
            TuiView::Board => self.board_header_context(),
            TuiView::Logs => {
                let filter = if self.log_search_filter.is_empty() {
                    String::new()
                } else {
                    format!(" · filter `{}`", self.log_search_filter)
                };
                Line::from(
                    self.selected_log()
                        .map(|doc| {
                            format!(
                                "Selected {} · {} {}{}",
                                doc.id(),
                                if is_canceled_log(doc) {
                                    "canceled"
                                } else {
                                    "completed"
                                },
                                logs::completed_at_compact(
                                    doc.field("completedAt").unwrap_or("unknown")
                                ),
                                filter
                            )
                        })
                        .unwrap_or_else(|| format!("No archived log selected{filter}")),
                )
            }
            TuiView::Rules => Line::from(self.rules_context()),
            TuiView::Decisions => Line::from(self.decisions_context()),
        };
        let tab_area = header_inner_row(area, 0);
        let header = Paragraph::new(vec![self.view_tab_line(tab_area.width), context])
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(self.title.clone(), self.theme.title_style()),
                        Span::raw(" · "),
                        Span::styled(
                            self.view.label(),
                            self.theme.text_style().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                    ]))
                    .border_style(self.theme.border_style(false))
                    .style(self.theme.panel_style()),
            );
        frame.render_widget(header, area);
        self.register_view_tab_hits(header_inner_row(area, 0));
    }

    pub(super) fn view_tab_line(&self, width: u16) -> Line<'static> {
        let counts = self.view_counts();
        let tab_widths = TuiView::ALL
            .into_iter()
            .enumerate()
            .map(|(index, view)| view_tab_text_width(view, counts[index]))
            .collect::<Vec<_>>();
        let content_width: u16 = tab_widths.iter().sum();
        let gaps = TuiView::ALL.len().saturating_sub(1) as u16;
        let gap_width = width
            .saturating_sub(content_width)
            .checked_div(gaps)
            .map_or(0, |gap_width| gap_width.clamp(3, 8));
        let total_width = content_width.saturating_add(gap_width.saturating_mul(gaps));
        let leading = width.saturating_sub(total_width) / 2;

        let mut spans = Vec::new();
        if leading > 0 {
            spans.push(Span::raw(" ".repeat(leading as usize)));
        }
        for (index, view) in TuiView::ALL.into_iter().enumerate() {
            if index > 0 {
                spans.push(Span::raw(" ".repeat(gap_width as usize)));
            }
            spans.extend(self.view_tab_spans(view, counts[index]));
        }
        Line::from(spans)
    }

    fn view_counts(&self) -> [usize; 4] {
        [
            self.board_docs().len(),
            self.logs.len(),
            self.rules_total(),
            self.decision_docs().len(),
        ]
    }

    fn view_tab_spans(&self, view: TuiView, count: usize) -> Vec<Span<'static>> {
        let selected = view == self.view;
        let label_style = if selected {
            self.theme.tab_selected_style()
        } else {
            self.theme.text_style()
        };
        let shortcut_style = if selected {
            self.theme.tab_selected_style()
        } else {
            self.theme.muted_style()
        };
        let count_style = self.theme.muted_style();

        vec![
            Span::styled(format!("[{}] ", view.shortcut()), shortcut_style),
            Span::styled(view.label().to_string(), label_style),
            Span::styled(format!(" ({count})"), count_style),
        ]
    }

    fn register_view_tab_hits(&mut self, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let counts = self.view_counts();
        let tab_widths = TuiView::ALL
            .into_iter()
            .enumerate()
            .map(|(index, view)| view_tab_text_width(view, counts[index]))
            .collect::<Vec<_>>();
        let content_width: u16 = tab_widths.iter().sum();
        let gaps = TuiView::ALL.len().saturating_sub(1) as u16;
        let gap_width = area
            .width
            .saturating_sub(content_width)
            .checked_div(gaps)
            .map_or(0, |gap_width| gap_width.clamp(3, 8));
        let total_width = content_width.saturating_add(gap_width.saturating_mul(gaps));
        let mut x = area
            .x
            .saturating_add(area.width.saturating_sub(total_width) / 2);
        let right = area.x.saturating_add(area.width);
        let y = area.y;
        for (index, view) in TuiView::ALL.into_iter().enumerate() {
            if index > 0 {
                x = x.saturating_add(gap_width);
            }
            let width = tab_widths[index];
            if x >= right {
                break;
            }
            let clamped_width = width.min(right.saturating_sub(x));
            if clamped_width > 0 {
                self.hits.push(HitRegion {
                    rect: Rect {
                        x,
                        y,
                        width: clamped_width,
                        height: 1,
                    },
                    action: HitAction::SwitchView(view),
                });
            }
            x = x.saturating_add(width);
        }
    }

    pub(super) fn draw_placeholder_view(&self, frame: &mut Frame<'_>, area: Rect) {
        let (title, lines) = match self.view {
            TuiView::Board => (" Board ".to_string(), Vec::new()),
            TuiView::Logs => self.logs_placeholder_lines(),
            TuiView::Rules => (" Rules ".to_string(), Vec::new()),
            TuiView::Decisions => (" Decisions ".to_string(), Vec::new()),
        };
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn logs_placeholder_lines(&self) -> (String, Vec<Line<'static>>) {
        let mut lines = vec![
            Line::from(Span::styled(
                "Logs fallback",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "{} completed log{} loaded from {}.",
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" },
                display_path(&self.workspace.logs_dir)
            )),
            Line::from(""),
        ];
        append_load_error_lines(&mut lines, &self.load_errors);
        if self.logs.is_empty() {
            lines.push(Line::from(Span::styled(
                "No completed logs found.",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "Recent logs:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
            for doc in self.logs.iter().take(10) {
                let completed = doc
                    .field("completedAt")
                    .unwrap_or("unknown completion time");
                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", doc.id()), Style::default().fg(Color::Cyan)),
                    Span::styled(completed.to_string(), Style::default().fg(Color::Gray)),
                    Span::raw(" — "),
                    Span::styled(truncate(doc.title(), 48), Style::default().fg(Color::White)),
                ]));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Logs list/detail/search render in the primary Logs view; this fallback should rarely appear.",
            Style::default().fg(Color::DarkGray),
        )));
        (" Logs ".to_string(), lines)
    }

    pub(super) fn with_status(&self, base: String) -> String {
        if self.status.is_empty() {
            base
        } else {
            format!("{base} · {}", self.status)
        }
    }

    pub(super) fn board_footer_text(&self) -> String {
        if !self.hierarchy.errors.is_empty() {
            return "board · HIERARCHY INVALID · fix referenced documents and reload · ? help"
                .to_string();
        }
        let context = match self.focus {
            FocusPane::Board => "board",
            FocusPane::Detail => "detail",
        };
        let selected_is_validation = self
            .selected_doc()
            .map(|doc| document_state_label(doc) == "validation")
            .unwrap_or(false);
        let arrangement_hint = match self.board_arrangement {
            BoardArrangement::State => "b Epic Board",
            BoardArrangement::Epic => "b State Board",
        };
        let enter_hint = match self.board_arrangement {
            BoardArrangement::State => "Enter expand/preview · Space preview",
            BoardArrangement::Epic => "Enter/Space preview",
        };
        let commands = if self.focus == FocusPane::Detail {
            format!("Tab board · j/k scroll · e edit · {arrangement_hint} · ? help")
        } else if selected_is_validation {
            format!("{enter_hint} · A accept · R rework · C apply accepted · {arrangement_hint} · ? help")
        } else if self.board_filters.is_active() {
            format!("{enter_hint} · F clear filter · H prev · L next · {arrangement_hint} · ? help")
        } else {
            format!("{enter_hint} · a add · t tag · p priority · {arrangement_hint} · ? help")
        };
        self.with_status(format!(
            "{context} · {} · {commands}",
            self.selected_state_summary()
        ))
    }

    pub(super) fn logs_footer_text(&self) -> String {
        if !self.log_search_filter.is_empty() {
            return self.with_status(format!(
                "Logs filter `{}` · Esc clear · / search · ? help",
                self.log_search_filter
            ));
        }
        let (context, commands) = match self.focus {
            FocusPane::Board => ("list", "Enter detail · / search · ? help"),
            FocusPane::Detail => ("detail", "Enter list · j/k scroll · ? help"),
        };
        self.with_status(format!("Logs {context} · {commands}"))
    }

    pub(super) fn draw_footer(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let footer_line = if let Some(input) = self.quick_add.as_ref() {
            Line::from(Span::styled(
                quick_add_status(input),
                self.theme.status_style(StatusTone::Warning),
            ))
        } else if self.log_search_input.is_some() || self.validation_prompt.is_some() {
            Line::from(Span::styled(
                self.status.clone(),
                self.theme.status_style(StatusTone::Warning),
            ))
        } else if let Some(status) = self.rules_prompt_status() {
            Line::from(Span::styled(
                status,
                self.theme.status_style(StatusTone::Warning),
            ))
        } else if let Some(status) = self.decision_prompt_status() {
            Line::from(Span::styled(
                status,
                self.theme.status_style(StatusTone::Warning),
            ))
        } else {
            self.footer_line_for_text(match self.view {
                TuiView::Board => self.board_footer_text(),
                TuiView::Logs => self.logs_footer_text(),
                TuiView::Rules => self.rules_footer_text(),
                TuiView::Decisions => self.decisions_footer_text(),
            })
        };
        let footer_text = footer_line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        frame.render_widget(Paragraph::new(footer_line), area);
        self.register_footer_hits(area, &footer_text);
    }

    pub(super) fn footer_line_for_text(&self, hints: String) -> Line<'static> {
        let hint_style = self.theme.text_style();
        let separator_style = self.theme.muted_style();
        let Some(status) = (!self.status.is_empty()).then_some(self.status.as_str()) else {
            return Line::from(Span::styled(hints, hint_style));
        };
        let suffix = format!(" · {status}");
        let Some(base) = hints.strip_suffix(&suffix) else {
            return Line::from(Span::styled(hints, hint_style));
        };
        Line::from(vec![
            Span::styled(base.to_string(), hint_style),
            Span::styled(" · ", separator_style),
            Span::styled(
                status.to_string(),
                self.theme.status_style(status_tone_for_message(status)),
            ),
        ])
    }

    fn register_footer_hits(&mut self, area: Rect, text: &str) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        match self.view {
            TuiView::Board => {
                self.register_footer_hit(
                    area,
                    text,
                    "Enter expand",
                    HitAction::ToggleBoardExpansion,
                );
                self.register_footer_hit(area, text, "Tab board", HitAction::ToggleBoardDetail);
                self.register_footer_hit(
                    area,
                    text,
                    "b Epic Board",
                    HitAction::ToggleBoardArrangement,
                );
                self.register_footer_hit(
                    area,
                    text,
                    "b State Board",
                    HitAction::ToggleBoardArrangement,
                );
                self.register_footer_hit(area, text, "a add", HitAction::StartQuickAdd);
                self.register_footer_hit(area, text, "t tag", HitAction::CycleBoardTagFilter);
                self.register_footer_hit(
                    area,
                    text,
                    "p priority",
                    HitAction::CycleBoardPriorityFilter,
                );
                self.register_footer_hit(
                    area,
                    text,
                    "F clear filter",
                    HitAction::ClearBoardFilters,
                );
                self.register_footer_hit(area, text, "H prev", HitAction::MoveSelectedTask(-1));
                self.register_footer_hit(area, text, "L next", HitAction::MoveSelectedTask(1));
                self.register_footer_hit(
                    area,
                    text,
                    "A accept",
                    HitAction::ShowValidationAction("accept"),
                );
                self.register_footer_hit(
                    area,
                    text,
                    "R rework",
                    HitAction::ShowValidationAction("rework"),
                );
                self.register_footer_hit(
                    area,
                    text,
                    "C apply accepted",
                    HitAction::ShowValidationAction("apply"),
                );
                self.register_footer_hit(area, text, "e edit", HitAction::OpenEditor);
            }
            TuiView::Logs => {
                self.register_footer_hit(area, text, "Enter detail", HitAction::ToggleFocus);
                self.register_footer_hit(area, text, "Enter list", HitAction::ToggleFocus);
                self.register_footer_hit(area, text, "/ search", HitAction::StartLogSearch);
            }
            TuiView::Rules | TuiView::Decisions => {}
        }
        self.register_footer_hit(area, text, "? help", HitAction::ShowHelp);
    }

    fn register_footer_hit(&mut self, area: Rect, text: &str, label: &str, action: HitAction) {
        if let Some(start) = text.find(label) {
            let x = area.x.saturating_add(start as u16);
            if x >= area.x.saturating_add(area.width) {
                return;
            }
            let width = (label.chars().count() as u16)
                .min(area.x.saturating_add(area.width).saturating_sub(x));
            if width > 0 {
                self.hits.push(HitRegion {
                    rect: Rect {
                        x,
                        y: area.y,
                        width,
                        height: 1,
                    },
                    action,
                });
            }
        }
    }

    pub(super) fn show_validation_action_hint(&mut self, action: &str) {
        match action {
            "accept" | "approve" => self.start_validation_accept(),
            "rework" => self.start_validation_rework(),
            "apply" | "archive" => self.start_validation_apply_accepted(),
            "complete" => self.show_validation_complete_hint(),
            _ => self.status = format!("Unknown Validation action `{action}`."),
        }
    }

    pub(super) fn help_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled("Tandem TUI help", self.theme.title_style())),
            Line::from(Span::styled(
                "Keyboard-first commands grouped by view. Press ? / Esc / q to close.",
                self.theme.muted_style(),
            )),
            Line::from(""),
        ];
        self.push_help_section(&mut lines, "Global");
        self.push_help_command(&mut lines, "q, Ctrl-C", "quit safely");
        self.push_help_command(&mut lines, "r", "reload board/config/log/theme data");
        self.push_help_command(
            &mut lines,
            "1 2 3 4",
            "switch Board, Logs, Rules, Decisions",
        );
        self.push_help_command(
            &mut lines,
            "mouse",
            "click tabs/lists/panes; wheel selects or scrolls",
        );

        self.push_help_section(&mut lines, "Navigation");
        self.push_help_command(
            &mut lines,
            "j/k, ↑/↓",
            "move selection; scroll detail when focused",
        );
        self.push_help_command(&mut lines, "h/l, ←/→", "move within the active view");
        self.push_help_command(
            &mut lines,
            "g/G",
            "first/last item in the active list or detail",
        );
        self.push_help_command(
            &mut lines,
            "Tab",
            "Board detail toggle; Logs/Decisions focus toggle",
        );

        self.push_help_section(&mut lines, "Board");
        self.push_help_command(
            &mut lines,
            "Enter",
            "expand/collapse task children; preview leaf rows",
        );
        self.push_help_command(&mut lines, "Space", "toggle inline row preview");
        self.push_help_command(&mut lines, "b", "toggle State/Epic Board arrangement");
        self.push_help_command(&mut lines, "a", "quick-add a task in the selected state");
        self.push_help_command(&mut lines, "e", "open the selected active task in $EDITOR");
        self.push_help_command(
            &mut lines,
            "t / p / F",
            "cycle tag filter, priority filter, clear filters",
        );
        self.push_help_command(
            &mut lines,
            "H / L",
            "move selected task to previous/next state",
        );

        self.push_help_section(&mut lines, "Validation");
        self.push_help_command(
            &mut lines,
            "A",
            "open accept confirmation for delivered work",
        );
        self.push_help_command(&mut lines, "R", "open feedback dialog and request rework");
        self.push_help_command(
            &mut lines,
            "C",
            "open Apply accepted dialog to archive accepted Validation tasks",
        );

        self.push_help_section(&mut lines, "Logs");
        self.push_help_command(&mut lines, "Enter", "toggle list/detail focus");
        self.push_help_command(
            &mut lines,
            "/",
            "search id, title, summary, body, validation, files",
        );
        self.push_help_command(&mut lines, "Esc", "clear search filter or return to list");
        self.push_help_command(
            &mut lines,
            "e",
            "read-only; generated history is not edited here",
        );

        self.push_help_section(&mut lines, "Rules");
        self.push_help_command(
            &mut lines,
            "h/l",
            "switch always/never/prefer/context categories",
        );
        self.push_help_command(&mut lines, "a or n", "add a rule");
        self.push_help_command(&mut lines, "e / d", "edit or delete the selected rule");

        self.push_help_section(&mut lines, "Decisions");
        self.push_help_command(&mut lines, "Enter", "toggle list/body focus");
        self.push_help_command(&mut lines, "a", "add a decision document");
        self.push_help_command(&mut lines, "PgUp/PgDn", "scroll selected decision body");
        self.push_help_command(
            &mut lines,
            "e",
            "use CLI decision update/withdraw; editor actions are deferred",
        );

        self.push_help_section(&mut lines, "Prompts");
        self.push_help_command(&mut lines, "Enter", "advance/save prompt input");
        self.push_help_command(&mut lines, "Esc", "cancel prompt or close help");
        self.push_help_command(&mut lines, "Ctrl-U", "clear current prompt field");
        lines
    }

    fn push_help_section(&self, lines: &mut Vec<Line<'static>>, title: &'static str) {
        if lines.last().is_some_and(|line| !line.spans.is_empty()) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            title,
            self.theme.label_style().add_modifier(Modifier::BOLD),
        )));
    }

    fn push_help_command(
        &self,
        lines: &mut Vec<Line<'static>>,
        keys: &'static str,
        detail: &'static str,
    ) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {keys:<12}"),
                self.theme.status_style(StatusTone::Accent),
            ),
            Span::styled(detail.to_string(), self.theme.text_style()),
        ]));
    }

    pub(super) fn draw_validation_prompt(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(prompt) = self.validation_prompt.as_ref() else {
            return;
        };
        let popup = centered_rect(76, 36, area);
        frame.render_widget(Clear, popup);
        let lines = validation_prompt_lines(prompt, &self.theme);
        let prompt_view = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(match prompt {
                        ValidationPrompt::Accept { .. } => " Accept sign-off ",
                        ValidationPrompt::Rework { .. } => " Request rework ",
                        ValidationPrompt::ApplyAccepted { .. } => " Apply accepted ",
                    })
                    .border_style(self.theme.border_style(true))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(prompt_view, popup);
    }

    pub(super) fn draw_help(&self, frame: &mut Frame<'_>, area: Rect) {
        let popup = centered_rect(78, 72, area);
        frame.render_widget(Clear, popup);
        let help = Paragraph::new(self.help_lines())
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help ")
                    .border_style(self.theme.border_style(true))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(help, popup);
    }
}

pub(super) fn status_tone_for_message(message: &str) -> StatusTone {
    let lower = message.to_ascii_lowercase();
    if lower.contains("error") || lower.contains("failed") || lower.contains("failure") {
        StatusTone::Error
    } else if lower.contains("warning") || lower.contains("canceled") || lower.contains("needs") {
        StatusTone::Warning
    } else if lower.contains("created")
        || lower.contains("moved")
        || lower.contains("loaded")
        || lower.contains("added")
        || lower.contains("edited")
        || lower.contains("deleted")
    {
        StatusTone::Success
    } else if lower.contains("active") {
        StatusTone::Accent
    } else {
        StatusTone::Muted
    }
}

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn view_tab_text_width(view: TuiView, count: usize) -> u16 {
    format!("[{}] {} ({count})", view.shortcut(), view.label())
        .chars()
        .count() as u16
}

fn header_inner_row(area: Rect, row: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1).saturating_add(row),
        width: area.width.saturating_sub(2),
        height: 1,
    }
}

pub(super) fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}
