//! TUI aggregate reload, external-change detection, diagnostics, and selection restoration.

use super::*;

const RELOAD_CHECK_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Default)]
pub(super) struct ReloadOutcome {
    pub(super) warning_count: usize,
    pub(super) first_warning: Option<String>,
}

impl ReloadOutcome {
    fn from_warnings(warnings: &[String]) -> Self {
        Self {
            warning_count: warnings.len(),
            first_warning: warnings.first().cloned(),
        }
    }

    pub(super) fn warning_note(&self) -> String {
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
pub(super) struct ReloadFingerprint {
    pub(super) files: BTreeMap<PathBuf, Option<FileSignature>>,
}

impl ReloadFingerprint {
    fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct ReloadSelection {
    board_doc_id: Option<String>,
    board_state: Option<String>,
    log_doc_id: Option<String>,
    rule_anchor: Option<(String, Option<usize>)>,
    decision_doc_id: Option<String>,
    papercut_id: Option<String>,
}

impl TuiApp {
    pub(super) fn reload(&mut self) -> ReloadOutcome {
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
        let mut docs = self
            .workspace
            .read_board_documents_tolerant(&mut load_errors);
        sort_documents(&mut docs);

        let (title, configured_states, rules) = match self.workspace.read_config_yaml() {
            Ok(root) => (
                workspace_title_from_root(root.as_ref()).unwrap_or_else(|| "Tandem".to_string()),
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
        let log_load = logs::load_logs(&self.workspace);
        let papercut_load = app::papercuts::load_open_inbox(&self.workspace);
        let hierarchy = TuiHierarchySnapshot::from_documents(&docs, &log_load.docs);
        load_errors.extend(log_load.warnings);
        let (log_events, event_warnings) = logs::load_log_events(&self.workspace);
        load_errors.extend(event_warnings);
        load_errors.extend(validation_load_errors_with_hierarchy(
            &docs,
            &log_load.docs,
            &configured_states,
            &hierarchy,
        ));
        match app::project::warnings(&self.workspace) {
            Ok(warnings) => load_errors.extend(warnings),
            Err(error) => load_errors.push(format!(
                "Compatibility diagnostics unavailable: {}",
                error.message
            )),
        }

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
        self.load_papercuts(papercut_load);
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
            "Reloaded {} active document{} from {} · {} open Papercut{} · {}{}",
            self.docs.len(),
            if self.docs.len() == 1 { "" } else { "s" },
            display_path(&self.workspace.board_dir),
            self.papercut_count(),
            if self.papercut_count() == 1 { "" } else { "s" },
            theme_note,
            load_note
        );
        self.reload_fingerprint = collect_reload_fingerprint(&self.workspace);
        self.last_reload_check = Instant::now();
        outcome
    }

    pub(super) fn capture_reload_selection(&self) -> ReloadSelection {
        ReloadSelection {
            board_doc_id: self.selected_doc().map(|doc| doc.id().to_string()),
            board_state: self.states.get(self.selected_state).cloned(),
            log_doc_id: self.selected_log().map(|doc| doc.id().to_string()),
            rule_anchor: self.selected_rule_anchor_for_reload(),
            decision_doc_id: self.selected_decision_id_for_reload(),
            papercut_id: self.selected_papercut_id_for_reload(),
        }
    }

    pub(super) fn restore_reload_selection(&mut self, selection: ReloadSelection) {
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
        self.restore_papercut_selection_after_reload(selection.papercut_id);
    }

    pub(super) fn runtime_warnings(&self) -> Vec<String> {
        self.load_errors
            .iter()
            .chain(self.theme_warnings.iter())
            .chain(self.papercut_warnings().iter())
            .cloned()
            .collect()
    }

    pub(super) fn text_input_active(&self) -> bool {
        self.quick_add.is_some()
            || matches!(
                self.validation_prompt,
                Some(ValidationPrompt::Rework { .. })
            )
            || self.log_search_input.is_some()
            || self.rules_text_prompt_active()
            || self.decision_prompt_active()
    }

    pub(super) fn input_overlay_active(&self) -> bool {
        self.quick_add.is_some()
            || self.board_picker.is_some()
            || self.validation_prompt.is_some()
            || self.log_search_input.is_some()
            || self.rules_prompt_active()
            || self.decision_prompt_active()
            || self.show_help
    }

    pub(super) fn reload_if_changed(&mut self) -> bool {
        if self.input_overlay_active() || self.last_reload_check.elapsed() < RELOAD_CHECK_INTERVAL {
            return false;
        }
        self.last_reload_check = Instant::now();
        let current = collect_reload_fingerprint(&self.workspace);
        if self.reload_fingerprint.is_empty() {
            self.reload_fingerprint = current;
            return false;
        }
        if current != self.reload_fingerprint {
            self.reload();
            self.status = format!("External changes detected; {}", self.status);
            return true;
        }
        false
    }

    pub(super) fn next_wake_timeout(&self) -> Duration {
        let reload_timeout = if self.input_overlay_active() {
            RELOAD_CHECK_INTERVAL
        } else {
            RELOAD_CHECK_INTERVAL.saturating_sub(self.last_reload_check.elapsed())
        };
        self.transient_status_timeout()
            .map_or(reload_timeout, |status_timeout| {
                reload_timeout.min(status_timeout)
            })
    }
}
