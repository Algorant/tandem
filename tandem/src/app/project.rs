//! Application operations for opening, initializing, and upgrading Tandem projects.

use std::env;

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

/// Opens the board. A board that an old checkout replaced with historical
/// files opens for reading; see [`historical`].
pub(crate) fn open() -> Result<TandemProject, Error> {
    let project = TandemProject::discover()?;
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
