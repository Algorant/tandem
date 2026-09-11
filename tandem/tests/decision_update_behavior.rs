//! Native regression coverage for type-aware `tandem update` on Decisions.
//!
//! These tests drive the real `tandem` binary over disposable workspaces so the
//! resolved-type dispatch, automatic timestamps, list replacement/clearing, and
//! no-op/no-write behavior are exercised at the process boundary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

fn failed(root: &Path, args: &[&str]) -> Output {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
    output
}

fn json(root: &Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&run(root, args)).unwrap()
}

fn changes(root: &Path, args: &[&str]) -> Vec<String> {
    json(root, args)["data"]["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

fn warnings(root: &Path, args: &[&str]) -> Vec<String> {
    json(root, args)["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

fn event_bytes(root: &Path) -> Vec<u8> {
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

fn decision_source(root: &Path, id: &str) -> String {
    std::fs::read_to_string(root.join(format!(".tandem/decisions/{id}.md"))).unwrap()
}

fn root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-cli-decision-update-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn init(root_dir: &Path) {
    run(root_dir, &["init", "--title", "decision update"]);
}

fn pin_decided_at(source: &str, value: &str) -> String {
    source
        .lines()
        .map(|line| {
            if line.starts_with("decidedAt:") {
                format!("decidedAt: \"{value}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn scalar_line(source: &str, prefix: &str) -> String {
    source
        .lines()
        .find(|line| line.starts_with(prefix))
        .map(|line| line[prefix.len()..].trim().trim_matches('"').to_string())
        .unwrap_or_else(|| panic!("missing {prefix} in:\n{source}"))
}

#[test]
fn decision_update_edits_common_metadata_and_status() {
    let root_dir = root("metadata");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(
        &root_dir,
        &[
            "add",
            "decision",
            "Original",
            "--tag",
            "adr",
            "--decider",
            "Original",
        ],
    );
    run(&root_dir, &["add", "decision", "Superseding target"]);

    let fields = changes(
        &root_dir,
        &[
            "update",
            "decision-1",
            "--title",
            "Revised",
            "--body",
            "## Decision\n\nEdited body.\n",
            "--status",
            "accepted",
            "--decider",
            "Algorant",
            "--decider",
            "Pi",
            "--supersedes",
            "decision-2",
            "--tag",
            "adr",
            "--tag",
            "cli",
            "--reference",
            "decision-2",
            "--reference",
            "https://example.com/artifacts/7",
            "--related-file",
            "docs/note.md",
            "--json",
        ],
    );
    assert_eq!(
        fields,
        vec![
            "title",
            "status",
            "decidedAt",
            "deciders",
            "supersedes",
            "references",
            "relatedFiles",
            "tags",
            "body",
        ]
    );

    let source = decision_source(&root_dir, "decision-1");
    assert!(source.contains("status: \"accepted\""));
    assert!(source.contains("decidedAt:"));
    assert!(source.contains("createdAt:"));
    assert!(source.contains("updatedAt:"));
    assert!(source.contains("deciders: [\"Algorant\", \"Pi\"]"));
    assert!(source.contains("supersedes: [\"decision-2\"]"));
    assert!(
        source.contains("references: [\"decision-2\", \"https://example.com/artifacts/7\"]"),
        "{source}"
    );
    assert!(source.contains("relatedFiles: [\"docs/note.md\"]"));
    assert!(source.contains("tags: [\"adr\", \"cli\"]"));
    assert!(source.contains("## Decision\n\nEdited body."));
    assert!(!source.contains("Original"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_status_transitions_retain_and_refresh_decided_at() {
    let root_dir = root("dates");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Dates"]);
    run(
        &root_dir,
        &["update", "decision-1", "--status", "accepted", "--json"],
    );
    let accepted = decision_source(&root_dir, "decision-1");
    assert!(accepted.contains("decidedAt:"));

    // Force a recognisable historical value so retention/refresh is observable.
    let pinned = pin_decided_at(&accepted, "2000-01-01T00:00:00Z");
    std::fs::write(root_dir.join(".tandem/decisions/decision-1.md"), &pinned).unwrap();

    // Repeating the unchanged status is a true no-op: no backfill, no rewrite.
    let before_events = event_bytes(&root_dir);
    let repeated = json(
        &root_dir,
        &["update", "decision-1", "--status", "accepted", "--json"],
    );
    assert_eq!(repeated["data"]["changes"], serde_json::json!([]));
    assert_eq!(decision_source(&root_dir, "decision-1"), pinned);
    assert_eq!(event_bytes(&root_dir), before_events);

    // Leaving the terminal status retains when the decision was made.
    run(
        &root_dir,
        &["update", "decision-1", "--status", "deprecated", "--json"],
    );
    assert!(
        decision_source(&root_dir, "decision-1").contains("decidedAt: \"2000-01-01T00:00:00Z\"")
    );

    // Re-entering a terminal status refreshes the automatic date.
    let refreshed = changes(
        &root_dir,
        &["update", "decision-1", "--status", "rejected", "--json"],
    );
    assert_eq!(refreshed, vec!["status", "decidedAt"]);
    let rejected = decision_source(&root_dir, "decision-1");
    assert!(rejected.contains("decidedAt:"));
    assert!(!rejected.contains("2000-01-01T00:00:00Z"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_noop_and_absent_clear_write_nothing() {
    let root_dir = root("noop");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Stable", "--tag", "adr"]);
    let before_bytes = std::fs::read(root_dir.join(".tandem/decisions/decision-1.md")).unwrap();
    let before_events = event_bytes(&root_dir);

    let output = run(
        &root_dir,
        &["update", "decision-1", "--title", "Stable", "--json"],
    );
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["data"]["changes"], serde_json::json!([]));
    assert_eq!(
        std::fs::read(root_dir.join(".tandem/decisions/decision-1.md")).unwrap(),
        before_bytes
    );
    assert_eq!(event_bytes(&root_dir), before_events);

    // Clearing an already-absent list is a no-op rather than a rewrite.
    run(
        &root_dir,
        &["update", "decision-1", "--clear", "references", "--json"],
    );
    assert_eq!(
        std::fs::read(root_dir.join(".tandem/decisions/decision-1.md")).unwrap(),
        before_bytes
    );
    assert_eq!(event_bytes(&root_dir), before_events);

    let human = run(&root_dir, &["update", "decision-1", "--title", "Stable"]);
    assert!(human.contains("No changes for decision-1"), "{human}");
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_invalid_requests_fail_without_writes() {
    let root_dir = root("rejects");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Guarded", "--tag", "adr"]);
    run(
        &root_dir,
        &["add", "task", "Task target", "--acceptance", "ok"],
    );
    let before_bytes = std::fs::read(root_dir.join(".tandem/decisions/decision-1.md")).unwrap();
    let before_events = event_bytes(&root_dir);

    for (exit_code, args) in [
        (1, vec!["update", "decision-1", "--status", "bogus"]),
        (1, vec!["update", "decision-1", "--status", " accepted "]),
        (1, vec!["update", "decision-1", "--status", "\taccepted"]),
        (1, vec!["update", "decision-1", "--status", "accepted\n"]),
        (1, vec!["update", "decision-1", "--status", "rejected "]),
        (2, vec!["update", "decision-1", "--clear", "title"]),
        (2, vec!["update", "decision-1", "--clear", "status"]),
        (2, vec!["update", "decision-1", "--clear", "decidedAt"]),
        (2, vec!["update", "decision-1", "--clear", "decider"]),
        (2, vec!["update", "decision-1", "--clear", "nonsense"]),
        (
            2,
            vec!["update", "decision-1", "--tag", "cli", "--clear", "tags"],
        ),
        (
            2,
            vec!["update", "decision-1", "--body", "x", "--clear", "body"],
        ),
        (2, vec!["update", "decision-1", "--body", ""]),
        (2, vec!["update", "decision-1", "--priority", "high"]),
        (2, vec!["update", "task-1", "--status", "accepted"]),
        (2, vec!["update", "task-1", "--decider", "Algorant"]),
    ] {
        let output = failed(&root_dir, &args);
        assert_eq!(output.status.code(), Some(exit_code), "{args:?}");
    }

    assert_eq!(
        std::fs::read(root_dir.join(".tandem/decisions/decision-1.md")).unwrap(),
        before_bytes
    );
    assert_eq!(event_bytes(&root_dir), before_events);

    // Existing Task update behavior is unchanged.
    run(
        &root_dir,
        &["update", "task-1", "--title", "Renamed", "--tag", "cli"],
    );
    let task = std::fs::read_to_string(root_dir.join(".tandem/tasks/task-1.md")).unwrap();
    assert!(task.contains("title: \"Renamed\""));
    assert!(task.contains("tags: [\"cli\"]"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_clear_removes_lists_and_body() {
    let root_dir = root("clear");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(
        &root_dir,
        &[
            "add",
            "decision",
            "Clearable",
            "--body",
            "Body to remove.",
            "--decider",
            "Algorant",
            "--reference",
            "missing-doc",
            "--tag",
            "adr",
        ],
    );
    run(&root_dir, &["add", "decision", "Supersede target"]);
    run(
        &root_dir,
        &[
            "update",
            "decision-1",
            "--supersedes",
            "decision-2",
            "--related-file",
            "src/lib.rs",
        ],
    );

    let fields = changes(
        &root_dir,
        &[
            "update",
            "decision-1",
            "--clear",
            "tags",
            "--clear",
            "references",
            "--clear",
            "relatedFiles",
            "--clear",
            "deciders",
            "--clear",
            "supersedes",
            "--clear",
            "body",
            "--json",
        ],
    );
    assert_eq!(
        fields,
        vec![
            "tags",
            "references",
            "relatedFiles",
            "deciders",
            "supersedes",
            "body",
        ]
    );
    let source = decision_source(&root_dir, "decision-1");
    assert!(!source.contains("tags:"));
    assert!(!source.contains("references:"));
    assert!(!source.contains("relatedFiles:"));
    assert!(!source.contains("deciders:"));
    assert!(!source.contains("supersedes:"));
    assert!(!source.contains("Body to remove."));

    // The established clear aliases are accepted too.
    run(
        &root_dir,
        &["add", "decision", "Alias clear", "--tag", "adr"],
    );
    run(
        &root_dir,
        &["update", "decision-3", "--related-file", "src/lib.rs"],
    );
    run(
        &root_dir,
        &[
            "update",
            "decision-3",
            "--clear",
            "tag",
            "--clear",
            "related-file",
            "--json",
        ],
    );
    let alias = decision_source(&root_dir, "decision-3");
    assert!(!alias.contains("tags:"));
    assert!(!alias.contains("relatedFiles:"));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_update_preserves_unknown_metadata_and_unrelated_body() {
    let root_dir = root("preserve");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Preserve"]);
    let path = root_dir.join(".tandem/decisions/decision-1.md");
    let original = std::fs::read_to_string(&path).unwrap();
    let annotated = original.replace("type: decision\n", "type: decision\ncustom: keep-me\n");
    let annotated = format!("{annotated}\nBody bytes stay untouched.\n");
    std::fs::write(&path, &annotated).unwrap();

    run(&root_dir, &["update", "decision-1", "--title", "Renamed"]);
    let updated = std::fs::read_to_string(&path).unwrap();
    assert!(updated.contains("custom: keep-me"));
    assert!(updated.contains("Body bytes stay untouched.\n"));
    assert!(updated.contains("title: \"Renamed\""));
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_terminal_transition_shares_one_timestamp() {
    let root_dir = root("timestamp");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Timestamped"]);

    for status in ["accepted", "rejected"] {
        run(
            &root_dir,
            &["update", "decision-1", "--status", status, "--json"],
        );
        let source = decision_source(&root_dir, "decision-1");
        let decided = scalar_line(&source, "decidedAt:");
        let updated = scalar_line(&source, "updatedAt:");
        assert!(!decided.is_empty(), "status {status} wrote no decidedAt");
        assert_eq!(
            decided, updated,
            "a terminal transition must share one timestamp"
        );
    }
    std::fs::remove_dir_all(root_dir).unwrap();
}

#[test]
fn decision_reference_and_supersession_warnings_follow_task_29_semantics() {
    let root_dir = root("warnings");
    std::fs::create_dir_all(&root_dir).unwrap();
    init(&root_dir);
    run(&root_dir, &["add", "decision", "Warned"]);
    run(
        &root_dir,
        &["add", "task", "Not a decision", "--acceptance", "ok"],
    );

    let reported = warnings(
        &root_dir,
        &[
            "update",
            "decision-1",
            "--reference",
            "https://example.com/artifacts/9",
            "--reference",
            "missing-doc",
            "--supersedes",
            "task-1",
            "--related-file",
            "docs/note.md",
            "--json",
        ],
    );
    assert!(
        reported
            .iter()
            .any(|warning| warning == "reference not found: missing-doc"),
        "{reported:?}"
    );
    assert!(
        reported
            .iter()
            .any(|warning| warning == "supersedes target task-1 is type task, not decision"),
        "{reported:?}"
    );
    assert!(
        !reported
            .iter()
            .any(|warning| warning.contains("https://") || warning.contains("docs/note.md")),
        "URLs and relatedFiles are not document references: {reported:?}"
    );

    // Human mode must surface the same warnings on stderr, not silently drop them.
    let output = bin()
        .current_dir(&root_dir)
        .args(["update", "decision-1", "--reference", "missing-doc"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Warning: reference not found: missing-doc"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(root_dir).unwrap();
}
