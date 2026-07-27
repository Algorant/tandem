//! Concrete per-actor event-ledger operations.

use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::project::{display_path, extract_json_string, TandemProject};
use crate::protocol::event::{self, CanonicalEventEnvelope};
use crate::CliError;

/// Appends a complete canonical envelope to the current writer's ledger.
/// Legacy global logs are read-only transition input and are never appended.
pub(crate) fn append_event(
    project: &TandemProject,
    event_name: &str,
    id: &str,
    summary: &str,
    timestamp: &str,
) -> Result<(), CliError> {
    debug_assert!(event::is_known_name(event_name));
    append_event_for_actor(project, event_name, id, summary, timestamp, &actor_id()?)
}

fn append_event_for_actor(
    project: &TandemProject,
    event_name: &str,
    id: &str,
    summary: &str,
    timestamp: &str,
    actor: &str,
) -> Result<(), CliError> {
    let path = project.actor_events_path(actor);
    fs::create_dir_all(project.events_dir())?;
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|error| append_error(&path, event_name, id, error))?;
    file.lock()
        .map_err(|error| append_error(&path, event_name, id, error))?;

    let result = (|| {
        let content = fs::read_to_string(&path)
            .map_err(|error| append_error(&path, event_name, id, error))?;
        let seq = next_sequence(&content, actor)?;
        let line = canonical_json_line(CanonicalEventEnvelope {
            ts: timestamp,
            event: event_name,
            id,
            summary,
            actor,
            seq,
        });
        (&file).write_all(line.as_bytes()).and_then(|_| file.sync_data()).map_err(|error| {
            CliError::user(format!(
                "Event append failure: could not append `{event_name}` for `{id}` to {}: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
                display_path(&path)
            ))
        })
    })();
    let _ = file.unlock();
    result
}

pub(crate) fn actor_id() -> Result<String, CliError> {
    let configured = std::env::var("TANDEM_ACTOR_ID").ok();
    // Without an explicit durable identity, isolate independent CLI processes
    // rather than letting them race on a shared user-name ledger. A process
    // may create a fresh opaque actor ledger; callers needing cross-process
    // identity set the filename-safe TANDEM_ACTOR_ID override.
    let candidate = configured.unwrap_or_else(fallback_actor_id);
    if is_safe_actor_id(&candidate) {
        Ok(candidate)
    } else {
        Err(CliError::user(
            "Event append failure: TANDEM_ACTOR_ID must be a filename-safe actor ID containing only ASCII letters, digits, `_`, `-`, or `.`",
        ))
    }
}

fn fallback_actor_id() -> String {
    format!("local-{}", std::process::id())
}

pub(crate) fn is_safe_actor_id(actor: &str) -> bool {
    !actor.is_empty()
        && actor != "."
        && actor != ".."
        && actor
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn next_sequence(content: &str, actor: &str) -> Result<u64, CliError> {
    let mut max = 0u64;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let line_actor = extract_json_string(line, "actor").ok_or_else(|| {
            CliError::user(format!(
                "Event append failure: malformed canonical event in {actor}.jsonl; repair the actor log before appending"
            ))
        })?;
        if line_actor != actor {
            return Err(CliError::user(format!(
                "Event append failure: actor log {actor}.jsonl contains an event owned by `{line_actor}`"
            )));
        }
        let seq = extract_json_u64(line, "seq").ok_or_else(|| {
            CliError::user(format!(
                "Event append failure: malformed canonical event sequence in {actor}.jsonl; repair the actor log before appending"
            ))
        })?;
        if seq <= max {
            return Err(CliError::user(format!(
                "Event append failure: non-monotonic sequence in {actor}.jsonl; repair the actor log before appending"
            )));
        }
        max = seq;
    }
    max.checked_add(1).ok_or_else(|| {
        CliError::user(format!(
            "Event append failure: sequence overflow for actor `{actor}`"
        ))
    })
}

fn extract_json_u64(line: &str, key: &str) -> Option<u64> {
    let key_pattern = format!("\"{key}\"");
    let after_key = line.find(&key_pattern)? + key_pattern.len();
    let colon_offset = line[after_key..].find(':')?;
    line[after_key + colon_offset + 1..]
        .trim_start()
        .split(|ch: char| !ch.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

fn canonical_json_line(event: CanonicalEventEnvelope<'_>) -> String {
    debug_assert_eq!(CanonicalEventEnvelope::required_fields().len(), 6);
    format!(
        "{{\"ts\":{},\"event\":{},\"id\":{},\"summary\":{},\"actor\":{},\"seq\":{}}}\n",
        json_string(event.ts),
        json_string(event.event),
        json_string(event.id),
        json_string(event.summary),
        json_string(event.actor),
        event.seq,
    )
}

fn append_error(
    path: &std::path::Path,
    event_name: &str,
    id: &str,
    error: std::io::Error,
) -> CliError {
    CliError::user(format!(
        "Event append failure: could not open {} while recording `{event_name}` for `{id}`: {error}. The file mutation may already be on disk; inspect the workspace and append a repair event if needed.",
        display_path(path)
    ))
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
    use std::sync::{Arc, Barrier};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn project() -> (TandemProject, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "tandem-project-events-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let data_dir = root.join(".tandem");
        (
            TandemProject::with_paths(root.clone(), data_dir.clone(), data_dir.join("tandem.md")),
            root,
        )
    }

    #[test]
    fn rejects_unsafe_actor_ids() {
        assert!(!is_safe_actor_id("../escape"));
        assert!(!is_safe_actor_id(""));
        assert!(is_safe_actor_id("actor_01-ABC.def"));
    }

    #[test]
    fn fallback_actor_is_filename_safe_and_process_isolated() {
        let actor = fallback_actor_id();
        assert!(is_safe_actor_id(&actor));
        assert_eq!(actor, format!("local-{}", std::process::id()));
    }

    #[test]
    fn actor_ledgers_use_matching_actor_and_monotonic_sequence() {
        let (project, root) = project();
        let path = project.actor_events_path("tester");
        fs::create_dir_all(project.events_dir()).unwrap();
        fs::write(&path, "{\"ts\":\"old\",\"event\":\"task.updated\",\"id\":\"task-1\",\"summary\":\"old\",\"actor\":\"tester\",\"seq\":4}\n").unwrap();
        append_event_for_actor(&project, "task.updated", "task-1", "next", "now", "tester")
            .unwrap();
        let line = fs::read_to_string(path)
            .unwrap()
            .lines()
            .last()
            .unwrap()
            .to_string();
        assert_eq!(extract_json_string(&line, "ts"), Some("now".to_string()));
        assert_eq!(
            extract_json_string(&line, "event"),
            Some("task.updated".to_string())
        );
        assert_eq!(extract_json_string(&line, "id"), Some("task-1".to_string()));
        assert_eq!(
            extract_json_string(&line, "summary"),
            Some("next".to_string())
        );
        assert_eq!(
            extract_json_string(&line, "actor"),
            Some("tester".to_string())
        );
        assert_eq!(extract_json_u64(&line, "seq"), Some(5));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_appends_produce_complete_unique_sequences() {
        let (project, root) = project();
        let project = Arc::new(project);
        let barrier = Arc::new(Barrier::new(6));
        let handles = (0..6)
            .map(|index| {
                let project = Arc::clone(&project);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    append_event_for_actor(
                        &project,
                        "task.updated",
                        "task-1",
                        &format!("{index}"),
                        "now",
                        "concurrent",
                    )
                    .unwrap();
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let content = fs::read_to_string(project.actor_events_path("concurrent")).unwrap();
        let sequences = content
            .lines()
            .map(|line| extract_json_u64(line, "seq").unwrap())
            .collect::<Vec<_>>();
        assert_eq!(sequences, vec![1, 2, 3, 4, 5, 6]);
        fs::remove_dir_all(root).unwrap();
    }
}
