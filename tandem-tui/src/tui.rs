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

mod decisions;
mod logs;
mod review;
mod rules;
mod theme;

use decisions::DecisionsState;
use rules::RulesState;
use theme::{StatusTone, TuiTheme};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TuiView {
    Board,
    Review,
    Logs,
    Rules,
    Decisions,
}

impl TuiView {
    const ALL: [Self; 5] = [
        Self::Board,
        Self::Review,
        Self::Logs,
        Self::Rules,
        Self::Decisions,
    ];

    fn index(self) -> usize {
        match self {
            Self::Board => 0,
            Self::Review => 1,
            Self::Logs => 2,
            Self::Rules => 3,
            Self::Decisions => 4,
        }
    }

    fn from_digit(ch: char) -> Option<Self> {
        match ch {
            '1' => Some(Self::Board),
            '2' => Some(Self::Review),
            '3' => Some(Self::Logs),
            '4' => Some(Self::Rules),
            '5' => Some(Self::Decisions),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Review => "Review",
            Self::Logs => "Logs",
            Self::Rules => "Rules",
            Self::Decisions => "Decisions",
        }
    }

    fn tab_label(self) -> &'static str {
        match self {
            Self::Board => "1 Board",
            Self::Review => "2 Review",
            Self::Logs => "3 Logs",
            Self::Rules => "4 Rules",
            Self::Decisions => "5 Decisions",
        }
    }
}

#[derive(Debug, Clone)]
enum HitAction {
    SwitchView(TuiView),
    SelectState(usize),
    SelectBoardItem(usize, usize),
    FocusDetail,
    FocusReviewList,
    SelectReviewItem(usize),
    FocusReviewDetail,
    SelectLog(usize),
    FocusLogList,
    FocusLogDetail,
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
    view: TuiView,
    states: Vec<String>,
    configured_states: Vec<String>,
    docs: Vec<Document>,
    logs: Vec<Document>,
    log_events: logs::LogEventsById,
    rules: RulesByCategory,
    load_errors: Vec<String>,
    theme: TuiTheme,
    theme_source: String,
    theme_warnings: Vec<String>,
    selected_state: usize,
    selected_item: usize,
    selected_review_item: usize,
    selected_log: usize,
    focus: FocusPane,
    detail_scroll: u16,
    review_detail_scroll: u16,
    log_detail_scroll: u16,
    log_search_filter: String,
    log_search_input: Option<String>,
    status: String,
    show_help: bool,
    quick_add: Option<QuickAddInput>,
    rules_view: RulesState,
    decisions_view: DecisionsState,
    hits: Vec<HitRegion>,
}

impl TuiApp {
    fn load(workspace: Workspace) -> Result<Self, CliError> {
        let mut app = Self {
            workspace,
            title: String::new(),
            view: TuiView::Board,
            states: Vec::new(),
            configured_states: Vec::new(),
            docs: Vec::new(),
            logs: Vec::new(),
            log_events: logs::LogEventsById::new(),
            rules: empty_rules(),
            load_errors: Vec::new(),
            theme: TuiTheme::default_dark(),
            theme_source: String::new(),
            theme_warnings: Vec::new(),
            selected_state: 0,
            selected_item: 0,
            selected_review_item: 0,
            selected_log: 0,
            focus: FocusPane::Board,
            detail_scroll: 0,
            review_detail_scroll: 0,
            log_detail_scroll: 0,
            log_search_filter: String::new(),
            log_search_input: None,
            status: String::new(),
            show_help: false,
            quick_add: None,
            rules_view: RulesState::default(),
            decisions_view: DecisionsState::default(),
            hits: Vec::new(),
        };
        app.reload()?;
        Ok(app)
    }

    fn reload(&mut self) -> Result<(), CliError> {
        let mut docs = read_documents(&self.workspace.board_dir, DocumentLocation::Board)?;
        sort_documents(&mut docs);
        let configured_states = read_workspace_states(&self.workspace)?;
        let theme_load = TuiTheme::load_for_workspace(&self.workspace);
        self.title = read_workspace_title(&self.workspace)?;

        let mut load_errors = Vec::new();
        let log_load = logs::load_logs(&self.workspace.logs_dir);
        load_errors.extend(log_load.warnings);
        let (log_events, event_warnings) = logs::load_log_events(&self.workspace.events_path);
        load_errors.extend(event_warnings);

        let rules = match read_rules(&self.workspace.config_path) {
            Ok(rules) => rules,
            Err(error) => {
                load_errors.push(format!("Rules load failed: {}", error.message));
                empty_rules()
            }
        };

        self.states = states_with_board_docs(configured_states.clone(), &docs);
        self.configured_states = configured_states;
        self.docs = docs;
        self.logs = log_load.docs;
        self.log_events = log_events;
        self.rules = rules;
        self.load_errors = load_errors;
        self.theme = theme_load.theme;
        self.theme_source = theme_load.source;
        self.theme_warnings = theme_load.warnings;
        self.clamp_selection();
        self.clamp_rules_state();
        self.clamp_decisions_state();
        let theme_note = if self.theme_warnings.is_empty() {
            format!("theme {}", self.theme.source_label(&self.theme_source))
        } else {
            format!(
                "theme {} ({} warning{})",
                self.theme.source_label(&self.theme_source),
                self.theme_warnings.len(),
                if self.theme_warnings.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        let load_note = if self.load_errors.is_empty() {
            String::new()
        } else {
            format!(
                "; {} load warning{}",
                self.load_errors.len(),
                if self.load_errors.len() == 1 { "" } else { "s" }
            )
        };
        self.status = format!(
            "Loaded {} active document{} from {} · {}{}",
            self.docs.len(),
            if self.docs.len() == 1 { "" } else { "s" },
            display_path(&self.workspace.board_dir),
            theme_note,
            load_note
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

        if self.log_search_input.is_some() {
            self.handle_log_search_key(key);
            return Ok(false);
        }

        if self.rules_prompt_active() {
            self.handle_rules_prompt_key(key);
            return Ok(false);
        }

        if self.decision_prompt_active() {
            self.handle_decision_prompt_key(key);
            return Ok(false);
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.show_help = false,
                _ => {}
            }
            return Ok(false);
        }

        if let KeyCode::Char(ch) = key.code {
            if let Some(view) = TuiView::from_digit(ch) {
                self.switch_view(view);
                return Ok(false);
            }
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('r') => match self.reload() {
                Ok(()) => {}
                Err(error) => self.status = format!("Reload failed: {}", error.message),
            },
            KeyCode::Char('a') if self.view == TuiView::Board => self.start_quick_add(),
            KeyCode::Char('a') if self.view == TuiView::Rules => self.start_rule_add_prompt(),
            KeyCode::Char('a') if self.view == TuiView::Decisions => {
                self.start_decision_add_prompt()
            }
            KeyCode::Char('a') => {
                self.status = "Add is available in Board, Rules, and Decisions views.".to_string()
            }
            KeyCode::Char('H') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(-1)
            }
            KeyCode::Char('L') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(1)
            }
            KeyCode::Char('H') | KeyCode::Char('L') => {
                self.status = "Task move is available in Board view; press 1 for Board.".to_string()
            }
            KeyCode::Char('/') if self.view == TuiView::Logs => self.start_log_search(),
            KeyCode::Char('/') => {
                self.status = "Search is available in Logs view; press 3 for Logs.".to_string()
            }
            KeyCode::Tab | KeyCode::BackTab => self.cycle_focus_or_hint(),
            KeyCode::Enter
                if matches!(self.view, TuiView::Board | TuiView::Review | TuiView::Logs) =>
            {
                self.toggle_focus()
            }
            KeyCode::Esc => match self.view {
                TuiView::Board | TuiView::Review if self.focus == FocusPane::Detail => {
                    self.focus = FocusPane::Board
                }
                TuiView::Logs => self.clear_log_filter_or_focus(),
                TuiView::Decisions if self.focus == FocusPane::Detail => {
                    self.focus = FocusPane::Board
                }
                _ => {}
            },
            _ => match self.view {
                TuiView::Board => match self.focus {
                    FocusPane::Board => self.handle_board_key(key),
                    FocusPane::Detail => self.handle_detail_key(key),
                },
                TuiView::Review => match self.focus {
                    FocusPane::Board => self.handle_review_key(key),
                    FocusPane::Detail => self.handle_review_detail_key(key),
                },
                TuiView::Logs => self.handle_logs_key(key),
                TuiView::Rules => self.handle_rules_key(key),
                TuiView::Decisions => self.handle_decisions_key(key),
            },
        }
        Ok(false)
    }

    fn switch_view(&mut self, view: TuiView) {
        self.view = view;
        self.focus = FocusPane::Board;
        if view == TuiView::Review {
            self.clamp_review_selection();
        }
        if view == TuiView::Logs {
            self.clamp_selection();
        }
        if view == TuiView::Rules {
            self.clamp_rules_state();
        }
        if view == TuiView::Decisions {
            self.clamp_decisions_state();
        }
        self.status = match view {
            TuiView::Board => {
                "Board view active. Use h/l or mouse tabs for state subviews, a to quick-add, and H/L to move tasks.".to_string()
            }
            TuiView::Review => {
                let count = self.review_items().len();
                let selected = self
                    .selected_review_item()
                    .map(|item| format!(" Selected {}.", item.id()))
                    .unwrap_or_default();
                format!(
                    "Review view active: {count} item{} need attention.{selected} Use j/k to navigate and Tab/Enter for queue/detail focus.",
                    if count == 1 { "" } else { "s" }
                )
            }
            TuiView::Logs => self.logs_status_message(),
            TuiView::Rules => format!(
                "Rules view active: {} project rule{} loaded. Use j/k select, h/l category, a/e/d add/edit/delete; Tab has no top-level fallback.",
                self.rules_total(),
                if self.rules_total() == 1 { "" } else { "s" }
            ),
            TuiView::Decisions => format!(
                "Decisions view active: {} decision{} loaded. Use j/k select, h/l or Tab for list/body focus, a add.",
                self.decision_docs().len(),
                if self.decision_docs().len() == 1 {
                    ""
                } else {
                    "s"
                }
            ),
        };
    }

    fn cycle_focus_or_hint(&mut self) {
        match self.view {
            TuiView::Board | TuiView::Review | TuiView::Logs | TuiView::Decisions => {
                self.toggle_focus()
            }
            TuiView::Rules => {
                self.status = "Rules has a single category/list focus area; Tab stays in Rules. Use h/l for categories and 1..5 for views.".to_string();
            }
        }
    }

    fn focus_previous_pane(&mut self) {
        if matches!(
            self.view,
            TuiView::Review | TuiView::Logs | TuiView::Decisions
        ) {
            self.focus = FocusPane::Board;
        }
    }

    fn focus_next_pane(&mut self) {
        if matches!(
            self.view,
            TuiView::Review | TuiView::Logs | TuiView::Decisions
        ) {
            self.focus = FocusPane::Detail;
        }
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

    fn handle_review_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.previous_review_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_review_item(),
            KeyCode::Home | KeyCode::Char('g') => self.selected_review_item = 0,
            KeyCode::End | KeyCode::Char('G') => self.last_review_item(),
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
        self.clamp_review_selection();
    }

    fn handle_review_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.scroll_review_detail_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_review_detail_down(1),
            KeyCode::PageUp | KeyCode::Char('u') => self.scroll_review_detail_up(6),
            KeyCode::PageDown | KeyCode::Char('d') => self.scroll_review_detail_down(6),
            KeyCode::Home | KeyCode::Char('g') => self.review_detail_scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.review_detail_scroll_to_end(),
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
    }

    fn handle_logs_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                FocusPane::Board => self.previous_log(),
                FocusPane::Detail => self.scroll_log_detail_up(1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                FocusPane::Board => self.next_log(),
                FocusPane::Detail => self.scroll_log_detail_down(1),
            },
            KeyCode::PageUp | KeyCode::Char('u') => match self.focus {
                FocusPane::Board => self.previous_log_page(),
                FocusPane::Detail => self.scroll_log_detail_up(6),
            },
            KeyCode::PageDown | KeyCode::Char('d') => match self.focus {
                FocusPane::Board => self.next_log_page(),
                FocusPane::Detail => self.scroll_log_detail_down(6),
            },
            KeyCode::Home | KeyCode::Char('g') => match self.focus {
                FocusPane::Board => {
                    self.selected_log = 0;
                    self.log_detail_scroll = 0;
                }
                FocusPane::Detail => self.log_detail_scroll = 0,
            },
            KeyCode::End | KeyCode::Char('G') => match self.focus {
                FocusPane::Board => self.last_log(),
                FocusPane::Detail => self.log_detail_scroll_to_end(),
            },
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
            _ => {}
        }
    }

    fn handle_log_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.log_search_input = None;
                if self.log_search_filter.is_empty() {
                    self.status = "Log search canceled.".to_string();
                } else {
                    self.status = self.logs_status_message();
                }
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => self.finish_log_search(),
            KeyCode::Char('m') | KeyCode::Char('j')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.finish_log_search()
            }
            KeyCode::Backspace => {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.pop();
                }
                self.refresh_log_search_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.clear();
                }
                self.refresh_log_search_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(input) = self.log_search_input.as_mut() {
                    input.push(ch);
                }
                self.refresh_log_search_status();
            }
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

    fn start_log_search(&mut self) {
        self.log_search_input = Some(self.log_search_filter.clone());
        self.focus = FocusPane::Board;
        self.refresh_log_search_status();
    }

    fn refresh_log_search_status(&mut self) {
        let query = self.log_search_input.as_deref().unwrap_or("");
        self.status = format!(
            "Search logs: {} · type filter, Enter apply, Esc cancel",
            if query.is_empty() { "<query>" } else { query }
        );
    }

    fn finish_log_search(&mut self) {
        let query = self
            .log_search_input
            .take()
            .unwrap_or_default()
            .trim()
            .to_string();
        self.log_search_filter = query;
        self.selected_log = 0;
        self.log_detail_scroll = 0;
        self.clamp_selection();
        self.status = self.logs_status_message();
    }

    fn clear_log_filter_or_focus(&mut self) {
        if !self.log_search_filter.is_empty() {
            self.log_search_filter.clear();
            self.selected_log = 0;
            self.log_detail_scroll = 0;
            self.status = "Cleared Logs search filter.".to_string();
            self.clamp_selection();
        } else if self.focus == FocusPane::Detail {
            self.focus = FocusPane::Board;
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
        if self.quick_add.is_some()
            || self.log_search_input.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
            || self.show_help
        {
            return;
        }

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
                        HitAction::SwitchView(view) => self.switch_view(view),
                        HitAction::SelectState(index) if self.view == TuiView::Board => {
                            self.selected_state = index.min(self.states.len().saturating_sub(1));
                            self.selected_item = 0;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::SelectState(_) => {}
                        HitAction::SelectBoardItem(state_index, item_index)
                            if self.view == TuiView::Board =>
                        {
                            self.selected_state =
                                state_index.min(self.states.len().saturating_sub(1));
                            self.selected_item = item_index;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::SelectBoardItem(_, _) => {}
                        HitAction::FocusDetail if self.view == TuiView::Board => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusDetail => {}
                        HitAction::FocusReviewList if self.view == TuiView::Review => {
                            self.focus = FocusPane::Board
                        }
                        HitAction::FocusReviewList => {}
                        HitAction::SelectReviewItem(index) if self.view == TuiView::Review => {
                            self.selected_review_item = index;
                            self.review_detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_review_selection();
                        }
                        HitAction::SelectReviewItem(_) => {}
                        HitAction::FocusReviewDetail if self.view == TuiView::Review => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusReviewDetail => {}
                        HitAction::SelectLog(index) if self.view == TuiView::Logs => {
                            self.selected_log = index;
                            self.log_detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                        }
                        HitAction::SelectLog(_) => {}
                        HitAction::FocusLogList if self.view == TuiView::Logs => {
                            self.focus = FocusPane::Board
                        }
                        HitAction::FocusLogList => {}
                        HitAction::FocusLogDetail if self.view == TuiView::Logs => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusLogDetail => {}
                    }
                }
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Board => match self.focus {
                FocusPane::Board => self.next_item(),
                FocusPane::Detail => self.scroll_detail_down(3),
            },
            MouseEventKind::ScrollUp if self.view == TuiView::Board => match self.focus {
                FocusPane::Board => self.previous_item(),
                FocusPane::Detail => self.scroll_detail_up(3),
            },
            MouseEventKind::ScrollDown if self.view == TuiView::Review => match self.focus {
                FocusPane::Board => self.next_review_item(),
                FocusPane::Detail => self.scroll_review_detail_down(3),
            },
            MouseEventKind::ScrollUp if self.view == TuiView::Review => match self.focus {
                FocusPane::Board => self.previous_review_item(),
                FocusPane::Detail => self.scroll_review_detail_up(3),
            },
            MouseEventKind::ScrollDown if self.view == TuiView::Logs => match self.focus {
                FocusPane::Board => self.next_log(),
                FocusPane::Detail => self.scroll_log_detail_down(3),
            },
            MouseEventKind::ScrollUp if self.view == TuiView::Logs => match self.focus {
                FocusPane::Board => self.previous_log(),
                FocusPane::Detail => self.scroll_log_detail_up(3),
            },
            MouseEventKind::ScrollDown if self.view == TuiView::Rules => self.next_rule_selection(),
            MouseEventKind::ScrollUp if self.view == TuiView::Rules => {
                self.previous_rule_selection()
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Decisions => {
                self.next_decision_selection()
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Decisions => {
                self.previous_decision_selection()
            }
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

    fn previous_log(&mut self) {
        if self.selected_log > 0 {
            self.selected_log -= 1;
            self.log_detail_scroll = 0;
        }
    }

    fn next_log(&mut self) {
        let count = self.filtered_logs().len();
        if self.selected_log + 1 < count {
            self.selected_log += 1;
            self.log_detail_scroll = 0;
        }
    }

    fn previous_log_page(&mut self) {
        self.selected_log = self.selected_log.saturating_sub(5);
        self.log_detail_scroll = 0;
    }

    fn next_log_page(&mut self) {
        let count = self.filtered_logs().len();
        if count > 0 {
            self.selected_log = (self.selected_log + 5).min(count - 1);
            self.log_detail_scroll = 0;
        }
    }

    fn last_log(&mut self) {
        let count = self.filtered_logs().len();
        if count > 0 {
            self.selected_log = count - 1;
            self.log_detail_scroll = 0;
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

    fn previous_review_item(&mut self) {
        if self.selected_review_item > 0 {
            self.selected_review_item -= 1;
            self.review_detail_scroll = 0;
        }
    }

    fn next_review_item(&mut self) {
        let count = self.review_items().len();
        if self.selected_review_item + 1 < count {
            self.selected_review_item += 1;
            self.review_detail_scroll = 0;
        }
    }

    fn last_review_item(&mut self) {
        let count = self.review_items().len();
        if count > 0 {
            self.selected_review_item = count - 1;
            self.review_detail_scroll = 0;
        }
    }

    fn scroll_review_detail_up(&mut self, amount: u16) {
        self.review_detail_scroll = self.review_detail_scroll.saturating_sub(amount);
    }

    fn scroll_review_detail_down(&mut self, amount: u16) {
        let max_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
        self.review_detail_scroll = self
            .review_detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    fn review_detail_scroll_to_end(&mut self) {
        self.review_detail_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
    }

    fn scroll_log_detail_up(&mut self, amount: u16) {
        self.log_detail_scroll = self.log_detail_scroll.saturating_sub(amount);
    }

    fn scroll_log_detail_down(&mut self, amount: u16) {
        let max_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
        self.log_detail_scroll = self
            .log_detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    fn log_detail_scroll_to_end(&mut self) {
        self.log_detail_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
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
        self.clamp_review_selection();

        let log_count = self.filtered_logs().len();
        if log_count == 0 {
            self.selected_log = 0;
        } else if self.selected_log >= log_count {
            self.selected_log = log_count - 1;
        }
        let max_log_scroll = self.log_detail_line_count().saturating_sub(1) as u16;
        self.log_detail_scroll = self.log_detail_scroll.min(max_log_scroll);
    }

    fn clamp_review_selection(&mut self) {
        let count = review::queue_len(&self.docs);
        if count == 0 {
            self.selected_review_item = 0;
        } else if self.selected_review_item >= count {
            self.selected_review_item = count - 1;
        }
        let max_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
        self.review_detail_scroll = self.review_detail_scroll.min(max_scroll);
    }

    fn selected_state_count(&self) -> usize {
        self.states
            .get(self.selected_state)
            .map(|state| self.docs_for_state(state).len())
            .unwrap_or(0)
    }

    fn selected_state_progress(&self) -> String {
        let Some(state) = self.states.get(self.selected_state) else {
            return "NO STATE 0/0".to_string();
        };
        let count = self.selected_state_count();
        let position = if count == 0 {
            0
        } else {
            self.selected_item.min(count - 1) + 1
        };
        format!("{} {position}/{count}", display_state_label(state))
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
            .map(|doc| detail_lines_for_doc(doc, &self.theme))
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn filtered_logs(&self) -> Vec<&Document> {
        logs::filter_logs(&self.logs, &self.log_search_filter)
    }

    fn selected_log(&self) -> Option<&Document> {
        self.filtered_logs().into_iter().nth(self.selected_log)
    }

    fn log_events_for(&self, id: &str) -> &[logs::LogEvent] {
        self.log_events.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    fn log_detail_line_count(&self) -> usize {
        self.selected_log()
            .map(|doc| logs::detail_lines_for_log(doc, self.log_events_for(doc.id()), &self.theme))
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn logs_status_message(&self) -> String {
        let visible = self.filtered_logs().len();
        if self.log_search_filter.is_empty() {
            format!(
                "Logs view active: {} completed item{} loaded. Press / to search, j/k to select, and h/l or Tab for list/detail focus.",
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Logs filter `{}` matched {} of {} completed item{}; Esc clears filter.",
                self.log_search_filter,
                visible,
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        }
    }

    fn review_items(&self) -> Vec<review::ReviewQueueItem> {
        review::queue_items(&self.docs)
    }

    fn selected_review_item(&self) -> Option<review::ReviewQueueItem> {
        review::selected_item(&self.docs, self.selected_review_item)
    }

    fn review_detail_line_count(&self) -> usize {
        let item = self.selected_review_item();
        review::detail_line_count(item.as_ref(), &self.theme)
    }

    fn decision_docs(&self) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| doc.doc_type() == "decision")
            .collect()
    }

    fn rules_total(&self) -> usize {
        self.rules.values().map(Vec::len).sum()
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.hits.clear();
        let area = frame.area();
        frame.render_widget(Block::default().style(self.theme.app_style()), area);
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
        if self.view == TuiView::Board {
            self.draw_board(frame, chunks[2]);
            self.draw_detail(frame, chunks[3]);
        } else {
            let view_area = Rect {
                x: chunks[2].x,
                y: chunks[2].y,
                width: chunks[2].width,
                height: chunks[2].height.saturating_add(chunks[3].height),
            };
            if self.view == TuiView::Review {
                self.draw_review(frame, view_area);
            } else if self.view == TuiView::Logs {
                self.draw_logs(frame, view_area);
            } else if self.view == TuiView::Rules {
                self.draw_rules_view(frame, view_area);
            } else if self.view == TuiView::Decisions {
                self.draw_decisions_view(frame, view_area);
            } else {
                self.draw_placeholder_view(frame, view_area);
            }
        }
        self.draw_footer(frame, chunks[4]);

        if self.rules_prompt_active() {
            self.draw_rules_prompt(frame, area);
        }

        if self.decision_prompt_active() {
            self.draw_decision_prompt(frame, area);
        }

        if self.show_help {
            self.draw_help(frame, area);
        }
    }

    fn draw_tiny(&self, frame: &mut Frame<'_>, area: Rect) {
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
                .title(" tdm tui ")
                .border_style(self.theme.border_style(true))
                .style(self.theme.panel_style()),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
    }

    fn draw_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let counts = format!(
            "Board {} · Review {} · Logs {} · Rules {} · Decisions {}",
            self.docs.len(),
            self.review_items().len(),
            self.logs.len(),
            self.rules_total(),
            self.decision_docs().len()
        );
        let context = match self.view {
            TuiView::Board => self
                .selected_doc()
                .map(|doc| format!("selected {}", doc.id()))
                .unwrap_or_else(|| "no selected item".to_string()),
            TuiView::Review => self
                .selected_review_item()
                .map(|item| format!("selected {} · {}", item.id(), item.reason_summary()))
                .unwrap_or_else(|| "review queue is empty".to_string()),
            TuiView::Logs => {
                let filter = if self.log_search_filter.is_empty() {
                    String::new()
                } else {
                    format!(" · filter `{}`", self.log_search_filter)
                };
                self.selected_log()
                    .map(|doc| {
                        format!(
                            "selected {} completed {}{}",
                            doc.id(),
                            doc.field("completedAt").unwrap_or("unknown"),
                            filter
                        )
                    })
                    .unwrap_or_else(|| format!("no completed log selected{filter}"))
            }
            TuiView::Rules => self.rules_context(),
            TuiView::Decisions => self.decisions_context(),
        };
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(self.title.clone(), self.theme.title_style()),
                Span::raw("  "),
                Span::styled(
                    format!("{} view", self.view.label()),
                    self.theme.text_style().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(counts, self.theme.muted_style()),
            ]),
            Line::from(Span::styled(context, self.theme.muted_style())),
        ])
        .style(self.theme.panel_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Tandem ")
                .border_style(self.theme.border_style(false))
                .style(self.theme.panel_style()),
        );
        frame.render_widget(header, area);
    }

    fn draw_view_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let titles = TuiView::ALL
            .into_iter()
            .map(|view| Line::from(view.tab_label()))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(false))
                    .style(self.theme.panel_style()),
            )
            .select(self.view.index())
            .style(self.theme.tab_style())
            .highlight_style(self.theme.tab_selected_style());
        frame.render_widget(tabs, area);
        self.register_view_tab_hits(area);
    }

    fn register_view_tab_hits(&mut self, area: Rect) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let mut x = area.x.saturating_add(1);
        let right = area.x.saturating_add(area.width).saturating_sub(1);
        let y = area.y.saturating_add(1);
        for view in TuiView::ALL {
            let width = (view.tab_label().chars().count() as u16).saturating_add(2);
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

    fn draw_review(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let items = self.review_items();
        review::render_review(
            frame,
            area,
            &items,
            self.selected_review_item,
            self.focus,
            self.review_detail_scroll,
            &self.theme,
            &self.load_errors,
            &mut self.hits,
        );
    }

    fn draw_logs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if area.width >= 100 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(area);
            self.draw_log_list(frame, chunks[0]);
            self.draw_log_detail(frame, chunks[1]);
        } else {
            let detail_height = (area.height / 2).max(6);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(detail_height)])
                .split(area);
            self.draw_log_list(frame, chunks[0]);
            self.draw_log_detail(frame, chunks[1]);
        }
    }

    fn draw_log_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusLogList,
        });

        let filtered = self.filtered_logs();
        let count = filtered.len();
        let title = if self.log_search_filter.is_empty() {
            format!(" Logs ({count}/{}) ", self.logs.len())
        } else {
            format!(
                " Logs filter `{}` ({count}/{}) ",
                self.log_search_filter,
                self.logs.len()
            )
        };
        let items = if self.logs.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                format!(
                    "No completed logs found in {}.",
                    display_path(&self.workspace.logs_dir)
                ),
                self.theme.muted_style(),
            )))]
        } else if filtered.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                format!(
                    "No logs match `{}`. Press Esc to clear.",
                    self.log_search_filter
                ),
                self.theme.muted_style(),
            )))]
        } else {
            filtered
                .iter()
                .map(|doc| logs::list_item_for_log(doc, &self.theme))
                .collect::<Vec<_>>()
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
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");

        if count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_log.min(count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            drop(filtered);
            self.register_log_row_hits(area, count);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn register_log_row_hits(&mut self, area: Rect, count: usize) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let top = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let bottom = area.y.saturating_add(area.height).saturating_sub(1);
        for index in 0..count {
            let y = top.saturating_add((index as u16).saturating_mul(2));
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: 2.min(bottom.saturating_sub(y)),
                },
                action: HitAction::SelectLog(index),
            });
        }
    }

    fn draw_log_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusLogDetail,
        });

        let focused = self.focus == FocusPane::Detail;
        let (title, lines) = match self.selected_log() {
            Some(doc) => (
                format!(" Log detail {} ", doc.id()),
                logs::detail_lines_for_log(doc, self.log_events_for(doc.id()), &self.theme),
            ),
            None if self.logs.is_empty() => (
                " Log detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No completed logs are available yet. Complete a task to create one.",
                    self.theme.muted_style(),
                ))],
            ),
            None => (
                " Log detail ".to_string(),
                vec![Line::from(Span::styled(
                    "No log matches the current filter.",
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
            .scroll((self.log_detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }

    fn draw_placeholder_view(&self, frame: &mut Frame<'_>, area: Rect) {
        let (title, lines) = match self.view {
            TuiView::Board => (" Board ".to_string(), Vec::new()),
            TuiView::Review => self.review_placeholder_lines(),
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

    fn review_placeholder_lines(&self) -> (String, Vec<Line<'static>>) {
        (
            " Review ".to_string(),
            vec![Line::from(
                "Review queue renders in the dedicated Review view.",
            )],
        )
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

    fn draw_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.draw_state_tabs(frame, area);
    }

    fn draw_state_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let subviews = board_subview_tabs(&self.states, &self.docs);
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

    fn register_state_tab_hits(&mut self, area: Rect, subviews: &[BoardSubviewTab]) {
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

    fn draw_state_list(&mut self, frame: &mut Frame<'_>, area: Rect, state_index: usize) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::SelectState(state_index),
        });

        let Some(state_name) = self.states.get(state_index) else {
            return;
        };
        let docs = self.docs_for_state(state_name);
        let count = docs.len();
        let content_width = area.width.saturating_sub(4) as usize;
        let items = if docs.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No active items in this state. Press a to quick-add here.",
                self.theme.muted_style(),
            )))]
        } else {
            docs.iter()
                .map(|doc| list_item_for_doc(doc, &self.theme, content_width))
                .collect::<Vec<_>>()
        };

        let title = format!(
            " {} · selected state {}/{} · {} item{} ",
            display_state_label(state_name),
            state_index + 1,
            self.states.len(),
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
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");

        if count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_item.min(count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            self.register_board_row_hits(area, state_index, count);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn register_board_row_hits(&mut self, area: Rect, state_index: usize, count: usize) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let top = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let bottom = area.y.saturating_add(area.height).saturating_sub(1);
        for index in 0..count {
            let y = top.saturating_add((index as u16).saturating_mul(2));
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: 2.min(bottom.saturating_sub(y)),
                },
                action: HitAction::SelectBoardItem(state_index, index),
            });
        }
    }

    fn draw_detail(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::FocusDetail,
        });

        let focused = self.focus == FocusPane::Detail;
        let (title, lines) = match self.selected_doc() {
            Some(doc) => (
                format!(" Detail {} ", doc.id()),
                detail_lines_for_doc(doc, &self.theme),
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

    fn draw_footer(&self, frame: &mut Frame<'_>, area: Rect) {
        let (hints, style) = if let Some(input) = self.quick_add.as_ref() {
            (
                quick_add_status(input),
                self.theme.status_style(StatusTone::Warning),
            )
        } else if self.log_search_input.is_some() {
            (
                self.status.clone(),
                self.theme.status_style(StatusTone::Warning),
            )
        } else if let Some(status) = self.rules_prompt_status() {
            (status, self.theme.status_style(StatusTone::Warning))
        } else if let Some(status) = self.decision_prompt_status() {
            (status, self.theme.status_style(StatusTone::Warning))
        } else if self.view == TuiView::Board {
            let focus = match self.focus {
                FocusPane::Board => "board",
                FocusPane::Detail => "detail",
            };
            (
                format!(
                    "{focus} · {} · 1..5 views · q quit · r reload · a add task · h/l state subview · H/L move task · j/k item/scroll · Tab/Enter detail · ? help · {}",
                    self.selected_state_progress(),
                    self.status
                ),
                self.theme.status_style(status_tone_for_message(&self.status)),
            )
        } else if self.view == TuiView::Review {
            let focus = match self.focus {
                FocusPane::Board => "queue",
                FocusPane::Detail => "detail",
            };
            (
                format!(
                    "Review {focus} · 1..5 views · q quit · r reload · j/k item/scroll · h/l or Tab queue/detail · Enter detail · read-only hints · ? help · {}",
                    self.status
                ),
                self.theme.status_style(status_tone_for_message(&self.status)),
            )
        } else if self.view == TuiView::Logs {
            let focus = match self.focus {
                FocusPane::Board => "list",
                FocusPane::Detail => "detail",
            };
            let filter = if self.log_search_filter.is_empty() {
                String::new()
            } else {
                format!("filter `{}` · Esc clear · ", self.log_search_filter)
            };
            (
                format!(
                    "Logs {focus} · {filter}/ search · j/k select/scroll · g/G top/bottom · h/l or Tab list/detail · Enter focus detail/list · r reload · q quit · {}",
                    self.status
                ),
                self.theme.status_style(status_tone_for_message(&self.status)),
            )
        } else if self.view == TuiView::Rules {
            (
                self.rules_footer_text(),
                self.theme
                    .status_style(status_tone_for_message(&self.status)),
            )
        } else if self.view == TuiView::Decisions {
            (
                self.decisions_footer_text(),
                self.theme
                    .status_style(status_tone_for_message(&self.status)),
            )
        } else {
            (
                format!(
                    "{} · 1..5 switch views · local keys stay in view · r reload · q quit · ? help · {}",
                    self.view.label(),
                    self.status
                ),
                self.theme.status_style(status_tone_for_message(&self.status)),
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
                "Tandem TUI view shell",
                self.theme.title_style(),
            )),
            Line::from(""),
            Line::from("q / Ctrl-C        Quit safely"),
            Line::from("r                 Reload board/log/rule data"),
            Line::from("1..5              Switch Board, Review, Logs, Rules, Decisions"),
            Line::from("click top tabs    Switch views with the mouse"),
            Line::from("tab / shift-tab   Cycle list/detail focus where available; never switches views"),
            Line::from("enter             Toggle list/detail focus in Board, Review, and Logs"),
            Line::from("a                 Board quick-add; Rules add rule; Decisions add decision"),
            Line::from("h/l or ←/→        Board: state subviews; Review/Logs/Decisions: list/detail focus; Rules: category"),
            Line::from("H/L               Board: move selected task to previous/next configured state"),
            Line::from("j/k or ↑/↓        Board/Review/Logs/Rules/Decisions: move items, or scroll detail when focused"),
            Line::from("g/G               First/last item in the active list/detail"),
            Line::from("e / d             Rules: edit selected rule / delete with confirmation"),
            Line::from("PgUp/PgDn         Logs/Decisions: scroll selected detail/body"),
            Line::from("/                 Logs: search by id, title, summary, body, validation, files"),
            Line::from("Esc               Logs: clear search filter; prompts: cancel"),
            Line::from("mouse wheel       Board/Review/Logs/Rules/Decisions: move selection or scroll detail"),
            Line::from("click tabs/list/detail Board/Review/Logs: switch subviews, select, or focus panes"),
            Line::from("Prompts           Type text, Enter advances or saves, Esc cancels, Ctrl-U clears field"),
            Line::from("Log search        Type a query, Enter applies, Esc cancels"),
            Line::from(""),
            Line::from("Board state subviews, quick-add/H/L moves, Review queue, Logs browser/search, Rules add/edit/delete, and Decisions browse/add are active. Built-in presets, XDG/~/.config user themes, and .tandem/theme.toml selectors/overrides are active; richer action buttons remain planned."),
        ])
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

#[cfg(test)]
fn review_attention_reason(doc: &Document) -> Option<String> {
    match accord_status(doc) {
        Some("delivered") => return Some("accord delivered".to_string()),
        Some("blocked") => return Some("accord blocked".to_string()),
        Some("failed") => return Some("accord failed".to_string()),
        Some("rework") => return Some("accord in rework".to_string()),
        Some("accepted") => return Some("accord accepted; not completed".to_string()),
        _ => {}
    }

    match review_status(doc) {
        Some("pending") => Some("review pending".to_string()),
        Some("changes-requested") => Some("changes requested".to_string()),
        Some("rejected") => Some("review rejected".to_string()),
        Some("failed") => Some("review failed".to_string()),
        _ if doc
            .field("blockers")
            .map(parse_field_values)
            .map(|blockers| !blockers.is_empty())
            .unwrap_or(false) =>
        {
            Some("has blockers".to_string())
        }
        _ => None,
    }
}

fn append_load_error_lines(lines: &mut Vec<Line<'static>>, load_errors: &[String]) {
    if load_errors.is_empty() {
        return;
    }
    lines.push(Line::from(Span::styled(
        "Load warnings:",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));
    for error in load_errors {
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(""));
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardSubviewTab {
    state: String,
    count: usize,
}

fn board_subview_tabs(states: &[String], docs: &[Document]) -> Vec<BoardSubviewTab> {
    states
        .iter()
        .map(|state| BoardSubviewTab {
            state: state.clone(),
            count: docs
                .iter()
                .filter(|doc| document_state_label(doc) == state.as_str())
                .count(),
        })
        .collect()
}

fn state_tab_title(state: &str, count: usize) -> String {
    format!(" {} {} ", display_state_label(state), count)
}

fn display_state_label(state: &str) -> String {
    state
        .trim()
        .replace('-', " ")
        .replace('_', " ")
        .to_uppercase()
}

fn list_item_for_doc(doc: &Document, theme: &TuiTheme, content_width: usize) -> ListItem<'static> {
    let priority = doc.field("priority").unwrap_or("-");
    let priority_badge = priority_badge(priority);
    let type_badge = format!("[{}]", doc.doc_type());
    let mut badges = Vec::new();
    if let Some(accord) = accord_status(doc).filter(|status| !status.trim().is_empty()) {
        badges.push((format!("[A:{accord}]"), theme.accord_style(accord)));
    }
    if let Some(review) = review_status(doc).filter(|status| !status.trim().is_empty()) {
        badges.push((format!("[R:{review}]"), theme.review_style(review)));
    }
    if let Some((completed, total)) = subtask_progress(doc) {
        let tone = if completed == total {
            StatusTone::Success
        } else {
            StatusTone::Warning
        };
        badges.push((format!("[{completed}/{total}]"), theme.status_style(tone)));
    }

    let badge_width = badges
        .iter()
        .map(|(badge, _)| text_width(badge))
        .sum::<usize>()
        + badges.len().saturating_sub(1);
    let badge_prefix_width = if badges.is_empty() { 0 } else { 1 };
    let fixed_width = text_width(&priority_badge)
        + 1
        + text_width(&type_badge)
        + 1
        + badge_prefix_width
        + badge_width
        + 1
        + text_width(doc.id());
    let title_width = content_width.saturating_sub(fixed_width).max(12);
    let title = truncate(doc.title(), title_width);
    let used_before_id = text_width(&priority_badge)
        + 1
        + text_width(&type_badge)
        + 1
        + text_width(&title)
        + badge_prefix_width
        + badge_width;
    let spacer_width = content_width
        .saturating_sub(used_before_id + text_width(doc.id()))
        .max(1);

    let mut title_spans = vec![
        Span::styled(priority_badge, theme.priority_style(priority)),
        Span::raw(" "),
        Span::styled(type_badge, theme.muted_style()),
        Span::raw(" "),
        Span::styled(title, theme.text_style().add_modifier(Modifier::BOLD)),
    ];
    if !badges.is_empty() {
        title_spans.push(Span::raw(" "));
        for (index, (badge, style)) in badges.into_iter().enumerate() {
            if index > 0 {
                title_spans.push(Span::raw(" "));
            }
            title_spans.push(Span::styled(badge, style));
        }
    }
    title_spans.push(Span::raw(" ".repeat(spacer_width)));
    title_spans.push(Span::styled(
        doc.id().to_string(),
        theme.status_style(StatusTone::Accent),
    ));

    ListItem::new(vec![
        Line::from(title_spans),
        Line::from(board_metadata_spans(doc, theme)),
    ])
}

fn priority_badge(priority: &str) -> String {
    let label = match priority.trim().to_ascii_lowercase().as_str() {
        "critical" | "urgent" => "CRIT".to_string(),
        "high" => "HIGH".to_string(),
        "medium" | "med" => "MED".to_string(),
        "low" => "LOW".to_string(),
        "" | "-" | "none" => "NONE".to_string(),
        other => other.chars().take(4).collect::<String>().to_uppercase(),
    };
    format!("[{label:<4}]")
}

fn board_metadata_spans(doc: &Document, theme: &TuiTheme) -> Vec<Span<'static>> {
    let mut segments: Vec<(String, Style)> = Vec::new();

    if let Some(tags) = doc.field("tags").map(parse_field_values) {
        let tags = tags
            .into_iter()
            .map(|tag| format!("#{tag}"))
            .collect::<Vec<_>>()
            .join(" ");
        if !tags.is_empty() {
            segments.push((tags, theme.status_style(StatusTone::Accent)));
        }
    }

    if let Some(assignee) = doc
        .field("assignee")
        .or_else(|| doc.field("accord.assignee"))
        .filter(|assignee| !assignee.trim().is_empty() && *assignee != "-")
    {
        segments.push((format!("@{assignee}"), theme.muted_style()));
    }

    if let Some(due) = doc
        .field("dueDate")
        .filter(|due| !due.trim().is_empty() && *due != "-")
    {
        segments.push((format!("due {}", truncate(due, 16)), theme.muted_style()));
    }

    if let Some(updated) = doc
        .field("updatedAt")
        .filter(|updated| !updated.trim().is_empty())
    {
        segments.push((
            format!("updated {}", compact_timestamp(updated)),
            theme.muted_style(),
        ));
    }

    let related_files = doc
        .field("relatedFiles")
        .map(parse_field_values)
        .unwrap_or_default();
    if !related_files.is_empty() {
        segments.push((
            format!("files {}", related_files.len()),
            theme.muted_style(),
        ));
    }

    let blockers = doc
        .field("blockers")
        .map(parse_field_values)
        .unwrap_or_default();
    if !blockers.is_empty() {
        segments.push((
            format!("blocked by {}", blockers.len()),
            theme.status_style(StatusTone::Warning),
        ));
    }

    segments.push((truncate(&display_path(&doc.path), 48), theme.muted_style()));

    join_metadata_segments(segments, theme)
}

fn join_metadata_segments(segments: Vec<(String, Style)>, theme: &TuiTheme) -> Vec<Span<'static>> {
    if segments.is_empty() {
        return vec![Span::styled("no metadata".to_string(), theme.muted_style())];
    }

    let mut spans = Vec::new();
    for (index, (text, style)) in segments.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", theme.muted_style()));
        }
        spans.push(Span::styled(text, style));
    }
    spans
}

fn subtask_progress(doc: &Document) -> Option<(usize, usize)> {
    let mut completed = 0;
    let mut total = 0;
    for (key, value) in &doc.fields {
        if key.starts_with("subtasks.") && key.ends_with(".completed") {
            total += 1;
            if matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "true" | "yes" | "done" | "1"
            ) {
                completed += 1;
            }
        }
    }
    (total > 0).then_some((completed, total))
}

fn compact_timestamp(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 16 && value.as_bytes().get(10) == Some(&b'T') {
        format!("{} {}", &value[..10], &value[11..16])
    } else {
        truncate(value, 16)
    }
}

fn text_width(value: &str) -> usize {
    value.chars().count()
}

fn detail_lines_for_doc(doc: &Document, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme.label_style()),
        Span::styled(doc.title().to_string(), theme.text_style()),
    ]));
    lines.push(detail_field_line("ID", doc.id(), theme));
    lines.push(detail_field_line("Type", doc.doc_type(), theme));
    push_optional_detail_line(&mut lines, "State", doc.field("state"), theme);
    push_optional_detail_line(&mut lines, "Priority", doc.field("priority"), theme);
    push_optional_detail_line(&mut lines, "Assignee", doc.field("assignee"), theme);
    push_optional_detail_line(&mut lines, "Due", doc.field("dueDate"), theme);
    push_optional_detail_line(&mut lines, "Tags", doc.field("tags"), theme);
    push_optional_detail_line(&mut lines, "Accord", accord_status(doc), theme);
    push_optional_detail_line(&mut lines, "Review", review_status(doc), theme);
    push_optional_detail_line(&mut lines, "Updated", doc.field("updatedAt"), theme);
    lines.push(detail_field_line("Path", &display_path(&doc.path), theme));
    push_board_accord_detail_section(&mut lines, doc, theme);
    lines.push(Line::from(""));
    lines.push(detail_section_heading("Body", theme));
    if doc.body.trim().is_empty() {
        lines.push(Line::from(Span::styled("(empty)", theme.muted_style())));
    } else {
        for line in doc.body.lines() {
            lines.push(markdownish_line(line, theme));
        }
    }
    lines
}

fn push_board_accord_detail_section(
    lines: &mut Vec<Line<'static>>,
    doc: &Document,
    theme: &TuiTheme,
) {
    if doc.doc_type() != "task" {
        return;
    }

    let status = accord_status(doc).unwrap_or("missing").trim();
    let status = if status.is_empty() { "missing" } else { status };

    lines.push(Line::from(""));
    lines.push(detail_section_heading("Accord", theme));
    lines.push(detail_status_line(
        "Status",
        status,
        accord_detail_status_style(status, theme),
        theme,
    ));
    lines.push(detail_field_line(
        "Signal",
        accord_state_signal(status),
        theme,
    ));
    push_optional_detail_line(
        lines,
        "Accord assignee",
        doc.field("accord.assignee"),
        theme,
    );
    push_optional_detail_line(lines, "Claimed", doc.field("accord.claimedAt"), theme);
    push_optional_detail_line(lines, "Delivered", doc.field("accord.deliveredAt"), theme);
    push_optional_detail_list_line(
        lines,
        "Deliverables",
        first_accord_list(doc, &["accord.deliverables"]),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Validation",
        first_accord_list(
            doc,
            &[
                "accord.validation.commands",
                "accord.validation",
                "accord.validations",
            ],
        ),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Constraints",
        first_accord_list(doc, &["accord.constraints"]),
        theme,
    );
    push_optional_detail_line(lines, "Summary", doc.field("accord.summary"), theme);
    push_optional_detail_list_line(
        lines,
        "Evidence",
        first_accord_list(doc, &["accord.evidence"]),
        theme,
    );
    push_optional_detail_list_line(
        lines,
        "Files changed",
        first_accord_list(doc, &["accord.filesChanged"]),
        theme,
    );
    push_optional_detail_line(lines, "Reviewer", doc.field("accord.reviewer"), theme);
    push_optional_detail_line(lines, "Note", doc.field("accord.note"), theme);
    push_optional_detail_line(lines, "Reason", doc.field("accord.reason"), theme);
    push_optional_detail_line(
        lines,
        "Accord updated",
        doc.field("accord.updatedAt"),
        theme,
    );
    lines.push(detail_field_line("Next", accord_next_action(status), theme));
    lines.push(Line::from(vec![
        Span::styled("CLI hint: ", theme.label_style()),
        Span::styled(accord_cli_hint(doc.id(), status), theme.text_style()),
    ]));
    lines.push(Line::from(Span::styled(
        "TUI accord mutations are planned; this Board detail pane is read-only.",
        theme.muted_style(),
    )));
}

fn detail_field_line(label: &str, value: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), theme.text_style()),
    ])
}

fn detail_status_line(label: &str, value: &str, style: Style, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(value.to_string(), style),
    ])
}

fn detail_section_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        theme
            .markdown_heading_style()
            .add_modifier(Modifier::UNDERLINED),
    ))
}

fn push_optional_detail_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: Option<&str>,
    theme: &TuiTheme,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(detail_field_line(label, value, theme));
    }
}

fn push_optional_detail_list_line(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    values: Vec<String>,
    theme: &TuiTheme,
) {
    if !values.is_empty() {
        lines.push(detail_field_line(label, &values.join(", "), theme));
    }
}

fn first_accord_list(doc: &Document, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .filter_map(|key| doc.field(key).map(parse_field_values))
        .find(|values| !values.is_empty())
        .unwrap_or_default()
}

fn accord_detail_status_style(status: &str, theme: &TuiTheme) -> Style {
    if normalized_accord_status(status) == "missing" {
        theme.muted_style()
    } else {
        theme.accord_style(status).add_modifier(Modifier::BOLD)
    }
}

fn accord_state_signal(status: &str) -> &'static str {
    match normalized_accord_status(status).as_str() {
        "ready" => "Ready: scope is recorded and the work can be claimed.",
        "claimed" => "Claimed: an owner is actively working the accord.",
        "delivered" => "Delivered: inspect summary/evidence, then accept or request rework.",
        "accepted" => "Accepted: accord review passed; completion/logging is still separate.",
        "rework" => "Rework: changes were requested before the accord can be accepted.",
        "blocked" => "Blocked: work cannot proceed until the recorded reason is resolved.",
        "failed" => "Failed: the accord attempt ended unsuccessfully and needs review.",
        "missing" | "" => "Missing: no accord metadata is recorded for this task yet.",
        _ => "Unknown: inspect the raw task before changing accord state.",
    }
}

fn accord_next_action(status: &str) -> &'static str {
    match normalized_accord_status(status).as_str() {
        "ready" => "Claim the accord when an owner is known.",
        "claimed" => "Deliver when complete, or block/fail with a reason if work cannot proceed.",
        "delivered" => "Inspect the delivery, then accept it or request rework.",
        "accepted" => "Complete/archive the task when it is ready to leave the Board.",
        "rework" => "Apply requested changes, then deliver again with a fresh summary.",
        "blocked" => "Resolve the blocker, then ready/claim/deliver; fail only if unrecoverable.",
        "failed" => "Review the failure and reset to ready if retrying the work.",
        "missing" | "" => {
            "Create a ready accord once scope, deliverables, and validation are known."
        }
        _ => "Inspect current metadata before choosing the next accord action.",
    }
}

fn accord_cli_hint(id: &str, status: &str) -> String {
    match normalized_accord_status(status).as_str() {
        "ready" => format!("tdm accord claim {id} --assignee <name>"),
        "claimed" => format!(
            "tdm accord deliver {id} --summary <text> [--evidence <text>] [--file-changed <path>]"
        ),
        "delivered" => format!(
            "tdm accord accept {id} [--reviewer <name>] [--note <text>] OR tdm accord rework {id} --note <text>"
        ),
        "accepted" => format!(
            "tdm complete {id} --summary <text> [--validation <text>] [--reviewer <name>]"
        ),
        "rework" => format!("tdm accord deliver {id} --summary <text> [--evidence <text>]"),
        "blocked" => format!(
            "tdm accord ready {id} [--assignee <name>] OR tdm accord fail {id} --reason <text>"
        ),
        "failed" => format!("tdm accord ready {id} [--assignee <name>]"),
        "missing" | "" => format!(
            "tdm accord ready {id} [--assignee <name>] [--deliverable <spec>] [--validation <command>]"
        ),
        _ => format!("tdm show {id}  # inspect accord metadata before mutating"),
    }
}

fn normalized_accord_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace('_', "-")
}

fn markdownish_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        Line::from(Span::styled(
            line.to_string(),
            theme.markdown_heading_style(),
        ))
    } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        Line::from(Span::styled(line.to_string(), theme.markdown_list_style()))
    } else if trimmed.starts_with("```") {
        Line::from(Span::styled(line.to_string(), theme.markdown_code_style()))
    } else {
        Line::from(Span::styled(line.to_string(), theme.text_style()))
    }
}

fn status_tone_for_message(message: &str) -> StatusTone {
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

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn decision_doc(id: &str) -> Document {
        let mut doc = doc_with_state(id, None);
        doc.fields
            .insert("type".to_string(), "decision".to_string());
        doc.fields
            .insert("title".to_string(), format!("Decision {id}"));
        doc.body = "## Decision\nKeep local navigation local.".to_string();
        doc
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn keyboard_test_app() -> TuiApp {
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("review")),
            decision_doc("decision-1"),
        ];
        TuiApp {
            workspace: Workspace {
                board_dir: PathBuf::from(".tandem/board"),
                logs_dir: PathBuf::from(".tandem/logs"),
                config_path: PathBuf::from(".tandem/tandem.md"),
                events_path: PathBuf::from(".tandem/events.jsonl"),
            },
            title: "Test".to_string(),
            view: TuiView::Board,
            states: vec![
                "todo".to_string(),
                "review".to_string(),
                "unfiled".to_string(),
            ],
            configured_states: vec!["todo".to_string(), "review".to_string()],
            docs,
            logs: Vec::new(),
            log_events: logs::LogEventsById::new(),
            rules: empty_rules(),
            load_errors: Vec::new(),
            theme: TuiTheme::default_dark(),
            theme_source: "test".to_string(),
            theme_warnings: Vec::new(),
            selected_state: 0,
            selected_item: 0,
            selected_review_item: 0,
            selected_log: 0,
            focus: FocusPane::Board,
            detail_scroll: 0,
            review_detail_scroll: 0,
            log_detail_scroll: 0,
            log_search_filter: String::new(),
            log_search_input: None,
            status: String::new(),
            show_help: false,
            quick_add: None,
            rules_view: RulesState::default(),
            decisions_view: DecisionsState::default(),
            hits: Vec::new(),
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
    fn numeric_keys_map_to_top_level_views() {
        assert_eq!(TuiView::from_digit('1'), Some(TuiView::Board));
        assert_eq!(TuiView::from_digit('2'), Some(TuiView::Review));
        assert_eq!(TuiView::from_digit('3'), Some(TuiView::Logs));
        assert_eq!(TuiView::from_digit('4'), Some(TuiView::Rules));
        assert_eq!(TuiView::from_digit('5'), Some(TuiView::Decisions));
        assert_eq!(TuiView::from_digit('6'), None);
    }

    #[test]
    fn numeric_keys_are_explicit_top_level_switchers() {
        let mut app = keyboard_test_app();
        app.handle_key(key(KeyCode::Char('3'))).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Board);

        app.handle_key(key(KeyCode::Char('1'))).unwrap();
        assert_eq!(app.view, TuiView::Board);
        assert_eq!(app.focus, FocusPane::Board);
    }

    #[test]
    fn tab_cycles_focus_without_switching_top_level_views() {
        let mut app = keyboard_test_app();
        app.switch_view(TuiView::Logs);

        app.handle_key(key(KeyCode::Tab)).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Detail);

        app.handle_key(key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Board);
    }

    #[test]
    fn tab_has_no_top_level_fallback_without_focusable_panes() {
        let mut app = keyboard_test_app();
        app.switch_view(TuiView::Rules);

        app.handle_key(key(KeyCode::Tab)).unwrap();
        assert_eq!(app.view, TuiView::Rules);
        assert_eq!(app.focus, FocusPane::Board);
        assert!(app.status.contains("Tab stays in Rules"));

        app.handle_key(key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.view, TuiView::Rules);
        assert_eq!(app.focus, FocusPane::Board);
        assert!(app.status.contains("Tab stays in Rules"));
    }

    #[test]
    fn hjkl_local_navigation_does_not_switch_top_level_views() {
        let mut app = keyboard_test_app();

        app.switch_view(TuiView::Board);
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view, TuiView::Board);
        assert_eq!(app.selected_state, 1);
        app.handle_key(key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view, TuiView::Board);
        assert_eq!(app.selected_state, 0);

        app.switch_view(TuiView::Review);
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view, TuiView::Review);
        assert_eq!(app.focus, FocusPane::Detail);
        app.handle_key(key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view, TuiView::Review);
        assert_eq!(app.focus, FocusPane::Board);

        app.switch_view(TuiView::Logs);
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Detail);
        app.handle_key(key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Board);

        app.switch_view(TuiView::Rules);
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view, TuiView::Rules);
        app.handle_key(key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view, TuiView::Rules);

        app.switch_view(TuiView::Decisions);
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view, TuiView::Decisions);
        assert_eq!(app.focus, FocusPane::Detail);
        app.handle_key(key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view, TuiView::Decisions);
        assert_eq!(app.focus, FocusPane::Board);
    }

    #[test]
    fn review_attention_reason_covers_delivered_and_pending_items() {
        let mut delivered = doc_with_state("task-1", Some("review"));
        delivered
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());
        assert_eq!(
            review_attention_reason(&delivered).as_deref(),
            Some("accord delivered")
        );

        let mut pending = doc_with_state("task-2", Some("review"));
        pending
            .fields
            .insert("review.status".to_string(), "pending".to_string());
        assert_eq!(
            review_attention_reason(&pending).as_deref(),
            Some("review pending")
        );
    }

    #[test]
    fn board_detail_includes_accord_metadata_hints_and_preserves_body() {
        let mut doc = doc_with_state("task-1", Some("review"));
        doc.fields
            .insert("accord.status".to_string(), "delivered".to_string());
        doc.fields
            .insert("accord.assignee".to_string(), "pi".to_string());
        doc.fields.insert(
            "accord.deliveredAt".to_string(),
            "2026-06-28T01:00:00Z".to_string(),
        );
        doc.fields.insert(
            "accord.deliverables".to_string(),
            "[\"code:src/lib.rs\", \"docs:README.md\"]".to_string(),
        );
        doc.fields.insert(
            "accord.validation.commands".to_string(),
            "[\"cargo test\", \"cargo build\"]".to_string(),
        );
        doc.fields.insert(
            "accord.constraints".to_string(),
            "[\"do not mutate task state\"]".to_string(),
        );
        doc.fields.insert(
            "accord.summary".to_string(),
            "Rendered accord metadata".to_string(),
        );
        doc.fields.insert(
            "accord.evidence".to_string(),
            "[\"tests passed\"]".to_string(),
        );
        doc.fields.insert(
            "accord.filesChanged".to_string(),
            "[\"src/lib.rs\"]".to_string(),
        );
        doc.body = "## Description\nKeep this body visible.".to_string();

        let theme = TuiTheme::default_dark();
        let lines = detail_lines_for_doc(&doc, &theme);
        let texts = lines.iter().map(line_text).collect::<Vec<_>>();

        let accord_index = texts.iter().position(|text| text == "Accord").unwrap();
        let body_index = texts.iter().position(|text| text == "Body").unwrap();
        assert!(accord_index < body_index);
        assert!(texts.contains(&"Status: delivered".to_string()));
        assert!(texts.iter().any(|text| text.contains(
            "Signal: Delivered: inspect summary/evidence, then accept or request rework."
        )));
        assert!(texts.contains(&"Accord assignee: pi".to_string()));
        assert!(texts.contains(&"Deliverables: code:src/lib.rs, docs:README.md".to_string()));
        assert!(texts.contains(&"Validation: cargo test, cargo build".to_string()));
        assert!(texts.contains(&"Constraints: do not mutate task state".to_string()));
        assert!(texts.contains(&"Summary: Rendered accord metadata".to_string()));
        assert!(texts.contains(&"Evidence: tests passed".to_string()));
        assert!(texts.contains(&"Files changed: src/lib.rs".to_string()));
        assert!(texts
            .iter()
            .any(|text| text
                .contains("Next: Inspect the delivery, then accept it or request rework.")));
        assert!(texts
            .iter()
            .any(|text| text.contains("CLI hint: tdm accord accept task-1")));
        assert!(texts.iter().any(|text| text
            .contains("TUI accord mutations are planned; this Board detail pane is read-only.")));
        assert!(texts.contains(&"## Description".to_string()));
        assert!(texts.contains(&"Keep this body visible.".to_string()));

        let status_line = lines
            .iter()
            .find(|line| line_text(line) == "Status: delivered")
            .unwrap();
        assert_eq!(
            status_line.spans[1].style,
            theme.accord_style("delivered").add_modifier(Modifier::BOLD)
        );
    }

    #[test]
    fn accord_detail_styles_key_review_states_distinctly() {
        let theme = TuiTheme::default_dark();
        let delivered = accord_detail_status_style("delivered", &theme);
        let accepted = accord_detail_status_style("accepted", &theme);
        let rework = accord_detail_status_style("rework", &theme);
        let blocked = accord_detail_status_style("blocked", &theme);

        assert_ne!(delivered, accepted);
        assert_ne!(delivered, rework);
        assert_ne!(delivered, blocked);
        assert_ne!(accepted, rework);
        assert_ne!(accepted, blocked);
        assert_ne!(rework, blocked);
        assert!(accord_state_signal("delivered").starts_with("Delivered:"));
        assert!(accord_state_signal("accepted").contains("completion/logging is still separate"));
        assert!(accord_state_signal("rework").starts_with("Rework:"));
        assert!(accord_state_signal("blocked").starts_with("Blocked:"));
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

    #[test]
    fn board_subview_tabs_count_visible_states() {
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("todo")),
            doc_with_state("task-3", Some("review")),
        ];
        let states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
        ];
        let tabs = board_subview_tabs(&states, &docs);
        assert_eq!(
            tabs,
            vec![
                BoardSubviewTab {
                    state: "todo".to_string(),
                    count: 2,
                },
                BoardSubviewTab {
                    state: "in-progress".to_string(),
                    count: 0,
                },
                BoardSubviewTab {
                    state: "review".to_string(),
                    count: 1,
                },
            ]
        );
        assert_eq!(state_tab_title("in-progress", 3), " IN PROGRESS 3 ");
    }

    #[test]
    fn subtask_progress_counts_completed_checklist_items() {
        let mut doc = doc_with_state("task-1", Some("todo"));
        doc.fields
            .insert("subtasks.0.completed".to_string(), "true".to_string());
        doc.fields
            .insert("subtasks.1.completed".to_string(), "false".to_string());
        doc.fields
            .insert("subtasks.2.completed".to_string(), "1".to_string());
        assert_eq!(subtask_progress(&doc), Some((2, 3)));
    }
}
