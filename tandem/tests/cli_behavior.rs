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
    assert!(landing_text.contains("checkpoint           Flush pending .tandem changes into Git"));
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
        vec!["checkpoint", "--help"],
        vec!["rules", "--help"],
        vec!["rules", "list", "--help"],
        vec!["rules", "add", "--help"],
        vec!["rules", "edit", "--help"],
        vec!["rules", "delete", "--help"],
        vec!["tui", "--help"],
        vec!["web", "--help"],
    ];
    assert_eq!(surfaces.len(), 28);
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
