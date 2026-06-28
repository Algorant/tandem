use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use yaml_rust2::Yaml;

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
    app.run(&mut session)
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

    fn suspend_for_editor(&mut self) -> Result<(), CliError> {
        self.terminal.show_cursor()?;
        self.terminal.backend_mut().flush()?;
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }

    fn resume_after_editor(&mut self) -> Result<(), CliError> {
        enable_raw_mode()?;
        if let Err(error) = execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        ) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        self.terminal.clear()?;
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    Continue,
    Quit,
    OpenEditor,
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

#[derive(Debug, Clone, Default)]
struct ReloadOutcome {
    warning_count: usize,
    first_warning: Option<String>,
}

impl ReloadOutcome {
    fn from_warnings(warnings: &[String]) -> Self {
        Self {
            warning_count: warnings.len(),
            first_warning: warnings.first().cloned(),
        }
    }

    fn warning_note(&self) -> String {
        match self.warning_count {
            0 => String::new(),
            1 => format!(
                "; reload warning: {}",
                truncate(
                    self.first_warning.as_deref().unwrap_or("inspect status"),
                    120
                )
            ),
            count => format!(
                "; {count} reload warnings; first: {}",
                truncate(
                    self.first_warning.as_deref().unwrap_or("inspect status"),
                    120
                )
            ),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ReloadFingerprint {
    files: BTreeMap<PathBuf, Option<FileSignature>>,
}

impl ReloadFingerprint {
    fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct ReloadSelection {
    board_doc_id: Option<String>,
    board_state: Option<String>,
    review_doc_id: Option<String>,
    log_doc_id: Option<String>,
    rule_anchor: Option<(String, Option<usize>)>,
    decision_doc_id: Option<String>,
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
    reload_fingerprint: ReloadFingerprint,
    last_reload_check: Instant,
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
            reload_fingerprint: ReloadFingerprint::default(),
            last_reload_check: Instant::now(),
        };
        app.reload();
        Ok(app)
    }

    fn reload(&mut self) -> ReloadOutcome {
        let selection = self.capture_reload_selection();
        let mut load_errors = Vec::new();
        let mut docs = read_documents_tolerant(
            &self.workspace.board_dir,
            DocumentLocation::Board,
            "Board",
            &mut load_errors,
        );
        sort_documents(&mut docs);

        let (title, configured_states, rules) =
            match read_frontmatter_yaml_file(&self.workspace.config_path) {
                Ok(root) => (
                    workspace_title_from_root(root.as_ref())
                        .unwrap_or_else(|| "Tandem".to_string()),
                    workspace_states_from_root(root.as_ref()),
                    parse_rules_from_yaml(root.as_ref()),
                ),
                Err(error) => {
                    load_errors.push(format!("Config load failed: {}", error.message));
                    (
                        if self.title.is_empty() {
                            "Tandem".to_string()
                        } else {
                            self.title.clone()
                        },
                        if self.configured_states.is_empty() {
                            default_workspace_states()
                        } else {
                            self.configured_states.clone()
                        },
                        self.rules.clone(),
                    )
                }
            };

        let theme_load = TuiTheme::load_for_workspace(&self.workspace);
        let log_load = logs::load_logs(&self.workspace.logs_dir);
        load_errors.extend(log_load.warnings);
        let (log_events, event_warnings) = logs::load_log_events(&self.workspace.events_path);
        load_errors.extend(event_warnings);
        load_errors.extend(validation_load_errors(
            &docs,
            &log_load.docs,
            &configured_states,
        ));

        self.title = title;
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
        self.restore_reload_selection(selection);
        self.clamp_selection();
        self.clamp_rules_state();
        self.clamp_decisions_state();
        let warnings = self.runtime_warnings();
        let outcome = ReloadOutcome::from_warnings(&warnings);
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
        let load_note = runtime_warning_status_note(&outcome);
        self.status = format!(
            "Reloaded {} active document{} from {} · {}{}",
            self.docs.len(),
            if self.docs.len() == 1 { "" } else { "s" },
            display_path(&self.workspace.board_dir),
            theme_note,
            load_note
        );
        self.reload_fingerprint = collect_reload_fingerprint(&self.workspace);
        self.last_reload_check = Instant::now();
        outcome
    }

    fn capture_reload_selection(&self) -> ReloadSelection {
        ReloadSelection {
            board_doc_id: self.selected_doc().map(|doc| doc.id().to_string()),
            board_state: self.states.get(self.selected_state).cloned(),
            review_doc_id: self
                .selected_review_item()
                .map(|item| item.id().to_string()),
            log_doc_id: self.selected_log().map(|doc| doc.id().to_string()),
            rule_anchor: self.selected_rule_anchor_for_reload(),
            decision_doc_id: self.selected_decision_id_for_reload(),
        }
    }

    fn restore_reload_selection(&mut self, selection: ReloadSelection) {
        let restored_board_doc = selection
            .board_doc_id
            .as_deref()
            .map(|id| self.select_document_by_id_preserving_scroll(id))
            .unwrap_or(false);
        if !restored_board_doc {
            if let Some(state) = selection.board_state.as_deref() {
                if let Some(index) = self.states.iter().position(|candidate| candidate == state) {
                    self.selected_state = index;
                }
            }
        }

        if let Some(id) = selection.review_doc_id.as_deref() {
            self.select_review_item_by_id_preserving_scroll(id);
        }
        if let Some(id) = selection.log_doc_id.as_deref() {
            self.select_log_by_id_preserving_scroll(id);
        }
        self.restore_rule_selection_after_reload(selection.rule_anchor);
        self.restore_decision_selection_after_reload(selection.decision_doc_id);
    }

    fn runtime_warnings(&self) -> Vec<String> {
        self.load_errors
            .iter()
            .chain(self.theme_warnings.iter())
            .cloned()
            .collect()
    }

    fn input_overlay_active(&self) -> bool {
        self.quick_add.is_some()
            || self.log_search_input.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
            || self.show_help
    }

    fn reload_if_changed(&mut self) {
        if self.input_overlay_active()
            || self.last_reload_check.elapsed() < Duration::from_millis(250)
        {
            return;
        }
        self.last_reload_check = Instant::now();
        let current = collect_reload_fingerprint(&self.workspace);
        if self.reload_fingerprint.is_empty() {
            self.reload_fingerprint = current;
            return;
        }
        if current != self.reload_fingerprint {
            self.reload();
            self.status = format!("External changes detected; {}", self.status);
        }
    }

    fn run(&mut self, session: &mut TerminalSession) -> Result<(), CliError> {
        loop {
            self.reload_if_changed();
            session.terminal_mut().draw(|frame| self.draw(frame))?;
            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => match self.handle_key(key)? {
                        KeyAction::Continue => {}
                        KeyAction::Quit => break,
                        KeyAction::OpenEditor => self.open_selected_item_in_editor(session)?,
                    },
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<KeyAction, CliError> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(KeyAction::Quit);
        }

        if self.quick_add.is_some() {
            self.handle_quick_add_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.log_search_input.is_some() {
            self.handle_log_search_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.rules_prompt_active() {
            self.handle_rules_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.decision_prompt_active() {
            self.handle_decision_prompt_key(key);
            return Ok(KeyAction::Continue);
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.show_help = false,
                _ => {}
            }
            return Ok(KeyAction::Continue);
        }

        if let KeyCode::Char(ch) = key.code {
            if let Some(view) = TuiView::from_digit(ch) {
                self.switch_view(view);
                return Ok(KeyAction::Continue);
            }
        }

        match key.code {
            KeyCode::Char('q') => return Ok(KeyAction::Quit),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('r') => {
                self.reload();
            }
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
            KeyCode::Char('e') if matches!(self.view, TuiView::Board | TuiView::Review) => {
                return Ok(KeyAction::OpenEditor)
            }
            KeyCode::Char('e') if self.view == TuiView::Logs => {
                self.status = "Completed logs are read-only in the TUI; $EDITOR is intentionally disabled for generated history.".to_string()
            }
            KeyCode::Char('e') if self.view == TuiView::Decisions => {
                self.status = "Decision document editing in $EDITOR is deferred; use Decisions add or edit the file manually for now.".to_string()
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
        Ok(KeyAction::Continue)
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
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = format!(
                    "Created {} in {}: {}{}",
                    outcome.id, outcome.state, outcome.title, reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
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
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = if outcome.changed {
                    format!(
                        "Moved {}: {} -> {}{}",
                        outcome.id, outcome.from, outcome.to, reload_note
                    )
                } else {
                    format!(
                        "{} is already in state {}{}",
                        outcome.id, outcome.to, reload_note
                    )
                };
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(doc_id);
                self.status = format!("Move error: {}{}", error.message, reload_note);
            }
        }
    }

    fn open_selected_item_in_editor(
        &mut self,
        session: &mut TerminalSession,
    ) -> Result<(), CliError> {
        let target = match self.selected_editor_target() {
            Ok(target) => target,
            Err(message) => {
                self.status = message;
                return Ok(());
            }
        };
        let editor = match editor_command_from_env() {
            Ok(editor) => editor,
            Err(error) => {
                self.status = format!("Editor error: {}", error.message);
                return Ok(());
            }
        };

        self.status = format!(
            "Opening {} in {} from {}...",
            target.id,
            editor.display_label(),
            editor.source
        );
        session.terminal_mut().draw(|frame| self.draw(frame))?;

        session.suspend_for_editor()?;
        let editor_result = run_editor_command(&editor, &target.path);
        let resume_result = session.resume_after_editor();
        if let Err(error) = resume_result {
            return Err(CliError::user(format!(
                "failed to restore terminal after editor exit: {}",
                error.message
            )));
        }

        let reload_note = self.reload().warning_note();
        let selection_note = if self.select_document_by_id(&target.id) {
            String::new()
        } else {
            "; edited item is not currently loadable or no longer active".to_string()
        };
        let reload_note = format!("{reload_note}{selection_note}");

        self.status = match editor_result {
            Ok(status) if status.success() => format!(
                "Edited {} via {}{}",
                target.id,
                editor.display_label(),
                reload_note
            ),
            Ok(status) => format!(
                "Editor exited with {status} for {}{}",
                target.id, reload_note
            ),
            Err(error) => format!(
                "Editor launch failed for {} using {}: {error}{}",
                target.id,
                editor.display_label(),
                reload_note
            ),
        };
        Ok(())
    }

    fn selected_editor_target(&self) -> Result<EditorTarget, String> {
        match self.view {
            TuiView::Board => self
                .selected_doc()
                .ok_or_else(|| "No active task selected to edit.".to_string())
                .and_then(editor_target_for_doc),
            TuiView::Review => {
                let item = self
                    .selected_review_item()
                    .ok_or_else(|| "No review item selected to edit.".to_string())?;
                self.docs
                    .iter()
                    .find(|doc| doc.id() == item.id())
                    .ok_or_else(|| format!("Review item {} is no longer loaded; press r to reload.", item.id()))
                    .and_then(editor_target_for_doc)
            }
            TuiView::Logs => Err("Completed logs are read-only in the TUI; $EDITOR is intentionally disabled for generated history.".to_string()),
            TuiView::Rules => Err("Rules use the in-TUI a/e/d prompts; raw config-file editing is deferred.".to_string()),
            TuiView::Decisions => Err("Decision document editing in $EDITOR is deferred; active task documents are supported first.".to_string()),
        }
    }

    fn select_document_by_id(&mut self, id: &str) -> bool {
        self.select_document_by_id_with_scroll(id, true)
    }

    fn select_document_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        self.select_document_by_id_with_scroll(id, false)
    }

    fn select_document_by_id_with_scroll(&mut self, id: &str, reset_scroll: bool) -> bool {
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
                        if reset_scroll {
                            self.detail_scroll = 0;
                        }
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
        if self.input_overlay_active() {
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

    fn select_log_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        let logs = self.filtered_logs();
        if let Some(index) = logs.iter().position(|doc| doc.id() == id) {
            self.selected_log = index;
            self.clamp_selection();
            true
        } else {
            self.clamp_selection();
            false
        }
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

    fn select_review_item_by_id_preserving_scroll(&mut self, id: &str) -> bool {
        let items = self.review_items();
        if let Some(index) = items.iter().position(|item| item.id() == id) {
            self.selected_review_item = index;
            self.clamp_review_selection();
            true
        } else {
            self.clamp_review_selection();
            false
        }
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
                "Tandem CLI/TUI needs a larger terminal",
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
            let show_doc_type = docs.iter().any(|doc| doc.doc_type() != "task");
            docs.iter()
                .map(|doc| list_item_for_doc(doc, &self.theme, content_width, show_doc_type))
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
            .highlight_style(self.theme.board_selected_style())
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
            let y = top.saturating_add(index as u16);
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: 1,
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
                    "{focus} · {} · 1..5 views · q quit · r reload · a add task · e edit task in $EDITOR · h/l state subview · H/L move task · j/k item/scroll · Tab/Enter detail · ? help · {}",
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
                    "Review {focus} · 1..5 views · q quit · r reload · e edit active task in $EDITOR · j/k item/scroll · h/l or Tab queue/detail · Enter detail · action hints read-only · ? help · {}",
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
                    "Logs {focus} · {filter}/ search · j/k select/scroll · g/G top/bottom · h/l or Tab list/detail · Enter focus detail/list · e read-only/no editor · r reload · q quit · {}",
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
                "Tandem CLI/TUI view shell",
                self.theme.title_style(),
            )),
            Line::from(""),
            Line::from("q / Ctrl-C        Quit safely"),
            Line::from("r                 Reload board/config/log/theme data (also auto-detected while idle)"),
            Line::from("1..5              Switch Board, Review, Logs, Rules, Decisions"),
            Line::from("click top tabs    Switch views with the mouse"),
            Line::from("tab / shift-tab   Cycle list/detail focus where available; never switches views"),
            Line::from("enter             Toggle list/detail focus in Board, Review, and Logs"),
            Line::from("a                 Board quick-add; Rules add rule; Decisions add decision"),
            Line::from("e                 Board/Review: open active task in $EDITOR; Rules: edit rule; Logs read-only; Decisions deferred"),
            Line::from("h/l or ←/→        Board: state subviews; Review/Logs/Decisions: list/detail focus; Rules: category"),
            Line::from("H/L               Board: move selected task to previous/next configured state"),
            Line::from("j/k or ↑/↓        Board/Review/Logs/Rules/Decisions: move items, or scroll detail when focused"),
            Line::from("g/G               First/last item in the active list/detail"),
            Line::from("d                 Rules: delete selected rule with confirmation"),
            Line::from("PgUp/PgDn         Logs/Decisions: scroll selected detail/body"),
            Line::from("/                 Logs: search by id, title, summary, body, validation, files"),
            Line::from("Esc               Logs: clear search filter; prompts: cancel"),
            Line::from("mouse wheel       Board/Review/Logs/Rules/Decisions: move selection or scroll detail"),
            Line::from("click tabs/list/detail Board/Review/Logs: switch subviews, select, or focus panes"),
            Line::from("Prompts           Type text, Enter advances or saves, Esc cancels, Ctrl-U clears field"),
            Line::from("Log search        Type a query, Enter applies, Esc cancels"),
            Line::from(""),
            Line::from("Board state subviews, quick-add/H/L moves, $EDITOR for active task documents, Review queue, Logs browser/search, Rules add/edit/delete, and Decisions browse/add are active. Logs stay read-only for generated history; decision/custom file editing is deferred. Built-in presets, XDG/~/.config user themes, and .tandem/theme.toml selectors/overrides are active; richer action buttons remain planned."),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditorTarget {
    id: String,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditorCommand {
    program: String,
    args: Vec<String>,
    source: &'static str,
}

impl EditorCommand {
    fn from_value(value: &str, source: &'static str) -> Result<Self, CliError> {
        let words = split_editor_command(value).map_err(|message| {
            CliError::user(format!(
                "could not parse {source} value `{value}`: {message}"
            ))
        })?;
        let Some((program, args)) = words.split_first() else {
            return Err(CliError::user(format!("{source} is empty")));
        };
        Ok(Self {
            program: program.clone(),
            args: args.to_vec(),
            source,
        })
    }

    fn display_label(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

fn editor_target_for_doc(doc: &Document) -> Result<EditorTarget, String> {
    if doc.location != DocumentLocation::Board {
        return Err("Only active board documents are editable in $EDITOR for now.".to_string());
    }
    if doc.doc_type() != "task" {
        return Err(format!(
            "Only active task documents open in $EDITOR for now; {} is type `{}` and is deferred.",
            doc.id(),
            doc.doc_type()
        ));
    }
    Ok(EditorTarget {
        id: doc.id().to_string(),
        path: doc.path.clone(),
    })
}

fn editor_command_from_env() -> Result<EditorCommand, CliError> {
    for (name, source) in [("EDITOR", "$EDITOR"), ("VISUAL", "$VISUAL")] {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                return EditorCommand::from_value(&value, source);
            }
        }
    }

    EditorCommand::from_value(default_editor_program(), "default editor")
}

fn default_editor_program() -> &'static str {
    if cfg!(windows) {
        "notepad"
    } else {
        "vi"
    }
}

fn run_editor_command(command: &EditorCommand, path: &Path) -> io::Result<ExitStatus> {
    Command::new(&command.program)
        .args(&command.args)
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

fn split_editor_command(value: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut word_started = false;

    for ch in value.chars() {
        if escaped {
            current.push(ch);
            word_started = true;
            escaped = false;
            continue;
        }

        if ch == '\\' && quote != Some('\'') {
            escaped = true;
            word_started = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                current.push(ch);
            }
            word_started = true;
            continue;
        }

        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            word_started = true;
        } else if ch.is_whitespace() {
            if word_started {
                words.push(current.clone());
                current.clear();
                word_started = false;
            }
        } else {
            current.push(ch);
            word_started = true;
        }
    }

    if escaped {
        current.push('\\');
    }
    if let Some(active_quote) = quote {
        return Err(format!("unterminated {active_quote} quote"));
    }
    if word_started {
        words.push(current);
    }
    if words.first().map(|word| word.is_empty()).unwrap_or(false) {
        return Err("editor command program is empty".to_string());
    }
    Ok(words)
}

fn workspace_title_from_root(root: Option<&Yaml>) -> Option<String> {
    root.and_then(|root| yaml_mapping_value(root, "title"))
        .and_then(yaml_scalar_to_string)
        .filter(|title| !title.trim().is_empty())
}

fn workspace_states_from_root(root: Option<&Yaml>) -> Vec<String> {
    let mut states = Vec::new();
    if let Some(states_yaml) = root.and_then(|root| yaml_mapping_value(root, "states")) {
        match states_yaml {
            Yaml::Array(items) => {
                for item in items {
                    if let Some(state) = yaml_scalar_to_string(item)
                        .or_else(|| yaml_mapping_value(item, "id").and_then(yaml_scalar_to_string))
                    {
                        if !state.trim().is_empty() {
                            states.push(state);
                        }
                    }
                }
            }
            _ => {
                if let Some(state) = yaml_scalar_to_string(states_yaml) {
                    if !state.trim().is_empty() {
                        states.push(state);
                    }
                }
            }
        }
    }
    if states.is_empty() {
        states = default_workspace_states();
    }
    states
}

fn default_workspace_states() -> Vec<String> {
    DEFAULT_STATES
        .iter()
        .map(|state| (*state).to_string())
        .collect()
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

fn read_documents_tolerant(
    dir: &Path,
    location: DocumentLocation,
    label: &str,
    load_errors: &mut Vec<String>,
) -> Vec<Document> {
    if !dir.exists() {
        return Vec::new();
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            load_errors.push(format!(
                "{label} load failed: could not read {}: {error}",
                display_path(dir)
            ));
            return Vec::new();
        }
    };

    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("md") {
                    paths.push(path);
                }
            }
            Err(error) => load_errors.push(format!(
                "{label} load warning: could not inspect entry in {}: {error}",
                display_path(dir)
            )),
        }
    }
    paths.sort();

    let mut docs = Vec::new();
    for path in paths {
        match read_document(&path, location) {
            Ok(doc) => docs.push(doc),
            Err(error) => load_errors.push(format!("{label} load warning: {}", error.message)),
        }
    }
    docs
}

fn validation_load_errors(
    docs: &[Document],
    logs: &[Document],
    configured_states: &[String],
) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut ids = BTreeSet::new();
    let mut id_paths: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for doc in docs.iter().chain(logs.iter()) {
        let id = doc.id().trim();
        if !id.is_empty() {
            ids.insert(id.to_string());
            id_paths
                .entry(id.to_string())
                .or_default()
                .push(display_path(&doc.path));
        }
    }

    for (id, paths) in id_paths.iter().filter(|(_, paths)| paths.len() > 1) {
        warnings.push(format!(
            "Validation error: duplicate id `{id}` in {}",
            paths.join(", ")
        ));
    }

    for doc in docs.iter().chain(logs.iter()) {
        let mut errors = Vec::new();
        if doc.id().trim().is_empty() {
            errors.push("missing required field `id`".to_string());
        }
        if doc.title().trim().is_empty() {
            errors.push("missing required field `title`".to_string());
        }

        match doc.field("type") {
            Some(doc_type) if !doc_type.trim().is_empty() => {}
            _ => errors.push("missing required field `type`".to_string()),
        }

        if doc.location == DocumentLocation::Board && doc.doc_type() == "task" {
            match doc.field("state") {
                Some(state) if configured_states.iter().any(|known| known == state) => {}
                Some(state) if !state.trim().is_empty() => errors.push(format!(
                    "unknown state `{state}`; known states: {}",
                    configured_states.join(", ")
                )),
                _ => errors.push("missing required field `state`".to_string()),
            }
        }

        if doc.location == DocumentLocation::Logs && doc.doc_type() == "task" {
            if doc.field("completedAt").is_none() {
                errors.push("missing required log field `completedAt`".to_string());
            }
            if completion_summary(doc).is_none() {
                errors.push("missing required log field `completion.summary`".to_string());
            }
        }

        if let Some(parent) = doc
            .field("parentId")
            .filter(|value| !value.trim().is_empty())
        {
            if !ids.contains(parent) {
                errors.push(format!("unresolved parentId `{parent}`"));
            }
        }
        for blocker in doc
            .field("blockers")
            .map(parse_field_values)
            .unwrap_or_default()
        {
            if !ids.contains(&blocker) {
                errors.push(format!("unresolved blocker `{blocker}`"));
            }
        }
        if has_metadata(doc, "accord") || doc.field("accordStatus").is_some() {
            match accord_status(doc) {
                Some(status) if ACCORD_STATUSES.contains(&status) => {}
                Some(status) => errors.push(format!("invalid accord.status `{status}`")),
                None => errors
                    .push("accord.status is required when accord metadata is present".to_string()),
            }
        }
        if has_metadata(doc, "review") || doc.field("reviewStatus").is_some() {
            match review_status(doc) {
                Some(status) if REVIEW_STATUSES.contains(&status) => {}
                Some(status) => errors.push(format!("invalid review.status `{status}`")),
                None => errors
                    .push("review.status is required when review metadata is present".to_string()),
            }
        }

        if !errors.is_empty() {
            warnings.push(format!(
                "Validation error: {}: {}",
                display_path(&doc.path),
                errors.join("; ")
            ));
        }
    }

    warnings
}

fn runtime_warning_status_note(outcome: &ReloadOutcome) -> String {
    match outcome.warning_count {
        0 => String::new(),
        1 => format!(
            "; 1 runtime warning: {}",
            truncate(
                outcome.first_warning.as_deref().unwrap_or("inspect status"),
                120
            )
        ),
        count => format!(
            "; {count} runtime warnings; first: {}",
            truncate(
                outcome.first_warning.as_deref().unwrap_or("inspect status"),
                120
            )
        ),
    }
}

fn collect_reload_fingerprint(workspace: &Workspace) -> ReloadFingerprint {
    let mut files = BTreeMap::new();
    insert_optional_fingerprint(&mut files, workspace.config_path.clone());
    insert_optional_fingerprint(&mut files, workspace.events_path.clone());
    insert_optional_fingerprint(
        &mut files,
        workspace.config_path.with_file_name("theme.toml"),
    );
    insert_directory_fingerprints(&mut files, &workspace.board_dir, "md");
    insert_directory_fingerprints(&mut files, &workspace.logs_dir, "md");
    if let Some(user_theme_dir) = theme::user_theme_dir_from_env() {
        insert_directory_fingerprints(&mut files, &user_theme_dir, "toml");
    }
    ReloadFingerprint { files }
}

fn insert_optional_fingerprint(
    files: &mut BTreeMap<PathBuf, Option<FileSignature>>,
    path: PathBuf,
) {
    let signature = file_signature(&path).ok();
    files.insert(path, signature);
}

fn insert_directory_fingerprints(
    files: &mut BTreeMap<PathBuf, Option<FileSignature>>,
    dir: &Path,
    extension: &str,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            insert_optional_fingerprint(files, path);
        }
    }
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

fn list_item_for_doc(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
) -> ListItem<'static> {
    ListItem::new(board_item_lines_for_doc(
        doc,
        theme,
        content_width,
        show_doc_type,
    ))
}

fn board_item_lines_for_doc(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
) -> Vec<Line<'static>> {
    // Board rows are intentionally sparse. The Board is for scanning and choosing work;
    // details belong in the detail pane. Add chips here only when they change the next
    // action or scan priority, and render them as real terminal badges (fg + bg), not
    // as another color in a noisy line of metadata.
    let priority = doc.field("priority").unwrap_or("-");
    let mut chips: Vec<(String, Style)> = Vec::new();
    if let Some(priority_chip) = priority_chip(priority) {
        chips.push((priority_chip, theme.priority_chip_style(priority)));
    }
    if let Some(accord) =
        accord_status(doc).filter(|status| board_should_surface_accord_status(status))
    {
        chips.push((status_chip(accord), theme.accord_chip_style(accord)));
    }
    if let Some(review) =
        review_status(doc).filter(|status| board_should_surface_review_status(status))
    {
        chips.push((status_chip(review), theme.review_chip_style(review)));
    }
    if let Some((completed, total)) = subtask_progress(doc) {
        let tone = if completed == total {
            StatusTone::Success
        } else {
            StatusTone::Warning
        };
        chips.push((
            chip_text(&format!("{completed}/{total}")),
            theme.progress_chip_style(tone),
        ));
    }

    let doc_type = doc_type_badge(doc, show_doc_type);
    let chip_width = chips
        .iter()
        .map(|(chip, _)| text_width(chip))
        .sum::<usize>()
        + chips.len();
    let doc_type_width = doc_type
        .as_ref()
        .map(|badge| text_width(badge) + 1)
        .unwrap_or(0);
    let fixed_width = doc_type_width + chip_width + 1 + text_width(doc.id());
    let title_width = content_width.saturating_sub(fixed_width).max(12);
    let title = truncate(doc.title(), title_width);
    let used_before_id = doc_type_width + text_width(&title) + chip_width;
    let spacer_width = content_width
        .saturating_sub(used_before_id + text_width(doc.id()))
        .max(1);

    let mut title_spans = Vec::new();
    if let Some(doc_type) = doc_type {
        title_spans.push(Span::styled(doc_type, theme.board_doc_type_style()));
        title_spans.push(Span::raw(" "));
    }
    title_spans.push(Span::styled(
        title,
        theme.text_style().add_modifier(Modifier::BOLD),
    ));
    for (chip, style) in chips {
        title_spans.push(Span::raw(" "));
        title_spans.push(Span::styled(chip, style));
    }
    title_spans.push(Span::raw(" ".repeat(spacer_width)));
    title_spans.push(Span::styled(doc.id().to_string(), theme.muted_style()));

    vec![Line::from(title_spans)]
}

fn doc_type_badge(doc: &Document, show_doc_type: bool) -> Option<String> {
    let doc_type = doc.doc_type().trim();
    if doc_type.is_empty() || (!show_doc_type && doc_type.eq_ignore_ascii_case("task")) {
        None
    } else {
        Some(doc_type.to_ascii_lowercase())
    }
}

fn priority_chip(priority: &str) -> Option<String> {
    let label = match priority.trim().to_ascii_lowercase().as_str() {
        "critical" | "urgent" => "CRIT",
        "high" => "HIGH",
        "medium" | "med" => "MED",
        "low" => "LOW",
        "" | "-" | "none" => return None,
        other => {
            return Some(chip_text(
                &other.chars().take(4).collect::<String>().to_uppercase(),
            ))
        }
    };
    Some(chip_text(label))
}

fn status_chip(status: &str) -> String {
    chip_text(&status.trim().replace('_', "-").to_uppercase())
}

fn chip_text(label: &str) -> String {
    format!(" {label} ")
}

fn board_should_surface_accord_status(status: &str) -> bool {
    matches!(
        normalized_accord_status(status).as_str(),
        "delivered" | "rework" | "blocked" | "failed"
    )
}

fn board_should_surface_review_status(status: &str) -> bool {
    matches!(
        status
            .trim()
            .replace('_', "-")
            .to_ascii_lowercase()
            .as_str(),
        "pending" | "changes-requested" | "rejected" | "failed"
    )
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
        lines.extend(markdownish_lines(&doc.body, theme));
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
        "ready" => format!("tandem accord claim {id} --assignee <name>"),
        "claimed" => format!(
            "tandem accord deliver {id} --summary <text> [--evidence <text>] [--file-changed <path>]"
        ),
        "delivered" => format!(
            "tandem accord accept {id} [--reviewer <name>] [--note <text>] OR tandem accord rework {id} --note <text>"
        ),
        "accepted" => format!(
            "tandem complete {id} --summary <text> [--validation <text>] [--reviewer <name>]"
        ),
        "rework" => format!("tandem accord deliver {id} --summary <text> [--evidence <text>]"),
        "blocked" => format!(
            "tandem accord ready {id} [--assignee <name>] OR tandem accord fail {id} --reason <text>"
        ),
        "failed" => format!("tandem accord ready {id} [--assignee <name>]"),
        "missing" | "" => format!(
            "tandem accord ready {id} [--assignee <name>] [--deliverable <spec>] [--validation <command>]"
        ),
        _ => format!("tandem show {id}  # inspect accord metadata before mutating"),
    }
}

fn normalized_accord_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace('_', "-")
}

fn markdownish_lines(markdown: &str, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_fence = false;

    for line in markdown.lines() {
        if markdown_fence_marker(line).is_some() {
            lines.push(markdown_code_fence_line(line, theme));
            in_code_fence = !in_code_fence;
        } else if in_code_fence {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                theme.markdown_code_style(),
            )));
        } else {
            lines.push(markdownish_line(line, theme));
        }
    }

    lines
}

fn markdownish_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return Line::from("");
    }

    let indent = &line[..line.len() - trimmed.len()];
    if let Some(heading) = markdown_heading_text(trimmed) {
        return Line::from(with_indent(
            indent,
            vec![Span::styled(
                heading.to_string(),
                theme.markdown_heading_style().add_modifier(Modifier::BOLD),
            )],
            theme,
        ));
    }

    if let Some(quote) = markdown_blockquote_text(trimmed) {
        let mut spans = with_indent(indent, vec![Span::styled("│ ", theme.muted_style())], theme);
        spans.extend(markdown_inline_spans(
            quote,
            theme,
            theme.muted_style().add_modifier(Modifier::ITALIC),
        ));
        return Line::from(spans);
    }

    if let Some((marker, content)) = markdown_list_parts(trimmed) {
        let mut spans = with_indent(
            indent,
            vec![Span::styled(marker, theme.markdown_list_style())],
            theme,
        );
        spans.extend(markdown_inline_spans(content, theme, theme.text_style()));
        return Line::from(spans);
    }

    Line::from(markdown_inline_spans(line, theme, theme.text_style()))
}

fn markdown_fence_marker(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn markdown_code_fence_line(line: &str, theme: &TuiTheme) -> Line<'static> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let marker = markdown_fence_marker(line).unwrap_or("```");
    let info = trimmed[marker.len()..].trim();
    let mut spans = with_indent(
        indent,
        vec![Span::styled(
            marker.to_string(),
            theme.markdown_code_style().add_modifier(Modifier::BOLD),
        )],
        theme,
    );
    if !info.is_empty() {
        spans.push(Span::styled(" ", theme.markdown_code_style()));
        spans.push(Span::styled(info.to_string(), theme.muted_style()));
    }
    Line::from(spans)
}

fn markdown_heading_text(trimmed: &str) -> Option<&str> {
    let marker_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }

    let rest = &trimmed[marker_count..];
    if rest.is_empty() || rest.chars().next().is_some_and(|ch| ch.is_whitespace()) {
        let heading = rest.trim_start();
        Some(if heading.is_empty() { trimmed } else { heading })
    } else {
        None
    }
}

fn markdown_blockquote_text(trimmed: &str) -> Option<&str> {
    let quote = trimmed.strip_prefix('>')?;
    Some(quote.strip_prefix(' ').unwrap_or(quote))
}

fn markdown_list_parts(trimmed: &str) -> Option<(String, &str)> {
    for bullet in ["-", "*", "+"] {
        let unchecked = format!("{bullet} [ ] ");
        if let Some(content) = trimmed.strip_prefix(&unchecked) {
            return Some(("☐ ".to_string(), content));
        }
        let checked_lower = format!("{bullet} [x] ");
        if let Some(content) = trimmed.strip_prefix(&checked_lower) {
            return Some(("☑ ".to_string(), content));
        }
        let checked_upper = format!("{bullet} [X] ");
        if let Some(content) = trimmed.strip_prefix(&checked_upper) {
            return Some(("☑ ".to_string(), content));
        }
        let marker = format!("{bullet} ");
        if let Some(content) = trimmed.strip_prefix(&marker) {
            return Some(("• ".to_string(), content));
        }
    }

    let digit_count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_count == 0 || digit_count + 2 > trimmed.len() {
        return None;
    }
    let suffix = &trimmed[digit_count..];
    if suffix.starts_with(". ") || suffix.starts_with(") ") {
        Some((
            trimmed[..digit_count + 2].to_string(),
            &trimmed[digit_count + 2..],
        ))
    } else {
        None
    }
}

fn markdown_inline_spans(text: &str, theme: &TuiTheme, base_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(strong) = rest.strip_prefix("**") {
            if let Some(end) = strong.find("**") {
                push_span(
                    &mut spans,
                    &strong[..end],
                    base_style.add_modifier(Modifier::BOLD),
                );
                rest = &strong[end + 2..];
                continue;
            }
        }

        if let Some(code) = rest.strip_prefix('`') {
            if let Some(end) = code.find('`') {
                push_span(&mut spans, &code[..end], theme.markdown_code_style());
                rest = &code[end + 1..];
                continue;
            }
        }

        if let Some((consumed, label, url)) = markdown_link_parts(rest) {
            let link_label = if label.is_empty() { url } else { label };
            push_span(
                &mut spans,
                link_label,
                theme
                    .status_style(StatusTone::Accent)
                    .add_modifier(Modifier::UNDERLINED),
            );
            if !url.is_empty() && url != label {
                push_span(&mut spans, " (", theme.muted_style());
                push_span(&mut spans, url, theme.muted_style());
                push_span(&mut spans, ")", theme.muted_style());
            }
            rest = &rest[consumed..];
            continue;
        }

        let next_special = next_markdown_inline_special(rest).unwrap_or(rest.len());
        if next_special > 0 {
            push_span(&mut spans, &rest[..next_special], base_style);
            rest = &rest[next_special..];
        } else {
            let ch = rest.chars().next().expect("rest is not empty");
            push_span(&mut spans, &rest[..ch.len_utf8()], base_style);
            rest = &rest[ch.len_utf8()..];
        }
    }

    spans
}

fn markdown_link_parts(text: &str) -> Option<(usize, &str, &str)> {
    let label_rest = text.strip_prefix('[')?;
    let label_end = label_rest.find(']')?;
    let after_label = &label_rest[label_end + 1..];
    let url_rest = after_label.strip_prefix('(')?;
    let url_end = url_rest.find(')')?;
    let consumed = 1 + label_end + 1 + 1 + url_end + 1;
    Some((consumed, &label_rest[..label_end], &url_rest[..url_end]))
}

fn next_markdown_inline_special(text: &str) -> Option<usize> {
    ['`', '[', '*']
        .iter()
        .filter_map(|needle| text.find(*needle))
        .min()
}

fn with_indent(
    indent: &str,
    mut spans: Vec<Span<'static>>,
    theme: &TuiTheme,
) -> Vec<Span<'static>> {
    if !indent.is_empty() {
        spans.insert(0, Span::styled(indent.to_string(), theme.text_style()));
    }
    spans
}

fn push_span(spans: &mut Vec<Span<'static>>, content: &str, style: Style) {
    if !content.is_empty() {
        spans.push(Span::styled(content.to_string(), style));
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
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn board_row_is_sparse_and_uses_real_chips_for_scan_signals() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-23", Some("todo"));
        doc.fields
            .insert("title".to_string(), "Polish Board rows".to_string());
        doc.fields
            .insert("priority".to_string(), "high".to_string());
        doc.fields
            .insert("tags".to_string(), "[\"tui\", \"board\"]".to_string());
        doc.fields
            .insert("accord.status".to_string(), "ready".to_string());

        let lines = board_item_lines_for_doc(&doc, &theme, 120, false);
        let title = line_text(&lines[0]);

        assert_eq!(lines.len(), 1);
        assert!(title.contains("Polish Board rows  HIGH"));
        assert!(!title.contains("accord ready"));
        assert!(!title.contains("tui"));
        assert!(!title.contains("[task]"));
        assert!(!title.contains("A:"));
        assert!(lines[0].spans.iter().any(|span| {
            span.content.as_ref() == " HIGH " && span.style == theme.priority_chip_style("high")
        }));
    }

    #[test]
    fn board_row_shows_type_only_when_mixed_or_non_default() {
        let theme = TuiTheme::default_dark();
        let mut task = doc_with_state("task-1", Some("todo"));
        task.fields
            .insert("title".to_string(), "Default work".to_string());
        task.fields
            .insert("priority".to_string(), "low".to_string());

        let default_context = line_text(&board_item_lines_for_doc(&task, &theme, 96, false)[0]);
        let mixed_context = line_text(&board_item_lines_for_doc(&task, &theme, 96, true)[0]);
        assert!(default_context.contains("Default work  LOW"));
        assert!(!default_context.contains("task Default work"));
        assert!(mixed_context.contains("task Default work  LOW"));

        let mut decision = task.clone();
        decision
            .fields
            .insert("id".to_string(), "decision-1".to_string());
        decision
            .fields
            .insert("type".to_string(), "decision".to_string());
        decision
            .fields
            .insert("title".to_string(), "Choose layout".to_string());
        let non_default = line_text(&board_item_lines_for_doc(&decision, &theme, 96, false)[0]);
        assert!(non_default.contains("decision Choose layout  LOW"));
    }

    #[test]
    fn markdownish_lines_render_common_markdown_constructs() {
        let theme = TuiTheme::default_dark();
        let lines = markdownish_lines(
            "# Heading\n\n- item with `code` and [docs](https://example.test)\n1. ordered\n> quoted `code`\n```rust\n- not a list\n```\nplain **bold**",
            &theme,
        );
        let texts = lines.iter().map(line_text).collect::<Vec<_>>();

        assert_eq!(
            texts,
            vec![
                "Heading",
                "",
                "• item with code and docs (https://example.test)",
                "1. ordered",
                "│ quoted code",
                "``` rust",
                "- not a list",
                "```",
                "plain bold",
            ]
        );
        assert_eq!(
            lines[0].spans[0].style,
            theme.markdown_heading_style().add_modifier(Modifier::BOLD)
        );
        assert_eq!(lines[2].spans[0].style, theme.markdown_list_style());
        assert!(lines[2].spans.iter().any(
            |span| span.content.as_ref() == "code" && span.style == theme.markdown_code_style()
        ));
        assert!(lines[2].spans.iter().any(|span| {
            span.content.as_ref() == "docs"
                && span.style
                    == theme
                        .status_style(StatusTone::Accent)
                        .add_modifier(Modifier::UNDERLINED)
        }));
        assert_eq!(lines[4].spans[0].content.as_ref(), "│ ");
        assert_eq!(
            lines[4].spans[1].style,
            theme.muted_style().add_modifier(Modifier::ITALIC)
        );
        assert_eq!(lines[6].spans[0].style, theme.markdown_code_style());
        assert!(lines[8]
            .spans
            .iter()
            .any(|span| span.content.as_ref() == "bold"
                && span.style == theme.text_style().add_modifier(Modifier::BOLD)));
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

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()))
    }

    fn temp_workspace(root: &Path) -> Workspace {
        let tandem_dir = root.join(".tandem");
        let workspace = Workspace {
            board_dir: tandem_dir.join("board"),
            logs_dir: tandem_dir.join("logs"),
            config_path: tandem_dir.join("tandem.md"),
            events_path: tandem_dir.join("events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\ntitle: Test Workspace\nstates: [todo, review]\n---\n",
        )
        .unwrap();
        workspace
    }

    fn write_task_doc(workspace: &Workspace, id: &str, title: &str, state: &str) {
        fs::write(
            workspace.board_dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\ntitle: {title}\nstate: {state}\n---\n\nBody for {id}.\n"
            ),
        )
        .unwrap();
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
            reload_fingerprint: ReloadFingerprint::default(),
            last_reload_check: Instant::now(),
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
    fn editor_key_requests_open_for_board_and_marks_read_only_views() {
        let mut app = keyboard_test_app();
        assert_eq!(
            app.handle_key(key(KeyCode::Char('e'))).unwrap(),
            KeyAction::OpenEditor
        );

        app.switch_view(TuiView::Logs);
        assert_eq!(
            app.handle_key(key(KeyCode::Char('e'))).unwrap(),
            KeyAction::Continue
        );
        assert!(app.status.contains("read-only"));

        app.switch_view(TuiView::Decisions);
        assert_eq!(
            app.handle_key(key(KeyCode::Char('e'))).unwrap(),
            KeyAction::Continue
        );
        assert!(app.status.contains("deferred"));
    }

    #[test]
    fn editor_targets_active_tasks_from_board_and_review_only() {
        let mut app = keyboard_test_app();
        assert_eq!(app.selected_editor_target().unwrap().id, "task-1");

        app.selected_state = 2;
        let error = app.selected_editor_target().unwrap_err();
        assert!(error.contains("type `decision`"));

        app.docs[1]
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());
        app.switch_view(TuiView::Review);
        assert_eq!(app.selected_editor_target().unwrap().id, "task-2");
    }

    #[test]
    fn split_editor_command_supports_arguments_and_quotes() {
        assert_eq!(
            split_editor_command("code --wait 'two words.md'").unwrap(),
            vec!["code", "--wait", "two words.md"]
        );
        assert_eq!(
            split_editor_command("\"/tmp/my editor\" --flag").unwrap(),
            vec!["/tmp/my editor", "--flag"]
        );
        assert!(split_editor_command("vim '")
            .unwrap_err()
            .contains("unterminated"));
    }

    #[test]
    fn reload_preserves_selected_document_by_id_after_external_state_change() {
        let root = unique_test_dir("tandem-reload-preserve");
        let workspace = temp_workspace(&root);
        write_task_doc(&workspace, "task-1", "Task one", "todo");
        write_task_doc(&workspace, "task-2", "Task two", "review");

        let mut app = TuiApp::load(workspace.clone()).unwrap();
        assert!(app.select_document_by_id("task-2"));
        assert_eq!(app.selected_doc().map(Document::id), Some("task-2"));

        write_task_doc(&workspace, "task-2", "Task two", "todo");
        let outcome = app.reload();

        assert_eq!(outcome.warning_count, 0);
        assert_eq!(app.selected_doc().map(Document::id), Some("task-2"));
        assert_eq!(
            app.states.get(app.selected_state).map(String::as_str),
            Some("todo")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reload_surfaces_parse_errors_without_panicking() {
        let root = unique_test_dir("tandem-reload-error");
        let workspace = temp_workspace(&root);
        write_task_doc(&workspace, "task-1", "Task one", "todo");

        let mut app = TuiApp::load(workspace.clone()).unwrap();
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\ntitle: Broken\nstate: todo\n\nmissing closing delimiter\n",
        )
        .unwrap();

        let outcome = app.reload();

        assert!(outcome.warning_count >= 1);
        assert!(app
            .load_errors
            .iter()
            .any(|error| error.contains("Board load warning")));
        assert!(app.status.contains("runtime warning"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn idle_hot_reload_detects_external_board_file_changes() {
        let root = unique_test_dir("tandem-auto-reload");
        let workspace = temp_workspace(&root);
        write_task_doc(&workspace, "task-1", "Task one", "todo");

        let mut app = TuiApp::load(workspace.clone()).unwrap();
        app.last_reload_check = Instant::now() - Duration::from_secs(1);
        write_task_doc(&workspace, "task-2", "Task two", "review");

        app.reload_if_changed();

        assert!(app.docs.iter().any(|doc| doc.id() == "task-2"));
        assert!(app.status.contains("External changes detected"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn run_editor_command_smoke_appends_to_document() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_test_dir("tandem-editor-smoke");
        fs::create_dir_all(&root).unwrap();
        let script = root.join("editor-smoke.sh");
        let doc = root.join("task-1.md");
        fs::write(
            &script,
            "#!/bin/sh\nprintf '\\nsmoke editor touched %s\\n' \"$1\" >> \"$1\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();
        fs::write(
            &doc,
            "---\nid: task-1\ntype: task\ntitle: Test\nstate: todo\n---\n",
        )
        .unwrap();

        let command = EditorCommand::from_value(&script.to_string_lossy(), "test editor").unwrap();
        let status = run_editor_command(&command, &doc).unwrap();
        assert!(status.success());
        assert!(fs::read_to_string(&doc)
            .unwrap()
            .contains("smoke editor touched"));
        fs::remove_dir_all(root).unwrap();
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
            .any(|text| text.contains("CLI hint: tandem accord accept task-1")));
        assert!(texts.iter().any(|text| text
            .contains("TUI accord mutations are planned; this Board detail pane is read-only.")));
        assert!(texts.contains(&"Description".to_string()));
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
