//! Application operations for opening, initializing, and upgrading Tandem projects.

use std::env;

use crate::app::Error;
use crate::project::{
    display_path, parse_frontmatter_fields, split_frontmatter, CheckpointOutcome, TandemProject,
};
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

pub(crate) fn open() -> Result<TandemProject, Error> {
    let project = TandemProject::discover()?;
    let _project_root = project.root();
    ensure_current_protocol(&project)?;
    Ok(project)
}

pub(crate) fn initialize(options: InitOptions) -> Result<InitOutcome, Error> {
    let root = env::current_dir()?;
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
    let project = TandemProject::initialize(&root, &default_project_config(&title))?;
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
    match protocol_version(project)?.as_str() {
        PROTOCOL_VERSION => Ok(()),
        version => Err(Error::user(format!(
            "Unsupported protocol version `{version}` detected; this Tandem version requires {PROTOCOL_VERSION}."
        ))),
    }
}

pub(crate) fn warnings(project: &TandemProject) -> Result<Vec<String>, Error> {
    let config = project.read_config_yaml()?;
    Ok(
        crate::protocol::config::embedded_rules_warning(config.as_ref())
            .into_iter()
            .collect(),
    )
}

/// Explicit native Git flush for commit/push workflows and adapters.
///
/// This is the authoritative integration point for the existing checkpointer:
/// it stages and commits only the owning `.tandem` path without role gating and
/// without writing or transitioning any Task, Accord, Rule, Decision, or event.
pub(crate) fn checkpoint(project: &TandemProject) -> CheckpointOutcome {
    crate::project::checkpoint(project)
}

pub(crate) fn consolidate_checkpoint(
    project: &TandemProject,
) -> Result<crate::project::Consolidation, String> {
    crate::project::consolidate_checkpoint(project)
}

pub(crate) fn default_title(root: &std::path::Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Tandem Workspace".to_string())
}
