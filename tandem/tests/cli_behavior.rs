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
    assert_eq!(
        help.stdout,
        concat!(
            "tandem - Tandem CLI\n\n",
            "Usage:\n",
            "  tandem init [--title <title>]\n",
            "  tandem upgrade\n",
            "  tandem list [--state <state>] [--type <type>] [--parent <id>] [--json]\n",
            "  tandem show <id> [--json]\n",
            "  tandem add --title <title> [--state <state>] [--kind epic] [--parent <id>] [--description <text>] [--priority <priority>] [--effort <effort>] [--json]\n",
            "  tandem move <id> --state <state>\n",
            "  tandem update <id> [--title <title>] [--body <markdown>] [--kind epic] [--parent <id>] [--priority <priority>] [--effort <effort>] ...\n",
            "  tandem complete <id> --summary <text>\n",
            "  tandem cancel <id> --reason <text>\n",
            "  tandem search <query> [--state <state>] [--type <type>] [--parent <id>] [--json]\n",
            "  tandem log list|show|search ...\n",
            "  tandem accord claim|deliver|accept|rework|block|fail ...\n",
            "  tandem rules list|add|edit|delete ...\n",
            "  tandem decision list|show|add ... [--status <status>] [--date <date>]\n",
            "  tandem tui\n",
            "  tandem version\n",
            "  tandem --version\n",
        )
    );
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
    assert_eq!(unknown_command.stderr, "Error: unknown command `unknown-command`. Supported commands: init, upgrade, list, show, add, move, update, complete, cancel, search, log, accord, rules, decision, tui, version\n");

    let unknown_flag = project.run(&["list", "--unknown"]);
    unknown_flag.assert_exit(2);
    assert!(unknown_flag.stdout.is_empty());
    assert_eq!(
        unknown_flag.stderr,
        "Error: unknown list flag `--unknown`\n"
    );

    let missing_project = project.run(&["list"]);
    missing_project.assert_exit(1);
    assert!(missing_project.stdout.is_empty());
    assert_eq!(
        missing_project.stderr,
        "Error: No Tandem workspace found. Run `tandem init` first.\n"
    );

    let invalid_tui = project.run(&["tui", "--json"]);
    invalid_tui.assert_exit(2);
    assert!(invalid_tui.stdout.is_empty());
    assert_eq!(invalid_tui.stderr, "Error: tui does not accept arguments\n");
}

#[test]
fn command_families_preserve_exact_success_output() {
    let project = TempProject::new("exact-output");

    let init = project.run(&["init", "--title", "Exact"]);
    init.assert_success();
    assert_eq!(init.stdout, "Created Tandem workspace\nTitle: Exact\nConfig: .tandem/tandem.md\nBoard:  .tandem/board\nLogs:   .tandem/logs\nEvents: .tandem/events\nStates: todo, in-progress, validation\n");
    assert!(init.stderr.is_empty());

    let add = project.run(&["add", "--title", "First", "--json"]);
    add.assert_success();
    assert_eq!(add.stdout, "{\"ok\":true,\"data\":{\"document\":{\"id\":\"task-1\",\"type\":\"task\",\"state\":\"todo\",\"title\":\"First\",\"path\":\".tandem/board/task-1.md\"}},\"warnings\":[]}\n");
    assert!(add.stderr.is_empty());

    let list = project.run(&["list", "--json"]);
    list.assert_success();
    assert_eq!(list.stdout, "{\"ok\":true,\"data\":{\"items\":[{\"id\":\"task-1\",\"type\":\"task\",\"title\":\"First\",\"state\":\"todo\"}],\"counts\":{\"total\":1,\"byState\":{\"todo\":1}}},\"warnings\":[]}\n");

    let moved = project.run(&["move", "task-1", "--state", "in-progress"]);
    moved.assert_success();
    assert_eq!(
        moved.stdout,
        "Moved task-1\nFrom: todo\nTo:   in-progress\nPath: .tandem/board/task-1.md\n"
    );

    let accord = project.run(&["accord", "claim", "task-1", "--assignee", "worker"]);
    accord.assert_success();
    assert_eq!(accord.stdout, "Updated accord\nID:     task-1\nFrom:   missing\nTo:     claimed\nPath:   .tandem/board/task-1.md\nEvent:  accord.claimed\n");

    let rule = project.run(&["rules", "add", "--category", "always", "--rule", "Test"]);
    rule.assert_success();
    assert_eq!(
        rule.stdout,
        "Added rule\nCategory: always\nID:       1\nRule:     Test\n"
    );
    let rules = project.run(&["rules", "list", "--json"]);
    rules.assert_success();
    assert_eq!(rules.stdout, "{\"ok\":true,\"data\":{\"rules\":{\"always\":[{\"id\":1,\"rule\":\"Test\"}],\"never\":[],\"prefer\":[],\"context\":[]},\"counts\":{\"always\":1,\"never\":0,\"prefer\":0,\"context\":0,\"total\":1}},\"warnings\":[]}\n");

    let decision = project.run(&[
        "decision",
        "add",
        "--title",
        "Choice",
        "--date",
        "2026-01-02",
    ]);
    decision.assert_success();
    assert_eq!(decision.stdout, "Created decision\nID:     decision-1\nStatus: proposed\nDate:   2026-01-02\nTitle:  Choice\nPath:   .tandem/board/decision-1.md\n");
    let decisions = project.run(&["decision", "list", "--json"]);
    decisions.assert_success();
    assert_eq!(decisions.stdout, "{\"ok\":true,\"data\":{\"items\":[{\"id\":\"decision-1\",\"type\":\"decision\",\"title\":\"Choice\",\"status\":\"proposed\",\"date\":\"2026-01-02\",\"references\":[],\"summary\":\"\"}],\"count\":1},\"warnings\":[]}\n");

    let search = project.run(&["search", "First", "--json"]);
    search.assert_success();
    assert_eq!(search.stdout, "{\"ok\":true,\"data\":{\"query\":\"First\",\"results\":[{\"id\":\"task-1\",\"type\":\"task\",\"title\":\"First\",\"location\":\"board\",\"state\":\"in-progress\",\"snippet\":\"First\"}]},\"warnings\":[]}\n");
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
        "---\nid: note-1\ntype: note\ntitle: Legacy note\nstate: todo\npriority: normal # legacy alias\neffort: xlarge\nunknownField: preserve\n---\n\nLegacy custom body.\n",
    )
    .expect("seed legacy custom document");
    let event_before = "{\"event\":\"legacy.event\",\"id\":\"note-1\"}\n";
    fs::write(project.path(".tandem/events.jsonl"), event_before).expect("seed legacy event");
    let log_before = "---\nid: task-99\ntype: task\ntitle: Historical log\npriority: med\ncompletedAt: 2026-01-01T00:00:00Z\nunknownLogField: keep\n---\n\nHistorical body.\n";
    fs::write(project.path(".tandem/logs/task-99.md"), log_before).expect("seed historical log");
    let canonical_before = "---\nid: task-98\ntype: task\ntitle: Canonical log\npriority: medium\ncompletedAt: 2026-01-02T00:00:00Z\n---\n\nCanonical body.\n";
    fs::write(project.path(".tandem/logs/task-98.md"), canonical_before)
        .expect("seed canonical historical log");

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
    assert!(upgrade.stdout.contains("canonicalizing legacy"));
    let upgraded_config = project.read(".tandem/tandem.md");
    assert!(upgraded_config.contains("protocolVersion: \"0.2.0\""));
    assert!(upgraded_config.contains("idPrefix: note"));
    assert!(upgraded_config.contains("requireReview: true"));
    assert_eq!(project.read(".tandem/board/note-1.md"), "---\nid: note-1\ntype: note\ntitle: Legacy note\nstate: todo\npriority: medium # legacy alias\neffort: xlarge\nunknownField: preserve\n---\n\nLegacy custom body.\n");
    assert_eq!(project.read(".tandem/events.jsonl"), event_before);
    assert_eq!(project.read(".tandem/logs/task-99.md"), "---\nid: task-99\ntype: task\ntitle: Historical log\npriority: medium\ncompletedAt: 2026-01-01T00:00:00Z\nunknownLogField: keep\n---\n\nHistorical body.\n");
    assert_eq!(project.read(".tandem/logs/task-98.md"), canonical_before);

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

#[test]
fn rules_and_decisions_share_durable_command_behavior() {
    let project = TempProject::new("rules-decisions");
    project.init();

    let added_rule = project.run(&[
        "rules",
        "add",
        "--category",
        "always",
        "--rule",
        "Preserve bytes",
        "--source",
        " missing-decision ",
    ]);
    added_rule.assert_success();
    assert!(added_rule
        .stdout
        .contains("Warning: rule source not found:  missing-decision "));
    assert!(added_rule.stdout.contains("ID:       1"));
    assert!(project
        .read(".tandem/tandem.md")
        .contains("source: \" missing-decision \""));

    project
        .run(&[
            "rules",
            "edit",
            "--category",
            "always",
            "--id",
            "1",
            "--rule",
            "Preserve exact bytes",
            "--source",
            "",
        ])
        .assert_success();

    let decision = project.run(&[
        "decision",
        "add",
        "--title",
        "Shared application seam",
        "--body",
        "  ## Decision\nUse one operation.  ",
        "--reference",
        "missing-task",
    ]);
    decision.assert_success();
    assert!(decision
        .stdout
        .contains("Warning: reference not found: missing-task"));
    assert!(decision.stdout.contains("ID:     decision-1"));

    let config = project.read(".tandem/tandem.md");
    assert!(config.contains("Preserve exact bytes"));
    assert!(!config.contains("source: \"\""));
    let record = project.read(".tandem/board/decision-1.md");
    assert!(record.contains("status: \"proposed\""));
    assert!(record.contains("references: [\"missing-task\"]"));
    assert!(record.contains("  ## Decision\nUse one operation.  \n"));
    let events = project.actor_events();
    assert!(events.contains("\"event\":\"rules.updated\""));
    assert!(events.contains("\"event\":\"decision.created\""));
}
