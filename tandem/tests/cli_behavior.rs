use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_PROJECT_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(label: &str) -> Self {
        let unique = TEMP_PROJECT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!(
            "tandem-cli-behavior-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary project directory");
        Self { root }
    }

    fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_tandem"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("run tandem executable");
        Run::from_output(output)
    }

    fn init(&self) {
        self.run(&["init", "--title", "Behavior project"])
            .assert_success();
    }

    fn read(&self, relative: impl AsRef<Path>) -> String {
        fs::read_to_string(self.path(relative)).expect("read temporary project file")
    }

    fn actor_events(&self) -> String {
        let mut paths = fs::read_dir(self.path(".tandem/events"))
            .expect("read actor event directory")
            .map(|entry| entry.expect("read actor event entry").path())
            .collect::<Vec<_>>();
        paths.sort();
        paths
            .into_iter()
            .map(|path| fs::read_to_string(path).expect("read actor event file"))
            .collect()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Run {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl Run {
    fn from_output(output: Output) -> Self {
        Self {
            status: output.status,
            stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
            stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
        }
    }

    fn assert_success(&self) -> &Self {
        assert!(
            self.status.success(),
            "expected success, got {:?}\nstdout:\n{}\nstderr:\n{}",
            self.status,
            self.stdout,
            self.stderr
        );
        self
    }

    fn assert_exit(&self, code: i32) -> &Self {
        assert_eq!(
            self.status.code(),
            Some(code),
            "stdout:\n{}\nstderr:\n{}",
            self.stdout,
            self.stderr
        );
        self
    }
}

#[test]
fn process_help_version_usage_and_missing_project_contracts() {
    let project = TempProject::new("process-contracts");

    let help = project.run(&["--help"]);
    help.assert_success();
    assert!(help.stdout.contains("tandem - Tandem CLI"));
    assert!(help.stdout.contains("tandem accord claim|deliver|accept"));
    assert!(help.stderr.is_empty());

    let version = project.run(&["--version"]);
    version.assert_success();
    assert_eq!(
        version.stdout,
        format!("tandem {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(version.stderr.is_empty());

    let unknown_command = project.run(&["unknown-command"]);
    unknown_command.assert_exit(2);
    assert!(unknown_command.stdout.is_empty());
    assert!(unknown_command
        .stderr
        .contains("unknown command `unknown-command`"));

    let unknown_flag = project.run(&["list", "--unknown"]);
    unknown_flag.assert_exit(2);
    assert!(unknown_flag.stdout.is_empty());
    assert!(unknown_flag
        .stderr
        .contains("unknown list flag `--unknown`"));

    let missing_project = project.run(&["list"]);
    missing_project.assert_exit(1);
    assert!(missing_project.stdout.is_empty());
    assert!(missing_project
        .stderr
        .contains("No Tandem workspace found. Run `tandem init` first."));
}

#[test]
fn protocol_upgrade_requires_explicit_conversion_and_preserves_legacy_content() {
    let project = TempProject::new("protocol-upgrade");
    project.init();
    let config_path = project.path(".tandem/tandem.md");
    let legacy_config = project
        .read(".tandem/tandem.md")
        .replacen("protocolVersion: 0.2.0", "protocolVersion: 0.1.0", 1)
        .replacen(
            "rules:\n",
            "types:\n  note:\n    idPrefix: note\ncompletion:\n  requireReview: true\nrules:\n",
            1,
        );
    fs::write(&config_path, legacy_config.clone()).expect("seed legacy protocol config");
    fs::write(
        project.path(".tandem/board/note-1.md"),
        "---\nid: note-1\ntype: note\ntitle: Legacy note\nstate: todo\neffort: xlarge\nunknownField: preserve\n---\n\nLegacy custom body.\n",
    )
    .expect("seed legacy custom document");
    let event_before = "{\"event\":\"legacy.event\",\"id\":\"note-1\"}\n";
    fs::write(project.path(".tandem/events.jsonl"), event_before).expect("seed legacy event");
    let log_before = "---\nid: task-99\ntype: task\ntitle: Historical log\ncompletedAt: 2026-01-01T00:00:00Z\n---\n\nHistorical body.\n";
    fs::write(project.path(".tandem/logs/task-99.md"), log_before).expect("seed historical log");

    let refused = project.run(&["list"]);
    refused.assert_exit(1);
    assert!(refused.stdout.is_empty());
    assert!(refused.stderr.contains("Protocol 0.1.0 project detected"));
    assert!(refused.stderr.contains("tandem upgrade"));
    project.run(&["--help"]).assert_success();
    project.run(&["help"]).assert_success();
    project.run(&["--version"]).assert_success();
    project.run(&["version"]).assert_success();
    let refused_init = project.run(&["init"]);
    refused_init.assert_exit(1);
    assert!(refused_init
        .stderr
        .contains("Protocol 0.1.0 project detected"));

    let upgrade = project.run(&["upgrade"]);
    upgrade.assert_success();
    assert!(upgrade.stdout.contains("0.1.0 -> 0.2.0"));
    assert!(upgrade.stdout.contains("without conversion"));
    let upgraded_config = project.read(".tandem/tandem.md");
    assert!(upgraded_config.contains("protocolVersion: \"0.2.0\""));
    assert!(upgraded_config.contains("idPrefix: note"));
    assert!(upgraded_config.contains("requireReview: true"));
    assert_eq!(project.read(".tandem/board/note-1.md"), "---\nid: note-1\ntype: note\ntitle: Legacy note\nstate: todo\neffort: xlarge\nunknownField: preserve\n---\n\nLegacy custom body.\n");
    assert_eq!(project.read(".tandem/events.jsonl"), event_before);
    assert_eq!(project.read(".tandem/logs/task-99.md"), log_before);

    let listed = project.run(&["list"]);
    listed.assert_success();
    assert!(listed.stdout.contains("note-1"));
    assert!(listed
        .stdout
        .contains("custom type declarations are deprecated"));
    assert!(listed.stdout.contains("legacy custom type `note`"));
    let shown = project.run(&["show", "note-1", "--json"]);
    shown.assert_success();
    assert!(shown.stdout.contains("\"type\":\"note\""));
    assert!(shown.stdout.contains("\"effort\":\"xlarge\""));
    let searched = project.run(&["search", "Legacy"]);
    searched.assert_success();
    assert!(searched.stdout.contains("note-1"));

    let mutation = project.run(&["move", "note-1", "--state", "in-progress"]);
    mutation.assert_exit(1);
    assert!(mutation.stderr.contains("only task documents can be moved"));
    let completion = project.run(&["complete", "note-1", "--summary", "Nope"]);
    completion.assert_exit(1);
    assert!(completion
        .stderr
        .contains("only task documents can be completed"));

    let canonical = project.run(&[
        "add",
        "--title",
        "Canonical task",
        "--priority",
        "high",
        "--effort",
        "small",
    ]);
    canonical.assert_success();
    assert!(canonical.stdout.contains("ID:    task-100"));
    let task = project.read(".tandem/board/task-100.md");
    assert!(task.contains("priority: \"high\""));
    assert!(task.contains("effort: \"small\""));
    let invalid_priority = project.run(&["add", "--title", "Bad priority", "--priority", "urgent"]);
    invalid_priority.assert_exit(1);
    assert!(invalid_priority
        .stderr
        .contains("invalid priority `urgent`"));
    let invalid_effort = project.run(&["update", "task-100", "--effort", "xlarge"]);
    invalid_effort.assert_exit(1);
    assert!(invalid_effort.stderr.contains("invalid effort `xlarge`"));

    let complete = project.run(&["complete", "task-100", "--summary", "Canonical completion"]);
    complete.assert_success();
    assert!(complete
        .stdout
        .contains("completion-policy settings are deprecated and ignored"));
    assert!(complete
        .stdout
        .contains("Completing anyway under the canonical protocol policy."));
}

#[test]
fn reads_mutations_accords_events_and_preservation_use_the_real_command() {
    let project = TempProject::new("mutations");
    project.init();

    let added = project.run(&[
        "add",
        "--title",
        "Preserved task",
        "--description",
        "Initial body",
        "--json",
    ]);
    added.assert_success();
    assert!(added.stdout.contains("\"id\":\"task-1\""));
    assert!(added.stdout.contains("\"state\":\"todo\""));
    assert!(added.stderr.is_empty());

    let task_path = project.path(".tandem/board/task-1.md");
    let original = fs::read_to_string(&task_path).expect("read created task");
    let preserved = original.replacen(
        "updatedAt:",
        "unknownField: \"preserve this\"\nupdatedAt:",
        1,
    );
    let preserved = preserved.replacen(
        "## Description\n\nInitial body",
        "## Exact body\n\nKeep this body exactly.",
        1,
    );
    fs::write(&task_path, preserved).expect("seed unknown field and markdown body");

    let moved = project.run(&["move", "task-1", "--state", "in-progress"]);
    moved.assert_success();
    assert!(moved.stdout.contains("Moved task-1"));
    let after_move = project.read(".tandem/board/task-1.md");
    assert!(after_move.contains("unknownField: \"preserve this\""));
    assert!(after_move.contains("## Exact body\n\nKeep this body exactly."));
    assert!(after_move.contains("state: \"in-progress\""));

    let claimed = project.run(&["accord", "claim", "task-1", "--assignee", "worker"]);
    claimed.assert_success();
    assert!(claimed.stdout.contains("Updated accord"));
    assert!(claimed.stdout.contains("From:   missing"));
    assert!(claimed.stdout.contains("To:     claimed"));

    let updated = project.run(&["update", "task-1", "--title", "Renamed task"]);
    updated.assert_success();
    assert!(updated.stdout.contains("Updated task-1"));
    let after_update = project.read(".tandem/board/task-1.md");
    assert!(after_update.contains("title: \"Renamed task\""));
    assert!(after_update.contains("unknownField: \"preserve this\""));
    assert!(after_update.contains("## Exact body\n\nKeep this body exactly."));

    let delivered = project.run(&[
        "accord",
        "deliver",
        "task-1",
        "--summary",
        "Ready for review",
        "--evidence",
        "cargo test",
    ]);
    delivered.assert_success();
    assert!(delivered.stdout.contains("Updated accord"));
    assert!(delivered.stdout.contains("From:   claimed"));
    assert!(delivered.stdout.contains("To:     delivered"));
    assert!(delivered
        .stdout
        .contains("State:  in-progress -> validation"));

    let human_read = project.run(&["show", "task-1"]);
    human_read.assert_success();
    assert!(human_read.stdout.contains("State:     validation"));
    assert!(human_read.stdout.contains("Accord:    delivered"));

    let json_read = project.run(&["list", "--json"]);
    json_read.assert_success();
    assert!(json_read.stdout.contains("\"ok\":true"));
    assert!(json_read.stdout.contains("\"items\""));
    assert!(json_read.stdout.contains("\"id\":\"task-1\""));
    assert!(json_read.stdout.contains("\"state\":\"validation\""));

    let events = project.actor_events();
    assert!(events.contains("\"event\":\"task.created\""));
    assert!(events.contains("\"event\":\"task.moved\""));
    assert!(events.contains("\"event\":\"accord.claimed\""));
    assert!(events.contains("\"event\":\"task.updated\""));
    assert!(events.contains("\"event\":\"accord.delivered\""));
}

#[test]
fn hierarchy_validation_completion_and_logs_remain_observable_at_the_process_boundary() {
    let project = TempProject::new("hierarchy-and-logs");
    project.init();

    let epic = project.run(&["add", "--title", "Epic", "--kind", "epic"]);
    epic.assert_success();
    assert!(epic.stdout.contains("ID:    task-1"));

    let task = project.run(&["add", "--title", "Epic task", "--parent", "task-1"]);
    task.assert_success();
    assert!(task.stdout.contains("ID:    task-2"));
    assert!(task.stdout.contains("Task of Epic: task-1"));

    let subtask = project.run(&["add", "--title", "Task subtask", "--parent", "task-2"]);
    subtask.assert_success();
    assert!(subtask.stdout.contains("Created subtask"));
    assert!(subtask.stdout.contains("ID:    task-2-1"));

    let invalid_child = project.run(&["add", "--title", "Nested child", "--parent", "task-2-1"]);
    invalid_child.assert_exit(1);
    assert!(invalid_child.stdout.is_empty());
    assert!(invalid_child
        .stderr
        .contains("cannot attach a child beneath Subtask task-2-1"));

    let missing_document = project.run(&["show", "task-404"]);
    missing_document.assert_exit(1);
    assert!(missing_document.stdout.is_empty());
    assert!(missing_document
        .stderr
        .contains("document not found: task-404"));

    let completed = project.run(&[
        "complete",
        "task-2-1",
        "--summary",
        "Finished hierarchy leaf",
        "--validation",
        "cargo test",
        "--reviewer",
        "reviewer",
    ]);
    completed.assert_success();
    assert!(completed
        .stdout
        .contains("Warning: task-2-1 has review.status=missing."));
    assert!(completed
        .stdout
        .contains("Warning: task-2-1 has accord.status=missing, not accepted."));
    assert!(completed.stdout.contains("Completed task-2-1"));
    assert!(!project.path(".tandem/board/task-2-1.md").exists());

    let log = project.read(".tandem/logs/task-2-1.md");
    assert!(log.contains("summary: \"Finished hierarchy leaf\""));
    assert!(log.contains("validation: \"cargo test\""));
    assert!(log.contains("reviewer: \"reviewer\""));
    assert!(log.contains("completion:"));

    let logs = project.run(&["log", "list", "--json"]);
    logs.assert_success();
    assert!(logs.stdout.contains("\"id\":\"task-2-1\""));
    assert!(logs.stdout.contains("\"outcome\":\"completed\""));

    let events = project.actor_events();
    assert!(events.contains("\"event\":\"task.completed\""));
}
