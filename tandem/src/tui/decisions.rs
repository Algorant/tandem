use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::theme::{StatusTone, TuiTheme};
use super::{
    centered_rect, detail_field_line, markdownish_lines, push_optional_detail_line, FocusPane,
    TuiApp,
};
use crate::{
    append_event, current_timestamp, display_path, first_body_line, next_sequential_id,
    write_atomic, yaml_double_quote, CliError, Document, Workspace,
};

#[derive(Debug, Default)]
pub(super) struct DecisionsState {
    selected: usize,
    detail_scroll: u16,
    prompt: Option<DecisionPrompt>,
}

impl DecisionsState {
    fn clamp(&mut self, decision_count: usize, detail_line_count: usize) {
        if decision_count == 0 {
            self.selected = 0;
        } else if self.selected >= decision_count {
            self.selected = decision_count - 1;
        }
        let max_scroll = detail_line_count.saturating_sub(1) as u16;
        self.detail_scroll = self.detail_scroll.min(max_scroll);
    }

    pub(super) fn has_prompt(&self) -> bool {
        self.prompt.is_some()
    }
}

#[derive(Debug, Clone)]
enum DecisionPrompt {
    Add {
        title: String,
        body: String,
        step: DecisionPromptStep,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecisionPromptStep {
    Title,
    Body,
}

#[derive(Debug)]
enum DecisionPromptAction {
    None,
    Status(String),
    Cancel(String),
    Add { title: String, body: String },
}

impl TuiApp {
    pub(super) fn clamp_decisions_state(&mut self) {
        let decision_count = self.sorted_decision_docs().len();
        let detail_line_count = self.decision_detail_line_count();
        self.decisions_view.clamp(decision_count, detail_line_count);
    }

    pub(super) fn decision_prompt_active(&self) -> bool {
        self.decisions_view.has_prompt()
    }

    pub(super) fn selected_decision_id_for_reload(&self) -> Option<String> {
        self.selected_decision_doc().map(|doc| doc.id().to_string())
    }

    pub(super) fn restore_decision_selection_after_reload(&mut self, id: Option<String>) {
        if let Some(id) = id.as_deref() {
            if self.select_decision_by_id_preserving_scroll(id) {
                return;
            }
        }
        self.clamp_decisions_state();
    }

    pub(super) fn decision_prompt_status(&self) -> Option<String> {
        self.decisions_view
            .prompt
            .as_ref()
            .map(DecisionPrompt::status_line)
    }

    pub(super) fn handle_decision_prompt_key(&mut self, key: KeyEvent) {
        let action = match self.decisions_view.prompt.as_mut() {
            Some(prompt) => prompt.handle_key(key),
            None => DecisionPromptAction::None,
        };

        match action {
            DecisionPromptAction::None => {
                if let Some(status) = self.decision_prompt_status() {
                    self.status = status;
                }
            }
            DecisionPromptAction::Status(status) => self.status = status,
            DecisionPromptAction::Cancel(status) => {
                self.decisions_view.prompt = None;
                self.status = status;
            }
            DecisionPromptAction::Add { title, body } => {
                self.decisions_view.prompt = None;
                self.finish_decision_add(title, body);
            }
        }
    }

    pub(super) fn handle_decisions_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                FocusPane::Board => self.previous_decision_selection(),
                FocusPane::Detail => self.scroll_decision_detail_up(1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                FocusPane::Board => self.next_decision_selection(),
                FocusPane::Detail => self.scroll_decision_detail_down(1),
            },
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            KeyCode::Home | KeyCode::Char('g') => match self.focus {
                FocusPane::Board => self.first_decision_selection(),
                FocusPane::Detail => self.decisions_view.detail_scroll = 0,
            },
            KeyCode::End | KeyCode::Char('G') => match self.focus {
                FocusPane::Board => self.last_decision_selection(),
                FocusPane::Detail => self.scroll_decision_detail_down(u16::MAX),
            },
            KeyCode::PageUp => self.scroll_decision_detail_up(6),
            KeyCode::PageDown => self.scroll_decision_detail_down(6),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_decision_detail_up(6)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_decision_detail_down(6)
            }
            KeyCode::Char('a') => self.start_decision_add_prompt(),
            KeyCode::Enter => {
                self.status = self
                    .selected_decision_doc()
                    .map(|doc| {
                        format!(
                            "Selected decision {}; body and metadata are shown in the detail pane. Use Tab or l to focus the body.",
                            doc.id()
                        )
                    })
                    .unwrap_or_else(|| "No decision selected; press a to add one.".to_string());
            }
            _ => {}
        }
    }

    pub(super) fn previous_decision_selection(&mut self) {
        if self.decisions_view.selected > 0 {
            self.decisions_view.selected -= 1;
            self.decisions_view.detail_scroll = 0;
        }
    }

    pub(super) fn next_decision_selection(&mut self) {
        let count = self.sorted_decision_docs().len();
        if self.decisions_view.selected + 1 < count {
            self.decisions_view.selected += 1;
            self.decisions_view.detail_scroll = 0;
        }
    }

    pub(super) fn draw_decisions_view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.clamp_decisions_state();
        let chunks = if area.width >= 96 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(area)
        };
        self.draw_decision_list(frame, chunks[0]);
        self.draw_decision_detail(frame, chunks[1]);
    }

    pub(super) fn draw_decision_prompt(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(prompt) = self.decisions_view.prompt.as_ref() else {
            return;
        };
        let popup = centered_rect(76, 42, area);
        frame.render_widget(Clear, popup);
        let prompt_view = Paragraph::new(prompt.modal_lines(&self.theme))
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(prompt.modal_title())
                    .border_style(self.theme.border_style(true))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(prompt_view, popup);
    }

    pub(super) fn decisions_context(&self) -> String {
        let decisions = self.sorted_decision_docs();
        decisions
            .get(self.decisions_view.selected)
            .map(|doc| {
                format!(
                    "selected {} · {} active decision{} loaded",
                    doc.id(),
                    decisions.len(),
                    if decisions.len() == 1 { "" } else { "s" }
                )
            })
            .unwrap_or_else(|| "no active decision documents loaded".to_string())
    }

    pub(super) fn decisions_footer_text(&self) -> String {
        let (context, commands) = match self.focus {
            FocusPane::Board => ("list", "Enter body · a add · ? help"),
            FocusPane::Detail => ("body", "Enter list · j/k scroll · ? help"),
        };
        self.with_status(format!("Decisions {context} · {commands}"))
    }

    fn draw_decision_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let decisions = self.sorted_decision_docs();
        let items = if decisions.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No active decisions. Press a to add one.",
                self.theme.muted_style(),
            )))]
        } else {
            decisions
                .iter()
                .map(|doc| decision_list_item(doc, &self.theme))
                .collect::<Vec<_>>()
        };
        let mut state = ListState::default();
        if !decisions.is_empty() {
            state.select(Some(
                self.decisions_view
                    .selected
                    .min(decisions.len().saturating_sub(1)),
            ));
        }
        let list = List::new(items)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Decisions ({}) ", decisions.len()))
                    .border_style(self.theme.border_style(self.focus == FocusPane::Board))
                    .style(self.theme.panel_style()),
            )
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn draw_decision_detail(&self, frame: &mut Frame<'_>, area: Rect) {
        let (title, lines) = match self.selected_decision_doc() {
            Some(doc) => (
                format!(" Decision {} ", doc.id()),
                decision_detail_lines(doc, &self.theme),
            ),
            None => (
                " Decision detail ".to_string(),
                vec![
                    Line::from(Span::styled(
                        "No decision selected.",
                        self.theme.muted_style(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press a to add a basic decision document.",
                        self.theme.status_style(StatusTone::Accent),
                    )),
                ],
            ),
        };
        let detail = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(self.theme.border_style(self.focus == FocusPane::Detail))
                    .style(self.theme.panel_style()),
            )
            .scroll((self.decisions_view.detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }

    fn sorted_decision_docs(&self) -> Vec<&Document> {
        let mut docs = self
            .docs
            .iter()
            .filter(|doc| doc.doc_type() == "decision")
            .collect::<Vec<_>>();
        docs.sort_by(|a, b| a.id().cmp(b.id()));
        docs
    }

    fn selected_decision_doc(&self) -> Option<&Document> {
        self.sorted_decision_docs()
            .into_iter()
            .nth(self.decisions_view.selected)
    }

    fn first_decision_selection(&mut self) {
        if !self.sorted_decision_docs().is_empty() {
            self.decisions_view.selected = 0;
            self.decisions_view.detail_scroll = 0;
        }
    }

    fn last_decision_selection(&mut self) {
        let count = self.sorted_decision_docs().len();
        if count > 0 {
            self.decisions_view.selected = count - 1;
            self.decisions_view.detail_scroll = 0;
        }
    }

    fn scroll_decision_detail_up(&mut self, amount: u16) {
        self.decisions_view.detail_scroll =
            self.decisions_view.detail_scroll.saturating_sub(amount);
    }

    fn scroll_decision_detail_down(&mut self, amount: u16) {
        let max_scroll = self.decision_detail_line_count().saturating_sub(1) as u16;
        self.decisions_view.detail_scroll = self
            .decisions_view
            .detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    fn decision_detail_line_count(&self) -> usize {
        self.selected_decision_doc()
            .map(|doc| decision_detail_lines(doc, &self.theme).len())
            .unwrap_or(1)
    }

    pub(super) fn start_decision_add_prompt(&mut self) {
        self.decisions_view.prompt = Some(DecisionPrompt::Add {
            title: String::new(),
            body: String::new(),
            step: DecisionPromptStep::Title,
        });
        if let Some(status) = self.decision_prompt_status() {
            self.status = status;
        }
    }

    fn finish_decision_add(&mut self, title: String, body: String) {
        match create_basic_decision(&self.workspace, &title, &body) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_decision_by_id(&outcome.id);
                self.status = format!(
                    "Created decision {}: {}{}",
                    outcome.id, outcome.title, reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Decision add error: {}{}", error.message, reload_note);
            }
        }
    }

    fn select_decision_by_id(&mut self, id: &str) -> bool {
        self.select_decision_by_id_with_scroll(id, true)
    }

    fn select_decision_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        self.select_decision_by_id_with_scroll(id, false)
    }

    fn select_decision_by_id_with_scroll(&mut self, id: &str, reset_scroll: bool) -> bool {
        let decisions = self.sorted_decision_docs();
        if let Some(index) = decisions.iter().position(|doc| doc.id() == id) {
            self.decisions_view.selected = index;
            if reset_scroll {
                self.decisions_view.detail_scroll = 0;
            }
            true
        } else {
            self.clamp_decisions_state();
            false
        }
    }
}

impl DecisionPrompt {
    fn handle_key(&mut self, key: KeyEvent) -> DecisionPromptAction {
        match self {
            Self::Add { title, body, step } => match key.code {
                KeyCode::Esc => DecisionPromptAction::Cancel("Decision add canceled.".to_string()),
                KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => match step {
                    DecisionPromptStep::Title => {
                        if title.trim().is_empty() {
                            DecisionPromptAction::Status(
                                "Decision title is required; type a title or Esc to cancel."
                                    .to_string(),
                            )
                        } else {
                            *step = DecisionPromptStep::Body;
                            DecisionPromptAction::Status(self.status_line())
                        }
                    }
                    DecisionPromptStep::Body => DecisionPromptAction::Add {
                        title: title.trim().to_string(),
                        body: body.trim().to_string(),
                    },
                },
                KeyCode::Backspace => {
                    match step {
                        DecisionPromptStep::Title => {
                            title.pop();
                        }
                        DecisionPromptStep::Body => {
                            body.pop();
                        }
                    }
                    DecisionPromptAction::None
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    match step {
                        DecisionPromptStep::Title => title.clear(),
                        DecisionPromptStep::Body => body.clear(),
                    }
                    DecisionPromptAction::None
                }
                KeyCode::Char(ch)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    match step {
                        DecisionPromptStep::Title => title.push(ch),
                        DecisionPromptStep::Body => body.push(ch),
                    }
                    DecisionPromptAction::None
                }
                _ => DecisionPromptAction::None,
            },
        }
    }

    fn status_line(&self) -> String {
        match self {
            Self::Add { title, body, step } => match step {
                DecisionPromptStep::Title => format!(
                    "Add decision title: {} · Enter body · Esc cancel",
                    if title.is_empty() { "<title>" } else { title }
                ),
                DecisionPromptStep::Body => format!(
                    "Add decision body (optional one line): {} · Enter create · Esc cancel",
                    if body.is_empty() { "<empty>" } else { body }
                ),
            },
        }
    }

    fn modal_title(&self) -> &'static str {
        match self {
            Self::Add { .. } => " Add decision ",
        }
    }

    fn modal_lines(&self, theme: &TuiTheme) -> Vec<Line<'static>> {
        match self {
            Self::Add { title, body, step } => vec![
                Line::from(Span::styled("Create a basic decision", theme.title_style())),
                Line::from(Span::styled(
                    "Matches `tandem decision add --title <title> --body <markdown>` without references/tags.",
                    theme.muted_style(),
                )),
                Line::from(""),
                prompt_input_line(
                    "Title",
                    title,
                    *step == DecisionPromptStep::Title,
                    theme,
                ),
                prompt_input_line("Body", body, *step == DecisionPromptStep::Body, theme),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter advances/creates · body is optional one line · Esc cancels · Ctrl-U clears field",
                    theme.muted_style(),
                )),
            ],
        }
    }
}

#[derive(Debug)]
struct DecisionMutationOutcome {
    id: String,
    title: String,
}

fn prompt_input_line(label: &str, value: &str, active: bool, theme: &TuiTheme) -> Line<'static> {
    let marker = if active { ">" } else { " " };
    let value = if value.is_empty() { "<empty>" } else { value };
    Line::from(vec![
        Span::styled(format!("{marker} {label}: "), theme.label_style()),
        Span::styled(
            value.to_string(),
            if active {
                theme.text_style().add_modifier(Modifier::BOLD)
            } else {
                theme.text_style()
            },
        ),
    ])
}

fn decision_list_item(doc: &Document, theme: &TuiTheme) -> ListItem<'static> {
    let references = doc.field("references").unwrap_or("");
    let tags = doc.field("tags").unwrap_or("");
    let summary = first_body_line(doc);
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", doc.id()),
                theme.status_style(StatusTone::Accent),
            ),
            Span::styled(
                doc.title().to_string(),
                theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if references.is_empty() {
                    "refs:- ".to_string()
                } else {
                    format!("refs:{references} ")
                },
                theme.muted_style(),
            ),
            Span::styled(
                if tags.is_empty() {
                    "tags:- ".to_string()
                } else {
                    format!("tags:{tags} ")
                },
                theme.muted_style(),
            ),
            Span::styled(crate::truncate(&summary, 44), theme.muted_style()),
        ]),
    ])
}

fn decision_detail_lines(doc: &Document, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme.label_style()),
        Span::styled(doc.title().to_string(), theme.text_style()),
    ]));
    lines.push(detail_field_line("ID", doc.id(), theme));
    lines.push(detail_field_line("Type", doc.doc_type(), theme));
    push_optional_detail_line(&mut lines, "References", doc.field("references"), theme);
    push_optional_detail_line(&mut lines, "Tags", doc.field("tags"), theme);
    push_optional_detail_line(&mut lines, "Created", doc.field("createdAt"), theme);
    push_optional_detail_line(&mut lines, "Updated", doc.field("updatedAt"), theme);
    lines.push(detail_field_line("Path", &display_path(&doc.path), theme));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Body",
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    )));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&doc.body, theme));
    }
    lines
}

fn create_basic_decision(
    workspace: &Workspace,
    title: &str,
    body: &str,
) -> Result<DecisionMutationOutcome, CliError> {
    let title = require_decision_title(title)?;
    let decision_id = next_sequential_id(workspace, "decision")?;
    let now = current_timestamp();
    let decision_path = workspace.board_dir.join(format!("{decision_id}.md"));
    let mut lines = vec![
        "---".to_string(),
        format!("id: {decision_id}"),
        "type: decision".to_string(),
        format!("title: {}", yaml_double_quote(title)),
        format!("createdAt: {}", yaml_double_quote(&now)),
        format!("updatedAt: {}", yaml_double_quote(&now)),
        "---".to_string(),
        String::new(),
    ];
    if !body.trim().is_empty() {
        lines.push(body.trim().to_string());
    }
    lines.push(String::new());
    write_atomic(&decision_path, &lines.join("\n"))?;
    append_event(workspace, "decision.created", &decision_id, title)?;

    Ok(DecisionMutationOutcome {
        id: decision_id,
        title: title.to_string(),
    })
}

fn require_decision_title(value: &str) -> Result<&str, CliError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CliError::usage("decision add requires --title <title>"))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_requires_title_before_body_step() {
        let mut prompt = DecisionPrompt::Add {
            title: String::new(),
            body: String::new(),
            step: DecisionPromptStep::Title,
        };
        let action = prompt.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(
            matches!(action, DecisionPromptAction::Status(message) if message.contains("required"))
        );
    }
}
