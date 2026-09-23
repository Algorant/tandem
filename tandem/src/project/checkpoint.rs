//! Native Git checkpoints for durable `.tandem` state.
//!
//! Plain checkpoint is a single forward-only flush. It stages only the owning
//! `.tandem` path and, when that path has staged changes, records them in one
//! ordinary commit. Only the explicit push-boundary consolidation mode rewrites
//! eligible unpushed history; plain flush leaves source commit IDs stable.
//! Unrelated index entries, worktree bytes, and untracked files are never
//! touched. A lock in the repository's common Git directory serializes linked
//! worktrees as well as ordinary processes.
//!
//! Native record and event writes persist immediately and never call this
//! module; a flush happens only when a host workflow explicitly runs
//! `tandem checkpoint` at a commit boundary or `--consolidate` before push.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    checkpoint_inner_locked(project, &repo_root)
}

fn checkpoint_inner_locked(
    project: &TandemProject,
    repo_root: &Path,
) -> Result<CheckpointOutcome, String> {
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

/// The explicit push-boundary rewrite result. Ordinary checkpoints never use this path.
pub(crate) struct Consolidation {
    pub(crate) old_head: String,
    pub(crate) new_head: String,
    pub(crate) collapsed: usize,
}

/// Collapse only unpushed fixed-subject metadata commits. The ref is moved
/// only after all preconditions and the final tree have been verified.
pub(crate) fn consolidate_checkpoint(project: &TandemProject) -> Result<Consolidation, String> {
    let repo = PathBuf::from(
        git_output(project.root(), &["rev-parse", "--show-toplevel"])?
            .stdout
            .trim(),
    );
    let common = PathBuf::from(
        git_output(
            &repo,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?
        .stdout
        .trim(),
    );
    let _lock = RepositoryLock::acquire(&common.join("tandem-checkpoint.lock"))?;
    no_git_operation(&repo)?;
    // This flush is still forward-only. A later refusal never rewrites it.
    if let CheckpointStatus::Failed { message } = checkpoint_inner_locked(project, &repo)?.status {
        return Err(message);
    }
    let relative = project
        .data_dir()
        .strip_prefix(&repo)
        .map_err(|_| "Tandem workspace is outside the Git repository".to_string())?
        .to_str()
        .ok_or("Tandem workspace path is not UTF-8")?;
    let old = git_output(&repo, &["rev-parse", "HEAD"])?
        .stdout
        .trim()
        .to_string();
    let upstream = git_output(&repo, &["rev-parse", "@{upstream}"])
        .map_err(|_| "checkpoint consolidation requires an upstream".to_string())?
        .stdout
        .trim()
        .to_string();
    let branch = git_output(&repo, &["symbolic-ref", "-q", "HEAD"])
        .map_err(|_| "checkpoint consolidation requires a checked-out branch".to_string())?
        .stdout
        .trim()
        .to_string();
    if !git_success(&repo, &["merge-base", "--is-ancestor", &upstream, &old])? {
        return Err("upstream is not an ancestor of HEAD; refusing consolidation".into());
    }
    no_git_operation(&repo)?;
    if !git_output(&repo, &["status", "--porcelain"])?
        .stdout
        .is_empty()
    {
        return Err(
            "consolidation requires a clean index and worktree after flushing metadata".into(),
        );
    }
    let range = git_output(
        &repo,
        &["rev-list", "--reverse", &format!("{upstream}..{old}")],
    )?
    .stdout;
    let commits: Vec<&str> = range.lines().collect();
    let mut checkpoints = 0;
    for sha in &commits {
        let parents = git_output(&repo, &["rev-list", "--parents", "-n", "1", sha])?.stdout;
        let parts: Vec<&str> = parents.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(format!(
                "merge or root commit {sha} in unpushed range; refusing consolidation"
            ));
        }
        let changes = git_output(
            &repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                parts[1],
                sha,
            ],
        )?
        .stdout;
        let paths: Vec<&str> = changes.split('\0').filter(|p| !p.is_empty()).collect();
        let only_metadata = !paths.is_empty()
            && paths
                .iter()
                .all(|p| *p == relative || p.starts_with(&format!("{relative}/")));
        let touches_metadata = paths
            .iter()
            .any(|p| *p == relative || p.starts_with(&format!("{relative}/")));
        let subject = git_output(&repo, &["log", "-1", "--format=%s", sha])?.stdout;
        if subject.trim_end() == CHECKPOINT_SUBJECT && only_metadata {
            checkpoints += 1;
        } else if touches_metadata || subject.trim_end() == CHECKPOINT_SUBJECT {
            return Err(format!("commit {sha} touches metadata outside an eligible checkpoint; refusing consolidation"));
        }
        let raw = git_output(&repo, &["cat-file", "-p", sha])?.stdout;
        let headers = raw.split_once("\n\n").ok_or("invalid Git commit")?.0;
        if headers.lines().any(|line| {
            !["tree ", "parent ", "author ", "committer "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        }) {
            return Err(format!(
                "commit {sha} has extra headers that cannot be replayed safely"
            ));
        }
    }
    let rewritten: std::collections::HashSet<&str> = commits.iter().copied().collect();
    let refs = git_output(
        &repo,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads",
        ],
    )?
    .stdout;
    for line in refs.lines() {
        let Some((name, tip)) = line.split_once(' ') else {
            continue;
        };
        if name != branch {
            refuse_shared_base(&repo, &old, tip, &rewritten)?;
        }
    }
    // An upstream need not be the only published ref. Reject any other remote
    // ref pointing into the range rather than rewriting a published commit.
    let remotes = git_output(
        &repo,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/remotes",
        ],
    )?
    .stdout;
    for line in remotes.lines() {
        if let Some((_, tip)) = line.split_once(' ') {
            refuse_shared_base(&repo, &old, tip, &rewritten)?;
        }
    }
    let tags = git_output(
        &repo,
        &[
            "for-each-ref",
            "--format=%(*objectname) %(objectname)",
            "refs/tags",
        ],
    )?
    .stdout;
    for line in tags.lines() {
        let Some((peeled, direct)) = line.split_once(' ') else {
            continue;
        };
        let tip = if peeled.is_empty() { direct } else { peeled };
        // Tags on non-commit objects have no merge-base; only commit tags
        // participate in ancestry. Never rewrite a tagged commit.
        if git_output(&repo, &["cat-file", "-t", tip])?.stdout.trim() == "commit" {
            refuse_shared_base(&repo, &old, tip, &rewritten)?;
        }
    }
    let worktrees = git_output(&repo, &["worktree", "list", "--porcelain"])?.stdout;
    let current = fs::canonicalize(&repo).map_err(|e| e.to_string())?;
    let mut path = None;
    for line in worktrees.lines().chain(std::iter::once("")) {
        if let Some(value) = line.strip_prefix("worktree ") {
            path = Some(value.to_string());
        }
        if let Some(tip) = line.strip_prefix("HEAD ") {
            if path
                .as_ref()
                .is_some_and(|p| fs::canonicalize(p).ok().as_ref() != Some(&current))
            {
                refuse_shared_base(&repo, &old, tip, &rewritten)?;
            }
        }
        if line.is_empty() {
            path = None;
        }
    }
    if checkpoints == 0 {
        return Ok(Consolidation {
            old_head: old.clone(),
            new_head: old,
            collapsed: 0,
        });
    }

    let index = common.join(format!(
        "tandem-consolidate-{}-{}.index",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ));
    let _temporary = TemporaryIndex(index.clone());
    let mut parent = upstream.clone();
    for sha in &commits {
        let subject = git_output(&repo, &["log", "-1", "--format=%s", sha])?.stdout;
        if subject.trim_end() == CHECKPOINT_SUBJECT {
            continue;
        }
        // Construct each real tree from its original snapshot with the
        // upstream metadata subtree, using a private index only.
        git_index(&repo, &index, &["read-tree", sha])?;
        git_index(
            &repo,
            &index,
            &[
                "restore",
                "--staged",
                &format!("--source={upstream}"),
                "--",
                relative,
            ],
        )?;
        let tree = git_index(&repo, &index, &["write-tree"])?
            .stdout
            .trim()
            .to_string();
        parent = replay_commit(&repo, sha, &tree, &parent)?;
    }
    let original_tree = git_output(&repo, &["rev-parse", &format!("{old}^{{tree}}")])?
        .stdout
        .trim()
        .to_string();
    let identity = git_output(&repo, &["var", "GIT_COMMITTER_IDENT"])?.stdout;
    let (name, email, date) = parse_identity(identity.trim())?;
    let new_head = commit_tree(
        &repo,
        &original_tree,
        &parent,
        &format!("{CHECKPOINT_SUBJECT}\n"),
        (&name, &email, &date),
        (&name, &email, &date),
    )?;
    let new_tree = git_output(&repo, &["rev-parse", &format!("{new_head}^{{tree}}")])?
        .stdout
        .trim()
        .to_string();
    if new_tree != original_tree {
        return Err("consolidation tree mismatch; branch was not moved".into());
    }
    // Revalidate mutable safety inputs immediately before the atomic ref move.
    if !git_output(&repo, &["status", "--porcelain"])?
        .stdout
        .is_empty()
    {
        return Err("worktree changed during consolidation".into());
    }
    if git_output(&repo, &["rev-parse", "@{upstream}"])?
        .stdout
        .trim()
        != upstream
    {
        return Err("upstream moved during consolidation".into());
    }
    no_git_operation(&repo)?;
    git_output(
        &repo,
        &[
            "update-ref",
            "-m",
            "tandem checkpoint --consolidate",
            &branch,
            &new_head,
            &old,
        ],
    )?;
    Ok(Consolidation {
        old_head: old,
        new_head,
        collapsed: checkpoints,
    })
}

struct TemporaryIndex(PathBuf);
impl Drop for TemporaryIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn git_index(cwd: &Path, index: &Path, args: &[&str]) -> Result<GitOutput, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_INDEX_FILE", index)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format_git_error(args, &output));
    }
    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        _stderr: String::new(),
    })
}

fn git_success(cwd: &Path, args: &[&str]) -> Result<bool, String> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| e.to_string())?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!("git {} failed: {status}", args.join(" "))),
    }
}

fn refuse_shared_base(
    repo: &Path,
    head: &str,
    tip: &str,
    range: &std::collections::HashSet<&str>,
) -> Result<(), String> {
    let result = Command::new("git")
        .args(["merge-base", head, tip])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    if result.status.code() == Some(1) {
        return Ok(());
    }
    if !result.status.success() {
        return Err(format_git_error(&["merge-base", head, tip], &result));
    }
    if range.contains(String::from_utf8_lossy(&result.stdout).trim()) {
        return Err(format!(
            "another branch or linked worktree is based inside the rewrite range ({tip})"
        ));
    }
    Ok(())
}

fn no_git_operation(repo: &Path) -> Result<(), String> {
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "REBASE_HEAD",
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "BISECT_LOG",
    ] {
        let path = git_output(
            repo,
            &["rev-parse", "--path-format=absolute", "--git-path", marker],
        )?
        .stdout;
        if Path::new(path.trim()).exists() {
            return Err(format!(
                "Git operation in progress ({marker}); refusing consolidation"
            ));
        }
    }
    Ok(())
}

fn parse_identity(value: &str) -> Result<(String, String, String), String> {
    let start = value.rfind(" <").ok_or("invalid Git identity")?;
    let end = value[start + 2..]
        .find("> ")
        .ok_or("invalid Git identity")?
        + start
        + 2;
    Ok((
        value[..start].to_string(),
        value[start + 2..end].to_string(),
        value[end + 2..].to_string(),
    ))
}

fn replay_commit(repo: &Path, sha: &str, tree: &str, parent: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["cat-file", "-p", sha])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format_git_error(&["cat-file", "-p", sha], &output));
    }
    let raw = String::from_utf8(output.stdout).map_err(|_| {
        format!("commit {sha} contains non-UTF-8 bytes and cannot be replayed safely")
    })?;
    let (headers, message) = raw.split_once("\n\n").ok_or("invalid Git commit")?;
    let author = headers
        .lines()
        .find_map(|l| l.strip_prefix("author "))
        .ok_or("missing author")?;
    let committer = headers
        .lines()
        .find_map(|l| l.strip_prefix("committer "))
        .ok_or("missing committer")?;
    let author = parse_identity(author)?;
    let committer = parse_identity(committer)?;
    commit_tree(
        repo,
        tree,
        parent,
        message,
        (&author.0, &author.1, &author.2),
        (&committer.0, &committer.1, &committer.2),
    )
}

fn commit_tree(
    repo: &Path,
    tree: &str,
    parent: &str,
    message: &str,
    author: (&str, &str, &str),
    committer: (&str, &str, &str),
) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(["commit-tree", tree, "-p", parent, "-F", "-"])
        .current_dir(repo)
        .env("GIT_AUTHOR_NAME", author.0)
        .env("GIT_AUTHOR_EMAIL", author.1)
        .env("GIT_AUTHOR_DATE", author.2)
        .env("GIT_COMMITTER_NAME", committer.0)
        .env("GIT_COMMITTER_EMAIL", committer.1)
        .env("GIT_COMMITTER_DATE", committer.2)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("missing commit-tree stdin")?
        .write_all(message.as_bytes())
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format_git_error(&["commit-tree"], &output));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
