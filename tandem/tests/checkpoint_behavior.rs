use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tandem"))
}

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tandem-real-git-{label}-{}-{}",
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

/// Exact Git stdout without trimming, for byte-for-byte blob comparisons.
fn git_raw(cwd: &Path, args: &[&str]) -> String {
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
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let output = bin().args(args).current_dir(root).output().unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

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

fn chore_commit(root: &Path, relative: &str) {
    fs::write(root.join(relative), "leftover checkpoint bytes\n").unwrap();
    git(root, &["add", ".tandem"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "-m",
            "chore(tandem): checkpoint metadata",
        ],
    );
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

#[test]
fn real_git_checkpoint_preserves_unrelated_index_worktree_and_untracked_state() {
    let root = setup("preservation");
    fs::write(root.join("partial.txt"), "base partial\n").unwrap();
    fs::write(root.join("deleted.txt"), "base deletion\n").unwrap();
    git(&root, &["add", "partial.txt", "deleted.txt"]);
    git(&root, &["commit", "--quiet", "-m", "unrelated fixture"]);
    fs::write(root.join("partial.txt"), "staged partial\n").unwrap();
    git(&root, &["add", "partial.txt"]);
    fs::write(root.join("partial.txt"), "unstaged partial\n").unwrap();
    fs::remove_file(root.join("deleted.txt")).unwrap();
    git(&root, &["add", "deleted.txt"]);
    fs::write(root.join("staged-add.txt"), "staged addition\n").unwrap();
    git(&root, &["add", "staged-add.txt"]);
    fs::write(root.join("untracked.txt"), "untracked bytes\n").unwrap();
    let partial_index_before = git(&root, &["ls-files", "--stage", "--", "partial.txt"]);
    let staged_paths_before = git(&root, &["diff", "--cached", "--name-status", "--", "."]);
    let head_before = git(&root, &["rev-parse", "HEAD"]);

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["recordWritten"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), head_before);
    assert_eq!(
        git(&root, &["ls-files", "--stage", "--", "partial.txt"]),
        partial_index_before
    );
    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status", "--", "."]),
        staged_paths_before
    );
    assert_eq!(
        fs::read_to_string(root.join("partial.txt")).unwrap(),
        "unstaged partial\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("untracked.txt")).unwrap(),
        "untracked bytes\n"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn boundaries_roll_up_into_one_amended_checkpoint_without_empty_commits() {
    let root = setup("boundaries");
    let baseline = git(&root, &["rev-parse", "HEAD"]);
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    let claim = json(&stdout);
    assert_eq!(claim["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(claim["data"]["checkpoint"]["amended"], true);
    let claim_head = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(claim_head, baseline);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "fixture baseline"
    );

    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "intermediate progress"],
    );
    assert!(ok, "update failed: {stderr}");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), claim_head);

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "ready",
            "--evidence",
            "observed",
        ],
    );
    assert!(ok, "deliver failed: {stderr}");
    let delivery = json(&stdout);
    assert_eq!(delivery["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(delivery["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");

    let (ok, stdout, stderr) = run(&root, &["--json", "complete", "task-1"]);
    assert!(ok, "complete failed: {stderr}");
    let complete = json(&stdout);
    assert_eq!(complete["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(complete["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "fixture baseline"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn milestones_batch_every_lifecycle_write_until_assignment_boundary() {
    let root = setup("milestone-boundaries");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Milestone",
            "--parent",
            "task-1",
            "--acceptance",
            "milestone outcome",
        ],
    );
    assert!(ok, "milestone add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "milestone fixture"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);

    for args in [
        vec![
            "--json",
            "accord",
            "claim",
            "task-1-1",
            "--assignee",
            "milestone-worker",
        ],
        vec!["--json", "accord", "block", "task-1-1", "--note", "waiting"],
        vec!["--json", "accord", "resume", "task-1-1"],
        vec![
            "--json",
            "accord",
            "deliver",
            "task-1-1",
            "--summary",
            "milestone ready",
            "--evidence",
            "milestone observed",
        ],
    ] {
        let (ok, stdout, stderr) = run(&root, &args);
        assert!(ok, "{args:?} failed: {stderr}");
        let value = json(&stdout);
        assert_eq!(value["data"]["checkpoint"]["status"], "batched");
        assert_eq!(git(&root, &["rev-parse", "HEAD"]), baseline);
    }
    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1-1", "--body", "milestone progress"],
    );
    assert!(ok, "milestone update failed: {stderr}");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), baseline);
    let (ok, stdout, stderr) = run(&root, &["--json", "complete", "task-1-1"]);
    assert!(ok, "milestone complete failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["status"], "batched");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), baseline);

    let (ok, stdout, stderr) = run(
        &root,
        &["--json", "accord", "claim", "task-1", "--assignee", "owner"],
    );
    assert!(ok, "root claim failed: {stderr}");
    assert_eq!(
        json(&stdout)["data"]["checkpoint"]["status"],
        "checkpointed"
    );
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), baseline);
    assert!(git(&root, &["show", "--format=", "--name-only", "HEAD"])
        .contains(".tandem/logs/task-1-1.md"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolved_epic_grouping_and_direct_epic_tasks_have_distinct_boundaries() {
    let root = setup("epic-roles");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Epic",
            "--kind",
            "epic",
            "--acceptance",
            "group work",
        ],
    );
    assert!(ok, "epic add failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Epic assignment",
            "--parent",
            "task-2",
            "--acceptance",
            "assignment outcome",
        ],
    );
    assert!(ok, "direct Epic Task add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "epic fixture"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);
    let (ok, stdout, stderr) = run(
        &root,
        &["--json", "accord", "claim", "task-2", "--assignee", "group"],
    );
    assert!(ok, "epic claim failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["status"], "batched");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), baseline);
    let (ok, stdout, stderr) = run(
        &root,
        &["--json", "accord", "claim", "task-3", "--assignee", "owner"],
    );
    assert!(ok, "direct Epic Task claim failed: {stderr}");
    assert_eq!(
        json(&stdout)["data"]["checkpoint"]["status"],
        "checkpointed"
    );
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), baseline);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_record_success_is_distinct_from_hook_checkpoint_failure() {
    let root = setup("hook-failure");
    let hook = root.join(".git/hooks/pre-commit");
    fs::write(
        &hook,
        "#!/bin/sh\necho intentional checkpoint hook failure >&2\nexit 1\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&hook).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).unwrap();
    }
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "native claim must remain successful: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["recordWritten"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "failed");
    assert!(value["data"]["checkpoint"]["error"]
        .as_str()
        .unwrap()
        .contains("intentional checkpoint hook failure"));
    let diff = Command::new("git")
        .args(["diff", "--quiet"])
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(
        diff.success(),
        "native write should leave no unstaged record diff"
    );
    assert!(
        !Command::new("git")
            .args(["diff", "--cached", "--quiet", "--", ".tandem"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success(),
        "failed checkpoint must leave Tandem staged for retry/inspection"
    );
    assert!(fs::read_to_string(root.join(".tandem/tasks/task-1.md"))
        .unwrap()
        .contains("in-progress"));
    // Recovery continues with the next real assignment boundary, not a
    // replay of the already-successful claim. Removing the failing hook is a
    // test-only availability repair; no claim is retried.
    fs::remove_file(hook).unwrap();
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "ready",
            "--evidence",
            "observed after hook repair",
        ],
    );
    assert!(ok, "delivery recovery failed: {stderr}");
    assert_eq!(
        json(&stdout)["data"]["checkpoint"]["status"],
        "checkpointed"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pre_commit_hook_can_read_tandem_without_hierarchy_lock_deadlock() {
    let root = setup("hook-read");
    let hook = root.join(".git/hooks/pre-commit");
    let binary = env!("CARGO_BIN_EXE_tandem");
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\ntimeout 3s '{}' show task-1 --json >/dev/null\nstatus=$?\necho hook-read-status=$status >&2\nexit $status\n",
            binary.replace('\'', "'\\''")
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).unwrap();
    }
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "hook read claim failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["recordWritten"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unpushed_ordinary_commits_absorb_tandem_and_pushed_commits_are_never_amended() {
    let root = setup("ordinary-absorbed");
    fs::write(root.join("ordinary.txt"), "ordinary\n").unwrap();
    git(&root, &["add", "ordinary.txt"]);
    git(&root, &["commit", "--quiet", "-m", "ordinary local commit"]);
    let ordinary_parent = git(&root, &["rev-parse", "HEAD^"]);
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], true);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "ordinary local commit"
    );
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), ordinary_parent);
    fs::remove_dir_all(&root).unwrap();

    let root = setup("pushed-preserved");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let pushed_head = git(&root, &["rev-parse", "HEAD"]);
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], false);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), pushed_head);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        pushed_head
    );
    // The next boundary may amend only the unpushed checkpoint above the
    // pushed HEAD; the pushed commit stays exactly HEAD^.
    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "ready",
            "--evidence",
            "observed",
        ],
    );
    assert!(ok, "deliver failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), pushed_head);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        pushed_head
    );
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}

#[test]
fn rolling_checkpoint_amends_on_a_dirty_unrelated_tree() {
    let root = setup("rolling-dirty");
    fs::write(root.join("unrelated-tracked.txt"), "base\n").unwrap();
    git(&root, &["add", "unrelated-tracked.txt"]);
    git(
        &root,
        &["commit", "--quiet", "-m", "unrelated tracked fixture"],
    );
    let parent = git(&root, &["rev-parse", "HEAD^"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);

    fs::write(root.join("unrelated-tracked.txt"), "dirty unstaged\n").unwrap();
    fs::write(root.join("staged-other.txt"), "staged\n").unwrap();
    git(&root, &["add", "staged-other.txt"]);
    let staged_before = git(&root, &["diff", "--cached", "--name-status"]);
    fs::write(root.join("untracked-other.txt"), "untracked\n").unwrap();

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    let claim = json(&stdout);
    assert_eq!(claim["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(claim["data"]["checkpoint"]["amended"], true);
    assert_eq!(claim["data"]["checkpoint"]["consolidated"], 0);
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), baseline);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), parent);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "unrelated tracked fixture"
    );

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "deliver",
            "task-1",
            "--summary",
            "ready",
            "--evidence",
            "observed",
        ],
    );
    assert!(ok, "deliver failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), parent);

    let (ok, stdout, stderr) = run(&root, &["--json", "complete", "task-1"]);
    assert!(ok, "complete failed: {stderr}");
    let complete = json(&stdout);
    assert_eq!(complete["data"]["checkpoint"]["amended"], true);
    assert_eq!(complete["data"]["checkpoint"]["consolidated"], 0);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), parent);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");

    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status"]),
        staged_before
    );
    assert_eq!(
        fs::read_to_string(root.join("unrelated-tracked.txt")).unwrap(),
        "dirty unstaged\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("untracked-other.txt")).unwrap(),
        "untracked\n"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn leftover_adjacent_chore_runs_collapse_on_a_safe_boundary() {
    let root = setup("reconcile-safe");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);

    chore_commit(&root, ".tandem/leftover-a.txt");
    chore_commit(&root, ".tandem/leftover-b.txt");
    fs::write(root.join("source.txt"), "source\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(
        &root,
        &["commit", "--quiet", "-m", "ordinary source commit"],
    );
    chore_commit(&root, ".tandem/leftover-c.txt");
    chore_commit(&root, ".tandem/leftover-d.txt");
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "6");

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], true);
    assert!(
        value["data"]["checkpoint"]["consolidated"]
            .as_u64()
            .unwrap()
            >= 2
    );

    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), baseline);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        baseline
    );
    let subjects = git(&root, &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<_>>(),
        vec!["ordinary source commit", "fixture baseline"]
    );
    let tree = git(&root, &["ls-tree", "-r", "--name-only", "HEAD"]);
    for path in [
        ".tandem/leftover-a.txt",
        ".tandem/leftover-b.txt",
        ".tandem/leftover-c.txt",
        ".tandem/leftover-d.txt",
    ] {
        assert!(tree.contains(path), "collapsed history lost {path}");
    }
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}

#[test]
fn leftover_chore_run_is_not_collapsed_when_rewrite_is_unsafe() {
    let root = setup("reconcile-unsafe");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);

    chore_commit(&root, ".tandem/leftover-a.txt");
    chore_commit(&root, ".tandem/leftover-b.txt");
    // An unrelated dirty byte makes the rewrite unsafe; the run stays. The
    // live amend still folds the new record into HEAD.
    fs::write(root.join("dirty-untracked.txt"), "dirty\n").unwrap();
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "3");

    let (ok, stdout, stderr) = run(
        &root,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "claim failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], true);
    assert_eq!(value["data"]["checkpoint"]["consolidated"], 0);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "3");
    assert_eq!(git(&root, &["rev-parse", "HEAD^^"]), baseline);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        baseline
    );

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}

#[test]
fn non_git_and_nested_workspaces_report_explicit_checkpoint_results() {
    let non_git = root("non-git");
    fs::create_dir_all(&non_git).unwrap();
    let (ok, _, stderr) = run(&non_git, &["init", "--title", "No Git"]);
    assert!(ok, "non-Git init failed: {stderr}");
    let (ok, _, stderr) = run(
        &non_git,
        &["add", "task", "No Git task", "--acceptance", "durable"],
    );
    assert!(ok, "non-Git add failed: {stderr}");
    let (ok, stdout, stderr) = run(
        &non_git,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "non-Git claim must preserve record success: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["recordWritten"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "failed");
    assert!(value["data"]["checkpoint"]["error"]
        .as_str()
        .unwrap()
        .contains("Git"));
    fs::remove_dir_all(&non_git).unwrap();

    let no_identity = setup("no-identity");
    git(&no_identity, &["config", "user.name", ""]);
    git(&no_identity, &["config", "user.email", ""]);
    let (ok, stdout, stderr) = run(
        &no_identity,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(
        ok,
        "identity failure must preserve record success: {stderr}"
    );
    let value = json(&stdout);
    assert_eq!(value["data"]["recordWritten"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "failed");
    assert!(value["data"]["checkpoint"]["error"]
        .as_str()
        .unwrap()
        .contains("identity"));
    fs::remove_dir_all(&no_identity).unwrap();

    let root = setup("nested");
    let nested = root.join("nested-workspace");
    fs::create_dir_all(&nested).unwrap();
    let (ok, _, stderr) = run(&nested, &["init", "--title", "Nested"]);
    assert!(ok, "nested init failed: {stderr}");
    let (ok, _, stderr) = run(
        &nested,
        &["add", "task", "Nested task", "--acceptance", "durable"],
    );
    assert!(ok, "nested add failed: {stderr}");
    git(&root, &["add", "."]);
    git(&root, &["commit", "--quiet", "-m", "nested fixture"]);
    let (ok, stdout, stderr) = run(
        &nested,
        &[
            "--json",
            "accord",
            "claim",
            "task-1",
            "--assignee",
            "worker",
        ],
    );
    assert!(ok, "nested claim failed: {stderr}");
    assert_eq!(
        json(&stdout)["data"]["checkpoint"]["status"],
        "checkpointed"
    );
    let committed = git(&root, &["show", "--format=", "--name-only", "HEAD"]);
    assert!(committed
        .lines()
        .any(|path| path.starts_with("nested-workspace/.tandem/")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_boundary_processes_share_git_repository_lock() {
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

    // These independent processes use separate worktrees but one Git common
    // directory. Both must return a complete JSON checkpoint result.
    let first = bin()
        .args(["--json", "accord", "claim", "task-1", "--assignee", "one"])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let second = bin()
        .args(["--json", "accord", "claim", "task-2", "--assignee", "two"])
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
    assert!(
        !first_output.stdout.is_empty(),
        "first stdout empty: {}",
        String::from_utf8_lossy(&first_output.stderr)
    );
    assert!(
        !second_output.stdout.is_empty(),
        "second stdout empty: {}",
        String::from_utf8_lossy(&second_output.stderr)
    );
    assert_eq!(
        json(&String::from_utf8_lossy(&first_output.stdout))["data"]["checkpoint"]["status"],
        "checkpointed"
    );
    assert_eq!(
        json(&String::from_utf8_lossy(&second_output.stdout))["data"]["checkpoint"]["status"],
        "checkpointed"
    );
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(git(&linked, &["rev-list", "--count", "HEAD"]), "2");
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

#[test]
fn explicit_checkpoint_flushes_pending_metadata_into_unpushed_source_commit() {
    let root = setup("explicit-absorb");

    // An ordinary unpushed source commit that does not include `.tandem`.
    fs::write(root.join("tracked-partial.txt"), "base partial\n").unwrap();
    fs::write(root.join("source.txt"), "source\n").unwrap();
    git(&root, &["add", "tracked-partial.txt", "source.txt"]);
    git(
        &root,
        &["commit", "--quiet", "-m", "ordinary source commit"],
    );
    let source_parent = git(&root, &["rev-parse", "HEAD^"]);

    // Pending owning `.tandem` metadata from ordinary intermediate commands.
    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "pending metadata"]);
    assert!(ok, "update failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &["add", "task", "Pending task", "--acceptance", "captured"],
    );
    assert!(ok, "add failed: {stderr}");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Pending milestone",
            "--parent",
            "task-1",
            "--acceptance",
            "milestone captured",
        ],
    );
    assert!(ok, "milestone add failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["rules", "add", "prefer", "Pending rule text"]);
    assert!(ok, "rule add failed: {stderr}");
    let rule_id = stdout
        .trim()
        .strip_prefix("Created rule ")
        .expect("rule add prints its composite id")
        .to_string();
    let rule_path = format!(".tandem/rules/{rule_id}.md");

    // Unrelated staged, unstaged, and untracked state must survive untouched.
    fs::write(root.join("tracked-partial.txt"), "staged partial\n").unwrap();
    git(&root, &["add", "tracked-partial.txt"]);
    fs::write(root.join("tracked-partial.txt"), "unstaged partial\n").unwrap();
    fs::write(root.join("untracked.txt"), "untracked\n").unwrap();

    let tasks_before = snapshot_tree(&root.join(".tandem/tasks"));
    let rules_before = snapshot_tree(&root.join(".tandem/rules"));
    let events_before = snapshot_tree(&root.join(".tandem/events"));
    let decisions_before = snapshot_tree(&root.join(".tandem/decisions"));
    let index_before = git(&root, &["ls-files", "--stage", "--", "tracked-partial.txt"]);
    let staged_before = git(&root, &["diff", "--cached", "--name-status", "--", "."]);
    let head_before = git(&root, &["rev-parse", "HEAD"]);
    assert!(!git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty());

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], true);

    let head_after = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(head_after, head_before);
    assert_eq!(value["data"]["checkpoint"]["commit"], head_after.as_str());
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), source_parent);
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "ordinary source commit"
    );
    assert!(git(&root, &["status", "--porcelain", "--", ".tandem"]).is_empty());

    let committed = git(&root, &["show", "--format=", "--name-only", "HEAD"]);
    for path in [
        ".tandem/tasks/task-1.md",
        ".tandem/tasks/task-2.md",
        ".tandem/tasks/task-1-1.md",
        rule_path.as_str(),
        ".tandem/events",
    ] {
        assert!(committed.contains(path), "checkpoint commit missing {path}");
    }

    // The pending Rule and Subtask bytes are committed exactly as written.
    for path in [rule_path.as_str(), ".tandem/tasks/task-1-1.md"] {
        let committed_bytes = git_raw(&root, &["show", &format!("HEAD:{path}")]);
        let worktree_bytes = fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(
            committed_bytes, worktree_bytes,
            "flush did not commit the pending bytes for {path}"
        );
    }
    let committed_subtask = git_raw(&root, &["show", "HEAD:.tandem/tasks/task-1-1.md"]);
    assert!(committed_subtask.contains("Pending milestone"));
    let committed_rule = git_raw(&root, &["show", &format!("HEAD:{rule_path}")]);
    assert!(committed_rule.contains("Pending rule text"));

    // The flush itself authors no Task, Rule, Decision, or event bytes.
    assert_eq!(snapshot_tree(&root.join(".tandem/tasks")), tasks_before);
    assert_eq!(snapshot_tree(&root.join(".tandem/rules")), rules_before);
    assert_eq!(snapshot_tree(&root.join(".tandem/events")), events_before);
    assert_eq!(
        snapshot_tree(&root.join(".tandem/decisions")),
        decisions_before
    );

    // Unrelated index, staged paths, and worktree bytes are unchanged.
    assert_eq!(
        git(&root, &["ls-files", "--stage", "--", "tracked-partial.txt"]),
        index_before
    );
    assert_eq!(
        git(&root, &["diff", "--cached", "--name-status", "--", "."]),
        staged_before
    );
    assert_eq!(
        fs::read_to_string(root.join("tracked-partial.txt")).unwrap(),
        "unstaged partial\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("untracked.txt")).unwrap(),
        "untracked\n"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_checkpoint_is_idempotent_and_keeps_one_rolling_commit() {
    // Local unpushed history: repeated flushes keep exactly one commit.
    let root = setup("explicit-idempotent");
    let baseline = git(&root, &["rev-parse", "HEAD"]);
    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "first"]);
    assert!(ok, "update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], true);
    let first = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(first, baseline);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "fixture baseline"
    );

    // Idempotent on a clean `.tandem`.
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "clean checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["checkpoint"]["status"], "clean");
    assert_eq!(value["data"]["checkpoint"]["commit"], Value::Null);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), first);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");

    // A second metadata mutation folds into the same rolling commit.
    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "second"]);
    assert!(ok, "second update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "second checkpoint failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "1");
    fs::remove_dir_all(root).unwrap();

    // Pushed history: the flush adds at most one rolling checkpoint commit.
    let root = setup("explicit-rolling");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let pushed = git(&root, &["rev-parse", "HEAD"]);
    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "pushed-first"]);
    assert!(ok, "update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], false);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    let rolling = git(&root, &["rev-parse", "HEAD"]);

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "pushed-second"]);
    assert!(ok, "second update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "second checkpoint failed: {stderr}");
    assert_eq!(json(&stdout)["data"]["checkpoint"]["amended"], true);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), rolling);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        pushed
    );
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}

#[test]
fn explicit_checkpoint_never_amends_a_merge_head() {
    let root = setup("explicit-merge");
    let base_branch = git(&root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    git(&root, &["checkout", "--quiet", "-b", "side"]);
    fs::write(root.join("side.txt"), "side\n").unwrap();
    git(&root, &["add", "side.txt"]);
    git(&root, &["commit", "--quiet", "-m", "side commit"]);
    git(&root, &["checkout", "--quiet", &base_branch]);
    fs::write(root.join("mainline.txt"), "main\n").unwrap();
    git(&root, &["add", "mainline.txt"]);
    git(&root, &["commit", "--quiet", "-m", "mainline commit"]);
    git(
        &root,
        &["merge", "--quiet", "--no-ff", "side", "-m", "merge side"],
    );
    let merge = git(&root, &["rev-parse", "HEAD"]);
    assert_eq!(
        git(&root, &["rev-list", "--parents", "-1", "HEAD"])
            .split_whitespace()
            .count(),
        3
    );
    let count_before: u32 = git(&root, &["rev-list", "--count", "HEAD"])
        .parse()
        .unwrap();

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "dirty after merge"]);
    assert!(ok, "update failed: {stderr}");
    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], false);
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), merge);
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), merge);
    let count_after: u32 = git(&root, &["rev-list", "--count", "HEAD"])
        .parse()
        .unwrap();
    assert_eq!(count_after, count_before + 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_checkpoint_fails_closed_with_structured_envelopes() {
    let non_git = root("explicit-non-git");
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
    assert_eq!(value["error"]["details"]["checkpoint"]["amended"], false);
    assert_eq!(value["error"]["details"]["checkpoint"]["consolidated"], 0);
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

    // A failing pre-commit hook keeps the pending change staged for inspection.
    let root = setup("explicit-hook-failure");
    let hook = root.join(".git/hooks/pre-commit");
    fs::write(
        &hook,
        "#!/bin/sh\necho intentional checkpoint hook failure >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).unwrap();
    }
    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "pending hook failure"],
    );
    assert!(ok, "update failed: {stderr}");
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
        !Command::new("git")
            .args(["diff", "--cached", "--quiet", "--", ".tandem"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success(),
        "failed checkpoint must leave .tandem staged for inspection"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_checkpoint_folds_leftover_own_chore_runs() {
    let root = setup("explicit-reconcile");
    let remote = root.with_extension("bare");
    git(&root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
    let baseline = git(&root, &["rev-parse", "HEAD"]);

    chore_commit(&root, ".tandem/leftover-a.txt");
    chore_commit(&root, ".tandem/leftover-b.txt");
    fs::write(root.join("source.txt"), "source\n").unwrap();
    git(&root, &["add", "source.txt"]);
    git(
        &root,
        &["commit", "--quiet", "-m", "ordinary source commit"],
    );
    chore_commit(&root, ".tandem/leftover-c.txt");
    chore_commit(&root, ".tandem/leftover-d.txt");
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "6");

    let (ok, _, stderr) = run(&root, &["update", "task-1", "--body", "pending reconcile"]);
    assert!(ok, "update failed: {stderr}");

    let (ok, stdout, stderr) = run(&root, &["--json", "checkpoint"]);
    assert!(ok, "checkpoint failed: {stderr}");
    let value = json(&stdout);
    assert_eq!(value["data"]["checkpoint"]["status"], "checkpointed");
    assert_eq!(value["data"]["checkpoint"]["amended"], true);
    assert!(
        value["data"]["checkpoint"]["consolidated"]
            .as_u64()
            .unwrap()
            >= 2,
        "expected leftover chore runs to fold: {value}"
    );

    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), baseline);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        baseline
    );
    let subjects = git(&root, &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<_>>(),
        vec!["ordinary source commit", "fixture baseline"]
    );
    let tree = git(&root, &["ls-tree", "-r", "--name-only", "HEAD"]);
    for path in [
        ".tandem/leftover-a.txt",
        ".tandem/leftover-b.txt",
        ".tandem/leftover-c.txt",
        ".tandem/leftover-d.txt",
    ] {
        assert!(tree.contains(path), "collapsed history lost {path}");
    }
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
}
