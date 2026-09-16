//! Native Git checkpoints for durable `.tandem` state.
//!
//! Checkpoints are deliberately narrow: only the owning `.tandem` path is
//! staged and committed. Git's index therefore retains unrelated staged
//! entries, while unrelated worktree and untracked bytes are never touched.
//! A lock in the repository's common Git directory serializes linked
//! worktrees as well as ordinary processes.
//!
//! Tandem is the only Git writer. Unpushed history must not accumulate
//! adjacent Tandem-only commits, so an assignment boundary amends its own
//! unpushed checkpoint HEAD, and the same boundary reconciles any remaining
//! adjacent own-checkpoint run when a rewrite is provably safe. Pushed commits
//! and ordinary/unproven work commits are never rewritten, and Tandem never
//! folds metadata into a neighboring source commit.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::{display_path, TandemProject};

const CHECKPOINT_SUBJECT: &str = "chore(tandem): checkpoint metadata";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointOutcome {
    pub(crate) status: CheckpointStatus,
    pub(crate) commit: Option<String>,
    /// True only when this boundary amended its own unpushed tandem-only HEAD.
    pub(crate) amended: bool,
    /// Number of adjacent own-checkpoint runs collapsed by this boundary's
    /// best-effort reconcile. Always `0` on the batched and failed paths.
    pub(crate) consolidated: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckpointStatus {
    /// The mutation belongs to a Subtask milestone or Epic grouping. Its
    /// tracked write is durable, but the next assignment boundary owns the
    /// checkpoint.
    Batched,
    /// No `.tandem` changes were present after staging, so no commit was made.
    Clean,
    /// A checkpoint commit was created or amended.
    Checkpointed,
    /// The record/event write already succeeded, but Git checkpointing failed.
    Failed { message: String },
}

impl CheckpointOutcome {
    pub(crate) fn batched() -> Self {
        Self {
            status: CheckpointStatus::Batched,
            commit: None,
            amended: false,
            consolidated: 0,
        }
    }

    pub(crate) fn clean(consolidated: u32) -> Self {
        Self {
            status: CheckpointStatus::Clean,
            commit: None,
            amended: false,
            consolidated,
        }
    }

    pub(crate) fn failed(message: impl Into<String>) -> Self {
        Self {
            status: CheckpointStatus::Failed {
                message: message.into(),
            },
            commit: None,
            amended: false,
            consolidated: 0,
        }
    }
}

struct RepositoryLock {
    file: File,
}

impl RepositoryLock {
    fn acquire(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| {
                format!("could not open checkpoint lock {}: {error}", path.display())
            })?;
        file.lock()
            .map_err(|error| format!("could not lock checkpoint repository: {error}"))?;
        Ok(Self { file })
    }
}

impl Drop for RepositoryLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[derive(Debug)]
struct GitOutput {
    stdout: String,
    _stderr: String,
}

/// Commit changed tracked Tandem state at one explicit work boundary.
///
/// This method is intentionally best-effort after the native record write:
/// failure is returned as a data-bearing outcome instead of an application
/// error, so callers can show that the lifecycle mutation succeeded and only
/// its Git checkpoint needs attention. There is no fallback or force path.
pub(crate) fn checkpoint(project: &TandemProject) -> CheckpointOutcome {
    match checkpoint_inner(project) {
        Ok(outcome) => outcome,
        Err(message) => CheckpointOutcome::failed(message),
    }
}

fn checkpoint_inner(project: &TandemProject) -> Result<CheckpointOutcome, String> {
    let repo_root = git_output(project.root(), &["rev-parse", "--show-toplevel"])?
        .stdout
        .trim()
        .to_string();
    if repo_root.is_empty() {
        return Err("Git returned an empty repository root".to_string());
    }
    let repo_root = PathBuf::from(repo_root);
    let common_dir = git_output(
        &repo_root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?
    .stdout
    .trim()
    .to_string();
    if common_dir.is_empty() {
        return Err("Git returned an empty common directory".to_string());
    }
    let common_dir = PathBuf::from(common_dir);
    let lock_path = common_dir.join("tandem-checkpoint.lock");
    let _lock = RepositoryLock::acquire(&lock_path)?;

    let data_dir = project.data_dir();
    let relative_data = data_dir
        .strip_prefix(&repo_root)
        .map_err(|_| {
            format!(
                "Tandem workspace {} is outside Git repository {}; refusing checkpoint",
                display_path(data_dir),
                repo_root.display()
            )
        })?
        .to_str()
        .ok_or_else(|| "Tandem workspace path is not valid UTF-8".to_string())?;
    if relative_data.is_empty() || relative_data == "." {
        return Err("refusing to checkpoint an empty Git pathspec".to_string());
    }

    // `-A -- .tandem` is the only mutating Git preparation operation. It does
    // not reset, stash, clean, or otherwise rewrite unrelated index entries.
    git_output(&repo_root, &["add", "-A", "--", relative_data])?;
    let has_staged = staged_tandem_changes(&repo_root, relative_data)?;

    // Live maintenance is primary: fold this boundary's `.tandem` write into
    // Tandem's own unpushed checkpoint HEAD, or append an ordinary commit.
    let mut amended = false;
    if has_staged {
        if should_amend_head(&repo_root, relative_data)? {
            git_output(
                &repo_root,
                &[
                    "commit",
                    "--amend",
                    "-m",
                    CHECKPOINT_SUBJECT,
                    "--",
                    relative_data,
                ],
            )?;
            amended = true;
        } else {
            git_output(
                &repo_root,
                &["commit", "-m", CHECKPOINT_SUBJECT, "--", relative_data],
            )?;
        }
    }

    // Reconcile is a best-effort backup for leftover runs from old binaries,
    // failed amends, or pre-change history. It never changes the primary
    // result: unsafe or failed rewrites are skipped and reported as zero.
    let consolidated = reconcile(&repo_root, relative_data);

    if !has_staged {
        return Ok(CheckpointOutcome::clean(consolidated));
    }
    let sha = git_output(&repo_root, &["rev-parse", "HEAD"])?
        .stdout
        .trim()
        .to_string();
    if sha.is_empty() {
        return Err("Git checkpoint reported success without a HEAD commit".to_string());
    }
    Ok(CheckpointOutcome {
        status: CheckpointStatus::Checkpointed,
        commit: Some(sha),
        amended,
        consolidated,
    })
}

fn staged_tandem_changes(repo_root: &Path, relative_data: &str) -> Result<bool, String> {
    let staged = Command::new("git")
        .args(["diff", "--cached", "--quiet", "--", relative_data])
        .current_dir(repo_root)
        .status()
        .map_err(|error| format!("could not inspect staged Tandem state: {error}"))?;
    match staged.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(format!(
            "Git diff --cached could not inspect Tandem state (status {staged})"
        )),
    }
}

fn should_amend_head(repo_root: &Path, relative_data: &str) -> Result<bool, String> {
    if !rev_exists(repo_root, "HEAD")? {
        return Ok(false);
    }
    if head_is_pushed(repo_root)? {
        return Ok(false);
    }
    is_own_checkpoint(repo_root, "HEAD", relative_data)
}

/// A commit is pushed when any remote-tracking ref already contains it. A
/// repository with no remote-tracking refs has nothing pushed, so its own
/// checkpoint HEAD is amendable.
fn head_is_pushed(repo_root: &Path) -> Result<bool, String> {
    let output = git_output(
        repo_root,
        &[
            "for-each-ref",
            "--contains=HEAD",
            "--format=%(refname)",
            "refs/remotes",
        ],
    )?;
    Ok(!output.stdout.trim().is_empty())
}

/// Tandem's own commit is identifiable by the fixed checkpoint subject; a
/// user-authored `.tandem`-only commit is unproven work and is never rewritten.
fn is_own_checkpoint(repo_root: &Path, rev: &str, relative_data: &str) -> Result<bool, String> {
    if commit_subject(repo_root, rev)? != CHECKPOINT_SUBJECT {
        return Ok(false);
    }
    is_tandem_only(repo_root, rev, relative_data)
}

fn commit_subject(repo_root: &Path, rev: &str) -> Result<String, String> {
    Ok(git_output(repo_root, &["log", "-1", "--format=%s", rev])?
        .stdout
        .trim()
        .to_string())
}

/// True when the commit changed at least one path and every changed path is
/// under the owning Tandem directory. Merge commits report no changed paths and
/// are therefore never treated as Tandem-only.
fn is_tandem_only(repo_root: &Path, rev: &str, relative_data: &str) -> Result<bool, String> {
    let output = git_output(
        repo_root,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--root",
            rev,
        ],
    )?;
    let mut changed = false;
    for path in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        changed = true;
        if !is_within(path, relative_data) {
            return Ok(false);
        }
    }
    Ok(changed)
}

fn is_within(path: &str, directory: &str) -> bool {
    path == directory
        || path
            .strip_prefix(directory)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn rev_exists(repo_root: &Path, rev: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", rev])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("could not resolve {rev}: {error}"))?;
    Ok(output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

/// Collapse every remaining adjacent own-checkpoint run in the unpushed range.
///
/// The result is a count of collapsed runs. Any unsafe condition or failed
/// rewrite is skipped, so this never fails the already-successful record write
/// or live commit.
fn reconcile(repo_root: &Path, relative_data: &str) -> u32 {
    match reconcile_inner(repo_root, relative_data) {
        Ok(consolidated) => consolidated,
        Err(_) => {
            abort_rebase(repo_root);
            0
        }
    }
}

fn reconcile_inner(repo_root: &Path, relative_data: &str) -> Result<u32, String> {
    let Some(upstream) = upstream_commit(repo_root)? else {
        return Ok(0);
    };
    if !is_ancestor(repo_root, &upstream, "HEAD")? {
        return Ok(0);
    }
    if !tree_is_clean(repo_root)? {
        return Ok(0);
    }
    if operation_in_progress(repo_root)? {
        return Ok(0);
    }

    let total = git_output(repo_root, &["rev-list", "--count", "HEAD"])?
        .stdout
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let mut consolidated = 0u32;
    // Each collapse removes at least one commit, so this is bounded by the
    // commit count; the guard only protects against an unexpected no-op.
    while consolidated < total {
        let Some((first, last)) = first_adjacent_own_run(repo_root, &upstream, relative_data)? else {
            break;
        };
        collapse_run(repo_root, &first, &last)?;
        consolidated += 1;
    }
    Ok(consolidated)
}

fn first_adjacent_own_run(
    repo_root: &Path,
    upstream: &str,
    relative_data: &str,
) -> Result<Option<(String, String)>, String> {
    let output = git_output(
        repo_root,
        &[
            "log",
            "--reverse",
            "--format=%H%x09%s",
            &format!("{upstream}..HEAD"),
        ],
    )?;
    let mut run: Vec<String> = Vec::new();
    let mut first_run: Option<(String, String)> = None;
    for line in output.stdout.lines() {
        let (sha, subject) = line.split_once('\t').unwrap_or((line, ""));
        let own = subject == CHECKPOINT_SUBJECT && is_tandem_only(repo_root, sha, relative_data)?;
        if own {
            run.push(sha.to_string());
            continue;
        }
        if run.len() >= 2 && first_run.is_none() {
            first_run = Some((run[0].clone(), run[run.len() - 1].clone()));
        }
        run.clear();
    }
    if run.len() >= 2 && first_run.is_none() {
        first_run = Some((run[0].clone(), run[run.len() - 1].clone()));
    }
    Ok(first_run)
}

fn collapse_run(repo_root: &Path, first: &str, last: &str) -> Result<(), String> {
    let base = git_output(repo_root, &["rev-parse", &format!("{first}^")])?
        .stdout
        .trim()
        .to_string();
    let tree = git_output(repo_root, &["rev-parse", &format!("{last}^{{tree}}")])?
        .stdout
        .trim()
        .to_string();
    if base.is_empty() || tree.is_empty() {
        return Err("could not resolve collapse range".to_string());
    }
    let replacement = git_output(
        repo_root,
        &[
            "commit-tree",
            &tree,
            "-p",
            &base,
            "-m",
            CHECKPOINT_SUBJECT,
        ],
    )?
    .stdout
    .trim()
    .to_string();
    if replacement.is_empty() {
        return Err("git commit-tree returned no commit".to_string());
    }
    match git_output(repo_root, &["rebase", "--onto", &replacement, last]) {
        Ok(_) => Ok(()),
        Err(error) => {
            abort_rebase(repo_root);
            Err(error)
        }
    }
}

fn upstream_commit(repo_root: &Path) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", "@{upstream}"])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("could not resolve @{{upstream}}: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!sha.is_empty()).then_some(sha))
}

fn is_ancestor(repo_root: &Path, ancestor: &str, descendant: &str) -> Result<bool, String> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(repo_root)
        .status()
        .map_err(|error| format!("could not inspect ancestry: {error}"))?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!(
            "Git merge-base --is-ancestor failed with status {status}"
        )),
    }
}

fn tree_is_clean(repo_root: &Path) -> Result<bool, String> {
    let output = git_output(repo_root, &["status", "--porcelain", "--untracked-files=all"])?;
    Ok(output.stdout.trim().is_empty())
}

fn operation_in_progress(repo_root: &Path) -> Result<bool, String> {
    for name in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
    ] {
        let output = git_output(
            repo_root,
            &["rev-parse", "--path-format=absolute", "--git-path", name],
        )?;
        let path = output.stdout.trim();
        if !path.is_empty() && Path::new(path).exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn abort_rebase(repo_root: &Path) {
    let _ = Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(repo_root)
        .output();
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<GitOutput, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("could not execute Git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format_git_error(args, &output));
    }
    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        _stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn format_git_error(args: &[&str], output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    if detail.is_empty() {
        format!(
            "Git {} failed with status {}",
            args.join(" "),
            output.status
        )
    } else {
        format!("Git {} failed: {detail}", args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tandem-checkpoint-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {:?}", output.stderr);
    }

    #[test]
    fn checkpoint_outcome_defaults_are_explicitly_non_amending() {
        assert_eq!(CHECKPOINT_SUBJECT, "chore(tandem): checkpoint metadata");
        let clean = CheckpointOutcome::clean(0);
        assert_eq!(clean.status, CheckpointStatus::Clean);
        assert!(!clean.amended);
        assert_eq!(clean.consolidated, 0);
    }

    #[test]
    fn clean_boundary_does_not_create_an_empty_commit() {
        let root = temp_root("clean");
        fs::create_dir_all(&root).unwrap();
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.3.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        git(&root, &["init", "--quiet"]);
        git(&root, &["config", "user.name", "Tandem Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        git(&root, &["add", ".tandem"]);
        git(&root, &["commit", "--quiet", "-m", "baseline"]);
        let before = git_output(&root, &["rev-list", "--count", "HEAD"])
            .unwrap()
            .stdout
            .trim()
            .to_string();
        let outcome = checkpoint(&project);
        assert_eq!(outcome.status, CheckpointStatus::Clean);
        assert!(!outcome.amended);
        assert_eq!(outcome.consolidated, 0);
        assert_eq!(
            git_output(&root, &["rev-list", "--count", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            before
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn clean_boundary_reconciles_leftover_runs_without_a_new_diff() {
        let root = temp_root("clean-reconcile");
        fs::create_dir_all(&root).unwrap();
        let project = TandemProject::initialize(
            &root,
            "---\nprotocolVersion: 0.3.0\nstates: [todo, in-progress, validation]\n---\n",
        )
        .unwrap();
        git(&root, &["init", "--quiet"]);
        git(&root, &["config", "user.name", "Tandem Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        git(&root, &["add", ".tandem"]);
        git(&root, &["commit", "--quiet", "-m", "baseline"]);
        let remote = root.with_extension("bare");
        git(&root, &["init", "--bare", remote.to_str().unwrap()]);
        git(&root, &["remote", "add", "origin", remote.to_str().unwrap()]);
        git(&root, &["push", "--quiet", "-u", "origin", "HEAD:main"]);
        for name in ["a", "b"] {
            fs::write(
                root.join(format!(".tandem/leftover-{name}.txt")),
                "leftover\n",
            )
            .unwrap();
            git(&root, &["add", ".tandem"]);
            git(
                &root,
                &["commit", "--quiet", "-m", CHECKPOINT_SUBJECT],
            );
        }
        assert_eq!(
            git_output(&root, &["rev-list", "--count", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            "3"
        );

        let outcome = checkpoint(&project);
        assert_eq!(outcome.status, CheckpointStatus::Clean);
        assert!(!outcome.amended);
        assert_eq!(outcome.consolidated, 1);
        assert_eq!(
            git_output(&root, &["rev-list", "--count", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            "2"
        );
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(remote).unwrap();
    }
}
