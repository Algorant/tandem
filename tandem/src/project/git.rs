//! Git process plumbing for the board's repository.
//!
//! Every Git call made by Tandem goes through [`run`], which strips
//! repository-selecting environment variables inherited from a caller (for
//! example a Git hook), never prompts for credentials, and can be bounded by a
//! timeout for network operations.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Repository locations for the checkout Tandem was started in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitContext {
    /// The main worktree, which holds the one local board of this clone.
    pub(crate) main_worktree: PathBuf,
    /// The shared Git directory of the clone (refs, objects, Tandem state).
    pub(crate) common_dir: PathBuf,
    /// The Git directory of the current checkout; differs from `common_dir`
    /// in a linked worktree. Holds that checkout's actor identity.
    pub(crate) git_dir: PathBuf,
}

impl GitContext {
    /// Directory for Tandem's private sync state inside the Git directory.
    pub(crate) fn state_dir(&self) -> PathBuf {
        self.common_dir.join("tandem")
    }
}

#[derive(Debug)]
pub(crate) struct GitOutput {
    pub(crate) code: Option<i32>,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: String,
    pub(crate) timed_out: bool,
}

impl GitOutput {
    pub(crate) fn success(&self) -> bool {
        self.code == Some(0)
    }

    pub(crate) fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).trim().to_string()
    }
}

/// Options for one Git invocation.
#[derive(Default)]
pub(crate) struct GitCall<'a> {
    pub(crate) env: Vec<(&'a str, OsString)>,
    pub(crate) stdin: Option<&'a [u8]>,
    pub(crate) timeout: Option<Duration>,
}

const SCRUBBED_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

pub(crate) fn run(dir: &Path, args: &[&str], call: GitCall<'_>) -> Result<GitOutput, String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(dir).args(args);
    for key in SCRUBBED_ENV {
        command.env_remove(key);
    }
    command.env("GIT_TERMINAL_PROMPT", "0");
    for (key, value) in &call.env {
        command.env(key, value);
    }
    command
        .stdin(if call.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("could not run git: {error}"))?;
    let writer = call.stdin.map(|input| {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let input = input.to_vec();
        std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        })
    });
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout.read_to_end(&mut buffer);
        buffer
    });
    let err_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer);
        buffer
    });
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(error) => return Err(format!("could not wait for git: {error}")),
        }
        if call.timeout.is_some_and(|limit| started.elapsed() >= limit) {
            let _ = child.kill();
            let _ = child.wait();
            timed_out = true;
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    let stdout = out_reader.join().unwrap_or_default();
    let stderr = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).to_string();
    Ok(GitOutput {
        code: status.and_then(|status| status.code()),
        stdout,
        stderr,
        timed_out,
    })
}

/// Runs a Git command that must succeed and returns its trimmed stdout.
pub(crate) fn text(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = run(dir, args, GitCall::default())?;
    if output.success() {
        Ok(output.stdout_text())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            output.stderr.trim()
        ))
    }
}

/// Resolves the Git context for `start`, or `None` outside a work tree.
pub(crate) fn detect(start: &Path) -> Option<GitContext> {
    let output = run(
        start,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-dir",
            "--git-common-dir",
            "--show-toplevel",
        ],
        GitCall::default(),
    )
    .ok()?;
    if !output.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut lines = text.lines();
    let git_dir = PathBuf::from(lines.next()?);
    let common_dir = PathBuf::from(lines.next()?);
    let toplevel = PathBuf::from(lines.next()?);
    let main_worktree = if normalized(&git_dir) == normalized(&common_dir) {
        toplevel
    } else {
        let list = run(
            start,
            &["worktree", "list", "--porcelain"],
            GitCall::default(),
        )
        .ok()?;
        let text = String::from_utf8_lossy(&list.stdout).to_string();
        PathBuf::from(text.lines().next()?.strip_prefix("worktree ")?)
    };
    Some(GitContext {
        main_worktree,
        common_dir,
        git_dir,
    })
}

fn normalized(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
