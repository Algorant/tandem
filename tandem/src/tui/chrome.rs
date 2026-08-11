//! Shared top-level TUI chrome and frame-local hit geometry.

use super::*;

const TRANSIENT_STATUS_TTL: Duration = Duration::from_secs(4);

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
        let header = Block::default()
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
            .style(self.theme.panel_style());
        frame.render_widget(header, area);

        let tab_area = header_inner_row(area, 0);
        frame.render_widget(
            Paragraph::new(self.view_tab_line(tab_area.width)).style(self.theme.panel_style()),
            tab_area,
        );
        let context_area = header_inner_row(area, 1);
        let indicator_width = (self.papercut_indicator_text().chars().count() as u16)
            .saturating_add(2)
            .min(context_area.width);
        let context_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(indicator_width)])
            .split(context_area);
        frame.render_widget(
            Paragraph::new(context).style(self.theme.panel_style()),
            context_chunks[0],
        );
        let indicator_area = Rect {
            x: context_chunks[1].x.saturating_add(1),
            y: context_chunks[1].y,
            width: context_chunks[1].width.saturating_sub(1),
            height: context_chunks[1].height,
        };
        frame.render_widget(
            Paragraph::new(self.papercut_indicator_line()).style(self.theme.panel_style()),
            indicator_area,
        );
        self.register_view_tab_hits(tab_area);
        if context_chunks[1].width > 0 {
            self.hits.push(HitRegion {
                rect: context_chunks[1],
                action: HitAction::TogglePapercuts,
            });
        }
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

    pub(super) fn expire_transient_status(&mut self) -> bool {
        if self.status != self.observed_status {
            self.observed_status.clone_from(&self.status);
            self.status_updated_at = Instant::now();
            return false;
        }
        if self.transient_status_timeout().is_none() {
            return false;
        }
        if self.status_updated_at.elapsed() >= TRANSIENT_STATUS_TTL {
            self.status.clear();
            self.observed_status.clear();
            return true;
        }
        false
    }

    pub(super) fn transient_status_timeout(&self) -> Option<Duration> {
        if self.status.is_empty()
            || self.text_input_active()
            || self.board_picker.is_some()
            || self.validation_prompt.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
            || self.papercuts_open()
            || self.show_help
        {
            None
        } else {
            Some(TRANSIENT_STATUS_TTL.saturating_sub(self.status_updated_at.elapsed()))
        }
    }

    pub(super) fn board_footer_text(&self) -> String {
        if !self.hierarchy.errors.is_empty() {
            return "board · HIERARCHY INVALID · fix referenced documents and reload · ? help"
                .to_string();
        }
        let arrangement_hint = match self.board_arrangement {
            BoardArrangement::State => "b Epic Board",
            BoardArrangement::Epic => "b State Board",
        };
        let commands = if self.focus == FocusPane::Detail {
            format!("e Edit · {arrangement_hint} · ? Help")
        } else {
            format!("a Add · e Edit · f Filter · m Move · v Validate · {arrangement_hint} · ? Help")
        };
        self.with_status(commands)
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
        let mut footer_line = if let Some(input) = self.quick_add.as_ref() {
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
        } else if self.papercuts_open() {
            Line::from(Span::styled(
                self.papercuts_footer_text(),
                self.theme.text_style(),
            ))
        } else {
            self.footer_line_for_text(match self.view {
                TuiView::Board => self.board_footer_text(),
                TuiView::Logs => self.logs_footer_text(),
                TuiView::Rules => self.rules_footer_text(),
                TuiView::Decisions => self.decisions_footer_text(),
            })
        };
        if self.text_input_active() {
            footer_line.spans.push(Span::styled(
                " · [Help]",
                self.theme.status_style(StatusTone::Accent),
            ));
        }
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
        if self.text_input_active() {
            self.register_footer_hit(area, text, "[Help]", HitAction::ShowHelp);
            return;
        }
        if self.papercuts_open() {
            self.register_footer_hit(area, text, "i/Esc close", HitAction::TogglePapercuts);
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
                self.register_footer_hit(area, text, "a Add", HitAction::StartQuickAdd);
                self.register_footer_hit(area, text, "f Filter", HitAction::OpenFilterPicker);
                self.register_footer_hit(area, text, "m Move", HitAction::OpenMovePicker);
                self.register_footer_hit(area, text, "v Validate", HitAction::OpenValidationPicker);
                self.register_footer_hit(area, text, "e Edit", HitAction::OpenEditor);
            }
            TuiView::Logs => {
                self.register_footer_hit(area, text, "Enter detail", HitAction::ToggleFocus);
                self.register_footer_hit(area, text, "Enter list", HitAction::ToggleFocus);
                self.register_footer_hit(area, text, "/ search", HitAction::StartLogSearch);
            }
            TuiView::Rules | TuiView::Decisions => {}
        }
        self.register_footer_hit(area, text, "? Help", HitAction::ShowHelp);
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
            Line::from(Span::styled(
                "Universal keybinding reference",
                self.theme.title_style(),
            )),
            Line::from(Span::styled(
                format!(
                    "Current context: {} · Tab/Shift-Tab or h/l changes section",
                    self.help_context_label()
                ),
                self.theme.muted_style(),
            )),
            Line::from(""),
        ];
        let selected = BindingScope::ALL[self.help_section.min(BindingScope::ALL.len() - 1)];
        let order = std::iter::once(selected).chain(
            BindingScope::ALL
                .into_iter()
                .filter(|scope| *scope != selected),
        );
        for scope in order {
            lines.push(Line::from(Span::styled(
                if scope == selected {
                    format!("◆ {}", scope.label())
                } else {
                    scope.label().to_string()
                },
                if scope == selected {
                    self.theme
                        .status_style(StatusTone::Accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    self.theme.label_style().add_modifier(Modifier::BOLD)
                },
            )));
            for binding in bindings_for(scope) {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<22}", binding.keys),
                        self.theme.status_style(StatusTone::Accent),
                    ),
                    Span::styled(binding.description, self.theme.text_style()),
                ]));
            }
            lines.push(Line::from(""));
        }
        lines
    }

    fn help_context_label(&self) -> &'static str {
        if self.board_picker.is_some() {
            return "Board action picker";
        }
        if self.validation_prompt.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
        {
            return "dialog";
        }
        if self.papercuts_open() {
            return "Utility inbox";
        }
        self.view.label()
    }

    pub(super) fn draw_validation_prompt(&mut self, frame: &mut Frame<'_>, area: Rect) {
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
        let buttons = Rect::new(
            popup.x.saturating_add(2),
            popup.bottom().saturating_sub(2),
            popup.width.saturating_sub(4),
            1,
        );
        let halves = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(buttons);
        frame.render_widget(
            Paragraph::new("[ Confirm ]").style(self.theme.status_style(StatusTone::Accent)),
            halves[0],
        );
        frame.render_widget(
            Paragraph::new("[ Cancel ]").style(self.theme.muted_style()),
            halves[1],
        );
        self.hits.push(HitRegion {
            rect: halves[0],
            action: HitAction::ConfirmModal,
        });
        self.hits.push(HitRegion {
            rect: halves[1],
            action: HitAction::CancelModal,
        });
    }

    pub(super) fn draw_help(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let popup = centered_rect(
            if area.width < 80 { 94 } else { 88 },
            if area.height < 24 { 94 } else { 86 },
            area,
        );
        frame.render_widget(Clear, popup);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Key reference ")
            .border_style(self.theme.border_style(true))
            .style(self.theme.panel_style());
        let inner = block.inner(popup);
        frame.render_widget(block, popup);
        let footer = Rect::new(inner.x, inner.bottom().saturating_sub(1), inner.width, 1);
        let content_height = inner.height.saturating_sub(1);
        let content = Rect::new(inner.x, inner.y, inner.width, content_height);
        if content.width >= 72 {
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(22), Constraint::Min(30)])
                .split(content);
            let sections = BindingScope::ALL
                .into_iter()
                .enumerate()
                .map(|(index, scope)| {
                    ListItem::new(Line::from(Span::styled(
                        scope.label(),
                        if index == self.help_section {
                            self.theme.tab_selected_style()
                        } else {
                            self.theme.text_style()
                        },
                    )))
                })
                .collect::<Vec<_>>();
            let mut state = ListState::default();
            state.select(Some(self.help_section));
            frame.render_stateful_widget(
                List::new(sections)
                    .block(Block::default().borders(Borders::RIGHT).title(" Sections "))
                    .highlight_style(self.theme.selected_style()),
                panes[0],
                &mut state,
            );
            for index in 0..BindingScope::ALL.len().min(panes[0].height as usize) {
                self.hits.push(HitRegion {
                    rect: Rect::new(panes[0].x, panes[0].y + index as u16, panes[0].width, 1),
                    action: HitAction::HelpSection(index),
                });
            }
            frame.render_widget(
                Paragraph::new(self.help_lines())
                    .scroll((self.help_scroll, 0))
                    .wrap(Wrap { trim: false }),
                panes[1],
            );
        } else {
            frame.render_widget(
                Paragraph::new(self.help_lines())
                    .scroll((self.help_scroll, 0))
                    .wrap(Wrap { trim: false }),
                content,
            );
        }
        let footer_text = format!(
            "Esc close · q quit · j/k scroll · section {}/{} · click here to close",
            self.help_section + 1,
            BindingScope::ALL.len()
        );
        frame.render_widget(
            Paragraph::new(Span::styled(footer_text, self.theme.muted_style())),
            footer,
        );
        self.hits.push(HitRegion {
            rect: footer,
            action: HitAction::CloseHelp,
        });
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
