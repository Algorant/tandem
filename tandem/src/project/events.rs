//! Concrete legacy event-log append operations.

use std::fs::OpenOptions;
use std::io::Write;

use crate::project::{display_path, TandemProject};
use crate::protocol::event::{self, CanonicalEventEnvelope, EventEnvelope};
use crate::CliError;

/// Appends one complete JSONL record after the requested project mutation.
/// The existing global-log shape is retained until the separate actor-log
/// compatibility operation changes persisted event layout.
pub(crate) fn append_event(
    project: &TandemProject,
    event_name: &str,
    id: &str,
    summary: &str,
    timestamp: &str,
) -> Result<(), CliError> {
    debug_assert!(event::is_known_name(event_name));
    debug_assert_eq!(CanonicalEventEnvelope::required_fields().len(), 6);
    let line = EventEnvelope {
        ts: timestamp,
        event: event_name,
        id,
        summary,
    }
    .legacy_json_line(json_string);
    let mut file = OpenOptions::new().create(true).append(true).open(&project.events_path).map_err(|error| {
        CliError::user(format!(
            "Event append failure: could not open {} while recording `{event_name}` for `{id}`: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
            display_path(&project.events_path)
        ))
    })?;
    file.write_all(line.as_bytes()).map_err(|error| {
        CliError::user(format!(
            "Event append failure: could not append `{event_name}` for `{id}` to {}: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
            display_path(&project.events_path)
        ))
    })
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('\"');
    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn append_keeps_one_complete_escaped_json_line_per_event() {
        let root = std::env::temp_dir().join(format!(
            "tandem-project-events-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let data_dir = root.join(".tandem");
        let project =
            TandemProject::with_paths(root.clone(), data_dir.clone(), data_dir.join("tandem.md"));
        fs::create_dir_all(&data_dir).unwrap();
        append_event(
            &project,
            "task.updated",
            "task-1",
            "line one\nline two",
            "now",
        )
        .unwrap();
        append_event(&project, "task.updated", "task-1", "again", "later").unwrap();
        let lines = fs::read_to_string(&project.events_path).unwrap();
        assert_eq!(lines.lines().count(), 2);
        assert!(lines.contains("line one\\nline two"));
        fs::remove_dir_all(root).unwrap();
    }
}
