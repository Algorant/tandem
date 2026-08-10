//! Peer terminal interface over shared application and protocol behavior.
//!
//! This module is the TUI wiring root: it composes terminal lifecycle,
//! transient state, input, reload, projection, feature, and rendering modules.
//! Durable mutations route through `app`; protocol inference and concrete
//! project-file safety remain outside the interface.

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

mod bindings;
mod board;
mod chrome;
mod decisions;
mod editor;
mod input;
mod logs;
mod papercuts;
mod pickers;
mod reload;
#[allow(
    dead_code,
    reason = "retained Review implementation is intentionally compiled pending a separate product decision"
)]
mod review;
mod rules;
mod state;
mod terminal;
mod text;
mod theme;
mod validation;

use bindings::{bindings_for, BindingScope};
use board::*;
#[cfg(test)]
use chrome::status_tone_for_message;
use chrome::{centered_rect, rect_contains};
use decisions::DecisionsState;
use editor::{editor_command_from_env, editor_target_for_doc, run_editor_command, EditorTarget};
use papercuts::PapercutsState;
use pickers::BoardPicker;
use reload::{ReloadFingerprint, ReloadOutcome};
use rules::RulesState;
use state::*;
use terminal::TerminalSession;
use text::markdownish_lines;
use theme::{default_tag_tone, StatusTone, TuiTheme};
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
#[allow(
    dead_code,
    reason = "Review hit actions remain available with the retained Review implementation"
)]
enum HitAction {
    SwitchView(TuiView),
    SelectState(usize),
    SelectBoardItem(usize, usize),
    ToggleBoardExpansion,
    ToggleBoardDetail,
    ToggleBoardArrangement,
    StartQuickAdd,
    OpenFilterPicker,
    OpenMovePicker,
    OpenValidationPicker,
    SelectPickerOption(usize),
    ActivatePicker,
    CancelPicker,
    SelectDecision(usize),
    FocusDecisionList,
    FocusDecisionDetail,
    ToggleRulePreview,
    FocusRuleList,
    FocusRulePreview,
    ConfirmModal,
    CancelModal,
    HelpSection(usize),
    CloseHelp,
    OpenEditor,
    ShowHelp,
    FocusDetail,
    FocusReviewList,
    SelectReviewItem(usize),
    FocusReviewDetail,
    SelectLog(usize),
    SelectRuleCategory(usize),
    SelectRuleItem(usize),
    FocusLogList,
    FocusLogDetail,
    StartLogSearch,
    ToggleFocus,
    TogglePapercuts,
    FocusPapercutList,
    SelectPapercut(usize),
    FocusPapercutDetail,
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
    help_scroll: u16,
    help_section: usize,
    quick_add: Option<QuickAddInput>,
    board_picker: Option<BoardPicker>,
    validation_prompt: Option<ValidationPrompt>,
    rules_view: RulesState,
    decisions_view: DecisionsState,
    papercuts_view: PapercutsState,
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
            papercuts_view: PapercutsState::default(),
            hits: Vec::new(),
            reload_fingerprint: ReloadFingerprint::default(),
            last_reload_check: Instant::now(),
            help_scroll: 0,
            help_section: 0,
            board_picker: None,
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

        if self.papercuts_open() {
            self.draw_papercuts_panel(frame, chunks[1]);
        }

        if self.board_picker.is_some() {
            self.draw_board_picker(frame, area);
        }

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
    fn board_row_renders_default_bug_feat_and_chore_badges_once() {
        let theme = TuiTheme::default_dark();
        let mut doc = doc_with_state("task-30", Some("todo"));
        doc.fields.insert(
            "title".to_string(),
            "Render common repository work tags".to_string(),
        );
        doc.fields.insert(
            "tags".to_string(),
            "[\"bug\", \"BUG\", \"feat\", \"chore\"]".to_string(),
        );

        let line = board_item_lines_for_doc(&doc, &theme, 140, false, false, false)[0].clone();
        let title = line_text(&line);
        assert_eq!(title.matches(" BUG ").count(), 1, "rendered row: {title}");
        assert_eq!(title.matches(" FEAT ").count(), 1, "rendered row: {title}");
        assert_eq!(title.matches(" CHORE ").count(), 1, "rendered row: {title}");
        for (label, tone) in [
            ("BUG", StatusTone::Orange),
            ("FEAT", StatusTone::Sand),
            ("CHORE", StatusTone::Purple),
        ] {
            assert!(
                line.spans.iter().any(|span| {
                    span.content.trim() == label && span.style == theme.progress_chip_style(tone)
                }),
                "missing {label} style in spans: {:?}",
                line.spans
            );
        }
    }

    #[test]
    fn configured_work_tag_overrides_and_disabled_badges_apply_to_built_ins() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_display_content(
            r#"
[board.badges]
disabled = ["feat"]

[board.badges.tags.bug]
label = "FIX"

[board.badges.tags.chore]
tone = "success"
"#,
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

        let mut doc = doc_with_state("task-31", Some("todo"));
        doc.fields
            .insert("title".to_string(), "Override work badges".to_string());
        doc.fields.insert(
            "tags".to_string(),
            "[\"bug\", \"feat\", \"chore\"]".to_string(),
        );

        let line = board_item_lines_for_doc(&doc, &theme, 140, false, false, false)[0].clone();
        let title = line_text(&line);
        assert!(title.contains(" FIX "), "rendered row: {title}");
        assert!(!title.contains(" BUG "), "rendered row: {title}");
        assert!(!title.contains(" FEAT "), "rendered row: {title}");
        assert!(title.contains(" CHORE "), "rendered row: {title}");
        assert!(line.spans.iter().any(|span| {
            span.content.trim() == "FIX"
                && span.style == theme.progress_chip_style(StatusTone::Orange)
        }));
        assert!(line.spans.iter().any(|span| {
            span.content.trim() == "CHORE"
                && span.style == theme.progress_chip_style(StatusTone::Success)
        }));
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

        app.handle_key(key(KeyCode::Char('f'))).unwrap();
        assert!(matches!(
            app.board_picker.as_ref().map(|picker| picker.kind),
            Some(pickers::PickerKind::Filter)
        ));
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.board_filters.tag.as_deref(), Some("research"));
        assert_eq!(app.selected_state_count(), 1);

        app.handle_key(key(KeyCode::Char('f'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.board_filters.priority.as_deref(), Some("high"));
        assert_eq!(app.selected_state_count(), 1);

        for removed in ['t', 'p', 'F', 'H', 'L', 'A', 'R', 'C', 'P', 'n'] {
            app.handle_key(key(KeyCode::Char(removed))).unwrap();
        }
        assert_eq!(app.board_filters.tag.as_deref(), Some("research"));
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
        assert!(footer.contains("f filter"));
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
        assert!(rendered.contains("f change or clear"));
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

    fn papercut_item(id: &str, title: &str, body: &str) -> crate::project::StoredPapercut {
        crate::project::StoredPapercut::new(
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

    fn write_papercut(workspace: &TandemProject, id: &str, title: &str, body: &str) {
        fs::create_dir_all(workspace.papercuts_dir()).unwrap();
        fs::write(
            workspace.papercuts_dir().join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntitle: {title}\nstatus: open\ncreatedAt: 2026-08-10T00:00:00Z\nupdatedAt: 2026-08-10T01:00:00Z\ntags: [tui, friction]\nreferences: [task-1]\n---\n{body}\n"
            ),
        )
        .unwrap();
    }

    fn terminal_text(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
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
            help_scroll: 0,
            help_section: 0,
            quick_add: None,
            board_picker: None,
            validation_prompt: None,
            rules_view: RulesState::default(),
            decisions_view: DecisionsState::default(),
            papercuts_view: PapercutsState::default(),
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
            "board · TODO · 1 row · Enter open · Space preview · a add · f filter · m move · v validate · b Epic Board · ? help"
        );
        assert!(!app.board_footer_text().contains("1/"));
        assert!(!app.board_footer_text().contains("1..4"));

        app.focus = FocusPane::Detail;
        assert_eq!(
            app.board_footer_text(),
            "detail · TODO · 1 row · Shift-Tab board · j/k scroll · e edit · b Epic Board · ? help"
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
            "Rules list · h/l category · j/k select · Enter preview · a add · e edit · d delete · ? help"
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
    fn papercuts_shortcut_opens_from_each_main_view_without_changing_main_state() {
        let mut app = keyboard_test_app();
        app.selected_item = 0;
        app.detail_scroll = 3;
        app.board_arrangement = BoardArrangement::Epic;
        app.load_papercuts(crate::app::papercuts::InboxLoad {
            items: vec![papercut_item("papercut-1", "First", "Body")],
            warnings: Vec::new(),
        });

        for view in TuiView::ALL {
            app.view = view;
            app.focus = FocusPane::Detail;
            let selected_item = app.selected_item;
            let detail_scroll = app.detail_scroll;
            let arrangement = app.board_arrangement;

            app.handle_key(key(KeyCode::Char('i'))).unwrap();
            assert!(app.papercuts_open());
            assert_eq!(app.view, view);
            assert_eq!(app.focus, FocusPane::Detail);
            app.handle_key(key(KeyCode::Char('i'))).unwrap();
            assert!(!app.papercuts_open());
            assert_eq!(app.view, view);
            assert_eq!(app.focus, FocusPane::Detail);
            assert_eq!(app.selected_item, selected_item);
            assert_eq!(app.detail_scroll, detail_scroll);
            assert_eq!(app.board_arrangement, arrangement);
        }
    }

    #[test]
    fn papercut_header_count_and_hit_target_are_global() {
        let mut app = keyboard_test_app();
        app.load_papercuts(crate::app::papercuts::InboxLoad {
            items: vec![
                papercut_item("papercut-1", "First", "Body"),
                papercut_item("papercut-2", "Second", "Body"),
                papercut_item("papercut-3", "Third", "Body"),
            ],
            warnings: Vec::new(),
        });
        let mut terminal = Terminal::new(TestBackend::new(120, 28)).unwrap();

        for view in TuiView::ALL {
            app.view = view;
            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert!(terminal_text(&terminal).contains("Papercuts 3"));
            let hit = app
                .hits
                .iter()
                .find(|hit| hit.action == HitAction::TogglePapercuts)
                .cloned()
                .expect("global header should expose a Papercuts hit target");
            app.handle_mouse(left_click(hit.rect.x, hit.rect.y));
            assert!(app.papercuts_open());
            app.handle_key(key(KeyCode::Char('i'))).unwrap();
        }
    }

    #[test]
    fn empty_papercut_indicator_is_muted_and_reports_zero() {
        let app = keyboard_test_app();
        let line = app.papercut_indicator_line();
        assert_eq!(line_text(&line), "Papercuts 0");
        assert_eq!(line.spans[0].style, app.theme.muted_style());
    }

    #[test]
    fn papercut_panel_renders_metadata_and_supports_keyboard_and_mouse_navigation() {
        let mut app = keyboard_test_app();
        let long_body = (0..30)
            .map(|index| format!("- body line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        app.load_papercuts(crate::app::papercuts::InboxLoad {
            items: vec![
                papercut_item("papercut-1", "First", "Body"),
                papercut_item("papercut-2", "Second", &long_body),
                papercut_item("papercut-3", "Third", "Body"),
            ],
            warnings: vec!["Papercuts load warning: malformed record".to_string()],
        });
        app.toggle_papercuts();
        let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        let second = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::SelectPapercut(1))
            .cloned()
            .expect("visible Papercut row should have a mouse target");
        app.handle_mouse(left_click(second.rect.x, second.rect.y));
        assert_eq!(
            app.selected_papercut_id_for_reload().as_deref(),
            Some("papercut-2")
        );
        app.handle_key(key(KeyCode::Enter)).unwrap();
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
        assert!(app.papercuts_view.detail_scroll > 0);

        terminal.draw(|frame| app.draw(frame)).unwrap();
        let text = terminal_text(&terminal);
        assert!(text.contains("Detail papercut-2"));
        assert!(text.contains("Status: open"));
        assert!(text.contains("Tags: tui, friction"));
        assert!(text.contains("1 load warning"));

        let detail = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::FocusPapercutDetail)
            .cloned()
            .expect("detail pane should have a mouse target");
        let before = app.papercuts_view.detail_scroll;
        app.handle_mouse(scroll_down(detail.rect.x, detail.rect.y));
        assert!(app.papercuts_view.detail_scroll > before);
    }

    #[test]
    fn papercut_mouse_rows_follow_the_scrolled_list_offset() {
        let mut app = keyboard_test_app();
        app.load_papercuts(crate::app::papercuts::InboxLoad {
            items: (1..=20)
                .map(|index| {
                    papercut_item(
                        &format!("papercut-{index}"),
                        &format!("Friction {index}"),
                        "Body",
                    )
                })
                .collect(),
            warnings: Vec::new(),
        });
        app.papercuts_view.selected = 19;
        app.toggle_papercuts();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        let first_visible = app
            .hits
            .iter()
            .filter_map(|hit| match hit.action {
                HitAction::SelectPapercut(index) => Some((index, hit.rect)),
                _ => None,
            })
            .min_by_key(|(_, rect)| rect.y)
            .expect("scrolled Papercut list should expose visible row hits");
        assert!(first_visible.0 > 0);
        app.handle_mouse(left_click(first_visible.1.x, first_visible.1.y));
        assert_eq!(app.papercuts_view.selected, first_visible.0);
    }

    #[test]
    fn papercut_reload_is_tolerant_updates_count_and_preserves_selection() {
        let root = unique_test_dir("tandem-tui-papercuts-reload");
        let workspace = temp_workspace(&root);
        write_task_doc(&workspace, "task-1", "Board remains available", "todo");
        write_papercut(&workspace, "papercut-1", "First", "Body one");
        write_papercut(&workspace, "papercut-2", "Second", "Body two");
        fs::write(
            workspace.papercuts_dir().join("papercut-3.md"),
            "not frontmatter",
        )
        .unwrap();
        let papercut_path = workspace.papercuts_dir().join("papercut-2.md");
        let source_before = fs::read_to_string(&papercut_path).unwrap();
        let mut app = TuiApp::load(workspace.clone()).unwrap();
        assert_eq!(app.papercut_count(), 2);
        assert_eq!(app.docs.len(), 1);
        assert_eq!(app.papercut_warnings().len(), 1);
        app.papercuts_view.selected = 1;
        app.toggle_papercuts();

        write_papercut(&workspace, "papercut-4", "Fourth", "Body four");
        app.handle_key(key(KeyCode::Char('r'))).unwrap();
        assert!(app.papercuts_open());
        assert_eq!(app.papercut_count(), 3);
        assert_eq!(
            app.selected_papercut_id_for_reload().as_deref(),
            Some("papercut-2")
        );
        assert_eq!(app.docs.len(), 1);

        write_papercut(&workspace, "papercut-5", "Fifth", "Body five");
        app.last_reload_check = Instant::now() - Duration::from_secs(1);
        app.reload_if_changed();
        assert_eq!(app.papercut_count(), 4);
        assert_eq!(
            app.selected_papercut_id_for_reload().as_deref(),
            Some("papercut-2")
        );

        app.handle_papercuts_key(key(KeyCode::Char('j')));
        app.handle_key(key(KeyCode::Esc)).unwrap();
        assert!(!app.papercuts_open());
        assert_eq!(fs::read_to_string(papercut_path).unwrap(), source_before);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rules_mouse_hits_select_categories_and_dense_rows() {
        let mut app = keyboard_test_app();
        app.view = TuiView::Rules;
        app.rules.get_mut("prefer").unwrap().extend([
            crate::protocol::config::RuleItem {
                id: 1,
                rule: "First rule".to_string(),
                source: None,
            },
            crate::protocol::config::RuleItem {
                id: 2,
                rule: "Second wrapped rule".to_string(),
                source: Some("decision-10".to_string()),
            },
        ]);
        app.hits = vec![
            HitRegion {
                rect: Rect::new(2, 2, 12, 1),
                action: HitAction::SelectRuleCategory(2),
            },
            HitRegion {
                rect: Rect::new(2, 7, 30, 1),
                action: HitAction::SelectRuleItem(1),
            },
        ];

        app.handle_mouse(left_click(3, 2));
        assert_eq!(app.rules_view.selected_category, 2);
        assert_eq!(app.rules_view.selected_item, 0);
        app.handle_mouse(left_click(4, 7));
        assert_eq!(app.rules_view.selected_item, 1);
    }

    #[test]
    fn rules_enter_toggles_preview_and_selection_follows_without_closing_it() {
        let mut app = keyboard_test_app();
        app.view = TuiView::Rules;
        app.rules.get_mut("always").unwrap().extend([
            crate::protocol::config::RuleItem {
                id: 1,
                rule: "First rule".to_string(),
                source: None,
            },
            crate::protocol::config::RuleItem {
                id: 2,
                rule: "Second rule".to_string(),
                source: Some("decision-10".to_string()),
            },
        ]);

        app.handle_rules_key(KeyEvent::from(KeyCode::Enter));
        assert!(app.rules_view.preview_open);
        app.handle_rules_key(KeyEvent::from(KeyCode::Char('j')));
        assert_eq!(app.rules_view.selected_item, 1);
        assert!(app.rules_view.preview_open);
        app.handle_rules_key(KeyEvent::from(KeyCode::Enter));
        assert!(!app.rules_view.preview_open);
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
            "Current view",
            "Board actions",
            "Validation",
            "Logs",
            "Rules",
            "Decisions",
            "Utility inbox",
            "Dialogs and text input",
            "Mouse",
        ] {
            assert!(text.contains(heading), "missing help heading {heading}");
        }
        for keys in ["1–4", "f", "m", "v", "Ctrl-U/D · PgUp/PgDn", "a/e/d", "/"] {
            assert!(text.contains(keys), "missing help binding {keys}");
        }
        for removed in ["t/p/F", "H/L", "A/R/C", "a or n"] {
            assert!(!text.contains(removed), "stale help binding {removed}");
        }
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
    fn rules_tab_and_shift_tab_change_preview_focus_only() {
        let mut app = keyboard_test_app();
        app.rules
            .get_mut("always")
            .unwrap()
            .push(crate::protocol::config::RuleItem {
                id: 1,
                rule: "A long preview".into(),
                source: None,
            });
        app.switch_view(TuiView::Rules);
        app.handle_key(key(KeyCode::Enter)).unwrap();

        app.handle_key(key(KeyCode::Tab)).unwrap();
        assert_eq!(app.view, TuiView::Rules);
        assert_eq!(app.rules_view.focus, rules::RuleFocus::Preview);

        app.handle_key(key(KeyCode::BackTab)).unwrap();
        assert_eq!(app.view, TuiView::Rules);
        assert_eq!(app.rules_view.focus, rules::RuleFocus::List);
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
        assert!(app.status.contains("CLI"));
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

        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::Accept { ref id, .. }) if id == "task-2"
        ));
        assert!(app.status.contains("Confirm acceptance"));

        app.handle_key(key(KeyCode::Esc)).unwrap();
        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(matches!(
            app.validation_prompt,
            Some(ValidationPrompt::Rework { ref id, .. }) if id == "task-2"
        ));
        assert!(app.status.contains("type feedback"));

        app.handle_key(key(KeyCode::Esc)).unwrap();
        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert!(app.status.contains("no accepted tasks"));
        assert!(!app.status.contains("tandem complete"));
    }

    #[test]
    fn rework_prompt_owns_hotkey_characters_as_text_input() {
        let mut app = keyboard_test_app();
        app.selected_state = 1;
        app.docs[1]
            .fields
            .insert("accord.status".to_string(), "delivered".to_string());

        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
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

        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
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

        app.handle_key(key(KeyCode::Char('v'))).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
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
    fn fixed_keymap_precedence_and_removed_aliases_are_safe() {
        let mut app = keyboard_test_app();
        for removed in ['P', 't', 'p', 'F', 'H', 'L', 'A', 'R', 'C', 'n', 'u', 'd'] {
            assert_eq!(
                app.handle_key(key(KeyCode::Char(removed))).unwrap(),
                KeyAction::Continue
            );
            assert!(app.board_picker.is_none());
            assert!(!app.papercuts_open());
        }
        app.handle_key(key(KeyCode::Char('a'))).unwrap();
        for printable in ['q', '?', 'i'] {
            app.handle_key(key(KeyCode::Char(printable))).unwrap();
        }
        assert_eq!(
            app.quick_add.as_ref().map(|input| input.title.as_str()),
            Some("q?i")
        );
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.hits.iter().any(|hit| hit.action == HitAction::ShowHelp));
        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
                .unwrap(),
            KeyAction::Quit
        );
    }

    #[test]
    fn help_preserves_picker_and_q_quits_non_text_context() {
        let mut app = keyboard_test_app();
        app.handle_key(key(KeyCode::Char('f'))).unwrap();
        app.handle_key(key(KeyCode::Char('?'))).unwrap();
        assert!(app.show_help);
        assert!(app.board_picker.is_some());
        app.handle_key(key(KeyCode::Esc)).unwrap();
        assert!(!app.show_help);
        assert!(app.board_picker.is_some());
        app.handle_key(key(KeyCode::Char('?'))).unwrap();
        assert_eq!(
            app.handle_key(key(KeyCode::Char('q'))).unwrap(),
            KeyAction::Quit
        );
    }

    #[test]
    fn control_and_page_keys_page_while_plain_u_d_do_not() {
        let mut app = keyboard_test_app();
        for index in 3..12 {
            app.docs
                .push(doc_with_state(&format!("task-{index}"), Some("todo")));
        }
        refresh_test_hierarchy(&mut app);
        app.handle_key(key(KeyCode::Char('d'))).unwrap();
        assert_eq!(app.selected_item, 0);
        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .unwrap();
        assert_eq!(app.selected_item, 5);
        app.handle_key(key(KeyCode::PageDown)).unwrap();
        assert!(app.selected_item > 5);
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL))
            .unwrap();
        assert!(app.selected_item < 10);
    }

    #[test]
    fn picker_mouse_controls_are_bounded_and_non_mutating_until_apply() {
        let mut app = keyboard_test_app();
        app.docs[0]
            .fields
            .insert("tags".into(), "[\"research\"]".into());
        app.start_filter_picker();
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        let row = app
            .hits
            .iter()
            .find(|hit| matches!(hit.action, HitAction::SelectPickerOption(_)))
            .unwrap()
            .clone();
        app.handle_mouse(left_click(row.rect.x, row.rect.y));
        assert!(app.board_picker.is_some());
        assert_eq!(app.board_filters, BoardFilters::default());

        terminal.draw(|frame| app.draw(frame)).unwrap();
        let apply = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::ActivatePicker)
            .unwrap()
            .clone();
        let cancel = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::CancelPicker)
            .unwrap()
            .clone();
        assert!(apply.rect.right() <= cancel.rect.x || cancel.rect.right() <= apply.rect.x);
        app.handle_mouse(left_click(cancel.rect.x, cancel.rect.y));
        assert!(app.board_picker.is_none());
        assert_eq!(app.board_filters, BoardFilters::default());

        app.start_filter_picker();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let help = app
            .hits
            .iter()
            .rev()
            .find(|hit| hit.action == HitAction::ShowHelp)
            .unwrap()
            .clone();
        let apply = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::ActivatePicker)
            .unwrap();
        assert!(apply.rect.right() <= help.rect.x || help.rect.right() <= apply.rect.x);
        app.handle_mouse(left_click(help.rect.x, help.rect.y));
        assert!(app.show_help);
        assert!(app.board_picker.is_some());
        assert_eq!(app.board_filters, BoardFilters::default());
    }

    #[test]
    fn pickers_start_on_the_first_enabled_option() {
        let mut app = keyboard_test_app();
        app.docs[0]
            .fields
            .insert("tags".into(), "[\"research\"]".into());
        app.docs[1]
            .fields
            .insert("tags".into(), "[\"spike\"]".into());
        app.board_filters.tag = Some("research".into());
        app.start_filter_picker();
        assert_eq!(app.board_picker.as_ref().unwrap().selected, 1);

        app.board_picker = None;
        app.board_filters = BoardFilters::default();
        app.selected_state = 0;
        app.start_move_picker();
        assert_eq!(app.board_picker.as_ref().unwrap().selected, 1);

        app.board_picker = None;
        app.selected_state = 1;
        app.docs[1]
            .fields
            .insert("accord.status".into(), "accepted".into());
        app.docs[1]
            .fields
            .insert("review.status".into(), "accepted".into());
        app.start_validation_picker();
        assert_eq!(app.board_picker.as_ref().unwrap().selected, 2);
    }

    #[test]
    fn scrolled_picker_mouse_hits_follow_visible_options() {
        let mut app = keyboard_test_app();
        for (index, tag) in [
            "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
            "juliet", "kilo", "lima",
        ]
        .into_iter()
        .enumerate()
        {
            app.docs.push(doc_with_state(
                &format!("task-{}", index + 20),
                Some("todo"),
            ));
            app.docs
                .last_mut()
                .unwrap()
                .fields
                .insert("tags".into(), format!("[\"{tag}\"]"));
        }
        app.start_filter_picker();
        app.handle_picker_key(key(KeyCode::End));
        let mut terminal = Terminal::new(TestBackend::new(80, 18)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app
            .hits
            .iter()
            .any(|hit| matches!(hit.action, HitAction::SelectPickerOption(index) if index > 9)));
    }

    #[test]
    fn papercuts_enter_opens_detail_without_returning_to_list() {
        let mut app = keyboard_test_app();
        app.papercuts_view
            .set_items(vec![papercut_item("papercut-1", "One", "Body")]);
        app.toggle_papercuts();
        app.handle_papercuts_key(key(KeyCode::Enter));
        assert!(app.papercuts_footer_text().contains("Shift-Tab list"));
        app.handle_papercuts_key(key(KeyCode::Enter));
        assert!(app.papercuts_footer_text().contains("Shift-Tab list"));
        app.handle_papercuts_key(key(KeyCode::BackTab));
        assert!(app.papercuts_footer_text().contains("Enter open detail"));
    }

    #[test]
    fn rules_preview_scrolls_with_focused_keys_and_pointer_pane() {
        let mut app = keyboard_test_app();
        app.switch_view(TuiView::Rules);
        app.rules
            .get_mut("always")
            .unwrap()
            .push(crate::protocol::config::RuleItem {
                id: 1,
                rule: (0..80)
                    .map(|index| format!("long rule word {index}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                source: None,
            });
        app.handle_rules_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Tab)).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .unwrap();
        let keyboard_scroll = app.rules_view.preview_scroll;
        assert!(keyboard_scroll > 0);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let preview = app
            .hits
            .iter()
            .find(|hit| hit.action == HitAction::FocusRulePreview)
            .unwrap()
            .clone();
        app.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: preview.rect.x.saturating_add(1),
            row: preview.rect.y.saturating_add(1),
            modifiers: KeyModifiers::NONE,
        });
        assert!(app.rules_view.preview_scroll > keyboard_scroll);

        app.rules.get_mut("always").unwrap()[0].rule = "short".into();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(app.rules_view.preview_scroll, 0);
    }

    #[test]
    fn decision_rows_rules_preview_and_modal_controls_have_mouse_hits() {
        let mut app = keyboard_test_app();
        app.switch_view(TuiView::Decisions);
        let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app
            .hits
            .iter()
            .any(|hit| matches!(hit.action, HitAction::SelectDecision(0))));
        assert!(app
            .hits
            .iter()
            .any(|hit| hit.action == HitAction::FocusDecisionDetail));

        app.switch_view(TuiView::Rules);
        app.rules
            .get_mut("always")
            .unwrap()
            .push(crate::protocol::config::RuleItem {
                id: 1,
                rule: "Keep controls coherent".into(),
                source: None,
            });
        app.rules_view.preview_open = true;
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app
            .hits
            .iter()
            .any(|hit| hit.action == HitAction::ToggleRulePreview));

        app.switch_view(TuiView::Board);
        app.selected_state = 1;
        app.docs[1]
            .fields
            .insert("accord.status".into(), "delivered".into());
        app.start_validation_accept();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app
            .hits
            .iter()
            .any(|hit| hit.action == HitAction::ConfirmModal));
        assert!(app
            .hits
            .iter()
            .any(|hit| hit.action == HitAction::CancelModal));
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
