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
            let (config_content, config_signature) = read_file_snapshot(&project.config_path)?;
            let patched_config = patch_frontmatter_content(
                &config_content,
                &BTreeMap::from([("protocolVersion".to_string(), PROTOCOL_VERSION.to_string())]),
                &[],
            )?;

            // Protocol 0.2 validates priority vocabulary strictly. Prepare all
            // recognized legacy aliases before writing anything, and patch the
            // config last so a failed document write remains safely retryable
            // as an explicit 0.1 -> 0.2 upgrade.
            let mut document_patches = Vec::new();
            for document in project.read_documents()? {
                if !matches!(document.field("priority"), Some("med" | "normal")) {
                    continue;
                }
                let (content, signature) = read_file_snapshot(&document.path)?;
                let legacy_priority = document
                    .field("priority")
                    .expect("legacy priority was matched above");
                let patched = patch_legacy_priority(&content, legacy_priority)?;
                document_patches.push((document.path.clone(), signature, patched));
            }

            ensure_file_unchanged(&project.config_path, &config_signature)?;
            for (path, signature, _) in &document_patches {
                ensure_file_unchanged(path, signature)?;
            }
            for (path, _, patched) in document_patches {
                write_atomic(&path, &patched)?;
            }
            write_atomic(&project.config_path, &patched_config)?;
            Ok(UpgradeOutcome::Upgraded)
        }
        version => Err(CliError::user(format!(
            "Cannot upgrade unsupported protocol version `{version}`; expected {LEGACY_PROTOCOL_VERSION} or {PROTOCOL_VERSION}."
        ))),
    }
}

fn patch_legacy_priority(content: &str, legacy_priority: &str) -> Result<String, CliError> {
    let (frontmatter, _) = split_frontmatter(content).map_err(CliError::user)?;
    let frontmatter_start = content
        .find(&frontmatter)
        .expect("split frontmatter is a slice of source content");
    let mut offset = frontmatter_start;
    for line in frontmatter.split_inclusive('\n') {
        let source_line = line.trim_end_matches(['\n', '\r']);
        if crate::project::frontmatter_line_key(source_line) == Some("priority") {
            let value_start = source_line.find(':').expect("frontmatter key has colon") + 1;
            if let Some(relative) = source_line[value_start..].find(legacy_priority) {
                let start = offset + value_start + relative;
                let end = start + legacy_priority.len();
                let mut patched = content.to_string();
                patched.replace_range(start..end, "medium");
                return Ok(patched);
            }
        }
        offset += line.len();
    }
    Err(CliError::user(
        "Upgrade failed: legacy priority could not be patched safely.",
    ))
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
