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
