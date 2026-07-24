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
#[allow(dead_code)]
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
    Logs,
    Rules,
    Decisions,
}

impl TuiView {
    const ALL: [Self; 4] = [Self::Board, Self::Logs, Self::Rules, Self::Decisions];

    fn from_digit(ch: char) -> Option<Self> {
        match ch {
            '1' => Some(Self::Board),
            '2' => Some(Self::Logs),
            '3' => Some(Self::Rules),
            '4' => Some(Self::Decisions),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Logs => "Logs",
            Self::Rules => "Rules",
            Self::Decisions => "Decisions",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::Board => "1",
            Self::Logs => "2",
            Self::Rules => "3",
            Self::Decisions => "4",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    Continue,
    Quit,
    OpenEditor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum HitAction {
    SwitchView(TuiView),
    SelectState(usize),
    SelectBoardItem(usize, usize),
    ToggleBoardExpansion,
    ToggleBoardDetail,
    ToggleBoardArrangement,
    StartQuickAdd,
    CycleBoardTagFilter,
    CycleBoardPriorityFilter,
    ClearBoardFilters,
    MoveSelectedTask(isize),
    ShowValidationAction(&'static str),
    OpenEditor,
    ShowHelp,
    FocusDetail,
    FocusReviewList,
    SelectReviewItem(usize),
    FocusReviewDetail,
    SelectLog(usize),
    FocusLogList,
    FocusLogDetail,
    StartLogSearch,
    ToggleFocus,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValidationPrompt {
    Accept {
        id: String,
        title: String,
    },
    Rework {
        id: String,
        title: String,
        feedback: String,
    },
    ApplyAccepted {
        candidates: Vec<ValidationApplyCandidate>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationApplyCandidate {
    id: String,
    title: String,
}

impl ValidationPrompt {
    fn id(&self) -> &str {
        match self {
            Self::Accept { id, .. } | Self::Rework { id, .. } => id,
            Self::ApplyAccepted { .. } => "accepted candidates",
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::Accept { title, .. } | Self::Rework { title, .. } => title,
            Self::ApplyAccepted { .. } => "Apply accepted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardArrangement {
    State,
    Epic,
}

impl BoardArrangement {
    fn label(self) -> &'static str {
        match self {
            Self::State => "State",
            Self::Epic => "Epic",
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::State => Self::Epic,
            Self::Epic => Self::State,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BoardFilters {
    tag: Option<String>,
    priority: Option<String>,
}

impl BoardFilters {
    fn is_active(&self) -> bool {
        self.tag.is_some() || self.priority.is_some()
    }

    fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(tag) = self.tag.as_deref() {
            parts.push(format!("#{}", tag));
        }
        if let Some(priority) = self.priority.as_deref() {
            parts.push(format!("priority {}", priority));
        }
        if parts.is_empty() {
            "no Board filters".to_string()
        } else {
            format!("filter {}", parts.join(" · "))
        }
    }
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
struct TuiHierarchySnapshot {
    index: Option<HierarchyIndex>,
    errors: Vec<String>,
}

impl TuiHierarchySnapshot {
    fn from_documents(active_docs: &[Document], completed_logs: &[Document]) -> Self {
        match hierarchy_index_for(active_docs, completed_logs) {
            Ok(index) => Self {
                errors: index.task_hierarchy_errors(),
                index: Some(index),
            },
            Err(error) => Self {
                index: None,
                errors: vec![error.message],
            },
        }
    }

    fn valid_index(&self) -> Option<&HierarchyIndex> {
        self.errors
            .is_empty()
            .then_some(self.index.as_ref())
            .flatten()
    }
}

#[derive(Debug, Clone, Default)]
struct ReloadSelection {
    board_doc_id: Option<String>,
    board_state: Option<String>,
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
    hierarchy: TuiHierarchySnapshot,
    log_events: logs::LogEventsById,
    rules: RulesByCategory,
    load_errors: Vec<String>,
    theme: TuiTheme,
    theme_source: String,
    theme_warnings: Vec<String>,
    selected_state: usize,
    selected_item: usize,
    selected_review_item: usize,
    board_filters: BoardFilters,
    board_arrangement: BoardArrangement,
    selected_log: usize,
    focus: FocusPane,
    show_board_detail: bool,
    expanded_board_doc_id: Option<String>,
    expanded_board_hierarchy_ids: BTreeSet<String>,
    detail_scroll: u16,
    review_detail_scroll: u16,
    log_detail_scroll: u16,
    log_search_filter: String,
    log_search_input: Option<String>,
    status: String,
    show_help: bool,
    quick_add: Option<QuickAddInput>,
    validation_prompt: Option<ValidationPrompt>,
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
            hierarchy: TuiHierarchySnapshot::default(),
            log_events: logs::LogEventsById::new(),
            rules: empty_rules(),
            load_errors: Vec::new(),
            theme: TuiTheme::default_dark(),
            theme_source: String::new(),
            theme_warnings: Vec::new(),
            selected_state: 0,
            selected_item: 0,
            selected_review_item: 0,
            board_filters: BoardFilters::default(),
            board_arrangement: BoardArrangement::State,
            selected_log: 0,
            focus: FocusPane::Board,
            show_board_detail: false,
            expanded_board_doc_id: None,
            expanded_board_hierarchy_ids: BTreeSet::new(),
            detail_scroll: 0,
            review_detail_scroll: 0,
            log_detail_scroll: 0,
            log_search_filter: String::new(),
            log_search_input: None,
            status: String::new(),
            show_help: false,
            quick_add: None,
            validation_prompt: None,
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
        let _hierarchy_lock = match HierarchyLock::acquire(&self.workspace) {
            Ok(lock) => lock,
            Err(error) => {
                let warning = format!("TUI reload failed closed: {}", error.message);
                self.load_errors = vec![warning.clone()];
                self.hierarchy = TuiHierarchySnapshot {
                    index: None,
                    errors: vec![warning.clone()],
                };
                self.status = warning.clone();
                return ReloadOutcome::from_warnings(&[warning]);
            }
        };
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
        let hierarchy = TuiHierarchySnapshot::from_documents(&docs, &log_load.docs);
        load_errors.extend(log_load.warnings);
        let (log_events, event_warnings) = logs::load_log_events(&self.workspace.events_path);
        load_errors.extend(event_warnings);
        load_errors.extend(validation_load_errors_with_hierarchy(
            &docs,
            &log_load.docs,
            &configured_states,
            &hierarchy,
        ));

        self.title = title;
        self.states = states_with_board_docs(configured_states.clone(), &docs);
        self.configured_states = configured_states;
        self.docs = docs;
        self.logs = log_load.docs;
        self.hierarchy = hierarchy;
        let active_ids = self
            .docs
            .iter()
            .map(|doc| doc.id().to_string())
            .collect::<BTreeSet<_>>();
        // Keep hierarchy expansion IDs across tolerant parse gaps. Stale IDs are inert,
        // while a task that reappears after an editor's partial write regains its state.
        if self
            .expanded_board_doc_id
            .as_ref()
            .is_some_and(|id| !active_ids.contains(id))
        {
            self.expanded_board_doc_id = None;
        }
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
            || self.validation_prompt.is_some()
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
                    Event::Mouse(mouse) => match self.handle_mouse(mouse) {
                        KeyAction::Continue => {}
                        KeyAction::Quit => break,
                        KeyAction::OpenEditor => self.open_selected_item_in_editor(session)?,
                    },
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

        if self.validation_prompt.is_some() {
            self.handle_validation_prompt_key(key);
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
            KeyCode::Char('A') if self.view == TuiView::Board => self.start_validation_accept(),
            KeyCode::Char('R') if self.view == TuiView::Board => self.start_validation_rework(),
            KeyCode::Char('C') if self.view == TuiView::Board => self.start_validation_apply_accepted(),
            KeyCode::Char('H') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(-1)
            }
            KeyCode::Char('L') if self.view == TuiView::Board => {
                self.move_selected_task_by_delta(1)
            }
            KeyCode::Char('b') if self.view == TuiView::Board => self.toggle_board_arrangement(),
            KeyCode::Char('t') if self.view == TuiView::Board => self.cycle_board_tag_filter(),
            KeyCode::Char('p') if self.view == TuiView::Board => self.cycle_board_priority_filter(),
            KeyCode::Char('F') if self.view == TuiView::Board => self.clear_board_filters(),
            KeyCode::Char('H') | KeyCode::Char('L') => {
                self.status = "Task move is available in Board view; press 1 for Board.".to_string()
            }
            KeyCode::Char('/') if self.view == TuiView::Logs => self.start_log_search(),
            KeyCode::Char('/') => {
                self.status = "Search is available in Logs view; press 2 for Logs.".to_string()
            }
            KeyCode::Char('e') if self.view == TuiView::Board => {
                return Ok(KeyAction::OpenEditor)
            }
            KeyCode::Char('e') if self.view == TuiView::Logs => {
                self.status = "Completed logs are read-only in the TUI; $EDITOR is intentionally disabled for generated history.".to_string()
            }
            KeyCode::Char('e') if self.view == TuiView::Decisions => {
                self.status = "Use `tandem decision update <id> …` or `tandem decision withdraw <id> --reason …`; editor-based decision actions are deferred.".to_string()
            }
            KeyCode::Tab | KeyCode::BackTab => self.cycle_focus_or_hint(),
            KeyCode::Enter if self.view == TuiView::Board => self.toggle_board_expansion(),
            KeyCode::Char(' ') if self.view == TuiView::Board => self.toggle_board_preview(),
            KeyCode::Enter if self.view == TuiView::Logs => self.toggle_focus(),
            KeyCode::Esc => match self.view {
                TuiView::Board if self.focus == FocusPane::Detail => {
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
                "Board view active. Use b to switch State/Epic Board arrangement, h/l for states, j/k for rows, t/p filters, F clear.".to_string()
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
            TuiView::Board => self.toggle_board_detail(),
            TuiView::Logs | TuiView::Decisions => self.toggle_focus(),
            TuiView::Rules => {
                self.status = "Rules has a single category/list focus area; Tab stays in Rules. Use h/l for categories and 1..4 for views.".to_string();
            }
        }
    }

    fn focus_previous_pane(&mut self) {
        if matches!(self.view, TuiView::Logs | TuiView::Decisions) {
            self.focus = FocusPane::Board;
        }
    }

    fn focus_next_pane(&mut self) {
        if matches!(self.view, TuiView::Logs | TuiView::Decisions) {
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

    fn cycle_board_tag_filter(&mut self) {
        let tags = board_filter_tags(&self.docs);
        if tags.is_empty() {
            self.status = "No Board tags are available to filter.".to_string();
            return;
        }
        self.board_filters.tag = next_filter_value(self.board_filters.tag.as_deref(), &tags);
        self.selected_item = 0;
        self.detail_scroll = 0;
        self.clamp_selection();
        self.status = format!(
            "Board {}. Press t to cycle tags, F to clear.",
            self.board_filters.summary()
        );
    }

    fn cycle_board_priority_filter(&mut self) {
        let priorities = board_filter_priorities(&self.docs);
        if priorities.is_empty() {
            self.status = "No Board priorities are available to filter.".to_string();
            return;
        }
        self.board_filters.priority =
            next_filter_value(self.board_filters.priority.as_deref(), &priorities);
        self.selected_item = 0;
        self.detail_scroll = 0;
        self.clamp_selection();
        self.status = format!(
            "Board {}. Press p to cycle priorities, F to clear.",
            self.board_filters.summary()
        );
    }

    fn clear_board_filters(&mut self) {
        if self.board_filters.is_active() {
            self.board_filters = BoardFilters::default();
            self.selected_item = 0;
            self.detail_scroll = 0;
            self.clamp_selection();
            self.status = "Board filters cleared.".to_string();
        } else {
            self.status = "No Board filters are active.".to_string();
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        if !self.hierarchy.errors.is_empty() {
            self.status =
                "Quick add disabled: fix the persistent Board hierarchy errors and reload first."
                    .to_string();
            return;
        }
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

    fn selected_validation_doc_summary(&self) -> Result<(String, String, String), String> {
        let Some(doc) = self.selected_doc() else {
            return Err("No selected Board task for Validation action.".to_string());
        };
        if document_state_label(doc) != "validation" {
            return Err(format!(
                "Validation actions apply in the Validation state; selected {} is in {}.",
                doc.id(),
                display_state_label(&document_state_label(doc))
            ));
        }
        Ok((
            doc.id().to_string(),
            doc.title().to_string(),
            accord_status(doc).unwrap_or("missing").to_string(),
        ))
    }

    fn start_validation_accept(&mut self) {
        let (id, title, status) = match self.selected_validation_doc_summary() {
            Ok(summary) => summary,
            Err(message) => {
                self.status = message;
                return;
            }
        };
        if normalized_accord_status(&status) != "delivered" {
            self.status = format!(
                "Accept expects a delivered accord; {id} is {status}. Inspect before signing off."
            );
            return;
        }
        self.validation_prompt = Some(ValidationPrompt::Accept { id, title });
        self.status = "Confirm acceptance: Enter/y accepts sign-off, Esc/n cancels.".to_string();
    }

    fn start_validation_rework(&mut self) {
        let (id, title, status) = match self.selected_validation_doc_summary() {
            Ok(summary) => summary,
            Err(message) => {
                self.status = message;
                return;
            }
        };
        if normalized_accord_status(&status) != "delivered" {
            self.status = format!(
                "Request rework expects a delivered accord; {id} is {status}. Inspect before changing."
            );
            return;
        }
        self.validation_prompt = Some(ValidationPrompt::Rework {
            id,
            title,
            feedback: String::new(),
        });
        self.status = "Request rework: type feedback, Enter sends, Esc cancels.".to_string();
    }

    fn show_validation_complete_hint(&mut self) {
        self.status = "Completion is intentionally de-emphasized in Validation. Accept sign-off first; use C / Apply accepted to archive accepted work explicitly.".to_string();
    }

    fn start_validation_apply_accepted(&mut self) {
        if !self.hierarchy.errors.is_empty() {
            self.status = "Apply accepted disabled: fix the persistent Board hierarchy errors and reload first."
                .to_string();
            return;
        }
        let candidates = accepted_validation_candidates(&self.docs);
        if candidates.is_empty() {
            self.status = "No accepted Validation tasks are ready to apply/archive.".to_string();
            return;
        }
        self.status = format!(
            "Apply/archive {} accepted Validation task{}? Enter confirms, Esc cancels.",
            candidates.len(),
            plural_suffix(candidates.len())
        );
        self.validation_prompt = Some(ValidationPrompt::ApplyAccepted { candidates });
    }

    fn handle_validation_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let action = match self.validation_prompt.take() {
                    Some(ValidationPrompt::Accept { .. }) => "Acceptance",
                    Some(ValidationPrompt::Rework { .. }) => "Rework request",
                    Some(ValidationPrompt::ApplyAccepted { .. }) => "Apply accepted",
                    None => "Validation action",
                };
                self.status = format!("{action} canceled.");
            }
            KeyCode::Char('n')
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. } | ValidationPrompt::ApplyAccepted { .. })
                ) =>
            {
                let action = match self.validation_prompt.take() {
                    Some(ValidationPrompt::Accept { .. }) => "Acceptance",
                    Some(ValidationPrompt::ApplyAccepted { .. }) => "Apply accepted",
                    _ => "Validation action",
                };
                self.status = format!("{action} canceled.");
            }
            KeyCode::Char('y')
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. } | ValidationPrompt::ApplyAccepted { .. })
                ) =>
            {
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::ApplyAccepted { .. })
                ) {
                    self.finish_validation_apply_accepted();
                } else {
                    self.finish_validation_accept();
                }
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => {
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. })
                ) {
                    self.finish_validation_accept();
                } else if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::ApplyAccepted { .. })
                ) {
                    self.finish_validation_apply_accepted();
                } else {
                    self.finish_validation_rework();
                }
            }
            KeyCode::Backspace => {
                if let Some(ValidationPrompt::Rework { feedback, .. }) =
                    self.validation_prompt.as_mut()
                {
                    feedback.pop();
                }
                self.refresh_validation_prompt_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ValidationPrompt::Rework { feedback, .. }) =
                    self.validation_prompt.as_mut()
                {
                    feedback.clear();
                }
                self.refresh_validation_prompt_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(ValidationPrompt::Rework { feedback, .. }) =
                    self.validation_prompt.as_mut()
                {
                    feedback.push(ch);
                    self.refresh_validation_prompt_status();
                }
            }
            _ => {}
        }
    }

    fn refresh_validation_prompt_status(&mut self) {
        self.status = match self.validation_prompt.as_ref() {
            Some(ValidationPrompt::Accept { id, .. }) => {
                format!("Confirm acceptance for {id}: Enter/y accepts sign-off, Esc/n cancels.")
            }
            Some(ValidationPrompt::Rework { id, feedback, .. }) => format!(
                "Request rework for {id}: {} · Enter sends, Esc cancels",
                if feedback.trim().is_empty() {
                    "<feedback>"
                } else {
                    feedback.as_str()
                }
            ),
            Some(ValidationPrompt::ApplyAccepted { candidates }) => format!(
                "Apply/archive {} accepted Validation task{}: Enter confirms, Esc cancels.",
                candidates.len(),
                plural_suffix(candidates.len())
            ),
            None => self.status.clone(),
        };
    }

    fn finish_validation_accept(&mut self) {
        let Some(ValidationPrompt::Accept { id, .. }) = self.validation_prompt.take() else {
            return;
        };
        match apply_validation_accept(&self.workspace, &id) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = format!("Accepted sign-off for {}{}", outcome.id, reload_note);
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Accept error: {}{}", error.message, reload_note);
            }
        }
    }

    fn finish_validation_apply_accepted(&mut self) {
        let Some(ValidationPrompt::ApplyAccepted { candidates }) = self.validation_prompt.take()
        else {
            return;
        };
        match apply_accepted_validation_tasks(&self.workspace, &candidates) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.status = format!(
                    "Applied/archived {} accepted Validation task{} to logs: {}{}",
                    outcome.completed_ids.len(),
                    plural_suffix(outcome.completed_ids.len()),
                    outcome.completed_ids.join(", "),
                    reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Apply accepted error: {}{}", error.message, reload_note);
            }
        }
    }

    fn finish_validation_rework(&mut self) {
        let Some(ValidationPrompt::Rework { id, feedback, .. }) = self.validation_prompt.as_ref()
        else {
            return;
        };
        let feedback = feedback.trim().to_string();
        if feedback.is_empty() {
            self.status = format!(
                "Request rework for {id} needs feedback. Type feedback, Enter sends, Esc cancels."
            );
            return;
        }
        let id = id.clone();
        self.validation_prompt = None;
        match apply_validation_rework(&self.workspace, &id, &feedback) {
            Ok(outcome) => {
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = format!(
                    "Requested rework for {}; moved to {}{}",
                    outcome.id, outcome.state, reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("Rework error: {}{}", error.message, reload_note);
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
                let reload_note = self.reload().warning_note();
                self.select_document_by_id(&outcome.id);
                self.status = if outcome.changed {
                    format!(
                        "Moved {}: {} -> {}{}{}",
                        outcome.id,
                        outcome.from,
                        outcome.to,
                        outcome
                            .accord_sync
                            .as_deref()
                            .map(|sync| format!("; accord {sync}"))
                            .unwrap_or_default(),
                        reload_note
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
        if self.board_arrangement == BoardArrangement::Epic {
            let epic_index = self
                .epic_board_entries()
                .iter()
                .position(|entry| entry.doc.id() == id);
            if let Some(index) = epic_index {
                self.selected_item = index;
                if reset_scroll {
                    self.detail_scroll = 0;
                }
                self.clamp_selection();
                return true;
            }
        }

        self.expand_active_task_ancestors(id);
        for state_index in 0..self.states.len() {
            let Some(state_name) = self.states.get(state_index) else {
                continue;
            };
            if let Some(item_index) = self
                .state_board_entries(state_name)
                .iter()
                .position(|entry| entry.doc.id() == id)
            {
                self.selected_state = state_index;
                self.selected_item = item_index;
                if reset_scroll {
                    self.detail_scroll = 0;
                }
                self.clamp_selection();
                return true;
            }
        }
        self.clamp_selection();
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> KeyAction {
        if self.input_overlay_active() {
            return KeyAction::Continue;
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
                            let state_index = state_index.min(self.states.len().saturating_sub(1));
                            let was_selected = self.selected_state == state_index
                                && self.selected_item == item_index
                                && self.focus == FocusPane::Board;
                            self.selected_state = state_index;
                            self.selected_item = item_index;
                            self.detail_scroll = 0;
                            self.focus = FocusPane::Board;
                            self.clamp_selection();
                            if was_selected {
                                self.toggle_board_expansion();
                            }
                        }
                        HitAction::SelectBoardItem(_, _) => {}
                        HitAction::ToggleBoardExpansion if self.view == TuiView::Board => {
                            self.toggle_board_expansion()
                        }
                        HitAction::ToggleBoardExpansion => {}
                        HitAction::ToggleBoardDetail if self.view == TuiView::Board => {
                            self.toggle_board_detail()
                        }
                        HitAction::ToggleBoardDetail => {}
                        HitAction::ToggleBoardArrangement if self.view == TuiView::Board => {
                            self.toggle_board_arrangement()
                        }
                        HitAction::ToggleBoardArrangement => {}
                        HitAction::StartQuickAdd if self.view == TuiView::Board => {
                            self.start_quick_add()
                        }
                        HitAction::StartQuickAdd => {}
                        HitAction::CycleBoardTagFilter if self.view == TuiView::Board => {
                            self.cycle_board_tag_filter()
                        }
                        HitAction::CycleBoardTagFilter => {}
                        HitAction::CycleBoardPriorityFilter if self.view == TuiView::Board => {
                            self.cycle_board_priority_filter()
                        }
                        HitAction::CycleBoardPriorityFilter => {}
                        HitAction::ClearBoardFilters if self.view == TuiView::Board => {
                            self.clear_board_filters()
                        }
                        HitAction::ClearBoardFilters => {}
                        HitAction::MoveSelectedTask(delta) if self.view == TuiView::Board => {
                            self.move_selected_task_by_delta(delta)
                        }
                        HitAction::MoveSelectedTask(_) => {}
                        HitAction::ShowValidationAction(action) if self.view == TuiView::Board => {
                            self.show_validation_action_hint(action)
                        }
                        HitAction::ShowValidationAction(_) => {}
                        HitAction::OpenEditor if self.view == TuiView::Board => {
                            return KeyAction::OpenEditor
                        }
                        HitAction::OpenEditor => {}
                        HitAction::ShowHelp => self.show_help = true,
                        HitAction::FocusDetail if self.view == TuiView::Board => {
                            self.focus = FocusPane::Detail
                        }
                        HitAction::FocusDetail => {}
                        HitAction::FocusReviewList => {}
                        HitAction::SelectReviewItem(_) => {}
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
                        HitAction::StartLogSearch if self.view == TuiView::Logs => {
                            self.start_log_search()
                        }
                        HitAction::StartLogSearch => {}
                        HitAction::ToggleFocus => self.toggle_focus(),
                    }
                }
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Board => {
                self.scroll_board_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Board => {
                self.scroll_board_at(mouse, -3)
            }
            MouseEventKind::ScrollDown if self.view == TuiView::Logs => {
                self.scroll_logs_at(mouse, 3)
            }
            MouseEventKind::ScrollUp if self.view == TuiView::Logs => {
                self.scroll_logs_at(mouse, -3)
            }
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
        KeyAction::Continue
    }

    fn mouse_hit_action(&self, column: u16, row: u16) -> Option<HitAction> {
        self.hits
            .iter()
            .rev()
            .find(|hit| rect_contains(hit.rect, column, row))
            .map(|hit| hit.action.clone())
    }

    fn scroll_board_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusDetail) => {
                self.focus = FocusPane::Detail;
                if amount > 0 {
                    self.scroll_detail_down(amount as u16);
                } else {
                    self.scroll_detail_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::SelectState(_))
            | Some(HitAction::SelectBoardItem(_, _))
            | Some(HitAction::ToggleBoardExpansion)
            | None => {
                self.focus = FocusPane::Board;
                if amount > 0 {
                    self.next_item();
                } else {
                    self.previous_item();
                }
            }
            _ => {}
        }
    }

    fn scroll_logs_at(&mut self, mouse: MouseEvent, amount: i16) {
        match self.mouse_hit_action(mouse.column, mouse.row) {
            Some(HitAction::FocusLogDetail) => {
                self.focus = FocusPane::Detail;
                if amount > 0 {
                    self.scroll_log_detail_down(amount as u16);
                } else {
                    self.scroll_log_detail_up(amount.unsigned_abs());
                }
            }
            Some(HitAction::FocusLogList) | Some(HitAction::SelectLog(_)) | None => {
                self.focus = FocusPane::Board;
                if amount > 0 {
                    self.next_log();
                } else {
                    self.previous_log();
                }
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

    fn toggle_board_detail(&mut self) {
        self.show_board_detail = !self.show_board_detail;
        self.focus = if self.show_board_detail {
            FocusPane::Detail
        } else {
            FocusPane::Board
        };
        self.status = if self.show_board_detail {
            "Board detail pane shown; Tab or Esc returns to the list.".to_string()
        } else {
            "Board detail pane hidden; Space toggles the selected row preview.".to_string()
        };
    }

    fn toggle_board_arrangement(&mut self) {
        let selected_id = self.selected_doc().map(|doc| doc.id().to_string());
        self.board_arrangement = self.board_arrangement.toggled();
        self.selected_item = 0;
        self.detail_scroll = 0;
        if let Some(id) = selected_id.as_deref() {
            self.select_document_by_id_preserving_scroll(id);
        } else {
            self.clamp_selection();
        }
        self.status = format!(
            "Board arrangement: {}. Press b to switch State/Epic Board.",
            self.board_arrangement.label()
        );
    }

    fn toggle_board_expansion(&mut self) {
        let Some((doc_id, role)) = self.selected_doc().map(|doc| {
            (
                doc.id().to_string(),
                self.hierarchy
                    .index
                    .as_ref()
                    .and_then(|hierarchy| hierarchy.task_role(doc).ok().flatten()),
            )
        }) else {
            self.status = "No selected Board item to expand or preview.".to_string();
            return;
        };
        if self.board_arrangement == BoardArrangement::Epic {
            self.toggle_board_preview();
            self.status = if self.expanded_board_doc_id.as_deref() == Some(doc_id.as_str()) {
                format!("Previewing {doc_id} inline; press Enter to close.")
            } else {
                format!("Closed preview for {doc_id}.")
            };
            return;
        }
        let has_active_descendants = role.is_some()
            && count_task_descendants(
                &doc_id,
                &self.docs,
                &self.logs,
                &mut BTreeSet::from([doc_id.clone()]),
            )
            .0 > 0;
        if has_active_descendants {
            let children = match role {
                Some(TaskRole::Epic) => "Tasks",
                Some(TaskRole::Task) => "Subtasks",
                _ => "children",
            };
            self.expanded_board_doc_id = None;
            if self.expanded_board_hierarchy_ids.remove(&doc_id) {
                self.status = format!("Collapsed {children} under {doc_id}.");
            } else {
                self.expanded_board_hierarchy_ids.insert(doc_id.clone());
                self.status =
                    format!("Expanded {children} under {doc_id}; press Enter to collapse.");
            }
            self.clamp_selection();
        } else {
            self.toggle_board_preview();
        }
    }

    fn toggle_board_preview(&mut self) {
        let Some(doc_id) = self.selected_doc().map(|doc| doc.id().to_string()) else {
            self.status = "No selected Board item to preview.".to_string();
            return;
        };
        if self.expanded_board_doc_id.as_deref() == Some(doc_id.as_str()) {
            self.expanded_board_doc_id = None;
            self.status = format!("Closed preview for {doc_id}.");
        } else {
            self.expanded_board_doc_id = Some(doc_id.clone());
            self.status = format!("Previewing {doc_id} inline; press Space to close.");
        }
    }

    fn expand_active_task_ancestors(&mut self, id: &str) {
        let mut current = id.to_string();
        let mut visited = BTreeSet::from([current.clone()]);
        while let Some(parent_id) = self
            .docs
            .iter()
            .find(|doc| doc.id() == current)
            .and_then(normalized_parent_id)
        {
            if !visited.insert(parent_id.clone())
                || !self
                    .docs
                    .iter()
                    .any(|doc| doc.id() == parent_id && is_task_doc(doc))
            {
                break;
            }
            self.expanded_board_hierarchy_ids.insert(parent_id.clone());
            current = parent_id;
        }
    }

    fn previous_state(&mut self) {
        if self.board_arrangement == BoardArrangement::Epic {
            self.status =
                "Epic Board groups all workflow states; press b for State Board tabs.".to_string();
            return;
        }
        if self.selected_state > 0 {
            self.selected_state -= 1;
            self.selected_item = 0;
            self.detail_scroll = 0;
        }
        self.clamp_selection();
    }

    fn next_state(&mut self) {
        if self.board_arrangement == BoardArrangement::Epic {
            self.status =
                "Epic Board groups all workflow states; press b for State Board tabs.".to_string();
            return;
        }
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

    #[allow(dead_code)]
    fn previous_review_item(&mut self) {
        if self.selected_review_item > 0 {
            self.selected_review_item -= 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(dead_code)]
    fn next_review_item(&mut self) {
        let count = self.review_items().len();
        if self.selected_review_item + 1 < count {
            self.selected_review_item += 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(dead_code)]
    fn last_review_item(&mut self) {
        let count = self.review_items().len();
        if count > 0 {
            self.selected_review_item = count - 1;
            self.review_detail_scroll = 0;
        }
    }

    #[allow(dead_code)]
    fn scroll_review_detail_up(&mut self, amount: u16) {
        self.review_detail_scroll = self.review_detail_scroll.saturating_sub(amount);
    }

    #[allow(dead_code)]
    fn scroll_review_detail_down(&mut self, amount: u16) {
        let max_scroll = self.review_detail_line_count().saturating_sub(1) as u16;
        self.review_detail_scroll = self
            .review_detail_scroll
            .saturating_add(amount)
            .min(max_scroll);
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        if self.board_arrangement == BoardArrangement::Epic {
            return self.epic_board_entries().len();
        }
        self.states
            .get(self.selected_state)
            .map(|state| self.state_board_entries(state).len())
            .unwrap_or(0)
    }

    fn selected_state_summary(&self) -> String {
        if self.board_arrangement == BoardArrangement::Epic {
            let count = self.selected_state_count();
            return format!("EPIC · {} row{}", count, if count == 1 { "" } else { "s" });
        }
        let Some(state) = self.states.get(self.selected_state) else {
            return "No state · 0 items".to_string();
        };
        let visible_rows = self.selected_state_count();
        let state_tasks = self
            .docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .filter(|doc| document_state_label(doc) == state.as_str())
            .filter(|doc| board_filters_match(doc, &self.board_filters))
            .count();
        if visible_rows == state_tasks {
            format!(
                "{} · {} row{}",
                display_state_label(state),
                visible_rows,
                if visible_rows == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} · {} task{} · {} row{}",
                display_state_label(state),
                state_tasks,
                if state_tasks == 1 { "" } else { "s" },
                visible_rows,
                if visible_rows == 1 { "" } else { "s" }
            )
        }
    }

    fn selected_doc(&self) -> Option<&Document> {
        if self.board_arrangement == BoardArrangement::Epic {
            return self
                .epic_board_entries()
                .into_iter()
                .nth(self.selected_item)
                .map(|entry| entry.doc);
        }
        let state = self.states.get(self.selected_state)?;
        self.state_board_entries(state)
            .into_iter()
            .nth(self.selected_item)
            .map(|entry| entry.doc)
    }

    fn state_board_entries(&self, state: &str) -> Vec<StateBoardEntry<'_>> {
        let Some(hierarchy) = self.hierarchy.valid_index() else {
            return Vec::new();
        };
        state_board_entries_with_hierarchy(
            &self.docs,
            &self.logs,
            state,
            &self.board_filters,
            &self.expanded_board_hierarchy_ids,
            hierarchy,
        )
    }

    fn epic_board_entries(&self) -> Vec<EpicBoardEntry<'_>> {
        let Some(hierarchy) = self.hierarchy.valid_index() else {
            return Vec::new();
        };
        epic_board_entries_with_hierarchy(&self.docs, &self.logs, &self.board_filters, hierarchy)
    }

    fn relationship_context(&self, doc: &Document) -> BoardRelationshipContext {
        relationship_context_for_doc_with_hierarchy(
            doc,
            &self.docs,
            &self.logs,
            self.hierarchy.index.as_ref(),
        )
    }

    fn detail_line_count(&self) -> usize {
        self.selected_doc()
            .map(|doc| {
                detail_lines_for_doc_with_context(
                    doc,
                    &self.theme,
                    &relationship_context_for_doc_with_hierarchy(
                        doc,
                        &self.docs,
                        &self.logs,
                        self.hierarchy.index.as_ref(),
                    ),
                )
            })
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn filtered_logs(&self) -> Vec<&Document> {
        logs::filter_logs(
            &self.logs,
            self.hierarchy.index.as_ref(),
            &self.log_search_filter,
        )
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
            .map(|doc| {
                logs::detail_lines_for_log(
                    doc,
                    self.hierarchy.index.as_ref(),
                    self.log_events_for(doc.id()),
                    &self.theme,
                )
            })
            .map(|lines| lines.len())
            .unwrap_or(1)
    }

    fn logs_status_message(&self) -> String {
        let visible = self.filtered_logs().len();
        if self.log_search_filter.is_empty() {
            format!(
                "Logs view active: {} archived item{} loaded. Press / to search, j/k to select, and h/l or Tab for list/detail focus.",
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Logs filter `{}` matched {} of {} archived item{}; Esc clears filter.",
                self.log_search_filter,
                visible,
                self.logs.len(),
                if self.logs.len() == 1 { "" } else { "s" }
            )
        }
    }

    #[allow(dead_code)]
    fn review_items(&self) -> Vec<review::ReviewQueueItem> {
        review::queue_items_with_hierarchy(&self.docs, &self.logs, self.hierarchy.index.as_ref())
    }

    #[allow(dead_code)]
    fn selected_review_item(&self) -> Option<review::ReviewQueueItem> {
        review::selected_item(&self.docs, &self.logs, self.selected_review_item)
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn review_detail_line_count(&self) -> usize {
        let item = self.selected_review_item();
        review::detail_line_count(item.as_ref(), &self.theme)
    }

    fn board_docs(&self) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .collect()
    }

    fn decision_docs(&self) -> Vec<&Document> {
        self.docs
            .iter()
            .filter(|doc| is_decision_doc(doc))
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(4),
                Constraint::Min(4),
                Constraint::Length(1),
            ])
            .split(area);

        self.draw_header(frame, chunks[0]);
        if self.view == TuiView::Board {
            self.draw_board(frame, chunks[1]);
        } else {
            let view_area = chunks[1];
            if self.view == TuiView::Logs {
                self.draw_logs(frame, view_area);
            } else if self.view == TuiView::Rules {
                self.draw_rules_view(frame, view_area);
            } else if self.view == TuiView::Decisions {
                self.draw_decisions_view(frame, view_area);
            } else {
                self.draw_placeholder_view(frame, view_area);
            }
        }
        self.draw_footer(frame, chunks[2]);

        if self.validation_prompt.is_some() {
            self.draw_validation_prompt(frame, area);
        }

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
            if total > 0 {
                let width = 24usize;
                let filled = completed * width / total;
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

    fn draw_header(&mut self, frame: &mut Frame<'_>, area: Rect) {
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

    fn view_tab_line(&self, width: u16) -> Line<'static> {
        let counts = self.view_counts();
        let tab_widths = TuiView::ALL
            .into_iter()
            .enumerate()
            .map(|(index, view)| view_tab_text_width(view, counts[index]))
            .collect::<Vec<_>>();
        let content_width: u16 = tab_widths.iter().sum();
        let gaps = TuiView::ALL.len().saturating_sub(1) as u16;
        let gap_width = if gaps == 0 {
            0
        } else {
            ((width.saturating_sub(content_width)) / gaps).clamp(3, 8)
        };
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
        let gap_width = if gaps == 0 {
            0
        } else {
            ((area.width.saturating_sub(content_width)) / gaps).clamp(3, 8)
        };
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

    #[allow(dead_code)]
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
                .map(|doc| {
                    logs::list_item_for_log(
                        doc,
                        self.hierarchy.index.as_ref(),
                        &self.theme,
                        area.width.saturating_sub(4),
                    )
                })
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
                logs::detail_lines_for_log(
                    doc,
                    self.hierarchy.index.as_ref(),
                    self.log_events_for(doc.id()),
                    &self.theme,
                ),
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

    fn draw_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let content_area = if self.board_filters.is_active() && area.height >= 7 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(4)])
                .split(area);
            self.draw_board_filter_bar(frame, chunks[0]);
            chunks[1]
        } else {
            area
        };

        if !self.hierarchy.errors.is_empty() {
            self.draw_hierarchy_errors(frame, content_area);
            return;
        }

        if self.show_board_detail {
            let detail_height = (content_area.height / 3)
                .clamp(5, 12)
                .min(content_area.height.saturating_sub(4));
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(detail_height)])
                .split(content_area);
            if self.board_arrangement == BoardArrangement::Epic {
                self.draw_epic_board(frame, chunks[0]);
            } else {
                self.draw_state_tabs(frame, chunks[0]);
            }
            self.draw_detail(frame, chunks[1]);
        } else if self.board_arrangement == BoardArrangement::Epic {
            self.draw_epic_board(frame, content_area);
        } else {
            self.draw_state_tabs(frame, content_area);
        }
    }

    fn draw_hierarchy_errors(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = vec![
            Line::from(Span::styled(
                "Board hierarchy is invalid; task rows and graph-sensitive mutations are disabled.",
                self.theme.status_style(StatusTone::Error),
            )),
            Line::from(Span::styled(
                "Fix the referenced documents, then reload. Canonical shape: Epic → Task → Subtask.",
                self.theme.muted_style(),
            )),
            Line::from(""),
        ];
        for error in &self.hierarchy.errors {
            lines.push(Line::from(vec![
                Span::styled("• ", self.theme.status_style(StatusTone::Error)),
                Span::styled(error.clone(), self.theme.text_style()),
            ]));
        }
        let panel = Paragraph::new(lines)
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Hierarchy errors ({}) ",
                        self.hierarchy.errors.len()
                    ))
                    .border_style(self.theme.status_style(StatusTone::Error))
                    .style(self.theme.panel_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, area);
    }

    fn draw_board_filter_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let filter_bar = Paragraph::new(board_filter_bar_line(&self.board_filters, &self.theme))
            .style(self.theme.panel_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Active Board filters ")
                    .border_style(self.theme.status_style(StatusTone::Warning))
                    .style(self.theme.panel_style()),
            );
        frame.render_widget(filter_bar, area);
    }

    fn draw_epic_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let mode_line = Line::from(vec![
            Span::styled(" Board arrangement ", self.theme.muted_style()),
            Span::styled(" State ", self.theme.tab_style()),
            Span::raw(" "),
            Span::styled(" Epic ", self.theme.state_tab_selected_style()),
            Span::styled("  b switch ", self.theme.muted_style()),
        ]);
        frame.render_widget(Paragraph::new(mode_line), chunks[0]);
        self.draw_epic_board_list(frame, chunks[1]);
    }

    fn draw_epic_board_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.hits.push(HitRegion {
            rect: area,
            action: HitAction::SelectState(self.selected_state),
        });

        let entries = self.epic_board_entries();
        let count = entries.len();
        let content_width = area
            .width
            .saturating_sub(4)
            .saturating_sub(BOARD_LIST_HIGHLIGHT_SYMBOL_WIDTH) as usize;
        let preview_line_limit = inline_preview_line_limit_for_area(area);
        let items = if entries.is_empty() {
            let empty_text = if self.board_filters.is_active() {
                "No Epic Board rows match the active filters. Press F to clear filters."
            } else {
                "No epic groups are available. Press b for State Board."
            };
            vec![ListItem::new(Line::from(Span::styled(
                empty_text,
                self.theme.muted_style(),
            )))]
        } else {
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let context = self.relationship_context(entry.doc);
                    epic_list_item_for_entry(
                        entry,
                        &context,
                        &self.theme,
                        content_width,
                        preview_line_limit,
                        self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()),
                        index == self.selected_item,
                    )
                })
                .collect::<Vec<_>>()
        };

        let title = format!(
            " Epic Board · {} row{} ",
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
            let row_heights = entries
                .iter()
                .map(|entry| {
                    if self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()) {
                        let context = self.relationship_context(entry.doc);
                        1 + inline_preview_height_with_context(
                            entry.doc,
                            &context,
                            content_width,
                            preview_line_limit,
                        )
                    } else {
                        1
                    }
                })
                .collect::<Vec<_>>();
            self.register_board_row_hits(area, self.selected_state, state.offset(), &row_heights);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn draw_state_tabs(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        let subviews = board_subview_tabs(&self.states, &self.docs, &self.board_filters);
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
        let entries = self.state_board_entries(state_name);
        let row_count = entries.len();
        let state_task_count = self
            .docs
            .iter()
            .filter(|doc| is_board_visible_doc(doc))
            .filter(|doc| document_state_label(doc) == state_name.as_str())
            .filter(|doc| board_filters_match(doc, &self.board_filters))
            .count();
        let content_width = area.width.saturating_sub(4) as usize;
        let preview_line_limit = inline_preview_line_limit_for_area(area);
        let items = if entries.is_empty() {
            let empty_text = if self.board_filters.is_active() {
                "No hierarchy matches the active Board filters. Press F to clear filters."
            } else if state_task_count > 0 {
                "Tasks in this state are nested under parents in other state tabs."
            } else {
                "No active items in this state. Press a to quick-add here."
            };
            vec![ListItem::new(Line::from(Span::styled(
                empty_text,
                self.theme.muted_style(),
            )))]
        } else {
            let show_doc_type = entries.iter().any(|entry| entry.doc.doc_type() != "task");
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let context = self.relationship_context(entry.doc);
                    state_list_item_for_entry(
                        entry,
                        &context,
                        &self.theme,
                        content_width,
                        show_doc_type,
                        preview_line_limit,
                        self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()),
                        index == self.selected_item,
                    )
                })
                .collect::<Vec<_>>()
        };

        let title = if row_count == state_task_count {
            format!(
                " {} · {} row{} ",
                display_state_label(state_name),
                row_count,
                if row_count == 1 { "" } else { "s" }
            )
        } else {
            format!(
                " {} · {} task{} · {} visible row{} ",
                display_state_label(state_name),
                state_task_count,
                if state_task_count == 1 { "" } else { "s" },
                row_count,
                if row_count == 1 { "" } else { "s" }
            )
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
            .highlight_style(self.theme.board_selected_style())
            .highlight_symbol("› ");

        if row_count > 0 {
            let mut state = ListState::default();
            state.select(Some(self.selected_item.min(row_count - 1)));
            frame.render_stateful_widget(list, area, &mut state);
            let row_heights = entries
                .iter()
                .map(|entry| {
                    if self.expanded_board_doc_id.as_deref() == Some(entry.doc.id()) {
                        let context = self.relationship_context(entry.doc);
                        1 + inline_preview_height_with_context(
                            entry.doc,
                            &context,
                            content_width,
                            preview_line_limit,
                        )
                    } else {
                        1
                    }
                })
                .collect::<Vec<_>>();
            self.register_board_row_hits(area, state_index, state.offset(), &row_heights);
        } else {
            frame.render_widget(list, area);
        }
    }

    fn register_board_row_hits(
        &mut self,
        area: Rect,
        state_index: usize,
        first_visible_index: usize,
        row_heights: &[u16],
    ) {
        if area.width <= 2 || area.height <= 2 {
            return;
        }
        let left = area.x.saturating_add(1);
        let mut y = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let bottom = area.y.saturating_add(area.height).saturating_sub(1);
        for (index, height) in row_heights
            .iter()
            .copied()
            .enumerate()
            .skip(first_visible_index)
        {
            if y >= bottom {
                break;
            }
            self.hits.push(HitRegion {
                rect: Rect {
                    x: left,
                    y,
                    width,
                    height: height.min(bottom.saturating_sub(y)),
                },
                action: HitAction::SelectBoardItem(state_index, index),
            });
            y = y.saturating_add(height);
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
                detail_lines_for_doc_with_context(
                    doc,
                    &self.theme,
                    &relationship_context_for_doc_with_hierarchy(
                        doc,
                        &self.docs,
                        &self.logs,
                        self.hierarchy.index.as_ref(),
                    ),
                ),
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

    fn with_status(&self, base: String) -> String {
        if self.status.is_empty() {
            base
        } else {
            format!("{base} · {}", self.status)
        }
    }

    fn board_footer_text(&self) -> String {
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

    fn logs_footer_text(&self) -> String {
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

    fn draw_footer(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let footer_line = if let Some(input) = self.quick_add.as_ref() {
            Line::from(Span::styled(
                quick_add_status(input),
                self.theme.status_style(StatusTone::Warning),
            ))
        } else if self.log_search_input.is_some() {
            Line::from(Span::styled(
                self.status.clone(),
                self.theme.status_style(StatusTone::Warning),
            ))
        } else if self.validation_prompt.is_some() {
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

    fn footer_line_for_text(&self, hints: String) -> Line<'static> {
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

    fn show_validation_action_hint(&mut self, action: &str) {
        match action {
            "accept" | "approve" => self.start_validation_accept(),
            "rework" => self.start_validation_rework(),
            "apply" | "archive" => self.start_validation_apply_accepted(),
            "complete" => self.show_validation_complete_hint(),
            _ => self.status = format!("Unknown Validation action `{action}`."),
        }
    }

    fn help_lines(&self) -> Vec<Line<'static>> {
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

    fn draw_validation_prompt(&self, frame: &mut Frame<'_>, area: Rect) {
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

    fn draw_help(&self, frame: &mut Frame<'_>, area: Rect) {
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
    for doc in docs.iter().filter(|doc| is_board_visible_doc(doc)) {
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

fn is_decision_doc(doc: &Document) -> bool {
    doc.doc_type().eq_ignore_ascii_case("decision")
}

fn is_board_visible_doc(doc: &Document) -> bool {
    doc.location == DocumentLocation::Board && !is_decision_doc(doc)
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

#[cfg(test)]
fn validation_load_errors(
    docs: &[Document],
    logs: &[Document],
    configured_states: &[String],
) -> Vec<String> {
    let hierarchy = TuiHierarchySnapshot::from_documents(docs, logs);
    validation_load_errors_with_hierarchy(docs, logs, configured_states, &hierarchy)
}

fn validation_load_errors_with_hierarchy(
    docs: &[Document],
    logs: &[Document],
    configured_states: &[String],
    hierarchy: &TuiHierarchySnapshot,
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

    warnings.extend(hierarchy.errors.iter().cloned());

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
            let outcome = completion_outcome(doc);
            if !COMPLETION_OUTCOMES.contains(&outcome) {
                errors.push(format!(
                    "invalid completion.outcome `{outcome}`; expected completed or canceled"
                ));
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
    insert_optional_fingerprint(&mut files, theme::workspace_theme_path(workspace));
    insert_optional_fingerprint(&mut files, theme::workspace_config_path(workspace));
    if let Some(user_config_path) = theme::user_config_path_from_env() {
        insert_optional_fingerprint(&mut files, user_config_path);
    }
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
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    HierarchyIndex::from_workspace(workspace)?.validate_all_task_hierarchies()?;
    validate_state(workspace, state)?;

    let now = current_timestamp();
    let created = create_new_sequential_document(workspace, "task", |task_id| {
        format!(
            "---\nid: {task_id}\ntype: task\ntitle: {}\nstate: {}\ncreatedAt: {}\nupdatedAt: {}\n---\n\n",
            yaml_double_quote(title),
            yaml_double_quote(state),
            yaml_double_quote(&now),
            yaml_double_quote(&now)
        )
    })?;
    append_event(workspace, "task.created", &created.id, title)?;

    Ok(QuickAddOutcome {
        id: created.id,
        state: state.to_string(),
        title: title.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationActionOutcome {
    id: String,
    state: String,
}

fn apply_validation_accept(
    workspace: &Workspace,
    id: &str,
) -> Result<ValidationActionOutcome, CliError> {
    apply_validation_action(workspace, id, ValidationAction::Accept { note: None })
}

fn apply_validation_rework(
    workspace: &Workspace,
    id: &str,
    feedback: &str,
) -> Result<ValidationActionOutcome, CliError> {
    let feedback = feedback.trim();
    if feedback.is_empty() {
        return Err(CliError::usage("rework feedback must not be empty"));
    }
    apply_validation_action(
        workspace,
        id,
        ValidationAction::Rework {
            feedback: feedback.to_string(),
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationApplyOutcome {
    completed_ids: Vec<String>,
}

fn accepted_validation_candidates(docs: &[Document]) -> Vec<ValidationApplyCandidate> {
    docs.iter()
        .filter(|doc| doc.doc_type() == "task")
        .filter(|doc| document_state_label(doc) == "validation")
        .filter(|doc| normalized_accord_status(accord_status(doc).unwrap_or("")) == "accepted")
        .filter(|doc| review_status(doc).unwrap_or("") == "accepted")
        .map(|doc| ValidationApplyCandidate {
            id: doc.id().to_string(),
            title: doc.title().to_string(),
        })
        .collect()
}

fn apply_accepted_validation_tasks(
    workspace: &Workspace,
    candidates: &[ValidationApplyCandidate],
) -> Result<ValidationApplyOutcome, CliError> {
    if candidates.is_empty() {
        return Err(CliError::usage("no accepted Validation tasks to apply"));
    }
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    HierarchyIndex::from_workspace(workspace)?.validate_all_task_hierarchies()?;
    let mut completed_ids = Vec::new();
    for candidate in candidates {
        complete_validation_candidate(workspace, &candidate.id)?;
        completed_ids.push(candidate.id.clone());
    }
    Ok(ValidationApplyOutcome { completed_ids })
}

fn complete_validation_candidate(workspace: &Workspace, id: &str) -> Result<(), CliError> {
    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can be applied/logged in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    if document_state_label(&doc) != "validation"
        || normalized_accord_status(accord_status(&doc).unwrap_or("")) != "accepted"
        || review_status(&doc).unwrap_or("") != "accepted"
    {
        return Err(CliError::user(format!(
            "{} is not an accepted Validation candidate",
            doc.id()
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;
    let unresolved = unresolved_blockers(workspace, doc.field("blockers"))?;
    if !unresolved.is_empty() {
        return Err(CliError::user(format!(
            "Validation failed: {} has unresolved blockers: {}",
            doc.id(),
            unresolved.join(", ")
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let summary = format!("Applied accepted Validation sign-off for {}", doc.id());
    let mut updates = BTreeMap::new();
    updates.insert("completedAt".to_string(), now.clone());
    updates.insert("updatedAt".to_string(), now);
    let patched = patch_frontmatter_content(
        &content,
        &updates,
        &[
            "state",
            "completionSummary",
            "completionValidation",
            "completionReviewer",
            "filesChanged",
        ],
    )?;
    let patched = patch_completion_content(
        &patched,
        &summary,
        None,
        &[],
        Some("Accepted by Validation apply-accepted workflow"),
        Some("tui"),
    )?;
    let log_path = workspace.logs_dir.join(file_name_for_path(&doc.path)?);
    if log_path.exists() {
        return Err(CliError::user(format!(
            "Validation failed: log document already exists: {}",
            display_path(&log_path)
        )));
    }
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&log_path, &patched)?;
    fs::remove_file(&doc.path).map_err(|error| {
        CliError::user(format!(
            "Write failure: could not remove active document {} after writing log {}: {error}",
            display_path(&doc.path),
            display_path(&log_path)
        ))
    })?;
    append_event(workspace, "task.completed", doc.id(), &summary)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValidationAction {
    Accept { note: Option<String> },
    Rework { feedback: String },
}

fn apply_validation_action(
    workspace: &Workspace,
    id: &str,
    action: ValidationAction,
) -> Result<ValidationActionOutcome, CliError> {
    let _hierarchy_lock = HierarchyLock::acquire(workspace)?;
    HierarchyIndex::from_workspace(workspace)?.validate_all_task_hierarchies()?;
    let doc = find_board_document(workspace, id)?
        .ok_or_else(|| CliError::user(format!("active task not found: {id}")))?;
    if doc.doc_type() != "task" {
        return Err(CliError::user(format!(
            "Validation failed: only task documents can use Validation actions in v0: {} is type {}",
            doc.id(),
            doc.doc_type()
        )));
    }
    if document_state_label(&doc) != "validation" {
        return Err(CliError::user(format!(
            "{} is in `{}`; Validation actions require state `validation`",
            doc.id(),
            document_state_label(&doc)
        )));
    }
    validate_task_document_for_mutation(workspace, &doc)?;

    let previous_status = accord_status(&doc).unwrap_or("missing").to_string();
    if normalized_accord_status(&previous_status) != "delivered" {
        return Err(CliError::user(format!(
            "{} has accord.status={previous_status}; Validation sign-off actions require delivered",
            doc.id()
        )));
    }

    let (content, signature) = read_file_snapshot(&doc.path)?;
    let now = current_timestamp();
    let mut accord = AccordRecord::from_document(&doc, &now);
    let (
        accord_action,
        status,
        note,
        review_status_value,
        event_name,
        event_summary,
        next_state,
        append_feedback,
    ) = match action {
        ValidationAction::Accept { note } => (
            "accept",
            "accepted",
            note,
            "accepted",
            "validation.accepted",
            format!("Accepted sign-off for {}", doc.id()),
            "validation".to_string(),
            false,
        ),
        ValidationAction::Rework { feedback } => (
            "rework",
            "rework",
            Some(feedback),
            "changes-requested",
            "validation.rework",
            format!("Requested rework for {}", doc.id()),
            "in-progress".to_string(),
            true,
        ),
    };
    let options = AccordOptions {
        id: doc.id().to_string(),
        note: note.clone(),
        reviewer: Some("tui".to_string()),
        ..AccordOptions::default()
    };
    apply_accord_action(&mut accord, accord_action, status, &options);
    let patched = patch_accord_content(&content, &accord)?;
    validate_state(workspace, &next_state)?;
    let mut updates = BTreeMap::new();
    updates.insert("updatedAt".to_string(), now.clone());
    updates.insert("state".to_string(), next_state.clone());
    updates.insert("review.status".to_string(), review_status_value.to_string());
    updates.insert("review.decidedAt".to_string(), now.clone());
    updates.insert("review.reviewer".to_string(), "tui".to_string());
    if let Some(note) = note.as_deref().filter(|value| !value.trim().is_empty()) {
        updates.insert("review.note".to_string(), note.to_string());
    }
    let patched = patch_frontmatter_content(&patched, &updates, &[])?;
    let patched = if append_feedback {
        append_feedback_entry(&patched, &now, "tui", note.as_deref().unwrap_or(""))?
    } else {
        patched
    };
    ensure_file_unchanged(&doc.path, &signature)?;
    write_atomic(&doc.path, &patched)?;
    append_event(workspace, event_name, doc.id(), &event_summary)?;

    Ok(ValidationActionOutcome {
        id: doc.id().to_string(),
        state: next_state,
    })
}

fn append_feedback_entry(
    content: &str,
    timestamp: &str,
    source: &str,
    feedback: &str,
) -> Result<String, CliError> {
    let (frontmatter, body) = split_frontmatter(content).map_err(CliError::user)?;
    let mut body = body.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    if !body.contains("\n## Feedback\n") && !body.trim_start().starts_with("## Feedback\n") {
        if !body.trim().is_empty() {
            body.push('\n');
        }
        body.push_str("## Feedback\n\n");
    } else if !body.ends_with("\n\n") {
        body.push('\n');
    }
    body.push_str(&format!(
        "- {timestamp} ({source}): {}\n",
        feedback.replace('\n', " ").trim()
    ));
    Ok(format!("---\n{}---\n{}", frontmatter, body))
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardSubviewTab {
    state: String,
    count: usize,
}

fn board_subview_tabs(
    states: &[String],
    docs: &[Document],
    filters: &BoardFilters,
) -> Vec<BoardSubviewTab> {
    states
        .iter()
        .map(|state| BoardSubviewTab {
            state: state.clone(),
            count: docs
                .iter()
                .filter(|doc| is_board_visible_doc(doc))
                .filter(|doc| document_state_label(doc) == state.as_str())
                .filter(|doc| board_filters_match(doc, filters))
                .count(),
        })
        .collect()
}

fn state_tab_title(state: &str, count: usize) -> String {
    format!(" {} {} ", display_state_label(state), count)
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

fn display_state_label(state: &str) -> String {
    state
        .trim()
        .replace('-', " ")
        .replace('_', " ")
        .to_uppercase()
}

fn validation_prompt_lines(prompt: &ValidationPrompt, theme: &TuiTheme) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Target: ", theme.label_style()),
            Span::styled(
                format!("{} — {}", prompt.id(), prompt.title()),
                theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];
    match prompt {
        ValidationPrompt::Accept { .. } => {
            lines.push(Line::from(Span::styled(
                "Accept this delivery as human sign-off?",
                theme.text_style(),
            )));
            lines.push(Line::from(Span::styled(
                "Enter/y accepts; Esc/n cancels. Completion/logging remains a separate later action.",
                theme.muted_style(),
            )));
        }
        ValidationPrompt::Rework { feedback, .. } => {
            lines.push(Line::from(Span::styled(
                "Feedback to append durably:",
                theme.label_style(),
            )));
            lines.push(Line::from(Span::styled(
                if feedback.is_empty() {
                    "<type feedback>".to_string()
                } else {
                    feedback.clone()
                },
                if feedback.is_empty() {
                    theme.muted_style()
                } else {
                    theme.text_style()
                },
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter requests rework and moves the item back to in-progress; Esc cancels without writing.",
                theme.muted_style(),
            )));
        }
        ValidationPrompt::ApplyAccepted { candidates } => {
            lines.push(Line::from(Span::styled(
                "These accepted Validation tasks will be completed and moved to logs:",
                theme.text_style(),
            )));
            for candidate in candidates.iter().take(8) {
                lines.push(Line::from(vec![
                    Span::styled("• ", theme.muted_style()),
                    Span::styled(candidate.id.clone(), theme.label_style()),
                    Span::styled(format!(" — {}", candidate.title), theme.text_style()),
                ]));
            }
            if candidates.len() > 8 {
                lines.push(Line::from(Span::styled(
                    format!("… and {} more", candidates.len() - 8),
                    theme.muted_style(),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter/y applies; Esc/n cancels without changing files. Delivered or rework items are excluded.",
                theme.muted_style(),
            )));
        }
    }
    lines
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BoardRelationshipHints {
    active_children: usize,
    completed_children: usize,
}

impl BoardRelationshipHints {
    fn total_children(self) -> usize {
        self.active_children + self.completed_children
    }

    fn has_children(self) -> bool {
        self.total_children() > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardRelatedChild {
    id: String,
    title: String,
    state: String,
    completed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BoardRelationshipContext {
    task_role: Option<TaskRole>,
    parent_relationship: Option<ParentRelationship>,
    parent_id: Option<String>,
    parent_title: Option<String>,
    parent_missing: bool,
    hierarchy_error: Option<String>,
    active_children: Vec<BoardRelatedChild>,
    completed_children: Vec<BoardRelatedChild>,
}

impl BoardRelationshipContext {
    fn hints(&self) -> BoardRelationshipHints {
        BoardRelationshipHints {
            active_children: self.active_children.len(),
            completed_children: self.completed_children.len(),
        }
    }

    fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }

    fn has_children(&self) -> bool {
        self.hints().has_children()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateBoardEntryRole {
    Root,
    Child,
}

#[derive(Debug, Clone, Copy)]
struct StateBoardEntry<'a> {
    doc: &'a Document,
    role: StateBoardEntryRole,
    task_role: Option<TaskRole>,
    depth: usize,
    active_descendants: usize,
    completed_descendants: usize,
    has_active_children: bool,
    expanded: bool,
    last_sibling: bool,
}

#[cfg(test)]
fn state_board_entries<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    state: &str,
    filters: &BoardFilters,
    expanded_ids: &BTreeSet<String>,
) -> Vec<StateBoardEntry<'a>> {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    let Some(hierarchy) = snapshot.valid_index() else {
        return Vec::new();
    };
    state_board_entries_with_hierarchy(
        active_docs,
        completed_logs,
        state,
        filters,
        expanded_ids,
        hierarchy,
    )
}

fn state_board_entries_with_hierarchy<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    state: &str,
    filters: &BoardFilters,
    expanded_ids: &BTreeSet<String>,
    hierarchy: &HierarchyIndex,
) -> Vec<StateBoardEntry<'a>> {
    let mut entries = Vec::new();
    for root in active_docs.iter().filter(|doc| {
        is_board_visible_doc(doc) && is_state_board_root(doc, active_docs, completed_logs)
    }) {
        let mut visited = BTreeSet::from([root.id().to_string()]);
        let root_matches_state = document_state_label(root) == state;
        let descendant_matches_state = is_task_doc(root)
            && task_subtree_matches_filters(
                root.id(),
                active_docs,
                completed_logs,
                state,
                filters,
                &mut visited,
            );
        // A state pane must expose every task counted by its tab, even when that task
        // sits below an ancestor in another workflow state. Keep the ancestor path as
        // context and automatically open the matching branch.
        let subtree_matches =
            (root_matches_state && board_filters_match(root, filters)) || descendant_matches_state;
        if !subtree_matches {
            continue;
        }
        let (active_descendants, completed_descendants) = if is_task_doc(root) {
            count_task_descendants(
                root.id(),
                active_docs,
                completed_logs,
                &mut BTreeSet::from([root.id().to_string()]),
            )
        } else {
            (0, 0)
        };
        let has_active_children = active_descendants > 0;
        // Auto-expand only an ancestor whose own state differs from this pane. A
        // same-state hierarchy remains collapsed by default and user-controlled.
        let expanded =
            expanded_ids.contains(root.id()) || (descendant_matches_state && !root_matches_state);
        let role = if normalized_parent_id(root).is_some_and(|parent_id| {
            completed_logs
                .iter()
                .any(|parent| parent.id() == parent_id && is_task_doc(parent))
        }) {
            StateBoardEntryRole::Child
        } else {
            StateBoardEntryRole::Root
        };
        entries.push(StateBoardEntry {
            doc: root,
            role,
            task_role: hierarchy.task_role(root).ok().flatten(),
            depth: 0,
            active_descendants,
            completed_descendants,
            has_active_children,
            expanded,
            last_sibling: false,
        });
        if is_task_doc(root) {
            collect_visible_state_descendants(
                root.id(),
                1,
                active_docs,
                completed_logs,
                state,
                filters,
                expanded_ids,
                expanded || filters.is_active(),
                hierarchy,
                &mut BTreeSet::from([root.id().to_string()]),
                &mut entries,
            );
        }
    }
    mark_state_board_last_siblings(&mut entries);
    entries
}

fn mark_state_board_last_siblings(entries: &mut [StateBoardEntry<'_>]) {
    for index in 0..entries.len() {
        let depth = entries[index].depth;
        if depth == 0 {
            continue;
        }
        entries[index].last_sibling = !entries[index + 1..]
            .iter()
            .take_while(|candidate| candidate.depth >= depth)
            .any(|candidate| candidate.depth == depth);
    }
}

fn collect_visible_state_descendants<'a>(
    parent_id: &str,
    depth: usize,
    active_docs: &'a [Document],
    completed_logs: &[Document],
    target_state: &str,
    filters: &BoardFilters,
    expanded_ids: &BTreeSet<String>,
    parent_open: bool,
    hierarchy: &HierarchyIndex,
    visited: &mut BTreeSet<String>,
    entries: &mut Vec<StateBoardEntry<'a>>,
) {
    for child in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(child.id().to_string()) {
            continue;
        }
        let mut match_visited = visited.clone();
        let subtree_matches = !filters.is_active()
            || (document_state_label(child) == target_state && board_filters_match(child, filters))
            || task_subtree_matches_filters(
                child.id(),
                active_docs,
                completed_logs,
                target_state,
                filters,
                &mut match_visited,
            );
        if !parent_open || !subtree_matches {
            continue;
        }
        let (active_descendants, completed_descendants) = count_task_descendants(
            child.id(),
            active_docs,
            completed_logs,
            &mut BTreeSet::from([child.id().to_string()]),
        );
        let has_active_children = active_descendants > 0;
        let descendant_matches_state = task_subtree_matches_filters(
            child.id(),
            active_docs,
            completed_logs,
            target_state,
            filters,
            &mut BTreeSet::from([child.id().to_string()]),
        );
        let child_matches_state = document_state_label(child) == target_state;
        let expanded =
            expanded_ids.contains(child.id()) || (descendant_matches_state && !child_matches_state);
        let task_role = hierarchy.task_role(child).ok().flatten();
        entries.push(StateBoardEntry {
            doc: child,
            role: StateBoardEntryRole::Child,
            task_role,
            depth,
            active_descendants,
            completed_descendants,
            has_active_children,
            expanded,
            last_sibling: false,
        });
        collect_visible_state_descendants(
            child.id(),
            depth + 1,
            active_docs,
            completed_logs,
            target_state,
            filters,
            expanded_ids,
            expanded || filters.is_active(),
            hierarchy,
            visited,
            entries,
        );
    }

    for completed in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(completed.id().to_string()) && parent_open {
            collect_visible_state_descendants(
                completed.id(),
                depth + 1,
                active_docs,
                completed_logs,
                target_state,
                filters,
                expanded_ids,
                true,
                hierarchy,
                visited,
                entries,
            );
        }
    }
}

fn is_state_board_root(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
) -> bool {
    if !is_task_doc(doc) {
        return true;
    }
    let mut current = doc;
    let mut visited = vec![doc.id().to_string()];
    let mut saw_active_ancestor = false;
    loop {
        let Some(parent_id) = normalized_parent_id(current) else {
            return !saw_active_ancestor;
        };
        if let Some(cycle_start) = visited.iter().position(|id| id == &parent_id) {
            let cycle_root = visited[cycle_start..]
                .iter()
                .filter(|id| {
                    active_docs
                        .iter()
                        .any(|candidate| candidate.id() == id.as_str())
                })
                .min();
            return cycle_root.is_some_and(|id| id == doc.id());
        }
        visited.push(parent_id.clone());
        if let Some(parent) = active_docs
            .iter()
            .find(|candidate| candidate.id() == parent_id && is_task_doc(candidate))
        {
            saw_active_ancestor = true;
            current = parent;
            continue;
        }
        if let Some(parent) = completed_logs
            .iter()
            .find(|candidate| candidate.id() == parent_id && is_task_doc(candidate))
        {
            current = parent;
            continue;
        }
        return !saw_active_ancestor;
    }
}

fn task_subtree_matches_filters(
    parent_id: &str,
    active_docs: &[Document],
    completed_logs: &[Document],
    target_state: &str,
    filters: &BoardFilters,
    visited: &mut BTreeSet<String>,
) -> bool {
    let active_match = active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
        .any(|child| {
            visited.insert(child.id().to_string())
                && ((document_state_label(child) == target_state
                    && board_filters_match(child, filters))
                    || task_subtree_matches_filters(
                        child.id(),
                        active_docs,
                        completed_logs,
                        target_state,
                        filters,
                        visited,
                    ))
        });
    active_match
        || completed_logs
            .iter()
            .filter(|doc| is_task_doc(doc))
            .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
            .any(|completed| {
                visited.insert(completed.id().to_string())
                    && task_subtree_matches_filters(
                        completed.id(),
                        active_docs,
                        completed_logs,
                        target_state,
                        filters,
                        visited,
                    )
            })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EpicBoardEntryRole {
    Epic,
    Task,
    Subtask,
}

#[derive(Debug, Clone, Copy)]
struct EpicBoardEntry<'a> {
    doc: &'a Document,
    role: EpicBoardEntryRole,
    depth: usize,
    active_descendants: usize,
    completed_descendants: usize,
}

#[cfg(test)]
fn epic_board_entries<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    filters: &BoardFilters,
) -> Vec<EpicBoardEntry<'a>> {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    let Some(hierarchy) = snapshot.valid_index() else {
        return Vec::new();
    };
    epic_board_entries_with_hierarchy(active_docs, completed_logs, filters, hierarchy)
}

fn epic_board_entries_with_hierarchy<'a>(
    active_docs: &'a [Document],
    completed_logs: &[Document],
    filters: &BoardFilters,
    hierarchy: &HierarchyIndex,
) -> Vec<EpicBoardEntry<'a>> {
    let mut entries = Vec::new();

    for epic in active_docs.iter().filter(|doc| {
        is_board_visible_doc(doc) && matches!(hierarchy.task_role(doc), Ok(Some(TaskRole::Epic)))
    }) {
        let mut visited = BTreeSet::from([epic.id().to_string()]);
        let mut descendants = Vec::new();
        collect_visible_epic_descendants(
            epic.id(),
            1,
            active_docs,
            completed_logs,
            filters,
            hierarchy,
            &mut visited,
            &mut descendants,
        );
        if !board_filters_match(epic, filters) && descendants.is_empty() {
            continue;
        }
        let (active_descendants, completed_descendants) = count_task_descendants(
            epic.id(),
            active_docs,
            completed_logs,
            &mut BTreeSet::from([epic.id().to_string()]),
        );
        entries.push(EpicBoardEntry {
            doc: epic,
            role: EpicBoardEntryRole::Epic,
            depth: 0,
            active_descendants,
            completed_descendants,
        });
        entries.extend(
            descendants
                .into_iter()
                .map(|(doc, depth, role)| EpicBoardEntry {
                    doc,
                    role: match role {
                        TaskRole::Task => EpicBoardEntryRole::Task,
                        TaskRole::Subtask => EpicBoardEntryRole::Subtask,
                        TaskRole::Epic => EpicBoardEntryRole::Epic,
                    },
                    depth,
                    active_descendants: 0,
                    completed_descendants: 0,
                }),
        );
    }
    entries
}

fn collect_visible_epic_descendants<'a>(
    parent_id: &str,
    depth: usize,
    active_docs: &'a [Document],
    completed_logs: &[Document],
    filters: &BoardFilters,
    hierarchy: &HierarchyIndex,
    visited: &mut BTreeSet<String>,
    entries: &mut Vec<(&'a Document, usize, TaskRole)>,
) -> bool {
    let mut any_visible = false;

    for child in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(child.id().to_string()) {
            continue;
        }
        let Ok(Some(role @ (TaskRole::Task | TaskRole::Subtask))) = hierarchy.task_role(child)
        else {
            continue;
        };
        let insert_at = entries.len();
        entries.push((child, depth, role));
        let descendant_visible = collect_visible_epic_descendants(
            child.id(),
            depth + 1,
            active_docs,
            completed_logs,
            filters,
            hierarchy,
            visited,
            entries,
        );
        if board_filters_match(child, filters) || descendant_visible {
            any_visible = true;
        } else {
            entries.remove(insert_at);
        }
    }

    for completed in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if !visited.insert(completed.id().to_string()) {
            continue;
        }
        if collect_visible_epic_descendants(
            completed.id(),
            depth + 1,
            active_docs,
            completed_logs,
            filters,
            hierarchy,
            visited,
            entries,
        ) {
            any_visible = true;
        }
    }

    any_visible
}

fn count_task_descendants(
    parent_id: &str,
    active_docs: &[Document],
    completed_logs: &[Document],
    visited: &mut BTreeSet<String>,
) -> (usize, usize) {
    let mut active = 0;
    let mut completed = 0;

    for doc in active_docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc) && is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(doc.id().to_string()) {
            active += 1;
            let nested = count_task_descendants(doc.id(), active_docs, completed_logs, visited);
            active += nested.0;
            completed += nested.1;
        }
    }

    for doc in completed_logs
        .iter()
        .filter(|doc| is_task_doc(doc))
        .filter(|doc| normalized_parent_id(doc).as_deref() == Some(parent_id))
    {
        if visited.insert(doc.id().to_string()) {
            if completion_outcome(doc) == COMPLETION_OUTCOME_COMPLETED {
                completed += 1;
            }
            let nested = count_task_descendants(doc.id(), active_docs, completed_logs, visited);
            active += nested.0;
            completed += nested.1;
        }
    }

    (active, completed)
}

fn hierarchy_index_for(
    active_docs: &[Document],
    completed_logs: &[Document],
) -> Result<HierarchyIndex, CliError> {
    HierarchyIndex::from_documents(
        active_docs
            .iter()
            .chain(completed_logs.iter())
            .cloned()
            .collect(),
    )
}

#[cfg(test)]
fn relationship_context_for_doc(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
) -> BoardRelationshipContext {
    let snapshot = TuiHierarchySnapshot::from_documents(active_docs, completed_logs);
    relationship_context_for_doc_with_hierarchy(
        doc,
        active_docs,
        completed_logs,
        snapshot.index.as_ref(),
    )
}

fn relationship_context_for_doc_with_hierarchy(
    doc: &Document,
    active_docs: &[Document],
    completed_logs: &[Document],
    hierarchy: Option<&HierarchyIndex>,
) -> BoardRelationshipContext {
    let (task_role, parent_relationship, hierarchy_error) = match hierarchy {
        Some(hierarchy) => {
            let task_role = hierarchy.task_role(doc);
            let relationship = hierarchy.relationship(doc);
            let validation = if doc.doc_type() == "task" {
                hierarchy.validate_task_hierarchy(doc).map(|_| ())
            } else {
                Ok(())
            };
            match (task_role, relationship, validation) {
                (Ok(role), Ok(relationship), Ok(())) => (role, relationship, None),
                (role, relationship, Err(error)) => (
                    role.ok().flatten(),
                    relationship.ok().flatten(),
                    Some(error.message.clone()),
                ),
                (Err(error), _, _) | (_, Err(error), _) => {
                    (None, None, Some(error.message.clone()))
                }
            }
        }
        None => (
            None,
            None,
            Some("Validation failed: hierarchy snapshot is unavailable".to_string()),
        ),
    };
    let parent_id = normalized_parent_id(doc).filter(|parent_id| parent_id.as_str() != doc.id());
    let parent_doc = parent_id.as_deref().and_then(|parent_id| {
        active_docs
            .iter()
            .chain(completed_logs.iter())
            .find(|candidate| candidate.id() == parent_id)
    });
    let parent_title = parent_doc.map(|parent| parent.title().to_string());
    let parent_missing = parent_id.is_some() && parent_doc.is_none();
    let active_children = if is_task_doc(doc) {
        active_docs
            .iter()
            .filter(|child| is_task_doc(child))
            .filter(|child| normalized_parent_id(child).as_deref() == Some(doc.id()))
            .filter(|child| child.id() != doc.id())
            .map(|child| related_child_summary(child, false))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let completed_children = if is_task_doc(doc) {
        completed_logs
            .iter()
            .filter(|child| is_task_doc(child))
            .filter(|child| completion_outcome(child) == COMPLETION_OUTCOME_COMPLETED)
            .filter(|child| normalized_parent_id(child).as_deref() == Some(doc.id()))
            .filter(|child| child.id() != doc.id())
            .map(|child| related_child_summary(child, true))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    BoardRelationshipContext {
        task_role,
        parent_relationship,
        parent_id,
        parent_title,
        parent_missing,
        hierarchy_error,
        active_children,
        completed_children,
    }
}

fn related_child_summary(doc: &Document, completed: bool) -> BoardRelatedChild {
    BoardRelatedChild {
        id: doc.id().to_string(),
        title: doc.title().to_string(),
        state: if completed {
            "log".to_string()
        } else {
            document_state_label(doc)
        },
        completed,
    }
}

fn normalized_parent_id(doc: &Document) -> Option<String> {
    doc.field("parentId")
        .map(str::trim)
        .filter(|parent_id| !parent_id.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
fn board_item_lines_for_doc(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    board_item_lines_for_doc_with_context(
        doc,
        theme,
        content_width,
        show_doc_type,
        &BoardRelationshipContext::default(),
        expanded,
        selected,
    )
}

#[cfg(test)]
fn board_item_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    relationship_context: &BoardRelationshipContext,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    board_item_lines_for_doc_with_context_and_limit(
        doc,
        theme,
        content_width,
        show_doc_type,
        relationship_context,
        INLINE_PREVIEW_MAX_LINES,
        expanded,
        selected,
    )
}

#[cfg(test)]
fn board_item_lines_for_doc_with_context_and_limit(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    relationship_context: &BoardRelationshipContext,
    preview_line_limit: usize,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    // Board rows are intentionally sparse. The Board is for scanning and choosing work;
    // details belong in expanded rows and the detail pane. Relationship context is shown
    // as nesting/expanded-row content, not noisy parent-id chips.
    let chips = board_scan_chips(doc, relationship_context.task_role, theme);

    let mut lines = vec![board_row_line(
        doc,
        theme,
        content_width,
        show_doc_type,
        chips,
        doc.id().to_string(),
        0,
        selected,
    )];
    if expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

fn board_scan_chips(
    doc: &Document,
    task_role: Option<TaskRole>,
    theme: &TuiTheme,
) -> Vec<(String, Style)> {
    let priority = doc.field("priority").unwrap_or("-");
    let mut chips = Vec::new();
    if let Some(priority_chip) = priority_chip(priority, theme) {
        chips.push((priority_chip, theme.priority_chip_style(priority)));
    }
    if task_role == Some(TaskRole::Epic) {
        chips.push((
            chip_text("EPIC", theme),
            theme.progress_chip_style(StatusTone::Accent),
        ));
    }
    for (kind_chip, tone) in work_type_tag_chips(doc, theme) {
        chips.push((kind_chip, theme.progress_chip_style(tone)));
    }
    if let Some((visual_chip, tone)) = validation_visual_chip(doc, theme) {
        chips.push((visual_chip, theme.progress_chip_style(tone)));
    }
    for (tag_chip, tone) in configured_tag_chips(doc, theme) {
        chips.push((tag_chip, theme.progress_chip_style(tone)));
    }
    if let Some(accord) =
        accord_status(doc).filter(|status| board_should_surface_accord_status(doc, status, theme))
    {
        chips.push((status_chip(accord, theme), theme.accord_chip_style(accord)));
    }
    if let Some(review) =
        review_status(doc).filter(|status| board_should_surface_review_status(status, theme))
    {
        chips.push((status_chip(review, theme), theme.review_chip_style(review)));
    }
    if let Some((completed, total)) = subtask_progress(doc).filter(|(completed, total)| {
        !theme.badge_disabled("subtasks")
            && !theme.badge_disabled("subtask-progress")
            && (*completed > 0 || completed == total)
    }) {
        let tone = if completed == total {
            StatusTone::Success
        } else {
            StatusTone::Warning
        };
        chips.push((
            chip_text(&format!("{completed}/{total}"), theme),
            theme.progress_chip_style(tone),
        ));
    }
    chips
}

fn state_list_item_for_entry(
    entry: &StateBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    preview_line_limit: usize,
    preview_expanded: bool,
    selected: bool,
) -> ListItem<'static> {
    ListItem::new(state_lines_for_entry(
        entry,
        relationship_context,
        theme,
        content_width,
        show_doc_type,
        preview_line_limit,
        preview_expanded,
        selected,
    ))
}

fn state_lines_for_entry(
    entry: &StateBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    preview_line_limit: usize,
    preview_expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    let doc = entry.doc;
    debug_assert!(entry.task_role.is_none() || is_task_doc(doc));
    let meta_width = content_width.saturating_div(2).min(32);
    let right_meta = if entry.active_descendants + entry.completed_descendants > 0 {
        truncate(
            &descendant_rollup(entry.active_descendants, entry.completed_descendants),
            meta_width,
        )
    } else {
        String::new()
    };
    let mut chips = vec![(state_hierarchy_prefix(entry), theme.muted_style())];
    chips.push(board_id_chip(doc, theme));
    let state = compact_epic_state(&document_state_label(doc));
    chips.push((
        chip_text(&format!("{state:<4}"), theme),
        theme.state_chip_style(&document_state_label(doc)),
    ));
    match entry.role {
        StateBoardEntryRole::Root => chips.extend(board_scan_chips(doc, entry.task_role, theme)),
        StateBoardEntryRole::Child if entry.depth == 0 => {
            chips.extend(board_scan_chips(doc, entry.task_role, theme));
        }
        StateBoardEntryRole::Child => {}
    }
    let mut lines = vec![board_row_line(
        doc,
        theme,
        content_width,
        show_doc_type && entry.depth == 0,
        chips,
        right_meta,
        0,
        selected,
    )];
    if preview_expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

fn state_hierarchy_prefix(entry: &StateBoardEntry<'_>) -> String {
    if entry.depth == 0 {
        return if entry.has_active_children {
            if entry.expanded {
                "▾"
            } else {
                "▸"
            }
        } else {
            " "
        }
        .to_string();
    }
    let branch = if entry.last_sibling { "└" } else { "├" };
    let disclosure = if entry.has_active_children {
        if entry.expanded {
            "▾"
        } else {
            "▸"
        }
    } else {
        "─"
    };
    format!(
        "{}{}{}",
        "│  ".repeat(entry.depth.saturating_sub(1)),
        branch,
        disclosure
    )
}

fn epic_list_item_for_entry(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    preview_line_limit: usize,
    expanded: bool,
    selected: bool,
) -> ListItem<'static> {
    ListItem::new(epic_lines_for_entry(
        entry,
        relationship_context,
        theme,
        content_width,
        preview_line_limit,
        expanded,
        selected,
    ))
}

fn epic_lines_for_entry(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    preview_line_limit: usize,
    expanded: bool,
    selected: bool,
) -> Vec<Line<'static>> {
    let doc = entry.doc;
    let mut lines = vec![epic_row_line(
        entry,
        relationship_context,
        theme,
        content_width,
        selected,
    )];
    if expanded {
        lines.extend(inline_preview_lines_for_doc_with_context(
            doc,
            theme,
            relationship_context,
            content_width,
            preview_line_limit,
        ));
    }
    lines
}

const EPIC_META_COLUMN_WIDTH: usize = 32;
const EPIC_MIN_TITLE_WIDTH: usize = 8;

fn epic_row_line(
    entry: &EpicBoardEntry<'_>,
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
    content_width: usize,
    selected: bool,
) -> Line<'static> {
    let doc = entry.doc;
    let indent = "  ".repeat(entry.depth);
    let title_style = if selected {
        theme.board_selected_title_style()
    } else {
        theme.text_style().add_modifier(Modifier::BOLD)
    };
    let mut prefix = vec![Span::styled(indent.clone(), theme.muted_style())];
    let mut prefix_width = match entry.role {
        EpicBoardEntryRole::Epic => {
            prefix.push(Span::styled(
                chip_text("EPIC", theme),
                theme.progress_chip_style(StatusTone::Accent),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent) + text_width(&chip_text("EPIC", theme)) + 1
        }
        EpicBoardEntryRole::Task => {
            let state = compact_epic_state(&document_state_label(doc));
            prefix.push(Span::styled(
                chip_text(&format!("{state:<4}"), theme),
                theme.state_chip_style(&document_state_label(doc)),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent) + text_width(&chip_text(&format!("{state:<4}"), theme)) + 1
        }
        EpicBoardEntryRole::Subtask => {
            let state = compact_epic_state(&document_state_label(doc));
            prefix.push(Span::styled(
                chip_text("SUB", theme),
                theme.progress_chip_style(StatusTone::Accent),
            ));
            prefix.push(Span::raw(" "));
            prefix.push(Span::styled(
                chip_text(&format!("{state:<4}"), theme),
                theme.state_chip_style(&document_state_label(doc)),
            ));
            prefix.push(Span::raw(" "));
            text_width(&indent)
                + text_width(&chip_text("SUB", theme))
                + text_width(&chip_text(&format!("{state:<4}"), theme))
                + 2
        }
    };
    let (id_chip, id_style) = board_id_chip(doc, theme);
    prefix_width += text_width(&id_chip) + 1;
    prefix.push(Span::styled(id_chip, id_style));
    prefix.push(Span::raw(" "));

    let meta_column_width = EPIC_META_COLUMN_WIDTH
        .min(content_width.saturating_sub(prefix_width + EPIC_MIN_TITLE_WIDTH + 1));
    let show_meta = meta_column_width >= 7;
    let meta_column_width = if show_meta { meta_column_width } else { 0 };
    let raw_meta = match entry.role {
        EpicBoardEntryRole::Epic => {
            descendant_rollup(entry.active_descendants, entry.completed_descendants)
        }
        EpicBoardEntryRole::Task | EpicBoardEntryRole::Subtask => compact_relationship_meta(
            relationship_context.parent_id.as_deref().unwrap_or("?"),
            doc.id(),
            meta_column_width,
        ),
    };
    let meta = if show_meta {
        truncate(&raw_meta, meta_column_width)
    } else {
        String::new()
    };
    let title_width =
        content_width.saturating_sub(prefix_width + meta_column_width + usize::from(show_meta));
    let title = truncate(doc.title(), title_width);
    let spacer_width = content_width
        .saturating_sub(prefix_width + text_width(&title) + meta_column_width)
        .max(usize::from(show_meta));

    prefix.push(Span::styled(title, title_style));
    prefix.push(Span::raw(" ".repeat(spacer_width)));
    if show_meta {
        prefix.push(Span::styled(
            format!("{meta:<meta_column_width$}"),
            theme.muted_style(),
        ));
    }
    Line::from(prefix)
}

fn board_id_chip(doc: &Document, theme: &TuiTheme) -> (String, Style) {
    let id = doc
        .id()
        .strip_prefix("task-")
        .map(|suffix| format!("#{suffix}"))
        .unwrap_or_else(|| doc.id().to_string());
    (chip_text(&id, theme), theme.muted_style())
}

fn compact_epic_state(state: &str) -> String {
    match normalize_filter_value(state).as_str() {
        "todo" => "TODO".to_string(),
        "in-progress" | "inprogress" | "doing" => "WIP".to_string(),
        "validation" | "review" => "VAL".to_string(),
        "blocked" => "BLK".to_string(),
        "ready" => "RDY".to_string(),
        "backlog" => "BACK".to_string(),
        "unfiled" => "UNFD".to_string(),
        "" => "UNFD".to_string(),
        other => truncate(&other.to_ascii_uppercase().replace('_', "-"), 4),
    }
}

fn compact_relationship_meta(parent_id: &str, child_id: &str, width: usize) -> String {
    let full = format!("{parent_id} → {child_id}");
    if text_width(&full) <= width {
        return full;
    }
    if width < 7 {
        return String::new();
    }
    let id_width = width.saturating_sub(3);
    let parent_width = id_width / 2;
    let child_width = id_width.saturating_sub(parent_width);
    format!(
        "{} → {}",
        truncate(parent_id, parent_width),
        truncate(child_id, child_width)
    )
}

fn board_row_line(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
    show_doc_type: bool,
    chips: Vec<(String, Style)>,
    right_meta: String,
    depth: usize,
    selected: bool,
) -> Line<'static> {
    let doc_type = doc_type_badge(doc, show_doc_type);
    let indent_width = depth.saturating_mul(3);
    let chip_width = chips
        .iter()
        .map(|(chip, _)| text_width(chip))
        .sum::<usize>()
        + chips.len();
    let doc_type_width = doc_type
        .as_ref()
        .map(|badge| text_width(badge) + 1)
        .unwrap_or(0);
    let title_separator_width = if chip_width > 0 || doc_type_width > 0 || indent_width > 0 {
        1
    } else {
        0
    };
    let base_width = indent_width + doc_type_width + chip_width + title_separator_width;
    let max_meta_width = content_width.saturating_sub(base_width).saturating_sub(1);
    let right_meta = truncate(&right_meta, max_meta_width);
    let meta_width = text_width(&right_meta);
    let spacer_min_width = if right_meta.is_empty() { 0 } else { 1 };
    let fixed_width = base_width + spacer_min_width + meta_width;
    let title_width = content_width.saturating_sub(fixed_width);
    let title = truncate(doc.title(), title_width);
    let used_before_meta =
        indent_width + doc_type_width + chip_width + title_separator_width + text_width(&title);
    let spacer_width = content_width
        .saturating_sub(used_before_meta + meta_width)
        .max(spacer_min_width);

    let mut spans = Vec::new();
    if indent_width > 0 {
        spans.push(Span::styled("  └".to_string(), theme.muted_style()));
        if indent_width > 3 {
            spans.push(Span::raw(" ".repeat(indent_width - 3)));
        }
    }
    if let Some(doc_type) = doc_type {
        spans.push(Span::styled(doc_type, theme.board_doc_type_style()));
        spans.push(Span::raw(" "));
    }
    for (index, (chip, style)) in chips.into_iter().enumerate() {
        if index > 0 || doc_type_width > 0 || indent_width > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(chip, style));
    }
    if chip_width > 0 || doc_type_width > 0 || indent_width > 0 {
        spans.push(Span::raw(" "));
    }
    let title_style = if selected {
        theme.board_selected_title_style()
    } else {
        theme.text_style().add_modifier(Modifier::BOLD)
    };
    spans.push(Span::styled(title, title_style));
    spans.push(Span::raw(" ".repeat(spacer_width)));
    spans.push(Span::styled(right_meta, theme.muted_style()));
    Line::from(spans)
}

fn descendant_rollup(active: usize, completed: usize) -> String {
    match (active, completed) {
        (0, 0) => "no descendants".to_string(),
        (active, 0) => format!("{active} active"),
        (0, completed) => format!("{completed} logged"),
        (active, completed) => format!("{active} active · {completed} logged"),
    }
}

fn relationship_detail_summary(context: &BoardRelationshipContext) -> String {
    let hints = context.hints();
    let mut parts = Vec::new();
    if hints.active_children > 0 {
        parts.push(format!(
            "{} active {}",
            hints.active_children,
            if hints.active_children == 1 {
                "child"
            } else {
                "children"
            }
        ));
    }
    if hints.completed_children > 0 {
        parts.push(format!(
            "{} completed {} in Logs",
            hints.completed_children,
            if hints.completed_children == 1 {
                "child"
            } else {
                "children"
            }
        ));
    }
    if parts.len() > 1 {
        format!("{} ({} total)", parts.join(", "), hints.total_children())
    } else if parts.is_empty() {
        "no linked children".to_string()
    } else {
        parts.join(", ")
    }
}

fn doc_type_badge(doc: &Document, show_doc_type: bool) -> Option<String> {
    let doc_type = doc.doc_type().trim();
    if doc_type.is_empty() || (!show_doc_type && doc_type == "task") {
        None
    } else {
        Some(doc_type.to_string())
    }
}

fn is_task_doc(doc: &Document) -> bool {
    doc.doc_type() == "task"
}

fn priority_chip(priority: &str, theme: &TuiTheme) -> Option<String> {
    let normalized = priority.trim().to_ascii_lowercase();
    let (label, badge_id) = match normalized.as_str() {
        "critical" | "urgent" => ("CRIT".to_string(), "priority:critical"),
        "high" => ("HIGH".to_string(), "priority:high"),
        "medium" | "med" => ("MED".to_string(), "priority:medium"),
        "low" => ("LOW".to_string(), "priority:low"),
        "" | "-" | "none" => return None,
        other => (
            other.chars().take(4).collect::<String>().to_uppercase(),
            "priority:other",
        ),
    };
    if theme.badge_disabled("priority") || theme.badge_disabled(badge_id) {
        return None;
    }
    Some(chip_text(&label, theme))
}

fn work_type_tag_chips(doc: &Document, theme: &TuiTheme) -> Vec<(String, StatusTone)> {
    let tags = document_tags(doc);
    let mut chips = Vec::new();
    for (tag, default_label) in [
        ("research", "RESEARCH"),
        ("spike", "SPIKE"),
        ("deliverable", "DELIVERABLE"),
    ] {
        if tags.iter().any(|candidate| tag_matches(candidate, tag))
            && !theme.badge_disabled(tag)
            && !theme.badge_disabled(&format!("tag:{tag}"))
        {
            chips.push(configured_or_default_tag_chip(tag, default_label, theme));
        }
    }
    chips
}

fn configured_tag_chips(doc: &Document, theme: &TuiTheme) -> Vec<(String, StatusTone)> {
    document_tags(doc)
        .into_iter()
        .filter(|tag| !is_builtin_work_type_tag(tag))
        .filter_map(|tag| {
            theme
                .tag_badge(&tag)
                .map(|config| (chip_text(&config.label_for(&tag), theme), config.tone()))
        })
        .collect()
}

fn configured_or_default_tag_chip(
    tag: &str,
    default_label: &str,
    theme: &TuiTheme,
) -> (String, StatusTone) {
    if let Some(config) = theme.tag_badge(tag) {
        (chip_text(&config.label_for(tag), theme), config.tone())
    } else {
        (chip_text(default_label, theme), StatusTone::Accent)
    }
}

fn is_builtin_work_type_tag(tag: &str) -> bool {
    ["research", "spike", "deliverable"]
        .iter()
        .any(|candidate| tag_matches(tag, candidate))
}

fn tag_matches(candidate: &str, expected: &str) -> bool {
    candidate.trim().eq_ignore_ascii_case(expected)
}

fn board_filter_bar_line(filters: &BoardFilters, theme: &TuiTheme) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            " FILTERS ",
            theme
                .status_style(StatusTone::Warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if let Some(tag) = filters.tag.as_deref() {
        spans.push(Span::styled(
            chip_text(&format!("#{}", tag), theme),
            theme.progress_chip_style(StatusTone::Accent),
        ));
        spans.push(Span::raw(" "));
    }
    if let Some(priority) = filters.priority.as_deref() {
        spans.push(Span::styled(" priority ", theme.muted_style()));
        spans.push(Span::styled(
            chip_text(priority, theme),
            theme.priority_chip_style(priority),
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(" t/p cycle · F clear ", theme.muted_style()));
    Line::from(spans)
}

fn board_filters_match(doc: &Document, filters: &BoardFilters) -> bool {
    if let Some(tag) = filters.tag.as_deref() {
        if !document_tags(doc)
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(tag))
        {
            return false;
        }
    }
    if let Some(priority) = filters.priority.as_deref() {
        if normalize_filter_value(doc.field("priority").unwrap_or("")) != priority {
            return false;
        }
    }
    true
}

fn board_filter_tags(docs: &[Document]) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for doc in docs.iter().filter(|doc| is_board_visible_doc(doc)) {
        for tag in document_tags(doc) {
            tags.insert(tag);
        }
    }
    tags.into_iter().collect()
}

fn board_filter_priorities(docs: &[Document]) -> Vec<String> {
    let mut priorities = docs
        .iter()
        .filter(|doc| is_board_visible_doc(doc))
        .filter_map(|doc| {
            let priority = normalize_filter_value(doc.field("priority").unwrap_or(""));
            if priority.is_empty() || priority == "-" || priority == "none" {
                None
            } else {
                Some(priority)
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    priorities.sort_by_key(|priority| priority_filter_sort_key(priority));
    priorities
}

fn next_filter_value(current: Option<&str>, values: &[String]) -> Option<String> {
    let next_index = current
        .and_then(|current| values.iter().position(|value| value == current))
        .map(|index| index + 1)
        .unwrap_or(0);
    values.get(next_index).cloned()
}

fn priority_filter_sort_key(priority: &str) -> (usize, String) {
    let rank = match priority {
        "critical" | "urgent" => 0,
        "high" => 1,
        "medium" | "med" => 2,
        "low" => 3,
        _ => 4,
    };
    (rank, priority.to_string())
}

fn document_tags(doc: &Document) -> Vec<String> {
    doc.field("tags")
        .map(|tags| format_inline_list(tags, ""))
        .unwrap_or_default()
        .into_iter()
        .map(|tag| normalize_filter_value(&tag))
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn normalize_filter_value(value: &str) -> String {
    value.trim().trim_start_matches('#').to_ascii_lowercase()
}

fn status_chip(status: &str, theme: &TuiTheme) -> String {
    chip_text(&status.trim().replace('_', "-").to_uppercase(), theme)
}

fn chip_text(label: &str, theme: &TuiTheme) -> String {
    theme.badge_label(label)
}

const INLINE_PREVIEW_MAX_LINES: usize = 25;
const BOARD_LIST_HIGHLIGHT_SYMBOL_WIDTH: u16 = 2;

fn inline_preview_line_limit_for_area(area: Rect) -> usize {
    area.height.saturating_sub(2).saturating_sub(1) as usize
}

fn inline_preview_height_with_context(
    doc: &Document,
    relationship_context: &BoardRelationshipContext,
    content_width: usize,
    preview_line_limit: usize,
) -> u16 {
    inline_preview_lines_for_doc_with_context(
        doc,
        &TuiTheme::default_dark(),
        relationship_context,
        content_width,
        preview_line_limit,
    )
    .len() as u16
}

fn inline_preview_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    relationship_context: &BoardRelationshipContext,
    content_width: usize,
    preview_line_limit: usize,
) -> Vec<Line<'static>> {
    let max_lines = preview_line_limit.min(INLINE_PREVIEW_MAX_LINES);
    if max_lines == 0 {
        return Vec::new();
    }

    let footer_lines = max_lines.min(2);
    let content_limit = max_lines.saturating_sub(footer_lines);
    let files = doc
        .field("relatedFiles")
        .map(|files| format_inline_list(files, ""))
        .unwrap_or_default();
    let subtasks = board_subtasks(doc);
    let checklist_progress = subtask_progress(doc);
    let relationship_lines = inline_relationship_preview_lines(relationship_context, theme);
    let mut trailing_sections = validation_inline_preview_sections(doc, theme, content_width);
    trailing_sections.extend(inline_preview_list_section(
        "Files",
        files,
        theme,
        content_width,
    ));
    trailing_sections.extend(inline_preview_subtasks_section(
        subtasks,
        checklist_progress,
        theme,
    ));

    let mut content_lines = Vec::new();
    if let Some(tags) = doc.field("tags") {
        let tags = format_hash_list(tags);
        content_lines.extend(inline_preview_key_value(
            "Tags",
            &tags,
            theme,
            content_width,
        ));
        content_lines.push(Line::from(""));
    }

    if !relationship_lines.is_empty() {
        content_lines.extend(relationship_lines);
        content_lines.push(Line::from(""));
    }

    if content_lines.len() < content_limit {
        let (summary_label, summary_text) = inline_preview_summary(doc);
        let reserved = content_lines
            .len()
            .saturating_add(1)
            .saturating_add(trailing_sections.len());
        let summary_capacity = content_limit
            .saturating_sub(reserved)
            .max(1)
            .min(INLINE_PREVIEW_MAX_LINES);
        content_lines.push(inline_preview_heading(summary_label, theme));
        content_lines.extend(inline_preview_markdownish(
            &summary_text,
            theme,
            content_width,
            summary_capacity,
        ));
    }

    content_lines.extend(trailing_sections);
    let overflow = content_lines.len() > content_limit;
    if overflow {
        content_lines.truncate(content_limit);
        if let Some(last) = content_lines.last_mut() {
            *last = Line::from(Span::styled("   …", theme.muted_style()));
        }
    }

    let mut lines = content_lines;
    if footer_lines == 2 {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "   Space close preview · Tab detail pane · e edit",
        theme.muted_style(),
    )));
    lines
}

fn inline_preview_summary(doc: &Document) -> (&'static str, String) {
    if document_state_label(doc) == "validation" {
        if let Some(summary) = doc
            .field("accord.summary")
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
        {
            return ("Delivery summary", summary.to_string());
        }
    }
    ("Summary", doc.body.clone())
}

fn validation_inline_preview_sections(
    doc: &Document,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    if document_state_label(doc) != "validation" {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.extend(inline_preview_list_section(
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
        content_width,
    ));
    lines.extend(inline_preview_list_section(
        "Evidence",
        first_accord_list(doc, &["accord.evidence"]),
        theme,
        content_width,
    ));
    lines.extend(inline_preview_list_section(
        "Files changed",
        first_accord_list(doc, &["accord.filesChanged"]),
        theme,
        content_width,
    ));
    lines
}

fn inline_relationship_preview_lines(
    relationship_context: &BoardRelationshipContext,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if relationship_context.task_role == Some(TaskRole::Epic) || relationship_context.has_children()
    {
        let (heading, children_label) = match relationship_context.task_role {
            Some(TaskRole::Epic) => ("Epic", "Tasks"),
            Some(TaskRole::Task) => ("Task", "Subtasks"),
            Some(TaskRole::Subtask) => ("Subtask", "Children"),
            None => ("Relationships", "Children"),
        };
        lines.push(inline_preview_heading(heading, theme));
        if relationship_context.has_children() {
            lines.push(Line::from(vec![
                Span::styled(format!("   {children_label}: "), theme.label_style()),
                Span::styled(
                    relationship_detail_summary(relationship_context),
                    theme.text_style(),
                ),
            ]));
            for child in relationship_context
                .active_children
                .iter()
                .chain(relationship_context.completed_children.iter())
                .take(8)
            {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("   • {} ", display_state_label(&child.state)),
                        theme.muted_style(),
                    ),
                    Span::styled(child.title.clone(), theme.text_style()),
                    Span::styled(format!(" ({})", child.id), theme.muted_style()),
                ]));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "   No linked children yet. Set parentId on child tasks to attach them.",
                theme.muted_style(),
            )));
        }
    }

    if relationship_context.has_parent() {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(inline_preview_heading("Relationship", theme));
        if relationship_context.parent_missing {
            let parent_id = relationship_context
                .parent_id
                .as_deref()
                .unwrap_or("unknown");
            lines.push(Line::from(vec![
                Span::styled("   Missing parent: ", theme.label_style()),
                Span::styled(
                    parent_id.to_string(),
                    theme.status_style(StatusTone::Warning),
                ),
            ]));
        } else if let Some(parent_id) = relationship_context.parent_id.as_deref() {
            let parent_title = relationship_context
                .parent_title
                .as_deref()
                .unwrap_or("untitled parent");
            let label = match relationship_context.parent_relationship {
                Some(ParentRelationship::EpicTask) => "   Task of Epic: ",
                Some(ParentRelationship::Subtask) => "   Subtask of: ",
                Some(ParentRelationship::Parent) | None => "   Parent: ",
            };
            lines.push(Line::from(vec![
                Span::styled(label, theme.label_style()),
                Span::styled(parent_title.to_string(), theme.text_style()),
                Span::styled(format!(" ({parent_id})"), theme.muted_style()),
            ]));
        }
    }
    lines
}

fn inline_preview_heading(label: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(Span::styled(format!("   {label}"), theme.label_style()))
}

fn inline_preview_key_value(
    label: &str,
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    let prefix = format!("   {label}: ");
    let value_width = content_width.saturating_sub(text_width(&prefix)).max(12);
    let wrapped = wrap_words(value, value_width);
    wrapped
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            if index == 0 {
                Line::from(vec![
                    Span::styled(prefix.clone(), theme.label_style()),
                    Span::styled(chunk, theme.text_style()),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" ".repeat(text_width(&prefix))),
                    Span::styled(chunk, theme.text_style()),
                ])
            }
        })
        .collect()
}

fn inline_preview_markdownish(
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
    max_lines: usize,
) -> Vec<Line<'static>> {
    if max_lines == 0 {
        return Vec::new();
    }

    let indent = "   ";
    let value_width = content_width.saturating_sub(text_width(indent)).max(24);
    let mut raw_lines = preview_markdownish_wrapped_lines(value, value_width);
    if raw_lines.len() > max_lines {
        raw_lines.truncate(max_lines);
        if let Some(last) = raw_lines.last_mut() {
            *last = append_preview_ellipsis(last, value_width);
        }
    }

    raw_lines
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                Line::from("")
            } else {
                markdownish_line(&format!("{indent}{line}"), theme)
            }
        })
        .collect()
}

fn preview_markdownish_wrapped_lines(value: &str, width: usize) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return vec!["(no body text)".to_string()];
    }

    let mut lines = Vec::new();
    for raw_line in trimmed.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(wrap_preview_markdownish_line(line, width));
        }
    }
    lines
}

fn wrap_preview_markdownish_line(line: &str, width: usize) -> Vec<String> {
    let width = width.max(12);
    let trimmed = line.trim_start();
    let leading = &line[..line.len() - trimmed.len()];

    if markdown_table_like_line(trimmed) {
        return vec![truncate(line, width)];
    }

    if let Some((marker, content)) = markdown_list_source_parts(trimmed) {
        let prefix = format!("{leading}{marker}");
        let value_width = width.saturating_sub(text_width(&prefix)).max(12);
        return wrap_words(content, value_width)
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| {
                if index == 0 {
                    format!("{prefix}{chunk}")
                } else {
                    format!("{}{chunk}", " ".repeat(text_width(&prefix)))
                }
            })
            .collect();
    }

    if let Some(content) = trimmed
        .strip_prefix("> ")
        .or_else(|| trimmed.strip_prefix('>'))
    {
        let prefix = format!("{leading}> ");
        let value_width = width.saturating_sub(text_width(&prefix)).max(12);
        return wrap_words(content, value_width)
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| {
                if index == 0 {
                    format!("{prefix}{chunk}")
                } else {
                    format!("{}{chunk}", " ".repeat(text_width(&prefix)))
                }
            })
            .collect();
    }

    wrap_words(line, width)
}

fn markdown_list_source_parts(trimmed: &str) -> Option<(&str, &str)> {
    for marker in [
        "- [ ] ", "* [ ] ", "+ [ ] ", "- [x] ", "* [x] ", "+ [x] ", "- [X] ", "* [X] ", "+ [X] ",
        "- ", "* ", "+ ",
    ] {
        if let Some(content) = trimmed.strip_prefix(marker) {
            return Some((&trimmed[..marker.len()], content));
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
        Some((&trimmed[..digit_count + 2], &trimmed[digit_count + 2..]))
    } else {
        None
    }
}

fn markdown_table_like_line(trimmed: &str) -> bool {
    let pipe_count = trimmed.chars().filter(|ch| *ch == '|').count();
    pipe_count >= 2 || trimmed.starts_with('|') || trimmed.ends_with('|')
}

fn append_preview_ellipsis(line: &str, width: usize) -> String {
    let width = width.max(1);
    let char_count = line.chars().count();
    if char_count + 2 <= width {
        format!("{line} …")
    } else {
        let mut truncated = line
            .chars()
            .take(width.saturating_sub(1))
            .collect::<String>();
        truncated.push('…');
        truncated
    }
}

fn inline_preview_list_section(
    label: &str,
    values: Vec<String>,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    if values.is_empty() {
        return Vec::new();
    }

    let total = values.len();
    let max_items = 4;
    let mut lines = vec![Line::from(""), inline_preview_heading(label, theme)];
    for value in values.into_iter().take(max_items) {
        lines.extend(inline_preview_bullet_value(&value, theme, content_width));
    }
    if total > max_items {
        lines.push(Line::from(Span::styled(
            format!("   … {} more", total - max_items),
            theme.muted_style(),
        )));
    }
    lines
}

fn inline_preview_bullet_value(
    value: &str,
    theme: &TuiTheme,
    content_width: usize,
) -> Vec<Line<'static>> {
    let prefix = "   • ";
    let value_width = content_width.saturating_sub(text_width(prefix)).max(12);
    wrap_words(value, value_width)
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            if index == 0 {
                Line::from(vec![
                    Span::styled(prefix.to_string(), theme.muted_style()),
                    Span::styled(chunk, theme.text_style()),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" ".repeat(text_width(prefix))),
                    Span::styled(chunk, theme.text_style()),
                ])
            }
        })
        .collect()
}

fn inline_preview_subtasks_section(
    subtasks: Vec<BoardSubtask>,
    checklist_progress: Option<(usize, usize)>,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    if subtasks.is_empty() {
        return Vec::new();
    }

    let (completed, total) = checklist_progress.unwrap_or((0, subtasks.len()));
    let mut lines = vec![
        Line::from(""),
        inline_preview_heading(&format!("Checklist {completed}/{total}"), theme),
    ];
    for subtask in subtasks {
        let marker = if subtask.completed { "[x]" } else { "[ ]" };
        lines.push(Line::from(vec![
            Span::styled(format!("   {marker} "), theme.muted_style()),
            Span::styled(subtask.title, theme.text_style()),
        ]));
    }
    lines
}

fn format_hash_list(value: &str) -> String {
    format_inline_list(value, "#").join(" ")
}

fn format_inline_list(value: &str, prefix: &str) -> Vec<String> {
    let trimmed = value.trim();
    let values = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(clean_inline_list_item)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    } else {
        trimmed
            .split(',')
            .map(clean_inline_list_item)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    };

    if values.is_empty() {
        vec![trimmed.to_string()]
    } else {
        values
            .into_iter()
            .map(|item| format!("{prefix}{item}"))
            .collect()
    }
}

fn clean_inline_list_item(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn wrap_words(value: &str, width: usize) -> Vec<String> {
    let width = width.max(12);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let separator = usize::from(!current.is_empty());
        if !current.is_empty() && text_width(&current) + separator + text_width(word) > width {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }
    lines
}

fn board_should_surface_accord_status(doc: &Document, status: &str, theme: &TuiTheme) -> bool {
    let normalized = normalized_accord_status(status);
    if document_state_label(doc) == "validation" && normalized == "delivered" {
        return false;
    }
    matches!(
        normalized.as_str(),
        "delivered" | "accepted" | "rework" | "blocked" | "failed"
    ) && !theme.badge_disabled("accord")
        && !theme.badge_disabled(&normalized)
        && !theme.badge_disabled(&format!("accord:{normalized}"))
}

fn validation_visual_chip(doc: &Document, theme: &TuiTheme) -> Option<(String, StatusTone)> {
    if document_state_label(doc) != "validation"
        || theme.badge_disabled("visual")
        || theme.badge_disabled("tag:visual")
        || theme.badge_disabled("validation:visual")
    {
        return None;
    }
    let tags = document_tags(doc);
    tags.iter()
        .any(|tag| {
            ["visual", "ui", "ux"]
                .iter()
                .any(|expected| tag_matches(tag, expected))
        })
        .then(|| configured_or_default_tag_chip("visual", "VISUAL", theme))
}

fn board_should_surface_review_status(status: &str, theme: &TuiTheme) -> bool {
    let normalized = status.trim().replace('_', "-").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "pending" | "changes-requested" | "rejected" | "failed"
    ) && !theme.badge_disabled("review")
        && !theme.badge_disabled(&normalized)
        && !theme.badge_disabled(&format!("review:{normalized}"))
}

#[derive(Debug, Clone)]
struct BoardSubtask {
    title: String,
    completed: bool,
}

fn board_subtasks(doc: &Document) -> Vec<BoardSubtask> {
    let mut by_index: BTreeMap<usize, BoardSubtask> = BTreeMap::new();
    for (key, value) in &doc.fields {
        let Some(rest) = key.strip_prefix("subtasks.") else {
            continue;
        };
        let Some((index, field)) = rest.split_once('.') else {
            continue;
        };
        let Ok(index) = index.parse::<usize>() else {
            continue;
        };
        let entry = by_index.entry(index).or_insert_with(|| BoardSubtask {
            title: format!("subtask {}", index + 1),
            completed: false,
        });
        match field {
            "title" => entry.title = value.to_string(),
            "completed" => entry.completed = is_completed_value(value),
            _ => {}
        }
    }
    by_index.into_values().collect()
}

fn is_completed_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "done" | "1"
    )
}

fn subtask_progress(doc: &Document) -> Option<(usize, usize)> {
    let subtasks = board_subtasks(doc);
    let total = subtasks.len();
    let completed = subtasks.iter().filter(|subtask| subtask.completed).count();
    (total > 0).then_some((completed, total))
}

fn text_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
fn detail_lines_for_doc(doc: &Document, theme: &TuiTheme) -> Vec<Line<'static>> {
    detail_lines_for_doc_with_context(doc, theme, &BoardRelationshipContext::default())
}

fn detail_lines_for_doc_with_context(
    doc: &Document,
    theme: &TuiTheme,
    relationship_context: &BoardRelationshipContext,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Title: ", theme.label_style()),
        Span::styled(doc.title().to_string(), theme.text_style()),
    ]));
    lines.push(detail_field_line("ID", doc.id(), theme));
    lines.push(detail_field_line("Type", doc.doc_type(), theme));
    push_optional_detail_line(&mut lines, "Kind", doc.field("kind"), theme);
    if let Some(role) = relationship_context.task_role {
        lines.push(detail_field_line("Role", role.as_str(), theme));
    }
    push_optional_detail_line(&mut lines, "State", doc.field("state"), theme);
    push_optional_detail_line(&mut lines, "Priority", doc.field("priority"), theme);
    push_optional_detail_line(&mut lines, "Assignee", doc.field("assignee"), theme);
    push_optional_detail_line(&mut lines, "Due", doc.field("dueDate"), theme);
    push_optional_detail_line(&mut lines, "Tags", doc.field("tags"), theme);
    if let Some(parent_id) = relationship_context.parent_id.as_deref() {
        let parent = relationship_context
            .parent_title
            .as_deref()
            .map(|title| format!("{title} ({parent_id})"))
            .unwrap_or_else(|| format!("missing parent {parent_id}"));
        let label = relationship_context
            .parent_relationship
            .map(ParentRelationship::human_label)
            .unwrap_or("Parent");
        lines.push(detail_field_line(label, &parent, theme));
    }
    if let Some(error) = relationship_context.hierarchy_error.as_deref() {
        lines.push(detail_field_line("Hierarchy error", error, theme));
    }
    if relationship_context.has_children() {
        let label = match relationship_context.task_role {
            Some(TaskRole::Epic) => "Tasks",
            Some(TaskRole::Task) => "Subtasks",
            _ => "Children",
        };
        lines.push(detail_field_line(
            label,
            &relationship_detail_summary(relationship_context),
            theme,
        ));
    }
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
    if let Some(warning) = accord_state_divergence_warning(doc) {
        let warning_style = theme.status_style(StatusTone::Warning);
        lines.push(Line::from(vec![
            Span::styled("Warning: ", warning_style.add_modifier(Modifier::BOLD)),
            Span::styled(warning, warning_style),
        ]));
    }
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
    if document_state_label(doc) == "validation" {
        lines.push(Line::from(Span::styled(
            "Board Validation: A opens accept sign-off confirmation, R opens feedback/rework, e opens the task; completion is intentionally separate.",
            theme.muted_style(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Board accord mutations beyond movement are CLI-guided from this detail pane.",
            theme.muted_style(),
        )));
    }
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
        "ready" => "Legacy ready: treat as unclaimed and claim when an owner is known.",
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
        "ready" => "Legacy status: claim the accord when an owner is known.",
        "claimed" => "Deliver when complete, or block/fail with a reason if work cannot proceed.",
        "delivered" => "Inspect the delivery, then accept it or request rework.",
        "accepted" => "Complete/archive the task when it is ready to leave the Board.",
        "rework" => "Apply requested changes, then deliver again with a fresh summary.",
        "blocked" => "Resolve the blocker, then claim/deliver; fail only if unrecoverable.",
        "failed" => "Review the failure and claim again if retrying the work.",
        "missing" | "" => "Claim the accord when an owner is known.",
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
            "tandem accord claim {id} --assignee <name> OR tandem accord fail {id} --reason <text>"
        ),
        "failed" => format!("tandem accord claim {id} --assignee <name>"),
        "missing" | "" => format!(
            "tandem accord claim {id} --assignee <name> [--deliverable <spec>] [--validation <command>]"
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

    use ratatui::backend::TestBackend;

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
    fn board_row_preserves_right_metadata_when_title_space_is_tight() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-102", Some("todo"));
        doc.fields.insert(
            "title".to_string(),
            "Very long nested child task title that must yield to metadata".to_string(),
        );

        let line = board_row_line(
            &doc,
            &theme,
            23,
            false,
            vec![(
                chip_text("TODO", &theme),
                theme.progress_chip_style(StatusTone::Muted),
            )],
            "task-102".to_string(),
            1,
            true,
        );
        let text = line_text(&line);

        assert!(
            text.ends_with("task-102"),
            "right metadata should not lose its trailing digit: {text:?}"
        );
        assert!(
            text_width(&text) <= 23,
            "row should fit its content width instead of clipping metadata: {text:?}"
        );
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

        let lines = board_item_lines_for_doc(&doc, &theme, 120, false, false, false);
        let title = line_text(&lines[0]);

        assert_eq!(lines.len(), 1);
        assert!(title.contains(" HIGH  Polish Board rows"));
        assert!(!title.contains("accord ready"));
        assert!(!title.contains("tui"));
        assert!(!title.contains("[task]"));
        assert!(!title.contains("A:"));
        assert!(lines[0].spans.iter().any(|span| {
            span.content.as_ref() == " HIGH " && span.style == theme.priority_chip_style("high")
        }));
    }

    #[test]
    fn board_row_badges_work_type_tags() {
        let theme = TuiTheme::default_dark();
        let mut research = doc_with_state("task-24", Some("todo"));
        research
            .fields
            .insert("title".to_string(), "Research docs platform".to_string());
        research
            .fields
            .insert("tags".to_string(), "[\"docs\", \"research\"]".to_string());
        let research_title =
            line_text(&board_item_lines_for_doc(&research, &theme, 120, false, false, false)[0]);
        assert!(research_title.contains(" RESEARCH  Research docs platform"));

        let mut spike = doc_with_state("task-25", Some("todo"));
        spike
            .fields
            .insert("title".to_string(), "Spike rendering approach".to_string());
        spike
            .fields
            .insert("tags".to_string(), "[\"tui\", \"spike\"]".to_string());
        let spike_title =
            line_text(&board_item_lines_for_doc(&spike, &theme, 120, false, false, false)[0]);
        assert!(spike_title.contains(" SPIKE  Spike rendering approach"));

        let mut deliverable = doc_with_state("task-28", Some("todo"));
        deliverable
            .fields
            .insert("title".to_string(), "Package release notes".to_string());
        deliverable
            .fields
            .insert("tags".to_string(), "[\"deliverable\"]".to_string());
        let deliverable_title =
            line_text(&board_item_lines_for_doc(&deliverable, &theme, 120, false, false, false)[0]);
        assert!(deliverable_title.contains(" DELIVERABLE  Package release notes"));
    }

    #[test]
    fn board_row_badges_epic_kind_and_keeps_it_in_workflow_states() {
        let theme = TuiTheme::default_dark();
        let mut epic = doc_with_state("task-80", Some("in-progress"));
        epic.fields
            .insert("title".to_string(), "Launch docs epic".to_string());
        epic.fields.insert("kind".to_string(), "epic".to_string());

        let context = relationship_context_for_doc(&epic, std::slice::from_ref(&epic), &[]);
        let title = line_text(
            &board_item_lines_for_doc_with_context(
                &epic, &theme, 120, false, &context, false, false,
            )[0],
        );
        assert_eq!(context.task_role, Some(TaskRole::Epic));
        assert!(title.contains(" EPIC  Launch docs epic"));
        assert!(!title.contains("task Launch docs epic"));

        let docs = vec![epic];
        let tabs = board_subview_tabs(
            &["todo".to_string(), "in-progress".to_string()],
            &docs,
            &BoardFilters::default(),
        );
        assert_eq!(tabs[0].count, 0);
        assert_eq!(tabs[1].count, 1);
    }

    #[test]
    fn mixed_case_task_and_epic_values_are_custom_or_invalid_not_canonical_roles() {
        let theme = TuiTheme::default_dark();
        let mut custom = doc_with_state("custom-1", Some("todo"));
        custom.fields.insert("type".to_string(), "Task".to_string());
        custom.fields.insert("kind".to_string(), "Epic".to_string());
        custom.fields.insert(
            "title".to_string(),
            "Mixed-case custom document".to_string(),
        );
        custom
            .fields
            .insert("parentId".to_string(), "task-91".to_string());
        assert!(!is_task_doc(&custom));
        assert_eq!(doc_type_badge(&custom, false), Some("Task".to_string()));
        let custom_hierarchy = hierarchy_index_for(std::slice::from_ref(&custom), &[]).unwrap();
        assert_eq!(custom_hierarchy.task_role(&custom).unwrap(), None);

        let mut epic = doc_with_state("task-91", Some("todo"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let docs = vec![epic, custom.clone()];
        let epic_entries = epic_board_entries(&docs, &[], &BoardFilters::default());
        assert_eq!(
            epic_entries
                .iter()
                .map(|entry| entry.doc.id())
                .collect::<Vec<_>>(),
            vec!["task-91"],
            "a custom `Task`/`Epic` document parented by an Epic must not be nested as a canonical Task"
        );
        let state_entries = state_board_entries(
            &docs,
            &[],
            "todo",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        let custom_entry = state_entries
            .iter()
            .find(|entry| entry.doc.id() == "custom-1")
            .expect("custom task-like document should remain a contextual root");
        let custom_context = relationship_context_for_doc(&custom, &docs, &[]);
        let custom_row = line_text(
            &state_lines_for_entry(
                custom_entry,
                &custom_context,
                &theme,
                100,
                true,
                INLINE_PREVIEW_MAX_LINES,
                false,
                false,
            )[0],
        );
        assert_eq!(custom_entry.task_role, None);
        assert!(!custom_row.contains("EPIC"));

        let mut invalid_kind = doc_with_state("task-90", Some("todo"));
        invalid_kind
            .fields
            .insert("kind".to_string(), "Epic".to_string());
        let snapshot =
            TuiHierarchySnapshot::from_documents(std::slice::from_ref(&invalid_kind), &[]);
        assert!(snapshot
            .errors
            .iter()
            .any(|error| error.contains("invalid kind `Epic`; expected one of: epic")));
        assert!(snapshot.valid_index().is_none());
    }

    #[test]
    fn board_row_and_detail_show_derived_child_relationship_hints() {
        let theme = TuiTheme::default_dark();
        let mut epic = doc_with_state("task-80", Some("in-progress"));
        epic.fields
            .insert("title".to_string(), "Launch docs epic".to_string());
        epic.fields.insert("kind".to_string(), "epic".to_string());

        let mut active_child = doc_with_state("task-81", Some("todo"));
        active_child
            .fields
            .insert("parentId".to_string(), "task-80".to_string());
        let mut completed_child = doc_with_state("task-82", Some("validation"));
        completed_child.location = DocumentLocation::Logs;
        completed_child
            .fields
            .insert("parentId".to_string(), "task-80".to_string());

        let active_docs = vec![epic.clone(), active_child.clone()];
        let completed_logs = vec![completed_child.clone()];
        let epic_context = relationship_context_for_doc(&epic, &active_docs, &completed_logs);
        assert_eq!(
            epic_context.hints(),
            BoardRelationshipHints {
                active_children: 1,
                completed_children: 1,
            }
        );

        let title = line_text(
            &board_item_lines_for_doc_with_context(
                &epic,
                &theme,
                120,
                false,
                &epic_context,
                false,
                false,
            )[0],
        );
        assert!(title.contains(" EPIC  Launch docs epic"));
        assert!(!title.contains("CHILDREN"));

        let child_context =
            relationship_context_for_doc(&active_child, &active_docs, &completed_logs);
        let child_title = line_text(
            &board_item_lines_for_doc_with_context(
                &active_child,
                &theme,
                120,
                false,
                &child_context,
                false,
                false,
            )[0],
        );
        assert!(!child_title.contains("P:task-80"));

        let expanded_text = board_item_lines_for_doc_with_context(
            &epic,
            &theme,
            120,
            false,
            &epic_context,
            true,
            false,
        )
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");
        assert!(expanded_text.contains("Epic"));
        assert!(
            expanded_text.contains("Tasks: 1 active child, 1 completed child in Logs (2 total)")
        );
        assert!(expanded_text.contains("Task task-81"));
        assert!(expanded_text.contains("Task task-82"));

        let parent_detail = detail_lines_for_doc_with_context(&epic, &theme, &epic_context)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();
        assert!(parent_detail.contains(&"Kind: epic".to_string()));
        assert!(parent_detail
            .contains(&"Tasks: 1 active child, 1 completed child in Logs (2 total)".to_string()));

        let child_detail = detail_lines_for_doc_with_context(&active_child, &theme, &child_context)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();
        assert!(child_detail.contains(&"Task of Epic: Launch docs epic (task-80)".to_string()));
    }

    #[test]
    fn state_board_collapses_task_children_and_expands_nested_cross_state_rows() {
        let mut parent = doc_with_state("task-103", Some("todo"));
        parent.fields.insert("kind".to_string(), "epic".to_string());
        let mut legacy_child = doc_with_state("task-9", Some("validation"));
        legacy_child
            .fields
            .insert("title".to_string(), "Epic task".to_string());
        legacy_child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let mut grandchild = doc_with_state("task-9-1", Some("todo"));
        grandchild
            .fields
            .insert("parentId".to_string(), "task-9".to_string());
        let mut generic_parent_child = doc_with_state("task-7", Some("todo"));
        generic_parent_child
            .fields
            .insert("parentId".to_string(), "decision-4".to_string());
        let decision = decision_doc("decision-4");
        let docs = vec![
            parent,
            legacy_child,
            grandchild,
            generic_parent_child,
            decision,
        ];
        let mut completed = doc_with_state("task-10", Some("validation"));
        completed.location = DocumentLocation::Logs;
        completed
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let logs = vec![completed];

        let collapsed = state_board_entries(
            &docs,
            &logs,
            "todo",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        assert_eq!(
            collapsed
                .iter()
                .map(|entry| entry.doc.id())
                .collect::<Vec<_>>(),
            vec!["task-103", "task-7"]
        );
        assert_eq!(collapsed[0].active_descendants, 2);
        assert_eq!(collapsed[0].completed_descendants, 1);
        assert!(!collapsed[0].expanded);
        assert_eq!(collapsed[1].role, StateBoardEntryRole::Root);

        let expanded = state_board_entries(
            &docs,
            &logs,
            "todo",
            &BoardFilters::default(),
            &BTreeSet::from(["task-103".to_string(), "task-9".to_string()]),
        );
        assert_eq!(
            expanded
                .iter()
                .map(|entry| (entry.doc.id(), entry.depth))
                .collect::<Vec<_>>(),
            vec![
                ("task-103", 0),
                ("task-9", 1),
                ("task-9-1", 2),
                ("task-7", 0),
            ]
        );
        let child = &expanded[1];
        let context = relationship_context_for_doc(child.doc, &docs, &logs);
        let rendered = state_lines_for_entry(
            child,
            &context,
            &TuiTheme::default_dark(),
            42,
            false,
            10,
            false,
            false,
        )
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");
        assert!(!rendered.contains("SUB"));
        assert!(rendered.contains("VAL"));
        assert!(!rendered.contains("task-9"));
        assert!(!rendered.contains('→'));
        assert!(rendered.contains("└▾"));
        assert!(rendered.contains("Epic task"));
        assert!(rendered.find("1 active").is_some());
        assert!(rendered.lines().all(|line| text_width(line) <= 42));
    }

    #[test]
    fn in_progress_subtask_is_visible_with_its_ancestor_path() {
        let mut epic = doc_with_state("task-1", Some("todo"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut task = doc_with_state("task-2", Some("todo"));
        task.fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut subtask = doc_with_state("task-2-1", Some("in-progress"));
        subtask
            .fields
            .insert("parentId".to_string(), "task-2".to_string());
        let docs = vec![epic, task, subtask];

        let entries = state_board_entries(
            &docs,
            &[],
            "in-progress",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );

        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.doc.id(), entry.depth))
                .collect::<Vec<_>>(),
            vec![("task-1", 0), ("task-2", 1), ("task-2-1", 2)]
        );
        assert!(entries[0].expanded);
        assert!(entries[1].expanded);
        assert_eq!(document_state_label(entries[2].doc), "in-progress");
        let subtask_row = state_lines_for_entry(
            &entries[2],
            &relationship_context_for_doc(entries[2].doc, &docs, &[]),
            &TuiTheme::default_dark(),
            100,
            false,
            0,
            false,
            false,
        )
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");
        assert!(subtask_row.contains("WIP"), "{subtask_row}");
    }

    #[test]
    fn board_state_chip_uses_the_configured_color_without_changing_wip_label() {
        let mut epic = doc_with_state("task-1", Some("todo"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut task = doc_with_state("task-2", Some("in-progress"));
        task.fields
            .insert("parentId".to_string(), "task-1".to_string());
        let docs = vec![epic, task];
        let entries = state_board_entries(
            &docs,
            &[],
            "in-progress",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        let task_entry = entries
            .iter()
            .find(|entry| entry.doc.id() == "task-2")
            .expect("in-progress task entry");
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_theme_content(
            r##"
[aliases]
active = "#e0af68"

[badges.states]
in-progress = "active"
"##,
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

        let line = &state_lines_for_entry(
            task_entry,
            &relationship_context_for_doc(task_entry.doc, &docs, &[]),
            &theme,
            100,
            false,
            0,
            false,
            false,
        )[0];
        let state = compact_epic_state("in-progress");
        let chip = chip_text(&format!("{state:<4}"), &theme);
        assert!(line_text(line).contains("WIP"));
        assert!(line.spans.iter().any(|span| {
            span.content.as_ref() == chip && span.style == theme.state_chip_style("in-progress")
        }));
    }

    #[test]
    fn state_board_rejects_parent_cycles_instead_of_promoting_a_fake_root() {
        let mut a = doc_with_state("task-1", Some("todo"));
        a.fields
            .insert("parentId".to_string(), "task-3".to_string());
        let mut b = doc_with_state("task-2", Some("todo"));
        b.fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut c = doc_with_state("task-3", Some("todo"));
        c.fields
            .insert("parentId".to_string(), "task-2".to_string());
        let docs = vec![a, b, c];

        let collapsed = state_board_entries(
            &docs,
            &[],
            "todo",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        assert!(collapsed.is_empty());
        let expanded = state_board_entries(
            &docs,
            &[],
            "todo",
            &BoardFilters::default(),
            &BTreeSet::from([
                "task-1".to_string(),
                "task-2".to_string(),
                "task-3".to_string(),
            ]),
        );
        assert!(expanded.is_empty());
        let errors = validation_load_errors(&docs, &[], &["todo".to_string()]);
        assert!(errors
            .iter()
            .any(|error| error.contains("task hierarchy cycle")));
    }

    #[test]
    fn archived_log_validation_rejects_unknown_completion_outcomes() {
        let mut log = doc_with_state("task-1", None);
        log.location = DocumentLocation::Logs;
        log.fields
            .insert("completedAt".to_string(), "now".to_string());
        log.fields
            .insert("completion.summary".to_string(), "Archived".to_string());
        log.fields
            .insert("completion.outcome".to_string(), "abandoned".to_string());

        let errors = validation_load_errors(&[], &[log], &["todo".to_string()]).join("\n");
        assert!(
            errors
                .contains("invalid completion.outcome `abandoned`; expected completed or canceled"),
            "{errors}"
        );
    }

    #[test]
    fn invalid_hierarchies_surface_actionable_diagnostics_and_render_no_flattened_rows() {
        let nested_epic_parent = doc_with_state("task-1", Some("todo"));
        let mut nested_epic = doc_with_state("task-2", Some("todo"));
        nested_epic
            .fields
            .insert("kind".to_string(), "epic".to_string());
        nested_epic
            .fields
            .insert("parentId".to_string(), "task-1".to_string());

        let task_parent = doc_with_state("task-10", Some("todo"));
        let mut subtask = doc_with_state("task-10-1", Some("todo"));
        subtask
            .fields
            .insert("parentId".to_string(), "task-10".to_string());
        let mut child_beneath_subtask = doc_with_state("task-10-1-1", Some("todo"));
        child_beneath_subtask
            .fields
            .insert("parentId".to_string(), "task-10-1".to_string());

        let mut epic = doc_with_state("task-20", Some("todo"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut hierarchical_epic_task = doc_with_state("task-20-1", Some("todo"));
        hierarchical_epic_task
            .fields
            .insert("parentId".to_string(), "task-20".to_string());

        let task = doc_with_state("task-30", Some("todo"));
        let mut global_subtask = doc_with_state("task-31", Some("todo"));
        global_subtask
            .fields
            .insert("parentId".to_string(), "task-30".to_string());

        let docs = vec![
            nested_epic_parent,
            nested_epic,
            task_parent,
            subtask,
            child_beneath_subtask,
            epic,
            hierarchical_epic_task,
            task,
            global_subtask,
        ];
        let errors = validation_load_errors(&docs, &[], &["todo".to_string()]).join("\n");
        assert!(
            errors.contains("Epic task-2 cannot have parentId"),
            "{errors}"
        );
        assert!(
            errors.contains("cannot be a child of Subtask task-10-1"),
            "{errors}"
        );
        assert!(
            errors.contains("task-20-1") && errors.contains("expected global `task-N`"),
            "{errors}"
        );
        assert!(
            errors.contains("task-31") && errors.contains("expected `task-30-M`"),
            "{errors}"
        );

        let entries = state_board_entries(
            &docs,
            &[],
            "todo",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        assert!(
            entries.is_empty(),
            "invalid descendants must not be flattened into roots"
        );
    }

    #[test]
    fn invalid_hierarchy_renders_persistent_actionable_panel_in_both_arrangements() {
        let root = unique_test_dir("tandem-invalid-hierarchy-panel");
        let workspace = temp_workspace(&root);
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\nkind: epic\ntitle: Epic\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-1-1.md"),
            "---\nid: task-1-1\ntype: task\ntitle: Invalid Epic child ID\nstate: todo\nparentId: task-1\n---\n",
        )
        .unwrap();
        write_task_doc(&workspace, "task-10", "Task parent", "todo");
        fs::write(
            workspace.board_dir.join("task-11.md"),
            "---\nid: task-11\ntype: task\ntitle: Invalid global Subtask ID\nstate: todo\nparentId: task-10\n---\n",
        )
        .unwrap();
        let mut app = TuiApp::load(workspace).unwrap();
        assert_eq!(app.hierarchy.errors.len(), 2);

        let render = |app: &mut TuiApp| {
            let mut terminal = Terminal::new(TestBackend::new(130, 28)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
        };
        for arrangement in [BoardArrangement::State, BoardArrangement::Epic] {
            app.board_arrangement = arrangement;
            let rendered = render(&mut app);
            assert!(rendered.contains("Hierarchy errors (2)"), "{rendered}");
            assert!(rendered.contains("task-1-1"), "{rendered}");
            assert!(rendered.contains("expected global `task-N`"), "{rendered}");
            assert!(rendered.contains("task-11"), "{rendered}");
            assert!(rendered.contains("expected `task-10-M`"), "{rendered}");
            assert!(!rendered.contains("No active items"), "{rendered}");
            assert!(!rendered.contains("No epic groups"), "{rendered}");
        }
        app.start_quick_add();
        assert!(app.quick_add.is_none());
        assert!(app.status.contains("Quick add disabled"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn state_board_keeps_generic_parent_tasks_normal_and_logged_parent_tasks_contextual() {
        let mut custom_parent = doc_with_state("note-1", Some("todo"));
        custom_parent
            .fields
            .insert("type".to_string(), "note".to_string());
        let mut generic_child = doc_with_state("task-7", Some("todo"));
        generic_child
            .fields
            .insert("parentId".to_string(), "note-1".to_string());
        let mut logged_parent = doc_with_state("task-8", Some("validation"));
        logged_parent.location = DocumentLocation::Logs;
        logged_parent
            .fields
            .insert("kind".to_string(), "epic".to_string());
        let mut active_child = doc_with_state("task-9", Some("todo"));
        active_child
            .fields
            .insert("parentId".to_string(), "task-8".to_string());
        active_child
            .fields
            .insert("priority".to_string(), "high".to_string());
        active_child
            .fields
            .insert("accord.status".to_string(), "blocked".to_string());
        let mut active_root = doc_with_state("task-10", Some("todo"));
        active_root
            .fields
            .insert("kind".to_string(), "epic".to_string());
        let mut logged_middle = doc_with_state("task-11", Some("validation"));
        logged_middle.location = DocumentLocation::Logs;
        logged_middle
            .fields
            .insert("parentId".to_string(), "task-10".to_string());
        let mut deep_active = doc_with_state("task-11-1", Some("validation"));
        deep_active
            .fields
            .insert("parentId".to_string(), "task-11".to_string());
        let docs = vec![
            custom_parent,
            generic_child,
            active_child,
            active_root,
            deep_active,
        ];
        let logs = vec![logged_parent, logged_middle];

        let entries = state_board_entries(
            &docs,
            &logs,
            "todo",
            &BoardFilters::default(),
            &BTreeSet::new(),
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.doc.id(), entry.role))
                .collect::<Vec<_>>(),
            vec![
                ("note-1", StateBoardEntryRole::Root),
                ("task-7", StateBoardEntryRole::Root),
                ("task-9", StateBoardEntryRole::Child),
                ("task-10", StateBoardEntryRole::Root),
            ]
        );
        assert!(!entries[0].has_active_children);
        assert_eq!(entries[0].active_descendants, 0);
        assert!(entries[3].has_active_children);
        assert_eq!(entries[3].active_descendants, 1);
        assert_eq!(entries[3].completed_descendants, 1);
        let contextual_root = &entries[2];
        assert_eq!(contextual_root.depth, 0);
        let contextual_row = line_text(
            &state_lines_for_entry(
                contextual_root,
                &relationship_context_for_doc(contextual_root.doc, &docs, &logs),
                &TuiTheme::default_dark(),
                100,
                false,
                10,
                false,
                false,
            )[0],
        );
        assert!(contextual_row.contains("HIGH"), "{contextual_row}");
        assert!(contextual_row.contains("BLOCKED"), "{contextual_row}");

        let expanded = state_board_entries(
            &docs,
            &logs,
            "todo",
            &BoardFilters::default(),
            &BTreeSet::from(["task-10".to_string()]),
        );
        assert!(expanded
            .iter()
            .any(|entry| entry.doc.id() == "task-11-1" && entry.depth == 2));
    }

    #[test]
    fn default_state_board_render_hides_then_reveals_subtask_rows() {
        let mut app = keyboard_test_app();
        app.docs[0]
            .fields
            .insert("title".to_string(), "Hierarchy parent".to_string());
        let mut child = doc_with_state("task-1-1", Some("validation"));
        child
            .fields
            .insert("title".to_string(), "Hidden subtask".to_string());
        child
            .fields
            .insert("parentId".to_string(), "task-1".to_string());
        app.docs.push(child);
        refresh_test_hierarchy(&mut app);

        let render = |app: &mut TuiApp| {
            let mut terminal = Terminal::new(TestBackend::new(110, 24)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
        };
        let collapsed = render(&mut app);
        assert!(collapsed.contains("Hierarchy parent"));
        assert!(collapsed.contains("1 active"));
        assert!(!collapsed.contains("Hidden subtask"));

        app.handle_key(key(KeyCode::Enter)).unwrap();
        let expanded = render(&mut app);
        assert!(expanded.contains("Hidden subtask"));
        assert!(!expanded.contains("SUB"));
        assert!(expanded.contains("VAL"));
        assert!(!expanded.contains("task-1 → task-9"));
        assert!(!expanded.contains("task-9"));
        assert!(expanded.contains("Selected task-1"));
    }

    #[test]
    fn state_board_rows_label_subtasks_and_align_child_titles() {
        let theme = TuiTheme::default_dark();
        let mut parent = doc_with_state("task-20", Some("todo"));
        parent
            .fields
            .insert("title".to_string(), "Quiet parent".to_string());
        let mut same_state = doc_with_state("task-20-1", Some("todo"));
        same_state
            .fields
            .insert("title".to_string(), "Same state child".to_string());
        same_state
            .fields
            .insert("parentId".to_string(), "task-20".to_string());
        let mut cross_state = doc_with_state("task-20-2", Some("in-progress"));
        cross_state
            .fields
            .insert("title".to_string(), "Cross state child".to_string());
        cross_state
            .fields
            .insert("parentId".to_string(), "task-20".to_string());
        let docs = vec![parent, same_state, cross_state];
        let entries = state_board_entries(
            &docs,
            &[],
            "todo",
            &BoardFilters::default(),
            &BTreeSet::from(["task-20".to_string()]),
        );
        let render = |entry: &StateBoardEntry<'_>| {
            line_text(
                &state_lines_for_entry(
                    entry,
                    &relationship_context_for_doc(entry.doc, &docs, &[]),
                    &theme,
                    100,
                    false,
                    10,
                    false,
                    false,
                )[0],
            )
        };
        let parent_line = render(&entries[0]);
        let same_line = render(&entries[1]);
        let cross_line = render(&entries[2]);

        assert!(!parent_line.contains("task-20"));
        assert!(parent_line.contains("2 active"));
        assert!(!same_line.contains("task-20-1"));
        assert!(!cross_line.contains("task-20-2"));
        assert!(same_line.contains("#20-1"));
        assert!(cross_line.contains("#20-2"));
        assert!(!same_line.contains("SUB"));
        assert!(!cross_line.contains("SUB"));
        assert!(same_line.contains("TODO"));
        assert!(cross_line.contains("WIP"));
        assert_eq!(
            same_line.find("Same state child"),
            cross_line.find("Cross state child")
        );
        assert!(same_line.contains("├─"));
        assert!(!same_line.contains('→'));
        assert!(!cross_line.contains('→'));
    }

    #[test]
    fn state_board_filters_reveal_matching_descendant_ancestor_path() {
        let mut parent = doc_with_state("task-1", Some("todo"));
        parent.fields.insert("kind".to_string(), "epic".to_string());
        parent
            .fields
            .insert("tags".to_string(), "[\"backend\"]".to_string());
        let mut child = doc_with_state("task-2", Some("in-progress"));
        child
            .fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut grandchild = doc_with_state("task-2-1", Some("validation"));
        grandchild
            .fields
            .insert("parentId".to_string(), "task-2".to_string());
        grandchild
            .fields
            .insert("tags".to_string(), "[\"ux\"]".to_string());
        let docs = vec![parent, child, grandchild];
        let filters = BoardFilters {
            tag: Some("ux".to_string()),
            priority: None,
        };

        let todo_entries = state_board_entries(&docs, &[], "todo", &filters, &BTreeSet::new());
        assert!(todo_entries.is_empty());
        let validation_entries =
            state_board_entries(&docs, &[], "validation", &filters, &BTreeSet::new());
        assert_eq!(
            validation_entries
                .iter()
                .map(|entry| entry.doc.id())
                .collect::<Vec<_>>(),
            vec!["task-1", "task-2", "task-2-1"]
        );
        let tabs = board_subview_tabs(
            &[
                "todo".to_string(),
                "in-progress".to_string(),
                "validation".to_string(),
            ],
            &docs,
            &filters,
        );
        assert_eq!(
            tabs.iter().map(|tab| tab.count).collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
    }

    #[test]
    fn state_board_enter_and_mouse_expand_children_while_space_controls_preview() {
        let mut app = keyboard_test_app();
        let mut child = doc_with_state("task-1-1", Some("validation"));
        child
            .fields
            .insert("parentId".to_string(), "task-1".to_string());
        app.docs.push(child);
        refresh_test_hierarchy(&mut app);

        assert_eq!(app.selected_state_count(), 1);
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(app.expanded_board_hierarchy_ids.contains("task-1"));
        assert!(app.status.contains("Expanded Subtasks under task-1"));
        assert_eq!(app.selected_state_count(), 2);
        app.next_item();
        assert_eq!(app.selected_doc().map(Document::id), Some("task-1-1"));
        app.handle_key(key(KeyCode::Char(' '))).unwrap();
        assert_eq!(app.expanded_board_doc_id.as_deref(), Some("task-1-1"));

        app.selected_item = 0;
        app.hits = vec![HitRegion {
            rect: Rect {
                x: 2,
                y: 4,
                width: 20,
                height: 1,
            },
            action: HitAction::SelectBoardItem(0, 0),
        }];
        app.handle_mouse(left_click(3, 4));
        assert!(!app.expanded_board_hierarchy_ids.contains("task-1"));
        assert_eq!(app.selected_state_count(), 1);
    }

    #[test]
    fn enter_labels_state_epic_tasks_and_previews_in_epic_arrangement() {
        let mut app = keyboard_test_app();
        app.docs.clear();
        let mut epic = doc_with_state("task-1", Some("todo"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut task = doc_with_state("task-2", Some("todo"));
        task.fields
            .insert("parentId".to_string(), "task-1".to_string());
        app.docs = vec![epic, task];
        app.states = vec!["todo".to_string()];
        app.configured_states = app.states.clone();
        refresh_test_hierarchy(&mut app);

        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(app.status.contains("Expanded Tasks under task-1"));
        assert!(app.expanded_board_hierarchy_ids.contains("task-1"));

        app.board_arrangement = BoardArrangement::Epic;
        app.selected_item = 0;
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.expanded_board_doc_id.as_deref(), Some("task-1"));
        assert!(app.status.contains("press Enter to close"));
        assert!(!app.status.contains("Expanded Tasks"));
        assert!(app.board_footer_text().contains("Enter/Space preview"));
    }

    #[test]
    fn state_board_reload_preserves_expansion_and_selected_child() {
        let root = unique_test_dir("tandem-state-hierarchy-reload");
        let workspace = temp_workspace(&root);
        write_task_doc(&workspace, "task-1", "Parent", "todo");
        fs::write(
            workspace.board_dir.join("task-1-1.md"),
            "---\nid: task-1-1\ntype: task\ntitle: Subtask\nstate: validation\nparentId: task-1\n---\n",
        )
        .unwrap();
        let mut app = TuiApp::load(workspace.clone()).unwrap();
        app.expanded_board_hierarchy_ids
            .insert("task-1".to_string());
        assert!(app.select_document_by_id("task-1-1"));

        fs::write(
            workspace.board_dir.join("task-1-1.md"),
            "---\nid: task-1-1\ntype: task\ntitle: Reloaded subtask\nstate: validation\nparentId: task-1\n---\n",
        )
        .unwrap();
        app.reload();
        assert!(app.expanded_board_hierarchy_ids.contains("task-1"));
        assert_eq!(app.selected_doc().map(Document::id), Some("task-1-1"));
        assert_eq!(
            app.selected_doc().map(Document::title),
            Some("Reloaded subtask")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn epic_board_entries_only_include_epics_and_their_task_children() {
        let mut epic = doc_with_state("task-80", Some("in-progress"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut epic_child = doc_with_state("task-81", Some("todo"));
        epic_child
            .fields
            .insert("parentId".to_string(), "task-80".to_string());
        let unparented_validation = doc_with_state("task-82", Some("validation"));
        let mut decision_child = decision_doc("decision-1");
        decision_child
            .fields
            .insert("parentId".to_string(), "task-80".to_string());
        let non_epic_parent = doc_with_state("task-83", Some("todo"));
        let mut non_epic_child = doc_with_state("task-83-1", Some("todo"));
        non_epic_child
            .fields
            .insert("parentId".to_string(), "task-83".to_string());

        let docs = vec![
            epic,
            epic_child,
            unparented_validation,
            decision_child,
            non_epic_parent,
            non_epic_child,
        ];
        let entries = epic_board_entries(&docs, &[], &BoardFilters::default());
        let ids = entries
            .iter()
            .map(|entry| entry.doc.id())
            .collect::<Vec<_>>();
        let roles = entries.iter().map(|entry| entry.role).collect::<Vec<_>>();
        let depths = entries.iter().map(|entry| entry.depth).collect::<Vec<_>>();

        assert_eq!(ids, vec!["task-80", "task-81"]);
        assert_eq!(
            roles,
            vec![EpicBoardEntryRole::Epic, EpicBoardEntryRole::Task]
        );
        assert_eq!(depths, vec![0, 1]);
    }

    #[test]
    fn epic_board_nests_canonical_tasks_and_subtasks_and_preserves_filter_context() {
        let mut epic = doc_with_state("task-103", Some("in-progress"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut child = doc_with_state("task-104", Some("todo"));
        child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let mut grandchild = doc_with_state("task-104-1", Some("validation"));
        grandchild
            .fields
            .insert("parentId".to_string(), "task-104".to_string());
        grandchild
            .fields
            .insert("tags".to_string(), "[\"ux\"]".to_string());
        let mut flat_child = doc_with_state("task-9", Some("todo"));
        flat_child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let docs = vec![epic, child, grandchild, flat_child];

        let entries = epic_board_entries(&docs, &[], &BoardFilters::default());
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.doc.id(), entry.depth))
                .collect::<Vec<_>>(),
            vec![
                ("task-103", 0),
                ("task-104", 1),
                ("task-104-1", 2),
                ("task-9", 1)
            ]
        );
        assert_eq!(entries[1].role, EpicBoardEntryRole::Task);
        assert_eq!(entries[2].role, EpicBoardEntryRole::Subtask);
        let theme = TuiTheme::default_dark();
        let task_row = line_text(&epic_row_line(
            &entries[1],
            &relationship_context_for_doc(entries[1].doc, &docs, &[]),
            &theme,
            100,
            false,
        ));
        let subtask_row = line_text(&epic_row_line(
            &entries[2],
            &relationship_context_for_doc(entries[2].doc, &docs, &[]),
            &theme,
            100,
            false,
        ));
        assert!(!task_row.contains("SUB"));
        assert!(subtask_row.contains("SUB"));

        let filtered = epic_board_entries(
            &docs,
            &[],
            &BoardFilters {
                tag: Some("ux".to_string()),
                priority: None,
            },
        );
        assert_eq!(
            filtered
                .iter()
                .map(|entry| entry.doc.id())
                .collect::<Vec<_>>(),
            vec!["task-103", "task-104", "task-104-1"]
        );
    }

    #[test]
    fn epic_board_rollup_counts_completed_nested_descendants_without_active_rows() {
        let mut epic = doc_with_state("task-103", Some("in-progress"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut active_child = doc_with_state("task-104", Some("todo"));
        active_child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let mut completed_grandchild = doc_with_state("task-104-1", Some("validation"));
        completed_grandchild.location = DocumentLocation::Logs;
        completed_grandchild
            .fields
            .insert("parentId".to_string(), "task-104".to_string());
        let docs = vec![epic, active_child];
        let logs = vec![completed_grandchild];

        let entries = epic_board_entries(&docs, &logs, &BoardFilters::default());
        assert_eq!(entries.len(), 2, "completed descendants are rollup-only");
        assert_eq!(entries[0].active_descendants, 1);
        assert_eq!(entries[0].completed_descendants, 1);
        assert_eq!(descendant_rollup(1, 1), "1 active · 1 logged");
    }

    #[test]
    fn canceled_descendants_do_not_count_as_successful_completion() {
        let mut epic = doc_with_state("task-103", Some("in-progress"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut canceled_child = doc_with_state("task-104", Some("todo"));
        canceled_child.location = DocumentLocation::Logs;
        canceled_child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        canceled_child.fields.insert(
            "completion.outcome".to_string(),
            COMPLETION_OUTCOME_CANCELED.to_string(),
        );
        let docs = vec![epic];
        let logs = vec![canceled_child];

        let entries = epic_board_entries(&docs, &logs, &BoardFilters::default());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].active_descendants, 0);
        assert_eq!(entries[0].completed_descendants, 0);
        let relationship = relationship_context_for_doc(&docs[0], &docs, &logs);
        assert!(relationship.completed_children.is_empty());
    }

    #[test]
    fn epic_board_traverses_logged_task_to_active_filtered_subtask() {
        let mut epic = doc_with_state("task-1", Some("in-progress"));
        epic.fields.insert("kind".to_string(), "epic".to_string());
        let mut completed_child = doc_with_state("task-2", Some("validation"));
        completed_child.location = DocumentLocation::Logs;
        completed_child
            .fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut active_grandchild = doc_with_state("task-2-1", Some("todo"));
        active_grandchild
            .fields
            .insert("parentId".to_string(), "task-2".to_string());
        active_grandchild
            .fields
            .insert("tags".to_string(), "[\"ux\"]".to_string());
        let docs = vec![epic, active_grandchild];
        let logs = vec![completed_child];
        let entries = epic_board_entries(
            &docs,
            &logs,
            &BoardFilters {
                tag: Some("ux".to_string()),
                priority: None,
            },
        );

        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.doc.id(), entry.depth))
                .collect::<Vec<_>>(),
            vec![("task-1", 0), ("task-2-1", 2)]
        );
        assert_eq!(entries[0].active_descendants, 1);
        assert_eq!(entries[0].completed_descendants, 1);
        let context = relationship_context_for_doc(&docs[1], &docs, &logs);
        assert_eq!(context.task_role, Some(TaskRole::Subtask));
        assert_eq!(
            context.parent_relationship,
            Some(ParentRelationship::Subtask)
        );
        assert_eq!(context.parent_id.as_deref(), Some("task-2"));
        let rendered = epic_lines_for_entry(
            &entries[1],
            &context,
            &TuiTheme::default_dark(),
            140,
            10,
            false,
            false,
        )
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");
        assert!(rendered.contains("task-2-1"));
        assert!(rendered.contains("task-2 → task-2-1"));
    }

    #[test]
    fn parent_context_labels_only_task_parents_as_subtasks() {
        let task_parent = doc_with_state("task-103", Some("todo"));
        let decision_parent = decision_doc("decision-4");
        let mut task_child = doc_with_state("task-103-1", Some("validation"));
        task_child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let mut generic_child = doc_with_state("task-7", Some("validation"));
        generic_child
            .fields
            .insert("parentId".to_string(), "decision-4".to_string());
        let docs = vec![
            task_parent,
            decision_parent,
            task_child.clone(),
            generic_child.clone(),
        ];
        let theme = TuiTheme::default_dark();

        let task_lines = detail_lines_for_doc_with_context(
            &task_child,
            &theme,
            &relationship_context_for_doc(&task_child, &docs, &[]),
        );
        let generic_lines = detail_lines_for_doc_with_context(
            &generic_child,
            &theme,
            &relationship_context_for_doc(&generic_child, &docs, &[]),
        );
        assert!(task_lines
            .iter()
            .map(line_text)
            .any(|line| line.starts_with("Subtask of:")));
        assert!(generic_lines
            .iter()
            .map(line_text)
            .any(|line| line.starts_with("Parent:")));
        assert!(!generic_lines
            .iter()
            .map(line_text)
            .any(|line| line.contains("Subtask")));
    }

    #[test]
    fn epic_board_navigation_selects_deep_descendants() {
        let root = unique_test_dir("tandem-epic-navigation");
        let workspace = temp_workspace(&root);
        fs::write(
            workspace.board_dir.join("task-103.md"),
            "---\nid: task-103\ntype: task\nkind: epic\ntitle: Epic\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-104.md"),
            "---\nid: task-104\ntype: task\ntitle: Task\nstate: todo\nparentId: task-103\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-104-1.md"),
            "---\nid: task-104-1\ntype: task\ntitle: Subtask\nstate: validation\nparentId: task-104\n---\n",
        )
        .unwrap();
        let mut app = TuiApp::load(workspace).unwrap();
        app.board_arrangement = BoardArrangement::Epic;
        app.next_item();
        app.next_item();
        assert_eq!(app.selected_doc().map(Document::id), Some("task-104-1"));
        app.previous_item();
        assert_eq!(app.selected_doc().map(Document::id), Some("task-104"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn epic_child_row_names_subtask_and_immediate_parent() {
        let theme = TuiTheme::default_dark();
        let parent = doc_with_state("task-103", Some("todo"));
        let mut child = doc_with_state("task-103-1", Some("validation"));
        child
            .fields
            .insert("parentId".to_string(), "task-103".to_string());
        let docs = vec![parent, child.clone()];
        let context = relationship_context_for_doc(&child, &docs, &[]);
        let lines = epic_lines_for_entry(
            &EpicBoardEntry {
                doc: &child,
                role: EpicBoardEntryRole::Subtask,
                depth: 2,
                active_descendants: 0,
                completed_descendants: 0,
            },
            &context,
            &theme,
            140,
            10,
            false,
            false,
        );
        let text = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
        assert!(text.contains("SUB"));
        assert!(!text.contains("SUBTASK"));
        assert!(text.contains("VAL"));
        assert!(text.contains("task-103 → task-103-1"));
        assert!(!text.contains("subtask of"));
    }

    #[test]
    fn epic_rows_use_compact_aligned_state_and_relationship_columns() {
        let theme = TuiTheme::default_dark();
        let mut parent = doc_with_state("task-1", Some("in-progress"));
        parent.fields.insert("kind".to_string(), "epic".to_string());
        let mut todo = doc_with_state("task-2", Some("todo"));
        todo.fields
            .insert("title".to_string(), "Todo title".to_string());
        todo.fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut wip = doc_with_state("task-3", Some("in-progress"));
        wip.fields
            .insert("title".to_string(), "Wip title".to_string());
        wip.fields
            .insert("parentId".to_string(), "task-1".to_string());
        let mut validation = doc_with_state("task-4", Some("validation"));
        validation
            .fields
            .insert("title".to_string(), "Val title".to_string());
        validation
            .fields
            .insert("parentId".to_string(), "task-1".to_string());
        let docs = vec![parent, todo, wip, validation];

        let render = |doc: &Document, width| {
            let context = relationship_context_for_doc(doc, &docs, &[]);
            line_text(&epic_row_line(
                &EpicBoardEntry {
                    doc,
                    role: EpicBoardEntryRole::Task,
                    depth: 1,
                    active_descendants: 0,
                    completed_descendants: 0,
                },
                &context,
                &theme,
                width,
                false,
            ))
        };
        let todo_line = render(&docs[1], 100);
        let wip_line = render(&docs[2], 100);
        let val_line = render(&docs[3], 100);

        assert!(!todo_line.contains("SUB"));
        assert!(todo_line.contains("TODO"));
        assert!(wip_line.contains("WIP"));
        assert!(val_line.contains("VAL"));
        assert_eq!(todo_line.find("Todo title"), wip_line.find("Wip title"));
        assert_eq!(todo_line.find("Todo title"), val_line.find("Val title"));
        assert_eq!(todo_line.find("task-1 →"), wip_line.find("task-1 →"));
        assert_eq!(todo_line.find("task-1 →"), val_line.find("task-1 →"));
        assert!(todo_line.contains("task-1 → task-2"));

        let narrow = render(&docs[3], 42);
        assert!(
            narrow.chars().count() <= 42,
            "narrow row overflowed: {narrow}"
        );
        assert!(narrow.contains('→'), "narrow row lost direction: {narrow}");
        assert!(!narrow.contains("subtask of"));
        assert_eq!(compact_epic_state("review"), "VAL");
        assert_eq!(compact_epic_state("blocked"), "BLK");
    }

    #[test]
    fn actual_board_render_surfaces_epic_grouping_and_expanded_relationships() {
        let root = unique_test_dir("tandem-epic-render");
        let workspace = temp_workspace(&root);
        fs::write(
            workspace.board_dir.join("task-1.md"),
            "---\nid: task-1\ntype: task\nkind: epic\ntitle: \"Ship hierarchical subtasks\"\nstate: todo\npriority: high\n---\n\nParent epic body.\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-2.md"),
            "---\nid: task-2\ntype: task\ntitle: First epic task\nstate: in-progress\npriority: medium\nparentId: task-1\n---\n\nGlobally allocated direct Epic task.\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-3.md"),
            "---\nid: task-3\ntype: task\ntitle: Second epic task\nstate: todo\nparentId: task-1\n---\n\nAnother globally allocated direct Epic task.\n",
        )
        .unwrap();
        fs::write(
            workspace.logs_dir.join("task-4.md"),
            "---\nid: task-4\ntype: task\ntitle: Completed epic task\nstate: validation\nparentId: task-1\ncompletedAt: \"2026-07-01T00:00:00Z\"\ncompletion:\n  summary: \"Completed epic task\"\n---\n\nCompleted task body.\n",
        )
        .unwrap();

        let mut app = TuiApp::load(workspace.clone()).unwrap();
        app.board_arrangement = BoardArrangement::Epic;
        assert!(app.select_document_by_id("task-1"));
        app.expanded_board_doc_id = Some("task-1".to_string());
        app.show_board_detail = true;
        let mut terminal = Terminal::new(TestBackend::new(150, 40)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(
            rendered.contains("EPIC"),
            "rendered Board should include EPIC badge: {rendered}"
        );
        assert!(
            rendered.contains("2 active") && rendered.contains("1 logged"),
            "Epic Board row should include concise active/logged rollup text: {rendered}"
        );
        assert!(
            rendered.contains("First epic task")
                && rendered.contains("task-1 → task-2")
                && rendered.contains("Second epic task")
                && rendered.contains("task-1 → task-3")
                && !rendered.contains("SUB"),
            "Epic Board should show global-ID direct Tasks without Subtask labels: {rendered}"
        );
        assert!(
            !rendered.contains("P:task-1"),
            "Epic Board should avoid noisy parent-id chips: {rendered}"
        );
        assert!(
            rendered.contains("Kind: epic"),
            "detail pane should include task kind: {rendered}"
        );
        assert!(
            rendered.contains("Tasks: 2 active children, 1 completed child in Logs (3 total)"),
            "detail pane should include derived child summary: {rendered}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validation_rows_suppress_redundant_delivered_and_show_review_signals() {
        let theme = TuiTheme::default_dark();
        let mut delivered = doc_with_state("task-26", Some("validation"));
        delivered
            .fields
            .insert("title".to_string(), "Inspect visual polish".to_string());
        delivered
            .fields
            .insert("tags".to_string(), "[\"visual\", \"ux\"]".to_string());
        delivered
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());
        let delivered_title =
            line_text(&board_item_lines_for_doc(&delivered, &theme, 120, false, false, false)[0]);
        assert!(delivered_title.contains(" VISUAL "));
        assert!(!delivered_title.contains(" DELIVERED "));

        let mut accepted = doc_with_state("task-27", Some("validation"));
        accepted
            .fields
            .insert("title".to_string(), "Signed off".to_string());
        accepted
            .fields
            .insert("accord.status".to_string(), "accepted".to_string());
        let accepted_title =
            line_text(&board_item_lines_for_doc(&accepted, &theme, 120, false, false, false)[0]);
        assert!(accepted_title.contains(" ACCEPTED "));
    }

    #[test]
    fn board_row_uses_configured_tag_badges_and_disabled_badges() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_display_content(
            r#"
[board.badges]
disabled = ["priority:high", "visual", "accord:accepted", "subtasks"]

[board.badges.tags.tui]
label = "TUI"
tone = "success"
"#,
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

        let mut doc = doc_with_state("task-29", Some("validation"));
        doc.fields
            .insert("title".to_string(), "Review TUI badge config".to_string());
        doc.fields
            .insert("priority".to_string(), "high".to_string());
        doc.fields
            .insert("tags".to_string(), "[\"tui\", \"visual\"]".to_string());
        doc.fields
            .insert("accord.status".to_string(), "accepted".to_string());
        doc.fields
            .insert("subtasks.0.title".to_string(), "Write docs".to_string());
        doc.fields
            .insert("subtasks.0.completed".to_string(), "true".to_string());

        let line = board_item_lines_for_doc(&doc, &theme, 140, false, false, false)[0].clone();
        let title = line_text(&line);
        assert!(title.contains(" TUI "), "rendered row: {title}");
        assert!(
            title.contains("Review TUI badge config"),
            "rendered row: {title}"
        );
        assert!(!title.contains(" HIGH "));
        assert!(!title.contains(" VISUAL "));
        assert!(!title.contains(" ACCEPTED "));
        assert!(!title.contains(" 1/1 "));
        assert!(
            line.spans.iter().any(|span| {
                span.content.trim() == "TUI"
                    && span.style == theme.progress_chip_style(StatusTone::Success)
            }),
            "spans: {:?}",
            line.spans
        );
    }

    #[test]
    fn board_filters_match_existing_tags_and_priorities() {
        let mut research = doc_with_state("task-24", Some("todo"));
        research
            .fields
            .insert("tags".to_string(), "[\"docs\", \"research\"]".to_string());
        research
            .fields
            .insert("priority".to_string(), "medium".to_string());
        let mut implementation = doc_with_state("task-52", Some("todo"));
        implementation
            .fields
            .insert("tags".to_string(), "[\"tui\", \"board\"]".to_string());
        implementation
            .fields
            .insert("priority".to_string(), "high".to_string());

        let docs = vec![research, implementation];
        let filters = BoardFilters {
            tag: Some("tui".to_string()),
            priority: Some("high".to_string()),
        };
        let tabs = board_subview_tabs(&["todo".to_string()], &docs, &filters);
        assert_eq!(tabs[0].count, 1);
        assert!(board_filters_match(&docs[1], &filters));
        assert!(!board_filters_match(&docs[0], &filters));
    }

    #[test]
    fn board_filter_key_cycles_and_clears_filters() {
        let mut app = keyboard_test_app();
        app.docs[0]
            .fields
            .insert("tags".to_string(), "[\"research\"]".to_string());
        app.docs[0]
            .fields
            .insert("priority".to_string(), "high".to_string());
        app.docs[1]
            .fields
            .insert("tags".to_string(), "[\"spike\"]".to_string());
        app.docs[1]
            .fields
            .insert("priority".to_string(), "low".to_string());

        app.handle_key(key(KeyCode::Char('t'))).unwrap();
        assert_eq!(app.board_filters.tag.as_deref(), Some("research"));
        assert_eq!(app.selected_state_count(), 1);

        app.handle_key(key(KeyCode::Char('p'))).unwrap();
        assert_eq!(app.board_filters.priority.as_deref(), Some("high"));
        assert_eq!(app.selected_state_count(), 1);

        app.handle_key(key(KeyCode::Char('F'))).unwrap();
        assert_eq!(app.board_filters, BoardFilters::default());
        assert!(app.status.contains("cleared"));
    }

    #[test]
    fn active_board_filters_render_as_prominent_bar_not_footer_criteria() {
        let mut app = keyboard_test_app();
        app.docs[0]
            .fields
            .insert("tags".to_string(), "[\"research\"]".to_string());
        app.docs[0]
            .fields
            .insert("priority".to_string(), "high".to_string());
        app.board_filters = BoardFilters {
            tag: Some("research".to_string()),
            priority: Some("high".to_string()),
        };

        let footer = app.board_footer_text();
        assert!(footer.contains("F clear"));
        assert!(!footer.contains("#research"));
        assert!(!footer.contains("priority high"));

        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Active Board filters"));
        assert!(rendered.contains("#research"));
        assert!(rendered.contains("priority"));
        assert!(rendered.contains("high"));
        assert!(rendered.contains("F clear"));
    }

    #[test]
    fn board_row_expansion_adds_at_a_glance_preview_without_metadata_dump() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-33", Some("todo"));
        doc.fields
            .insert("title".to_string(), "Refine Board layout".to_string());
        doc.fields
            .insert("tags".to_string(), "[\"tui\", \"board\"]".to_string());
        doc.fields.insert(
            "relatedFiles".to_string(),
            "[\"tandem/src/tui.rs\", \"tandem/src/tui/theme.rs\"]".to_string(),
        );
        doc.fields.insert(
            "subtasks.0.title".to_string(),
            "Keep tags clean".to_string(),
        );
        doc.fields
            .insert("subtasks.0.completed".to_string(), "true".to_string());
        doc.fields.insert(
            "subtasks.1.title".to_string(),
            "Add checklist preview".to_string(),
        );
        doc.fields
            .insert("subtasks.1.completed".to_string(), "false".to_string());
        doc.body = "## Description\n\nUse one large Board pane by default and keep metadata inline. This expanded row should read as a paragraph instead of a terse metadata dump."
            .to_string();

        let collapsed = board_item_lines_for_doc(&doc, &theme, 96, false, false, false);
        let expanded = board_item_lines_for_doc(&doc, &theme, 96, false, true, false);
        let expanded_text = expanded
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(collapsed.len(), 1);
        assert!(expanded.len() > collapsed.len());
        assert!(expanded_text.contains("Tags: #tui #board"));
        assert!(!expanded_text.contains("[\"tui\""));
        assert!(expanded_text.contains("Summary"));
        assert!(expanded_text.contains("This expanded row should read"));
        assert!(expanded_text.contains("paragraph instead of a terse metadata dump"));
        assert!(expanded_text.contains("Files"));
        assert!(expanded_text.contains("• tandem/src/tui.rs"));
        assert!(expanded_text.contains("• tandem/src/tui/theme.rs"));
        assert!(expanded_text.contains("Checklist 1/2"));
        assert!(expanded_text.contains("[x] Keep tags clean"));
        assert!(expanded_text.contains("[ ] Add checklist preview"));
        assert!(!expanded_text.contains("updatedAt"));
    }

    #[test]
    fn board_row_expansion_preserves_markdown_body_lines_across_states() {
        let theme = TuiTheme::default_dark();
        let body = "## Body heading\n\nIntro paragraph.\n\n- first bullet\n- second bullet\n\n| Check | Result |\n| --- | --- |\n| preview | pass |";

        for state in ["todo", "in-progress", "validation"] {
            let mut doc = doc_with_state(&format!("task-{state}"), Some(state));
            doc.body = body.to_string();

            let expanded = board_item_lines_for_doc(&doc, &theme, 120, false, true, false);
            let expanded_text = expanded
                .iter()
                .map(line_text)
                .collect::<Vec<_>>()
                .join("\n");

            assert!(expanded.len() <= 1 + INLINE_PREVIEW_MAX_LINES);
            assert!(expanded_text.contains("Summary"));
            assert!(expanded_text.contains("   Body heading"));
            assert!(expanded_text.contains("   • first bullet"));
            assert!(expanded_text.contains("   • second bullet"));
            assert!(expanded_text.contains("   | Check | Result |"));
            assert!(expanded_text.contains("\n\n"));
            assert!(!expanded_text.contains("Intro paragraph. - first bullet"));
        }
    }

    #[test]
    fn validation_expanded_preview_prefers_delivery_summary_and_structured_accord_fields() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-77", Some("validation"));
        doc.fields
            .insert("accord.status".to_string(), "delivered".to_string());
        doc.fields.insert(
            "accord.summary".to_string(),
            "Implemented changes:\n\n- preserved bullets\n- kept table rows\n\n| Command | Result |\n| --- | --- |\n| cargo test | pass |".to_string(),
        );
        doc.fields.insert(
            "accord.validation.commands".to_string(),
            "[\"cargo test\"]".to_string(),
        );
        doc.fields.insert(
            "accord.evidence".to_string(),
            "[\"expanded preview test covers bullets and tables\"]".to_string(),
        );
        doc.fields.insert(
            "accord.filesChanged".to_string(),
            "[\"tandem/src/tui.rs\"]".to_string(),
        );
        doc.body = "Original task body belongs in the detail pane.".to_string();

        let expanded = board_item_lines_for_doc(&doc, &theme, 120, false, true, false);
        let expanded_text = expanded
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(expanded.len() <= 1 + INLINE_PREVIEW_MAX_LINES);
        assert!(expanded_text.contains("Delivery summary"));
        assert!(expanded_text.contains("   Implemented changes:"));
        assert!(expanded_text.contains("   • preserved bullets"));
        assert!(expanded_text.contains("   | Command | Result |"));
        assert!(expanded_text.contains("Validation"));
        assert!(expanded_text.contains("   • cargo test"));
        assert!(expanded_text.contains("Evidence"));
        assert!(expanded_text.contains("Files changed"));
        assert!(expanded_text.contains("   • tandem/src/tui.rs"));
        assert!(!expanded_text.contains("Original task body belongs"));
    }

    #[test]
    fn expanded_bottom_board_item_preview_is_capped_to_viewport_and_visible() {
        let mut app = keyboard_test_app();
        app.states = vec!["validation".to_string()];
        app.configured_states = app.states.clone();
        app.selected_state = 0;
        app.selected_item = 5;
        app.docs = (0..6)
            .map(|index| {
                let id = index + 1;
                let mut doc = doc_with_state(&format!("task-{id}"), Some("validation"));
                doc.fields
                    .insert("title".to_string(), format!("Validation task {index}"));
                doc.fields
                    .insert("accord.status".to_string(), "delivered".to_string());
                doc.fields.insert(
                    "accord.summary".to_string(),
                    "Review payload:\n\n- first visible bullet\n- second visible bullet\n\n| Check | Result |\n| --- | --- |\n| viewport | visible |".to_string(),
                );
                doc
            })
            .collect();
        app.expanded_board_doc_id = Some("task-6".to_string());

        let mut terminal = Terminal::new(TestBackend::new(100, 18)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(
            rendered.contains("Validation task 5"),
            "selected bottom row should render: {rendered}"
        );
        assert!(
            rendered.contains("Delivery summary"),
            "expanded preview should be scrolled into view: {rendered}"
        );
        assert!(
            rendered.contains("first visible bullet"),
            "expanded preview should show markdown-ish content: {rendered}"
        );
    }

    #[test]
    fn board_row_shows_type_only_when_mixed_or_non_default() {
        let theme = TuiTheme::default_dark();
        let mut task = doc_with_state("task-1", Some("todo"));
        task.fields
            .insert("title".to_string(), "Default work".to_string());
        task.fields
            .insert("priority".to_string(), "low".to_string());

        let default_context =
            line_text(&board_item_lines_for_doc(&task, &theme, 96, false, false, false)[0]);
        let mixed_context =
            line_text(&board_item_lines_for_doc(&task, &theme, 96, true, false, false)[0]);
        assert!(default_context.contains(" LOW   Default work"));
        assert!(!default_context.contains("task Default work"));
        assert!(mixed_context.contains("task"));
        assert!(mixed_context.contains(" LOW "));
        assert!(mixed_context.contains("Default work"));

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
        let non_default =
            line_text(&board_item_lines_for_doc(&decision, &theme, 96, false, false, false)[0]);
        assert!(non_default.contains("decision"));
        assert!(non_default.contains(" LOW "));
        assert!(non_default.contains("Choose layout"));
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
            "---\nprotocolVersion: 0.1.0\ntitle: Test Workspace\nstates: [todo, in-progress, validation]\n---\n",
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

    fn write_delivered_validation_task(workspace: &Workspace, id: &str) {
        fs::write(
            workspace.board_dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\ntitle: Delivered task\nstate: validation\naccord:\n  status: delivered\n  updatedAt: 2026-06-28T00:00:00Z\n  deliveredAt: 2026-06-28T00:00:00Z\n  summary: ready for sign-off\n---\n\nBody for {id}.\n"
            ),
        )
        .unwrap();
    }

    fn write_accepted_validation_task(workspace: &Workspace, id: &str, title: &str) {
        fs::write(
            workspace.board_dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\ntitle: {title}\nstate: validation\naccord:\n  status: accepted\nreview:\n  status: accepted\n---\n\nBody for {id}.\n"
            ),
        )
        .unwrap();
    }

    fn refresh_test_hierarchy(app: &mut TuiApp) {
        app.hierarchy = TuiHierarchySnapshot::from_documents(&app.docs, &app.logs);
    }

    fn keyboard_test_app() -> TuiApp {
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("validation")),
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
            states: vec!["todo".to_string(), "validation".to_string()],
            configured_states: vec!["todo".to_string(), "validation".to_string()],
            hierarchy: TuiHierarchySnapshot::from_documents(&docs, &[]),
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
            board_filters: BoardFilters::default(),
            board_arrangement: BoardArrangement::State,
            selected_log: 0,
            focus: FocusPane::Board,
            show_board_detail: false,
            expanded_board_doc_id: None,
            expanded_board_hierarchy_ids: BTreeSet::new(),
            detail_scroll: 0,
            review_detail_scroll: 0,
            log_detail_scroll: 0,
            log_search_filter: String::new(),
            log_search_input: None,
            status: String::new(),
            show_help: false,
            quick_add: None,
            validation_prompt: None,
            rules_view: RulesState::default(),
            decisions_view: DecisionsState::default(),
            hits: Vec::new(),
            reload_fingerprint: ReloadFingerprint::default(),
            last_reload_check: Instant::now(),
        }
    }

    #[test]
    fn states_include_unfiled_and_unknown_board_tasks_but_not_decisions() {
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("blocked")),
            doc_with_state("task-3", None),
            decision_doc("decision-1"),
        ];
        let states =
            states_with_board_docs(vec!["todo".to_string(), "validation".to_string()], &docs);
        assert_eq!(states, vec!["todo", "validation", "blocked", "unfiled"]);
    }

    #[test]
    fn document_without_state_uses_unfiled_label() {
        let doc = doc_with_state("task-3", None);
        assert_eq!(document_state_label(&doc), "unfiled");
    }

    #[test]
    fn numeric_keys_map_to_top_level_views() {
        assert_eq!(TuiView::from_digit('1'), Some(TuiView::Board));
        assert_eq!(TuiView::from_digit('2'), Some(TuiView::Logs));
        assert_eq!(TuiView::from_digit('3'), Some(TuiView::Rules));
        assert_eq!(TuiView::from_digit('4'), Some(TuiView::Decisions));
        assert_eq!(TuiView::from_digit('5'), None);
    }

    #[test]
    fn top_header_tabs_separate_shortcuts_labels_and_counts() {
        let app = keyboard_test_app();
        let line = line_text(&app.view_tab_line(96));
        assert!(line.contains("[1] Board (2)"));
        assert!(line.contains("[2] Logs (0)"));
        assert!(line.contains("[3] Rules (0)"));
        assert!(line.contains("[4] Decisions (1)"));
        assert!(!line.contains("1 Board 2"));
    }

    #[test]
    fn footer_hints_are_contextual_and_compact() {
        let mut app = keyboard_test_app();
        assert_eq!(
            app.board_footer_text(),
            "board · TODO · 1 row · Enter expand/preview · Space preview · a add · t tag · p priority · b Epic Board · ? help"
        );
        assert!(!app.board_footer_text().contains("1/"));
        assert!(!app.board_footer_text().contains("1..4"));

        app.focus = FocusPane::Detail;
        assert_eq!(
            app.board_footer_text(),
            "detail · TODO · 1 row · Tab board · j/k scroll · e edit · b Epic Board · ? help"
        );

        app.switch_view(TuiView::Logs);
        app.status.clear();
        assert_eq!(
            app.logs_footer_text(),
            "Logs list · Enter detail · / search · ? help"
        );

        app.switch_view(TuiView::Rules);
        app.status.clear();
        assert_eq!(
            app.rules_footer_text(),
            "Rules · h/l category · j/k select · n new · e edit · d delete · ? help"
        );
    }

    fn left_click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn scroll_down(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn mouse_hit_map_selects_rows_expands_selected_row_and_noops_elsewhere() {
        let mut app = keyboard_test_app();
        app.hits = vec![HitRegion {
            rect: Rect {
                x: 2,
                y: 4,
                width: 20,
                height: 1,
            },
            action: HitAction::SelectBoardItem(0, 0),
        }];

        assert_eq!(app.handle_mouse(left_click(90, 20)), KeyAction::Continue);
        assert_eq!(app.selected_item, 0);
        assert!(app.expanded_board_doc_id.is_none());

        assert_eq!(app.handle_mouse(left_click(3, 4)), KeyAction::Continue);
        assert_eq!(app.expanded_board_doc_id.as_deref(), Some("task-1"));

        assert_eq!(app.handle_mouse(left_click(3, 4)), KeyAction::Continue);
        assert!(app.expanded_board_doc_id.is_none());
    }

    #[test]
    fn mouse_row_hits_follow_scrolled_list_viewport_offsets() {
        let mut app = keyboard_test_app();
        app.states = vec!["todo".to_string()];
        app.configured_states = app.states.clone();
        app.docs = (0..20)
            .map(|index| doc_with_state(&format!("task-{}", index + 1), Some("todo")))
            .collect();
        app.selected_state = 0;
        app.selected_item = 19;

        let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let first_visible_hit = app
            .hits
            .iter()
            .filter_map(|hit| match hit.action {
                HitAction::SelectBoardItem(0, index) => Some((hit.rect.y, index, hit.rect)),
                _ => None,
            })
            .min_by_key(|(y, _, _)| *y)
            .expect("scrolled Board should register visible row hits");
        assert!(first_visible_hit.1 > 0);

        app.handle_mouse(left_click(first_visible_hit.2.x, first_visible_hit.2.y));
        assert_eq!(app.selected_item, first_visible_hit.1);
    }

    #[test]
    fn mouse_footer_action_hits_reuse_keyboard_paths() {
        let mut app = keyboard_test_app();
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        let add_hit = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::StartQuickAdd)
            .cloned()
            .expect("footer should register quick-add action");
        assert_eq!(
            app.handle_mouse(left_click(add_hit.rect.x, add_hit.rect.y)),
            KeyAction::Continue
        );
        assert!(app.quick_add.is_some());
    }

    #[test]
    fn mouse_wheel_scrolls_pane_under_pointer() {
        let mut app = keyboard_test_app();
        app.show_board_detail = true;
        app.hits = vec![HitRegion {
            rect: Rect {
                x: 0,
                y: 10,
                width: 80,
                height: 5,
            },
            action: HitAction::FocusDetail,
        }];

        assert_eq!(app.focus, FocusPane::Board);
        assert_eq!(app.handle_mouse(scroll_down(1, 11)), KeyAction::Continue);
        assert_eq!(app.focus, FocusPane::Detail);
        assert!(app.detail_scroll > 0);
    }

    #[test]
    fn footer_status_style_does_not_leak_into_hotkey_hints() {
        let mut app = keyboard_test_app();
        app.status = "Logs view active: 0 archived logs loaded.".to_string();
        let line = app.footer_line_for_text(app.logs_footer_text());

        assert_eq!(line_text(&line), app.logs_footer_text());
        assert_eq!(line.spans.len(), 3);
        assert!(line.spans[0].content.contains("Logs list"));
        assert_eq!(line.spans[0].style, app.theme.text_style());
        assert_eq!(line.spans[1].style, app.theme.muted_style());
        assert_eq!(
            line.spans[2].style,
            app.theme.status_style(status_tone_for_message(&app.status))
        );
    }

    #[test]
    fn help_popup_groups_current_commands_by_view() {
        let app = keyboard_test_app();
        let lines = app.help_lines();
        let text = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");

        for heading in [
            "Global",
            "Navigation",
            "Board",
            "Validation",
            "Logs",
            "Rules",
            "Decisions",
            "Prompts",
        ] {
            assert!(text.contains(heading), "missing help heading {heading}");
        }
        assert!(text.contains("1 2 3 4"));
        assert!(text.contains("b           toggle State/Epic Board arrangement"));
        assert!(!text.contains("E           toggle State/Epic Board arrangement"));
        assert!(text.contains("A           open accept confirmation"));
        assert!(text.contains("/           search id, title"));
        assert!(text.contains("e / d       edit or delete"));
        assert!(text.contains("use CLI decision update/withdraw; editor actions are deferred"));
        assert!(!text.contains("Review actions"));
    }

    #[test]
    fn numeric_keys_are_explicit_top_level_switchers() {
        let mut app = keyboard_test_app();
        app.handle_key(key(KeyCode::Char('2'))).unwrap();
        assert_eq!(app.view, TuiView::Logs);
        assert_eq!(app.focus, FocusPane::Board);

        app.handle_key(key(KeyCode::Char('1'))).unwrap();
        assert_eq!(app.view, TuiView::Board);
        assert_eq!(app.focus, FocusPane::Board);
    }

    #[test]
    fn board_arrangement_shortcut_uses_b_not_uppercase_e() {
        let mut app = keyboard_test_app();
        assert_eq!(app.board_arrangement, BoardArrangement::State);

        assert_eq!(
            app.handle_key(key(KeyCode::Char('E'))).unwrap(),
            KeyAction::Continue
        );
        assert_eq!(app.board_arrangement, BoardArrangement::State);
        assert_eq!(
            app.handle_key(key(KeyCode::Char('e'))).unwrap(),
            KeyAction::OpenEditor
        );

        assert_eq!(
            app.handle_key(key(KeyCode::Char('b'))).unwrap(),
            KeyAction::Continue
        );
        assert_eq!(app.board_arrangement, BoardArrangement::Epic);
        assert!(app.status.contains("Press b"));
        assert!(!app.status.contains("Press E"));
        assert!(app.board_footer_text().contains("b State Board"));
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
    fn editor_targets_active_tasks_from_board_only() {
        let mut app = keyboard_test_app();
        assert_eq!(app.selected_editor_target().unwrap().id, "task-1");
        assert!(!app.select_document_by_id("decision-1"));
        assert_eq!(app.selected_doc().map(Document::id), Some("task-1"));

        app.switch_view(TuiView::Decisions);
        let error = app.selected_editor_target().unwrap_err();
        assert!(error.contains("Decision document editing"));
    }

    #[test]
    fn validation_action_keys_open_signoff_prompts_and_deemphasize_complete() {
        let mut app = keyboard_test_app();
        app.selected_state = 1;
        app.docs[1]
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());

        app.handle_key(key(KeyCode::Char('A'))).unwrap();
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::Accept { ref id, .. }) if id == "task-2"
        ));
        assert!(app.status.contains("Confirm acceptance"));

        app.handle_key(key(KeyCode::Esc)).unwrap();
        app.handle_key(key(KeyCode::Char('R'))).unwrap();
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::Rework { ref id, .. }) if id == "task-2"
        ));
        assert!(app.status.contains("type feedback"));

        app.handle_key(key(KeyCode::Esc)).unwrap();
        app.handle_key(key(KeyCode::Char('C'))).unwrap();
        assert!(app.status.contains("No accepted Validation tasks"));
        assert!(!app.status.contains("tandem complete"));
    }

    #[test]
    fn rework_prompt_owns_hotkey_characters_as_text_input() {
        let mut app = keyboard_test_app();
        app.selected_state = 1;
        app.docs[1]
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());

        app.handle_key(key(KeyCode::Char('R'))).unwrap();
        for ch in ['n', 'a', 'e', '/'] {
            app.handle_key(key(KeyCode::Char(ch))).unwrap();
        }

        assert_eq!(app.view, TuiView::Board);
        assert!(app.quick_add.is_none());
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::Rework { ref feedback, .. }) if feedback == "nae/"
        ));
    }

    #[test]
    fn accept_confirmation_updates_accord_and_review_without_rework_feedback() {
        let root = unique_test_dir("tandem-validation-accept");
        let workspace = temp_workspace(&root);
        write_delivered_validation_task(&workspace, "task-1");

        let outcome = apply_validation_accept(&workspace, "task-1").unwrap();
        assert_eq!(outcome.state, "validation");
        let content = fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap();
        assert!(content.contains("status: \"accepted\""));
        assert!(content.contains("review.status: \"accepted\""));
        assert!(!content.contains("## Feedback"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rework_appends_feedback_and_moves_back_to_actionable_state() {
        let root = unique_test_dir("tandem-validation-rework");
        let workspace = temp_workspace(&root);
        write_delivered_validation_task(&workspace, "task-1");

        let outcome =
            apply_validation_rework(&workspace, "task-1", "Please fix the contrast.").unwrap();
        assert_eq!(outcome.state, "in-progress");
        let content = fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap();
        assert!(content.contains("state: \"in-progress\""));
        assert!(content.contains("status: \"rework\""));
        assert!(content.contains("review.status: \"changes-requested\""));
        assert!(content.contains("## Feedback"));
        assert!(content.contains("Please fix the contrast."));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rework_cancel_keeps_task_file_unchanged() {
        let root = unique_test_dir("tandem-validation-cancel");
        let workspace = temp_workspace(&root);
        write_delivered_validation_task(&workspace, "task-1");
        let before = fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap();
        let mut app = TuiApp::load(workspace.clone()).unwrap();
        assert!(app.select_document_by_id("task-1"));

        app.handle_key(key(KeyCode::Char('R'))).unwrap();
        app.handle_key(key(KeyCode::Char('x'))).unwrap();
        app.handle_key(key(KeyCode::Esc)).unwrap();

        let after = fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap();
        assert_eq!(after, before);
        assert!(app.validation_prompt.is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn apply_accepted_candidates_excludes_delivered_and_rework_items() {
        let mut accepted = doc_with_state("task-1", Some("validation"));
        accepted
            .fields
            .insert("accord.status".to_string(), "accepted".to_string());
        accepted
            .fields
            .insert("review.status".to_string(), "accepted".to_string());
        let mut delivered = doc_with_state("task-2", Some("validation"));
        delivered
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());
        let mut rework = doc_with_state("task-3", Some("in-progress"));
        rework
            .fields
            .insert("accord.status".to_string(), "rework".to_string());

        let candidates = accepted_validation_candidates(&[accepted, delivered, rework]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "task-1");
    }

    #[test]
    fn apply_accepted_cancel_keeps_task_files_unchanged() {
        let root = unique_test_dir("tandem-apply-cancel");
        let workspace = temp_workspace(&root);
        write_accepted_validation_task(&workspace, "task-1", "Accepted one");
        let before = fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap();
        let mut app = TuiApp::load(workspace.clone()).unwrap();
        app.selected_state = app
            .states
            .iter()
            .position(|state| state == "validation")
            .unwrap();

        app.handle_key(key(KeyCode::Char('C'))).unwrap();
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::ApplyAccepted { .. })
        ));
        app.handle_key(key(KeyCode::Esc)).unwrap();

        assert_eq!(
            fs::read_to_string(workspace.board_dir.join("task-1.md")).unwrap(),
            before
        );
        assert!(!workspace.logs_dir.join("task-1.md").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn apply_accepted_confirm_completes_only_accepted_candidates_to_logs() {
        let root = unique_test_dir("tandem-apply-confirm");
        let workspace = temp_workspace(&root);
        write_accepted_validation_task(&workspace, "task-1", "Accepted one");
        write_delivered_validation_task(&workspace, "task-2");
        let candidates = accepted_validation_candidates(
            &read_documents(&workspace.board_dir, DocumentLocation::Board).unwrap(),
        );

        let outcome = apply_accepted_validation_tasks(&workspace, &candidates).unwrap();

        assert_eq!(outcome.completed_ids, vec!["task-1"]);
        assert!(!workspace.board_dir.join("task-1.md").exists());
        assert!(workspace.board_dir.join("task-2.md").exists());
        let log = fs::read_to_string(workspace.logs_dir.join("task-1.md")).unwrap();
        assert!(log.contains("completedAt:"));
        assert!(log.contains("Applied accepted Validation sign-off for task-1"));
        assert!(log.contains("  reviewer: \"tui\""));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn graph_sensitive_tui_mutations_fail_closed_on_fresh_invalid_snapshot() {
        let root = unique_test_dir("tandem-mutation-hierarchy-lock");
        let workspace = temp_workspace(&root);
        fs::write(
            workspace.board_dir.join("task-10.md"),
            "---\nid: task-10\ntype: task\nkind: epic\ntitle: Epic\nstate: todo\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("task-10-1.md"),
            "---\nid: task-10-1\ntype: task\ntitle: Invalid Epic Task ID\nstate: todo\nparentId: task-10\n---\n",
        )
        .unwrap();

        let add_error = create_basic_task(&workspace, "Must not be created", "todo").unwrap_err();
        assert!(add_error.message.contains("expected global `task-N`"));
        assert!(!workspace.board_dir.join("task-11.md").exists());

        write_accepted_validation_task(&workspace, "task-20", "Accepted candidate");
        let candidates = vec![ValidationApplyCandidate {
            id: "task-20".to_string(),
            title: "Accepted candidate".to_string(),
        }];
        let apply_error = apply_accepted_validation_tasks(&workspace, &candidates).unwrap_err();
        assert!(apply_error.message.contains("expected global `task-N`"));
        assert!(workspace.board_dir.join("task-20.md").exists());
        assert!(!workspace.logs_dir.join("task-20.md").exists());
        fs::remove_dir_all(root).unwrap();
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
        app.reload();

        assert!(
            app.load_errors.is_empty(),
            "unexpected document reload warnings: {:?}",
            app.load_errors
        );
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
    fn board_detail_warns_about_state_accord_divergence() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-1", Some("todo"));
        doc.fields
            .insert("accord.status".to_string(), "claimed".to_string());

        let texts = detail_lines_for_doc(&doc, &theme)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();

        assert!(texts.iter().any(|text| text
            .contains("Warning: task-1 has workflow state `todo` but accord.status `claimed` suggests `in-progress`")));
    }

    #[test]
    fn board_detail_includes_accord_metadata_hints_and_preserves_body() {
        let mut doc = doc_with_state("task-1", Some("validation"));
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
        assert!(texts
            .iter()
            .any(|text| text.contains("Board Validation: A opens accept sign-off")));
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
        let mut decision = decision_doc("decision-1");
        decision
            .fields
            .insert("state".to_string(), "todo".to_string());
        let docs = vec![
            doc_with_state("task-1", Some("todo")),
            doc_with_state("task-2", Some("todo")),
            doc_with_state("task-3", Some("review")),
            decision,
        ];
        let states = vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
        ];
        let tabs = board_subview_tabs(&states, &docs, &BoardFilters::default());
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
