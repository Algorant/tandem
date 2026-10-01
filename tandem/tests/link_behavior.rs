//! Typed link behavior at the CLI boundary.

use std::path::{Path, PathBuf};
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

fn fail(root: &Path, args: &[&str]) -> String {
    let output = bin().current_dir(root).args(args).output().unwrap();
    assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn json(root: &Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&run(root, args)).unwrap()
}

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tandem-cli-links-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    run(&root, &["init", "--title", "links"]);
    for title in ["Papercut", "Fixer", "Other"] {
        run(
            &root,
            &["add", "task", title, "--acceptance", "criterion", "--json"],
        );
    }
    root
}

fn ids(value: &serde_json::Value) -> Vec<String> {
    value["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn link_add_remove_show_and_list_filters() {
    let root = workspace("lifecycle");
    let added = json(
        &root,
        &["link", "add", "task-1", "relates-to", "task-3", "--json"],
    );
    assert_eq!(added["ok"], true);
    assert_eq!(added["data"]["id"], "task-1");
    assert_eq!(added["data"]["type"], "relates-to");
    assert_eq!(added["data"]["target"], "task-3");
    assert_eq!(added["data"]["changed"], true);
    assert_eq!(added["data"]["recordWritten"], true);

    let again = json(
        &root,
        &["link", "add", "task-1", "relates-to", "task-3", "--json"],
    );
    assert_eq!(again["data"]["changed"], false);
    assert!(
        run(&root, &["link", "add", "task-1", "relates-to", "task-3"]).starts_with("Unchanged:")
    );

    let shown = json(&root, &["show", "task-1", "--json"]);
    assert_eq!(shown["data"]["links"][0]["type"], "relates-to");
    assert_eq!(shown["data"]["links"][0]["target"], "task-3");
    assert_eq!(shown["data"]["links"][0]["title"], "Other");
    assert_eq!(shown["data"]["links"][0]["location"], "board");
    let target = json(&root, &["show", "task-3", "--json"]);
    assert_eq!(target["data"]["incomingLinks"][0]["type"], "relates-to");
    assert_eq!(target["data"]["incomingLinks"][0]["source"], "task-1");
    let text = run(&root, &["show", "task-1"]);
    assert!(
        text.contains("Links:\n  relates-to task-3 - Other"),
        "{text}"
    );

    run(&root, &["link", "add", "task-1", "duplicates", "task-2"]);
    assert_eq!(
        ids(&json(&root, &["list", "--link", "duplicates", "--json"])),
        vec!["task-1"]
    );
    assert_eq!(
        ids(&json(&root, &["list", "--link", "duplicated-by", "--json"])),
        vec!["task-2"]
    );
    assert_eq!(
        ids(&json(&root, &["list", "--linked-to", "task-1", "--json"])),
        vec!["task-2", "task-3"]
    );
    assert_eq!(
        ids(&json(
            &root,
            &[
                "list",
                "--link",
                "relates-to",
                "--linked-to",
                "task-1",
                "--json"
            ]
        )),
        vec!["task-3"]
    );

    let removed = json(
        &root,
        &["link", "remove", "task-1", "relates-to", "task-3", "--json"],
    );
    assert_eq!(removed["data"]["changed"], true);
    let shown = json(&root, &["show", "task-1", "--json"]);
    assert_eq!(shown["data"]["links"].as_array().unwrap().len(), 1);
}

#[test]
fn link_errors_name_the_problem_and_change_nothing() {
    let root = workspace("errors");
    for (args, expected) in [
        (
            vec!["link", "add", "task-1", "blocks", "task-2"],
            "invalid link type `blocks`; expected one of: relates-to, duplicates, fixed-by, fixes, supersedes",
        ),
        (
            vec!["link", "add", "task-1", "relates-to", "task-1"],
            "task-1 cannot link to itself",
        ),
        (
            vec!["link", "add", "task-1", "relates-to", "task-99"],
            "link target not found: task-99",
        ),
        (
            vec!["link", "remove", "task-1", "relates-to", "task-2"],
            "task-1 has no relates-to link to task-2",
        ),
        (
            vec!["list", "--link", "blocks"],
            "invalid link type `blocks`",
        ),
    ] {
        let stderr = fail(&root, &args);
        assert!(stderr.contains(expected), "{args:?}: {stderr}");
    }
    let shown = json(&root, &["show", "task-1", "--json"]);
    assert!(shown["data"]["links"].as_array().unwrap().is_empty());
}

#[test]
fn archived_targets_are_linkable_and_only_completed_ones_can_fix() {
    let root = workspace("archived");
    run(&root, &["complete", "task-2"]);
    run(&root, &["cancel", "task-3", "--note", "dropped"]);
    run(&root, &["link", "add", "task-1", "relates-to", "task-3"]);
    run(&root, &["link", "add", "task-1", "fixed-by", "task-2"]);
    let stderr = fail(&root, &["link", "add", "task-1", "fixed-by", "task-3"]);
    assert!(
        stderr.contains(
            "task-3 is archived with outcome canceled; only a completed record can fix task-1"
        ),
        "{stderr}"
    );
    let shown = json(&root, &["show", "task-1", "--json"]);
    let links = shown["data"]["links"].as_array().unwrap();
    assert_eq!(links[0]["location"], "logs");
    // Archived Logs cannot be edited.
    let stderr = fail(&root, &["link", "add", "task-2", "relates-to", "task-1"]);
    assert!(
        stderr.contains("only active tasks own links: task-2 is archived"),
        "{stderr}"
    );
}

#[test]
fn complete_fixed_by_resolves_the_task_as_completed_with_a_citation() {
    let root = workspace("fixed-by");
    let completed = json(
        &root,
        &["complete", "task-1", "--fixed-by", "task-2", "--json"],
    );
    assert_eq!(completed["ok"], true);
    assert!(completed["warnings"].as_array().unwrap().is_empty());

    let archived = json(&root, &["show", "task-1", "--json"]);
    assert_eq!(archived["data"]["location"], "logs");
    assert_eq!(archived["data"]["resolution"]["outcome"], "completed");
    assert_eq!(archived["data"]["resolution"]["note"], "Fixed by task-2");
    assert_eq!(archived["data"]["links"][0]["type"], "fixed-by");
    assert_eq!(archived["data"]["links"][0]["target"], "task-2");

    let fixer = json(&root, &["show", "task-2", "--json"]);
    assert_eq!(fixer["data"]["incomingLinks"][0]["type"], "fixes");
    assert_eq!(fixer["data"]["incomingLinks"][0]["source"], "task-1");
    assert_eq!(
        ids(&json(
            &root,
            &[
                "list",
                "--scope",
                "archived",
                "--link",
                "fixed-by",
                "--linked-to",
                "task-2",
                "--json"
            ]
        )),
        vec!["task-1"]
    );

    // Invalid targets are rejected before anything is archived.
    let stderr = fail(&root, &["complete", "task-3", "--fixed-by", "task-3"]);
    assert!(stderr.contains("task-3 cannot link to itself"), "{stderr}");
    let stderr = fail(&root, &["complete", "task-3", "--fixed-by", "task-99"]);
    assert!(
        stderr.contains("link target not found: task-99"),
        "{stderr}"
    );
    assert_eq!(
        json(&root, &["show", "task-3", "--json"])["data"]["location"],
        "board"
    );

    let explicit = json(
        &root,
        &[
            "complete",
            "task-3",
            "--fixed-by",
            "task-2",
            "--note",
            "covered by the rewrite",
            "--json",
        ],
    );
    assert_eq!(explicit["ok"], true);
    assert_eq!(
        json(&root, &["show", "task-3", "--json"])["data"]["resolution"]["note"],
        "covered by the rewrite"
    );
}

#[test]
fn missing_link_targets_warn_for_active_records() {
    let root = workspace("warn");
    run(&root, &["link", "add", "task-1", "relates-to", "task-2"]);
    let path = root.join(".tandem/tasks/task-1.md");
    let content = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        content.replace(
            "relates-to: [\"task-2\"]",
            "relates-to: [\"task-77\"]\n  blocks: [\"task-2\"]",
        ),
    )
    .unwrap();
    let warnings = json(&root, &["show", "task-1", "--json"])["warnings"].clone();
    let warnings = warnings
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| warning.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(
        warnings.contains(&"task-1 links missing target task-77 (relates-to).".to_string()),
        "{warnings:?}"
    );
    assert!(
        warnings.contains(&"task-1 has unknown link type blocks.".to_string()),
        "{warnings:?}"
    );
}
