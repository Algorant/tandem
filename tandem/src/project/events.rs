//! Concrete per-actor event-ledger operations.

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

use crate::project::{display_path, extract_json_string, extract_json_u64, TandemProject};
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
    append_event_for_actor(
        project,
        event_name,
        id,
        summary,
        timestamp,
        &actor_id(project)?,
    )
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

pub(crate) fn actor_id(project: &TandemProject) -> Result<String, CliError> {
    persisted_actor_id(project)
}

fn persisted_actor_id(project: &TandemProject) -> Result<String, CliError> {
    let path = project.data_dir().join("actor-id");
    fs::create_dir_all(project.data_dir()).map_err(|error| identity_error(&path, error))?;
    ensure_git_ignored(project, &path)?;
    match fs::read_to_string(&path) {
        Ok(content) => parse_persisted_actor_id(&path, &content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let generated = Uuid::new_v4().hyphenated().to_string();
            let created = crate::project::write::write_new_atomic(&path, &format!("{generated}\n"))
                .map_err(|error| {
                    CliError::user(format!(
                        "Event actor identity failure: could not atomically persist {}: {}",
                        display_path(&path),
                        error.message
                    ))
                })?;
            if created {
                Ok(generated)
            } else {
                let content =
                    fs::read_to_string(&path).map_err(|error| identity_error(&path, error))?;
                parse_persisted_actor_id(&path, &content)
            }
        }
        Err(error) => Err(identity_error(&path, error)),
    }
}

fn parse_persisted_actor_id(path: &Path, content: &str) -> Result<String, CliError> {
    let candidate = content.strip_suffix('\n').unwrap_or(content);
    let candidate = candidate.strip_suffix('\r').unwrap_or(candidate);
    let valid = !candidate.contains(['\n', '\r'])
        && Uuid::parse_str(candidate).is_ok_and(|uuid| uuid.hyphenated().to_string() == candidate);
    if valid {
        Ok(candidate.to_string())
    } else {
        Err(CliError::user(format!(
            "Event actor identity failure: {} must contain one canonical lowercase hyphenated UUID",
            display_path(path)
        )))
    }
}

fn identity_error(path: &Path, error: std::io::Error) -> CliError {
    CliError::user(format!(
        "Event actor identity failure: could not read or write {}: {error}",
        display_path(path)
    ))
}

fn ensure_git_ignored(project: &TandemProject, actor_path: &Path) -> Result<(), CliError> {
    let Some((git_root, exclude_path)) = git_paths(project.root())? else {
        return Ok(());
    };
    let relative = actor_path.strip_prefix(&git_root).map_err(|_| {
        CliError::user(format!(
            "Event actor identity failure: {} is outside Git worktree {}",
            display_path(actor_path),
            display_path(&git_root)
        ))
    })?;
    let relative_arg = relative.as_os_str();
    if let Some(parent) = exclude_path.parent() {
        fs::create_dir_all(parent).map_err(|error| identity_error(&exclude_path, error))?;
    }
    let mut exclude = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&exclude_path)
        .map_err(|error| identity_error(&exclude_path, error))?;
    exclude
        .lock()
        .map_err(|error| identity_error(&exclude_path, error))?;
    let result = (|| {
        let ignored = Command::new("git")
            .arg("-C")
            .arg(&git_root)
            .args(["check-ignore", "--quiet", "--"])
            .arg(relative_arg)
            .status()
            .map_err(|error| git_identity_error(project.root(), error))?;
        if ignored.success() {
            return Ok(());
        }
        if ignored.code() != Some(1) {
            return Err(CliError::user(format!(
                "Event actor identity failure: Git could not check whether {} is ignored",
                display_path(actor_path)
            )));
        }
        let pattern = format!(
            "/{}",
            relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        );
        exclude
            .seek(SeekFrom::Start(0))
            .map_err(|error| identity_error(&exclude_path, error))?;
        let mut content = String::new();
        exclude
            .read_to_string(&mut content)
            .map_err(|error| identity_error(&exclude_path, error))?;
        if !content.lines().any(|line| line == pattern) {
            if !content.is_empty() && !content.ends_with('\n') {
                exclude
                    .write_all(b"\n")
                    .map_err(|error| identity_error(&exclude_path, error))?;
            }
            exclude
                .write_all(format!("{pattern}\n").as_bytes())
                .and_then(|_| exclude.sync_data())
                .map_err(|error| identity_error(&exclude_path, error))?;
        }
        Ok(())
    })();
    let _ = exclude.unlock();
    result
}

fn git_paths(root: &Path) -> Result<Option<(PathBuf, PathBuf)>, CliError> {
    let top = match Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(output) => output,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && nearest_git_metadata(root).is_none() =>
        {
            return Ok(None);
        }
        Err(error) => return Err(git_identity_error(root, error)),
    };
    if !top.status.success() {
        if let Some(metadata) = nearest_git_metadata(root) {
            return Err(CliError::user(format!(
                "Event actor identity failure: Git could not inspect {} despite Git metadata at {}: {}",
                display_path(root),
                display_path(&metadata),
                String::from_utf8_lossy(&top.stderr).trim()
            )));
        }
        return Ok(None);
    }
    let git_root = output_path(root, "worktree root", &top)?;
    let exclude = git_output(root, &["rev-parse", "--git-path", "info/exclude"])?;
    if !exclude.status.success() {
        return Err(CliError::user(
            "Event actor identity failure: Git could not resolve its exclude file",
        ));
    }
    let exclude_path = output_path(root, "exclude file", &exclude)?;
    let exclude_path = if exclude_path.is_absolute() {
        exclude_path
    } else {
        root.join(exclude_path)
    };
    Ok(Some((git_root, exclude_path)))
}

fn nearest_git_metadata(root: &Path) -> Option<PathBuf> {
    root.ancestors()
        .map(|ancestor| ancestor.join(".git"))
        .find(|path| path.exists())
}

fn git_output(root: &Path, args: &[&str]) -> Result<Output, CliError> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| git_identity_error(root, error))
}

fn git_identity_error(root: &Path, error: std::io::Error) -> CliError {
    let action = if error.kind() == std::io::ErrorKind::NotFound {
        "the Git executable is required for this Git workspace but was not found"
    } else {
        "could not run Git"
    };
    CliError::user(format!(
        "Event actor identity failure: {action} for {}: {error}",
        display_path(root)
    ))
}

fn output_path(root: &Path, label: &str, output: &Output) -> Result<PathBuf, CliError> {
    let value = std::str::from_utf8(&output.stdout).map_err(|_| {
        CliError::user(format!(
            "Event actor identity failure: Git returned a non-Unicode {label} for {}",
            display_path(root)
        ))
    })?;
    let value = value.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        Err(CliError::user(format!(
            "Event actor identity failure: Git returned an empty {label} for {}",
            display_path(root)
        )))
    } else {
        Ok(PathBuf::from(value))
    }
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
    fn persisted_actor_is_reused_and_malformed_state_fails() {
        let (project, root) = project();
        let first = persisted_actor_id(&project).unwrap();
        assert_eq!(
            Uuid::parse_str(&first).unwrap().hyphenated().to_string(),
            first
        );
        assert_eq!(persisted_actor_id(&project).unwrap(), first);
        fs::write(project.data_dir().join("actor-id"), "not-an-identity\n").unwrap();
        let error = persisted_actor_id(&project).unwrap_err();
        assert!(error
            .message
            .contains("canonical lowercase hyphenated UUID"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn concurrent_first_use_converges_on_one_persisted_actor() {
        let (project, root) = project();
        let project = Arc::new(project);
        let barrier = Arc::new(Barrier::new(8));
        let handles = (0..8)
            .map(|_| {
                let project = Arc::clone(&project);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    persisted_actor_id(&project).unwrap()
                })
            })
            .collect::<Vec<_>>();
        let actors = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert!(actors.iter().all(|actor| actor == &actors[0]));
        assert_eq!(
            fs::read_to_string(project.data_dir().join("actor-id")).unwrap(),
            format!("{}\n", actors[0])
        );
        fs::remove_dir_all(root).unwrap();
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
    fn escaped_json_looking_summary_does_not_confuse_next_sequence() {
        let (project, root) = project();
        append_event_for_actor(
            &project,
            "task.updated",
            "task-1",
            "mentioned \"actor\":\"other\" and \"seq\":999",
            "now",
            "tester",
        )
        .unwrap();
        append_event_for_actor(
            &project,
            "task.updated",
            "task-1",
            "next",
            "later",
            "tester",
        )
        .unwrap();
        let content = fs::read_to_string(project.actor_events_path("tester")).unwrap();
        let lines = content.lines().collect::<Vec<_>>();
        assert_eq!(
            extract_json_string(lines[0], "actor"),
            Some("tester".to_string())
        );
        assert_eq!(extract_json_u64(lines[0], "seq"), Some(1));
        assert_eq!(extract_json_u64(lines[1], "seq"), Some(2));
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
