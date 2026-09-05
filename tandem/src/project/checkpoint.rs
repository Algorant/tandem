//! Native Git checkpoints for durable `.tandem` state.
//!
//! Checkpoints are deliberately narrow: only the owning `.tandem` path is
//! staged and committed. Git's index therefore retains unrelated staged
//! entries, while unrelated worktree and untracked bytes are never touched.
//! A lock in the repository's common Git directory serializes linked
//! worktrees as well as ordinary processes.

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
    /// The mutation belongs to a Subtask milestone or Epic grouping. Its
    /// tracked write is durable, but the next assignment boundary owns the
    /// checkpoint.
    Batched,
    /// No `.tandem` changes were present after staging, so no commit was made.
    Clean,
    /// A new ordinary commit was created. Native checkpoints never amend.
    Checkpointed,
    /// The record/event write already succeeded, but Git checkpointing failed.
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
    let staged = Command::new("git")
        .args(["diff", "--cached", "--quiet", "--", relative_data])
        .current_dir(&repo_root)
        .status()
        .map_err(|error| format!("could not inspect staged Tandem state: {error}"))?;
    match staged.code() {
        Some(0) => return Ok(CheckpointOutcome::clean()),
        Some(1) => {}
        _ => {
            return Err(format!(
                "Git diff --cached could not inspect Tandem state (status {})",
                staged
            ));
        }
    }

    let commit = git_output(
        &repo_root,
        &["commit", "-m", CHECKPOINT_SUBJECT, "--", relative_data],
    )?;
    let sha = git_output(&repo_root, &["rev-parse", "HEAD"])?
        .stdout
        .trim()
        .to_string();
    if sha.is_empty() {
        return Err(format!(
            "Git checkpoint reported success without a HEAD commit: {}",
            commit.stdout.trim()
        ));
    }
    Ok(CheckpointOutcome {
        status: CheckpointStatus::Checkpointed,
        commit: Some(sha),
    })
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
    fn checkpoint_status_is_explicitly_non_amending() {
        assert_eq!(CHECKPOINT_SUBJECT, "chore(tandem): checkpoint metadata");
        assert_eq!(CheckpointOutcome::clean().status, CheckpointStatus::Clean);
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
        assert_eq!(
            git_output(&root, &["rev-list", "--count", "HEAD"])
                .unwrap()
                .stdout
                .trim(),
            before
        );
        fs::remove_dir_all(root).unwrap();
    }
}
