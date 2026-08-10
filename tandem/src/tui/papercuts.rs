//! Read-only global Papercuts utility inbox state, rendering, and interaction.
//!
//! The panel is transient TUI state. Protocol meaning and concrete file reads
//! remain in the protocol, project, and application layers.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::papercuts::InboxLoad;
use crate::project::{display_path, StoredPapercut};

use super::{
    detail_field_line, detail_section_heading, markdownish_lines, HitAction, HitRegion, StatusTone,
    TuiApp, TuiTheme,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PapercutFocus {
    #[default]
    List,
    Detail,
}

#[derive(Debug, Default)]
pub(super) struct PapercutsState {
    open: bool,
    items: Vec<StoredPapercut>,
    warnings: Vec<String>,
    pub(super) selected: usize,
    focus: PapercutFocus,
    pub(super) detail_scroll: u16,
    list_offset: usize,
}

impl PapercutsState {
    #[cfg(test)]
    pub(super) fn set_items(&mut self, items: Vec<StoredPapercut>) {
        self.items = items;
        self.clamp();
    }

    fn replace_load(&mut self, load: InboxLoad) {
        self.items = load.items;
        self.warnings = load.warnings;
        self.clamp();
    }

    fn clamp(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
            self.list_offset = 0;
        } else if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
        self.detail_scroll = self.detail_scroll.min(
            papercut_detail_line_count(self.selected_item(), Some(&self.warnings)).saturating_sub(1)
                as u16,
        );
    }

    fn selected_item(&self) -> Option<&StoredPapercut> {
        self.items.get(self.selected)
    }
}

impl TuiApp {
    pub(super) fn load_papercuts(&mut self, load: InboxLoad) {
        self.papercuts_view.replace_load(load);
    }

    pub(super) fn papercut_count(&self) -> usize {
        self.papercuts_view.items.len()
    }

    pub(super) fn papercut_warnings(&self) -> &[String] {
        &self.papercuts_view.warnings
    }

    pub(super) fn papercuts_open(&self) -> bool {
        self.papercuts_view.open
    }

    pub(super) fn selected_papercut_id_for_reload(&self) -> Option<String> {
        self.papercuts_view
            .selected_item()
            .map(|item| item.id().to_string())
    }

    pub(super) fn restore_papercut_selection_after_reload(&mut self, id: Option<String>) {
        if let Some(id) = id.as_deref() {
            if let Some(index) = self
                .papercuts_view
                .items
                .iter()
                .position(|item| item.id() == id)
            {
                self.papercuts_view.selected = index;
                self.papercuts_view.clamp();
                return;
            }
        }
        self.papercuts_view.clamp();
    }

    pub(super) fn toggle_papercuts(&mut self) {
        if self.papercuts_view.open {
            self.close_papercuts();
            return;
        }
        self.papercuts_view.open = true;
        self.papercuts_view.clamp();
        self.status = match (
            self.papercuts_view.items.len(),
            self.papercuts_view.warnings.len(),
        ) {
            (0, 0) => "Papercuts inbox is empty; press i or Esc to close.".to_string(),
            (count, 0) => format!(
                "Papercuts inbox open: {count} open record{} · read-only.",
                if count == 1 { "" } else { "s" }
            ),
            (count, warnings) => format!(
                "Papercuts inbox open: {count} open record{} · {warnings} load warning{} · read-only.",
                if count == 1 { "" } else { "s" },
                if warnings == 1 { "" } else { "s" }
            ),
        };
    }

    pub(super) fn close_papercuts(&mut self) {
        self.papercuts_view.open = false;
        self.status = "Papercuts inbox closed; previous view restored.".to_string();
    }

    pub(super) fn handle_papercuts_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') | KeyCode::Esc => self.close_papercuts(),
            KeyCode::Char('r') => {
                self.reload();
            }
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => self.papercuts_view.focus = PapercutFocus::Detail,
            KeyCode::BackTab => self.papercuts_view.focus = PapercutFocus::List,
            KeyCode::Enter => self.papercuts_view.focus = PapercutFocus::Detail,
            KeyCode::Left | KeyCode::Char('h') => self.papercuts_view.focus = PapercutFocus::List,
            KeyCode::Right | KeyCode::Char('l') => {
                self.papercuts_view.focus = PapercutFocus::Detail
            }
            KeyCode::Up | KeyCode::Char('k') => match self.papercuts_view.focus {
                PapercutFocus::List => self.previous_papercut(),
                PapercutFocus::Detail => self.scroll_papercut_detail_up(1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.papercuts_view.focus {
                PapercutFocus::List => self.next_papercut(),
                PapercutFocus::Detail => self.scroll_papercut_detail_down(1),
            },
            KeyCode::PageUp => match self.papercuts_view.focus {
                PapercutFocus::List => self.move_papercut_selection(-5),
                PapercutFocus::Detail => self.scroll_papercut_detail_up(6),
            },
            KeyCode::PageDown => match self.papercuts_view.focus {
                PapercutFocus::List => self.move_papercut_selection(5),
                PapercutFocus::Detail => self.scroll_papercut_detail_down(6),
            },
            KeyCode::Char('u')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                match self.papercuts_view.focus {
                    PapercutFocus::List => self.move_papercut_selection(-5),
                    PapercutFocus::Detail => self.scroll_papercut_detail_up(6),
                }
            }
            KeyCode::Char('d')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                match self.papercuts_view.focus {
                    PapercutFocus::List => self.move_papercut_selection(5),
                    PapercutFocus::Detail => self.scroll_papercut_detail_down(6),
                }
            }
            KeyCode::Home | KeyCode::Char('g') => match self.papercuts_view.focus {
                PapercutFocus::List => self.select_first_papercut(),
                PapercutFocus::Detail => self.papercuts_view.detail_scroll = 0,
            },
            KeyCode::End | KeyCode::Char('G') => match self.papercuts_view.focus {
                PapercutFocus::List => self.select_last_papercut(),
                PapercutFocus::Detail => self.scroll_papercut_detail_down(u16::MAX),
            },
            _ => {}
        }
    }

    pub(super) fn handle_papercuts_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                match self.mouse_hit_action(mouse.column, mouse.row) {
                    Some(HitAction::TogglePapercuts) => self.close_papercuts(),
                    Some(HitAction::FocusPapercutList) => {
                        self.papercuts_view.focus = PapercutFocus::List
                    }
                    Some(HitAction::SelectPapercut(index)) => {
                        self.select_papercut(index);
                        self.papercuts_view.focus = PapercutFocus::List;
                    }
                    Some(HitAction::FocusPapercutDetail) => {
                        self.papercuts_view.focus = PapercutFocus::Detail
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollDown => match self.mouse_hit_action(mouse.column, mouse.row) {
                Some(HitAction::FocusPapercutDetail) => {
                    self.papercuts_view.focus = PapercutFocus::Detail;
                    self.scroll_papercut_detail_down(3);
                }
                Some(HitAction::FocusPapercutList) | Some(HitAction::SelectPapercut(_)) => {
                    self.papercuts_view.focus = PapercutFocus::List;
                    self.next_papercut();
                }
                _ => {}
            },
            MouseEventKind::ScrollUp => match self.mouse_hit_action(mouse.column, mouse.row) {
                Some(HitAction::FocusPapercutDetail) => {
                    self.papercuts_view.focus = PapercutFocus::Detail;
                    self.scroll_papercut_detail_up(3);
                }
                Some(HitAction::FocusPapercutList) | Some(HitAction::SelectPapercut(_)) => {
                    self.papercuts_view.focus = PapercutFocus::List;
                    self.previous_papercut();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn select_papercut(&mut self, index: usize) {
        if !self.papercuts_view.items.is_empty() {
            self.papercuts_view.selected = index.min(self.papercuts_view.items.len() - 1);
            self.papercuts_view.detail_scroll = 0;
        }
    }

    fn previous_papercut(&mut self) {
        if self.papercuts_view.selected > 0 {
            self.papercuts_view.selected -= 1;
            self.papercuts_view.detail_scroll = 0;
        }
    }

    fn next_papercut(&mut self) {
        if self.papercuts_view.selected + 1 < self.papercuts_view.items.len() {
            self.papercuts_view.selected += 1;
            self.papercuts_view.detail_scroll = 0;
        }
    }

    fn move_papercut_selection(&mut self, delta: isize) {
        if self.papercuts_view.items.is_empty() {
            return;
        }
        let last = self.papercuts_view.items.len() - 1;
        self.papercuts_view.selected = self
            .papercuts_view
            .selected
            .saturating_add_signed(delta)
            .min(last);
        self.papercuts_view.detail_scroll = 0;
    }

    fn select_first_papercut(&mut self) {
        if !self.papercuts_view.items.is_empty() {
            self.papercuts_view.selected = 0;
            self.papercuts_view.detail_scroll = 0;
        }
    }

    fn select_last_papercut(&mut self) {
        if !self.papercuts_view.items.is_empty() {
            self.papercuts_view.selected = self.papercuts_view.items.len() - 1;
            self.papercuts_view.detail_scroll = 0;
        }
    }

    fn scroll_papercut_detail_up(&mut self, amount: u16) {
        self.papercuts_view.detail_scroll =
            self.papercuts_view.detail_scroll.saturating_sub(amount);
    }

    fn scroll_papercut_detail_down(&mut self, amount: u16) {
        let max_scroll = papercut_detail_line_count(
            self.papercuts_view.selected_item(),
            Some(&self.papercuts_view.warnings),
        )
        .saturating_sub(1) as u16;
        self.papercuts_view.detail_scroll = self
            .papercuts_view
            .detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    pub(super) fn papercut_indicator_text(&self) -> String {
        let warning = if self.papercuts_view.warnings.is_empty() {
            ""
        } else {
            " !"
        };
        format!("Papercuts {}{warning}", self.papercuts_view.items.len())
    }

    pub(super) fn papercut_indicator_line(&self) -> Line<'static> {
        let style = if self.papercuts_view.open {
            self.theme.tab_selected_style()
        } else if self.papercuts_view.items.is_empty() {
            self.theme.muted_style()
        } else {
            self.theme
                .status_style(StatusTone::Accent)
                .add_modifier(Modifier::BOLD)
        };
        let mut spans = vec![Span::styled(
            format!("Papercuts {}", self.papercuts_view.items.len()),
            style,
        )];
        if !self.papercuts_view.warnings.is_empty() {
            spans.push(Span::styled(
                " !".to_string(),
                self.theme
                    .status_style(StatusTone::Error)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        Line::from(spans)
    }

    pub(super) fn papercuts_footer_text(&self) -> String {
        let commands = match self.papercuts_view.focus {
            PapercutFocus::List => "j/k · Enter open detail",
            PapercutFocus::Detail => "j/k scroll · Shift-Tab list",
        };
        format!("Papercuts · read-only · i/Esc close · {commands}")
    }

    pub(super) fn draw_papercuts_panel(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let popup = papercut_panel_rect(area);
        frame.render_widget(Clear, area);
        let title = format!(
            " Papercuts inbox · {} open · read-only ",
            self.papercuts_view.items.len()
        );
        let outer = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(self.theme.border_style(true))
            .style(self.theme.panel_style());
        let inner = outer.inner(popup);
        frame.render_widget(outer, popup);
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let (warning_area, content_area) = if self.papercuts_view.warnings.is_empty() {
            (None, inner)
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(inner);
            (Some(chunks[0]), chunks[1])
        };
        if let Some(warning_area) = warning_area {
            let count = self.papercuts_view.warnings.len();
            let warning = format!(
                "! {count} load warning{} · {}",
                if count == 1 { "" } else { "s" },
                self.papercuts_view
                    .warnings
                    .first()
                    .map(String::as_str)
                    .unwrap_or("inspect Papercut storage")
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    warning,
                    self.theme.status_style(StatusTone::Error),
                )))
                .style(self.theme.panel_style()),
                warning_area,
            );
        }

        if content_area.height < 9 {
            match self.papercuts_view.focus {
                PapercutFocus::List => self.draw_papercut_list(frame, content_area),
                PapercutFocus::Detail => self.draw_papercut_detail(frame, content_area),
            }
            return;
        }

        let chunks = if content_area.width >= 92 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(content_area)
        } else {
            let max_list_height = content_area
                .height
                .saturating_mul(50)
                .checked_div(100)
                .unwrap_or(0)
                .max(5)
                .min(content_area.height.saturating_sub(4));
            let list_height = (self.papercuts_view.items.len() as u16)
                .saturating_add(2)
                .clamp(5, max_list_height);
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(list_height), Constraint::Min(4)])
                .split(content_area)
        };
        self.draw_papercut_list(frame, chunks[0]);
        self.draw_papercut_detail(frame, chunks[1]);
    }

    fn draw_papercut_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusPapercutList,
        });
        let count = self.papercuts_view.items.len();
        let items = if count == 0 {
            vec![ListItem::new(Line::from(Span::styled(
                "No open Papercuts.",
                self.theme.muted_style(),
            )))]
        } else {
            self.papercuts_view
                .items
                .iter()
                .map(|item| papercut_list_item(item, &self.theme, area.width.saturating_sub(4)))
                .collect::<Vec<_>>()
        };
        let list = List::new(items)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Open ({count}) "))
                    .border_style(
                        self.theme
                            .border_style(self.papercuts_view.focus == PapercutFocus::List),
                    )
                    .style(self.theme.panel_style()),
            )
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");
        if count == 0 {
            frame.render_widget(list, area);
            return;
        }

        let mut state = ListState::default().with_offset(self.papercuts_view.list_offset);
        state.select(Some(self.papercuts_view.selected.min(count - 1)));
        frame.render_stateful_widget(list, area, &mut state);
        self.papercuts_view.list_offset = state.offset();
        self.register_papercut_row_hits(area, state.offset(), count);
    }

    fn register_papercut_row_hits(&mut self, area: Rect, offset: usize, count: usize) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let top = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let visible = area.height.saturating_sub(2) as usize;
        for index in offset..count.min(offset.saturating_add(visible)) {
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y: top.saturating_add(index.saturating_sub(offset) as u16),
                    width,
                    height: 1,
                },
                action: HitAction::SelectPapercut(index),
            });
        }
    }

    fn draw_papercut_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusPapercutDetail,
        });
        let (title, lines) = match self.papercuts_view.selected_item() {
            Some(item) => (
                format!(" Detail {} ", item.id()),
                papercut_detail_lines(item, &self.papercuts_view.warnings, &self.theme),
            ),
            None => (
                " Detail ".to_string(),
                empty_papercut_detail_lines(&self.papercuts_view.warnings, &self.theme),
            ),
        };
        let detail = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(
                        self.theme
                            .border_style(self.papercuts_view.focus == PapercutFocus::Detail),
                    )
                    .style(self.theme.panel_style()),
            )
            .scroll((self.papercuts_view.detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }
}

fn papercut_panel_rect(area: Rect) -> Rect {
    let horizontal = u16::from(area.width >= 72);
    let vertical = u16::from(area.height >= 16);
    Rect {
        x: area.x.saturating_add(horizontal),
        y: area.y.saturating_add(vertical),
        width: area.width.saturating_sub(horizontal.saturating_mul(2)),
        height: area.height.saturating_sub(vertical.saturating_mul(2)),
    }
}

fn papercut_list_item(
    item: &StoredPapercut,
    theme: &TuiTheme,
    available_width: u16,
) -> ListItem<'static> {
    let id_width = item.id().chars().count().saturating_add(2);
    let title_width = (available_width as usize).saturating_sub(id_width);
    ListItem::new(Line::from(vec![
        Span::styled(
            format!("{}  ", item.id()),
            theme.status_style(StatusTone::Accent),
        ),
        Span::styled(
            super::truncate(item.title(), title_width),
            theme.text_style(),
        ),
    ]))
}

fn papercut_detail_lines(
    item: &StoredPapercut,
    warnings: &[String],
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(item.title().to_string(), theme.title_style())),
        Line::from(""),
        detail_section_heading("Papercut", theme),
        detail_field_line("ID", item.id(), theme),
        detail_field_line("Title", item.title(), theme),
        detail_field_line("Status", item.status(), theme),
    ];
    push_optional_field(&mut lines, "Created", item.field("createdAt"), theme);
    push_optional_field(&mut lines, "Updated", item.field("updatedAt"), theme);
    push_values_field(&mut lines, "Tags", item.values("tags"), theme);
    push_values_field(&mut lines, "References", item.values("references"), theme);
    lines.push(detail_field_line("Path", &display_path(&item.path), theme));
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Body", theme));
    if item.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        lines.extend(markdownish_lines(&item.body, theme));
    }
    append_warning_lines(&mut lines, warnings, theme);
    lines
}

fn empty_papercut_detail_lines(warnings: &[String], theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            "No open Papercut selected.",
            theme.muted_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Papercuts stay outside Board workflow. Use the CLI or integration tools to capture or resolve them.",
            theme.text_style(),
        )),
    ];
    append_warning_lines(&mut lines, warnings, theme);
    lines
}

fn append_warning_lines(lines: &mut Vec<Line<'static>>, warnings: &[String], theme: &TuiTheme) {
    if warnings.is_empty() {
        return;
    }
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Load warnings", theme));
    for warning in warnings {
        lines.push(Line::from(vec![
            Span::styled("! ", theme.status_style(StatusTone::Error)),
            Span::styled(warning.clone(), theme.status_style(StatusTone::Error)),
        ]));
    }
}

fn push_optional_field(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(detail_field_line(label, value, theme));
    }
}

fn push_values_field(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    values: Vec<String>,
    theme: &TuiTheme,
) {
    if !values.is_empty() {
        lines.push(detail_field_line(label, &values.join(", "), theme));
    }
}

fn papercut_detail_line_count(item: Option<&StoredPapercut>, warnings: Option<&[String]>) -> usize {
    let item_lines = item.map_or(3, |item| {
        let tag_line = usize::from(!item.values("tags").is_empty());
        let reference_line = usize::from(!item.values("references").is_empty());
        11usize
            .saturating_add(tag_line)
            .saturating_add(reference_line)
            .saturating_add(item.body.lines().count().max(1))
    });
    item_lines.saturating_add(warnings.map_or(0, |warnings| warnings.len().saturating_add(2)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::tui::{rect_contains, FocusPane};

    fn item(id: &str, title: &str, body: &str) -> StoredPapercut {
        StoredPapercut::new(
            PathBuf::from(format!(".tandem/papercuts/{id}.md")),
            HashMap::from([
                ("id".to_string(), id.to_string()),
                ("title".to_string(), title.to_string()),
                ("status".to_string(), "open".to_string()),
                ("createdAt".to_string(), "2026-08-10T00:00:00Z".to_string()),
                ("updatedAt".to_string(), "2026-08-10T01:00:00Z".to_string()),
                ("tags".to_string(), "[\"tui\", \"friction\"]".to_string()),
                ("references".to_string(), "[\"task-1\"]".to_string()),
            ]),
            body.to_string(),
        )
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn detail_uses_protocol_fields_and_shared_markdown_rendering() {
        let theme = TuiTheme::default_dark();
        let lines = papercut_detail_lines(
            &item("papercut-1", "Small friction", "# Notes\n\n- first\n`code`"),
            &["Papercuts load warning: malformed record".to_string()],
            &theme,
        )
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

        assert!(lines.iter().any(|line| line.contains("ID: papercut-1")));
        assert!(lines.iter().any(|line| line.contains("Status: open")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Tags: tui, friction")));
        assert!(lines.iter().any(|line| line.contains("References: task-1")));
        assert!(lines.iter().any(|line| line == "Notes"));
        assert!(lines.iter().any(|line| line == "• first"));
        assert!(lines.iter().any(|line| line.contains("malformed record")));
    }

    #[test]
    fn compact_panel_rect_stays_inside_the_current_view() {
        for area in [Rect::new(0, 4, 45, 7), Rect::new(0, 4, 120, 30)] {
            let panel = papercut_panel_rect(area);
            assert!(rect_contains(area, panel.x, panel.y));
            assert!(panel.x.saturating_add(panel.width) <= area.x.saturating_add(area.width));
            assert!(panel.y.saturating_add(panel.height) <= area.y.saturating_add(area.height));
        }
    }

    #[test]
    fn focus_is_independent_from_main_view_focus() {
        let mut state = PapercutsState::default();
        state.replace_load(InboxLoad {
            items: vec![item("papercut-1", "First", "Body")],
            warnings: Vec::new(),
        });
        state.focus = PapercutFocus::Detail;
        let main_focus = FocusPane::Board;
        assert_eq!(state.focus, PapercutFocus::Detail);
        assert_eq!(main_focus, FocusPane::Board);
    }
}
