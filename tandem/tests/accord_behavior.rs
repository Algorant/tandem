use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn run(root: &std::path::Path, args: &[&str]) -> String {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn failed(root: &std::path::Path, args: &[&str]) -> Output {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
    output
}

fn event_bytes(root: &std::path::Path) -> Vec<u8> {
    let events = root.join(".tandem/events");
    let mut paths = std::fs::read_dir(events)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().fold(Vec::new(), |mut bytes, path| {
        bytes.extend(std::fs::read(path).unwrap());
        bytes
    })
}

fn root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-cli-accord-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn release_rejects_invalid_disposition_with_usage_exit_code() {
    let root_dir = root("invalid-disposition");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "invalid disposition"]);
    run(
        &root_dir,
        &["add", "task", "Task", "--acceptance", "accept"],
    );
    let output = failed(
        &root_dir,
        &[
            "accord",
            "release",
            "task-1",
            "--note",
            "invalid",
            "--disposition",
            "bogus",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("disposition"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn accord_counts_cover_rework_and_discarded_attempts_on_archived_tasks() {
    let root_dir = root("counts");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "counts"]);
    run(
        &root_dir,
        &["add", "task", "Task", "--acceptance", "accept"],
    );
    run(
        &root_dir,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "first",
            "--evidence",
            "observed first",
        ],
    );
    run(
        &root_dir,
        &["accord", "rework", "task-1", "--note", "fix it"],
    );
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "second",
            "--evidence",
            "observed second",
        ],
    );
    run(
        &root_dir,
        &[
            "accord",
            "release",
            "task-1",
            "--note",
            "discard first attempt",
            "--disposition",
            "discarded",
        ],
    );
    let released =
        serde_json::from_str::<serde_json::Value>(&run(&root_dir, &["show", "task-1", "--json"]))
            .unwrap();
    assert_eq!(released["data"]["state"], "todo");
    assert_eq!(released["data"]["accordStatus"], "ready");
    let released_event = event_bytes(&root_dir);
    assert!(String::from_utf8_lossy(&released_event)
        .contains("\"data\":{\"disposition\":\"discarded\"}"));

    run(
        &root_dir,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "final",
            "--evidence",
            "observed final",
        ],
    );
    run(&root_dir, &["complete", "task-1"]);

    let shown: serde_json::Value =
        serde_json::from_str(&run(&root_dir, &["show", "task-1", "--json"])).unwrap();
    assert_eq!(shown["data"]["attemptCount"], 2);
    assert_eq!(shown["data"]["reworkCount"], 1);
    assert_eq!(shown["data"]["discardedCount"], 1);

    let assignment: serde_json::Value =
        serde_json::from_str(&run(&root_dir, &["assignment", "task-1", "--json"])).unwrap();
    assert_eq!(assignment["data"]["root"]["attemptCount"], 2);
    assert_eq!(assignment["data"]["root"]["reworkCount"], 1);
    assert_eq!(assignment["data"]["root"]["discardedCount"], 1);
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn cli_deliver_rejects_empty_evidence_without_record_or_event_mutation() {
    let root_dir = root("evidence");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "evidence"]);
    run(
        &root_dir,
        &["add", "task", "Task", "--acceptance", "accept"],
    );
    run(
        &root_dir,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    let task_path = root_dir.join(".tandem/tasks/task-1.md");
    let before_record = std::fs::read(&task_path).unwrap();
    let before_events = event_bytes(&root_dir);

    for args in [
        vec!["accord", "deliver", "task-1", "--summary", "observed"],
        vec![
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "observed",
            "--evidence",
            "",
        ],
        vec![
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "observed",
            "--evidence",
            "  \n\t",
        ],
    ] {
        let output = failed(&root_dir, &args);
        assert!(String::from_utf8_lossy(&output.stderr).contains("non-empty --evidence"));
        assert_eq!(std::fs::read(&task_path).unwrap(), before_record);
        assert_eq!(event_bytes(&root_dir), before_events);
    }

    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "observed",
            "--evidence",
            "observed output",
        ],
    );
    let shown: serde_json::Value =
        serde_json::from_str(&run(&root_dir, &["show", "task-1", "--json"])).unwrap();
    assert_eq!(shown["data"]["accordStatus"], "delivered");
    assert_eq!(
        shown["data"]["accord"]["evidence"],
        serde_json::json!(["observed output"])
    );
    assert!(event_bytes(&root_dir).len() > before_events.len());
    std::fs::remove_dir_all(root_dir).unwrap();
}
