//! Task kinds through the real CLI: vocabulary, minimum fields, papercut
//! defaults and placement, `--kind` filters, and the assignment projection.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tandem-cli-kind-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let output = bin()
        .current_dir(&dir)
        .args(["init", "--title", "kinds"])
        .output()
        .unwrap();
    assert!(output.status.success());
    dir
}

fn run(root: &Path, args: &[&str]) -> Value {
    let mut argv = vec!["--json"];
    argv.extend(args);
    let output = bin().current_dir(root).args(&argv).output().unwrap();
    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn fails(root: &Path, args: &[&str]) -> String {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(
        !output.status.success(),
        "{args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn show(root: &Path, id: &str) -> Value {
    run(root, &["show", id])["data"].clone()
}

fn add(root: &Path, args: &[&str]) -> String {
    let mut argv = vec!["add", "task"];
    argv.extend(args);
    run(root, &argv)["data"]["id"].as_str().unwrap().to_string()
}

const PLACEMENT: &str =
    "cannot be a Subtask; a papercut must be a root Task or a direct child of an Epic";

#[test]
fn kind_vocabulary_is_epic_research_papercut() {
    let dir = root("vocabulary");
    for kind in ["epic", "research", "papercut"] {
        let id = add(&dir, &["Kinded", "--kind", kind, "--acceptance", "ok"]);
        assert_eq!(show(&dir, &id)["kind"], kind);
    }
    let plain = add(&dir, &["Plain", "--acceptance", "ok"]);
    assert_eq!(show(&dir, &plain)["kind"], Value::Null);

    let error = fails(
        &dir,
        &[
            "add",
            "task",
            "Bad",
            "--kind",
            "bogus",
            "--acceptance",
            "ok",
        ],
    );
    assert!(
        error.contains(
            "Validation failed: invalid kind `bogus`; expected one of: epic, research, papercut"
        ),
        "{error}"
    );
    let error = fails(&dir, &["update", &plain, "--kind", "bogus"]);
    assert!(
        error.contains("invalid kind `bogus`; expected one of: epic, research, papercut"),
        "{error}"
    );
}

#[test]
fn only_a_papercut_may_omit_acceptance_and_defaults_to_low_priority() {
    let dir = root("minimum");
    let id = add(&dir, &["Rough edge", "--kind", "papercut"]);
    let record = show(&dir, &id);
    assert_eq!(record["kind"], "papercut");
    assert_eq!(record["priority"], "low");
    assert_eq!(record["accord"]["acceptance"], serde_json::json!([]));

    let high = add(
        &dir,
        &["Urgent edge", "--kind", "papercut", "--priority", "high"],
    );
    assert_eq!(show(&dir, &high)["priority"], "high");

    let task = add(&dir, &["Normal", "--acceptance", "ok"]);
    assert_eq!(show(&dir, &task)["priority"], Value::Null);

    for args in [
        vec!["add", "task", "No acceptance"],
        vec!["add", "task", "No acceptance", "--kind", "research"],
        vec!["add", "task", "No acceptance", "--kind", "epic"],
    ] {
        let error = fails(&dir, &args);
        assert!(
            error.contains(
                "add requires at least one --acceptance <text>; only --kind papercut may omit it"
            ),
            "{args:?}: {error}"
        );
    }
}

#[test]
fn papercut_placement_is_root_or_epic_child_never_subtask() {
    let dir = root("placement");
    let epic = add(&dir, &["Epic", "--kind", "epic", "--acceptance", "ok"]);
    let parent = add(&dir, &["Parent task", "--acceptance", "ok"]);
    let root_papercut = add(&dir, &["Root papercut", "--kind", "papercut"]);
    let epic_papercut = add(
        &dir,
        &["Epic papercut", "--kind", "papercut", "--parent", &epic],
    );
    assert_eq!(
        show(&dir, &epic_papercut)["parentRelationship"],
        "epic-task"
    );
    assert_eq!(show(&dir, &root_papercut)["role"], "task");

    // Research may sit anywhere, including as a Subtask.
    let research = add(
        &dir,
        &[
            "Sub research",
            "--kind",
            "research",
            "--parent",
            &parent,
            "--acceptance",
            "ok",
        ],
    );
    assert_eq!(show(&dir, &research)["role"], "subtask");

    let error = fails(
        &dir,
        &[
            "add",
            "task",
            "Sub papercut",
            "--kind",
            "papercut",
            "--parent",
            &parent,
        ],
    );
    assert!(
        error.contains(&format!(
            "Validation failed: a papercut under {parent} {PLACEMENT}"
        )),
        "{error}"
    );

    // Reparenting into a Task, or re-kinding an existing Subtask, fails.
    let error = fails(&dir, &["update", &root_papercut, "--parent", &parent]);
    assert!(
        error.contains(&format!(
            "Validation failed: papercut {root_papercut} {PLACEMENT}"
        )),
        "{error}"
    );
    let plain_sub = add(
        &dir,
        &["Plain sub", "--parent", &parent, "--acceptance", "ok"],
    );
    let error = fails(&dir, &["update", &plain_sub, "--kind", "papercut"]);
    assert!(
        error.contains(&format!(
            "Validation failed: papercut {plain_sub} {PLACEMENT}"
        )),
        "{error}"
    );
    // Moving a papercut under an Epic is fine.
    run(&dir, &["update", &root_papercut, "--parent", &epic]);
}

#[test]
fn acceptance_rules_follow_the_kind_on_update() {
    let dir = root("update");
    let papercut = add(
        &dir,
        &["Edge", "--kind", "papercut", "--acceptance", "fixed"],
    );
    run(&dir, &["update", &papercut, "--clear", "acceptance"]);
    assert_eq!(
        show(&dir, &papercut)["accord"]["acceptance"],
        serde_json::json!([])
    );

    let task = add(&dir, &["Task", "--acceptance", "ok"]);
    let error = fails(&dir, &["update", &task, "--clear", "acceptance"]);
    assert!(
        error.contains(&format!("{task} cannot clear acceptance; an active task requires at least one criterion (only a papercut may have none)")),
        "{error}"
    );

    // Re-kinding an acceptance-less papercut needs acceptance in the same call.
    let error = fails(&dir, &["update", &papercut, "--kind", "research"]);
    assert!(
        error.contains(&format!("{papercut} requires at least one acceptance criterion unless it is a papercut; add --acceptance <text>")),
        "{error}"
    );
    run(
        &dir,
        &[
            "update",
            &papercut,
            "--kind",
            "research",
            "--acceptance",
            "done",
        ],
    );
    assert_eq!(show(&dir, &papercut)["kind"], "research");
}

#[test]
fn list_and_search_filter_by_one_validated_kind() {
    let dir = root("filters");
    let research = add(
        &dir,
        &[
            "Needle research",
            "--kind",
            "research",
            "--acceptance",
            "ok",
        ],
    );
    let papercut = add(&dir, &["Needle papercut", "--kind", "papercut"]);
    let plain = add(&dir, &["Needle plain", "--acceptance", "ok"]);
    let ids = |value: Value| -> Vec<String> {
        value["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|doc| doc["id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        ids(run(&dir, &["list", "--kind", "research"])),
        vec![research.clone()]
    );
    assert_eq!(
        ids(run(&dir, &["list", "--kind", "papercut"])),
        vec![papercut.clone()]
    );
    assert_eq!(ids(run(&dir, &["list"])).len(), 3);
    assert_eq!(
        ids(run(&dir, &["search", "Needle", "--kind", "papercut"])),
        vec![papercut]
    );
    assert_eq!(
        ids(run(&dir, &["search", "Needle", "--kind", "research"])),
        vec![research]
    );
    assert!(ids(run(&dir, &["list", "--kind", "epic"])).is_empty());
    let _ = plain;

    for args in [
        vec!["list", "--kind", "bogus"],
        vec!["search", "x", "--kind", "bogus"],
    ] {
        let error = fails(&dir, &args);
        assert!(
            error.contains("Validation failed: invalid kind `bogus`; expected one of: epic, research, papercut"),
            "{args:?}: {error}"
        );
    }
}

#[test]
fn assignment_root_exposes_kind_and_null_for_a_standard_task() {
    let dir = root("assignment");
    let standard = add(&dir, &["Standard", "--acceptance", "ok"]);
    let research = add(
        &dir,
        &["Research", "--kind", "research", "--acceptance", "ok"],
    );
    let milestone = add(
        &dir,
        &[
            "Milestone",
            "--kind",
            "research",
            "--parent",
            &research,
            "--acceptance",
            "ok",
        ],
    );
    assert_eq!(
        run(&dir, &["assignment", &standard])["data"]["root"]["kind"],
        Value::Null
    );
    let assignment = run(&dir, &["assignment", &research])["data"].clone();
    assert_eq!(assignment["root"]["kind"], "research");
    assert_eq!(assignment["milestones"][0]["id"], milestone);
    assert_eq!(assignment["milestones"][0]["kind"], "research");
}
