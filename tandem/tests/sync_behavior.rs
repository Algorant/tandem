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
