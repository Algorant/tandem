use std::collections::BTreeMap;
use std::io;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

use super::*;

pub(crate) fn run_tui() -> Result<(), CliError> {
    let workspace = discover_workspace()?;
    let mut app = TuiApp::load(workspace)?;
    let mut session = TerminalSession::enter()?;
    app.run(session.terminal_mut())
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self, CliError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(mut terminal) => {
                if let Err(error) = terminal.clear() {
                    restore_terminal(terminal.backend_mut());
                    return Err(error.into());
                }
                Ok(Self { terminal })
            }
            Err(error) => {
                let _ = disable_raw_mode();
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
                Err(error.into())
            }
        }
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        restore_terminal(self.terminal.backend_mut());
        let _ = self.terminal.show_cursor();
    }
}

fn restore_terminal(backend: &mut CrosstermBackend<io::Stdout>) {
    let _ = disable_raw_mode();
    let _ = execute!(backend, LeaveAlternateScreen, DisableMouseCapture);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Board,
    Detail,
}

#[derive(Debug, Clone)]
enum HitAction {
    SelectState(usize),
    FocusDetail,
}

#[derive(Debug, Clone)]
struct HitRegion {
    rect: Rect,
    action: HitAction,
}

#[derive(Debug, Clone)]
struct QuickAddInput {
    state: String,
    title: String,
    fallback_note: Option<String>,
}

struct TuiApp {
    workspace: Workspace,
    title: String,
    states: Vec<String>,
    configured_states: Vec<String>,
    docs: Vec<Document>,
    selected_state: usize,
    selected_item: usize,
    focus: FocusPane,
    detail_scroll: u16,
    status: String,
    show_help: bool,
    quick_add: Option<QuickAddInput>,
    hits: Vec<HitRegion>,
}

impl TuiApp {
    fn load(workspace: Workspace) -> Result<Self, CliError> {
        let mut app = Self {
            workspace,
            title: String::new(),
            states: Vec::new(),
            configured_states: Vec::new(),
            docs: Vec::new(),
            selected_state: 0,
            selected_item: 0,
            focus: FocusPane::Board,
            detail_scroll: 0,
            status: String::new(),
            show_help: false,
            quick_add: None,
            hits: Vec::new(),
        };
        app.reload()?;
        Ok(app)
    }

    fn reload(&mut self) -> Result<(), CliError> {
        let mut docs = read_documents(&self.workspace.board_dir, DocumentLocation::Board)?;
        sort_documents(&mut docs);
        let configured_states = read_workspace_states(&self.workspace)?;
        self.title = read_workspace_title(&self.workspace)?;
        self.states = states_with_board_docs(configured_states.clone(), &docs);
        self.configured_states = configured_states;
        self.docs = docs;
        self.clamp_selection();
        self.status = format!(
            "Loaded {} active document{} from {}",
            self.docs.len(),
            if self.docs.len() == 1 { "" } else { "s" },
            display_path(&self.workspace.board_dir)
        );
        Ok(())
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), CliError> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, CliError> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        if self.quick_add.is_some() {
            self.handle_quick_add_key(key);
            return Ok(false);
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.show_help = false,
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('r') => match self.reload() {
                Ok(()) => {}
                Err(error) => self.status = format!("Reload failed: {}", error.message),
            },
            KeyCode::Char('a') => self.start_quick_add(),
            KeyCode::Char('H') => self.move_selected_task_by_delta(-1),
            KeyCode::Char('L') => self.move_selected_task_by_delta(1),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Enter => self.toggle_focus(),
            KeyCode::Char('1') => self.status = "Board view is active".to_string(),
            KeyCode::Char('2') => {
                self.status =
                    "Review view is planned; Board shell is active in this slice".to_string()
            }
            KeyCode::Char('3') => {
                self.status =
                    "Logs view is planned; Board shell is active in this slice".to_string()
            }
            KeyCode::Char('4') => {
                self.status =
                    "Rules view is planned; Board shell is active in this slice".to_string()
            }
            KeyCode::Char('5') => {
                self.status =
                    "Decisions view is planned; Board shell is active in this slice".to_string()
            }
            KeyCode::Esc => {
                if self.focus == FocusPane::Detail {
                    self.focus = FocusPane::Board;
                }
            }
            _ => match self.focus {
                FocusPane::Board => self.handle_board_key(key),
                FocusPane::Detail => self.handle_detail_key(key),
            },
        }
        Ok(false)
    }

    fn handle_board_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.previous_state(),
            KeyCode::Right | KeyCode::Char('l') => self.next_state(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_item(),
            KeyCode::Home | KeyCode::Char('g') => self.selected_item = 0,
            KeyCode::End | KeyCode::Char('G') => self.last_item(),
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.scroll_detail_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_detail_down(1),
            KeyCode::PageUp | KeyCode::Char('u') => self.scroll_detail_up(6),
            KeyCode::PageDown | KeyCode::Char('d') => self.scroll_detail_down(6),
            KeyCode::Home | KeyCode::Char('g') => self.detail_scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.detail_scroll_to_end(),
            KeyCode::Left | KeyCode::Char('h') => self.previous_state(),
            KeyCode::Right | KeyCode::Char('l') => self.next_state(),
            _ => {}
        }
    }

    fn handle_quick_add_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.quick_add = None;
                self.status = "Quick add canceled.".to_string();
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => self.finish_quick_add(),
            KeyCode::Char('m') | KeyCode::Char('j')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.finish_quick_add()
            }
            KeyCode::Backspace => {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.pop();
                }
                self.refresh_quick_add_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.clear();
                }
                self.refresh_quick_add_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(input) = self.quick_add.as_mut() {
                    input.title.push(ch);
                }
                self.refresh_quick_add_status();
            }
            _ => {}
        }
    }

    fn start_quick_add(&mut self) {
        let (state, fallback_note) = quick_add_state_for_selection(
            &self.configured_states,
            &self.states,
            self.selected_state,
        );
        self.quick_add = Some(QuickAddInput {
            state,
            title: String::new(),
            fallback_note,
        });
        self.focus = FocusPane::Board;
        self.refresh_quick_add_status();
    }

    fn refresh_quick_add_status(&mut self) {
        if let Some(input) = self.quick_add.as_ref() {
            self.status = quick_add_status(input);
        }
    }

    fn finish_quick_add(&mut self) {
        let Some(input) = self.quick_add.as_ref() else {
            return;
        };
        let title = input.title.trim().to_string();
        if title.is_empty() {
            self.status = format!(
                "Quick add needs a title. Add task in {}: type title, Enter create, Esc cancel",
                input.state
            );
            return;
        }
        let state = input.state.clone();
        self.quick_add = None;

        match create_basic_task(&self.workspace, &title, &state) {
            Ok(outcome) => {
                let reload_error = match self.reload() {
                    Ok(()) => {
                        self.select_document_by_id(&outcome.id);
                        None
                    }
                    Err(error) => Some(error.message),
                };
                self.status = format!(
                    "Created {} in {}: {}{}",
                    outcome.id,
                    outcome.state,
                    outcome.title,
                    reload_error
                        .map(|message| format!("; reload failed: {message}"))
                        .unwrap_or_default()
                );
            }
            Err(error) => {
                let reload_note = match self.reload() {
                    Ok(()) => String::new(),
                    Err(reload_error) => format!(" Reload failed: {}", reload_error.message),
                };
                self.status = format!("Add error: {}{}", error.message, reload_note);
            }
        }
    }

    fn move_selected_task_by_delta(&mut self, delta: isize) {
        let Some((doc_id, current_state)) = self
            .selected_doc()
            .map(|doc| (doc.id().to_string(), doc.field("state").map(str::to_string)))
        else {
            self.status = "No selected item to move.".to_string();
            return;
        };

        let target_state = match adjacent_configured_state(
            &self.configured_states,
            current_state.as_deref(),
            delta,
        ) {
            Ok(state) => state,
            Err(message) => {
                self.status = message;
                return;
            }
        };

        self.move_selected_task_to_state(&doc_id, &target_state);
    }

    fn move_selected_task_to_state(&mut self, doc_id: &str, target_state: &str) {
        match move_task_to_state(&self.workspace, doc_id, target_state) {
            Ok(outcome) => {
                let reload_error = match self.reload() {
                    Ok(()) => {
                        self.select_document_by_id(&outcome.id);
                        None
                    }
                    Err(error) => Some(error.message),
                };
                self.status = if outcome.changed {
                    format!(
                        "Moved {}: {} -> {}{}",
                        outcome.id,
                        outcome.from,
                        outcome.to,
                        reload_error
                            .map(|message| format!("; reload failed: {message}"))
                            .unwrap_or_default()
                    )
                } else {
                    format!("{} is already in state {}", outcome.id, outcome.to)
                };
            }
            Err(error) => {
                let reload_note = match self.reload() {
                    Ok(()) => {
                        self.select_document_by_id(doc_id);
                        String::new()
                    }
                    Err(reload_error) => format!(" Reload failed: {}", reload_error.message),
                };
                self.status = format!("Move error: {}{}", error.message, reload_note);
            }
        }
    }

    fn select_document_by_id(&mut self, id: &str) -> bool {
        for state_index in 0..self.states.len() {
            let Some(state_name) = self.states.get(state_index) else {
                continue;
            };
            let mut item_index = 0;
            for doc in &self.docs {
                if document_state_label(doc) == state_name.as_str() {
                    if doc.id() == id {
                        self.selected_state = state_index;
                        self.selected_item = item_index;
                        self.detail_scroll = 0;
                        self.clamp_selection();
                        return true;
                    }
                    item_index += 1;
                }
            }
        }
        self.clamp_selection();
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
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
                        HitAction::SelectState(index) => {
                            self.selected_state = index.min(self.states.len().saturating_sub(1));
                            self.selected_item = 0;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::FocusDetail => self.focus = FocusPane::Detail,
                    }
                }
            }
            MouseEventKind::ScrollDown => match self.focus {
                FocusPane::Board => self.next_item(),
                FocusPane::Detail => self.scroll_detail_down(3),
            },
            MouseEventKind::ScrollUp => match self.focus {
                FocusPane::Board => self.previous_item(),
                FocusPane::Detail => self.scroll_detail_up(3),
            },
            _ => {}
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Board => FocusPane::Detail,
            FocusPane::Detail => FocusPane::Board,
        };
    }

    fn previous_state(&mut self) {
        if self.selected_state > 0 {
            self.selected_state -= 1;
            self.selected_item = 0;
            self.detail_scroll = 0;
        }
        self.clamp_selection();
    }

    fn next_state(&mut self) {
        if self.selected_state + 1 < self.states.len() {
            self.selected_state += 1;
            self.selected_item = 0;
            self.detail_scroll = 0;
        }
        self.clamp_selection();
    }

    fn previous_item(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.detail_scroll = 0;
        }
    }

    fn next_item(&mut self) {
        let count = self.selected_state_count();
        if self.selected_item + 1 < count {
            self.selected_item += 1;
            self.detail_scroll = 0;
        }
    }

    fn last_item(&mut self) {
        let count = self.selected_state_count();
        if count > 0 {
            self.selected_item = count - 1;
            self.detail_scroll = 0;
        }
    }

    fn scroll_detail_up(&mut self, amount: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(amount);
    }

    fn scroll_detail_down(&mut self, amount: u16) {
        let max_scroll = self.detail_line_count().saturating_sub(1) as u16;
        self.detail_scroll = self.detail_scroll.saturating_add(amount).min(max_scroll);
    }

    fn detail_scroll_to_end(&mut self) {
        self.detail_scroll = self.detail_line_count().saturating_sub(1) as u16;
    }

    fn clamp_selection(&mut self) {
        if self.states.is_empty() {
            self.states.push("todo".to_string());
        }
        if self.selected_state >= self.states.len() {
            self.selected_state = self.states.len().saturating_sub(1);
        }
        let count = self.selected_state_count();
        if count == 0 {
            self.selected_item = 0;
        } else if self.selected_item >= count {
            self.selected_item = count - 1;
        }
        let max_scroll = self.detail_line_count().saturating_sub(1) as u16;
        self.detail_scroll = self.detail_scroll.min(max_scroll);
    }

    fn selected_state_count(&self) -> usize {
        self.states
            .get(self.selected_state)
            .map(|state| self.docs_for_state(state).len())
            .unwrap_or(0)
    }

    fn selected_doc(&self) -> Option<&Document> {
        let state = self.states.get(self.selected_state)?;
        self.docs_for_state(state)
            .into_iter()
            .nth(self.selected_item)
    }

    fn docs_for_state(&self, state: &str) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| document_state_label(doc) == state)
            .collect()
    }

    fn detail_line_count(&self) -> usize {
        self.selected_doc()
            .map(detail_lines_for_doc)
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.hits.clear();
        let area = frame.area();
        if area.width < 45 || area.height < 12 {
            self.draw_tiny(frame, area);
            return;
        }

        let detail_height = (area.height / 3).clamp(5, 12);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(4),
                Constraint::Length(detail_height),
                Constraint::Length(1),
            ])
            .split(area);

        self.draw_header(frame, chunks[0]);
        self.draw_view_tabs(frame, chunks[1]);
        self.draw_board(frame, chunks[2]);
        self.draw_detail(frame, chunks[3]);
        self.draw_footer(frame, chunks[4]);

        if self.show_help {
            self.draw_help(frame, area);
        }
    }

    fn draw_tiny(&self, frame: &mut Frame<'_>, area: Rect) {
        let message = Paragraph::new(vec![
            Line::from(Span::styled(
                "Tandem TUI needs a larger terminal",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "Current: {}x{} · minimum: 45x12",
                area.width, area.height
            )),
            Line::from("Press q to quit after resizing if needed."),
        ])
        .block(Block::default().borders(Borders::ALL).title(" tdm tui "))
        .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
    }

    fn draw_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let counts = self
            .states
            .iter()
            .map(|state| format!("{state} {}", self.docs_for_state(state).len()))
            .collect::<Vec<_>>()
            .join(" · ");
        let selected = self
            .selected_doc()
            .map(|doc| format!("selected {}", doc.id()))
            .unwrap_or_else(|| "no selected item".to_string());
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    self.title.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(counts, Style::default().fg(Color::Gray)),
            ]),
            Line::from(Span::styled(selected, Style::default().fg(Color::DarkGray))),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Tandem "));
        frame.render_widget(header, area);
    }

    fn draw_view_tabs(&self, frame: &mut Frame<'_>, area: Rect) {
        let titles = ["Board", "Review", "Logs", "Rules", "Decisions"]
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .select(0)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(tabs, area);
    }

    fn draw_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if area.width >= 100 && self.states.len() > 1 {
            let constraints =
                vec![Constraint::Ratio(1, self.states.len() as u32); self.states.len()];
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(area);
            for index in 0..self.states.len() {
                self.draw_state_list(frame, columns[index], index, true);
            }
        } else {
            self.draw_state_tabs(frame, area);
        }
    }

    fn draw_state_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let titles = self
            .states
            .iter()
            .map(|state| Line::from(format!(" {state} ({}) ", self.docs_for_state(state).len())))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.selected_state)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );
        frame.render_widget(tabs, chunks[0]);
        self.draw_state_list(frame, chunks[1], self.selected_state, false);
    }

    fn draw_state_list(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        state_index: usize,
        compact_title: bool,
    ) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::SelectState(state_index),
        });

        let Some(state_name) = self.states.get(state_index) else {
            return;
        };
        let docs = self.docs_for_state(state_name);
        let count = docs.len();
        let items = if docs.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "(empty)",
                Style::default().fg(Color::DarkGray),
            )))]
        } else {
            docs.iter()
                .map(|doc| list_item_for_doc(doc))
                .collect::<Vec<_>>()
        };

        let title = if compact_title {
            format!(" {state_name} ({count}) ")
        } else {
            format!(
                " State: {state_name} ({}/{}) · {count} item{} ",
                state_index + 1,
                self.states.len(),
                if count == 1 { "" } else { "s" }
            )
        };
        let selected = state_index == self.selected_state;
        let border_style = if selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        if selected && count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_item.min(count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn draw_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusDetail,
        });

        let focus_style = if self.focus == FocusPane::Detail {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let (title, lines) = match self.selected_doc() {
            Some(doc) => (format!(" Detail {} ", doc.id()), detail_lines_for_doc(doc)),
            None => (
                " Detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No item selected in this state.",
                    Style::default().fg(Color::DarkGray),
                ))],
            ),
        };
        let detail = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(focus_style),
            )
            .scroll((self.detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }

    fn draw_footer(&self, frame: &mut Frame<'_>, area: Rect) {
        let (hints, style) = if let Some(input) = self.quick_add.as_ref() {
            (quick_add_status(input), Style::default().fg(Color::Yellow))
        } else {
            let focus = match self.focus {
                FocusPane::Board => "board",
                FocusPane::Detail => "detail",
            };
            (
                format!(
                    "{focus} · q quit · r reload · a add task · h/l state · H/L move task · j/k item/scroll · tab/enter detail · ? help · {}",
                    self.status
                ),
                Style::default().fg(Color::DarkGray),
            )
        };
        let footer = Paragraph::new(Line::from(Span::styled(hints, style)));
        frame.render_widget(footer, area);
    }

    fn draw_help(&self, frame: &mut Frame<'_>, area: Rect) {
        let popup = centered_rect(72, 60, area);
        frame.render_widget(Clear, popup);
        let help = Paragraph::new(vec![
            Line::from(Span::styled(
                "Tandem TUI Board shell",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("q / Ctrl-C        Quit safely"),
            Line::from("r                 Reload .tandem/board"),
            Line::from("a                 Quick-add task in selected/default configured state"),
            Line::from("h/l or ←/→        Move between states"),
            Line::from("H/L               Move selected task to previous/next configured state"),
            Line::from("j/k or ↑/↓        Move between items, or scroll detail when focused"),
            Line::from("g/G               First/last item or detail line"),
            Line::from("tab / enter       Toggle Board vs Detail focus"),
            Line::from("mouse wheel       Move selection or scroll detail"),
            Line::from("click column/detail Select state or focus detail"),
            Line::from("1..5              Show planned view status"),
            Line::from("Quick add         Type a title, Enter creates, Esc cancels"),
            Line::from(""),
            Line::from("This slice supports Board mutations for quick-add and moving tasks between configured states. Review/Logs/Rules/Decisions views, theme files, and richer mouse hit maps remain planned."),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: true });
        frame.render_widget(help, popup);
    }
}

fn read_workspace_title(workspace: &Workspace) -> Result<String, CliError> {
    let root = read_frontmatter_yaml_file(&workspace.config_path)?;
    Ok(root
        .as_ref()
        .and_then(|root| yaml_mapping_value(root, "title"))
        .and_then(yaml_scalar_to_string)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Tandem".to_string()))
}

fn states_with_board_docs(mut states: Vec<String>, docs: &[Document]) -> Vec<String> {
    for doc in docs {
        let state = document_state_label(doc);
        if !states.iter().any(|known| known == &state) {
            states.push(state);
        }
    }
    if states.is_empty() {
        states.push("todo".to_string());
    }
    states
}

fn document_state_label(doc: &Document) -> String {
    doc.field("state")
        .filter(|state| !state.trim().is_empty())
        .unwrap_or("unfiled")
        .to_string()
}

fn quick_add_state_for_selection(
    configured_states: &[String],
    visible_states: &[String],
    selected_state: usize,
) -> (String, Option<String>) {
    let fallback = configured_states
        .first()
        .cloned()
        .unwrap_or_else(|| "todo".to_string());
    let Some(selected) = visible_states.get(selected_state) else {
        return (fallback, Some("no selected state".to_string()));
    };
    if configured_states.iter().any(|state| state == selected) {
        (selected.clone(), None)
    } else {
        (
            fallback,
            Some(format!(
                "selected bucket `{selected}` is not a configured state"
            )),
        )
    }
}

fn quick_add_status(input: &QuickAddInput) -> String {
    let fallback = input
        .fallback_note
        .as_ref()
        .map(|note| format!(" ({note})"))
        .unwrap_or_default();
    let title = if input.title.is_empty() {
        "<title>".to_string()
    } else {
        input.title.clone()
    };
    format!(
        "Add task in {}{}: {} · Enter create · Esc cancel",
        input.state, fallback, title
    )
}

#[derive(Debug)]
struct QuickAddOutcome {
    id: String,
    state: String,
    title: String,
}

fn create_basic_task(
    workspace: &Workspace,
    title: &str,
    state: &str,
) -> Result<QuickAddOutcome, CliError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CliError::usage("task title must not be empty"));
    }
    validate_state(workspace, state)?;

    let task_id = next_sequential_id(workspace, "task")?;
    let now = current_timestamp();
    let task_path = workspace.board_dir.join(format!("{task_id}.md"));
    let content = format!(
        "---\nid: {task_id}\ntype: task\ntitle: {}\nstate: {}\ncreatedAt: {}\nupdatedAt: {}\n---\n\n",
        yaml_double_quote(title),
        yaml_double_quote(state),
        yaml_double_quote(&now),
        yaml_double_quote(&now)
    );
    write_atomic(&task_path, &content)?;
    append_event(workspace, "task.created", &task_id, title)?;

    Ok(QuickAddOutcome {
        id: task_id,
        state: state.to_string(),
        title: title.to_string(),
    })
}

fn adjacent_configured_state(
    configured_states: &[String],
    current_state: Option<&str>,
    delta: isize,
) -> Result<String, String> {
    if configured_states.is_empty() {
        return Err("No configured states are available for task moves.".to_string());
    }
    if configured_states.len() == 1 {
        return Err(format!(
            "Only one configured state (`{}`); selected task cannot move left/right.",
            configured_states[0]
        ));
    }

    let current = current_state
        .map(str::trim)
        .filter(|state| !state.is_empty())
        .unwrap_or("unfiled");
    let Some(current_index) = configured_states.iter().position(|state| state == current) else {
        return Err(format!(
            "Selected item is in `{current}`, which is not a configured state ({}).",
            configured_states.join(", ")
        ));
    };

    let target_index = current_index as isize + delta;
    if target_index < 0 {
        Err(format!(
            "Selected item is already in the first configured state `{current}`."
        ))
    } else if target_index >= configured_states.len() as isize {
        Err(format!(
            "Selected item is already in the last configured state `{current}`."
        ))
    } else {
        Ok(configured_states[target_index as usize].clone())
    }
}

#[derive(Debug)]
struct MoveStateOutcome {
    id: String,
    from: String,
    to: String,
    changed: bool,
}

fn move_task_to_state(
    workspace: &Workspace,
    id: &str,
    state: &str,
) -> Result<MoveStateOutcome, CliError> {
    validate_state(workspace, state)?;

    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be moved in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;

    let doc_id = doc.id().to_string();
    let previous_state = doc.field("state").unwrap_or("-").to_string();
    if previous_state == state {
        return Ok(MoveStateOutcome {
            id: doc_id,
            from: previous_state,
            to: state.to_string(),
            changed: false,
        });
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut updates = BTreeMap::new();
    updates.insert("state".to_string(), state.to_string());
    updates.insert("updatedAt".to_string(), now);
    let patched = patch_frontmatter_content(&content, &updates, &[])?;
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(
        workspace,
        "task.moved",
        &doc_id,
        &format!("Moved {doc_id} from {previous_state} to {state}"),
    )?;

    Ok(MoveStateOutcome {
        id: doc_id,
        from: previous_state,
        to: state.to_string(),
        changed: true,
    })
}

fn list_item_for_doc(doc: &Document) -> ListItem<'static> {
    let priority = doc.field("priority").unwrap_or("-");
    let accord = accord_status(doc).unwrap_or("-");
    let review = review_status(doc).unwrap_or("-");
    let assignee = doc.field("assignee").unwrap_or("-");
    let tags = doc.field("tags").unwrap_or("");
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                format!("{:<4}", truncate(priority, 4).to_uppercase()),
                priority_style(priority),
            ),
            Span::raw(" "),
            Span::styled(
                format!("[{}] ", doc.doc_type()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                truncate(doc.title(), 56),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("{} ", doc.id()), Style::default().fg(Color::Cyan)),
            Span::styled(format!("@{assignee} "), Style::default().fg(Color::Gray)),
            Span::styled(format!("A:{accord} "), accord_style(accord)),
            Span::styled(format!("R:{review} "), review_style(review)),
            Span::styled(tags.to_string(), Style::default().fg(Color::DarkGray)),
        ]),
    ])
}

fn detail_lines_for_doc(doc: &Document) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", label_style()),
        Span::styled(doc.title().to_string(), Style::default().fg(Color::White)),
    ]));
    lines.push(detail_field_line("ID", doc.id()));
    lines.push(detail_field_line("Type", doc.doc_type()));
    push_optional_detail_line(&mut lines, "State", doc.field("state"));
    push_optional_detail_line(&mut lines, "Priority", doc.field("priority"));
    push_optional_detail_line(&mut lines, "Assignee", doc.field("assignee"));
    push_optional_detail_line(&mut lines, "Due", doc.field("dueDate"));
    push_optional_detail_line(&mut lines, "Tags", doc.field("tags"));
    push_optional_detail_line(&mut lines, "Accord", accord_status(doc));
    push_optional_detail_line(&mut lines, "Review", review_status(doc));
    push_optional_detail_line(&mut lines, "Updated", doc.field("updatedAt"));
    lines.push(detail_field_line("Path", &display_path(&doc.path)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Body",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled(
            "(empty)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for line in doc.body.lines() {
            lines.push(markdownish_line(line));
        }
    }
    lines
}

fn detail_field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), label_style()),
        Span::raw(value.to_string()),
    ])
}

fn push_optional_detail_line(lines: &mut Vec<Line<'static>>, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(detail_field_line(label, value));
    }
}

fn markdownish_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Gray),
        ))
    } else if trimmed.starts_with("```") {
        Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::from(line.to_string())
    }
}

fn label_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD)
}

fn priority_style(priority: &str) -> Style {
    match priority.to_ascii_lowercase().as_str() {
        "critical" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "high" => Style::default().fg(Color::LightRed),
        "medium" | "med" => Style::default().fg(Color::Yellow),
        "low" => Style::default().fg(Color::DarkGray),
        _ => Style::default().fg(Color::Gray),
    }
}

fn accord_style(status: &str) -> Style {
    match status {
        "accepted" => Style::default().fg(Color::Green),
        "delivered" => Style::default().fg(Color::Magenta),
        "claimed" => Style::default().fg(Color::Blue),
        "blocked" | "failed" => Style::default().fg(Color::Red),
        "rework" => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Gray),
    }
}

fn review_style(status: &str) -> Style {
    match status {
        "accepted" => Style::default().fg(Color::Green),
        "pending" => Style::default().fg(Color::Yellow),
        "changes-requested" | "failed" => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::Gray),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn doc_with_state(id: &str, state: Option<&str>) -> Document {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), id.to_string());
        fields.insert("type".to_string(), "task".to_string());
        fields.insert("title".to_string(), format!("Task {id}"));
        if let Some(state) = state {
            fields.insert("state".to_string(), state.to_string());
        }
        Document {
            path: PathBuf::from(format!("{id}.md")),
            location: DocumentLocation::Board,
            fields,
            body: String::new(),
        }
    }

    #[test]
    fn states_include_unfiled_and_unknown_board_docs() {
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("blocked")),
            doc_with_state("decision-1", None),
        ];
        let states = states_with_board_docs(vec!["todo".to_string(), "review".to_string()], &docs);
        assert_eq!(states, vec!["todo", "review", "blocked", "unfiled"]);
    }

    #[test]
    fn document_without_state_uses_unfiled_label() {
        let doc = doc_with_state("decision-1", None);
        assert_eq!(document_state_label(&doc), "unfiled");
    }

    #[test]
    fn quick_add_uses_selected_configured_state() {
        let configured = vec!["todo".to_string(), "in-progress".to_string()];
        let visible = vec![
            "todo".to_string(),
            "blocked".to_string(),
            "in-progress".to_string(),
        ];
        assert_eq!(
            quick_add_state_for_selection(&configured, &visible, 2),
            ("in-progress".to_string(), None)
        );
    }

    #[test]
    fn quick_add_falls_back_for_unconfigured_state() {
        let configured = vec!["todo".to_string(), "in-progress".to_string()];
        let visible = vec!["unfiled".to_string()];
        let (state, note) = quick_add_state_for_selection(&configured, &visible, 0);
        assert_eq!(state, "todo");
        assert!(note.unwrap().contains("not a configured state"));
    }

    #[test]
    fn adjacent_configured_state_moves_left_and_right() {
        let states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
        ];
        assert_eq!(
            adjacent_configured_state(&states, Some("in-progress"), -1).unwrap(),
            "todo"
        );
        assert_eq!(
            adjacent_configured_state(&states, Some("in-progress"), 1).unwrap(),
            "review"
        );
    }

    #[test]
    fn adjacent_configured_state_rejects_unconfigured_state() {
        let states = vec!["todo".to_string(), "review".to_string()];
        let error = adjacent_configured_state(&states, Some("blocked"), 1).unwrap_err();
        assert!(error.contains("not a configured state"));
    }
}
