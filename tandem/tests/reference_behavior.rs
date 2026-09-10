//! Reference vocabulary regression coverage at the CLI boundary.

use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn run(root: &Path, args: &[&str]) -> String {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn warnings(root: &Path, args: &[&str]) -> Vec<String> {
    let parsed: serde_json::Value = serde_json::from_str(&run(root, args)).unwrap();
    parsed["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

fn root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-cli-reference-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn task_add_and_update_skip_url_references_but_warn_missing_ids() {
    let root_dir = root("task-mutation");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "reference warnings"]);

    let created = warnings(
        &root_dir,
        &[
            "add",
            "task",
            "Board source",
            "--acceptance",
            "criterion",
            "--reference",
            "https://example.com/artifacts/42",
            "--reference",
            "HTTPS://Example.COM/Upper?x=1#fragment",
            "--reference",
            "missing-task",
            "--json",
        ],
    );
    assert!(
        !created
            .iter()
            .any(|warning| warning.contains("https://") || warning.contains("HTTPS://")),
        "absolute URL references must not warn: {created:?}"
    );
    assert!(
        created
            .iter()
            .any(|warning| warning == "reference not found: missing-task"),
        "an unresolved document ID must still warn: {created:?}"
    );

    let updated = warnings(
        &root_dir,
        &[
            "update",
            "task-1",
            "--reference",
            "http://localhost:3000/artifacts/43",
            "--reference",
            "missing-task",
            "--reference",
            "other-missing",
            "--json",
        ],
    );
    assert!(
        !updated
            .iter()
            .any(|warning| warning.contains("http://localhost:3000")),
        "URL references must not warn on update: {updated:?}"
    );
    assert!(
        updated
            .iter()
            .any(|warning| warning == "reference not found: other-missing"),
        "an unresolved document ID must still warn on update: {updated:?}"
    );

    let read = warnings(&root_dir, &["show", "task-1", "--json"]);
    assert!(
        read.iter()
            .any(|warning| warning == "task-1 references missing target missing-task."),
        "board read must warn the unresolved ID: {read:?}"
    );
    assert!(
        read.iter()
            .any(|warning| warning == "task-1 references missing target other-missing."),
        "board read must warn the other unresolved ID: {read:?}"
    );
    assert!(
        !read.iter().any(|warning| warning.contains("https://")
            || warning.contains("HTTPS://")
            || warning.contains("http://localhost:3000")),
        "board read must not warn stored URL references: {read:?}"
    );
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_add_skips_url_references_but_warns_missing_ids() {
    let root_dir = root("decision-mutation");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "decision references"]);

    let created = warnings(
        &root_dir,
        &[
            "add",
            "decision",
            "Choose the reference seam",
            "--reference",
            "https://example.com/decisions/7",
            "--reference",
            "missing-decision",
            "--json",
        ],
    );
    assert!(
        !created
            .iter()
            .any(|warning| warning.contains("https://example.com/decisions/7")),
        "URL references must not warn on decision add: {created:?}"
    );
    assert!(
        created
            .iter()
            .any(|warning| warning == "reference not found: missing-decision"),
        "an unresolved decision reference must still warn: {created:?}"
    );
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn archived_logs_never_warn_and_board_references_resolve_to_logs() {
    let root_dir = root("archived");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "archived references"]);
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Archived later",
            "--acceptance",
            "criterion",
            "--reference",
            "missing-archived",
        ],
    );
    run(
        &root_dir,
        &["cancel", "task-1", "--note", "superseded by task-2"],
    );
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Board reader",
            "--acceptance",
            "criterion",
            "--reference",
            "task-1",
        ],
    );

    let log_warnings = warnings(&root_dir, &["show", "task-1", "--json"]);
    assert!(
        !log_warnings
            .iter()
            .any(|warning| warning.contains("references missing target")),
        "archived Logs must not emit reference warnings: {log_warnings:?}"
    );
    let board_warnings = warnings(&root_dir, &["show", "task-2", "--json"]);
    assert!(
        !board_warnings
            .iter()
            .any(|warning| warning.contains("task-2 references missing target task-1.")),
        "a Board reference to an archived ID must resolve: {board_warnings:?}"
    );
    std::fs::remove_dir_all(root_dir).unwrap();
}
