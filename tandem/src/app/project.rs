//! Application operations for opening, initializing, and upgrading Tandem projects.

use std::env;
use std::path::{Path, PathBuf};

use crate::app::Error;
use crate::project::{display_path, parse_frontmatter_fields, split_frontmatter, TandemProject};
use crate::protocol::config::{default_project_config, PROTOCOL_VERSION};
use crate::protocol::document::normalize_fields;

#[derive(Debug)]
pub(crate) struct InitOptions {
    pub(crate) title: Option<String>,
    pub(crate) force: bool,
}

#[derive(Debug)]
pub(crate) struct InitOutcome {
    pub(crate) title: String,
    pub(crate) project: TandemProject,
}

/// One board opened during the current TUI process.
#[derive(Debug, Clone)]
pub(crate) struct SessionBoard {
    pub(crate) alias: String,
    pub(crate) project: TandemProject,
}

/// Ephemeral set of board handles shared across in-place TUI switches.
#[derive(Debug, Clone)]
pub(crate) struct SessionRoster {
    entries: Vec<SessionBoard>,
}

impl SessionRoster {
    pub(crate) fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub(crate) fn seed(project: TandemProject) -> Self {
        let mut roster = Self::empty();
        roster.insert(project);
        roster
    }

    pub(crate) fn entries(&self) -> &[SessionBoard] {
        &self.entries
    }

    pub(crate) fn add_path(&mut self, path: &Path) -> Result<(SessionBoard, bool), Error> {
        let project = open_at(path)?;
        let root = project.root().to_path_buf();
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.project.root() == root)
        {
            return Ok((self.entries[index].clone(), false));
        }
        self.insert(project);
        Ok((
            self.entries.last().expect("inserted session board").clone(),
            true,
        ))
    }

    pub(crate) fn remove(&mut self, root: &Path, current: &Path) -> bool {
        if root == current {
            return false;
        }
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.project.root() == root)
        else {
            return false;
        };
        self.entries.remove(index);
        true
    }

    fn insert(&mut self, project: TandemProject) {
        let alias = project
            .root()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Workspace")
            .to_string();
        self.entries.push(SessionBoard { alias, project });
    }
}

/// Opens the board for the current directory. A board that an old checkout
/// replaced with historical files opens for reading; see [`historical`].
pub(crate) fn open() -> Result<TandemProject, Error> {
    open_at(&env::current_dir()?)
}

/// Opens the board at a selected path with normal discovery and protocol checks.
/// Canonicalizing the input before discovery makes symlinks and linked worktrees
/// resolve to the same board root for session-roster deduplication.
pub(crate) fn open_at(path: &Path) -> Result<TandemProject, Error> {
    let expanded = if path == Path::new("~") {
        Some(PathBuf::from(
            env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .ok_or_else(|| Error::user("cannot resolve `~` because HOME is not set"))?,
        ))
    } else if let Ok(relative) = path.strip_prefix("~/") {
        Some(
            PathBuf::from(
                env::var_os("HOME")
                    .filter(|home| !home.is_empty())
                    .ok_or_else(|| {
                        Error::user(format!(
                            "cannot resolve `{}` because HOME is not set",
                            path.display()
                        ))
                    })?,
            )
            .join(relative),
        )
    } else {
        None
    };
    let project = TandemProject::discover_from(expanded.as_deref().unwrap_or(path))?;
    ensure_current_protocol(&project)?;
    Ok(project)
}

/// Opens the board for a change. Refuses a historical board.
pub(crate) fn open_for_write() -> Result<TandemProject, Error> {
    let project = open()?;
    if let Some(message) = historical(&project) {
        return Err(Error::user(format!("cannot change the board: {message}")));
    }
    Ok(project)
}

/// Explains why the board currently shows files from an older commit.
pub(crate) fn historical(project: &TandemProject) -> Option<String> {
    let version = protocol_version(project).ok()?;
    (version != PROTOCOL_VERSION).then(|| crate::project::sync::historical_message(&version))
}

pub(crate) fn initialize(options: InitOptions) -> Result<InitOutcome, Error> {
    let cwd = env::current_dir()?;
    let git = crate::project::git::detect(&cwd);
    let root = git
        .as_ref()
        .map(|git| git.main_worktree.clone())
        .unwrap_or(cwd);
    if let Some(git) = &git {
        if crate::project::sync::remote_has_board(git) {
            return Err(Error::user(
                "this repository already has a Tandem board on its `tandem` branch; run any tandem command to download it",
            ));
        }
    }
    let title = match options.title.as_deref() {
        Some(title) => {
            let title = title.trim();
            if title.is_empty() {
                return Err(Error::usage("--title must not be empty"));
            }
            title.to_string()
        }
        None => default_title(&root),
    };
    let data_dir = root.join(".tandem");
    if data_dir.exists() || data_dir.join("tandem.md").exists() {
        let hint = if options.force {
            " --force overwrite is not implemented yet."
        } else {
            ""
        };
        return Err(Error::user(format!(
            "Tandem workspace already exists at {}.{hint}",
            data_dir.display()
        )));
    }
    let workspace_id = crate::app::support::new_uid();
    let project = TandemProject::initialize(&root, &default_project_config(&title, &workspace_id))?;
    if let Some(git) = project.git() {
        crate::project::sync::ensure_excluded(git)?;
    }
    Ok(InitOutcome { title, project })
}

pub(crate) fn protocol_version(project: &TandemProject) -> Result<String, Error> {
    let (frontmatter, _) = split_frontmatter(&project.read_config_raw()?).map_err(|message| {
        Error::user(format!(
            "Parse failure: {}: {message}",
            display_path(&project.config_path)
        ))
    })?;
    let mut fields = parse_frontmatter_fields(&frontmatter).map_err(|message| {
        Error::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(&project.config_path)
        ))
    })?;
    normalize_fields(&mut fields);
    fields.get("protocolVersion").cloned().ok_or_else(|| {
        Error::user(format!(
            "Validation failed for {}: missing required field `protocolVersion`",
            display_path(&project.config_path)
        ))
    })
}

pub(crate) fn ensure_current_protocol(project: &TandemProject) -> Result<(), Error> {
    let version = protocol_version(project)?;
    if version == PROTOCOL_VERSION {
        return Ok(());
    }
    if version == "0.3.0" {
        if let Some(git) = project.git() {
            if crate::project::sync::has_local_sync_state(git) {
                // An old checkout put historical files in place; readable.
                return Ok(());
            }
            let hint = if crate::project::sync::remote_has_board(git) {
                "Another machine already moved this board to the `tandem` branch: run `tandem migrate --adopt` if this checkout has board changes that were never pushed, otherwise just `git pull`."
            } else {
                "Run `tandem migrate` to move it to the repository's `tandem` branch."
            };
            return Err(Error::user(format!(
                "This board uses protocol 0.3.0; this Tandem version requires {PROTOCOL_VERSION}. {hint}"
            )));
        }
    }
    if version == "0.4.0" {
        return Err(Error::user(format!(
            "This board uses protocol 0.4.0; this Tandem version requires {PROTOCOL_VERSION}. Run `tandem migrate` to upgrade it. Every machine that shares this board must install this Tandem version before the board is migrated and synced, because older versions cannot read the new kinds or protocol version."
        )));
    }
    Err(Error::user(format!(
        "Unsupported protocol version `{version}` detected; this Tandem version requires {PROTOCOL_VERSION}."
    )))
}

pub(crate) fn warnings(project: &TandemProject) -> Result<Vec<String>, Error> {
    let config = project.read_config_yaml()?;
    Ok(
        crate::protocol::config::embedded_rules_warning(config.as_ref())
            .into_iter()
            .collect(),
    )
}

pub(crate) fn default_title(root: &std::path::Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Tandem Workspace".to_string())
}
