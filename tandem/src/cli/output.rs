//! Unified process-level error rendering for the clap CLI.
use crate::CliError;

pub(crate) fn usage_error(message: impl Into<String>, json: bool) -> CliError {
    CliError::usage(message).with_json(json)
}
