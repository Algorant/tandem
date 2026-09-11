//! Child-based Epic completion policy matrix.
//!
//! These tests drive the real CLI. The missing-delivery warning is surfaced in
//! the JSON `warnings` array and, for human output, on stderr.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const POLICY_WARNING: &str = "complete normally follows a delivered Accord";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-epic-completion-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn git(cwd: &Path, args: &[&str]) {
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
}

/// A disposable Git-backed workspace with a clean checkpoint baseline.
fn setup(label: &str) -> PathBuf {
    let root = root(label);
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "--quiet"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Tandem Tests"]);
    let output = run(&root, &["init", "--title", "Epic completion tests"]);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "fixture baseline"]);
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    bin().args(args).current_dir(root).output().unwrap()
}

fn ok(root: &Path, args: &[&str]) -> String {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn fail(root: &Path, args: &[&str]) -> Output {
    let output = run(root, args);
    assert!(
        !output.status.success(),
        "{args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    output
}

fn json(stdout: &str) -> Value {
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!("expected one complete JSON envelope, got {stdout:?}: {error}")
    })
}

fn warnings(stdout: &str) -> Vec<String> {
    json(stdout)["warnings"]
        .as_array()
        .expect("warnings array")
        .iter()
        .map(|warning| warning.as_str().expect("warning string").to_string())
        .collect()
}

fn add_epic(root: &Path, title: &str) {
    let output = run(
        root,
        &[
            "add",
            "task",
            title,
            "--kind",
            "epic",
            "--acceptance",
            "epic accepted",
        ],
    );
    assert!(output.status.success());
}

fn complete(root: &Path, id: &str) {
    let output = run(root, &["complete", id]);
    assert!(
        output.status.success(),
        "complete {id} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn eligible_epic_closes_warning_free_without_fabricating_delivery() {
    let root = setup("eligible");
    ok(
        &root,
        &[
            "add",
            "task",
            "Epic",
            "--kind",
            "epic",
            "--acceptance",
            "epic accepted",
        ],
    );
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    ok(
        &root,
        &[
            "add",
            "task",
            "Subtask",
            "--parent",
            "task-2",
            "--acceptance",
            "subtask accepted",
        ],
    );
    complete(&root, "task-2-1");
    complete(&root, "task-2");

    let epic_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&epic_json).is_empty(),
        "an eligible Epic must close without the delivery warning: {epic_json}"
    );
    assert_eq!(json(&epic_json)["data"]["id"], "task-1");

    let archived = fs::read_to_string(root.join(".tandem/logs/task-1.md")).unwrap();
    assert!(archived.contains("status: \"ready\""), "{archived}");
    assert!(archived.contains("outcome: \"completed\""), "{archived}");
    assert!(
        !archived.contains("status: \"delivered\"") && !archived.contains("status: \"accepted\""),
        "child-based closure must not fabricate delivery or acceptance: {archived}"
    );
    assert!(
        !archived.contains("evidence:"),
        "child-based closure must not copy child evidence: {archived}"
    );
    assert!(!root.join(".tandem/tasks/task-1.md").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn eligible_epic_human_completion_has_no_policy_warning_on_stderr() {
    let root = setup("eligible-human");
    add_epic(&root, "Epic");
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    complete(&root, "task-2");

    let output = run(&root, &["complete", "task-1"]);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(POLICY_WARNING),
        "human eligible Epic closure must not print the delivery warning: {stderr}"
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Completed task-1"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_epic_retains_policy_warning_in_json_and_human() {
    let root = setup("empty");
    add_epic(&root, "Empty json Epic");
    add_epic(&root, "Empty human Epic");

    let json_output = ok(&root, &["complete", "task-1", "--json"]);
    let json_warnings = warnings(&json_output);
    assert_eq!(json_warnings.len(), 1, "{json_output}");
    assert!(json_warnings[0].contains(POLICY_WARNING), "{json_output}");

    let human = run(&root, &["complete", "task-2"]);
    assert!(human.status.success());
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert_eq!(
        stderr.matches(POLICY_WARNING).count(),
        1,
        "the policy warning must appear exactly once on stderr: {stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn epic_with_delivered_but_active_child_is_rejected() {
    let root = setup("active-child");
    add_epic(&root, "Epic");
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    ok(
        &root,
        &["accord", "claim", "task-2", "--assignee", "worker"],
    );
    ok(
        &root,
        &[
            "accord",
            "deliver",
            "task-2",
            "--summary",
            "delivered",
            "--evidence",
            "delivery evidence",
        ],
    );

    let output = fail(&root, &["complete", "task-1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot complete"), "{stderr}");
    assert!(stderr.contains("task-2"), "{stderr}");
    assert!(root.join(".tandem/tasks/task-1.md").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn canceled_child_retains_policy_warning() {
    let root = setup("canceled-child");
    add_epic(&root, "Epic");
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    ok(&root, &["cancel", "task-2", "--note", "not needed"]);

    let epic_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&epic_json)
            .iter()
            .any(|warning| warning.contains(POLICY_WARNING)),
        "a canceled child retains the delivery warning: {epic_json}"
    );
    let archived = fs::read_to_string(root.join(".tandem/logs/task-1.md")).unwrap();
    assert!(archived.contains("status: \"ready\""), "{archived}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_child_retains_policy_warning() {
    let root = setup("failed-child");
    add_epic(&root, "Epic");
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    ok(
        &root,
        &["accord", "claim", "task-2", "--assignee", "worker"],
    );
    ok(
        &root,
        &[
            "accord",
            "deliver",
            "task-2",
            "--summary",
            "delivered",
            "--evidence",
            "delivery evidence",
        ],
    );
    ok(&root, &["accord", "fail", "task-2", "--note", "failed"]);

    let epic_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&epic_json)
            .iter()
            .any(|warning| warning.contains(POLICY_WARNING)),
        "a failed child retains the delivery warning: {epic_json}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_completion_outcome_alone_does_not_qualify() {
    let root = setup("legacy-child");
    add_epic(&root, "Epic");
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "child accepted",
        ],
    );
    complete(&root, "task-2");

    // Replace the canonical D47 field with the legacy `completion.outcome`
    // shape: only positive `resolution.outcome=completed` may qualify.
    let log_path = root.join(".tandem/logs/task-2.md");
    let legacy = fs::read_to_string(&log_path).unwrap().replace(
        "resolution:\n  outcome: \"completed\"",
        "completion:\n  outcome: \"completed\"",
    );
    fs::write(&log_path, legacy).unwrap();

    let epic_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&epic_json)
            .iter()
            .any(|warning| warning.contains(POLICY_WARNING)),
        "a legacy-only child outcome must not count as positive evidence: {epic_json}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_task_with_completed_subtask_retains_policy_warning() {
    let root = setup("ordinary-task");
    ok(
        &root,
        &["add", "task", "Task", "--acceptance", "task accepted"],
    );
    ok(
        &root,
        &[
            "add",
            "task",
            "Subtask",
            "--parent",
            "task-1",
            "--acceptance",
            "subtask accepted",
        ],
    );
    complete(&root, "task-1-1");

    let task_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&task_json)
            .iter()
            .any(|warning| warning.contains(POLICY_WARNING)),
        "an ordinary Task never uses the Epic exception: {task_json}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicitly_delivered_epic_is_warning_free() {
    let root = setup("delivered-epic");
    add_epic(&root, "Epic");
    ok(
        &root,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    ok(
        &root,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "delivered",
            "--evidence",
            "delivery evidence",
        ],
    );

    let epic_json = ok(&root, &["complete", "task-1", "--json"]);
    assert!(
        warnings(&epic_json).is_empty(),
        "an explicitly delivered Epic keeps normal warning-free acceptance: {epic_json}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn epic_unresolved_blocker_still_blocks_closure() {
    let root = setup("blocked-epic");
    ok(
        &root,
        &["add", "task", "Blocker", "--acceptance", "blocker accepted"],
    );
    ok(
        &root,
        &[
            "add",
            "task",
            "Epic",
            "--kind",
            "epic",
            "--acceptance",
            "epic accepted",
            "--blocker",
            "task-1",
        ],
    );
    ok(
        &root,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-2",
            "--acceptance",
            "child accepted",
        ],
    );
    complete(&root, "task-3");

    let output = fail(&root, &["complete", "task-2"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unresolved blockers"), "{stderr}");
    assert!(root.join(".tandem/tasks/task-2.md").exists());
    fs::remove_dir_all(root).unwrap();
}
