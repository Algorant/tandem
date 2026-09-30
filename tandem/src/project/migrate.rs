//! One-time move of a protocol 0.3.0 board from source commits to the
//! repository's `tandem` branch.
//!
//! `migrate` runs once, on one machine: it gives every record a permanent
//! `uid`, publishes the board as the first commit of the `tandem` branch, and
//! creates one source commit that stops tracking `.tandem/`. `adopt` runs on
//! any other machine that still has unpushed or uncommitted 0.3.0 board
//! changes: it merges them into the shared board (renumbering records that
//! were never published) before that machine pulls the source commit.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::git::{self, GitCall, GitContext};
use super::sync::{self, top_field, Files, Report};
use crate::protocol::config::PROTOCOL_VERSION;
use crate::protocol::ids::{provisional_id, replace_id_token};
use crate::CliError;

const LEGACY_VERSION: &str = "0.3.0";
const SOURCE_COMMIT_MESSAGE: &str = "chore(tandem): move the Tandem board to the tandem branch";

#[derive(Debug)]
pub(crate) struct MigrateReport {
    pub(crate) dry_run: bool,
    pub(crate) remote: String,
    pub(crate) records: usize,
    pub(crate) files: usize,
    pub(crate) migrated_from: String,
    pub(crate) source_commit: Option<String>,
}

#[derive(Debug)]
pub(crate) struct AdoptReport {
    pub(crate) dry_run: bool,
    pub(crate) remote: String,
    /// Records that only existed on this machine, by their old ID.
    pub(crate) new_records: Vec<String>,
    /// Records this machine changed that also exist on the shared board.
    pub(crate) changed_records: Vec<String>,
    pub(crate) sync: Option<Report>,
    /// Old legacy ID to final ID for records that were renumbered.
    pub(crate) renumbered: Vec<(String, String)>,
    pub(crate) unpushed_commits: Vec<String>,
    pub(crate) needs_pull: bool,
}

struct Legacy {
    ctx: GitContext,
    board: PathBuf,
    remote: String,
}

fn legacy(start: &Path) -> Result<Legacy, CliError> {
    let ctx = git::detect(start)
        .ok_or_else(|| CliError::user("tandem migrate must run inside a Git repository"))?;
    let board = ctx.main_worktree.join(".tandem");
    let remote = sync::remote_name(&ctx).ok_or_else(|| {
        CliError::user("tandem migrate needs a Git remote to host the `tandem` branch")
    })?;
    Ok(Legacy { ctx, board, remote })
}

fn git_text(ctx: &GitContext, args: &[&str]) -> Result<String, CliError> {
    git::text(&ctx.main_worktree, args).map_err(CliError::user)
}

fn git_ok(ctx: &GitContext, args: &[&str]) -> Result<bool, CliError> {
    Ok(git::run(&ctx.main_worktree, args, GitCall::default())
        .map_err(CliError::user)?
        .success())
}

fn fetch_source(ctx: &GitContext, remote: &str) -> Result<(), CliError> {
    let output = git::run(
        &ctx.main_worktree,
        &["fetch", "--quiet", remote],
        GitCall {
            timeout: Some(std::time::Duration::from_secs(60)),
            ..Default::default()
        },
    )
    .map_err(CliError::user)?;
    if output.success() {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "could not fetch from `{remote}`: {}",
            output.stderr.trim()
        )))
    }
}

/// Converts a 0.3.0 record by inserting its permanent `uid` after `id`.
pub(crate) fn with_uid(content: &str, uid: &str) -> String {
    if top_field(content, "uid").is_some() {
        return content.to_string();
    }
    let mut output = String::with_capacity(content.len() + 48);
    let mut inserted = false;
    for line in content.split_inclusive('\n') {
        output.push_str(line);
        if !inserted && line.starts_with("id:") {
            output.push_str(&format!("uid: {uid}\n"));
            inserted = true;
        }
    }
    output
}

/// Converts a 0.3.0 `tandem.md` to the current protocol.
pub(crate) fn convert_config(content: &str, workspace_id: &str, migrated_from: &str) -> String {
    let mut output = String::with_capacity(content.len() + 128);
    let mut done = false;
    for line in content.split_inclusive('\n') {
        if !done && line.starts_with("protocolVersion:") {
            output.push_str(&format!(
                "protocolVersion: {PROTOCOL_VERSION}\nworkspaceId: {workspace_id}\nmigratedFrom: {migrated_from}\n"
            ));
            done = true;
        } else {
            output.push_str(line);
        }
    }
    output
}

fn is_record(path: &str) -> bool {
    ["tasks/", "decisions/", "rules/", "logs/"]
        .iter()
        .any(|prefix| path.starts_with(prefix))
        && path.ends_with(".md")
}

fn record_id(path: &str, content: &str) -> String {
    top_field(content, "id").unwrap_or_else(|| {
        Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default()
    })
}

fn new_uid() -> String {
    uuid::Uuid::new_v4().hyphenated().to_string()
}

fn legacy_version(board: &Path) -> Result<String, CliError> {
    let config = fs::read_to_string(board.join("tandem.md"))
        .map_err(|error| CliError::user(format!("could not read .tandem/tandem.md: {error}")))?;
    Ok(top_field(&config, "protocolVersion").unwrap_or_default())
}

/// Moves the checkout's actor identity out of the board into its Git
/// directory, keeping its event ledger continuous.
fn move_actor_id(ctx: &GitContext, board: &Path) -> Result<(), CliError> {
    let old = board.join("actor-id");
    let new = ctx.git_dir.join("tandem-actor-id");
    if old.is_file() && !new.exists() {
        fs::copy(&old, &new)?;
    }
    if old.is_file() {
        fs::remove_file(&old)?;
    }
    Ok(())
}

pub(crate) fn migrate(start: &Path, dry_run: bool) -> Result<MigrateReport, CliError> {
    let Legacy { ctx, board, remote } = legacy(start)?;
    match legacy_version(&board)?.as_str() {
        LEGACY_VERSION => {}
        PROTOCOL_VERSION => return Err(CliError::user("this board is already migrated")),
        other => {
            return Err(CliError::user(format!(
                "tandem migrate converts protocol {LEGACY_VERSION} boards; found `{other}`"
            )))
        }
    }
    if sync::fetch_remote_board(&ctx, &remote)?.is_some() {
        return Err(CliError::user(format!(
            "`{remote}` already has a `tandem` branch: another machine migrated this repository. Run `tandem migrate --adopt` here instead."
        )));
    }
    fetch_source(&ctx, &remote)?;
    let migrated_from = git_text(&ctx, &["rev-parse", "--verify", "@{upstream}"]).map_err(|_| {
        CliError::user("the current branch needs an upstream so other machines can find the migration point")
    })?;
    if !git_ok(
        &ctx,
        &["merge-base", "--is-ancestor", "@{upstream}", "HEAD"],
    )? {
        return Err(CliError::user(
            "this branch is behind its upstream; run `git pull` first so no board changes are left out",
        ));
    }
    if !git_ok(&ctx, &["diff", "--cached", "--quiet"])? {
        return Err(CliError::user(
            "you have staged changes; commit or unstage them first, because migration creates one source commit",
        ));
    }
    let legacy_files = sync::read_board_dir(&board).map_err(CliError::user)?;
    let workspace_id = new_uid();
    let mut converted = Files::new();
    let mut records = 0;
    for (path, content) in &legacy_files {
        let content = if path == "tandem.md" {
            convert_config(content, &workspace_id, &migrated_from)
        } else if is_record(path) {
            records += 1;
            with_uid(content, &new_uid())
        } else {
            content.clone()
        };
        converted.insert(path.clone(), content);
    }
    let mut report = MigrateReport {
        dry_run,
        remote: remote.clone(),
        records,
        files: converted.len(),
        migrated_from,
        source_commit: None,
    };
    if dry_run {
        return Ok(report);
    }
    for (path, content) in &converted {
        super::write::write_atomic(&board.join(path), content)?;
    }
    if let Err(error) = sync::publish_initial(&ctx, &board, &remote) {
        for (path, content) in &legacy_files {
            super::write::write_atomic(&board.join(path), content)?;
        }
        return Err(error);
    }
    move_actor_id(&ctx, &board)?;
    // One source commit: stop tracking .tandem/ (the folder stays) and ignore it.
    git_text(&ctx, &["rm", "-r", "-q", "--cached", "--", ".tandem"])?;
    let gitignore = ctx.main_worktree.join(".gitignore");
    let mut ignore = fs::read_to_string(&gitignore).unwrap_or_default();
    if !ignore
        .lines()
        .any(|line| matches!(line.trim(), "/.tandem/" | ".tandem/" | ".tandem"))
    {
        if !ignore.is_empty() && !ignore.ends_with('\n') {
            ignore.push('\n');
        }
        ignore.push_str("/.tandem/\n");
        fs::write(&gitignore, ignore)?;
    }
    git_text(&ctx, &["add", "--", ".gitignore"])?;
    git_text(
        &ctx,
        &[
            "commit",
            "--quiet",
            "--no-verify",
            "-m",
            SOURCE_COMMIT_MESSAGE,
        ],
    )?;
    report.source_commit = Some(git_text(&ctx, &["rev-parse", "--short", "HEAD"])?);
    Ok(report)
}

pub(crate) fn adopt(start: &Path, dry_run: bool) -> Result<AdoptReport, CliError> {
    let Legacy { ctx, board, remote } = legacy(start)?;
    let config_present = board.join("tandem.md").is_file();
    if config_present && legacy_version(&board)? == PROTOCOL_VERSION {
        return Err(CliError::user(
            "this board already uses the tandem branch; nothing to adopt",
        ));
    }
    let (remote_commit, remote_files) =
        sync::fetch_remote_board(&ctx, &remote)?.ok_or_else(|| {
            CliError::user(format!(
                "`{remote}` has no `tandem` branch yet; run `tandem migrate` on the first machine"
            ))
        })?;
    let remote_config = remote_files
        .get("tandem.md")
        .ok_or_else(|| CliError::user("the shared board has no tandem.md"))?;
    let workspace_id = top_field(remote_config, "workspaceId").unwrap_or_default();
    let migrated_from = top_field(remote_config, "migratedFrom").ok_or_else(|| {
        CliError::user("the shared board does not record where it was migrated from")
    })?;
    fetch_source(&ctx, &remote)?;
    let merge_base = git_text(&ctx, &["merge-base", "HEAD", &migrated_from]).map_err(|_| {
        CliError::user(format!(
            "could not relate this checkout to the migration commit {migrated_from}"
        ))
    })?;
    let legacy_base = sync::source_board_files(&ctx, &merge_base)?;
    let legacy_local = if config_present {
        sync::read_board_dir(&board).map_err(CliError::user)?
    } else {
        // Already pulled: only untracked leftovers remain on disk.
        let mut files = legacy_base.clone();
        files.extend(sync::read_board_dir(&board).map_err(CliError::user)?);
        files
    };

    // Permanent uids: shared records take the shared board's uid.
    let mut uids: HashMap<String, String> = remote_files
        .iter()
        .filter(|(path, _)| is_record(path))
        .filter_map(|(path, content)| Some((record_id(path, content), top_field(content, "uid")?)))
        .collect();
    let base_ids: Vec<String> = legacy_base
        .iter()
        .filter(|(path, _)| is_record(path))
        .map(|(path, content)| record_id(path, content))
        .collect();
    let mut new_records = Vec::new();
    let mut changed_records = Vec::new();
    let mut handles: Vec<(String, String)> = Vec::new();
    for (path, content) in &legacy_local {
        if !is_record(path) {
            continue;
        }
        let id = record_id(path, content);
        if !base_ids.contains(&id) {
            let uid = new_uid();
            let prefix = if path.starts_with("rules/") {
                top_field(content, "category").unwrap_or_else(|| "rule".to_string())
            } else if path.starts_with("decisions/") {
                "decision".to_string()
            } else {
                "task".to_string()
            };
            handles.push((id.clone(), provisional_id(&prefix, &uid)));
            uids.insert(format!("new:{id}"), uid);
            new_records.push(id);
        } else if legacy_base.get(path) != Some(content) {
            changed_records.push(id);
        }
    }
    for (path, content) in &legacy_base {
        if is_record(path)
            && !legacy_local.contains_key(path)
            && !legacy_local
                .iter()
                .any(|(local_path, local)| record_id(local_path, local) == record_id(path, content))
        {
            changed_records.push(record_id(path, content));
        }
    }
    let mut report = AdoptReport {
        dry_run,
        remote: remote.clone(),
        new_records: new_records.clone(),
        changed_records,
        sync: None,
        renumbered: Vec::new(),
        unpushed_commits: Vec::new(),
        needs_pull: false,
    };
    if config_present {
        let log = git_text(
            &ctx,
            &[
                "log",
                "--format=%h %s",
                "@{upstream}..HEAD",
                "--",
                ".tandem",
            ],
        )
        .unwrap_or_default();
        report.unpushed_commits = log.lines().map(str::to_string).collect();
    }
    if dry_run {
        return Ok(report);
    }

    let convert = |files: &Files, local: bool, uids: &mut HashMap<String, String>| -> Files {
        let mut output = Files::new();
        for (path, content) in files {
            if path == "tandem.md" {
                output.insert(
                    path.clone(),
                    convert_config(content, &workspace_id, &migrated_from),
                );
                continue;
            }
            let mut path = path.clone();
            let mut content = content.clone();
            if is_record(&path) {
                let id = record_id(&path, &content);
                let uid = if local && new_records.contains(&id) {
                    uids[&format!("new:{id}")].clone()
                } else {
                    uids.entry(id.clone()).or_insert_with(new_uid).clone()
                };
                content = with_uid(&content, &uid);
            }
            if local {
                for (old, handle) in &handles {
                    content = replace_id_token(&content, old, handle);
                    if let Some((dir, name)) = path.split_once('/') {
                        if name == format!("{old}.md") {
                            path = format!("{dir}/{handle}.md");
                        }
                    }
                }
            }
            output.insert(path, content);
        }
        output
    };
    let base = convert(&legacy_base, false, &mut uids);
    let local = convert(&legacy_local, true, &mut uids);
    let (sync_report, merged) =
        sync::publish_adopted(&ctx, &board, &remote, base, local, &remote_commit)?;
    report.renumbered = handles
        .iter()
        .map(|(old, handle)| (old.clone(), sync_report.renamed(handle)))
        .collect();
    report.sync = Some(sync_report);
    sync::store_safety_copy(&ctx, &merged)?;
    move_actor_id(&ctx, &board)?;
    let tracked = git_ok(
        &ctx,
        &["ls-files", "--error-unmatch", "--", ".tandem/tandem.md"],
    )?;
    if tracked {
        // Leave the tracked legacy files exactly as committed so `git pull`
        // can remove them; the adopted board returns from the safety copy.
        git_text(&ctx, &["checkout", "HEAD", "--", ".tandem"])?;
        git_text(&ctx, &["clean", "-f", "-q", "--", ".tandem"])?;
        report.needs_pull = true;
    } else {
        for path in sync::read_board_dir(&board).map_err(CliError::user)?.keys() {
            if !merged.contains_key(path) {
                fs::remove_file(board.join(path))?;
            }
        }
        for (path, content) in &merged {
            super::write::write_atomic(&board.join(path), content)?;
        }
        sync::ensure_excluded(&ctx)?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_inserts_uid_and_config_identity_deterministically() {
        let record = "---\nid: task-3\ntype: task\ntitle: \"T\"\n---\nBody\n";
        let converted = with_uid(record, "u-1");
        assert_eq!(
            converted,
            "---\nid: task-3\nuid: u-1\ntype: task\ntitle: \"T\"\n---\nBody\n"
        );
        assert_eq!(with_uid(&converted, "other"), converted);
        let config = "---\nprotocolVersion: 0.3.0\ntype: workspace\n---\n";
        assert_eq!(
            convert_config(config, "ws", "abc"),
            format!("---\nprotocolVersion: {PROTOCOL_VERSION}\nworkspaceId: ws\nmigratedFrom: abc\ntype: workspace\n---\n")
        );
    }
}
