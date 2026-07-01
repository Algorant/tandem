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
    centered_rect, detail_field_line, detail_section_heading, is_decision_doc, markdownish_lines,
    FocusPane, TuiApp,
};
use crate::{
    append_event, current_timestamp, date_from_timestamp, display_path, next_sequential_id,
    parse_field_values, write_atomic, yaml_double_quote, CliError, Document, Workspace,
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
                    "selected {} · {} decision document{} loaded",
                    doc.id(),
                    decisions.len(),
                    if decisions.len() == 1 { "" } else { "s" }
                )
            })
            .unwrap_or_else(|| "no decision documents loaded".to_string())
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
                "No decision documents. Press a to add one.",
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
            .filter(|doc| is_decision_doc(doc))
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
                    "Matches `tandem decision add --title <title> --body <markdown>` with default ADR status/date and without references/tags.",
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
    let status = decision_field(doc, DECISION_STATUS_KEYS)
        .map(|status| format!("status:{status}"))
        .unwrap_or_else(|| "status:-".to_string());
    let date = decision_field(doc, DECISION_LIST_DATE_KEYS)
        .map(|date| format!("date:{}", compact_decision_date(date)))
        .unwrap_or_else(|| "date:-".to_string());
    let deciders = formatted_decision_values(doc, DECISION_DECIDER_KEYS, "", ", ")
        .map(|deciders| format!("deciders:{deciders}"))
        .unwrap_or_else(|| "deciders:-".to_string());
    let summary = decision_body_summary(doc);
    let metadata = format!("{status} · {date} · {deciders}");

    let mut metadata_spans = vec![Span::styled(metadata, theme.muted_style())];
    if !summary.is_empty() {
        metadata_spans.push(Span::styled(" — ".to_string(), theme.muted_style()));
        metadata_spans.push(Span::styled(
            crate::truncate(&summary, 48),
            theme.muted_style(),
        ));
    }

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
        Line::from(metadata_spans),
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
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Decision metadata", theme));
    push_optional_decision_scalar_line(&mut lines, "Status", doc, DECISION_STATUS_KEYS, theme);
    push_optional_decision_scalar_line(&mut lines, "Date", doc, DECISION_DATE_KEYS, theme);
    push_optional_decision_list_line(
        &mut lines,
        "Deciders",
        doc,
        DECISION_DECIDER_KEYS,
        "",
        ", ",
        theme,
    );
    push_optional_decision_scalar_line(&mut lines, "Context", doc, &["context"], theme);
    push_optional_decision_list_line(
        &mut lines,
        "Consequences",
        doc,
        &["consequences"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(
        &mut lines,
        "Alternatives",
        doc,
        &["alternatives"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(
        &mut lines,
        "Consulted",
        doc,
        &["consulted"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(&mut lines, "Informed", doc, &["informed"], "", ", ", theme);
    push_optional_decision_list_line(
        &mut lines,
        "Supersedes",
        doc,
        &["supersedes"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(
        &mut lines,
        "Superseded by",
        doc,
        &["supersededBy", "superseded-by"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(
        &mut lines,
        "References",
        doc,
        &["references"],
        "",
        ", ",
        theme,
    );
    push_optional_decision_list_line(&mut lines, "Tags", doc, &["tags"], "#", " ", theme);
    push_optional_decision_scalar_line(&mut lines, "Created", doc, &["createdAt"], theme);
    push_optional_decision_scalar_line(&mut lines, "Updated", doc, &["updatedAt"], theme);
    lines.push(detail_field_line("Path", &display_path(&doc.path), theme));
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Decision record", theme));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled(
            "(empty; recommended sections: Context, Decision, Consequences, Alternatives)",
            theme.muted_style(),
        )));
    } else {
        lines.extend(markdownish_lines(&doc.body, theme));
    }
    lines
}

const DECISION_STATUS_KEYS: &[&str] = &["status", "decisionStatus", "decision.status"];
const DECISION_DATE_KEYS: &[&str] = &["date", "decidedAt", "decision.date"];
const DECISION_LIST_DATE_KEYS: &[&str] = &["date", "decidedAt", "decision.date", "createdAt"];
const DECISION_DECIDER_KEYS: &[&str] = &["deciders", "decisionMakers", "decision.deciders"];

fn decision_field<'a>(doc: &'a Document, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| doc.field(key).filter(|value| !value.trim().is_empty()))
}

fn push_optional_decision_scalar_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    doc: &Document,
    keys: &[&str],
    theme: &TuiTheme,
) {
    if let Some(value) = decision_field(doc, keys) {
        lines.push(detail_field_line(label, value, theme));
    }
}

fn push_optional_decision_list_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    doc: &Document,
    keys: &[&str],
    prefix: &str,
    separator: &str,
    theme: &TuiTheme,
) {
    if let Some(value) = formatted_decision_values(doc, keys, prefix, separator) {
        lines.push(detail_field_line(label, &value, theme));
    }
}

fn formatted_decision_values(
    doc: &Document,
    keys: &[&str],
    prefix: &str,
    separator: &str,
) -> Option<String> {
    let raw = decision_field(doc, keys)?;
    let values = decision_field_values(raw);
    if values.is_empty() {
        return None;
    }
    Some(
        values
            .into_iter()
            .map(|value| format!("{prefix}{value}"))
            .collect::<Vec<_>>()
            .join(separator),
    )
}

fn decision_field_values(raw: &str) -> Vec<String> {
    let mut values = parse_field_values(raw);
    if values.len() == 1 && !raw.trim().starts_with('[') && values[0].contains(',') {
        values = values[0]
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }
    values
}

fn compact_decision_date(value: &str) -> String {
    value.split('T').next().unwrap_or(value).to_string()
}

fn decision_body_summary(doc: &Document) -> String {
    let mut first_heading = None;
    for line in doc.body.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("```") {
            continue;
        }
        if line.starts_with('#') {
            first_heading.get_or_insert_with(|| clean_decision_summary_line(line));
            continue;
        }
        return clean_decision_summary_line(line);
    }
    first_heading.unwrap_or_default()
}

fn clean_decision_summary_line(line: &str) -> String {
    line.trim()
        .trim_start_matches('#')
        .trim_start_matches('>')
        .trim_start_matches('-')
        .trim_start_matches('*')
        .trim()
        .trim_matches('`')
        .to_string()
}

fn create_basic_decision(
    workspace: &Workspace,
    title: &str,
    body: &str,
) -> Result<DecisionMutationOutcome, CliError> {
    let title = require_decision_title(title)?;
    let decision_id = next_sequential_id(workspace, "decision")?;
    let now = current_timestamp();
    let date = date_from_timestamp(&now);
    let decision_path = workspace.board_dir.join(format!("{decision_id}.md"));
    let mut lines = vec![
        "---".to_string(),
        format!("id: {decision_id}"),
        "type: decision".to_string(),
        format!("title: {}", yaml_double_quote(title)),
        "status: \"proposed\"".to_string(),
        format!("date: {}", yaml_double_quote(&date)),
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
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::DocumentLocation;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn adr_decision_doc() -> Document {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), "decision-42".to_string());
        fields.insert("type".to_string(), "decision".to_string());
        fields.insert("title".to_string(), "Choose the TUI layout".to_string());
        fields.insert("status".to_string(), "accepted".to_string());
        fields.insert("date".to_string(), "2026-07-01".to_string());
        fields.insert("deciders".to_string(), "[\"Ada\", \"Grace\"]".to_string());
        fields.insert("supersedes".to_string(), "[\"decision-7\"]".to_string());
        fields.insert("supersededBy".to_string(), "decision-99".to_string());
        fields.insert(
            "references".to_string(),
            "[\"task-1\", \"task-2\"]".to_string(),
        );
        fields.insert("tags".to_string(), "[\"adr\", \"tui\"]".to_string());
        fields.insert("state".to_string(), "todo".to_string());
        fields.insert("createdAt".to_string(), "2026-07-01T10:00:00Z".to_string());
        fields.insert("updatedAt".to_string(), "2026-07-01T11:00:00Z".to_string());
        Document {
            path: PathBuf::from(".tandem/board/decision-42.md"),
            location: DocumentLocation::Board,
            fields,
            body: "## Context\n\nThe Board is noisy when decisions are mixed into task state buckets.\n\n## Decision\n\nRender decisions in their own pane.".to_string(),
        }
    }

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

    #[test]
    fn decision_detail_renders_adr_metadata_without_workflow_state() {
        let theme = TuiTheme::default_dark();
        let doc = adr_decision_doc();
        let texts = decision_detail_lines(&doc, &theme)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();

        assert!(texts.contains(&"Decision metadata".to_string()));
        assert!(texts.contains(&"Status: accepted".to_string()));
        assert!(texts.contains(&"Date: 2026-07-01".to_string()));
        assert!(texts.contains(&"Deciders: Ada, Grace".to_string()));
        assert!(texts.contains(&"Supersedes: decision-7".to_string()));
        assert!(texts.contains(&"Superseded by: decision-99".to_string()));
        assert!(texts.contains(&"References: task-1, task-2".to_string()));
        assert!(texts.contains(&"Tags: #adr #tui".to_string()));
        assert!(texts.contains(&"Decision record".to_string()));
        assert!(texts.contains(&"Context".to_string()));
        assert!(texts.contains(&"Decision".to_string()));
        assert!(!texts.contains(&"State: todo".to_string()));
    }

    #[test]
    fn decision_body_summary_skips_adr_section_headings() {
        let doc = adr_decision_doc();
        assert_eq!(
            decision_body_summary(&doc),
            "The Board is noisy when decisions are mixed into task state buckets."
        );
    }
}
