//! Private support shared by Task and accord application use cases.
//!
//! `CliError` remains a temporary crate-root exception pending task-159; this
//! module otherwise depends only on project/protocol ownership boundaries.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::project::{self, TandemProject};
use crate::CliError;

pub(crate) fn require_nonempty<'a>(
    value: Option<&'a str>,
    message: &str,
) -> Result<&'a str, CliError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::usage(message))
}

pub(crate) fn current_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_unix_timestamp(seconds)
}

pub(crate) fn append_event(
    project: &TandemProject,
    event_name: &str,
    id: &str,
    summary: &str,
) -> Result<(), CliError> {
    project::events::append_event(project, event_name, id, summary, &current_timestamp())
}

fn format_unix_timestamp(seconds: u64) -> String {
    let seconds = seconds as i64;
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + i64::from(month <= 2), month, day)
}
