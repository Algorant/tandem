//! Application operations for opening, initializing, and upgrading Tandem projects.

use std::collections::BTreeMap;
use std::env;

use crate::app::support::workspace_deprecation_warnings;
use crate::project::write::{ensure_file_unchanged, read_file_snapshot};
use crate::project::{
    display_path, parse_frontmatter_fields, patch_frontmatter_content, split_frontmatter,
    write_atomic, TandemProject,
};
use crate::protocol::config::{default_project_config, LEGACY_PROTOCOL_VERSION, PROTOCOL_VERSION};
use crate::protocol::document::normalize_fields;
use crate::CliError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpgradeOutcome {
    AlreadyCurrent,
    Upgraded,
}

pub(crate) fn open() -> Result<TandemProject, CliError> {
    let project = TandemProject::discover()?;
    let _project_root = project.root();
    ensure_current_protocol(&project)?;
    Ok(project)
}

pub(crate) fn discover_unchecked() -> Result<TandemProject, CliError> {
    TandemProject::discover()
}

pub(crate) fn initialize(options: InitOptions) -> Result<InitOutcome, CliError> {
    if let Ok(project) = discover_unchecked() {
        if protocol_version(&project)? == LEGACY_PROTOCOL_VERSION {
            ensure_current_protocol(&project)?;
        }
    }
    let root = env::current_dir()?;
    let title = match options.title.as_deref() {
        Some(title) => {
            let title = title.trim();
            if title.is_empty() {
                return Err(CliError::usage("--title must not be empty"));
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
        return Err(CliError::user(format!(
            "Tandem workspace already exists at {}.{hint}",
            data_dir.display()
        )));
    }
    let project = TandemProject::initialize(&root, &default_project_config(&title))?;
    Ok(InitOutcome { title, project })
}

pub(crate) fn upgrade() -> Result<UpgradeOutcome, CliError> {
    let project = discover_unchecked()?;
    match protocol_version(&project)?.as_str() {
        PROTOCOL_VERSION => Ok(UpgradeOutcome::AlreadyCurrent),
        LEGACY_PROTOCOL_VERSION => {
            let (content, signature) = read_file_snapshot(&project.config_path)?;
            let patched = patch_frontmatter_content(
                &content,
                &BTreeMap::from([("protocolVersion".to_string(), PROTOCOL_VERSION.to_string())]),
                &[],
            )?;
            ensure_file_unchanged(&project.config_path, &signature)?;
            write_atomic(&project.config_path, &patched)?;
            Ok(UpgradeOutcome::Upgraded)
        }
        version => Err(CliError::user(format!(
            "Cannot upgrade unsupported protocol version `{version}`; expected {LEGACY_PROTOCOL_VERSION} or {PROTOCOL_VERSION}."
        ))),
    }
}

pub(crate) fn protocol_version(project: &TandemProject) -> Result<String, CliError> {
    let (frontmatter, _) = split_frontmatter(&project.read_config_raw()?).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {}: {message}",
            display_path(&project.config_path)
        ))
    })?;
    let mut fields = parse_frontmatter_fields(&frontmatter).map_err(|message| {
        CliError::user(format!(
            "Parse failure: {} frontmatter YAML: {message}",
            display_path(&project.config_path)
        ))
    })?;
    normalize_fields(&mut fields);
    fields.get("protocolVersion").cloned().ok_or_else(|| {
        CliError::user(format!(
            "Validation failed for {}: missing required field `protocolVersion`",
            display_path(&project.config_path)
        ))
    })
}

pub(crate) fn ensure_current_protocol(project: &TandemProject) -> Result<(), CliError> {
    match protocol_version(project)?.as_str() {
        PROTOCOL_VERSION => Ok(()),
        LEGACY_PROTOCOL_VERSION => Err(CliError::user(format!(
            "Protocol {LEGACY_PROTOCOL_VERSION} project detected. Run `tandem upgrade` explicitly before using project commands."
        ))),
        version => Err(CliError::user(format!(
            "Unsupported protocol version `{version}`; this Tandem version supports {PROTOCOL_VERSION}."
        ))),
    }
}

pub(crate) fn warnings(project: &TandemProject) -> Result<Vec<String>, CliError> {
    workspace_deprecation_warnings(project)
}

pub(crate) fn default_title(root: &std::path::Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Tandem Workspace".to_string())
}
