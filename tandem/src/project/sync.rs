//! Independent metadata sync over the repository's `tandem` branch.
//!
//! The board is the ignored `.tandem/` folder of the main worktree. Tandem
//! keeps three local refs:
//!
//! - `refs/tandem/pending`: safety copy of the latest local board snapshot,
//!   used to restore unsynced changes after an old checkout or `git clean`;
//! - `refs/tandem/base`: the last `tandem` branch commit this board merged;
//! - `refs/tandem/remote`: the most recently fetched remote tip.
//!
//! Sync snapshots the board, merges base/local/remote record by record
//! (matching records by `uid`), numbers provisional records, validates the
//! result, pushes it without force, and only then writes it into the board.
//! Nothing here touches the source branch, index, or working tree.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::git::{self, GitCall, GitContext};
use super::write::HierarchyLock;
use super::{parse_document, split_frontmatter, ProjectHierarchy, TandemProject};
use crate::protocol::hierarchy::DocumentLocation;
use crate::protocol::ids::{
    assign_sequential_ids, is_provisional, replace_id_token, NumberingKind, NumberingRecord,
};
use crate::protocol::merge::{merge_ledger, merge_markdown, Merged};
use crate::CliError;

pub(crate) const BRANCH: &str = "tandem";
const REMOTE_BRANCH_REF: &str = "refs/heads/tandem";
const BASE_REF: &str = "refs/tandem/base";
const PENDING_REF: &str = "refs/tandem/pending";
const FETCHED_REF: &str = "refs/tandem/remote";
pub(crate) const PROTOCOL_VERSION: &str = crate::protocol::config::PROTOCOL_VERSION;
const MAX_ATTEMPTS: usize = 5;
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// File path (relative to the board) to text content.
pub(crate) type Files = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Publish local changes; fetch only when the remote has moved.
    Publish,
    /// Fetch incoming changes and publish local ones.
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Local and remote boards agree.
    Synced,
    /// Local changes are saved and waiting; the reason says why.
    Pending(String),
    /// The repository has no remote; the board is local only.
    LocalOnly,
    /// The board is not in a Git repository.
    NotGit,
}

#[derive(Debug, Clone)]
pub(crate) struct ConflictSummary {
    pub(crate) id: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Held {
    pub(crate) path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Report {
    pub(crate) outcome: Outcome,
    pub(crate) remote: Option<String>,
    pub(crate) renames: Vec<(String, String)>,
    pub(crate) conflicts: Vec<ConflictSummary>,
    pub(crate) held: Vec<Held>,
    pub(crate) restored: usize,
    pub(crate) received: bool,
    pub(crate) published: bool,
}

impl Report {
    fn new(outcome: Outcome) -> Self {
        Self {
            outcome,
            remote: None,
            renames: Vec::new(),
            conflicts: Vec::new(),
            held: Vec::new(),
            restored: 0,
            received: false,
            published: false,
        }
    }

    /// A report for a sync that could not run; local changes stay saved.
    pub(crate) fn pending(reason: String) -> Self {
        Self::new(Outcome::Pending(reason))
    }

    /// The current ID for an ID reported before sync numbered it.
    pub(crate) fn renamed(&self, id: &str) -> String {
        self.renames
            .iter()
            .find(|(old, _)| old == id)
            .map(|(_, new)| new.clone())
            .unwrap_or_else(|| id.to_string())
    }
}

/// One stored conflict: a record whose local change was held back because it
/// contradicts a change from another machine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Conflict {
    pub(crate) key: String,
    pub(crate) id: String,
    pub(crate) reason: String,
    #[serde(rename = "basePath")]
    pub(crate) base_path: Option<String>,
    #[serde(rename = "localPath")]
    pub(crate) local_path: Option<String>,
    #[serde(rename = "remotePath")]
    pub(crate) remote_path: Option<String>,
    pub(crate) base: Option<String>,
    pub(crate) local: Option<String>,
    pub(crate) remote: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keep {
    Local,
    Remote,
    Edited,
}

// ---------------------------------------------------------------------------
// Public operations
// ---------------------------------------------------------------------------

/// Synchronizes the board. Local work is never lost: when the remote is
/// unreachable the report is `Pending` and changes stay in the board and the
/// safety copy.
pub(crate) fn sync(project: &TandemProject, mode: Mode) -> Result<Report, CliError> {
    let Some(ctx) = project.git() else {
        return Ok(Report::new(Outcome::NotGit));
    };
    let _lock = HierarchyLock::acquire(project)?;
    let engine = Engine::new(ctx, project.data_dir());
    engine.run(mode).map_err(CliError::user)
}

/// Local-only status: snapshot, held edits, and open conflicts, without
/// network access.
pub(crate) fn status(project: &TandemProject) -> Result<StatusReport, CliError> {
    let Some(ctx) = project.git() else {
        return Ok(StatusReport::default());
    };
    let _lock = HierarchyLock::acquire(project)?;
    let engine = Engine::new(ctx, project.data_dir());
    engine.status().map_err(CliError::user)
}

#[derive(Debug, Default)]
pub(crate) struct StatusReport {
    pub(crate) git: bool,
    pub(crate) remote: Option<String>,
    pub(crate) published: bool,
    pub(crate) pending: bool,
    pub(crate) conflicts: Vec<ConflictSummary>,
    pub(crate) held: Vec<Held>,
    pub(crate) last_fetch: Option<u64>,
    pub(crate) last_error: Option<String>,
}

/// Whether the last successful fetch is older than `max_age`.
pub(crate) fn is_stale(project: &TandemProject, max_age: Duration) -> bool {
    let Some(ctx) = project.git() else {
        return false;
    };
    let state = State::load(ctx);
    match state.last_fetch {
        Some(last) => now_secs().saturating_sub(last) >= max_age.as_secs(),
        None => true,
    }
}

/// Whether the board syncs through a remote (so new records start with
/// provisional IDs).
pub(crate) fn has_remote(project: &TandemProject) -> bool {
    project.git().is_some_and(|ctx| remote_name(ctx).is_some())
}

/// Records that Tandem itself removed `path` (relative to the board), so the
/// next snapshot does not restore it from the safety copy.
pub(crate) fn note_removed(project: &TandemProject, path: &Path) -> Result<(), CliError> {
    let Some(ctx) = project.git() else {
        return Ok(());
    };
    let Ok(relative) = path.strip_prefix(project.data_dir()) else {
        return Ok(());
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    let file = ctx.state_dir().join("removed");
    fs::create_dir_all(ctx.state_dir())?;
    let mut content = fs::read_to_string(&file).unwrap_or_default();
    content.push_str(&relative);
    content.push('\n');
    fs::write(file, content)?;
    Ok(())
}

/// Finds the open conflict for a record ID.
pub(crate) fn conflict_for(project: &TandemProject, id: &str) -> Option<Conflict> {
    let ctx = project.git()?;
    load_conflicts(ctx)
        .into_iter()
        .find(|conflict| conflict.id == id)
}

/// Resolves an open conflict and leaves the chosen version in the board for
/// the next sync to publish.
pub(crate) fn resolve(project: &TandemProject, id: &str, keep: Keep) -> Result<Conflict, CliError> {
    let Some(ctx) = project.git() else {
        return Err(CliError::user("this board is not in a Git repository"));
    };
    let _lock = HierarchyLock::acquire(project)?;
    let conflict = load_conflicts(ctx)
        .into_iter()
        .find(|conflict| conflict.id == id)
        .ok_or_else(|| CliError::user(format!("no open sync conflict for {id}")))?;
    let board = project.data_dir();
    let (content, path, other) = match keep {
        Keep::Local => (
            conflict.local.clone(),
            conflict.local_path.clone(),
            conflict.remote_path.clone(),
        ),
        Keep::Remote => (
            conflict.remote.clone(),
            conflict.remote_path.clone(),
            conflict.local_path.clone(),
        ),
        Keep::Edited => (None, None, None),
    };
    if keep != Keep::Edited {
        if let Some(other) = other.filter(|other| Some(other) != path.as_ref()) {
            let other_path = board.join(&other);
            if other_path.exists() {
                fs::remove_file(&other_path)?;
                note_removed(project, &other_path)?;
            }
        }
        match (content, path) {
            (Some(content), Some(path)) => super::write::write_atomic(&board.join(path), &content)?,
            (None, Some(path)) => {
                let path = board.join(path);
                if path.exists() {
                    fs::remove_file(&path)?;
                    note_removed(project, &path)?;
                }
            }
            _ => {}
        }
    }
    fs::remove_file(conflict_path(ctx, &conflict.key))?;
    Ok(conflict)
}

/// Restores a missing board from the safety copy, or downloads it from the
/// remote `tandem` branch into a fresh clone. Returns whether a board exists
/// afterwards.
pub(crate) fn recover_board(ctx: &GitContext) -> Result<bool, CliError> {
    let board = ctx.main_worktree.join(".tandem");
    let engine = Engine::new(ctx, &board);
    engine.recover().map_err(CliError::user)
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

struct Engine<'a> {
    ctx: &'a GitContext,
    board: PathBuf,
    state: PathBuf,
}

/// One side of a merge: path to blob ID plus loaded text.
#[derive(Debug, Clone, Default)]
struct Side {
    oids: BTreeMap<String, String>,
    files: Files,
}

struct Snapshot {
    raw: Side,
    /// Effective local side: held paths replaced by their base version.
    local: Side,
    held: Vec<Held>,
    held_paths: Vec<String>,
    restored: usize,
}

struct MergeResult {
    files: Files,
    renames: Vec<(String, String)>,
    conflicts: Vec<Conflict>,
    held: Vec<Held>,
    held_paths: Vec<String>,
}

enum Push {
    Accepted,
    Rejected,
}

impl<'a> Engine<'a> {
    fn new(ctx: &'a GitContext, board: &Path) -> Self {
        Self {
            ctx,
            board: board.to_path_buf(),
            state: ctx.state_dir(),
        }
    }

    fn run(&self, mode: Mode) -> Result<Report, String> {
        self.ensure_current_board()?;
        let base = self.ref_commit(BASE_REF)?;
        let base_side = match &base {
            Some(commit) => self.load_tree(commit)?,
            None => Side::default(),
        };
        let open = load_conflicts(self.ctx);
        let snapshot = self.snapshot(&base_side, &open)?;
        let remote = remote_name(self.ctx);
        let mut report = Report::new(Outcome::Synced);
        report.remote = remote.clone();
        report.restored = snapshot.restored;
        report.held = snapshot.held.clone();
        let Some(remote) = remote else {
            report.outcome = Outcome::LocalOnly;
            report.conflicts = summaries(&open);
            return Ok(report);
        };
        let local_changed = base.is_none() || snapshot.local.oids != base_side.oids;
        if mode == Mode::Publish && !local_changed {
            report.conflicts = summaries(&open);
            return Ok(report);
        }
        let mut optimistic = mode == Mode::Publish && base.is_some();
        for _ in 0..MAX_ATTEMPTS {
            let (remote_commit, remote_side) = if optimistic {
                (base.clone(), base_side.clone())
            } else {
                match self.fetch(&remote) {
                    Ok(Some(commit)) => {
                        let side = self.load_tree(&commit)?;
                        (Some(commit), side)
                    }
                    // The remote branch does not exist (yet): republish.
                    Ok(None) => (base.clone(), base_side.clone()),
                    Err(error) => {
                        self.record_error(&error);
                        report.outcome = Outcome::Pending(error);
                        report.conflicts = summaries(&open);
                        return Ok(report);
                    }
                }
            };
            if base.is_none() && remote_commit.is_some() && !optimistic {
                return Err(format!(
                    "the remote already has a Tandem board on its `{BRANCH}` branch, but this .tandem/ was created separately. Move .tandem/ aside and run any tandem command to download the shared board."
                ));
            }
            let merged = self.merge(&base_side, &snapshot, &remote_side, &open)?;
            let conflicts = merged.conflicts.clone();
            let commit = if remote_commit.is_some() && merged.files == remote_side.files {
                remote_commit.clone().expect("checked")
            } else {
                let tree = self.write_tree(&merged.files, &[&snapshot.raw, &remote_side])?;
                let parents: Vec<String> = remote_commit.iter().cloned().collect();
                let commit = self.commit(&tree, &parents, "tandem: sync")?;
                match self.push(&remote, &commit) {
                    Ok(Push::Accepted) => {}
                    Ok(Push::Rejected) => {
                        optimistic = false;
                        continue;
                    }
                    Err(error) => {
                        self.record_error(&error);
                        report.outcome = Outcome::Pending(error);
                        report.conflicts = summaries(&open);
                        return Ok(report);
                    }
                }
                report.published = true;
                commit
            };
            if !optimistic {
                self.record_fetch();
            }
            report.received = remote_commit != base && !optimistic;
            self.apply(&merged, &snapshot, &base_side, &open)?;
            self.update_ref(BASE_REF, &commit)?;
            let mut all_conflicts = open.clone();
            for conflict in &conflicts {
                all_conflicts.retain(|existing| existing.key != conflict.key);
                all_conflicts.push(conflict.clone());
            }
            self.refresh_open_conflicts(&mut all_conflicts, &merged.files)?;
            save_conflicts(self.ctx, &all_conflicts)?;
            let _ = self.snapshot(&self.load_tree(&commit)?, &all_conflicts)?;
            self.clear_error();
            report.renames = merged.renames;
            report.held = merged.held;
            report.conflicts = summaries(&all_conflicts);
            report.outcome = if report.conflicts.is_empty() && report.held.is_empty() {
                Outcome::Synced
            } else {
                Outcome::Pending("some changes are held; see `tandem sync status`".to_string())
            };
            return Ok(report);
        }
        report.outcome =
            Outcome::Pending("the remote kept changing during sync; try again".to_string());
        report.conflicts = summaries(&open);
        Ok(report)
    }

    fn status(&self) -> Result<StatusReport, String> {
        self.ensure_current_board()?;
        let base = self.ref_commit(BASE_REF)?;
        let base_side = match &base {
            Some(commit) => self.load_tree(commit)?,
            None => Side::default(),
        };
        let open = load_conflicts(self.ctx);
        let snapshot = self.snapshot(&base_side, &open)?;
        let state = State::load(self.ctx);
        Ok(StatusReport {
            git: true,
            remote: remote_name(self.ctx),
            published: base.is_some(),
            pending: base.is_none() || snapshot.local.oids != base_side.oids,
            conflicts: summaries(&open),
            held: snapshot.held,
            last_fetch: state.last_fetch,
            last_error: state.last_error,
        })
    }

    fn recover(&self) -> Result<bool, String> {
        if let Some(pending) = self.ref_commit(PENDING_REF)? {
            let side = self.load_tree(&pending)?;
            if side.files.contains_key("tandem.md") {
                for (path, content) in &side.files {
                    let target = self.board.join(path);
                    if !target.exists() {
                        write_file(&target, content)?;
                    }
                }
                return Ok(true);
            }
        }
        let Some(remote) = remote_name(self.ctx) else {
            return Ok(false);
        };
        let tracking = format!("refs/remotes/{remote}/{BRANCH}");
        if self.ref_commit(&tracking)?.is_none() {
            return Ok(false);
        }
        let Some(commit) = self.fetch(&remote)? else {
            return Ok(false);
        };
        let side = self.load_tree(&commit)?;
        if !read_board_dir(&self.board)?.is_empty() {
            return Err(format!(
                "found leftover files in {} from before this repository moved its board to the `{BRANCH}` branch. Run `tandem migrate --adopt` to bring them over.",
                self.board.display()
            ));
        }
        for (path, content) in &side.files {
            write_file(&self.board.join(path), content)?;
        }
        self.update_ref(BASE_REF, &commit)?;
        self.record_fetch();
        self.ensure_excluded()?;
        let empty = Side::default();
        let _ = self.snapshot(&empty, &[])?;
        Ok(true)
    }

    /// Refuses to sync files that an old checkout put into the board.
    fn ensure_current_board(&self) -> Result<(), String> {
        let config = fs::read_to_string(self.board.join("tandem.md"))
            .map_err(|error| format!("could not read .tandem/tandem.md: {error}"))?;
        let version = top_field(&config, "protocolVersion").unwrap_or_default();
        if version != PROTOCOL_VERSION {
            return Err(historical_message(&version));
        }
        Ok(())
    }

    // -- snapshot ------------------------------------------------------------

    fn snapshot(&self, base: &Side, open: &[Conflict]) -> Result<Snapshot, String> {
        fs::create_dir_all(&self.state).map_err(|error| error.to_string())?;
        let restored = self.restore_missing()?;
        let index = self.state.join("index");
        self.board_git(&["add", "-A", "--", "."], &index)?;
        // Keep only synced board content (no editor swap files or runtime state).
        let listed = self.board_git(&["ls-files", "-z", "--cached"], &index)?;
        let other: Vec<&str> = listed
            .split('\0')
            .filter(|path| !path.is_empty() && !is_board_path(path))
            .collect();
        if !other.is_empty() {
            let mut args = vec!["rm", "--cached", "-q", "--"];
            args.extend(other);
            self.board_git(&args, &index)?;
        }
        let tree = self.board_git(&["write-tree"], &index)?;
        let raw = self.load_tree(&tree)?;
        // Update the safety copy when the board changed.
        let pending_tree = self.ref_commit(&format!("{PENDING_REF}^{{tree}}"))?;
        if pending_tree.as_deref() != Some(tree.as_str()) {
            let commit = self.commit(&tree, &[], "tandem: local safety copy")?;
            self.update_ref(PENDING_REF, &commit)?;
        }
        let _ = fs::remove_file(self.state.join("removed"));

        let open_keys: Vec<&str> = open.iter().map(|conflict| conflict.key.as_str()).collect();
        let base_uids = uid_index(&base.files);
        let mut local = raw.clone();
        let mut held = Vec::new();
        let mut held_paths = Vec::new();
        let paths: Vec<String> = raw
            .files
            .keys()
            .chain(base.files.keys())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        for path in paths {
            let local_content = raw.files.get(&path);
            let base_content = base.files.get(&path);
            if local_content == base_content {
                continue;
            }
            let key = local_content
                .map(|content| entry_key(&path, content))
                .or_else(|| base_content.map(|content| entry_key(&path, content)))
                .unwrap_or_else(|| format!("path:{path}"));
            let problem = if open_keys.contains(&key.as_str()) {
                Some("has an unresolved sync conflict; run `tandem sync status`".to_string())
            } else {
                local_content.and_then(|content| {
                    local_problem(&path, content, base_content.map(String::as_str), &base_uids)
                })
            };
            if let Some(reason) = problem {
                if !open_keys.contains(&key.as_str()) {
                    held.push(Held {
                        path: format!(".tandem/{path}"),
                        reason,
                    });
                }
                held_paths.push(path.clone());
                match base_content {
                    Some(content) => {
                        local.files.insert(path.clone(), content.clone());
                        local.oids.insert(path.clone(), base.oids[&path].clone());
                    }
                    None => {
                        local.files.remove(&path);
                        local.oids.remove(&path);
                    }
                }
            }
        }
        Ok(Snapshot {
            raw,
            local,
            held,
            held_paths,
            restored,
        })
    }

    /// Restores board files that disappeared without a Tandem command, for
    /// example after `git clean -fdx` or checking out an older commit.
    fn restore_missing(&self) -> Result<usize, String> {
        let Some(pending) = self.ref_commit(PENDING_REF)? else {
            return Ok(0);
        };
        let removed: Vec<String> = fs::read_to_string(self.state.join("removed"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
        let entries = self.ls_tree(&pending)?;
        let missing: Vec<(&String, &String)> = entries
            .iter()
            .filter(|(path, _)| {
                if self.board.join(path).exists() || removed.contains(path) {
                    return false;
                }
                // An archive move leaves the same file name in logs/.
                match path.strip_prefix("tasks/") {
                    Some(name) => !self.board.join("logs").join(name).exists(),
                    None => true,
                }
            })
            .collect();
        if missing.is_empty() {
            return Ok(0);
        }
        let blobs = self.cat_blobs(missing.iter().map(|(_, oid)| oid.as_str()))?;
        for (path, oid) in &missing {
            write_file(&self.board.join(path), &blobs[*oid])?;
        }
        Ok(missing.len())
    }

    // -- merge ---------------------------------------------------------------

    fn merge(
        &self,
        base: &Side,
        snapshot: &Snapshot,
        remote: &Side,
        open: &[Conflict],
    ) -> Result<MergeResult, String> {
        let mut extra_held: Vec<(String, String)> = Vec::new();
        for _ in 0..10 {
            let mut local = snapshot.local.clone();
            for (key, _) in &extra_held {
                revert_key(&mut local, base, key);
            }
            let mut result = self.merge_once(base, &local, remote, open)?;
            result.held = snapshot.held.clone();
            result.held_paths = snapshot.held_paths.clone();
            for (key, reason) in &extra_held {
                for path in paths_for_key(&snapshot.raw.files, key) {
                    result.held.push(Held {
                        path: format!(".tandem/{path}"),
                        reason: reason.clone(),
                    });
                    result.held_paths.push(path);
                }
            }
            match validate_board(&result.files) {
                Ok(()) => return Ok(result),
                Err(message) => {
                    let culprits: Vec<String> = changed_keys(base, &local)
                        .into_iter()
                        .filter(|(_, id)| mentions(&message, id))
                        .map(|(key, _)| key)
                        .collect();
                    if culprits.is_empty() {
                        return Err(format!(
                            "sync stopped: the combined board would be invalid ({message}). Your changes are saved locally."
                        ));
                    }
                    for key in culprits {
                        extra_held.push((
                            key,
                            format!("would make the shared board invalid: {message}"),
                        ));
                    }
                }
            }
        }
        Err("sync stopped: could not produce a valid combined board".to_string())
    }

    fn merge_once(
        &self,
        base: &Side,
        local: &Side,
        remote: &Side,
        open: &[Conflict],
    ) -> Result<MergeResult, String> {
        let mut files = Files::new();
        let mut touched: Vec<&String> = Vec::new();
        let all_paths: std::collections::BTreeSet<&String> = base
            .files
            .keys()
            .chain(local.files.keys())
            .chain(remote.files.keys())
            .collect();
        for path in all_paths {
            let (b, l, r) = (
                base.oids.get(path),
                local.oids.get(path),
                remote.oids.get(path),
            );
            if b == l && l == r {
                if let Some(content) = remote.files.get(path) {
                    files.insert(path.clone(), content.clone());
                }
            } else {
                touched.push(path);
            }
        }
        // Group touched paths by record identity.
        let mut keys: BTreeMap<String, [Option<(String, String)>; 3]> = BTreeMap::new();
        for path in touched {
            for (index, side) in [base, local, remote].into_iter().enumerate() {
                if let Some(content) = side.files.get(path) {
                    let key = entry_key(path, content);
                    keys.entry(key).or_default()[index] = Some((path.clone(), content.clone()));
                }
            }
        }
        let mut conflicts = Vec::new();
        for (key, [b, l, r]) in keys {
            let open_conflict = open.iter().any(|conflict| conflict.key == key);
            let (output, conflict) = if open_conflict {
                (r.clone(), None)
            } else {
                merge_entry(&b, &l, &r, &|base, local, remote| {
                    self.text_merge(base, local, remote)
                })
            };
            if let Some(reason) = conflict {
                let id = l
                    .as_ref()
                    .or(r.as_ref())
                    .or(b.as_ref())
                    .map(|(path, content)| record_id(path, content))
                    .unwrap_or_else(|| key.clone());
                conflicts.push(Conflict {
                    key: key.clone(),
                    id,
                    reason,
                    base_path: b.as_ref().map(|(path, _)| path.clone()),
                    local_path: l.as_ref().map(|(path, _)| path.clone()),
                    remote_path: r.as_ref().map(|(path, _)| path.clone()),
                    base: b.map(|(_, content)| content),
                    local: l.map(|(_, content)| content),
                    remote: r.map(|(_, content)| content),
                });
            }
            if let Some((path, content)) = output {
                files.insert(path, content);
            }
        }
        let renames = number_provisional(&mut files);
        Ok(MergeResult {
            files,
            renames,
            conflicts,
            held: Vec::new(),
            held_paths: Vec::new(),
        })
    }

    fn text_merge(&self, base: &str, local: &str, remote: &str) -> Option<String> {
        let dir = self.state.join("tmp");
        fs::create_dir_all(&dir).ok()?;
        let paths = ["base", "local", "remote"].map(|name| dir.join(name));
        for (path, content) in paths.iter().zip([base, local, remote]) {
            fs::write(path, content).ok()?;
        }
        let args = [
            "merge-file",
            "-p",
            "--quiet",
            paths[1].to_str()?,
            paths[0].to_str()?,
            paths[2].to_str()?,
        ];
        let output = git::run(&self.ctx.main_worktree, &args, GitCall::default()).ok()?;
        output
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).to_string())
    }

    // -- apply ---------------------------------------------------------------

    /// Writes the published result into the board. A file edited while sync
    /// ran, or held back, is left alone for the next sync.
    fn apply(
        &self,
        merged: &MergeResult,
        snapshot: &Snapshot,
        base: &Side,
        open: &[Conflict],
    ) -> Result<(), String> {
        let open_paths: Vec<String> = open
            .iter()
            .flat_map(|conflict| {
                [&conflict.local_path, &conflict.remote_path]
                    .into_iter()
                    .flatten()
                    .cloned()
            })
            .collect();
        let unchanged_since = |path: &str, expected: Option<&String>| -> bool {
            let current = fs::read_to_string(self.board.join(path)).ok();
            current.as_ref() == expected
        };
        for (path, content) in &merged.files {
            if snapshot.raw.files.get(path) == Some(content) {
                continue;
            }
            let held = merged.held_paths.contains(path);
            let expected = if held || open_paths.contains(path) {
                base.files.get(path)
            } else {
                snapshot.raw.files.get(path)
            };
            if !unchanged_since(path, expected) {
                continue;
            }
            write_file(&self.board.join(path), content)?;
        }
        for (path, content) in &snapshot.raw.files {
            if merged.files.contains_key(path) || merged.held_paths.contains(path) {
                continue;
            }
            if unchanged_since(path, Some(content)) {
                fs::remove_file(self.board.join(path)).map_err(|error| error.to_string())?;
                self.note_removed(path)?;
            }
        }
        Ok(())
    }

    /// Records a board file this engine removed so the safety copy does not
    /// bring it back.
    fn note_removed(&self, path: &str) -> Result<(), String> {
        fs::create_dir_all(&self.state).map_err(|error| error.to_string())?;
        let file = self.state.join("removed");
        let mut content = fs::read_to_string(&file).unwrap_or_default();
        content.push_str(path);
        content.push('\n');
        fs::write(file, content).map_err(|error| error.to_string())
    }

    fn refresh_open_conflicts(
        &self,
        conflicts: &mut [Conflict],
        files: &Files,
    ) -> Result<(), String> {
        for conflict in conflicts.iter_mut() {
            let current = files
                .iter()
                .find(|(path, content)| entry_key(path, content) == conflict.key);
            conflict.remote_path = current.map(|(path, _)| path.clone());
            conflict.remote = current.map(|(_, content)| content.clone());
        }
        Ok(())
    }

    // -- Git plumbing --------------------------------------------------------

    fn git(&self, args: &[&str]) -> Result<String, String> {
        git::text(&self.ctx.main_worktree, args)
    }

    fn board_git(&self, args: &[&str], index: &Path) -> Result<String, String> {
        fs::create_dir_all(&self.board).map_err(|error| error.to_string())?;
        let git_dir = format!("--git-dir={}", self.ctx.common_dir.display());
        let work_tree = format!("--work-tree={}", self.board.display());
        let mut full = vec![git_dir.as_str(), work_tree.as_str()];
        full.extend(args);
        let output = git::run(
            &self.board,
            &full,
            GitCall {
                env: vec![("GIT_INDEX_FILE", OsString::from(index))],
                ..Default::default()
            },
        )?;
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

    fn ref_commit(&self, name: &str) -> Result<Option<String>, String> {
        let output = git::run(
            &self.ctx.main_worktree,
            &["rev-parse", "--verify", "--quiet", name],
            GitCall::default(),
        )?;
        Ok(output.success().then(|| output.stdout_text()))
    }

    fn update_ref(&self, name: &str, commit: &str) -> Result<(), String> {
        self.git(&["update-ref", name, commit]).map(|_| ())
    }

    fn ls_tree(&self, rev: &str) -> Result<BTreeMap<String, String>, String> {
        let output = git::run(
            &self.ctx.main_worktree,
            &["ls-tree", "-r", "-z", rev],
            GitCall::default(),
        )?;
        if !output.success() {
            return Err(format!("git ls-tree failed: {}", output.stderr.trim()));
        }
        let mut entries = BTreeMap::new();
        for record in output.stdout.split(|byte| *byte == 0) {
            let record = String::from_utf8_lossy(record);
            let Some((meta, path)) = record.split_once('\t') else {
                continue;
            };
            let mut parts = meta.split_whitespace();
            let (_, kind, oid) = (parts.next(), parts.next(), parts.next());
            if kind == Some("blob") {
                if let Some(oid) = oid {
                    entries.insert(path.to_string(), oid.to_string());
                }
            }
        }
        Ok(entries)
    }

    fn cat_blobs<'b>(
        &self,
        oids: impl Iterator<Item = &'b str>,
    ) -> Result<HashMap<String, String>, String> {
        let mut input = String::new();
        let mut wanted = Vec::new();
        for oid in oids {
            input.push_str(oid);
            input.push('\n');
            wanted.push(oid.to_string());
        }
        if wanted.is_empty() {
            return Ok(HashMap::new());
        }
        let output = git::run(
            &self.ctx.main_worktree,
            &["cat-file", "--batch"],
            GitCall {
                stdin: Some(input.as_bytes()),
                ..Default::default()
            },
        )?;
        if !output.success() {
            return Err(format!("git cat-file failed: {}", output.stderr.trim()));
        }
        let data = output.stdout;
        let mut blobs = HashMap::new();
        let mut cursor = 0;
        for oid in wanted {
            let header_end = data[cursor..]
                .iter()
                .position(|byte| *byte == b'\n')
                .ok_or("truncated git cat-file output")?
                + cursor;
            let header = String::from_utf8_lossy(&data[cursor..header_end]).to_string();
            let size: usize = header
                .split_whitespace()
                .nth(2)
                .and_then(|size| size.parse().ok())
                .ok_or_else(|| format!("missing Git object {oid}"))?;
            let start = header_end + 1;
            let end = start + size;
            blobs.insert(
                oid,
                String::from_utf8_lossy(data.get(start..end).ok_or("truncated blob")?).to_string(),
            );
            cursor = end + 1;
        }
        Ok(blobs)
    }

    fn load_tree(&self, rev: &str) -> Result<Side, String> {
        let oids = self.ls_tree(rev)?;
        let blobs = self.cat_blobs(oids.values().map(String::as_str))?;
        let files = oids
            .iter()
            .map(|(path, oid)| (path.clone(), blobs[oid].clone()))
            .collect();
        Ok(Side { oids, files })
    }

    fn hash_blob(&self, content: &str) -> Result<String, String> {
        let output = git::run(
            &self.ctx.main_worktree,
            &["hash-object", "-w", "--stdin"],
            GitCall {
                stdin: Some(content.as_bytes()),
                ..Default::default()
            },
        )?;
        if output.success() {
            Ok(output.stdout_text())
        } else {
            Err(format!("git hash-object failed: {}", output.stderr.trim()))
        }
    }

    fn write_tree(&self, files: &Files, known: &[&Side]) -> Result<String, String> {
        let mut info = String::new();
        for (path, content) in files {
            let oid = known
                .iter()
                .find_map(|side| {
                    (side.files.get(path) == Some(content)).then(|| side.oids[path].clone())
                })
                .map(Ok)
                .unwrap_or_else(|| self.hash_blob(content))?;
            info.push_str(&format!("100644 {oid}\t{path}\n"));
        }
        fs::create_dir_all(&self.state).map_err(|error| error.to_string())?;
        let index = self.state.join("build-index");
        let _ = fs::remove_file(&index);
        let env = vec![("GIT_INDEX_FILE", OsString::from(&index))];
        let output = git::run(
            &self.ctx.main_worktree,
            &["update-index", "--add", "--index-info"],
            GitCall {
                env: env.clone(),
                stdin: Some(info.as_bytes()),
                ..Default::default()
            },
        )?;
        if !output.success() {
            return Err(format!("git update-index failed: {}", output.stderr.trim()));
        }
        let output = git::run(
            &self.ctx.main_worktree,
            &["write-tree"],
            GitCall {
                env,
                ..Default::default()
            },
        )?;
        let _ = fs::remove_file(&index);
        if output.success() {
            Ok(output.stdout_text())
        } else {
            Err(format!("git write-tree failed: {}", output.stderr.trim()))
        }
    }

    fn commit(&self, tree: &str, parents: &[String], message: &str) -> Result<String, String> {
        let host = hostname();
        let message = format!("{message} from {host}");
        let mut args = vec!["commit-tree", "--no-gpg-sign", tree];
        for parent in parents {
            args.push("-p");
            args.push(parent);
        }
        args.push("-m");
        args.push(&message);
        let identity = [
            ("GIT_AUTHOR_NAME", "Tandem"),
            ("GIT_AUTHOR_EMAIL", "tandem@localhost"),
            ("GIT_COMMITTER_NAME", "Tandem"),
            ("GIT_COMMITTER_EMAIL", "tandem@localhost"),
        ];
        let output = git::run(
            &self.ctx.main_worktree,
            &args,
            GitCall {
                env: identity
                    .iter()
                    .map(|(key, value)| (*key, OsString::from(value)))
                    .collect(),
                ..Default::default()
            },
        )?;
        if output.success() {
            Ok(output.stdout_text())
        } else {
            Err(format!("git commit-tree failed: {}", output.stderr.trim()))
        }
    }

    fn fetch(&self, remote: &str) -> Result<Option<String>, String> {
        let refspec = format!("+{REMOTE_BRANCH_REF}:{FETCHED_REF}");
        let output = git::run(
            &self.ctx.main_worktree,
            &["fetch", "--no-tags", "--quiet", remote, &refspec],
            GitCall {
                timeout: Some(timeout()),
                ..Default::default()
            },
        )?;
        if output.timed_out {
            return Err(format!("offline: fetching from `{remote}` timed out"));
        }
        if output.success() {
            let _ = self.update_tracking(remote);
            return self.ref_commit(FETCHED_REF);
        }
        if output.stderr.contains("couldn't find remote ref") {
            return Ok(None);
        }
        Err(format!(
            "offline: could not fetch from `{remote}`: {}",
            first_line(&output.stderr)
        ))
    }

    fn update_tracking(&self, remote: &str) -> Result<(), String> {
        if let Some(commit) = self.ref_commit(FETCHED_REF)? {
            self.update_ref(&format!("refs/remotes/{remote}/{BRANCH}"), &commit)?;
        }
        Ok(())
    }

    fn push(&self, remote: &str, commit: &str) -> Result<Push, String> {
        let refspec = format!("{commit}:{REMOTE_BRANCH_REF}");
        let output = git::run(
            &self.ctx.main_worktree,
            &["push", "--porcelain", "--no-verify", remote, &refspec],
            GitCall {
                timeout: Some(timeout()),
                ..Default::default()
            },
        )?;
        if output.timed_out {
            return Err(format!("offline: pushing to `{remote}` timed out"));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if output.success() {
            self.update_ref(&format!("refs/remotes/{remote}/{BRANCH}"), commit)?;
            return Ok(Push::Accepted);
        }
        if stdout.contains("[rejected]") {
            return Ok(Push::Rejected);
        }
        Err(format!(
            "offline: could not push to `{remote}`: {}",
            first_line(&output.stderr)
        ))
    }

    fn ensure_excluded(&self) -> Result<(), String> {
        let output = git::run(
            &self.ctx.main_worktree,
            &["check-ignore", "-q", ".tandem/tandem.md"],
            GitCall::default(),
        )?;
        if output.success() {
            return Ok(());
        }
        let exclude = self.ctx.common_dir.join("info").join("exclude");
        let mut content = fs::read_to_string(&exclude).unwrap_or_default();
        if !content.lines().any(|line| line.trim() == "/.tandem/") {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("/.tandem/\n");
            if let Some(parent) = exclude.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::write(&exclude, content).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn record_fetch(&self) {
        let mut state = State::load(self.ctx);
        state.last_fetch = Some(now_secs());
        state.save(self.ctx);
    }

    fn record_error(&self, error: &str) {
        let mut state = State::load(self.ctx);
        state.last_error = Some(error.to_string());
        state.save(self.ctx);
    }

    fn clear_error(&self) {
        let mut state = State::load(self.ctx);
        if state.last_error.take().is_some() {
            state.save(self.ctx);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers shared with migration
// ---------------------------------------------------------------------------

/// Makes `.tandem/` ignored by the source repository through the clone-local
/// exclude file.
pub(crate) fn ensure_excluded(ctx: &GitContext) -> Result<(), CliError> {
    Engine::new(ctx, &ctx.main_worktree.join(".tandem"))
        .ensure_excluded()
        .map_err(CliError::user)
}

/// Publishes `board` (already written to disk) as the first commit of the
/// remote `tandem` branch and records it as the local base.
pub(crate) fn publish_initial(
    ctx: &GitContext,
    board: &Path,
    remote: &str,
) -> Result<String, CliError> {
    let engine = Engine::new(ctx, board);
    (|| -> Result<String, String> {
        let snapshot = engine.snapshot(&Side::default(), &[])?;
        let tree = engine.write_tree(&snapshot.raw.files, &[&snapshot.raw])?;
        let commit = engine.commit(&tree, &[], "tandem: move the board to the tandem branch")?;
        match engine.push(remote, &commit)? {
            Push::Accepted => {}
            Push::Rejected => {
                return Err(format!(
                    "the remote `{remote}` already has a `{BRANCH}` branch; run `tandem migrate --adopt` instead"
                ))
            }
        }
        engine.update_ref(BASE_REF, &commit)?;
        engine.record_fetch();
        Ok(commit)
    })()
    .map_err(CliError::user)
}

/// How a protocol upgrade reached the shared board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Upgraded {
    /// This checkout published the upgraded board as one commit.
    Published,
    /// Another machine already upgraded the shared board; it was downloaded.
    Received,
    /// This board has never been synced; the files were not touched.
    NotSynced,
}

/// Moves a synced board from the previous protocol version to the current
/// one in one commit on the `tandem` branch.
///
/// `converted` is the complete upgraded file set computed from the board on
/// disk. The board is written, then pushed without force; a rejected or failed
/// push restores the original files. When another machine has already
/// published the upgraded board, this checkout only downloads it, and refuses
/// if it holds changes that were never synced.
pub(crate) fn publish_upgrade(
    ctx: &GitContext,
    board: &Path,
    remote: &str,
    converted: &Files,
) -> Result<Upgraded, CliError> {
    let engine = Engine::new(ctx, board);
    (|| -> Result<Upgraded, String> {
        let Some(base_commit) = engine.ref_commit(BASE_REF)? else {
            return Ok(Upgraded::NotSynced);
        };
        if !load_conflicts(ctx).is_empty() {
            return Err("this board has unresolved sync conflicts; resolve them with the previous Tandem release (`tandem sync status`) before migrating".to_string());
        }
        let base_side = engine.load_tree(&base_commit)?;
        let snapshot = engine.snapshot(&base_side, &[])?;
        let Some(remote_commit) = engine.fetch(remote)? else {
            return Err(format!("`{remote}` has no `{BRANCH}` branch to upgrade"));
        };
        let remote_side = engine.load_tree(&remote_commit)?;
        let remote_version = remote_side
            .files
            .get("tandem.md")
            .and_then(|config| top_field(config, "protocolVersion"));
        if remote_version.as_deref() == Some(PROTOCOL_VERSION) {
            if snapshot.raw.files != base_side.files {
                return Err(format!(
                    "another machine already upgraded this board to protocol {PROTOCOL_VERSION}, but this checkout has changes that were never synced. Sync them with the previous Tandem release first, or move them aside, then run `tandem migrate` again"
                ));
            }
            for path in snapshot.raw.files.keys() {
                if !remote_side.files.contains_key(path) {
                    fs::remove_file(board.join(path)).map_err(|error| error.to_string())?;
                }
            }
            for (path, content) in &remote_side.files {
                if snapshot.raw.files.get(path) != Some(content) {
                    write_file(&board.join(path), content)?;
                }
            }
            engine.update_ref(BASE_REF, &remote_commit)?;
            engine.record_fetch();
            let _ = engine.snapshot(&remote_side, &[])?;
            return Ok(Upgraded::Received);
        }
        if remote_commit != base_commit {
            return Err("this board is behind the shared board; sync it with the previous Tandem release (`tandem sync`), then run `tandem migrate` again".to_string());
        }
        validate_board(converted).map_err(|message| {
            format!("cannot migrate: the upgraded board would be invalid ({message})")
        })?;
        let restore = |engine: &Engine<'_>| -> Result<(), String> {
            for (path, content) in &snapshot.raw.files {
                if converted.get(path) != Some(content) {
                    write_file(&engine.board.join(path), content)?;
                }
            }
            Ok(())
        };
        for (path, content) in converted {
            if snapshot.raw.files.get(path) != Some(content) {
                write_file(&board.join(path), content)?;
            }
        }
        let published = (|| -> Result<String, String> {
            let tree = engine.write_tree(converted, &[&snapshot.raw, &remote_side])?;
            let commit = engine.commit(
                &tree,
                &[remote_commit.clone()],
                &format!("tandem: upgrade the board to protocol {PROTOCOL_VERSION}"),
            )?;
            match engine.push(remote, &commit)? {
                Push::Accepted => Ok(commit),
                Push::Rejected => Err("the shared board changed while migrating; run `tandem migrate` again".to_string()),
            }
        })();
        let commit = match published {
            Ok(commit) => commit,
            Err(error) => {
                restore(&engine)?;
                return Err(error);
            }
        };
        engine.update_ref(BASE_REF, &commit)?;
        engine.record_fetch();
        let _ = engine.snapshot(&engine.load_tree(&commit)?, &[])?;
        Ok(Upgraded::Published)
    })()
    .map_err(CliError::user)
}

/// Fetches the remote board, returning its commit and files.
pub(crate) fn fetch_remote_board(
    ctx: &GitContext,
    remote: &str,
) -> Result<Option<(String, Files)>, CliError> {
    let engine = Engine::new(ctx, &ctx.main_worktree.join(".tandem"));
    (|| -> Result<Option<(String, Files)>, String> {
        let Some(commit) = engine.fetch(remote)? else {
            return Ok(None);
        };
        let side = engine.load_tree(&commit)?;
        Ok(Some((commit, side.files)))
    })()
    .map_err(CliError::user)
}

/// Loads the `.tandem/` files of a source commit (legacy boards).
pub(crate) fn source_board_files(ctx: &GitContext, commit: &str) -> Result<Files, CliError> {
    let engine = Engine::new(ctx, &ctx.main_worktree.join(".tandem"));
    let rev = format!("{commit}:.tandem");
    engine
        .load_tree(&rev)
        .map(|side| {
            side.files
                .into_iter()
                .filter(|(path, _)| is_board_path(path))
                .collect()
        })
        .map_err(CliError::user)
}

/// Three-way merges `local` onto `remote` from `base` and publishes the
/// result on top of `remote_commit`. Used by `migrate --adopt`, whose base is
/// a converted legacy board rather than a previous sync.
pub(crate) fn publish_adopted(
    ctx: &GitContext,
    board: &Path,
    remote: &str,
    base: Files,
    local: Files,
    remote_commit: &str,
) -> Result<(Report, Files), CliError> {
    let engine = Engine::new(ctx, board);
    (|| -> Result<(Report, Files), String> {
        let to_side = |files: Files| Side {
            oids: files
                .iter()
                .map(|(path, content)| (path.clone(), fake_oid(content)))
                .collect(),
            files,
        };
        let base = to_side(base);
        let local = to_side(local);
        let remote_side = engine.load_tree(remote_commit)?;
        // Blob IDs must be comparable across sides: rehash base/local with the
        // same function used for remote contents.
        let remote_cmp = to_side(remote_side.files.clone());
        let snapshot = Snapshot {
            raw: local.clone(),
            local: local.clone(),
            held: Vec::new(),
            held_paths: Vec::new(),
            restored: 0,
        };
        let merged = engine.merge(&base, &snapshot, &remote_cmp, &[])?;
        let mut report = Report::new(Outcome::Synced);
        let commit = if merged.files == remote_side.files {
            remote_commit.to_string()
        } else {
            let tree = engine.write_tree(&merged.files, &[&remote_side])?;
            let commit = engine.commit(
                &tree,
                &[remote_commit.to_string()],
                "tandem: adopt legacy board changes",
            )?;
            match engine.push(remote, &commit)? {
                Push::Accepted => {}
                Push::Rejected => {
                    return Err(
                        "the remote changed during adoption; run the command again".to_string()
                    )
                }
            }
            report.published = true;
            commit
        };
        engine.update_ref(BASE_REF, &commit)?;
        engine.record_fetch();
        save_conflicts(ctx, &merged.conflicts)?;
        report.renames = merged.renames.clone();
        report.conflicts = summaries(&merged.conflicts);
        report.held = merged.held.clone();
        Ok((report, merged.files))
    })()
    .map_err(CliError::user)
}

/// Stores `files` as the local safety copy so the board can be restored.
pub(crate) fn store_safety_copy(ctx: &GitContext, files: &Files) -> Result<(), CliError> {
    let engine = Engine::new(ctx, &ctx.main_worktree.join(".tandem"));
    (|| -> Result<(), String> {
        let tree = engine.write_tree(files, &[])?;
        let commit = engine.commit(&tree, &[], "tandem: local safety copy")?;
        engine.update_ref(PENDING_REF, &commit)
    })()
    .map_err(CliError::user)
}

/// Whether this clone has synced or snapshotted a current board before.
pub(crate) fn has_local_sync_state(ctx: &GitContext) -> bool {
    [BASE_REF, PENDING_REF].iter().any(|name| {
        git::run(
            &ctx.main_worktree,
            &["rev-parse", "--verify", "--quiet", name],
            GitCall::default(),
        )
        .is_ok_and(|output| output.success())
    })
}

/// Whether the remote is known to host a board (from the last fetch).
pub(crate) fn remote_has_board(ctx: &GitContext) -> bool {
    let Some(remote) = remote_name(ctx) else {
        return false;
    };
    git::run(
        &ctx.main_worktree,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/remotes/{remote}/{BRANCH}"),
        ],
        GitCall::default(),
    )
    .is_ok_and(|output| output.success())
}

pub(crate) fn remote_name(ctx: &GitContext) -> Option<String> {
    if let Ok(name) = git::text(&ctx.main_worktree, &["config", "--get", "tandem.remote"]) {
        if !name.is_empty() {
            return Some(name);
        }
    }
    let remotes = git::text(&ctx.main_worktree, &["remote"]).ok()?;
    let names: Vec<&str> = remotes.lines().collect();
    if names.contains(&"origin") {
        Some("origin".to_string())
    } else {
        names.first().map(|name| name.to_string())
    }
}

pub(crate) fn historical_message(version: &str) -> String {
    format!(
        "the .tandem/ folder holds files from a commit that predates the `{BRANCH}` branch (protocol {version}), so the board shown may be out of date. If you just ran `tandem migrate --adopt`, run `git pull`; if you checked out an older commit, switch back. Your board is safe."
    )
}

/// Whether a board-relative path is synced board content.
pub(crate) fn is_board_path(path: &str) -> bool {
    if path == "tandem.md" {
        return true;
    }
    if !path.contains('/') {
        return path.ends_with(".toml");
    }
    let Some((dir, name)) = path.split_once('/') else {
        return false;
    };
    if name.contains('/') || name.starts_with('.') {
        return false;
    }
    match dir {
        "tasks" | "decisions" | "rules" | "logs" => name.ends_with(".md"),
        "events" => name.ends_with(".jsonl"),
        _ => false,
    }
}

/// Reads a top-level scalar frontmatter field without a YAML parser.
pub(crate) fn top_field(content: &str, key: &str) -> Option<String> {
    let rest = content.strip_prefix("---\n")?;
    for line in rest.lines() {
        if line.trim_end() == "---" {
            break;
        }
        if let Some(value) = line
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
        {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value);
            return Some(value.to_string());
        }
    }
    None
}

/// Reads every synced board file below `board`.
pub(crate) fn read_board_dir(board: &Path) -> Result<Files, String> {
    let mut files = Files::new();
    let mut visit = |relative: String| -> Result<(), String> {
        if is_board_path(&relative) {
            let content = fs::read_to_string(board.join(&relative))
                .map_err(|error| format!("could not read .tandem/{relative}: {error}"))?;
            files.insert(relative, content);
        }
        Ok(())
    };
    let Ok(entries) = fs::read_dir(board) else {
        return Ok(Files::new());
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_dir() {
            for child in fs::read_dir(&path)
                .map_err(|error| error.to_string())?
                .flatten()
            {
                if child.path().is_file() {
                    visit(format!("{name}/{}", child.file_name().to_string_lossy()))?;
                }
            }
        } else {
            visit(name)?;
        }
    }
    Ok(files)
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

fn is_record_path(path: &str) -> bool {
    ["tasks/", "decisions/", "rules/", "logs/"]
        .iter()
        .any(|prefix| path.starts_with(prefix))
        && path.ends_with(".md")
}

/// Merge identity of a file: records by uid, everything else by path.
fn entry_key(path: &str, content: &str) -> String {
    if is_record_path(path) {
        if let Some(uid) = top_field(content, "uid").filter(|uid| !uid.is_empty()) {
            return format!("uid:{uid}");
        }
    }
    format!("path:{path}")
}

fn record_id(path: &str, content: &str) -> String {
    top_field(content, "id").unwrap_or_else(|| {
        Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    })
}

fn uid_index(files: &Files) -> HashMap<String, (String, String)> {
    files
        .iter()
        .filter(|(path, _)| is_record_path(path))
        .filter_map(|(path, content)| {
            let uid = top_field(content, "uid")?;
            Some((uid, (path.clone(), record_id(path, content))))
        })
        .collect()
}

/// Why a locally changed file cannot be published, if it cannot.
fn local_problem(
    path: &str,
    content: &str,
    base_same_path: Option<&str>,
    base_uids: &HashMap<String, (String, String)>,
) -> Option<String> {
    if path == "tandem.md" {
        let (frontmatter, _) = match split_frontmatter(content) {
            Ok(parts) => parts,
            Err(message) => return Some(format!("cannot be parsed: {message}")),
        };
        if let Err(message) = super::parse_frontmatter_fields(&frontmatter) {
            return Some(format!("cannot be parsed: {message}"));
        }
        if top_field(content, "protocolVersion").as_deref() != Some(PROTOCOL_VERSION) {
            return Some(format!("protocolVersion must stay {PROTOCOL_VERSION}"));
        }
        if let Some(base) = base_same_path {
            if top_field(base, "workspaceId") != top_field(content, "workspaceId") {
                return Some("workspaceId cannot change".to_string());
            }
        }
        return None;
    }
    if !is_record_path(path) {
        return None;
    }
    let (frontmatter, _) = match split_frontmatter(content) {
        Ok(parts) => parts,
        Err(message) => return Some(format!("cannot be parsed: {message}")),
    };
    let fields = match super::parse_frontmatter_fields(&frontmatter) {
        Ok(fields) => fields,
        Err(message) => return Some(format!("cannot be parsed: {message}")),
    };
    let stem = Path::new(path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_default();
    let Some(id) = fields.get("id") else {
        return Some("missing `id`".to_string());
    };
    if id != &stem {
        return Some(format!("id `{id}` does not match the file name"));
    }
    let Some(uid) = fields.get("uid").filter(|uid| !uid.is_empty()) else {
        return Some("missing `uid`; create records with tandem commands".to_string());
    };
    if let Some(base) = base_same_path {
        if top_field(base, "uid").as_ref() != Some(uid) {
            return Some("`uid` cannot change".to_string());
        }
    }
    if let Some((_, base_id)) = base_uids.get(uid) {
        if base_id != id && !is_provisional(base_id) {
            return Some(format!("published ID `{base_id}` cannot change"));
        }
    }
    None
}

type Entry = Option<(String, String)>;

/// Merges one record (or file) across base, local, and remote. Returns the
/// output and, on conflict, the reason; a conflict keeps the remote version.
fn merge_entry(
    b: &Entry,
    l: &Entry,
    r: &Entry,
    text_merge: &dyn Fn(&str, &str, &str) -> Option<String>,
) -> (Entry, Option<String>) {
    if l == b || l == r {
        return (r.clone(), None);
    }
    if r == b {
        return (l.clone(), None);
    }
    let (Some(lv), Some(rv)) = (l, r) else {
        let reason = if l.is_none() {
            "deleted here but changed on another machine"
        } else {
            "changed here but deleted on another machine"
        };
        return (r.clone(), Some(reason.to_string()));
    };
    let Some(bv) = b else {
        return (
            r.clone(),
            Some("created differently on two machines".to_string()),
        );
    };
    let local_moved = lv.0 != bv.0;
    let remote_moved = rv.0 != bv.0;
    if local_moved != remote_moved || (local_moved && lv.0 != rv.0) {
        let reason = if local_moved && !remote_moved && lv.0.starts_with("logs/") {
            "archived here but edited on another machine"
        } else if remote_moved && !local_moved && rv.0.starts_with("logs/") {
            "archived on another machine but edited here"
        } else {
            "moved differently on two machines"
        };
        return (r.clone(), Some(reason.to_string()));
    }
    let path = &lv.0;
    let merged = if path.starts_with("events/") {
        merge_ledger(&lv.1, &rv.1)
    } else if path.ends_with(".md") {
        merge_markdown(&bv.1, &lv.1, &rv.1, text_merge)
    } else {
        Merged::Conflict(vec!["content".to_string()])
    };
    match merged {
        Merged::Clean(content) => (Some((path.clone(), content)), None),
        Merged::Conflict(fields) => (
            r.clone(),
            Some(format!(
                "{} changed differently here and on another machine",
                fields.join(", ")
            )),
        ),
    }
}

/// Assigns permanent IDs to provisional records and rewrites every reference.
fn number_provisional(files: &mut Files) -> Vec<(String, String)> {
    let records: Vec<(String, String)> = files
        .iter()
        .filter(|(path, _)| is_record_path(path))
        .map(|(path, content)| (path.clone(), content.clone()))
        .collect();
    if !records.iter().any(|(path, _)| {
        Path::new(path)
            .file_stem()
            .is_some_and(|stem| is_provisional(&stem.to_string_lossy()))
    }) {
        return Vec::new();
    }
    let by_id: HashMap<String, &String> = records
        .iter()
        .map(|(path, content)| (record_id(path, content), content))
        .collect();
    let numbering: Vec<NumberingRecord> = records
        .iter()
        .map(|(path, content)| {
            let id = record_id(path, content);
            let kind = if path.starts_with("rules/") {
                NumberingKind::Rule {
                    category: top_field(content, "category").unwrap_or_default(),
                }
            } else if top_field(content, "type").as_deref() == Some("decision") {
                NumberingKind::Decision
            } else {
                let parent = top_field(content, "parentId");
                let subtask = parent.as_ref().is_some_and(|parent| {
                    by_id.get(parent).is_some_and(|parent| {
                        top_field(parent, "type").as_deref() == Some("task")
                            && top_field(parent, "kind").as_deref() != Some("epic")
                    })
                });
                NumberingKind::Task { parent, subtask }
            };
            NumberingRecord {
                id,
                kind,
                created_at: top_field(content, "createdAt"),
            }
        })
        .collect();
    let renames = assign_sequential_ids(&numbering);
    if renames.is_empty() {
        return renames;
    }
    let mut output = Files::new();
    for (path, content) in std::mem::take(files) {
        let mut content = content;
        let mut path = path;
        for (old, new) in &renames {
            if content.contains(old.as_str()) {
                content = replace_id_token(&content, old, new);
            }
            if let Some((dir, name)) = path.split_once('/') {
                if name == format!("{old}.md") {
                    path = format!("{dir}/{new}.md");
                }
            }
        }
        output.insert(path, content);
    }
    *files = output;
    renames
}

/// Validates the merged board as a whole.
fn validate_board(files: &Files) -> Result<(), String> {
    let mut documents = Vec::new();
    for (path, content) in files {
        let location = if path.starts_with("logs/") {
            DocumentLocation::Logs
        } else if path.starts_with("tasks/") || path.starts_with("decisions/") {
            DocumentLocation::Board
        } else {
            continue;
        };
        if !path.ends_with(".md") {
            continue;
        }
        let document = parse_document(&PathBuf::from(".tandem").join(path), location, content)
            .map_err(|error| error.message)?;
        documents.push(document);
    }
    let hierarchy = ProjectHierarchy::from_documents(documents).map_err(|error| error.message)?;
    hierarchy
        .validate_document_metadata()
        .map_err(|error| error.message)?;
    hierarchy
        .validate_all_task_hierarchies()
        .map_err(|error| error.message)
}

fn changed_keys(base: &Side, local: &Side) -> Vec<(String, String)> {
    local
        .files
        .iter()
        .filter(|(path, content)| base.files.get(*path) != Some(*content))
        .map(|(path, content)| (entry_key(path, content), record_id(path, content)))
        .collect()
}

fn mentions(message: &str, id: &str) -> bool {
    replace_id_token(message, id, "\u{0}") != message
}

fn revert_key(local: &mut Side, base: &Side, key: &str) {
    let local_paths = paths_for_key(&local.files, key);
    for path in local_paths {
        local.files.remove(&path);
        local.oids.remove(&path);
    }
    for path in paths_for_key(&base.files, key) {
        local.files.insert(path.clone(), base.files[&path].clone());
        local.oids.insert(path.clone(), base.oids[&path].clone());
    }
}

fn paths_for_key(files: &Files, key: &str) -> Vec<String> {
    files
        .iter()
        .filter(|(path, content)| entry_key(path, content) == key)
        .map(|(path, _)| path.clone())
        .collect()
}

fn summaries(conflicts: &[Conflict]) -> Vec<ConflictSummary> {
    conflicts
        .iter()
        .map(|conflict| ConflictSummary {
            id: conflict.id.clone(),
            reason: conflict.reason.clone(),
        })
        .collect()
}

fn fake_oid(content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("content:{:016x}:{}", hasher.finish(), content.len())
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    super::write::write_atomic(path, content).map_err(|error| error.message)
}

fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("unknown error")
        .to_string()
}

fn timeout() -> Duration {
    let secs = std::env::var("TANDEM_SYNC_TIMEOUT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

fn hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown host".to_string())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Persistent local state
// ---------------------------------------------------------------------------

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct State {
    #[serde(rename = "lastFetch", default)]
    last_fetch: Option<u64>,
    #[serde(rename = "lastError", default)]
    last_error: Option<String>,
}

impl State {
    fn load(ctx: &GitContext) -> Self {
        fs::read_to_string(ctx.state_dir().join("state.json"))
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save(&self, ctx: &GitContext) {
        let _ = fs::create_dir_all(ctx.state_dir());
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(ctx.state_dir().join("state.json"), content);
        }
    }
}

fn conflicts_dir(ctx: &GitContext) -> PathBuf {
    ctx.state_dir().join("conflicts")
}

fn conflict_path(ctx: &GitContext, key: &str) -> PathBuf {
    let name: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    conflicts_dir(ctx).join(format!("{name}.json"))
}

pub(crate) fn load_conflicts(ctx: &GitContext) -> Vec<Conflict> {
    let Ok(entries) = fs::read_dir(conflicts_dir(ctx)) else {
        return Vec::new();
    };
    let mut conflicts: Vec<Conflict> = entries
        .flatten()
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|content| serde_json::from_str(&content).ok())
        .collect();
    conflicts.sort_by(|a, b| a.id.cmp(&b.id));
    conflicts
}

fn save_conflicts(ctx: &GitContext, conflicts: &[Conflict]) -> Result<(), String> {
    let dir = conflicts_dir(ctx);
    let _ = fs::remove_dir_all(&dir);
    if conflicts.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    for conflict in conflicts {
        let content = serde_json::to_string_pretty(conflict).map_err(|error| error.to_string())?;
        fs::write(conflict_path(ctx, &conflict.key), content).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, content: &str) -> Entry {
        Some((path.to_string(), content.to_string()))
    }

    fn none(_: &str, _: &str, _: &str) -> Option<String> {
        None
    }

    const TASK: &str = "---\nid: task-1\nuid: u1\ntype: task\ntitle: \"T\"\n---\n";

    #[test]
    fn archive_versus_edit_is_a_conflict_that_keeps_the_remote() {
        let edited = TASK.replace("\"T\"", "\"Edited\"");
        let archived = TASK.replace("title", "archivedAt: now\ntitle");
        let (output, conflict) = merge_entry(
            &entry("tasks/task-1.md", TASK),
            &entry("logs/task-1.md", &archived),
            &entry("tasks/task-1.md", &edited),
            &none,
        );
        assert_eq!(output, entry("tasks/task-1.md", &edited));
        assert_eq!(
            conflict.as_deref(),
            Some("archived here but edited on another machine")
        );
    }

    #[test]
    fn provisional_records_are_numbered_and_references_rewritten() {
        let mut files = Files::new();
        files.insert("tasks/task-7.md".into(), TASK.replace("task-1", "task-7"));
        files.insert(
            "tasks/task-new-aaaaaaaa.md".into(),
            "---\nid: task-new-aaaaaaaa\nuid: aaaaaaaa-1\ntype: task\ntitle: \"New\"\nblockers: [\"task-7\"]\n---\n".into(),
        );
        files.insert(
            "tasks/task-new-bbbbbbbb.md".into(),
            "---\nid: task-new-bbbbbbbb\nuid: bbbbbbbb-1\ntype: task\ntitle: \"Child\"\nparentId: task-new-aaaaaaaa\n---\nSee task-new-aaaaaaaa.\n".into(),
        );
        files.insert(
            "events/a.jsonl".into(),
            "{\"id\":\"task-new-aaaaaaaa\"}\n".into(),
        );
        let renames = number_provisional(&mut files);
        assert_eq!(
            renames,
            vec![
                ("task-new-aaaaaaaa".to_string(), "task-8".to_string()),
                ("task-new-bbbbbbbb".to_string(), "task-8-1".to_string())
            ]
        );
        assert!(files["tasks/task-8-1.md"].contains("parentId: task-8\n"));
        assert!(files["tasks/task-8-1.md"].contains("See task-8."));
        assert!(files["tasks/task-8.md"].contains("id: task-8\n"));
        assert_eq!(files["events/a.jsonl"], "{\"id\":\"task-8\"}\n");
        assert!(!files.keys().any(|path| path.contains("-new-")));
    }

    #[test]
    fn board_paths_exclude_editor_and_runtime_files() {
        assert!(is_board_path("tandem.md"));
        assert!(is_board_path("config.toml"));
        assert!(is_board_path("tasks/task-1.md"));
        assert!(is_board_path("events/abc.jsonl"));
        assert!(!is_board_path("actor-id"));
        assert!(!is_board_path("tasks/.task-1.md.swp"));
        assert!(!is_board_path("tasks/task-1.md~"));
        assert!(!is_board_path("tasks/sub/task-1.md"));
    }
}
