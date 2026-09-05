use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    serde_json::from_str(stdout.lines().last().unwrap()).unwrap()
}

#[test]
fn real_git_checkpoint_preserves_unrelated_index_worktree_and_untracked_state() {
    let root = setup("preservation");
    fs::write(root.join("unrelated.txt"), "staged version\n").unwrap();
    git(&root, &["add", "unrelated.txt"]);
    fs::write(root.join("unrelated.txt"), "unstaged version\n").unwrap();
    fs::write(root.join("untracked.txt"), "untracked bytes\n").unwrap();
    let index_before = git(&root, &["ls-files", "--stage", "--", "unrelated.txt"]);
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
        git(&root, &["ls-files", "--stage", "--", "unrelated.txt"]),
        index_before
    );
    assert_eq!(
        fs::read_to_string(root.join("unrelated.txt")).unwrap(),
        "unstaged version\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("untracked.txt")).unwrap(),
        "untracked bytes\n"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn boundaries_batch_progress_and_never_create_empty_or_amending_commits() {
    let root = setup("boundaries");
    let baseline = git(&root, &["rev-parse", "HEAD"]);
    let (ok, _, stderr) = run(
        &root,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    assert!(ok, "claim failed: {stderr}");
    let claim_head = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(claim_head, baseline);
    let (ok, _, stderr) = run(
        &root,
        &["update", "task-1", "--body", "intermediate progress"],
    );
    assert!(ok, "update failed: {stderr}");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), claim_head);
    let (ok, _, stderr) = run(
        &root,
        &[
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
    let delivery_head = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(delivery_head, claim_head);
    let (ok, _, stderr) = run(&root, &["complete", "task-1"]);
    assert!(ok, "complete failed: {stderr}");
    let final_head = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(final_head, delivery_head);
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "4");
    assert_eq!(
        git(&root, &["log", "-1", "--format=%s"]),
        "chore(tandem): checkpoint metadata"
    );
    assert_eq!(
        git(&root, &["show", "--format=%P", "--no-patch", "HEAD"]),
        delivery_head
    );

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
    let (ok, _, stderr) = run(
        &root,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    assert!(!ok, "retry must not repeat a successful claim: {stderr}");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pushed_and_unproven_ordinary_commits_are_never_amended() {
    let root = setup("ordinary-preserved");
    fs::write(root.join("ordinary.txt"), "ordinary\n").unwrap();
    git(&root, &["add", "ordinary.txt"]);
    git(&root, &["commit", "--quiet", "-m", "ordinary local commit"]);
    let ordinary_head = git(&root, &["rev-parse", "HEAD"]);
    let (ok, _, stderr) = run(
        &root,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    assert!(ok, "claim failed: {stderr}");
    assert_eq!(
        git(&root, &["rev-parse", "HEAD^"]),
        ordinary_head,
        "native checkpoint must append after an ordinary unproven commit"
    );
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
    let (ok, _, stderr) = run(
        &root,
        &["accord", "claim", "task-1", "--assignee", "worker"],
    );
    assert!(ok, "claim failed: {stderr}");
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), pushed_head);
    assert_eq!(
        git(&root, &["rev-parse", "refs/remotes/origin/main"]),
        pushed_head
    );
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(remote).unwrap();
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
    let mut first = bin()
        .args(["accord", "claim", "task-1", "--assignee", "one"])
        .current_dir(&root)
        .spawn()
        .unwrap();
    let mut second = bin()
        .args(["accord", "claim", "task-2", "--assignee", "two"])
        .current_dir(&root)
        .spawn()
        .unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "3");
    assert!(git(&root, &["fsck", "--no-progress", "--full"]).is_empty());

    // A linked worktree has a different `.git` file and branch but one common
    // Git directory. Both native calls must use that common lock.
    let linked = root.with_extension("linked");
    let (ok, _, stderr) = run(
        &root,
        &[
            "add",
            "task",
            "Third checkpoint task",
            "--acceptance",
            "boundary is durable",
        ],
    );
    assert!(ok, "third add failed: {stderr}");
    git(&root, &["add", ".tandem"]);
    git(&root, &["commit", "--quiet", "-m", "fixture third task"]);
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
    let mut first = bin()
        .args(["accord", "block", "task-1", "--note", "linked boundary"])
        .current_dir(&root)
        .spawn()
        .unwrap();
    let mut second = bin()
        .args(["accord", "claim", "task-3", "--assignee", "linked-two"])
        .current_dir(&linked)
        .spawn()
        .unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), "5");
    assert_eq!(git(&linked, &["rev-list", "--count", "HEAD"]), "5");
    git(
        &root,
        &["worktree", "remove", "--force", linked.to_str().unwrap()],
    );
    fs::remove_dir_all(root).unwrap();
    if linked.exists() {
        fs::remove_dir_all(linked).unwrap();
    }
}
