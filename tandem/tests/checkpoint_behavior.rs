//! Forward-only native checkpoint behavior against disposable real Git and
//! Worktrunk repositories.
//!
//! These tests use the actual CLI binary. They prove lifecycle mutations never
//! touch Git, that an explicit flush batches all owning `.tandem` changes into
//! one ordinary forward commit, that existing history is never rewritten, and
//! that a source-only Worktrunk worker merge integrates over pending target
//! metadata without stale replay.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const CHECKPOINT_SUBJECT: &str = "chore(tandem): checkpoint metadata";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-forward-checkpoint-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git_status(cwd: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap()
        .success()
}

fn run(cwd: &Path, args: &[&str]) -> (bool, String, String) {
    let output = bin().args(args).current_dir(cwd).output().unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn head(cwd: &Path) -> String {
    git(cwd, &["rev-parse", "HEAD"])
}

fn commit_count(cwd: &Path) -> u32 {
    git(cwd, &["rev-list", "--count", "HEAD"]).parse().unwrap()
}

/// A disposable Git-backed workspace whose baseline commit contains Tandem
/// metadata.
fn setup(label: &str) -> PathBuf {
    let root = root(label);
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "--quiet"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Tandem Tests"]);
    let (ok, _, stderr) = run(&root, &["init", "--title", "Checkpoint tests"]);
    assert!(ok, "init failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Checkpoint task",
            "--acceptance",
            "boundary is durable",
        ],
    );
    assert!(ok, "add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "fixture baseline"]);
    root
}

fn json(stdout: &str) -> Value {
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!("expected one complete JSON envelope, got {stdout:?}: {error}")
    })
}

/// Every file under `dir` as a sorted `(relative path, contents)` snapshot.
/// Used to prove the explicit flush authors no document, rule, or event bytes.
fn snapshot_tree(dir: &Path) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    if !dir.exists() {
        return entries;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut children = fs::read_dir(&current)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        children.sort();
        for path in children {
            if path.is_dir() {
                stack.push(path);
            } else {
                entries.push((
                    path.strip_prefix(dir).unwrap().display().to_string(),
                    fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    entries.sort();
    entries
}

fn write_hook(root: &Path, name: &str, script: &str) {
    let hook = root.join(".git/hooks").join(name);
    fs::write(&hook, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).unwrap();
    }
}

/// `wt` is mandatory for the Worktrunk integration evidence. A missing tool is
/// a hard failure, not a silent skip.
fn require_wt() {
    let output = Command::new("wt").arg("--version").output();
    match output {
        Ok(output) if output.status.success() => {}
        _ => panic!("Worktrunk `wt` is required for the integration evidence but is not runnable"),
    }
}

fn wt_merge(worktree: &Path, target: &str) -> (bool, String) {
    let output = Command::new("wt")
        .args(["merge", target, "--no-remove", "--yes"])
        .current_dir(worktree)
        .output()
        .expect("failed to run wt");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn remove_worktree(root: &Path, worktree: &Path) {
    let _ = Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(worktree)
        .output();
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force", worktree.to_str().unwrap()])
        .current_dir(root)
        .output();
    let _ = fs::remove_dir_all(worktree);
}

fn upstream(root: &Path) {
    let bare = root.with_extension("upstream.git");
    git(root, &["init", "--bare", bare.to_str().unwrap()]);
    git(root, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git(root, &["push", "--quiet", "-u", "origin", "HEAD"]);
}

#[test]
fn consolidate_interleaved_checkpoints_preserves_tree_and_real_commit_order() {
    let root = setup("consolidate-interleaved");
    upstream(&root);
    let base = head(&root);
    fs::write(root.join(".tandem/one.txt"), "first\n").unwrap();
    assert!(run(&root, &["checkpoint"]).0);
    fs::write(root.join("source.txt"), "source one\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(&root, &["commit", "--quiet", "-m", "feat: first"]);
    let real_one = head(&root);
    fs::write(root.join(".tandem/two.txt"), "second\n").unwrap();
    assert!(run(&root, &["checkpoint"]).0);
    fs::write(root.join("source.txt"), "source two\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(&root, &["commit", "--quiet", "-m", "fix: second"]);
    let real_two = head(&root);
    fs::write(root.join(".tandem/three.txt"), "third\n").unwrap();
    let original_tree = git(&root, &["rev-parse", "HEAD^{tree}"]);
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(ok, "{stdout} {stderr}");
    let result = json(&stdout);
    assert_eq!(result["data"]["checkpoint"]["collapsed"], 3);
    let old = result["data"]["checkpoint"]["oldHead"].as_str().unwrap();
    let new = result["data"]["checkpoint"]["newHead"].as_str().unwrap();
    assert_eq!(head(&root), new);
    assert_ne!(old, new);
    assert_eq!(git(&root, &["rev-parse", "@{upstream}"]), base);
    assert_eq!(git(&root, &["merge-base", "HEAD", &base]), base);
    assert_eq!(
        git(&root, &["rev-list", "--count", "@{upstream}..HEAD"]),
        "3"
    );
    assert_eq!(
        git(
            &root,
            &["log", "--reverse", "--format=%s", "@{upstream}..HEAD"]
        ),
        format!("feat: first\nfix: second\n{CHECKPOINT_SUBJECT}")
    );
    assert_ne!(git(&root, &["rev-parse", "HEAD~2"]), real_one);
    assert_ne!(git(&root, &["rev-parse", "HEAD~1"]), real_two);
    assert_eq!(
        git(&root, &["rev-parse", "HEAD^{tree}"]),
        git(&root, &["rev-parse", &format!("{old}^{{tree}}")])
    );
    assert_ne!(original_tree, git(&root, &["rev-parse", "HEAD^{tree}"]));
    assert!(git(&root, &["status", "--porcelain"]).is_empty());
    fs::remove_dir_all(root.with_extension("upstream.git")).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn consolidation_preserves_unrelated_staged_unstaged_and_untracked_state() {
    let root = setup("consolidate-dirty-target");
    fs::write(root.join("source.txt"), "baseline\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(&root, &["commit", "--quiet", "-m", "source baseline"]);
    upstream(&root);
    let pushed = head(&root);

    fs::write(root.join(".tandem/one.txt"), "first\n").unwrap();
    assert!(run(&root, &["checkpoint"]).0);
    fs::write(root.join("feature.txt"), "committed source\n").unwrap();
    git(&root, &["add", "feature.txt"]);
    git(&root, &["commit", "--quiet", "-m", "feature"]);
    fs::write(root.join(".tandem/two.txt"), "second\n").unwrap();

    // Staged modification with a further unstaged edit to that same path,
    // staged addition, and untracked source file must all survive exactly.
    fs::write(root.join("source.txt"), "staged\n").unwrap();
    git(&root, &["add", "source.txt"]);
    fs::write(root.join("source.txt"), "unstaged\n").unwrap();
    fs::write(root.join("new-staged.txt"), "staged addition\n").unwrap();
    git(&root, &["add", "new-staged.txt"]);
    fs::write(root.join("untracked.txt"), "untracked\n").unwrap();
    let paths = ["source.txt", "new-staged.txt", "untracked.txt"];
    let before_index = git(
        &root,
        &["ls-files", "--stage", "--", "source.txt", "new-staged.txt"],
    );
    let before_staged = git(
        &root,
        &[
            "diff",
            "--cached",
            "--binary",
            "--",
            "source.txt",
            "new-staged.txt",
        ],
    );
    let before_status = git(
        &root,
        &["status", "--porcelain", "--", paths[0], paths[1], paths[2]],
    );
    let before_bytes = paths.map(|path| fs::read(root.join(path)).unwrap());

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(ok, "{stdout} {stderr}");
    let result = json(&stdout);
    assert_eq!(result["data"]["checkpoint"]["collapsed"], 2);
    let old = result["data"]["checkpoint"]["oldHead"].as_str().unwrap();
    assert_ne!(old, head(&root));
    assert_eq!(
        git(&root, &["rev-parse", "HEAD^{tree}"]),
        git(&root, &["rev-parse", &format!("{old}^{{tree}}")])
    );
    assert_eq!(git(&root, &["rev-parse", "@{upstream}"]), pushed);
    assert_eq!(git(&root, &["show", "HEAD:source.txt"]), "baseline");
    assert_eq!(
        git(
            &root,
            &["ls-files", "--stage", "--", "source.txt", "new-staged.txt"]
        ),
        before_index
    );
    assert_eq!(
        git(
            &root,
            &[
                "diff",
                "--cached",
                "--binary",
                "--",
                "source.txt",
                "new-staged.txt"
            ]
        ),
        before_staged
    );
    assert_eq!(
        git(
            &root,
            &["status", "--porcelain", "--", paths[0], paths[1], paths[2]]
        ),
        before_status
    );
    assert_eq!(
        paths.map(|path| fs::read(root.join(path)).unwrap()),
        before_bytes
    );
    assert!(git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty());
    fs::remove_dir_all(root.with_extension("upstream.git")).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn consolidate_refuses_live_worktree_without_rewriting() {
    let root = setup("consolidate-live-worktree");
    upstream(&root);
    fs::write(root.join(".tandem/pending.txt"), "first\n").unwrap();
    assert!(run(&root, &["checkpoint"]).0);
    let branch_point = head(&root);
    let linked = root.with_extension("linked");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "worker",
            linked.to_str().unwrap(),
        ],
    );
    fs::write(root.join("source.txt"), "source\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(&root, &["commit", "--quiet", "-m", "real"]);
    let before = head(&root);
    fs::write(root.join(".tandem/late.txt"), "pending at push boundary\n").unwrap();
    let (ok, stdout, _) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(!ok);
    assert_eq!(json(&stdout)["error"]["code"], "checkpoint");
    assert!(json(&stdout)["error"]["message"]
        .as_str()
        .unwrap()
        .contains("branch or linked worktree"));
    assert_ne!(
        head(&root),
        before,
        "pending metadata may flush before refusal"
    );
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), before);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        CHECKPOINT_SUBJECT
    );
    let flushed = head(&root);
    let (ok, stdout, _) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(!ok);
    assert_eq!(json(&stdout)["error"]["code"], "checkpoint");
    assert_eq!(head(&root), flushed, "repeat refusal must not rewrite");
    assert_eq!(head(&linked), branch_point);
    git(
        &root,
        &["worktree", "remove", "--force", linked.to_str().unwrap()],
    );
    fs::remove_dir_all(root.with_extension("upstream.git")).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn consolidate_refuses_unsafe_ranges_and_git_operations() {
    // A pending flush can append one ordinary checkpoint before a missing
    // upstream is detected; it must never rewrite the previous HEAD.
    let root = setup("consolidate-no-upstream");
    let before = head(&root);
    fs::write(root.join(".tandem/late.txt"), "pending at push boundary\n").unwrap();
    let (ok, stdout, _) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(!ok);
    assert_eq!(json(&stdout)["error"]["code"], "checkpoint");
    assert_ne!(head(&root), before);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), before);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        CHECKPOINT_SUBJECT
    );
    let flushed = head(&root);
    let (ok, stdout, _) = run(&root, &["--json", "checkpoint", "--consolidate"]);
    assert!(!ok);
    assert_eq!(json(&stdout)["error"]["code"], "checkpoint");
    assert_eq!(head(&root), flushed, "repeat refusal must not rewrite");
    fs::remove_dir_all(root).unwrap();

    for kind in ["real-metadata", "merge", "operation"] {
        let root = setup(&format!("consolidate-{kind}"));
        upstream(&root);
        fs::write(root.join(".tandem/pending.txt"), "pending\n").unwrap();
        assert!(run(&root, &["checkpoint"]).0);
        match kind {
            "real-metadata" => {
                fs::write(root.join(".tandem/pending.txt"), "manual change\n").unwrap();
                git(&root, &["add", ".tandem/pending.txt"]);
                git(&root, &["commit", "--quiet", "-m", "manual metadata"]);
            }
            "merge" => {
                git(&root, &["checkout", "--quiet", "-b", "side"]);
                fs::write(root.join("side.txt"), "side\n").unwrap();
                git(&root, &["add", "side.txt"]);
                git(&root, &["commit", "--quiet", "-m", "side"]);
                git(&root, &["checkout", "--quiet", "-"]);
                fs::write(root.join("main.txt"), "main\n").unwrap();
                git(&root, &["add", "main.txt"]);
                git(&root, &["commit", "--quiet", "-m", "main"]);
                git(
                    &root,
                    &["merge", "--quiet", "--no-ff", "side", "-m", "merge"],
                );
            }
            "operation" => {
                fs::write(root.join(".git/MERGE_HEAD"), head(&root)).unwrap();
            }
            _ => unreachable!(),
        }
        let before = head(&root);
        let (ok, stdout, _) = run(&root, &["--json", "checkpoint", "--consolidate"]);
        assert!(!ok, "{kind}: {stdout}");
        assert_eq!(json(&stdout)["error"]["code"], "checkpoint");
        assert_eq!(head(&root), before, "{kind}");
        fs::remove_dir_all(root.with_extension("upstream.git")).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn lifecycle_mutations_persist_without_per_event_commits_or_history_rewrites() {
    let root = setup("lifecycle-persist");
    let baseline = head(&root);
    let baseline_count = commit_count(&root);

    let steps: Vec<Vec<&str>> = vec![
        vec!["--json", "update", "task-1", "--body", "first progress"],
        vec![
            "--json",
            "add",
            "task",
            "Child task",
            "--parent",
            "task-1",
            "--acceptance",
            "child outcome",
        ],
        vec![
            "--json",
            "add",
            "task",
            "Second child",
            "--parent",
            "task-1",
            "--acceptance",
            "second outcome",
        ],
        vec![
            "--json",
            "rules",
            "add",
            "prefer",
            "Batch metadata for the host boundary",
        ],
        vec![
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
        vec![
            "--json",
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "ready",
            "--evidence",
            "observed",
        ],
        vec![
            "--json",
            "review",
            "task-1",
            "--criterion",
            "boundary is durable",
            "--note",
            "human check",
        ],
    ];

    for args in &steps {
        let (ok, stdout, stderr) = run(&root, args);
        assert!(ok, "{args:?} failed: {stderr}");
        let value = json(&stdout);
        if let Some(checkpoint) = value["data"].get("checkpoint") {
            assert_eq!(
                checkpoint["status"], "batched",
                "{args:?} must report pending batched metadata"
            );
            assert_eq!(checkpoint["commit"], Value::Null, "{args:?}");
        }
        assert_eq!(head(&root), baseline, "{args:?} must not commit");
    }

    assert_eq!(commit_count(&root), baseline_count);
    assert!(
        !git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty(),
        "lifecycle writes must leave pending metadata"
    );
    assert_eq!(
        git(&root, &["rev-parse", "HEAD"]),
        baseline,
        "no history rewrite may occur"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_checkpoint_batches_additions_modifications_and_deletions_in_one_forward_commit() {
    let root = setup("batch-flush");
    fs::write(root.join("src-tracked.txt"), "base\n").unwrap();
    fs::write(root.join(".tandem/tracked-scratch.txt"), "scratch\n").unwrap();
    git(
        &root,
        &["add", "src-tracked.txt", ".tandem/tracked-scratch.txt"],
    );
    git(
        &root,
        &["commit", "--quiet", "-m", "source and scratch baseline"],
    );
    let baseline = head(&root);

    // Pending owning `.tandem` metadata: modification, addition, and deletion.
    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "pending modification"],
    );
    assert!(ok, "update failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Pending addition",
            "--acceptance",
            "captured",
        ],
    );
    assert!(ok, "add failed: {stderr}");
    let (ok, _, stderr) = run(&root, &["rules", "add", "prefer", "Pending rule text"]);
    assert!(ok, "rule add failed: {stderr}");
    fs::remove_file(root.join(".tandem/tracked-scratch.txt")).unwrap();

    // Unrelated staged, unstaged, and untracked source state.
    fs::write(root.join("src-tracked.txt"), "staged\n").unwrap();
    git(&root, &["add", "src-tracked.txt"]);
    fs::write(root.join("src-tracked.txt"), "unstaged\n").unwrap();
    fs::write(root.join("src-staged-add.txt"), "staged addition\n").unwrap();
    git(&root, &["add", "src-staged-add.txt"]);
    fs::write(root.join("src-untracked.txt"), "untracked\n").unwrap();

    let index_before = git(&root, &["ls-files", "--stage", "--", "src-tracked.txt"]);
    let staged_before = git(&root, &["diff", "--cached", "--name-status", "--", "."]);
    let tasks_before = snapshot_tree(&root.join(".tandem/tasks"));
    let events_before = snapshot_tree(&root.join(".tandem/events"));

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    let commit = value["data"]["checkpoint"]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(head(&root), commit);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), baseline);
    assert_eq!(commit_count(&root), 3);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        CHECKPOINT_SUBJECT
    );

    // The flush captures every owning change, including the deletion.
    assert!(git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty());
    let tree = git(&root, &["ls-tree", "-r", "--name-only", "HEAD"]);
    assert!(tree.contains(".tandem/tasks/task-2.md"), "addition missing");
    assert!(
        !tree.contains(".tandem/tracked-scratch.txt"),
        "deletion not committed"
    );
    assert!(tree.contains(".tandem/rules"), "rule metadata missing");
    let committed_task = git(&root, &["show", "HEAD:.tandem/tasks/task-1.md"]);
    assert!(committed_task.contains("pending modification"));

    // The flush itself authors no Task, Rule, Decision, or event bytes.
    assert_eq!(snapshot_tree(&root.join(".tandem/tasks")), tasks_before);
    assert_eq!(snapshot_tree(&root.join(".tandem/events")), events_before);

    // Unrelated index, staged paths, and worktree bytes are unchanged.
    assert_eq!(
        git(&root, &["ls-files", "--stage", "--", "src-tracked.txt"]),
        index_before
    );
    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status", "--", "."]),
        staged_before
    );
    assert_eq!(
        fs::read_to_string(root.join("src-tracked.txt")).unwrap(),
        "unstaged\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("src-staged-add.txt")).unwrap(),
        "staged addition\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("src-untracked.txt")).unwrap(),
        "untracked\n"
    );

    // A repeated clean flush adds nothing.
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "repeat flush failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "clean");
    assert_eq!(value["data"]["checkpoint"]["commit"], Value::Null);
    assert_eq!(head(&root), commit);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn metadata_only_flush_creates_one_forward_commit_and_keeps_source_identity() {
    let root = setup("metadata-only");
    let baseline = head(&root);
    fs::write(root.join("source.txt"), "source\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(
        &root,
        &["commit", "--quiet", "-m", "ordinary source commit"],
    );
    let source_commit = head(&root);

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "metadata only"]);
    assert!(ok, "update failed: {stderr}");

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    let commit = value["data"]["checkpoint"]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    // The source commit is neither amended nor rebased: it is HEAD^, its
    // subject is intact, and the original baseline is still its parent.
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), source_commit);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s", "HEAD^"]),
        "ordinary source commit"
    );
    assert_eq!(git(&root, &["rev-parse", "HEAD^^"]), baseline);
    assert_eq!(commit_count(&root), 3);

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "repeat flush failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["status"], "clean");
    assert_eq!(head(&root), commit);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pushed_commits_are_never_rewritten_and_the_flush_appends() {
    let root = setup("pushed-preserved");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let pushed = head(&root);

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "after push"]);
    assert!(ok, "update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_ne!(head(&root), pushed);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), pushed);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        pushed
    );

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}

#[test]
fn clean_flush_is_a_noop_and_ignores_unrelated_dirt() {
    let root = setup("clean-noop");
    let baseline = head(&root);
    fs::write(root.join("dirty-untracked.txt"), "dirty\n").unwrap();
    fs::write(root.join(".tandem-adjacent.txt"), "not tandem\n").unwrap();

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "clean");
    assert_eq!(value["data"]["checkpoint"]["commit"], Value::Null);
    assert_eq!(head(&root), baseline);
    assert!(root.join("dirty-untracked.txt").exists());
    assert!(root.join(".tandem-adjacent.txt").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_checkpoint_fails_closed_on_non_git_and_hook_failure() {
    // A non-Git workspace has no forward flush.
    let non_git = root("non-git");
    fs::create_dir_all(&non_git).unwrap();
    let (ok, _, stderr) = run(&non_git, &["init", "--title", "No Git"]);
    assert!(ok, "init failed: {stderr}");
    let (ok, _, stderr) = run(
        &non_git,
        &["add", "task", "No Git task", "--acceptance", "durable"],
    );
    assert!(ok, "add failed: {stderr}");
    let output = bin()
        .args(["--json", "checkpoint"])
        .current_dir(&non_git)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty(), "JSON failure must be stdout-only");
    let value = json(&String::from_utf8_lossy(&output.stdout));
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "checkpoint");
    assert_eq!(value["error"]["details"]["checkpoint"]["status"], "failed");
    assert_eq!(
        value["error"]["details"]["checkpoint"]["commit"],
        Value::Null
    );
    let message = value["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("Git"),
        "message lost Git detail: {message}"
    );
    assert_eq!(value["error"]["details"]["checkpoint"]["error"], message);

    let output = bin()
        .arg("checkpoint")
        .current_dir(&non_git)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "human failure must not write stdout"
    );
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("Error:"));
    fs::remove_dir_all(&non_git).unwrap();

    // A failing commit hook leaves the pending change staged for inspection.
    let root = setup("hook-failure");
    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "pending hook failure"],
    );
    assert!(ok, "update failed: {stderr}");
    write_hook(
        &root,
        "pre-commit",
        "#!/bin/sh\necho intentional checkpoint hook failure >&2\nexit 1\n",
    );
    let output = bin()
        .args(["--json", "checkpoint"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let value = json(&String::from_utf8_lossy(&output.stdout));
    assert_eq!(value["error"]["code"], "checkpoint");
    assert!(value["error"]["details"]["checkpoint"]["error"]
        .as_str()
        .unwrap()
        .contains("intentional checkpoint hook failure"));
    assert!(
        !git_status(&root, &["diff", "--cached", "--quiet", "--", ".tandem"]),
        "failed checkpoint must leave .tandem staged for inspection"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn worktrunk_source_only_merge_keeps_base_identity_and_pending_metadata() {
    require_wt();
    let root = root("worktrunk-merge");
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "--quiet", "-b", "main"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Tandem Tests"]);
    let (ok, _, stderr) = run(&root, &["init", "--title", "Worktrunk fixture"]);
    assert!(ok, "init failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Target task",
            "--acceptance",
            "target outcome",
        ],
    );
    assert!(ok, "add failed: {stderr}");
    fs::write(root.join("src.txt"), "base\n").unwrap();
    fs::write(root.join(".tandem/tracked-scratch.txt"), "scratch\n").unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "--quiet", "-m", "baseline"]);
    let base = head(&root);

    // Source-only worker branch: one commit, no Tandem metadata.
    let worker = root.with_extension("worker");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "worker",
            worker.to_str().unwrap(),
            &base,
        ],
    );
    fs::write(worker.join("worker-source.txt"), "worker source\n").unwrap();
    git(&worker, &["add", "worker-source.txt"]);
    git(&worker, &["commit", "--quiet", "-m", "worker source"]);

    // The target advances with a forward metadata commit, then accumulates
    // pending tracked modification, tracked deletion, and untracked addition
    // metadata plus unrelated staged/unstaged/untracked source dirt.
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Target forward",
            "--acceptance",
            "forward outcome",
        ],
    );
    assert!(ok, "forward add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", CHECKPOINT_SUBJECT]);
    let target_commit = head(&root);

    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "pending target modification"],
    );
    assert!(ok, "target update failed: {stderr}");
    fs::remove_file(root.join(".tandem/tracked-scratch.txt")).unwrap();
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Pending untracked",
            "--acceptance",
            "pending captured",
        ],
    );
    assert!(ok, "pending add failed: {stderr}");
    fs::write(root.join("src.txt"), "staged\n").unwrap();
    git(&root, &["add", "src.txt"]);
    fs::write(root.join("src.txt"), "unstaged\n").unwrap();
    fs::write(root.join("src-untracked.txt"), "untracked\n").unwrap();
    let index_before = git(&root, &["ls-files", "--stage", "--", "src.txt"]);
    let staged_before = git(&root, &["diff", "--cached", "--name-status", "--", "."]);

    let (merged, output) = wt_merge(&worker, "main");
    assert!(merged, "wt merge failed: {output}");
    assert!(
        !output.contains("CONFLICT") && !output.contains("could not apply"),
        "unexpected conflict in a source-only integration: {output}"
    );

    // Pre-existing source commit identities stay in history, unchanged.
    let history = git(&root, &["log", "--format=%H"]);
    assert!(history.lines().any(|line| line == base), "base commit lost");
    assert!(
        history.lines().any(|line| line == target_commit),
        "target commit was rewritten"
    );
    assert_eq!(git(&root, &["log", "-1", "--format=%s"]), "worker source");
    assert!(root.join("worker-source.txt").exists());

    // Pending target metadata survived the merge and was not replayed stale.
    let pending_task = fs::read_to_string(root.join(".tandem/tasks/task-1.md")).unwrap();
    assert!(pending_task.contains("pending target modification"));
    assert!(!root.join(".tandem/tracked-scratch.txt").exists());
    assert!(root.join(".tandem/tasks/task-3.md").exists());
    assert!(
        !git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty(),
        "pending metadata must still await a flush"
    );
    assert_eq!(
        fs::read_to_string(root.join("src.txt")).unwrap(),
        "unstaged\n"
    );
    assert_eq!(
        git(&root, &["ls-files", "--stage", "--", "src.txt"]),
        index_before,
        "unrelated staged index entry must survive the merge"
    );
    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status", "--", "."]),
        staged_before,
        "unrelated staged paths must survive the merge"
    );
    assert!(root.join("src-untracked.txt").exists());

    // One later forward flush commits the pending metadata on top of the merge.
    let merge_head = head(&root);
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "flush failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    let commit = value["data"]["checkpoint"]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), merge_head);
    assert!(git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty());
    assert_eq!(
        git(&root, &["ls-files", "--stage", "--", "src.txt"]),
        index_before,
        "the flush must not touch the unrelated index"
    );
    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status", "--", "."]),
        staged_before,
        "the flush must not touch unrelated staged paths"
    );
    assert_eq!(
        fs::read_to_string(root.join("src.txt")).unwrap(),
        "unstaged\n"
    );
    assert!(root.join("src-untracked.txt").exists());

    // A clean repeat flush adds nothing.
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "repeat flush failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["status"], "clean");
    assert_eq!(head(&root), commit);

    remove_worktree(&root, &worker);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn worktrunk_overlapping_metadata_conflict_is_reported_without_suppression() {
    require_wt();
    let root = root("worktrunk-conflict");
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "--quiet", "-b", "main"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Tandem Tests"]);
    let (ok, _, stderr) = run(&root, &["init", "--title", "Conflict fixture"]);
    assert!(ok, "init failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Conflict task",
            "--acceptance",
            "conflict outcome",
        ],
    );
    assert!(ok, "add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "baseline"]);
    let base = head(&root);

    let worker = root.with_extension("worker-conflict");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "worker-conflict",
            worker.to_str().unwrap(),
            &base,
        ],
    );
    let (ok, _, stderr) = run(&worker, &["update", "task-1", "--body", "worker overlap"]);
    assert!(ok, "worker update failed: {stderr}");
    git(&worker, &["add", ".tandem"]);
    git(
        &worker,
        &["commit", "--quiet", "-m", "worker metadata change"],
    );

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "main overlap"]);
    assert!(ok, "target update failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", CHECKPOINT_SUBJECT]);
    let main_head = head(&root);

    let (merged, output) = wt_merge(&worker, "main");
    assert!(
        !merged,
        "overlapping metadata must not be auto-merged: {output}"
    );
    assert!(
        output.contains("CONFLICT") || output.contains("could not apply"),
        "expected a real conflict report: {output}"
    );
    let conflicted = fs::read_to_string(worker.join(".tandem/tasks/task-1.md")).unwrap();
    assert!(
        conflicted.contains("<<<<<<<"),
        "conflict must be left for a human, not resolved: {conflicted}"
    );
    assert_eq!(head(&root), main_head, "the target must be untouched");

    remove_worktree(&root, &worker);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_flush_processes_share_git_repository_lock() {
    let root = setup("concurrent");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Second checkpoint task",
            "--acceptance",
            "boundary is durable",
        ],
    );
    assert!(ok, "second add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "concurrent fixture"]);
    let linked = root.with_extension("linked");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            linked.to_str().unwrap(),
            "HEAD",
        ],
    );

    let first = bin()
        .args(["--json", "checkpoint"])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let second = bin()
        .args(["--json", "checkpoint"])
        .current_dir(&linked)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let first_output = first.wait_with_output().unwrap();
    let second_output = second.wait_with_output().unwrap();
    assert!(
        first_output.status.success(),
        "first: {}",
        String::from_utf8_lossy(&first_output.stderr)
    );
    assert!(
        second_output.status.success(),
        "second: {}",
        String::from_utf8_lossy(&second_output.stderr)
    );
    assert_eq!(
        json(&String::from_utf8_lossy(&first_output.stdout))["data"]["checkpoint"]["status"],
        "clean"
    );
    assert_eq!(
        json(&String::from_utf8_lossy(&second_output.stdout))["data"]["checkpoint"]["status"],
        "clean"
    );
    assert!(git(&root, &["fsck", "--no-progress", "--full"]).is_empty());
    git(
        &root,
        &["worktree", "remove", "--force", linked.to_str().unwrap()],
    );
    fs::remove_dir_all(root).unwrap();
    if linked.exists() {
        fs::remove_dir_all(linked).unwrap();
    }
}
