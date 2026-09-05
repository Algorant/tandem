use std::process::Command;
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

fn data(root: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let output = run(root, args);
    serde_json::from_str::<serde_json::Value>(&output).unwrap()["data"].clone()
}

fn root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-cli-assignment-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn assignment_json_is_complete_for_long_root_and_ten_milestones() {
    let root_dir = root("complete");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "assignment"]);
    let long_body = format!("{}ROOT-TAIL", "root definition ".repeat(2_000));
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Root assignment",
            "--acceptance",
            "root acceptance",
            "--constraint",
            "root constraint",
            "--validation",
            "root validation",
            "--related-file",
            "src/root.rs",
            "--body",
            &long_body,
        ],
    );
    for index in 1..=10 {
        let body = format!("milestone body {index} MILESTONE-TAIL-{index}");
        run(
            &root_dir,
            &[
                "add",
                "task",
                &format!("Milestone {index}"),
                "--parent",
                "task-1",
                "--acceptance",
                &format!("acceptance {index}"),
                "--constraint",
                &format!("constraint {index}"),
                "--validation",
                &format!("validation {index}"),
                "--related-file",
                &format!("src/milestone-{index}.rs"),
                "--body",
                &body,
            ],
        );
    }

    let assignment = data(&root_dir, &["assignment", "task-1", "--json"]);
    let root_show = data(&root_dir, &["show", "task-1", "--json"]);
    assert_eq!(assignment["root"]["id"], "task-1");
    assert_eq!(assignment["root"]["body"], root_show["body"]);
    assert_eq!(
        assignment["root"]["body"],
        format!("\n## Description\n\n{long_body}\n")
    );
    assert_eq!(
        assignment["root"]["acceptance"],
        serde_json::json!(["root acceptance"])
    );
    assert_eq!(
        assignment["root"]["constraints"],
        serde_json::json!(["root constraint"])
    );
    assert_eq!(
        assignment["root"]["plannedValidation"],
        serde_json::json!(["root validation"])
    );
    assert_eq!(
        assignment["root"]["ownedScope"],
        serde_json::json!(["src/root.rs"])
    );
    let milestones = assignment["milestones"].as_array().unwrap();
    assert_eq!(milestones.len(), 10);
    for index in 1..=10 {
        let id = format!("task-1-{index}");
        let body = format!("milestone body {index} MILESTONE-TAIL-{index}");
        let milestone = milestones.iter().find(|item| item["id"] == id).unwrap();
        let child_show = data(&root_dir, &["show", &id, "--json"]);
        assert_eq!(milestone["body"], child_show["body"]);
        assert_eq!(milestone["body"], format!("\n## Description\n\n{body}\n"));
        assert_eq!(
            milestone["acceptance"],
            serde_json::json!([format!("acceptance {index}")])
        );
        assert_eq!(
            milestone["constraints"],
            serde_json::json!([format!("constraint {index}")])
        );
        assert_eq!(
            milestone["plannedValidation"],
            serde_json::json!([format!("validation {index}")])
        );
        assert_eq!(
            milestone["ownedScope"],
            serde_json::json!([format!("src/milestone-{index}.rs")])
        );
    }
    assert!(assignment["dependencyReadiness"]["allClear"]
        .as_bool()
        .unwrap());
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn assignment_token_changes_only_for_current_definition_scope() {
    let root_dir = root("freshness");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "assignment"]);
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Root",
            "--acceptance",
            "accept",
            "--constraint",
            "constraint",
            "--validation",
            "validation",
            "--related-file",
            "src/root.rs",
            "--body",
            "root body",
        ],
    );
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Milestone",
            "--parent",
            "task-1",
            "--acceptance",
            "milestone accept",
            "--constraint",
            "milestone constraint",
            "--validation",
            "milestone validation",
            "--related-file",
            "src/milestone.rs",
            "--body",
            "milestone body",
        ],
    );
    run(
        &root_dir,
        &["add", "task", "Root blocker", "--acceptance", "accept"],
    );
    run(
        &root_dir,
        &["add", "task", "Milestone blocker", "--acceptance", "accept"],
    );
    let token = |root_dir: &std::path::Path| {
        data(root_dir, &["assignment", "task-1", "--json"])["definitionToken"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let mut previous = token(&root_dir);
    run(&root_dir, &["update", "task-1", "--blocker", "task-2"]);
    assert_ne!(
        token(&root_dir),
        previous,
        "root dependency scope must refresh token"
    );
    previous = token(&root_dir);
    run(&root_dir, &["update", "task-1-1", "--blocker", "task-3"]);
    assert_ne!(
        token(&root_dir),
        previous,
        "milestone dependency scope must refresh token"
    );
    run(&root_dir, &["update", "task-1", "--clear", "blockers"]);
    run(&root_dir, &["update", "task-1-1", "--clear", "blockers"]);
    previous = token(&root_dir);
    for args in [
        vec!["update", "task-1", "--body", "root body changed"],
        vec![
            "update",
            "task-1",
            "--acceptance",
            "root acceptance changed",
        ],
        vec![
            "update",
            "task-1",
            "--constraint",
            "root constraint changed",
        ],
        vec![
            "update",
            "task-1",
            "--validation",
            "root validation changed",
        ],
        vec!["update", "task-1", "--related-file", "src/other-root.rs"],
        vec!["update", "task-1-1", "--body", "milestone body changed"],
        vec![
            "update",
            "task-1-1",
            "--acceptance",
            "milestone acceptance changed",
        ],
        vec![
            "update",
            "task-1-1",
            "--constraint",
            "milestone constraint changed",
        ],
        vec![
            "update",
            "task-1-1",
            "--validation",
            "milestone validation changed",
        ],
        vec![
            "update",
            "task-1-1",
            "--related-file",
            "src/other-milestone.rs",
        ],
    ] {
        run(&root_dir, &args);
        let current = token(&root_dir);
        assert_ne!(
            current, previous,
            "scope edit {:?} did not refresh token",
            args
        );
        previous = current;
    }

    let stable = token(&root_dir);
    run(
        &root_dir,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    assert_eq!(token(&root_dir), stable, "claim is progress, not scope");
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "progress",
            "--evidence",
            "observed",
        ],
    );
    assert_eq!(
        token(&root_dir),
        stable,
        "delivery evidence is progress, not scope"
    );
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Unrelated",
            "--acceptance",
            "unrelated acceptance",
            "--body",
            "one",
        ],
    );
    assert_eq!(
        token(&root_dir),
        stable,
        "unrelated Task is not assignment scope"
    );
    run(
        &root_dir,
        &["update", "task-4", "--body", "unrelated changed"],
    );
    assert_eq!(
        token(&root_dir),
        stable,
        "unrelated Task edit is not assignment scope"
    );

    run(
        &root_dir,
        &["accord", "claim", "task-1-1", "--assignee", "worker"],
    );
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1-1",
            "--summary",
            "milestone done",
            "--evidence",
            "tested",
        ],
    );
    run(&root_dir, &["complete", "task-1-1"]);
    let archived = data(&root_dir, &["assignment", "task-1", "--json"]);
    assert_eq!(archived["definitionToken"], stable);
    assert_eq!(archived["milestones"][0]["location"], "logs");
    assert_eq!(archived["milestones"][0]["resolutionOutcome"], "completed");
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn assignment_token_is_stable_when_progress_reorders_multiple_milestones() {
    let root_dir = root("progress-order");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "assignment"]);
    run(
        &root_dir,
        &["add", "task", "Root", "--acceptance", "accept"],
    );
    for index in 1..=3 {
        run(
            &root_dir,
            &[
                "add",
                "task",
                &format!("Milestone {index}"),
                "--parent",
                "task-1",
                "--acceptance",
                "accept",
            ],
        );
    }
    let token = || {
        data(&root_dir, &["assignment", "task-1", "--json"])["definitionToken"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let stable = token();

    run(
        &root_dir,
        &["accord", "claim", "task-1-2", "--assignee", "worker"],
    );
    assert_eq!(token(), stable, "claim must not hash presentation order");
    run(
        &root_dir,
        &["accord", "claim", "task-1-1", "--assignee", "worker"],
    );
    run(
        &root_dir,
        &["accord", "block", "task-1-1", "--note", "waiting"],
    );
    assert_eq!(token(), stable, "block must not hash presentation order");
    run(&root_dir, &["accord", "resume", "task-1-1"]);
    assert_eq!(token(), stable, "resume must not hash presentation order");
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1-2",
            "--summary",
            "delivered",
            "--evidence",
            "tested",
        ],
    );
    assert_eq!(token(), stable, "deliver must not hash presentation order");
    run(
        &root_dir,
        &["accord", "claim", "task-1-3", "--assignee", "worker"],
    );
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-1-3",
            "--summary",
            "done",
            "--evidence",
            "tested",
        ],
    );
    run(&root_dir, &["complete", "task-1-3"]);
    let archived = data(&root_dir, &["assignment", "task-1", "--json"]);
    assert_eq!(archived["definitionToken"], stable);
    assert_eq!(archived["milestones"].as_array().unwrap().len(), 3);
    assert_eq!(
        archived["milestones"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "task-1-3")
            .unwrap()["location"],
        "logs"
    );
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn assignment_readiness_reports_exact_native_blocker_reasons() {
    let root_dir = root("readiness");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "assignment"]);
    run(
        &root_dir,
        &["add", "task", "Assignment", "--acceptance", "accept"],
    );
    for title in ["Active", "Completed", "Canceled", "Failed", "Reference"] {
        run(&root_dir, &["add", "task", title, "--acceptance", "accept"]);
    }
    run(
        &root_dir,
        &[
            "accord",
            "deliver",
            "task-3",
            "--summary",
            "done",
            "--evidence",
            "tested",
        ],
    );
    run(&root_dir, &["complete", "task-3"]);
    run(&root_dir, &["cancel", "task-4", "--note", "not needed"]);
    run(&root_dir, &["accord", "fail", "task-5", "--note", "failed"]);
    run(
        &root_dir,
        &[
            "update",
            "task-1",
            "--blocker",
            "task-2",
            "--blocker",
            "task-3",
            "--blocker",
            "task-4",
            "--blocker",
            "task-5",
            "--reference",
            "task-6",
        ],
    );
    let task_path = root_dir.join(".tandem/tasks/task-1.md");
    let source = std::fs::read_to_string(&task_path).unwrap();
    std::fs::write(
        &task_path,
        source.replace(
            "blockers: [\"task-2\", \"task-3\", \"task-4\", \"task-5\"]",
            "blockers: [\"task-2\", \"task-3\", \"task-4\", \"task-5\", \"task-99\"]",
        ),
    )
    .unwrap();

    let assignment = data(&root_dir, &["assignment", "task-1", "--json"]);
    let dependencies = assignment["root"]["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 5);
    assert_eq!(
        dependencies[0],
        serde_json::json!({"id":"task-2","status":"active","ready":false,"reason":"active blocker"})
    );
    assert_eq!(
        dependencies[1],
        serde_json::json!({"id":"task-3","status":"completed","ready":true,"reason":"archived blocker: completed"})
    );
    assert_eq!(
        dependencies[2],
        serde_json::json!({"id":"task-4","status":"canceled","ready":true,"reason":"archived blocker: canceled"})
    );
    assert_eq!(
        dependencies[3],
        serde_json::json!({"id":"task-5","status":"failed","ready":true,"reason":"archived blocker: failed"})
    );
    assert_eq!(
        dependencies[4],
        serde_json::json!({"id":"task-99","status":"missing","ready":false,"reason":"missing blocker"})
    );
    assert_eq!(assignment["dependencyReadiness"]["allClear"], false);
    assert_eq!(
        assignment["dependencyReadiness"]["issues"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        assignment["dependencyReadiness"]["issues"][0]["ownerId"],
        "task-1"
    );
    assert_eq!(
        assignment["dependencyReadiness"]["issues"][0]["id"],
        "task-2"
    );
    assert_eq!(
        assignment["dependencyReadiness"]["issues"][1]["id"],
        "task-99"
    );
    assert!(dependencies
        .iter()
        .all(|dependency| dependency["id"] != "task-6"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn assignment_human_warnings_stay_on_stderr() {
    let root_dir = root("human-warnings");
    std::fs::create_dir_all(&root_dir).unwrap();
    run(&root_dir, &["init", "--title", "assignment"]);
    run(
        &root_dir,
        &[
            "add",
            "task",
            "Root",
            "--acceptance",
            "accept",
            "--reference",
            "missing-reference",
        ],
    );
    let output = bin()
        .current_dir(&root_dir)
        .args(["assignment", "task-1"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Assignment task-1"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Warning:"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Warning:"));
    std::fs::remove_dir_all(root_dir).unwrap();
}
