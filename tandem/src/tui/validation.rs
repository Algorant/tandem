//! Validation prompts and the TUI adapter over shared application operations.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ValidationPrompt {
    Accept {
        id: String,
        title: String,
    },
    Rework {
        id: String,
        title: String,
        feedback: String,
    },
}

impl ValidationPrompt {
    pub(super) fn id(&self) -> &str {
        match self {
            Self::Accept { id, .. } | Self::Rework { id, .. } => id,
        }
    }

    pub(super) fn title(&self) -> &str {
        match self {
            Self::Accept { title, .. } | Self::Rework { title, .. } => title,
        }
    }
}

impl TuiApp {
    pub(super) fn activate_confirmation(&mut self) {
        if self.validation_prompt.is_some() {
            self.handle_validation_prompt_key(KeyEvent::from(KeyCode::Enter));
        } else if self.workflow_prompt.is_some() {
            self.handle_workflow_prompt_key(KeyEvent::from(KeyCode::Enter));
        } else if self.rules_prompt_active() {
            self.handle_rules_prompt_key(KeyEvent::from(KeyCode::Enter));
        }
    }

    pub(super) fn cancel_top_modal(&mut self) {
        if self.board_picker.is_some() {
            self.handle_picker_key(KeyEvent::from(KeyCode::Esc));
        } else if self.validation_prompt.is_some() {
            self.handle_validation_prompt_key(KeyEvent::from(KeyCode::Esc));
        } else if self.workflow_prompt.is_some() {
            self.handle_workflow_prompt_key(KeyEvent::from(KeyCode::Esc));
        } else if self.rules_prompt_active() {
            self.handle_rules_prompt_key(KeyEvent::from(KeyCode::Esc));
        } else if self.decision_prompt_active() {
            self.handle_decision_prompt_key(KeyEvent::from(KeyCode::Esc));
        }
    }

    pub(super) fn selected_validation_doc_summary(
        &self,
    ) -> Result<(String, String, String), String> {
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

    pub(super) fn start_validation_accept(&mut self) {
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

    pub(super) fn start_validation_rework(&mut self) {
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

    pub(super) fn show_validation_complete_hint(&mut self) {
        self.status = "Completion is intentionally de-emphasized in Validation. Accepting sign-off archives the Task; use the CLI to escalate work that needs human judgment.".to_string();
    }

    pub(super) fn handle_validation_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let action = match self.validation_prompt.take() {
                    Some(ValidationPrompt::Accept { .. }) => "Acceptance",
                    Some(ValidationPrompt::Rework { .. }) => "Rework request",
                    None => "Validation action",
                };
                self.status = format!("{action} canceled.");
            }
            KeyCode::Char('n')
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. })
                ) =>
            {
                self.status = "Acceptance canceled.".to_string();
            }
            KeyCode::Char('y')
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. })
                ) =>
            {
                self.finish_validation_accept();
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => {
                if matches!(
                    self.validation_prompt,
                    Some(ValidationPrompt::Accept { .. })
                ) {
                    self.finish_validation_accept();
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

    pub(super) fn refresh_validation_prompt_status(&mut self) {
        self.status = match self.validation_prompt.as_ref() {
            Some(ValidationPrompt::Accept { id, .. }) => {
                format!("Confirm acceptance for {id}: Enter/y accepts sign-off, Esc/n cancels.")
            }
            Some(ValidationPrompt::Rework { id, feedback, .. }) => format!(
                "Request changes for {id}: {} · Enter submits, Esc cancels",
                if feedback.trim().is_empty() {
                    "<note>"
                } else {
                    feedback.as_str()
                }
            ),
            None => self.status.clone(),
        };
    }

    pub(super) fn finish_validation_accept(&mut self) {
        let Some(ValidationPrompt::Accept { id, .. }) = self.validation_prompt.take() else {
            return;
        };
        match app::accord::accept_validation(&self.workspace, &id, "tui") {
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

    pub(super) fn finish_validation_rework(&mut self) {
        let Some(ValidationPrompt::Rework { id, feedback, .. }) = self.validation_prompt.as_ref()
        else {
            return;
        };
        let feedback = feedback.trim().to_string();
        if feedback.is_empty() {
            self.status =
                "Request changes requires a note. Type the reason and press Enter.".into();
            return;
        }
        let id = id.clone();
        self.validation_prompt = None;
        match app::accord::request_validation_rework(&self.workspace, &id, "tui", &feedback) {
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
}
