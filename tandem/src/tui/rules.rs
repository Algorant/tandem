use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::board::wrap_words;
use super::theme::{StatusTone, TuiTheme};
use super::{centered_rect, detail_field_line, HitAction, HitRegion, TuiApp};
use crate::app;
use crate::protocol::config::{RuleItem, RulesByCategory};

const RULE_CATEGORIES: [&str; 4] = ["always", "never", "prefer", "context"];
const MIN_RULE_LIST_HEIGHT: u16 = 3;
const MIN_RULE_PREVIEW_HEIGHT: u16 = 5;

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn rule_category_tab_width(index: usize, category: &str, count: usize) -> u16 {
    format!("[{}]  {} ({count}) ", index + 1, title_case(category))
        .chars()
        .count() as u16
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum RuleFocus {
    #[default]
    List,
    Preview,
}

#[derive(Debug, Default)]
pub(super) struct RulesState {
    pub(super) selected_category: usize,
    pub(super) selected_item: usize,
    pub(super) list_offset: usize,
    pub(super) preview_open: bool,
    pub(super) preview_scroll: u16,
    preview_max_scroll: u16,
    pub(super) focus: RuleFocus,
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
        self.clamp_rule_preview_scroll();
    }

    pub(super) fn rules_prompt_active(&self) -> bool {
        self.rules_view.has_prompt()
    }

    pub(super) fn rules_text_prompt_active(&self) -> bool {
        matches!(self.rules_view.prompt, Some(RulePrompt::Text { .. }))
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
            self.clamp_rules_state();
            return;
        };
        let restored = id
            .map(|id| self.select_rule_by_id(&category, id))
            .unwrap_or(false);
        if !restored {
            self.select_rule_category(&category);
        }
        self.clamp_rules_state();
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
        let preview_focused =
            self.rules_view.focus == RuleFocus::Preview && self.rules_view.preview_open;
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if preview_focused => self.focus_rule_list(),
            KeyCode::Right | KeyCode::Char('l') if self.rules_view.preview_open => {
                self.focus_rule_preview()
            }
            KeyCode::Left | KeyCode::Char('h') => self.previous_rule_category(),
            KeyCode::Right | KeyCode::Char('l') => self.next_rule_category(),
            KeyCode::Up | KeyCode::Char('k') if preview_focused => self.scroll_rule_preview_up(1),
            KeyCode::Down | KeyCode::Char('j') if preview_focused => {
                self.scroll_rule_preview_down(1)
            }
            KeyCode::Up | KeyCode::Char('k') => self.previous_rule_selection(),
            KeyCode::Down | KeyCode::Char('j') => self.next_rule_selection(),
            KeyCode::Home | KeyCode::Char('g') if preview_focused => {
                self.rules_view.preview_scroll = 0
            }
            KeyCode::End | KeyCode::Char('G') if preview_focused => {
                self.rules_view.preview_scroll = u16::MAX
            }
            KeyCode::Home | KeyCode::Char('g') => self.first_rule_selection(),
            KeyCode::End | KeyCode::Char('G') => self.last_rule_selection(),
            KeyCode::PageUp if preview_focused => self.scroll_rule_preview_up(6),
            KeyCode::PageDown if preview_focused => self.scroll_rule_preview_down(6),
            KeyCode::PageUp => self.move_rule_selection(-5),
            KeyCode::PageDown => self.move_rule_selection(5),
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) && preview_focused =>
            {
                self.scroll_rule_preview_up(6)
            }
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) && preview_focused =>
            {
                self.scroll_rule_preview_down(6)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_rule_selection(-5)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_rule_selection(5)
            }
            KeyCode::Char('a') => self.start_rule_add_prompt(),
            KeyCode::Char('e') => self.start_rule_edit_prompt(),
            KeyCode::Char('d') => self.start_rule_delete_prompt(),
            KeyCode::Enter => {
                if self.selected_rule().is_some() {
                    self.rules_view.preview_open = !self.rules_view.preview_open;
                    self.rules_view.preview_scroll = 0;
                    if !self.rules_view.preview_open {
                        self.rules_view.focus = RuleFocus::List;
                    }
                    self.status = if self.rules_view.preview_open {
                        "Rule preview opened; selection changes update it.".to_string()
                    } else {
                        "Rule preview closed.".to_string()
                    };
                } else {
                    self.status = format!(
                        "No {} rules defined. Press a to add one.",
                        self.selected_rule_category()
                    );
                }
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

    pub(super) fn focus_rule_list(&mut self) {
        self.rules_view.focus = RuleFocus::List;
    }

    pub(super) fn focus_rule_preview(&mut self) {
        if self.rules_view.preview_open {
            self.rules_view.focus = RuleFocus::Preview;
        }
    }

    pub(super) fn scroll_rule_preview_up(&mut self, amount: u16) {
        self.rules_view.preview_scroll = self.rules_view.preview_scroll.saturating_sub(amount);
    }

    pub(super) fn scroll_rule_preview_down(&mut self, amount: u16) {
        self.rules_view.preview_scroll = self
            .rules_view
            .preview_scroll
            .saturating_add(amount)
            .min(self.rules_view.preview_max_scroll);
    }

    fn clamp_rule_preview_scroll(&mut self) {
        self.rules_view.preview_scroll = self
            .rules_view
            .preview_scroll
            .min(self.rules_view.preview_max_scroll);
        if !self.rules_view.preview_open {
            self.rules_view.focus = RuleFocus::List;
        }
    }

    pub(super) fn draw_rules_view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.clamp_rules_state();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(3)])
            .split(area);
        self.draw_rule_category_tabs(frame, chunks[0]);
        let content = chunks[1];
        let has_selected_rule = self.selected_rule().is_some();
        let visible_rule_rows = self
            .rules
            .get(self.selected_rule_category())
            .map_or(1, |rules| rules.len().max(1));
        let (list, preview) = rule_view_layout(
            content,
            self.rules_view.preview_open && has_selected_rule,
            visible_rule_rows,
        );
        self.draw_rules_list(frame, list);
        if let Some(preview) = preview {
            self.draw_rule_preview(frame, preview);
        }
    }

    pub(super) fn draw_rules_prompt(&mut self, frame: &mut Frame<'_>, area: Rect) {
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
            Paragraph::new("[ Confirm / next ]").style(self.theme.status_style(StatusTone::Accent)),
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
        let controls = if self.rules_view.focus == RuleFocus::Preview
            && self.rules_view.preview_open
        {
            "Rules preview · j/k scroll · Ctrl-U/D page · Shift-Tab list · Enter close · ? help"
        } else if self.rules_view.preview_open {
            "Rules list · j/k select · Tab preview · Enter close · a add · e edit · d delete · ? help"
        } else {
            "Rules list · h/l category · j/k select · Enter preview · a add · e edit · d delete · ? help"
        };
        self.with_status(controls.to_string())
    }

    fn draw_rule_category_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let tab_widths = RULE_CATEGORIES
            .iter()
            .enumerate()
            .map(|(index, category)| {
                let count = self.rules.get(*category).map(Vec::len).unwrap_or(0);
                rule_category_tab_width(index, category, count)
            })
            .collect::<Vec<_>>();
        let content_width: u16 = tab_widths.iter().sum();
        let gaps = RULE_CATEGORIES.len().saturating_sub(1) as u16;
        let gap_width = area
            .width
            .saturating_sub(content_width)
            .checked_div(gaps)
            .map_or(0, |gap_width| gap_width.clamp(4, 10));
        let total_width = content_width.saturating_add(gap_width.saturating_mul(gaps));
        let leading = area.width.saturating_sub(total_width) / 2;

        let mut spans = Vec::new();
        if leading > 0 {
            spans.push(Span::raw(" ".repeat(leading as usize)));
        }
        let mut x = area.x.saturating_add(leading);
        for (index, category) in RULE_CATEGORIES.iter().enumerate() {
            if index > 0 {
                spans.push(Span::raw(" ".repeat(gap_width as usize)));
                x = x.saturating_add(gap_width);
            }
            let count = self.rules.get(*category).map(Vec::len).unwrap_or(0);
            let selected = index == self.rules_view.selected_category;
            spans.push(Span::styled(
                format!("[{}] ", index + 1),
                self.theme.muted_style(),
            ));
            spans.push(Span::styled(
                format!(" {} ({count}) ", title_case(category)),
                self.theme.rule_category_style(category, selected),
            ));
            self.hits.push(HitRegion {
                rect: Rect::new(x, area.y, tab_widths[index], 1),
                action: HitAction::SelectRuleCategory(index),
            });
            x = x.saturating_add(tab_widths[index]);
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(self.theme.panel_style()),
            area,
        );
    }

    fn draw_rules_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusRuleList,
        });
        let category = self.selected_rule_category();
        let list_area = Block::default().borders(Borders::ALL).inner(area);
        let row_width = list_area.width.max(12) as usize;
        let rows = self.rule_display_rows(row_width);
        let selected_row = self.selected_rule_row_index(&rows);
        let items = rows
            .iter()
            .map(|row| ListItem::new(row.lines.clone()))
            .collect::<Vec<_>>();
        let mut state = ListState::default().with_offset(self.rules_view.list_offset);
        state.select(selected_row);
        let list = List::new(items)
            .style(self.theme.panel_style())
            .highlight_style(self.theme.rule_selected_row_style())
            .block(rule_list_block(
                &self.theme,
                category,
                self.rules_view.focus == RuleFocus::List,
            ));
        frame.render_stateful_widget(list, area, &mut state);
        self.rules_view.list_offset = state.offset();
        self.register_rule_row_hits(list_area, &rows);
    }

    fn draw_rule_preview(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusRulePreview,
        });
        self.hits.push(HitRegion {
            rect: Rect::new(area.x, area.y, area.width, area.height.min(1)),
            action: HitAction::ToggleRulePreview,
        });
        let Some((category, rule)) = self
            .selected_rule()
            .map(|(category, rule)| (category.to_string(), rule.clone()))
        else {
            return;
        };
        let lines = rule_preview_lines(
            &rule,
            &category,
            area.width.saturating_sub(2).max(8),
            &self.theme,
        );
        self.rules_view.preview_max_scroll =
            (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
        self.rules_view.preview_scroll = self
            .rules_view
            .preview_scroll
            .min(self.rules_view.preview_max_scroll);
        let focused = self.rules_view.focus == RuleFocus::Preview;
        frame.render_widget(
            Paragraph::new(lines)
                .style(self.theme.panel_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Preview · Enter to close ")
                        .border_style(if focused {
                            self.theme.border_style(true)
                        } else {
                            self.theme.rule_row_accent_style(&category)
                        }),
                )
                .scroll((self.rules_view.preview_scroll, 0))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn register_rule_row_hits(&mut self, area: Rect, rows: &[RuleDisplayRow]) {
        self.hits.extend(rule_row_hit_regions(
            area,
            self.rules_view.list_offset,
            rows,
        ));
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
            self.rules_view.list_offset = 0;
            self.rules_view.preview_scroll = 0;
            self.rules_view.clamp(&self.rules);
        }
    }

    fn next_rule_category(&mut self) {
        if self.rules_view.selected_category + 1 < RULE_CATEGORIES.len() {
            self.rules_view.selected_category += 1;
            self.rules_view.selected_item = 0;
            self.rules_view.list_offset = 0;
            self.rules_view.preview_scroll = 0;
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
        self.rules_view.preview_scroll = 0;
    }

    fn first_rule_selection(&mut self) {
        if let Some((category, item)) = self.rule_positions().first().copied() {
            self.rules_view.selected_category = category;
            self.rules_view.selected_item = item;
            self.rules_view.preview_scroll = 0;
        }
    }

    fn last_rule_selection(&mut self) {
        if let Some((category, item)) = self.rule_positions().last().copied() {
            self.rules_view.selected_category = category;
            self.rules_view.selected_item = item;
            self.rules_view.preview_scroll = 0;
        }
    }

    fn rule_positions(&self) -> Vec<(usize, usize)> {
        let category_index = self.rules_view.selected_category;
        let category = RULE_CATEGORIES[category_index.min(RULE_CATEGORIES.len().saturating_sub(1))];
        let count = self.rules.get(category).map(Vec::len).unwrap_or(0);
        (0..count)
            .map(|item_index| (category_index, item_index))
            .collect()
    }

    fn current_rule_position_index(&self, positions: &[(usize, usize)]) -> Option<usize> {
        positions.iter().position(|(category, item)| {
            *category == self.rules_view.selected_category && *item == self.rules_view.selected_item
        })
    }

    fn rule_display_rows(&self, width: usize) -> Vec<RuleDisplayRow> {
        let mut rows = Vec::new();
        let category_index = self.rules_view.selected_category;
        let category = self.selected_rule_category();
        let items = self.rules.get(category).map(Vec::as_slice).unwrap_or(&[]);
        if items.is_empty() {
            rows.push(RuleDisplayRow {
                category_index,
                item_index: None,
                empty_marker: true,
                lines: vec![Line::from(Span::styled(
                    format!("No {category} rules defined. Press a to add one."),
                    self.theme.muted_style(),
                ))],
            });
            return rows;
        }

        for (item_index, item) in items.iter().enumerate() {
            rows.push(RuleDisplayRow {
                category_index,
                item_index: Some(item_index),
                empty_marker: false,
                lines: vec![rule_row_line(
                    item,
                    category,
                    width,
                    item_index == self.rules_view.selected_item,
                    &self.theme,
                )],
            });
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
        match app::rules::add(
            &self.workspace,
            &category,
            &rule,
            Some(normalized_rule_source(&source)),
        ) {
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
        match app::rules::edit(
            &self.workspace,
            &category,
            id,
            &rule,
            Some(normalized_rule_source(&source)),
        ) {
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
        match app::rules::delete(&self.workspace, &category, id) {
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
                KeyCode::Esc | KeyCode::Char('n') => {
                    RulePromptAction::Cancel("Rule delete canceled.".to_string())
                }
                KeyCode::Enter | KeyCode::Char('y') => RulePromptAction::Delete {
                    category: category.clone(),
                    id: *id,
                },
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
    lines: Vec<Line<'static>>,
}

fn rule_preview_lines(
    rule: &RuleItem,
    category: &str,
    width: u16,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let source = rule
        .source
        .as_deref()
        .filter(|source| !source.trim().is_empty())
        .unwrap_or("project config");
    let mut lines = vec![
        Line::from(vec![
            Span::styled(title_case(category), theme.rule_row_accent_style(category)),
            Span::styled(format!("  Rule #{}", rule.id), theme.label_style()),
            Span::styled(format!("  ·  {source}"), theme.muted_style()),
        ]),
        Line::from(""),
    ];
    lines.extend(
        wrap_words(&rule.rule, width.max(8) as usize)
            .into_iter()
            .map(|line| Line::from(Span::styled(line, theme.text_style()))),
    );
    lines
}

fn rule_row_line(
    item: &RuleItem,
    category: &str,
    width: usize,
    selected: bool,
    theme: &TuiTheme,
) -> Line<'static> {
    let width = width.max(12);
    let cursor = if selected { "› " } else { "  " };
    let id = format!("#{:<3}", item.id);
    let source = item
        .source
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("project config");
    let source_width = (width / 4).clamp(8, 24);
    let source = super::truncate(source, source_width);
    let fixed_width = cursor.chars().count() + id.chars().count() + source.chars().count() + 5;
    let preview_width = width.saturating_sub(fixed_width).max(1);
    let preview = super::truncate(&item.rule, preview_width);

    Line::from(vec![
        Span::styled(
            cursor.to_string(),
            if selected {
                theme.rule_row_accent_style(category)
            } else {
                theme.muted_style()
            },
        ),
        Span::styled(id, theme.rule_row_accent_style(category)),
        Span::styled(format!("  {preview}"), theme.text_style()),
        Span::styled(format!("  ·  {source}"), theme.muted_style()),
    ])
}

fn rule_list_block<'a>(theme: &TuiTheme, category: &str, focused: bool) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} rules ", title_case(category)))
        .border_style(if focused {
            theme.rule_row_accent_style(category)
        } else {
            theme.border_style(false)
        })
        .style(theme.panel_style())
}

fn rule_row_hit_regions(area: Rect, list_offset: usize, rows: &[RuleDisplayRow]) -> Vec<HitRegion> {
    rows.iter()
        .skip(list_offset)
        .take(area.height as usize)
        .enumerate()
        .filter_map(|(visible_index, row)| {
            row.item_index.map(|item_index| HitRegion {
                rect: Rect::new(
                    area.x,
                    area.y.saturating_add(visible_index as u16),
                    area.width,
                    1,
                ),
                action: HitAction::SelectRuleItem(item_index),
            })
        })
        .collect()
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

fn rule_view_layout(
    area: Rect,
    preview_open: bool,
    visible_rule_rows: usize,
) -> (Rect, Option<Rect>) {
    if preview_open {
        if let Some([list, preview]) = rule_preview_layout(area, visible_rule_rows) {
            return (list, Some(preview));
        }
    }
    (area, None)
}

fn rule_preview_layout(area: Rect, visible_rule_rows: usize) -> Option<[Rect; 2]> {
    if area.height < MIN_RULE_LIST_HEIGHT + MIN_RULE_PREVIEW_HEIGHT {
        return None;
    }

    let required_list_height = u16::try_from(visible_rule_rows)
        .unwrap_or(u16::MAX)
        .saturating_add(2)
        .max(MIN_RULE_LIST_HEIGHT);
    // The list cap is two thirds, rounded up by assigning the remainder to it.
    let list_cap = area.height.saturating_sub(area.height / 3);
    let list_height = required_list_height
        .min(list_cap)
        .min(area.height.saturating_sub(MIN_RULE_PREVIEW_HEIGHT));
    let preview_height = area.height.saturating_sub(list_height);
    let panes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(list_height),
            Constraint::Length(preview_height),
        ])
        .split(area);
    Some([panes[0], panes[1]])
}

fn nearest_rule_position(positions: &[(usize, usize)], category_index: usize) -> usize {
    positions
        .iter()
        .position(|(category, _)| *category >= category_index)
        .unwrap_or_else(|| positions.len().saturating_sub(1))
}

fn normalized_rule_source(source: &str) -> String {
    source.trim().to_string()
}

fn format_rule_outcome(
    verb: &str,
    outcome: &app::rules::MutationOutcome,
    reload_note: &str,
) -> String {
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
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn nearest_rule_position_prefers_current_or_next_category() {
        let positions = vec![(0, 0), (2, 0), (2, 1), (3, 0)];
        assert_eq!(nearest_rule_position(&positions, 0), 0);
        assert_eq!(nearest_rule_position(&positions, 1), 1);
        assert_eq!(nearest_rule_position(&positions, 3), 3);
        assert_eq!(nearest_rule_position(&positions, 4), 3);
    }

    #[test]
    fn tui_rule_source_trims_and_clears_before_app_input() {
        assert_eq!(normalized_rule_source("  decision-1  "), "decision-1");
        assert_eq!(normalized_rule_source("   "), "");
    }

    #[test]
    fn compact_rule_row_is_one_line_and_truncates_at_narrow_widths() {
        let item = RuleItem {
            id: 17,
            rule: "Keep every important word readable instead of expanding row heights".to_string(),
            source: Some("decision-10 selected direction".to_string()),
        };
        let line = rule_row_line(&item, "prefer", 32, false, &TuiTheme::default_dark());
        let text = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(text.contains("#17"));
        assert!(text.contains('…'));
        assert_eq!(line.spans[2].style, TuiTheme::default_dark().text_style());
        assert_eq!(line.spans[3].style, TuiTheme::default_dark().muted_style());
    }

    #[test]
    fn fixed_height_rule_rows_scroll_selected_row_into_view() {
        let mut terminal = Terminal::new(TestBackend::new(40, 3)).unwrap();
        let rows = (0..8)
            .map(|index| ListItem::new(Line::from(format!("row {index}"))))
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(Some(7));

        terminal
            .draw(|frame| frame.render_stateful_widget(List::new(rows), frame.area(), &mut state))
            .unwrap();

        assert!(state.offset() >= 5);
    }

    #[test]
    fn selected_rule_row_has_cursor_category_id_and_neutral_preview() {
        let theme = TuiTheme::default_dark();
        let item = RuleItem {
            id: 2,
            rule: "Use a restrained active treatment".to_string(),
            source: None,
        };
        let selected = rule_row_line(&item, "never", 56, true, &theme);
        let rendered = selected
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(selected.spans[0].content, "› ");
        assert_eq!(
            selected.spans[1].style,
            theme.rule_row_accent_style("never")
        );
        assert_eq!(selected.spans[2].style, theme.text_style());
        assert!(rendered.contains("project config"));
    }

    #[test]
    fn preview_closed_and_below_threshold_keep_full_height_list() {
        let area = Rect::new(3, 4, 80, 20);
        assert_eq!(rule_view_layout(area, false, 100), (area, None));

        let short = Rect::new(3, 4, 80, 7);
        assert_eq!(rule_view_layout(short, true, 2), (short, None));
    }

    #[test]
    fn preview_layout_requires_minimum_list_and_drawer_space() {
        assert!(rule_preview_layout(Rect::new(0, 0, 80, 7), 2).is_none());

        let [list, preview] = rule_preview_layout(Rect::new(0, 0, 80, 8), 2).unwrap();
        assert_eq!((list.height, preview.height), (3, 5));
        assert_eq!(preview.y, list.bottom());
    }

    #[test]
    fn preview_layout_fits_small_and_empty_categories() {
        for rows in [0, 1, 2] {
            let [list, preview] = rule_preview_layout(Rect::new(4, 6, 80, 24), rows).unwrap();
            let expected_list = (rows as u16).saturating_add(2).max(3);
            assert_eq!(list.height, expected_list);
            assert_eq!(preview.height, 24 - expected_list);
            assert_eq!(preview.y, list.bottom());
        }
    }

    #[test]
    fn preview_layout_caps_large_categories_at_rounded_up_two_thirds() {
        for (height, expected_list) in [(15, 10), (30, 20), (31, 21), (40, 27)] {
            let [list, preview] = rule_preview_layout(Rect::new(4, 6, 80, height), 100).unwrap();
            assert_eq!(list.height, expected_list);
            assert_eq!(list.height + preview.height, height);
            assert!(preview.height >= MIN_RULE_PREVIEW_HEIGHT);
        }
    }

    #[test]
    fn preview_layout_guarantees_preview_minimum_before_two_thirds_cap() {
        let [list, preview] = rule_preview_layout(Rect::new(0, 0, 80, 9), 100).unwrap();
        assert_eq!((list.height, preview.height), (4, 5));
    }

    #[test]
    fn rule_list_block_uses_category_border_style() {
        let theme = TuiTheme::default_dark();
        let mut terminal = Terminal::new(TestBackend::new(30, 3)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(rule_list_block(&theme, "context", true), frame.area());
            })
            .unwrap();
        let border = terminal.backend().buffer().cell((0, 0)).unwrap();
        assert_eq!(
            border.fg,
            theme.rule_row_accent_style("context").fg.unwrap()
        );
    }

    #[test]
    fn bordered_list_scrolls_with_inner_height() {
        let mut terminal = Terminal::new(TestBackend::new(40, 5)).unwrap();
        let rows = (0..8)
            .map(|index| ListItem::new(Line::from(format!("row {index}"))))
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(Some(7));
        terminal
            .draw(|frame| {
                frame.render_stateful_widget(
                    List::new(rows).block(Block::default().borders(Borders::ALL)),
                    frame.area(),
                    &mut state,
                )
            })
            .unwrap();
        assert_eq!(state.offset(), 5);
    }

    #[test]
    fn rule_mouse_hits_start_inside_border_and_respect_scroll_offset() {
        let rows = (0..5)
            .map(|item_index| RuleDisplayRow {
                category_index: 0,
                item_index: Some(item_index),
                empty_marker: false,
                lines: vec![Line::from("rule")],
            })
            .collect::<Vec<_>>();
        let hits = rule_row_hit_regions(Rect::new(11, 8, 30, 2), 2, &rows);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].rect, Rect::new(11, 8, 30, 1));
        assert!(matches!(hits[0].action, HitAction::SelectRuleItem(2)));
        assert_eq!(hits[1].rect.y, 9);
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
