//! Minimal Board workflow actions backed by the native app layer.
//!
//! This module owns only transient prompts and input translation. Transition
//! validity, persistence, and completion policy remain in `app` and protocol.

use super::*;

fn checkpoint_note(outcome: &CheckpointOutcome) -> String {
    match &outcome.status {
        CheckpointStatus::Batched => {
            "; milestone/grouping write batched to assignment boundary".to_string()
        }
        CheckpointStatus::Checkpointed => {
            format!("; record written; Git checkpointed{}", checkpoint_note_suffix(outcome))
        }
        CheckpointStatus::Clean => {
            format!("; Git checkpoint clean{}", checkpoint_note_suffix(outcome))
        }
        CheckpointStatus::Failed { message } => {
            format!("; RECORD WRITTEN but Git checkpoint FAILED: {message}")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeliveryField {
    Summary,
    Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WorkflowPrompt {
    Claim {
        id: String,
        title: String,
        assignee: String,
    },
    Deliver {
        id: String,
        title: String,
        summary: String,
        evidence: String,
        field: DeliveryField,
    },
    Block {
        id: String,
        title: String,
        note: String,
    },
    Complete {
        id: String,
        title: String,
    },
}

impl WorkflowPrompt {
    fn id(&self) -> &str {
        match self {
            Self::Claim { id, .. }
            | Self::Deliver { id, .. }
            | Self::Block { id, .. }
            | Self::Complete { id, .. } => id,
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::Claim { title, .. }
            | Self::Deliver { title, .. }
            | Self::Block { title, .. }
            | Self::Complete { title, .. } => title,
        }
    }

    fn modal_title(&self) -> &'static str {
        match self {
            Self::Claim { .. } => " Claim accord ",
            Self::Deliver { .. } => " Deliver accord ",
            Self::Block { .. } => " Block accord ",
            Self::Complete { .. } => " Complete task ",
        }
    }
}

impl TuiApp {
    pub(super) fn start_workflow_action(&mut self, action: &str) {
        let Some(doc) = self.selected_doc() else {
            self.status = "No selected Board task for workflow action.".into();
            return;
        };
        let id = doc.id().to_string();
        let title = doc.title().to_string();
        match action {
            "claim" => {
                self.workflow_prompt = Some(WorkflowPrompt::Claim {
                    id,
                    title,
                    assignee: String::new(),
                });
                self.status = "Claim accord: type an assignee and press Enter; Esc cancels.".into();
            }
            "deliver" => {
                self.workflow_prompt = Some(WorkflowPrompt::Deliver {
                    id,
                    title,
                    summary: String::new(),
                    evidence: String::new(),
                    field: DeliveryField::Summary,
                });
                self.status = "Deliver accord: type a summary and press Enter; Esc cancels.".into();
            }
            "block" => {
                self.workflow_prompt = Some(WorkflowPrompt::Block {
                    id,
                    title,
                    note: String::new(),
                });
                self.status = "Block accord: type a reason and press Enter; Esc cancels.".into();
            }
            "complete" => {
                self.workflow_prompt = Some(WorkflowPrompt::Complete { id, title });
                self.status = "Confirm completion: Enter archives the task; Esc cancels.".into();
            }
            "resume" => {
                let id = self.selected_doc().map(|doc| doc.id().to_string());
                if let Some(id) = id {
                    self.finish_workflow_transition("resume", &id, None, None);
                } else {
                    self.status = "No selected Board task for resume.".into();
                }
            }
            _ => self.status = format!("Unknown Board workflow action `{action}`."),
        }
    }

    pub(super) fn handle_workflow_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.workflow_prompt = None;
                self.status = "Board workflow action canceled.".into();
            }
            KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => {
                self.advance_workflow_prompt()
            }
            KeyCode::Backspace => {
                if let Some(input) = self.workflow_prompt_input_mut() {
                    input.pop();
                }
                self.refresh_workflow_prompt_status();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(input) = self.workflow_prompt_input_mut() {
                    input.clear();
                }
                self.refresh_workflow_prompt_status();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(input) = self.workflow_prompt_input_mut() {
                    input.push(ch);
                }
                self.refresh_workflow_prompt_status();
            }
            _ => {}
        }
    }

    fn workflow_prompt_input_mut(&mut self) -> Option<&mut String> {
        match self.workflow_prompt.as_mut()? {
            WorkflowPrompt::Claim { assignee, .. } => Some(assignee),
            WorkflowPrompt::Deliver {
                summary,
                evidence,
                field,
                ..
            } => Some(match field {
                DeliveryField::Summary => summary,
                DeliveryField::Evidence => evidence,
            }),
            WorkflowPrompt::Block { note, .. } => Some(note),
            WorkflowPrompt::Complete { .. } => None,
        }
    }

    fn advance_workflow_prompt(&mut self) {
        let Some(prompt) = self.workflow_prompt.as_mut() else {
            return;
        };
        match prompt {
            WorkflowPrompt::Claim { assignee, .. } if assignee.trim().is_empty() => {
                self.status = "Claim requires an assignee. Type a name and press Enter.".into();
            }
            WorkflowPrompt::Claim { id, assignee, .. } => {
                let id = id.clone();
                let assignee = assignee.trim().to_string();
                self.finish_workflow_transition("claim", &id, Some(assignee), None);
            }
            WorkflowPrompt::Deliver {
                summary,
                field: DeliveryField::Summary,
                ..
            } if summary.trim().is_empty() => {
                self.status = "Deliver requires a summary. Type the delivery summary.".into();
            }
            WorkflowPrompt::Deliver { field, .. } if *field == DeliveryField::Summary => {
                *field = DeliveryField::Evidence;
                self.status =
                    "Delivery evidence is required; type it and press Enter to submit.".into();
            }
            WorkflowPrompt::Deliver {
                id,
                summary,
                evidence,
                field: DeliveryField::Evidence,
                ..
            } => {
                let Some(evidence) = parse_evidence_input(evidence) else {
                    self.status = "Deliver requires one non-empty evidence item. Commas are preserved as literal text; type evidence and press Enter.".into();
                    return;
                };
                if let Err(error) = accord::validate_delivery_evidence(&evidence) {
                    self.status = error;
                    return;
                }
                let id = id.clone();
                let summary = summary.trim().to_string();
                self.finish_workflow_transition("deliver", &id, Some(summary), Some(evidence));
            }
            WorkflowPrompt::Deliver {
                field: DeliveryField::Summary,
                ..
            } => {
                unreachable!("summary delivery prompt is handled above");
            }
            WorkflowPrompt::Block { note, .. } if note.trim().is_empty() => {
                self.status = "Block requires a note. Type the reason work is blocked.".into();
            }
            WorkflowPrompt::Block { id, note, .. } => {
                let id = id.clone();
                let note = note.trim().to_string();
                self.finish_workflow_transition("block", &id, Some(note), None);
            }
            WorkflowPrompt::Complete { id, .. } => {
                let id = id.clone();
                self.finish_workflow_completion(&id);
            }
        }
    }

    fn finish_workflow_transition(
        &mut self,
        action: &str,
        id: &str,
        value: Option<String>,
        evidence: Option<Vec<String>>,
    ) {
        let result = app::accord::transition(
            &self.workspace,
            action,
            app::accord::AccordOptions {
                id: id.to_string(),
                assignee: (action == "claim").then_some(value.clone()).flatten(),
                summary: (action == "deliver").then_some(value.clone()).flatten(),
                note: (action == "block").then_some(value).flatten(),
                evidence: evidence.unwrap_or_default(),
                ..Default::default()
            },
        );
        match result {
            Ok(outcome) => {
                self.workflow_prompt = None;
                let reload_note = self.reload().warning_note();
                self.status = format!(
                    "Accord {}: {}{}{}",
                    outcome.id,
                    outcome.status,
                    checkpoint_note(&outcome.checkpoint),
                    reload_note
                );
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("{} error: {}{}", action, error.message, reload_note);
            }
        }
    }

    fn finish_workflow_completion(&mut self, id: &str) {
        match app::tasks::complete(
            &self.workspace,
            app::tasks::CompleteOptions {
                id: id.to_string(),
                ..Default::default()
            },
        ) {
            Ok(outcome) => {
                self.workflow_prompt = None;
                let warning = outcome.warnings.first().cloned();
                let reload_note = self.reload().warning_note();
                self.status = match warning {
                    Some(warning) => format!(
                        "Completed {} with warning: {}{}{}",
                        outcome.id,
                        warning,
                        checkpoint_note(&outcome.checkpoint),
                        reload_note
                    ),
                    None => format!(
                        "Completed {}{}{}",
                        outcome.id,
                        checkpoint_note(&outcome.checkpoint),
                        reload_note
                    ),
                };
            }
            Err(error) => {
                let reload_note = self.reload().warning_note();
                self.status = format!("complete error: {}{}", error.message, reload_note);
            }
        }
    }

    fn refresh_workflow_prompt_status(&mut self) {
        let Some(prompt) = self.workflow_prompt.as_ref() else {
            return;
        };
        self.status = match prompt {
            WorkflowPrompt::Claim { id, assignee, .. } => format!(
                "Claim {id}: {} · Enter submits, Esc cancels",
                if assignee.trim().is_empty() {
                    "<assignee>"
                } else {
                    assignee
                }
            ),
            WorkflowPrompt::Deliver {
                id,
                summary,
                evidence,
                field,
                ..
            } => match field {
                DeliveryField::Summary => format!(
                    "Deliver {id}: {} · Enter continues, Esc cancels",
                    if summary.trim().is_empty() {
                        "<summary>"
                    } else {
                        summary
                    }
                ),
                DeliveryField::Evidence => format!(
                    "Deliver {id} evidence: {} · Enter submits, Esc cancels",
                    if evidence.trim().is_empty() {
                        "<evidence>"
                    } else {
                        evidence
                    }
                ),
            },
            WorkflowPrompt::Block { id, note, .. } => format!(
                "Block {id}: {} · Enter submits, Esc cancels",
                if note.trim().is_empty() {
                    "<reason>"
                } else {
                    note
                }
            ),
            WorkflowPrompt::Complete { id, .. } => {
                format!("Confirm completion for {id}: Enter archives, Esc cancels")
            }
        };
    }

    pub(super) fn draw_workflow_prompt(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let Some(prompt) = self.workflow_prompt.as_ref() else {
            return;
        };
        let popup = centered_rect(82, 42, area);
        frame.render_widget(Clear, popup);
        let mut lines = vec![Line::from(vec![
            Span::styled("Target: ", self.theme.label_style()),
            Span::styled(
                format!("{} — {}", prompt.id(), prompt.title()),
                self.theme.text_style().add_modifier(Modifier::BOLD),
            ),
        ])];
        match prompt {
            WorkflowPrompt::Claim { assignee, .. } => {
                lines.push(workflow_input_line("Assignee", assignee, &self.theme))
            }
            WorkflowPrompt::Deliver {
                summary,
                evidence,
                field,
                ..
            } => {
                lines.push(workflow_input_line("Summary", summary, &self.theme));
                lines.push(workflow_input_line("Evidence", evidence, &self.theme));
                lines.push(Line::from(Span::styled(
                    if *field == DeliveryField::Summary {
                        "Editing summary"
                    } else {
                        "Editing evidence (commas preserved)"
                    },
                    self.theme.muted_style(),
                )));
            }
            WorkflowPrompt::Block { note, .. } => {
                lines.push(workflow_input_line("Reason", note, &self.theme))
            }
            WorkflowPrompt::Complete { .. } => lines.push(Line::from(Span::styled(
                "Routine completion archives delivered work and records final resolution evidence.",
                self.theme.muted_style(),
            ))),
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enter submits or advances · Esc cancels · Ctrl-U clears the active field",
            self.theme.muted_style(),
        )));
        let prompt_view = Paragraph::new(lines)
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
            Paragraph::new("[ Confirm ]").style(self.theme.status_style(StatusTone::Accent)),
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
}

fn workflow_input_line(label: &str, value: &str, theme: &TuiTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme.label_style()),
        Span::styled(
            if value.is_empty() {
                "<type here>".to_string()
            } else {
                value.to_string()
            },
            if value.is_empty() {
                theme.muted_style()
            } else {
                theme.text_style()
            },
        ),
    ])
}

fn parse_evidence_input(value: &str) -> Option<Vec<String>> {
    let value = value.trim();
    if value.is_empty()
        || value
            .chars()
            .all(|character| character == ',' || character.is_whitespace())
    {
        return None;
    }
    // Evidence is one opaque item per prompt. Commas remain literal so prose
    // such as `tests, with coverage` is never silently rewritten or split.
    Some(vec![value.to_string()])
}
