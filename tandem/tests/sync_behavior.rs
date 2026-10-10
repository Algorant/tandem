//! Two-machine synchronization through the repository's `tandem` branch.
//!
//! Every scenario uses a local bare remote and independent clones standing in
//! for separate machines, and drives the real CLI. Besides board content,
//! scenarios assert that sync never changes the source branch, index, or
//! working tree.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tandem-sync-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} in {}: {}",
        cwd.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn tandem(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
        .args(args)
        .current_dir(cwd)
        .env("TANDEM_SYNC_TIMEOUT", "5")
        .output()
        .unwrap()
}

/// Runs a JSON command that must succeed and returns its envelope.
fn ok(cwd: &Path, args: &[&str]) -> Value {
    let mut argv = vec!["--json"];
    argv.extend(args);
    let output = tandem(cwd, &argv);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "tandem {args:?} in {} failed: {stdout} {}",
        cwd.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(stdout.trim()).unwrap()
}

fn fails(cwd: &Path, args: &[&str]) -> String {
    let output = tandem(cwd, args);
    assert!(
        !output.status.success(),
        "tandem {args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn configure(repo: &Path) {
    git(repo, &["config", "user.email", "tests@example.invalid"]);
    git(repo, &["config", "user.name", "Tandem Tests"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
    git(repo, &["config", "pull.rebase", "false"]);
}

struct World {
    root: PathBuf,
    remote: PathBuf,
}

impl World {
    /// A bare remote whose `main` holds one source commit.
    fn new(label: &str) -> Self {
        let root = scratch(label);
        let remote = root.join("remote.git");
        git(
            &root,
            &[
                "init",
                "--quiet",
                "--bare",
                "--initial-branch=main",
                remote.to_str().unwrap(),
            ],
        );
        let seed = root.join("seed");
        git(
            &root,
            &[
                "init",
                "--quiet",
                "--initial-branch=main",
                seed.to_str().unwrap(),
            ],
        );
        configure(&seed);
        fs::write(seed.join("README.md"), "source\n").unwrap();
        git(&seed, &["add", "README.md"]);
        git(&seed, &["commit", "--quiet", "-m", "source baseline"]);
        git(
            &seed,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git(&seed, &["push", "--quiet", "-u", "origin", "main"]);
        Self { root, remote }
    }

    fn clone(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        git(
            &self.root,
            &[
                "clone",
                "--quiet",
                self.remote.to_str().unwrap(),
                path.to_str().unwrap(),
            ],
        );
        configure(&path);
        path
    }

    /// Machine A creates and publishes the board; machine B downloads it.
    fn two_machines(label: &str) -> (Self, PathBuf, PathBuf) {
        let world = Self::new(label);
        let a = world.clone("a");
        ok(&a, &["init", "--title", "Shared board"]);
        let b = world.clone("b");
        ok(&b, &["list"]);
        (world, a, b)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Source-side state that sync must never change.
fn source_state(repo: &Path) -> (String, String, String) {
    (
        git(repo, &["rev-parse", "HEAD"]),
        git(repo, &["status", "--porcelain", "--untracked-files=all"]),
        git(repo, &["diff", "--cached", "--name-only"]),
    )
}

fn add_task(repo: &Path, title: &str) -> Value {
    ok(repo, &["add", "task", title, "--acceptance", "done"])
}

fn ids(repo: &Path) -> Vec<String> {
    let mut ids: Vec<String> = ok(repo, &["list", "--scope", "all"])["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["id"].as_str().unwrap().to_string())
        .collect();
    ids.sort();
    ids
}

fn title(repo: &Path, id: &str) -> String {
    ok(repo, &["show", id])["data"]["title"]
        .as_str()
        .unwrap()
        .to_string()
}

fn go_offline(repo: &Path) {
    git(
        repo,
        &["remote", "set-url", "origin", "/nonexistent/remote.git"],
    );
}

fn go_online(world: &World, repo: &Path) {
    git(
        repo,
        &[
            "remote",
            "set-url",
            "origin",
            world.remote.to_str().unwrap(),
        ],
    );
}

#[test]
fn fresh_clone_downloads_the_board_and_source_stays_clean() {
    let (world, a, b) = World::two_machines("hydrate");
    add_task(&a, "Created on A");
    let c = world.clone("c");
    let before = source_state(&c);
    assert_eq!(ids(&c), vec!["task-1"]);
    assert_eq!(title(&c, "task-1"), "Created on A");
    assert_eq!(source_state(&c), before);
    assert!(git(&b, &["status", "--porcelain"]).is_empty());
    assert!(git(&a, &["status", "--porcelain"]).is_empty());
}

#[test]
fn independent_creation_on_two_machines_never_collides() {
    let (_world, a, b) = World::two_machines("create");
    let before_b = source_state(&b);
    let first = add_task(&a, "Work from A");
    assert_eq!(first["data"]["id"], "task-1");
    assert_eq!(first["data"]["sync"]["status"], "synced");
    // B still has the old base; its push is rejected, then merged and numbered.
    let second = add_task(&b, "Different work from B");
    assert_eq!(second["data"]["id"], "task-2");
    assert_eq!(second["data"]["sync"]["status"], "synced");
    ok(&a, &["sync"]);
    assert_eq!(ids(&a), vec!["task-1", "task-2"]);
    assert_eq!(ids(&b), vec!["task-1", "task-2"]);
    assert_eq!(title(&a, "task-2"), "Different work from B");
    assert_eq!(title(&b, "task-1"), "Work from A");
    assert_eq!(source_state(&b), before_b);
}

#[test]
fn offline_work_is_saved_with_a_temporary_id_and_numbered_on_reconnect() {
    let (world, a, b) = World::two_machines("offline");
    go_offline(&b);
    let created = add_task(&b, "Written on a plane");
    let handle = created["data"]["id"].as_str().unwrap().to_string();
    assert!(handle.starts_with("task-new-"), "{handle}");
    assert_eq!(created["data"]["sync"]["status"], "pending");
    // A child can reference the unsynced parent by its temporary ID.
    let child = ok(
        &b,
        &[
            "add",
            "task",
            "Child",
            "--acceptance",
            "done",
            "--parent",
            &handle,
        ],
    );
    let child_handle = child["data"]["id"].as_str().unwrap().to_string();
    add_task(&a, "Meanwhile on A");
    // The source branch still fast-forwards while board changes are pending.
    fs::write(a.join("README.md"), "source v2\n").unwrap();
    git(&a, &["commit", "--quiet", "-am", "source change"]);
    git(&a, &["push", "--quiet"]);
    go_online(&world, &b);
    git(&b, &["pull", "--ff-only", "--quiet"]);
    let synced = ok(&b, &["sync"]);
    assert_eq!(synced["data"]["sync"]["renamed"][&handle], "task-2");
    assert_eq!(synced["data"]["sync"]["renamed"][&child_handle], "task-2-1");
    assert_eq!(ids(&b), vec!["task-1", "task-2", "task-2-1"]);
    assert_eq!(ok(&b, &["show", "task-2-1"])["data"]["parentId"], "task-2");
    // The old temporary ID still finds the record.
    assert_eq!(ok(&b, &["show", &handle])["data"]["id"], "task-2");
    ok(&a, &["sync"]);
    assert_eq!(ids(&a), vec!["task-1", "task-2", "task-2-1"]);
    assert!(git(&b, &["status", "--porcelain"]).is_empty());
}

#[test]
fn rules_and_subtasks_from_two_machines_get_distinct_numbers() {
    let (_world, a, b) = World::two_machines("rules");
    add_task(&a, "Parent");
    ok(&b, &["sync"]);
    ok(&a, &["rules", "add", "always", "Rule from A"]);
    ok(
        &a,
        &[
            "add",
            "task",
            "Sub A",
            "--acceptance",
            "x",
            "--parent",
            "task-1",
        ],
    );
    let rule_b = ok(&b, &["rules", "add", "always", "Rule from B"]);
    let sub_b = ok(
        &b,
        &[
            "add",
            "task",
            "Sub B",
            "--acceptance",
            "x",
            "--parent",
            "task-1",
        ],
    );
    assert_eq!(rule_b["data"]["id"], "always-2");
    assert_eq!(sub_b["data"]["id"], "task-1-2");
    ok(&a, &["sync"]);
    let rules: Vec<String> = ok(&a, &["rules", "list"])["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(rules, vec!["always-1", "always-2"]);
    assert_eq!(ids(&a), vec!["task-1", "task-1-1", "task-1-2"]);
}

#[test]
fn compatible_edits_to_one_record_combine() {
    let (_world, a, b) = World::two_machines("combine");
    add_task(&a, "Shared");
    ok(&b, &["sync"]);
    ok(
        &a,
        &[
            "update",
            "task-1",
            "--title",
            "Renamed on A",
            "--tag",
            "alpha",
        ],
    );
    ok(
        &b,
        &["update", "task-1", "--priority", "high", "--tag", "beta"],
    );
    ok(&a, &["sync"]);
    for repo in [&a, &b] {
        let shown = ok(repo, &["show", "task-1"]);
        assert_eq!(shown["data"]["title"], "Renamed on A");
        assert_eq!(shown["data"]["priority"], "high");
        assert_eq!(shown["data"]["tags"], serde_json::json!(["beta", "alpha"]));
    }
}

#[test]
fn contradictory_edits_stop_only_that_record_until_resolved() {
    let (_world, a, b) = World::two_machines("conflict");
    add_task(&a, "Shared");
    add_task(&a, "Other");
    ok(&b, &["sync"]);
    ok(&a, &["update", "task-1", "--title", "Title from A"]);
    let result = ok(&b, &["update", "task-1", "--title", "Title from B"]);
    assert_eq!(result["data"]["sync"]["status"], "pending");
    assert_eq!(result["data"]["sync"]["conflicts"][0]["id"], "task-1");
    // The board shows the shared version; unrelated work still syncs.
    assert_eq!(title(&b, "task-1"), "Title from A");
    ok(&b, &["update", "task-2", "--title", "Other from B"]);
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-2"), "Other from B");
    let status = ok(&b, &["sync", "status"]);
    assert_eq!(status["data"]["conflicts"][0]["id"], "task-1");
    let refused = fails(&b, &["update", "task-1", "--priority", "low"]);
    assert!(refused.contains("unresolved sync conflict"), "{refused}");
    ok(&b, &["sync", "resolve", "task-1", "--keep", "local"]);
    assert!(ok(&b, &["sync", "status"])["data"]["conflicts"]
        .as_array()
        .unwrap()
        .is_empty());
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-1"), "Title from B");
    assert_eq!(title(&b, "task-1"), "Title from B");
}

#[test]
fn completing_on_one_machine_while_editing_on_another_asks() {
    let (_world, a, b) = World::two_machines("archive");
    add_task(&a, "Finish me");
    ok(&b, &["sync"]);
    ok(&a, &["cancel", "task-1", "--note", "not needed"]);
    let edited = ok(&b, &["update", "task-1", "--title", "Still editing"]);
    assert_eq!(edited["data"]["sync"]["conflicts"][0]["id"], "task-1");
    assert!(edited["data"]["sync"]["conflicts"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("archived on another machine"));
    ok(&b, &["sync", "resolve", "task-1", "--keep", "remote"]);
    let shown = ok(&b, &["show", "task-1"]);
    assert_eq!(shown["data"]["location"], "logs");
    assert_eq!(shown["data"]["title"], "Finish me");
}

#[test]
fn unsynced_changes_survive_git_clean_and_old_checkouts() {
    let (_world, a, b) = World::two_machines("safety");
    add_task(&a, "Published");
    ok(&b, &["sync"]);
    go_offline(&b);
    let pending = add_task(&b, "Not yet synced");
    let handle = pending["data"]["id"].as_str().unwrap().to_string();

    git(&b, &["clean", "-fdxq"]);
    assert!(!b.join(".tandem").exists());
    assert!(ids(&b).contains(&handle));

    // An old commit that tracked a 0.3.0 board overwrites the folder.
    let head = git(&b, &["rev-parse", "HEAD"]);
    git(&b, &["checkout", "--quiet", "-b", "legacy"]);
    fs::write(
        b.join(".tandem/tandem.md"),
        "---\nprotocolVersion: 0.3.0\ntype: workspace\ntitle: Old\nstates: [todo, in-progress, validation]\n---\n",
    )
    .unwrap();
    git(&b, &["add", "-f", ".tandem/tandem.md"]);
    git(&b, &["commit", "--quiet", "-m", "legacy board"]);
    let legacy = git(&b, &["rev-parse", "HEAD"]);
    git(&b, &["checkout", "--quiet", "--detach", &head]);
    git(&b, &["checkout", "--quiet", &legacy]);
    let listed = tandem(&b, &["list"]);
    assert!(listed.status.success());
    assert!(String::from_utf8_lossy(&listed.stderr).contains("predates the `tandem` branch"));
    let refused = fails(&b, &["add", "task", "Nope", "--acceptance", "x"]);
    assert!(refused.contains("cannot change the board"), "{refused}");
    git(&b, &["checkout", "--quiet", &head]);
    assert!(ids(&b).contains(&handle), "board restored after returning");
}

#[test]
fn direct_markdown_edits_sync_and_broken_ones_are_held() {
    let (_world, a, b) = World::two_machines("edits");
    add_task(&a, "One");
    add_task(&a, "Two");
    ok(&b, &["sync"]);
    let path = b.join(".tandem/tasks/task-1.md");
    let content = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        content.replace("title: \"One\"", "title: \"One, edited by hand\""),
    )
    .unwrap();
    let broken = b.join(".tandem/tasks/task-2.md");
    fs::write(&broken, "---\nid: task-2\ntitle: [unclosed\n---\n").unwrap();
    let synced = ok(&b, &["sync"]);
    let held = synced["data"]["sync"]["held"].as_array().unwrap();
    assert_eq!(held.len(), 1);
    assert!(held[0]["path"]
        .as_str()
        .unwrap()
        .ends_with("tasks/task-2.md"));
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-1"), "One, edited by hand");
    assert_eq!(title(&a, "task-2"), "Two");
    // The broken file is left for its owner to fix.
    assert!(fs::read_to_string(&broken).unwrap().contains("[unclosed"));
}

#[test]
fn linked_worktrees_share_the_main_board() {
    let (world, a, _b) = World::two_machines("worktree");
    let worktree = world.root.join("a-feature");
    git(
        &a,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature",
            worktree.to_str().unwrap(),
        ],
    );
    assert!(!worktree.join(".tandem").exists());
    let created = add_task(&worktree, "From the worktree");
    assert_eq!(created["data"]["id"], "task-1");
    assert_eq!(ids(&a), vec!["task-1"]);
    assert!(!worktree.join(".tandem").exists());
    add_task(&a, "From the main checkout");
    let worktree_actor =
        fs::read_to_string(a.join(".git/worktrees/a-feature/tandem-actor-id")).unwrap();
    let main_actor = fs::read_to_string(a.join(".git/tandem-actor-id")).unwrap();
    assert_ne!(
        worktree_actor, main_actor,
        "each checkout keeps its own identity"
    );
    for actor in [worktree_actor, main_actor] {
        assert!(a
            .join(format!(".tandem/events/{}.jsonl", actor.trim()))
            .is_file());
    }
    assert_eq!(ids(&worktree), vec!["task-1", "task-2"]);
    assert!(git(&worktree, &["status", "--porcelain"]).is_empty());
}

#[test]
fn a_held_broken_edit_does_not_stop_other_work() {
    let (world, a, b) = World::two_machines("held-broken");
    add_task(&a, "One");
    add_task(&a, "Two");
    ok(&b, &["sync"]);
    let broken = b.join(".tandem/tasks/task-2.md");
    let bytes = "---\nid: task-2\ntitle: [unclosed\n---\n";
    fs::write(&broken, bytes).unwrap();
    let listed = ok(&b, &["list"]);
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert!(listed["warnings"][0]
        .as_str()
        .unwrap()
        .contains("held from sync"));
    let worktree = world.root.join("b-feature");
    git(
        &b,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature",
            worktree.to_str().unwrap(),
        ],
    );
    let created = add_task(&worktree, "From the worktree");
    assert_eq!(created["data"]["id"], "task-3");
    assert!(created["data"]["sync"]["held"][0]["path"]
        .as_str()
        .unwrap()
        .ends_with("tasks/task-2.md"));
    ok(&b, &["sync"]);
    ok(&a, &["sync"]);
    assert_eq!(ids(&a), vec!["task-1", "task-2", "task-3"]);
    assert_eq!(title(&a, "task-2"), "Two");
    assert_eq!(fs::read_to_string(&broken).unwrap(), bytes);
}

/// Machine B holds a Task with one Subtask, both published. The Subtask file
/// is then rewritten by hand without its `parentId`: it parses, but a
/// Subtask-shaped ID without a parent would make the shared board invalid.
fn held_subtask(label: &str) -> (World, PathBuf, PathBuf, PathBuf, String) {
    let (world, a, b) = World::two_machines(label);
    add_task(&a, "Parent");
    ok(
        &a,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "done",
        ],
    );
    ok(&b, &["sync"]);
    let child = b.join(".tandem/tasks/task-1-1.md");
    let shared = fs::read_to_string(&child).unwrap();
    assert!(shared.contains("parentId:"));
    let invalid: String = shared
        .lines()
        .filter(|line| !line.starts_with("parentId:"))
        .map(|line| format!("{line}\n"))
        .collect();
    fs::write(&child, invalid).unwrap();
    (world, a, b, child, shared)
}

fn held_entries(envelope: &Value, pointer: &str) -> Vec<(String, String, Option<String>)> {
    envelope
        .pointer(pointer)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|held| {
            (
                held["path"].as_str().unwrap().to_string(),
                held["reason"].as_str().unwrap().to_string(),
                held["id"].as_str().map(str::to_string),
            )
        })
        .collect()
}

#[test]
fn a_validation_invalid_edit_is_held_and_other_work_continues() {
    let (world, a, b, child, shared) = held_subtask("held-invalid");
    let invalid = fs::read_to_string(&child).unwrap();

    // Reads still list the records and name the held file.
    let listed = ok(&b, &["list"]);
    let titles: Vec<&str> = listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["id"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["task-1", "task-1-1"]);
    let warnings = listed["warnings"].to_string();
    assert!(warnings.contains(".tandem/tasks/task-1-1.md"), "{warnings}");
    assert!(warnings.contains("held from sync"), "{warnings}");
    // A read that validates the whole board works too.
    let shown = ok(&b, &["show", "task-1"]);
    assert_eq!(shown["data"]["children"][0]["parentId"], "task-1");
    assert!(shown["warnings"].to_string().contains("task-1-1.md"));

    // Unrelated mutations succeed and report the held file.
    let created = add_task(&b, "Other");
    assert_eq!(created["data"]["id"], "task-2");
    assert!(created["data"]["sync"]["held"][0]["path"]
        .as_str()
        .unwrap()
        .ends_with("tasks/task-1-1.md"));
    ok(&b, &["update", "task-1", "--title", "Parent, renamed"]);

    // The held record itself is refused, and nothing it holds is rewritten.
    for args in [
        vec!["update", "task-1-1", "--title", "Nope"],
        vec!["complete", "task-1-1"],
        vec!["accord", "claim", "task-1-1", "--assignee", "me"],
    ] {
        let refused = fails(&b, &args);
        assert!(
            refused.contains("tandem sync resolve task-1-1 --keep remote"),
            "{args:?}: {refused}"
        );
    }
    assert_eq!(fs::read_to_string(&child).unwrap(), invalid);

    // The invalid file never reaches the shared branch.
    ok(&b, &["sync"]);
    assert_eq!(
        git(&world.remote, &["show", "tandem:tasks/task-1-1.md"]),
        shared.trim()
    );
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-1"), "Parent, renamed");
    assert_eq!(title(&a, "task-2"), "Other");
    assert_eq!(
        fs::read_to_string(a.join(".tandem/tasks/task-1-1.md")).unwrap(),
        shared
    );
}

#[test]
fn sync_status_lists_validation_holds_like_sync_does() {
    let (_world, _a, b, child, _shared) = held_subtask("held-status");
    let synced = ok(&b, &["sync"]);
    let from_sync = held_entries(&synced, "/data/sync/held");
    assert_eq!(from_sync.len(), 1);
    assert!(from_sync[0].0.ends_with("tasks/task-1-1.md"));
    assert!(from_sync[0]
        .1
        .starts_with("would make the shared board invalid:"));
    assert_eq!(from_sync[0].2.as_deref(), Some("task-1-1"));

    let status = ok(&b, &["sync", "status"]);
    assert_eq!(held_entries(&status, "/data/held"), from_sync);

    let text = String::from_utf8_lossy(&tandem(&b, &["sync", "status"]).stdout).to_string();
    assert!(
        text.contains("Held edit: .tandem/tasks/task-1-1.md: would make the shared board invalid:"),
        "{text}"
    );
    assert!(
        text.contains("resolve: tandem sync resolve task-1-1 --keep remote"),
        "{text}"
    );
    assert!(child.exists());
}

#[test]
fn sync_resolve_keep_remote_restores_a_validation_held_record() {
    let (_world, a, b, child, shared) = held_subtask("held-resolve");
    let invalid = fs::read_to_string(&child).unwrap();

    // Only the shared version can repair it; the others fail clearly.
    for keep in ["local", "edited"] {
        let refused = fails(&b, &["sync", "resolve", "task-1-1", "--keep", keep]);
        assert!(refused.contains("--keep remote"), "{refused}");
    }
    let unknown = fails(&b, &["sync", "resolve", "task-1", "--keep", "remote"]);
    assert!(
        unknown.contains("no open sync conflict for task-1"),
        "{unknown}"
    );
    assert_eq!(fs::read_to_string(&child).unwrap(), invalid);

    let resolved = ok(&b, &["sync", "resolve", "task-1-1", "--keep", "remote"]);
    assert_eq!(resolved["data"]["sync"]["status"], "synced");
    assert_eq!(fs::read_to_string(&child).unwrap(), shared);
    assert!(ok(&b, &["list"])["warnings"].as_array().unwrap().is_empty());
    assert!(ok(&b, &["sync", "status"])["data"]["held"]
        .as_array()
        .unwrap()
        .is_empty());
    ok(&b, &["update", "task-1-1", "--title", "Child, edited"]);
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-1-1"), "Child, edited");
}

#[test]
fn a_held_parent_is_read_as_its_shared_version_so_its_children_stay_valid() {
    let (_world, a, b) = World::two_machines("held-parent");
    add_task(&a, "Parent");
    ok(
        &a,
        &[
            "add",
            "task",
            "Child",
            "--parent",
            "task-1",
            "--acceptance",
            "done",
        ],
    );
    add_task(&a, "Elsewhere");
    ok(&b, &["sync"]);
    // An Epic cannot have a parent, so this edit is invalid for the Parent.
    let parent = b.join(".tandem/tasks/task-1.md");
    let shared = fs::read_to_string(&parent).unwrap();
    fs::write(
        &parent,
        shared.replace(
            "type: task\n",
            "type: task\nkind: epic\nparentId: \"task-2\"\n",
        ),
    )
    .unwrap();
    let invalid = fs::read_to_string(&parent).unwrap();

    let listed = ok(&b, &["list"]);
    assert_eq!(listed["data"].as_array().unwrap().len(), 3);
    assert!(listed["warnings"].to_string().contains("tasks/task-1.md"));
    let shown = ok(&b, &["show", "task-1-1"]);
    assert_eq!(shown["data"]["parentId"], "task-1");
    assert_eq!(shown["data"]["role"], "subtask");

    ok(&b, &["update", "task-2", "--title", "Elsewhere, renamed"]);
    let sibling = ok(
        &b,
        &[
            "add",
            "task",
            "Second child",
            "--parent",
            "task-1",
            "--acceptance",
            "done",
        ],
    );
    assert!(sibling["data"]["id"]
        .as_str()
        .unwrap()
        .starts_with("task-1-"));
    let refused = fails(&b, &["update", "task-1", "--title", "Nope"]);
    assert!(
        refused.contains("tandem sync resolve task-1 --keep remote"),
        "{refused}"
    );
    assert_eq!(fs::read_to_string(&parent).unwrap(), invalid);

    ok(&b, &["sync", "resolve", "task-1", "--keep", "remote"]);
    assert_eq!(fs::read_to_string(&parent).unwrap(), shared);
    ok(&a, &["sync"]);
    assert_eq!(title(&a, "task-2"), "Elsewhere, renamed");
}

#[test]
fn a_held_new_record_is_skipped_and_resolve_removes_it() {
    let (world, a, b, child, shared) = held_subtask("held-new");
    // Repair the first hold, then add a record that was never shared and
    // that would make the board invalid: a Subtask cannot have children.
    ok(&b, &["sync", "resolve", "task-1-1", "--keep", "remote"]);
    assert_eq!(fs::read_to_string(&child).unwrap(), shared);
    let fresh = b.join(".tandem/tasks/task-9.md");
    fs::write(
        &fresh,
        "---\nid: task-9\nuid: 6b0b9f0e-1d2c-4f55-9a53-0d6f4a0a7e11\ntype: task\ntitle: \"Grandchild\"\nstate: todo\nparentId: \"task-1-1\"\naccord:\n  status: \"ready\"\n  acceptance: [\"done\"]\ncreatedAt: \"2026-01-01T00:00:00Z\"\nupdatedAt: \"2026-01-01T00:00:00Z\"\n---\n",
    )
    .unwrap();

    let listed = ok(&b, &["list"]);
    let listed_ids: Vec<&str> = listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["id"].as_str().unwrap())
        .collect();
    assert_eq!(listed_ids, vec!["task-1", "task-1-1"]);
    assert!(listed["warnings"].to_string().contains("tasks/task-9.md"));
    add_task(&b, "Other");
    let refused = fails(&b, &["update", "task-9", "--title", "Nope"]);
    assert!(
        refused.contains("tandem sync resolve task-9 --keep remote"),
        "{refused}"
    );

    ok(&b, &["sync", "resolve", "task-9", "--keep", "remote"]);
    assert!(!fresh.exists());
    ok(&b, &["sync"]);
    ok(&a, &["sync"]);
    assert_eq!(ids(&a), vec!["task-1", "task-1-1", "task-2"]);
    assert!(
        git(&world.remote, &["ls-tree", "-r", "--name-only", "tandem"])
            .lines()
            .all(|path| path != "tasks/task-9.md")
    );
}

fn legacy_board(repo: &Path) {
    let board = repo.join(".tandem");
    fs::create_dir_all(board.join("tasks")).unwrap();
    fs::create_dir_all(board.join("rules")).unwrap();
    fs::write(
        board.join("tandem.md"),
        "---\nprotocolVersion: 0.3.0\ntype: workspace\ntitle: \"Legacy\"\nstates:\n  - id: todo\n    title: To Do\n  - id: in-progress\n    title: In Progress\n  - id: validation\n    title: Validation\n---\n\n# Legacy\n",
    )
    .unwrap();
    fs::write(
        board.join("tasks/task-1.md"),
        legacy_task("task-1", "Legacy one"),
    )
    .unwrap();
    fs::write(
        board.join("rules/always-1.md"),
        "---\nid: always-1\ncategory: always\n---\nKeep it.\n",
    )
    .unwrap();
}

fn legacy_task(id: &str, title: &str) -> String {
    format!(
        "---\nid: {id}\ntype: task\ntitle: \"{title}\"\nstate: todo\naccord:\n  status: \"ready\"\n  acceptance: [\"ok\"]\n  updatedAt: \"2026-01-01T00:00:00Z\"\ncreatedAt: \"2026-01-01T00:00:00Z\"\nupdatedAt: \"2026-01-01T00:00:00Z\"\n---\n"
    )
}

#[test]
fn migration_moves_a_legacy_board_and_adopts_unpushed_work_elsewhere() {
    let world = World::new("migrate");
    let a = world.clone("a");
    legacy_board(&a);
    git(&a, &["add", ".tandem"]);
    git(&a, &["commit", "--quiet", "-m", "legacy board"]);
    git(&a, &["push", "--quiet"]);
    let b = world.clone("b");
    // A publishes task-2 the old way; B creates its own task-2 and edits task-1.
    fs::write(
        a.join(".tandem/tasks/task-2.md"),
        legacy_task("task-2", "A's second"),
    )
    .unwrap();
    git(&a, &["add", ".tandem"]);
    git(&a, &["commit", "--quiet", "-m", "legacy task-2"]);
    git(&a, &["push", "--quiet"]);
    fs::write(
        b.join(".tandem/tasks/task-2.md"),
        legacy_task("task-2", "B's second"),
    )
    .unwrap();
    fs::write(
        b.join(".tandem/tasks/task-1.md"),
        legacy_task("task-1", "Legacy one, edited on B"),
    )
    .unwrap();

    let refused = fails(&b, &["list"]);
    assert!(refused.contains("tandem migrate"), "{refused}");

    let dry = ok(&a, &["migrate", "--dry-run"]);
    assert_eq!(dry["data"]["records"], 3);
    assert!(
        git(&a, &["status", "--porcelain"]).is_empty(),
        "dry run changes nothing"
    );
    let migrated = ok(&a, &["migrate"]);
    assert!(migrated["data"]["sourceCommit"].is_string());
    assert!(git(&a, &["ls-files", ".tandem"]).is_empty());
    assert!(git(&a, &["status", "--porcelain"]).is_empty());
    git(&a, &["push", "--quiet"]);
    assert_eq!(ids(&a), vec!["task-1", "task-2"]);
    assert!(fs::read_to_string(a.join(".tandem/tasks/task-1.md"))
        .unwrap()
        .contains("\nuid: "));

    let adopted = ok(&b, &["migrate", "--adopt"]);
    assert_eq!(adopted["data"]["newRecords"], serde_json::json!(["task-2"]));
    assert_eq!(adopted["data"]["renumbered"][0]["old"], "task-2");
    assert_eq!(adopted["data"]["renumbered"][0]["new"], "task-3");
    assert_eq!(adopted["data"]["needsPull"], true);
    git(&b, &["pull", "--quiet", "--ff-only"]);
    assert_eq!(ids(&b), vec!["task-1", "task-2", "task-3"]);
    assert_eq!(title(&b, "task-1"), "Legacy one, edited on B");
    assert_eq!(title(&b, "task-2"), "A's second");
    assert_eq!(title(&b, "task-3"), "B's second");
    assert!(git(&b, &["status", "--porcelain"]).is_empty());
    ok(&a, &["sync"]);
    assert_eq!(ids(&a), vec!["task-1", "task-2", "task-3"]);
    assert_eq!(title(&a, "task-1"), "Legacy one, edited on B");

    // A machine with no local board changes simply pulls.
    let c_path = world.root.join("c");
    git(
        &world.root,
        &[
            "clone",
            "--quiet",
            world.remote.to_str().unwrap(),
            c_path.to_str().unwrap(),
        ],
    );
    assert_eq!(ids(&c_path), vec!["task-1", "task-2", "task-3"]);
}

// -- protocol 0.4.0 -> 0.5.0 -------------------------------------------------

/// Rewrites the board's protocol version on disk and publishes the result as
/// a raw commit on the shared `tandem` branch, the way a 0.4.0 Tandem would
/// have left it, then rebases this checkout's sync bookkeeping onto it.
fn downgrade_board_to_previous_protocol(repo: &Path) {
    let config = repo.join(".tandem/tandem.md");
    let content = fs::read_to_string(&config).unwrap();
    fs::write(
        &config,
        content.replace("protocolVersion: 0.5.0", "protocolVersion: 0.4.0"),
    )
    .unwrap();
    let git_dir = repo.join(".git");
    let index = git_dir.join("tandem-test-index");
    let run = |args: &[&str]| -> String {
        let output = Command::new("git")
            .args(["--git-dir", git_dir.to_str().unwrap()])
            .args(["--work-tree", repo.join(".tandem").to_str().unwrap()])
            .args(args)
            .env("GIT_INDEX_FILE", &index)
            .current_dir(repo.join(".tandem"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };
    run(&[
        "add",
        "-A",
        "--",
        "tandem.md",
        "tasks",
        "logs",
        "decisions",
        "rules",
        "events",
    ]);
    let tree = run(&["write-tree"]);
    let parent = run(&["rev-parse", "refs/tandem/base"]);
    let commit = run(&[
        "commit-tree",
        &tree,
        "-p",
        &parent,
        "-m",
        "previous protocol",
    ]);
    run(&["update-ref", "refs/tandem/base", &commit]);
    run(&[
        "push",
        "--quiet",
        "origin",
        &format!("{commit}:refs/heads/tandem"),
    ]);
    let _ = fs::remove_file(index);
}

fn field(repo: &Path, id: &str, name: &str) -> Value {
    ok(repo, &["show", id])["data"][name].clone()
}

#[test]
fn migrate_upgrades_a_previous_protocol_board_once_for_every_machine() {
    let (world, a, _b) = World::two_machines("upgrade");
    let papercut = add_task(&a, "Papercut");
    let research = add_task(&a, "Research");
    let both = add_task(&a, "Both tags");
    let epic = add_task(&a, "Epic");
    let archived = add_task(&a, "Archived papercut");
    let id = |value: &Value| value["data"]["id"].as_str().unwrap().to_string();
    let (papercut, research, both, epic, archived) = (
        id(&papercut),
        id(&research),
        id(&both),
        id(&epic),
        id(&archived),
    );
    let sub = id(&ok(
        &a,
        &[
            "add",
            "task",
            "Papercut subtask",
            "--parent",
            &research,
            "--acceptance",
            "done",
            "--tag",
            "papercut",
        ],
    ));
    // Build the 0.4.0 tag vocabulary: kinds did not exist, tags carried them.
    ok(
        &a,
        &[
            "update", &papercut, "--tag", "papercut", "--tag", "friction",
        ],
    );
    ok(&a, &["update", &research, "--tag", "research"]);
    ok(
        &a,
        &["update", &both, "--tag", "research", "--tag", "papercut"],
    );
    ok(
        &a,
        &["update", &epic, "--kind", "epic", "--tag", "research"],
    );
    ok(&a, &["update", &archived, "--tag", "papercut"]);
    ok(&a, &["cancel", &archived, "--note", "not needed"]);
    let log_path = a.join(format!(".tandem/logs/{archived}.md"));
    let log_before = fs::read(&log_path).unwrap();
    downgrade_board_to_previous_protocol(&a);

    // Machine B downloads the 0.4.0 board and is refused with the upgrade path.
    let b = world.clone("b2");
    let refused = fails(&b, &["list"]);
    assert!(refused.contains("This board uses protocol 0.4.0; this Tandem version requires 0.5.0. Run `tandem migrate` to upgrade it. Every machine that shares this board must install this Tandem version before the board is migrated and synced"), "{refused}");
    let refused = fails(&a, &["sync"]);
    assert!(
        refused.contains("Run `tandem migrate` to upgrade it"),
        "{refused}"
    );

    let dry = ok(&a, &["migrate", "--dry-run"]);
    assert_eq!(dry["data"]["fromVersion"], "0.4.0");
    assert_eq!(dry["data"]["toVersion"], "0.5.0");
    assert_eq!(
        dry["data"]["kinds"]["converted"].as_array().unwrap().len(),
        2
    );
    assert!(fs::read_to_string(a.join(".tandem/tandem.md"))
        .unwrap()
        .contains("protocolVersion: 0.4.0"));

    let migrated = ok(&a, &["migrate"]);
    assert_eq!(migrated["data"]["upgraded"], "published");
    assert_eq!(
        migrated["data"]["kinds"]["converted"],
        serde_json::json!([
            {"id": papercut, "kind": "papercut"},
            {"id": research, "kind": "research"},
        ])
    );
    let skipped = migrated["data"]["kinds"]["skipped"].as_array().unwrap();
    let reasons: Vec<(&str, &str)> = skipped
        .iter()
        .map(|item| {
            (
                item["id"].as_str().unwrap(),
                item["reason"].as_str().unwrap(),
            )
        })
        .collect();
    assert!(
        reasons.contains(&(both.as_str(), "tagged both research and papercut")),
        "{reasons:?}"
    );
    assert!(
        reasons.contains(&(
            epic.as_str(),
            "already has kind `epic` and a `research` tag"
        )),
        "{reasons:?}"
    );
    assert!(
        reasons
            .iter()
            .any(|(id, reason)| *id == sub && reason.contains("a papercut cannot be a Subtask")),
        "{reasons:?}"
    );

    assert_eq!(field(&a, &papercut, "kind"), "papercut");
    assert_eq!(
        field(&a, &papercut, "tags"),
        serde_json::json!(["friction"])
    );
    assert_eq!(field(&a, &research, "kind"), "research");
    assert_eq!(field(&a, &research, "tags"), serde_json::json!([]));
    assert_eq!(field(&a, &both, "kind"), Value::Null);
    assert_eq!(
        field(&a, &both, "tags"),
        serde_json::json!(["research", "papercut"])
    );
    assert_eq!(field(&a, &epic, "kind"), "epic");
    assert_eq!(field(&a, &sub, "kind"), Value::Null);
    assert_eq!(
        fs::read(&log_path).unwrap(),
        log_before,
        "Logs stay byte-identical"
    );
    assert!(git(&a, &["show", "refs/tandem/base:tandem.md"]).contains("protocolVersion: 0.5.0"));
    assert_eq!(ok(&a, &["sync"])["data"]["sync"]["status"], "synced");

    // B downloads the already upgraded shared board instead of converting.
    let received = ok(&b, &["migrate"]);
    assert_eq!(received["data"]["upgraded"], "received");
    assert_eq!(field(&b, &papercut, "kind"), "papercut");
    assert_eq!(
        fs::read(b.join(".tandem/tasks").join(format!("{papercut}.md"))).unwrap(),
        fs::read(a.join(".tandem/tasks").join(format!("{papercut}.md"))).unwrap()
    );

    let again = fails(&a, &["migrate"]);
    assert!(
        again.contains("this board is already at protocol 0.5.0; nothing to migrate"),
        "{again}"
    );
}

#[test]
fn migrate_refuses_an_unknown_protocol_version() {
    let (_world, a, _b) = World::two_machines("unknown-version");
    let config = a.join(".tandem/tandem.md");
    let content = fs::read_to_string(&config).unwrap();
    fs::write(
        &config,
        content.replace("protocolVersion: 0.5.0", "protocolVersion: 0.2.0"),
    )
    .unwrap();
    let error = fails(&a, &["migrate"]);
    assert!(
        error.contains("tandem migrate converts protocol 0.3.0 and 0.4.0 boards; found `0.2.0`"),
        "{error}"
    );
}
