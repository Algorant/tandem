//! Shared Task lifecycle operations.

use crate::{
    AddOptions, AddOutcome, CancelOutcome, CliError, CompleteOptions, CompleteOutcome,
    MoveTaskOutcome, TandemProject, UpdateOptions, UpdateOutcome,
};

/// Create a Task, Epic, or Subtask after canonical hierarchy validation.
pub(crate) fn add(project: &TandemProject, input: AddOptions) -> Result<AddOutcome, CliError> {
    crate::add_task(project, input)
}

/// Move one active Task and perform its canonical accord synchronization.
pub(crate) fn move_to_state(
    project: &TandemProject,
    id: &str,
    state: &str,
) -> Result<MoveTaskOutcome, CliError> {
    crate::move_task_to_state(project, id, state)
}

/// Apply supported Task metadata changes while preserving unknown source.
pub(crate) fn update(
    project: &TandemProject,
    input: UpdateOptions,
) -> Result<UpdateOutcome, CliError> {
    crate::update_task_metadata(project, input)
}

/// Archive an active Task with canonical completion metadata and warnings.
pub(crate) fn complete(
    project: &TandemProject,
    input: CompleteOptions,
) -> Result<CompleteOutcome, CliError> {
    crate::complete_task(project, input)
}

/// Archive an active Task as canceled.
pub(crate) fn cancel(
    project: &TandemProject,
    id: &str,
    reason: &str,
) -> Result<CancelOutcome, CliError> {
    crate::cancel_task(project, id, reason)
}
