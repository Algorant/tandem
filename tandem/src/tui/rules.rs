use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::theme::{StatusTone, TuiTheme};
use super::{centered_rect, detail_field_line, push_optional_detail_line, TuiApp};
use crate::{
    append_event, display_path, document_exists, ensure_file_unchanged, parse_rules_from_content,
    patch_rules_category_content, read_file_snapshot, validate_rule_category, write_atomic,
    CliError, RuleItem, RulesByCategory, Workspace,
};

const RULE_CATEGORIES: [&str; 4] = ["always", "never", "prefer", "context"];

#[derive(Debug, Default)]
pub(super) struct RulesState {
    selected_category: usize,
    selected_item: usize,
    prompt: Option<RulePrompt>,
}

impl RulesState {
    fn clamp(&mut self, rules: &RulesByCategory) {
        if self.selected_category >= RULE_CATEGORIES.len() {
            self.selected_category = RULE_CATEGORIES.len().saturating_sub(1);
        }
        let count = rules
            .get(RULE_CATEGORIES[self.selected_category])
            .map(Vec::len)
            .unwrap_or(0);
        if count == 0 {
            self.selected_item = 0;
        } else if self.selected_item >= count {
            self.selected_item = count - 1;
        }
    }

    pub(super) fn has_prompt(&self) -> bool {
        self.prompt.is_some()
    }
}

#[derive(Debug, Clone)]
enum RulePrompt {
    Text {
        mode: RulePromptMode,
        category: String,
        id: Option<usize>,
        rule: String,
        source: String,
        step: RulePromptStep,
    },
    Delete {
        category: String,
        id: usize,
        rule: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RulePromptMode {
    Add,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RulePromptStep {
    Rule,
    Source,
}

#[derive(Debug)]
enum RulePromptAction {
    None,
    Status(String),
    Cancel(String),
    Add {
        category: String,
        rule: String,
        source: String,
    },
    Edit {
        category: String,
        id: usize,
        rule: String,
        source: String,
    },
    Delete {
        category: String,
        id: usize,
    },
}

impl TuiApp {
    pub(super) fn clamp_rules_state(&mut self) {
        self.rules_view.clamp(&self.rules);
    }

    pub(super) fn rules_prompt_active(&self) -> bool {
        self.rules_view.has_prompt()
    }

    pub(super) fn selected_rule_anchor_for_reload(&self) -> Option<(String, Option<usize>)> {
        Some((
            self.selected_rule_category().to_string(),
            self.selected_rule().map(|(_, rule)| rule.id),
        ))
    }

    pub(super) fn restore_rule_selection_after_reload(
        &mut self,
        anchor: Option<(String, Option<usize>)>,
    ) {
        let Some((category, id)) = anchor else {
            self.rules_view.clamp(&self.rules);
            return;
        };
        let restored = id
            .map(|id| self.select_rule_by_id(&category, id))
            .unwrap_or(false);
        if !restored {
            self.select_rule_category(&category);
        }
        self.rules_view.clamp(&self.rules);
    }

    pub(super) fn rules_prompt_status(&self) -> Option<String> {
        self.rules_view.prompt.as_ref().map(RulePrompt::status_line)
    }

    pub(super) fn handle_rules_prompt_key(&mut self, key: KeyEvent) {
        let action = match self.rules_view.prompt.as_mut() {
            Some(prompt) => prompt.handle_key(key),
            None => RulePromptAction::None,
        };

        match action {
            RulePromptAction::None => {
                if let Some(status) = self.rules_prompt_status() {
                    self.status = status;
                }
            }
            RulePromptAction::Status(status) => self.status = status,
            RulePromptAction::Cancel(status) => {
                self.rules_view.prompt = None;
                self.status = status;
            }
            RulePromptAction::Add {
                category,
                rule,
                source,
            } => {
                self.rules_view.prompt = None;
                self.finish_rule_add(category, rule, source);
            }
            RulePromptAction::Edit {
                category,
                id,
                rule,
                source,
            } => {
                self.rules_view.prompt = None;
                self.finish_rule_edit(category, id, rule, source);
            }
            RulePromptAction::Delete { category, id } => {
                self.rules_view.prompt = None;
                self.finish_rule_delete(category, id);
            }
        }
    }

    pub(super) fn handle_rules_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.previous_rule_category(),
            KeyCode::Right | KeyCode::Char('l') => self.next_rule_category(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_rule_selection(),
            KeyCode::Down | KeyCode::Char('j') => self.next_rule_selection(),
            KeyCode::Home | KeyCode::Char('g') => self.first_rule_selection(),
            KeyCode::End | KeyCode::Char('G') => self.last_rule_selection(),
            KeyCode::Char('a') => self.start_rule_add_prompt(),
            KeyCode::Char('e') => self.start_rule_edit_prompt(),
            KeyCode::Char('d') => self.start_rule_delete_prompt(),
            KeyCode::Enter => {
                self.status = self
                    .selected_rule()
                    .map(|(category, rule)| {
                        format!(
                            "Selected {category} #{}; press e to edit or d to delete.",
                            rule.id
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "{} has no selected rule; press a to add one.",
                            self.selected_rule_category()
                        )
                    });
            }
            _ => {}
        }
    }

    pub(super) fn previous_rule_selection(&mut self) {
        self.move_rule_selection(-1);
    }

    pub(super) fn next_rule_selection(&mut self) {
        self.move_rule_selection(1);
    }

    pub(super) fn draw_rules_view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.rules_view.clamp(&self.rules);
        let chunks = if area.width >= 92 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(area)
        };
        self.draw_rules_list(frame, chunks[0]);
        self.draw_rule_detail(frame, chunks[1]);
    }

    pub(super) fn draw_rules_prompt(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(prompt) = self.rules_view.prompt.as_ref() else {
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

    pub(super) fn rules_context(&self) -> String {
        let total = self.rules_total();
        self.selected_rule()
            .map(|(category, rule)| {
                format!(
                    "selected {category} #{} · {} project rule{} loaded",
                    rule.id,
                    total,
                    if total == 1 { "" } else { "s" }
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} has no selected rules · {} project rule{} loaded",
                    self.selected_rule_category(),
                    total,
                    if total == 1 { "" } else { "s" }
                )
            })
    }

    pub(super) fn rules_footer_text(&self) -> String {
        self.with_status("Rules · h/l category · a add · e/d edit/delete · ? help".to_string())
    }

    fn draw_rules_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let rows = self.rule_display_rows();
        let selected_row = self.selected_rule_row_index(&rows);
        let items = rows
            .iter()
            .map(|row| ListItem::new(row.line.clone()))
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(selected_row);
        let list = List::new(items)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Rules grouped by category · {} selected ",
                        self.selected_rule_category()
                    ))
                    .border_style(self.theme.border_style(true))
                    .style(self.theme.panel_style()),
            )
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn draw_rule_detail(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = Vec::new();
        if let Some((category, rule)) = self.selected_rule() {
            lines.push(detail_field_line("Category", category, &self.theme));
            lines.push(detail_field_line("ID", &rule.id.to_string(), &self.theme));
            lines.push(Line::from(vec![
                Span::styled("Rule: ", self.theme.label_style()),
                Span::styled(rule.rule.clone(), self.theme.text_style()),
            ]));
            push_optional_detail_line(&mut lines, "Source", rule.source.as_deref(), &self.theme);
            lines.push(detail_field_line(
                "Config",
                &display_path(&self.workspace.config_path),
                &self.theme,
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Actions: a add · e edit selected · d delete selected",
                self.theme.status_style(StatusTone::Accent),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{} rules", self.selected_rule_category()),
                self.theme.title_style(),
            )));
            lines.push(Line::from(Span::styled(
                "No rules in this category.",
                self.theme.muted_style(),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press a to add a rule to the selected category.",
                self.theme.status_style(StatusTone::Accent),
            )));
            lines.push(detail_field_line(
                "Config",
                &display_path(&self.workspace.config_path),
                &self.theme,
            ));
        }
        let detail = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Rule detail ")
                    .border_style(self.theme.border_style(false))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }

    fn selected_rule_category(&self) -> &'static str {
        RULE_CATEGORIES[self
            .rules_view
            .selected_category
            .min(RULE_CATEGORIES.len().saturating_sub(1))]
    }

    fn selected_rule(&self) -> Option<(&'static str, &RuleItem)> {
        let category = self.selected_rule_category();
        let items = self.rules.get(category)?;
        let rule = items.get(self.rules_view.selected_item)?;
        Some((category, rule))
    }

    fn previous_rule_category(&mut self) {
        if self.rules_view.selected_category > 0 {
            self.rules_view.selected_category -= 1;
            self.rules_view.selected_item = 0;
            self.rules_view.clamp(&self.rules);
        }
    }

    fn next_rule_category(&mut self) {
        if self.rules_view.selected_category + 1 < RULE_CATEGORIES.len() {
            self.rules_view.selected_category += 1;
            self.rules_view.selected_item = 0;
            self.rules_view.clamp(&self.rules);
        }
    }

    fn move_rule_selection(&mut self, delta: isize) {
        let positions = self.rule_positions();
        if positions.is_empty() {
            return;
        }
        let current = self
            .current_rule_position_index(&positions)
            .unwrap_or_else(|| {
                nearest_rule_position(&positions, self.rules_view.selected_category)
            });
        let next = (current as isize + delta).clamp(0, positions.len().saturating_sub(1) as isize);
        let (category, item) = positions[next as usize];
        self.rules_view.selected_category = category;
        self.rules_view.selected_item = item;
    }

    fn first_rule_selection(&mut self) {
        if let Some((category, item)) = self.rule_positions().first().copied() {
            self.rules_view.selected_category = category;
            self.rules_view.selected_item = item;
        }
    }

    fn last_rule_selection(&mut self) {
        if let Some((category, item)) = self.rule_positions().last().copied() {
            self.rules_view.selected_category = category;
            self.rules_view.selected_item = item;
        }
    }

    fn rule_positions(&self) -> Vec<(usize, usize)> {
        RULE_CATEGORIES
            .iter()
            .enumerate()
            .flat_map(|(category_index, category)| {
                let count = self.rules.get(*category).map(Vec::len).unwrap_or(0);
                (0..count).map(move |item_index| (category_index, item_index))
            })
            .collect()
    }

    fn current_rule_position_index(&self, positions: &[(usize, usize)]) -> Option<usize> {
        positions.iter().position(|(category, item)| {
            *category == self.rules_view.selected_category && *item == self.rules_view.selected_item
        })
    }

    fn rule_display_rows(&self) -> Vec<RuleDisplayRow> {
        let mut rows = Vec::new();
        for (category_index, category) in RULE_CATEGORIES.iter().enumerate() {
            let items = self.rules.get(*category).map(Vec::as_slice).unwrap_or(&[]);
            rows.push(RuleDisplayRow {
                category_index,
                item_index: None,
                empty_marker: false,
                line: Line::from(vec![
                    Span::styled(
                        format!("{category} "),
                        self.theme.title_style().add_modifier(
                            if category_index == self.rules_view.selected_category {
                                Modifier::UNDERLINED
                            } else {
                                Modifier::empty()
                            },
                        ),
                    ),
                    Span::styled(
                        format!(
                            "({} rule{})",
                            items.len(),
                            if items.len() == 1 { "" } else { "s" }
                        ),
                        self.theme.muted_style(),
                    ),
                ]),
            });
            if items.is_empty() {
                rows.push(RuleDisplayRow {
                    category_index,
                    item_index: None,
                    empty_marker: true,
                    line: Line::from(Span::styled("  (none)", self.theme.muted_style())),
                });
            } else {
                for (item_index, item) in items.iter().enumerate() {
                    let source = item
                        .source
                        .as_ref()
                        .map(|source| format!(" · source {source}"))
                        .unwrap_or_default();
                    rows.push(RuleDisplayRow {
                        category_index,
                        item_index: Some(item_index),
                        empty_marker: false,
                        line: Line::from(vec![
                            Span::styled(
                                format!("  #{} ", item.id),
                                self.theme.status_style(StatusTone::Accent),
                            ),
                            Span::styled(crate::truncate(&item.rule, 72), self.theme.text_style()),
                            Span::styled(source, self.theme.muted_style()),
                        ]),
                    });
                }
            }
        }
        rows
    }

    fn selected_rule_row_index(&self, rows: &[RuleDisplayRow]) -> Option<usize> {
        rows.iter().position(|row| {
            row.category_index == self.rules_view.selected_category
                && match row.item_index {
                    Some(item_index) => item_index == self.rules_view.selected_item,
                    None => row.empty_marker,
                }
        })
    }

    pub(super) fn start_rule_add_prompt(&mut self) {
        let category = self.selected_rule_category().to_string();
        self.rules_view.prompt = Some(RulePrompt::Text {
            mode: RulePromptMode::Add,
            category,
            id: None,
            rule: String::new(),
            source: String::new(),
            step: RulePromptStep::Rule,
        });
        if let Some(status) = self.rules_prompt_status() {
            self.status = status;
        }
    }

    fn start_rule_edit_prompt(&mut self) {
        let Some((category, rule)) = self
            .selected_rule()
            .map(|(category, rule)| (category.to_string(), rule.clone()))
        else {
            self.status = "No selected rule to edit; press a to add one.".to_string();
            return;
        };
        self.rules_view.prompt = Some(RulePrompt::Text {
            mode: RulePromptMode::Edit,
            category,
            id: Some(rule.id),
            rule: rule.rule,
            source: rule.source.unwrap_or_default(),
            step: RulePromptStep::Rule,
        });
        if let Some(status) = self.rules_prompt_status() {
            self.status = status;
        }
    }

    fn start_rule_delete_prompt(&mut self) {
        let Some((category, rule)) = self
            .selected_rule()
            .map(|(category, rule)| (category.to_string(), rule.clone()))
        else {
            self.status = "No selected rule to delete.".to_string();
            return;
        };
        self.rules_view.prompt = Some(RulePrompt::Delete {
            category,
            id: rule.id,
            rule: rule.rule,
        });
        if let Some(status) = self.rules_prompt_status() {
            self.status = status;
        }
    }

    fn finish_rule_add(&mut self, category: String, rule: String, source: String) {
        match add_rule_to_workspace(&self.workspace, &category, &rule, &source) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_rule_by_id(&outcome.category, outcome.id);
                self.status = format_rule_outcome("Added", &outcome, &reload_note);
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Rule add error: {}{}", error.message, reload_note);
            }
        }
    }

    fn finish_rule_edit(&mut self, category: String, id: usize, rule: String, source: String) {
        match edit_rule_in_workspace(&self.workspace, &category, id, &rule, &source) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_rule_by_id(&outcome.category, outcome.id);
                self.status = format_rule_outcome("Edited", &outcome, &reload_note);
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Rule edit error: {}{}", error.message, reload_note);
            }
        }
    }

    fn finish_rule_delete(&mut self, category: String, id: usize) {
        match delete_rule_from_workspace(&self.workspace, &category, id) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_rule_category(&outcome.category);
                self.rules_view.clamp(&self.rules);
                self.status = format!(
                    "Deleted {} #{}{}",
                    outcome.category, outcome.id, reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Rule delete error: {}{}", error.message, reload_note);
            }
        }
    }

    fn select_rule_by_id(&mut self, category: &str, id: usize) -> bool {
        let Some(category_index) = RULE_CATEGORIES
            .iter()
            .position(|candidate| *candidate == category)
        else {
            return false;
        };
        self.rules_view.selected_category = category_index;
        if let Some(item_index) = self
            .rules
            .get(category)
            .and_then(|items| items.iter().position(|item| item.id == id))
        {
            self.rules_view.selected_item = item_index;
            true
        } else {
            self.rules_view.clamp(&self.rules);
            false
        }
    }

    fn select_rule_category(&mut self, category: &str) -> bool {
        if let Some(category_index) = RULE_CATEGORIES
            .iter()
            .position(|candidate| *candidate == category)
        {
            self.rules_view.selected_category = category_index;
            self.rules_view.selected_item = 0;
            true
        } else {
            false
        }
    }
}

impl RulePrompt {
    fn handle_key(&mut self, key: KeyEvent) -> RulePromptAction {
        match self {
            Self::Delete { category, id, .. } => match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    RulePromptAction::Cancel("Rule delete canceled.".to_string())
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    RulePromptAction::Delete {
                        category: category.clone(),
                        id: *id,
                    }
                }
                _ => RulePromptAction::None,
            },
            Self::Text {
                mode,
                category,
                id,
                rule,
                source,
                step,
            } => match key.code {
                KeyCode::Esc => RulePromptAction::Cancel("Rule prompt canceled.".to_string()),
                KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => match step {
                    RulePromptStep::Rule => {
                        if rule.trim().is_empty() {
                            RulePromptAction::Status(
                                "Rule text is required; type a rule or Esc to cancel.".to_string(),
                            )
                        } else {
                            *step = RulePromptStep::Source;
                            RulePromptAction::Status(self.status_line())
                        }
                    }
                    RulePromptStep::Source => match mode {
                        RulePromptMode::Add => RulePromptAction::Add {
                            category: category.clone(),
                            rule: rule.trim().to_string(),
                            source: source.trim().to_string(),
                        },
                        RulePromptMode::Edit => RulePromptAction::Edit {
                            category: category.clone(),
                            id: id.unwrap_or_default(),
                            rule: rule.trim().to_string(),
                            source: source.trim().to_string(),
                        },
                    },
                },
                KeyCode::Backspace => {
                    match step {
                        RulePromptStep::Rule => {
                            rule.pop();
                        }
                        RulePromptStep::Source => {
                            source.pop();
                        }
                    }
                    RulePromptAction::None
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    match step {
                        RulePromptStep::Rule => rule.clear(),
                        RulePromptStep::Source => source.clear(),
                    }
                    RulePromptAction::None
                }
                KeyCode::Char(ch)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    match step {
                        RulePromptStep::Rule => rule.push(ch),
                        RulePromptStep::Source => source.push(ch),
                    }
                    RulePromptAction::None
                }
                _ => RulePromptAction::None,
            },
        }
    }

    fn status_line(&self) -> String {
        match self {
            Self::Text {
                mode,
                category,
                id,
                rule,
                source,
                step,
            } => {
                let action = match mode {
                    RulePromptMode::Add => "Add rule",
                    RulePromptMode::Edit => "Edit rule",
                };
                let id = id.map(|id| format!(" #{id}")).unwrap_or_default();
                match step {
                    RulePromptStep::Rule => format!(
                        "{action}{id} in {category}: {} · Enter source · Esc cancel",
                        if rule.is_empty() { "<rule>" } else { rule }
                    ),
                    RulePromptStep::Source => format!(
                        "{action}{id} in {category} source (optional): {} · Enter save · Esc cancel",
                        if source.is_empty() { "<none>" } else { source }
                    ),
                }
            }
            Self::Delete { category, id, .. } => {
                format!("Delete {category} #{id}? Press y/Enter to delete, n/Esc to cancel")
            }
        }
    }

    fn modal_title(&self) -> &'static str {
        match self {
            Self::Text {
                mode: RulePromptMode::Add,
                ..
            } => " Add rule ",
            Self::Text {
                mode: RulePromptMode::Edit,
                ..
            } => " Edit rule ",
            Self::Delete { .. } => " Delete rule ",
        }
    }

    fn modal_lines(&self, theme: &TuiTheme) -> Vec<Line<'static>> {
        match self {
            Self::Text {
                mode,
                category,
                id,
                rule,
                source,
                step,
            } => {
                let action = match mode {
                    RulePromptMode::Add => "Add a project rule",
                    RulePromptMode::Edit => "Edit the selected project rule",
                };
                let mut lines = vec![
                    Line::from(Span::styled(action, theme.title_style())),
                    detail_field_line("Category", category, theme),
                ];
                if let Some(id) = id {
                    lines.push(detail_field_line("ID", &id.to_string(), theme));
                }
                lines.push(Line::from(""));
                lines.push(prompt_input_line(
                    "Rule",
                    rule,
                    *step == RulePromptStep::Rule,
                    theme,
                ));
                lines.push(prompt_input_line(
                    "Source",
                    source,
                    *step == RulePromptStep::Source,
                    theme,
                ));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Enter advances/saves · source is optional · Esc cancels · Ctrl-U clears field",
                    theme.muted_style(),
                )));
                lines
            }
            Self::Delete { category, id, rule } => vec![
                Line::from(Span::styled(
                    "Confirm destructive rule delete",
                    theme
                        .status_style(StatusTone::Warning)
                        .add_modifier(Modifier::BOLD),
                )),
                detail_field_line("Category", category, theme),
                detail_field_line("ID", &id.to_string(), theme),
                Line::from(vec![
                    Span::styled("Rule: ", theme.label_style()),
                    Span::styled(rule.clone(), theme.text_style()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Press y or Enter to delete. Press n or Esc to cancel.",
                    theme.status_style(StatusTone::Warning),
                )),
            ],
        }
    }
}

#[derive(Debug)]
struct RuleDisplayRow {
    category_index: usize,
    item_index: Option<usize>,
    empty_marker: bool,
    line: Line<'static>,
}

#[derive(Debug)]
struct RuleMutationOutcome {
    category: String,
    id: usize,
    rule: String,
    warning: Option<String>,
}

#[derive(Debug)]
struct RuleDeleteOutcome {
    category: String,
    id: usize,
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

fn nearest_rule_position(positions: &[(usize, usize)], category_index: usize) -> usize {
    positions
        .iter()
        .position(|(category, _)| *category >= category_index)
        .unwrap_or_else(|| positions.len().saturating_sub(1))
}

fn add_rule_to_workspace(
    workspace: &Workspace,
    category: &str,
    rule: &str,
    source: &str,
) -> Result<RuleMutationOutcome, CliError> {
    validate_rule_category(category)?;
    let rule = require_rule_text(rule, "rules add requires --rule <text>")?;
    let source = optional_rule_source(source);
    let warning = missing_rule_source_warning(workspace, source.as_deref())?;

    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let next_id = rules
        .get(category)
        .into_iter()
        .flatten()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    rules
        .entry(category.to_string())
        .or_default()
        .push(RuleItem {
            id: next_id,
            rule: rule.to_string(),
            source,
        });
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        workspace,
        "rules.updated",
        "rules",
        &format!("Added rule {next_id} to {category}"),
    )?;

    Ok(RuleMutationOutcome {
        category: category.to_string(),
        id: next_id,
        rule: rule.to_string(),
        warning,
    })
}

fn edit_rule_in_workspace(
    workspace: &Workspace,
    category: &str,
    id: usize,
    rule: &str,
    source: &str,
) -> Result<RuleMutationOutcome, CliError> {
    validate_rule_category(category)?;
    let rule = require_rule_text(rule, "rules edit requires --rule <text>")?;
    let source = optional_rule_source(source);
    let warning = missing_rule_source_warning(workspace, source.as_deref())?;

    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let items = rules.entry(category.to_string()).or_default();
    let item = items
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| CliError::user(format!("rule not found: {category} #{id}")))?;
    item.rule = rule.to_string();
    item.source = source;
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        workspace,
        "rules.updated",
        "rules",
        &format!("Edited rule {id} in {category}"),
    )?;

    Ok(RuleMutationOutcome {
        category: category.to_string(),
        id,
        rule: rule.to_string(),
        warning,
    })
}

fn delete_rule_from_workspace(
    workspace: &Workspace,
    category: &str,
    id: usize,
) -> Result<RuleDeleteOutcome, CliError> {
    validate_rule_category(category)?;
    let (content, signature) = read_file_snapshot(&workspace.config_path)?;
    let mut rules = parse_rules_from_content(&content, &workspace.config_path)?;
    let items = rules.entry(category.to_string()).or_default();
    let before_len = items.len();
    items.retain(|item| item.id != id);
    if items.len() == before_len {
        return Err(CliError::user(format!("rule not found: {category} #{id}")));
    }
    let patched = patch_rules_category_content(&content, category, &rules)?;
    ensure_file_unchanged(&workspace.config_path, &signature)?;
    write_atomic(&workspace.config_path, &patched)?;
    append_event(
        workspace,
        "rules.updated",
        "rules",
        &format!("Deleted rule {id} from {category}"),
    )?;

    Ok(RuleDeleteOutcome {
        category: category.to_string(),
        id,
    })
}

fn require_rule_text<'a>(value: &'a str, message: &str) -> Result<&'a str, CliError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CliError::usage(message))
    } else {
        Ok(value)
    }
}

fn optional_rule_source(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn missing_rule_source_warning(
    workspace: &Workspace,
    source: Option<&str>,
) -> Result<Option<String>, CliError> {
    if let Some(source) = source.filter(|source| !source.trim().is_empty()) {
        if !document_exists(workspace, source)? {
            return Ok(Some(format!("rule source not found: {source}")));
        }
    }
    Ok(None)
}

fn format_rule_outcome(verb: &str, outcome: &RuleMutationOutcome, reload_note: &str) -> String {
    format!(
        "{verb} {} #{}: {}{}{}",
        outcome.category,
        outcome.id,
        outcome.rule,
        outcome
            .warning
            .as_ref()
            .map(|warning| format!("; warning: {warning}"))
            .unwrap_or_default(),
        reload_note
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rule_position_prefers_current_or_next_category() {
        let positions = vec![(0, 0), (2, 0), (2, 1), (3, 0)];
        assert_eq!(nearest_rule_position(&positions, 0), 0);
        assert_eq!(nearest_rule_position(&positions, 1), 1);
        assert_eq!(nearest_rule_position(&positions, 3), 3);
        assert_eq!(nearest_rule_position(&positions, 4), 3);
    }

    #[test]
    fn prompt_requires_rule_before_source_step() {
        let mut prompt = RulePrompt::Text {
            mode: RulePromptMode::Add,
            category: "always".to_string(),
            id: None,
            rule: String::new(),
            source: String::new(),
            step: RulePromptStep::Rule,
        };
        let action = prompt.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(
            matches!(action, RulePromptAction::Status(message) if message.contains("required"))
        );
    }
}
