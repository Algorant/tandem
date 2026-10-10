use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

#[test]
fn landing_and_help_need_no_workspace() {
    let landing = bin().output().unwrap();
    assert!(landing.status.success());
    let landing_text = String::from_utf8_lossy(&landing.stdout);
    assert!(landing_text.contains("Work\n"));
    assert!(landing_text.contains("Agreements\n"));
    assert!(landing_text.contains("Workspace\n"));
    assert!(landing_text.contains("accord claim         Claim a task"));
    assert!(landing_text.contains("review               Request exceptional human validation"));
    assert!(landing_text.contains("rules list|add|edit|delete  Manage project rules"));
    assert!(landing_text.contains("sync                 Sync the board with the tandem branch"));
    assert!(landing_text.contains("migrate              Upgrade a 0.3.0 or 0.4.0 board"));
    assert!(landing_text.contains("Run 'tandem <command> --help' for detailed usage."));
    let help = bin().arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
}

#[test]
fn json_usage_errors_are_stdout_only() {
    let output = bin().args(["--json", "unknown-command"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("\"ok\":false") && text.contains("\"code\":\"usage\""));
    assert!(output.stderr.is_empty());
}

#[test]
fn generated_help_covers_exact_target_surfaces_without_workspace() {
    let surfaces: Vec<Vec<&str>> = vec![
        vec!["--help"],
        vec!["init", "--help"],
        vec!["add", "--help"],
        vec!["add", "task", "--help"],
        vec!["add", "decision", "--help"],
        vec!["show", "--help"],
        vec!["list", "--help"],
        vec!["search", "--help"],
        vec!["update", "--help"],
        vec!["accord", "--help"],
        vec!["accord", "claim", "--help"],
        vec!["accord", "deliver", "--help"],
        vec!["accord", "rework", "--help"],
        vec!["accord", "block", "--help"],
        vec!["accord", "resume", "--help"],
        vec!["accord", "release", "--help"],
        vec!["accord", "fail", "--help"],
        vec!["review", "--help"],
        vec!["complete", "--help"],
        vec!["cancel", "--help"],
        vec!["sync", "--help"],
        vec!["sync", "status", "--help"],
        vec!["sync", "resolve", "--help"],
        vec!["migrate", "--help"],
        vec!["rules", "--help"],
        vec!["rules", "list", "--help"],
        vec!["rules", "add", "--help"],
        vec!["rules", "edit", "--help"],
        vec!["rules", "delete", "--help"],
        vec!["tui", "--help"],
        vec!["web", "--help"],
    ];
    assert_eq!(surfaces.len(), 31);
    for argv in surfaces {
        let output = bin().args(argv.iter()).output().unwrap();
        assert!(output.status.success(), "{argv:?}: {:?}", output.stderr);
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
}

#[test]
fn process_stream_and_exit_contracts_are_semantic() {
    let human = bin().args(["list", "--unknown"]).output().unwrap();
    assert_eq!(human.status.code(), Some(2));
    assert!(human.stdout.is_empty());
    assert!(String::from_utf8_lossy(&human.stderr).starts_with("Error:"));

    let json = bin()
        .args(["--json", "list", "--unknown"])
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(2));
    assert!(json.stderr.is_empty());
    assert!(String::from_utf8_lossy(&json.stdout).contains("\"code\":\"usage\""));

    let missing_dir =
        std::env::temp_dir().join(format!("tandem-cli-missing-{}", std::process::id()));
    std::fs::create_dir_all(&missing_dir).unwrap();
    let missing = bin()
        .current_dir(&missing_dir)
        .args(["list"])
        .output()
        .unwrap();
    std::fs::remove_dir_all(&missing_dir).unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("No Tandem workspace"));

    for argv in [
        ["--json", "--help"],
        ["--json", "--version"],
        ["--help", "--json"],
    ] {
        let output = bin().args(argv).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains("tandem"));
    }
}

#[test]
fn removed_commands_fail_usage() {
    for command in ["move", "upgrade", "version", "log", "papercut", "decision"] {
        let output = bin().arg(command).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{command}");
    }
}

#[test]
fn show_json_returns_the_full_record_body_and_location() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "tandem-cli-show-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let run = |args: &[&str]| {
        let output = bin().args(args).current_dir(&root).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let data = |args: &[&str]| -> serde_json::Value {
        let text = run(args);
        serde_json::from_str::<serde_json::Value>(&text).unwrap()["data"].clone()
    };

    run(&["init", "--title", "show contract"]);
    run(&[
        "add",
        "task",
        "alpha",
        "--acceptance",
        "criterion one exactly",
        "--body",
        "real body text",
    ]);
    run(&["add", "decision", "a choice", "--body", "decision body"]);
    run(&["accord", "claim", "task-1", "--assignee", "worker-a"]);

    let claimed = data(&["show", "task-1", "--json"]);
    assert_eq!(
        claimed["accord"]["acceptance"][0], "criterion one exactly",
        "a claimed task must still report its acceptance criteria: {claimed}"
    );
    assert!(claimed["body"].as_str().unwrap().contains("real body text"));
    assert_eq!(claimed["location"], "board");
    assert_eq!(claimed["state"], "in-progress");
    assert_eq!(claimed["accordStatus"], "claimed");

    let decision = data(&["show", "decision-1", "--json"]);
    assert_eq!(decision["location"], "board");
    assert_eq!(decision["type"], "decision");
    assert!(decision["decision"].is_object());

    run(&[
        "accord",
        "deliver",
        "task-1",
        "--summary",
        "s",
        "--evidence",
        "e",
    ]);
    run(&["complete", "task-1"]);
    let archived = data(&["show", "task-1", "--json"]);
    assert_eq!(
        archived["location"], "logs",
        "location is the archived signal: {archived}"
    );
    assert_eq!(archived["accord"]["acceptance"][0], "criterion one exactly");

    std::fs::remove_dir_all(&root).unwrap();
}

fn clear_parent_workspace(name: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dir = std::env::temp_dir().join(format!(
        "tandem-cli-clear-parent-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let init = bin()
        .current_dir(&dir)
        .args(["init", "--title", "clear parent"])
        .output()
        .unwrap();
    assert!(init.status.success());
    dir
}

fn clear_parent_run(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    bin().current_dir(dir).args(args).output().unwrap()
}

fn clear_parent_add(dir: &std::path::Path, args: &[&str]) -> String {
    let mut argv = vec!["--json", "add", "task"];
    argv.extend(args);
    let output = clear_parent_run(dir, &argv);
    assert!(
        output.status.success(),
        "{argv:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["data"]["id"].as_str().unwrap().to_string()
}

fn clear_parent_record(dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    let board = dir.join(".tandem").join("tasks");
    std::fs::read_dir(&board)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .any(|line| line.trim() == format!("id: {id}"))
        })
        .unwrap_or_else(|| panic!("no board record for {id}"))
}

#[test]
fn update_clear_parent_on_a_subtask_is_refused_without_writing() {
    let dir = clear_parent_workspace("subtask");
    let parent = clear_parent_add(&dir, &["Parent", "--acceptance", "ok"]);
    let subtask = clear_parent_add(&dir, &["Child", "--parent", &parent, "--acceptance", "ok"]);
    assert_eq!(subtask, format!("{parent}-1"));
    let record = clear_parent_record(&dir, &subtask);
    let before = std::fs::read_to_string(&record).unwrap();

    let output = clear_parent_run(&dir, &["update", &subtask, "--clear", "parent"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("would change its canonical role from subtask to task")
            && stderr.contains("IDs are immutable"),
        "{stderr}"
    );
    assert_eq!(std::fs::read_to_string(&record).unwrap(), before);

    // The board stays readable and still accepts valid updates.
    assert!(clear_parent_run(&dir, &["show", &subtask]).status.success());
    assert!(
        clear_parent_run(&dir, &["update", &subtask, "--priority", "high"])
            .status
            .success()
    );
    assert!(clear_parent_run(&dir, &["list"]).status.success());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn update_clear_parent_on_root_task_and_epic_task_still_works() {
    let dir = clear_parent_workspace("task");
    let root_task = clear_parent_add(&dir, &["Root", "--acceptance", "ok"]);
    let before = std::fs::read_to_string(clear_parent_record(&dir, &root_task)).unwrap();
    let noop = clear_parent_run(&dir, &["update", &root_task, "--clear", "parent"]);
    assert!(noop.status.success());
    assert_eq!(
        std::fs::read_to_string(clear_parent_record(&dir, &root_task)).unwrap(),
        before
    );

    let epic = clear_parent_add(&dir, &["Epic", "--kind", "epic", "--acceptance", "ok"]);
    let child = clear_parent_add(&dir, &["Child", "--parent", &epic, "--acceptance", "ok"]);
    assert!(!child.contains('-') || child.matches('-').count() == 1);
    let record = clear_parent_record(&dir, &child);
    assert!(std::fs::read_to_string(&record)
        .unwrap()
        .contains("parentId:"));
    let cleared = clear_parent_run(&dir, &["update", &child, "--clear", "parent"]);
    assert!(
        cleared.status.success(),
        "{}",
        String::from_utf8_lossy(&cleared.stderr)
    );
    assert!(!std::fs::read_to_string(&record)
        .unwrap()
        .contains("parentId:"));
    assert!(clear_parent_run(&dir, &["show", &child]).status.success());
    std::fs::remove_dir_all(&dir).unwrap();
}
