//! Native Git checkpoints for durable `.tandem` state.
//!
//! A checkpoint is a single forward-only flush. It stages only the owning
//! `.tandem` path and, when that path has staged changes, records them in one
//! ordinary commit. It never amends, rebases, folds, or otherwise rewrites an
//! existing commit, so source commit identities are stable across a flush.
//! Unrelated index entries, worktree bytes, and untracked files are never
//! touched. A lock in the repository's common Git directory serializes linked
//! worktrees as well as ordinary processes.
//!
//! Native record and event writes persist immediately and never call this
//! module; a flush happens only when a host workflow explicitly runs
//! `tandem checkpoint` at a commit/push boundary.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::{display_path, TandemProject};

const CHECKPOINT_SUBJECT: &str = "chore(tandem): checkpoint metadata";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointOutcome {
    pub(crate) status: CheckpointStatus,
    pub(crate) commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckpointStatus {
    /// The lifecycle write is durable in the worktree but no flush has run.
    /// Its metadata is pending for the next explicit checkpoint.
    Batched,
    /// No `.tandem` changes were present after staging, so no commit was made.
    Clean,
    /// One checkpoint commit was created.
    Checkpointed,
    /// The flush failed; the pending `.tandem` change stays staged.
    Failed { message: String },
}

impl CheckpointOutcome {
    pub(crate) fn batched() -> Self {
        Self {
            status: CheckpointStatus::Batched,
            commit: None,
        }
    }

    pub(crate) fn clean() -> Self {
        Self {
            status: CheckpointStatus::Clean,
            commit: None,
        }
    }

    pub(crate) fn failed(message: impl Into<String>) -> Self {
        Self {
            status: CheckpointStatus::Failed {
                message: message.into(),
            },
            commit: None,
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

/// Flush pending tracked `.tandem` state into one forward-only commit.
///
/// Failure is returned as a data-bearing outcome instead of an application
/// error so lifecycle callers can report it without replaying a successful
/// record write. The explicit `tandem checkpoint` command converts a failure
/// into a fail-closed process exit.
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
    if !staged_tandem_changes(&repo_root, relative_data)? {
        return Ok(CheckpointOutcome::clean());
    }

    // A new commit limited to the owning path: existing history is never
    // rewritten and unrelated staged entries stay in the index.
    git_output(
        &repo_root,
        &["commit", "-m", CHECKPOINT_SUBJECT, "--", relative_data],
    )?;

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
    fn forward_only_flush_creates_one_commit_and_is_idempotent() {
        let root = temp_root("forward-only");
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
        let baseline = git_output(&root, &["rev-parse", "HEAD"])
            .unwrap()
            .stdout
            .trim()
            .to_string();

        // A clean flush is a no-op.
        let outcome = checkpoint(&project);
        assert_eq!(outcome.status, CheckpointStatus::Clean);
        assert_eq!(outcome.commit, None);

        fs::write(root.join(".tandem/leftover.txt"), "pending\n").unwrap();
        let outcome = checkpoint(&project);
        assert_eq!(outcome.status, CheckpointStatus::Checkpointed);
        let commit = outcome.commit.clone().unwrap();
        assert_eq!(
            git_output(&root, &["rev-parse", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            commit
        );
        assert_eq!(
            git_output(&root, &["rev-parse", "HEAD^"])
                .unwrap()
                .stdout
                .trim(),
            baseline
        );
        assert_eq!(
            git_output(&root, &["log", "-1", "--format=%s"])
                .unwrap()
                .stdout
                .trim(),
            CHECKPOINT_SUBJECT
        );

        // A repeated flush on a clean `.tandem` adds nothing.
        let outcome = checkpoint(&project);
        assert_eq!(outcome.status, CheckpointStatus::Clean);
        assert_eq!(
            git_output(&root, &["rev-parse", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            commit
        );

        fs::remove_dir_all(root).unwrap();
    }
}
