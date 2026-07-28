use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use yaml_rust2::Yaml;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app;
use crate::app::accord::ValidationApplyCandidate;
use crate::app::tasks::AddOptions;
use crate::project::rules::{empty_rules, parse_rules_from_yaml};
use crate::project::write::{file_signature, FileSignature, HierarchyLock};
use crate::project::{
    display_path, yaml_mapping_value, yaml_scalar_to_string, ProjectHierarchy as HierarchyIndex,
    StoredDocument as Document, TandemProject,
};
use crate::protocol::accord::{self, status as accord_status};
use crate::protocol::config::RulesByCategory;
use crate::protocol::document::parse_field_values;
use crate::protocol::hierarchy::{DocumentLocation, ParentRelationship, TaskRole};
use crate::protocol::review::status as review_status;
use crate::protocol::workflow::{
    self, completion_outcome, workflow_states, COMPLETION_OUTCOME_CANCELED,
    COMPLETION_OUTCOME_COMPLETED,
};
use crate::CliError;

mod board;
mod decisions;
mod editor;
mod input;
mod logs;
mod reload;
#[allow(dead_code)]
mod review;
mod rules;
mod terminal;
mod theme;
mod validation;

use board::*;
use decisions::DecisionsState;
use editor::{editor_command_from_env, editor_target_for_doc, run_editor_command, EditorTarget};
use reload::{ReloadFingerprint, ReloadOutcome};
use rules::RulesState;
use terminal::TerminalSession;
use theme::{StatusTone, TuiTheme};
use validation::ValidationPrompt;

pub(crate) fn run_tui(workspace: TandemProject) -> Result<(), CliError> {
    let mut app = TuiApp::load(workspace)?;
    let mut session = TerminalSession::enter()?;
    app.run(&mut session)
}

fn sort_documents(docs: &mut [Document]) {
    docs.sort_by(|a, b| {
        a.field("state")
            .unwrap_or("")
            .cmp(b.field("state").unwrap_or(""))
            .then_with(|| a.id().cmp(b.id()))
    });
}

fn is_canceled_log(doc: &Document) -> bool {
    doc.location == DocumentLocation::Logs && completion_outcome(doc) == COMPLETION_OUTCOME_CANCELED
}

fn truncate(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return value.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let mut truncated = chars[..max_chars.saturating_sub(1)]
        .iter()
        .collect::<String>();
    truncated.push('…');
    truncated
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

struct TuiApp {
    workspace: TandemProject,
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
    fn load(workspace: TandemProject) -> Result<Self, CliError> {
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

        match app::tasks::add(
            &self.workspace,
            AddOptions {
                title: Some(title.clone()),
                state: Some(state.clone()),
                ..AddOptions::default()
            },
        ) {
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
        match app::tasks::move_to_state(&self.workspace, doc_id, target_state) {
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

fn workspace_title_from_root(root: Option<&Yaml>) -> Option<String> {
    root.and_then(|root| yaml_mapping_value(root, "title"))
        .and_then(yaml_scalar_to_string)
        .filter(|title| !title.trim().is_empty())
}

fn workspace_states_from_root(root: Option<&Yaml>) -> Vec<String> {
    workflow_states(root)
}

fn default_workspace_states() -> Vec<String> {
    workflow::DEFAULT_STATES
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
        let mut errors = crate::protocol::diagnostic::metadata_diagnostics(
            doc,
            doc.location == DocumentLocation::Logs,
        )
        .into_iter()
        .filter(|diagnostic| diagnostic.severity == crate::protocol::diagnostic::Severity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
        if let Some(diagnostic) = crate::protocol::diagnostic::workflow_state_diagnostic(
            doc,
            doc.location == DocumentLocation::Board && doc.doc_type() == "task",
            configured_states,
        ) {
            errors.push(diagnostic.message);
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

fn collect_reload_fingerprint(workspace: &TandemProject) -> ReloadFingerprint {
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
    insert_directory_fingerprints(&mut files, &workspace.events_dir(), "jsonl");
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
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use ratatui::{backend::TestBackend, Terminal};

    use super::*;
    use crate::project::read_documents;

    fn doc_with_state(id: &str, state: Option<&str>) -> Document {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), id.to_string());
        fields.insert("type".to_string(), "task".to_string());
        fields.insert("title".to_string(), format!("Task {id}"));
        if let Some(state) = state {
            fields.insert("state".to_string(), state.to_string());
        }
        Document::new(
            PathBuf::from(format!("{id}.md")),
            DocumentLocation::Board,
            fields,
            String::new(),
        )
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
            vec![(
                chip_text("TODO", &theme),
                theme.progress_chip_style(StatusTone::Muted),
            )],
            (23, false, "task-102".to_string(), 1, true),
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
                (100, true, INLINE_PREVIEW_MAX_LINES, false, false),
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
            (42, false, 10, false, false),
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
            (100, false, 0, false, false),
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
            (100, false, 0, false, false),
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
                (100, false, 10, false, false),
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
                    (100, false, 10, false, false),
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

    fn temp_workspace(root: &Path) -> TandemProject {
        let tandem_dir = root.join(".tandem");
        let workspace = TandemProject {
            root: PathBuf::new(),
            data_dir: PathBuf::new(),
            board_dir: tandem_dir.join("board"),
            logs_dir: tandem_dir.join("logs"),
            config_path: tandem_dir.join("tandem.md"),
            events_path: tandem_dir.join("events.jsonl"),
        };
        fs::create_dir_all(&workspace.board_dir).unwrap();
        fs::create_dir_all(&workspace.logs_dir).unwrap();
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.1.0\ntitle: Test TandemProject\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        workspace
    }

    fn write_task_doc(workspace: &TandemProject, id: &str, title: &str, state: &str) {
        fs::write(
            workspace.board_dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\ntitle: {title}\nstate: {state}\n---\n\nBody for {id}.\n"
            ),
        )
        .unwrap();
    }

    fn write_delivered_validation_task(workspace: &TandemProject, id: &str) {
        fs::write(
            workspace.board_dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\ntitle: Delivered task\nstate: validation\naccord:\n  status: delivered\n  updatedAt: 2026-06-28T00:00:00Z\n  deliveredAt: 2026-06-28T00:00:00Z\n  summary: ready for sign-off\n---\n\nBody for {id}.\n"
            ),
        )
        .unwrap();
    }

    fn write_accepted_validation_task(workspace: &TandemProject, id: &str, title: &str) {
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
            workspace: TandemProject {
                root: PathBuf::new(),
                data_dir: PathBuf::new(),
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

        let outcome = app::accord::accept_validation(&workspace, "task-1", "tui").unwrap();
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

        let outcome = app::accord::request_validation_rework(
            &workspace,
            "task-1",
            "tui",
            "Please fix the contrast.",
        )
        .unwrap();
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

        let candidates =
            app::accord::accepted_validation_candidates(&[accepted, delivered, rework]);

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
        let candidates = app::accord::accepted_validation_candidates(
            &read_documents(&workspace.board_dir, DocumentLocation::Board).unwrap(),
        );

        let outcome =
            app::accord::apply_accepted_validation(&workspace, &candidates, "tui").unwrap();

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

        let add_error = app::tasks::add(
            &workspace,
            AddOptions {
                title: Some("Must not be created".to_string()),
                state: Some("todo".to_string()),
                ..AddOptions::default()
            },
        )
        .unwrap_err();
        assert!(add_error.message.contains("expected global `task-N`"));
        assert!(!workspace.board_dir.join("task-11.md").exists());

        write_accepted_validation_task(&workspace, "task-20", "Accepted candidate");
        let candidates = vec![ValidationApplyCandidate {
            id: "task-20".to_string(),
            title: "Accepted candidate".to_string(),
        }];
        let apply_error =
            app::accord::apply_accepted_validation(&workspace, &candidates, "tui").unwrap_err();
        assert!(apply_error.message.contains("expected global `task-N`"));
        assert!(workspace.board_dir.join("task-20.md").exists());
        assert!(!workspace.logs_dir.join("task-20.md").exists());
        fs::remove_dir_all(root).unwrap();
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

    #[test]
    fn tui_load_surfaces_workspace_compatibility_warnings() {
        let root = unique_test_dir("tandem-tui-compatibility");
        let workspace = temp_workspace(&root);
        fs::write(
            &workspace.config_path,
            "---\nprotocolVersion: 0.2.0\ntitle: Test TandemProject\nstates: [todo, in-progress, validation]\ntypes:\n  note:\n    idPrefix: note\ncompletion:\n  requireReview: true\n---\n",
        )
        .unwrap();
        fs::write(
            workspace.board_dir.join("note-1.md"),
            "---\nid: note-1\ntype: note\ntitle: Legacy note\nstate: todo\neffort: xlarge\n---\n\nLegacy body.\n",
        )
        .unwrap();

        let app = TuiApp::load(workspace).unwrap();
        let warnings = app.runtime_warnings();
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("custom type declarations are deprecated")));
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("completion-policy settings are deprecated")));
        assert_eq!(app.docs[0].id(), "note-1");
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
        doc.fields.insert("effort".to_string(), "small".to_string());
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
        assert!(texts.contains(&"Effort: small".to_string()));
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
