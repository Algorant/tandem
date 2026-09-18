//! Behavioral coverage for `tandem review` validation escalation.
//!
//! Review must name one of the Task's current `accord.acceptance` criteria
//! byte-exactly. Rejected requests must not mutate the record, the event log,
//! or Git checkpoint state.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCEPTANCE: &str = "preserve a, b; and c (exact)";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-review-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run(root: &Path, args: &[&str]) -> Output {
    bin().args(args).current_dir(root).output().unwrap()
}

fn json(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|error| panic!("expected one JSON envelope, got {stdout:?}: {error}"))
}

fn event_bytes(root: &Path) -> Vec<u8> {
    let events = root.join(".tandem/events");
    let mut paths = fs::read_dir(events)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().fold(Vec::new(), |mut bytes, path| {
        bytes.extend(fs::read(path).unwrap());
        bytes
    })
}

fn setup(label: &str) -> PathBuf {
    let root = root(label);
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "--quiet"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Tandem Tests"]);
    let init = run(&root, &["init", "--title", "Review behavior"]);
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    let add = run(
        &root,
        &["add", "task", "Review task", "--acceptance", ACCEPTANCE],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "fixture baseline"]);
    root
}

#[test]
fn exact_acceptance_criterion_enters_validation_and_batches_metadata() {
    let root = setup("exact");
    let before_head = git(&root, &["rev-parse", "HEAD"]);
    let review = run(
        &root,
        &[
            "review",
            "task-1",
            "--criterion",
            ACCEPTANCE,
            "--note",
            "human must confirm",
            "--json",
        ],
    );
    assert!(
        review.status.success(),
        "review failed: {}",
        String::from_utf8_lossy(&review.stderr)
    );
    let envelope = json(&review);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["state"], "validation");
    assert_eq!(envelope["data"]["checkpoint"]["status"], "batched");

    let shown = json(&run(&root, &["show", "task-1", "--json"]));
    assert_eq!(shown["data"]["state"], "validation");
    assert_eq!(
        shown["data"]["accord"]["acceptance"],
        serde_json::json!([ACCEPTANCE])
    );
    assert_eq!(shown["data"]["validation"]["criterion"], ACCEPTANCE);
    assert_eq!(
        git(&root, &["rev-parse", "HEAD"]),
        before_head,
        "root Task review must persist metadata without a checkpoint commit"
    );
    assert!(String::from_utf8_lossy(&event_bytes(&root)).contains("review.requested"));
    assert!(
        !git(&root, &["status", "--porcelain"]).is_empty(),
        "review must leave pending metadata for the next explicit checkpoint"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mismatched_criterion_is_rejected_without_record_event_or_checkpoint_mutation() {
    let root = setup("mismatch");
    let task_path = root.join(".tandem/tasks/task-1.md");
    let before_record = fs::read(&task_path).unwrap();
    let before_events = event_bytes(&root);
    let before_head = git(&root, &["rev-parse", "HEAD"]);
    let before_status = git(&root, &["status", "--porcelain"]);

    for criterion in [
        "totally made up",
        "preserve a, b; and c",
        "Preserve a, b; and c (exact)",
        "preserve a, b; and c (exact) ",
    ] {
        let rejected = run(
            &root,
            &[
                "review",
                "task-1",
                "--criterion",
                criterion,
                "--note",
                "why",
            ],
        );
        assert_eq!(
            rejected.status.code(),
            Some(1),
            "criterion {criterion:?}: {}",
            String::from_utf8_lossy(&rejected.stderr)
        );
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            stderr.contains("no acceptance criterion"),
            "unexpected error for {criterion:?}: {stderr}"
        );
        assert_eq!(
            fs::read(&task_path).unwrap(),
            before_record,
            "record mutated for {criterion:?}"
        );
        assert_eq!(
            event_bytes(&root),
            before_events,
            "events mutated for {criterion:?}"
        );
        assert_eq!(
            git(&root, &["rev-parse", "HEAD"]),
            before_head,
            "HEAD moved for {criterion:?}"
        );
        assert_eq!(
            git(&root, &["status", "--porcelain"]),
            before_status,
            "git status changed for {criterion:?}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_or_blank_criterion_is_a_usage_error_without_mutation() {
    let root = setup("empty");
    let task_path = root.join(".tandem/tasks/task-1.md");
    let before_record = fs::read(&task_path).unwrap();
    let before_events = event_bytes(&root);
    let before_head = git(&root, &["rev-parse", "HEAD"]);

    for criterion in ["", "   "] {
        let rejected = run(
            &root,
            &[
                "review",
                "task-1",
                "--criterion",
                criterion,
                "--note",
                "why",
            ],
        );
        assert_eq!(
            rejected.status.code(),
            Some(2),
            "criterion {criterion:?}: {}",
            String::from_utf8_lossy(&rejected.stderr)
        );
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            stderr.contains("requires --criterion"),
            "unexpected error for {criterion:?}: {stderr}"
        );
        assert_eq!(fs::read(&task_path).unwrap(), before_record);
        assert_eq!(event_bytes(&root), before_events);
        assert_eq!(git(&root, &["rev-parse", "HEAD"]), before_head);
    }
    fs::remove_dir_all(root).unwrap();
}
