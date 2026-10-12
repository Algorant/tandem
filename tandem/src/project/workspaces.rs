//! The user-level, hand-written list of Tandem workspaces.
//!
//! `workspaces.toml` lives beside the user's other Tandem config
//! (`$XDG_CONFIG_HOME/tandem/` or `~/.config/tandem/`). It is machine-local:
//! paths never enter a synced board. Tandem only reads it. Opening the TUI or
//! the web interface never adds, rewrites, or discovers entries.
//!
//! ```toml
//! [[workspace]]
//! alias = "notes"
//! path = "~/Projects/notes"
//! ```
//!
//! Loading never fails as a whole because of one bad entry: every problem is a
//! per-entry diagnostic, and the entries that are valid stay usable.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

/// File name of the list inside the user's Tandem config directory.
pub(crate) const WORKSPACE_LIST_FILE: &str = "workspaces.toml";

/// Keys an entry may carry; anything else is reported, never ignored.
const ENTRY_KEYS: [&str; 2] = ["alias", "path"];

/// One valid entry: an alias and the canonical workspace root it resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceEntry {
    pub(crate) alias: String,
    /// Canonical directory that directly contains the workspace's board.
    pub(crate) root: PathBuf,
}

/// The loaded list: valid entries in file order plus a diagnostic for every
/// problem found. Diagnostics name the file and entry so they can be shown as
/// they are.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceList {
    pub(crate) source: Option<PathBuf>,
    pub(crate) entries: Vec<WorkspaceEntry>,
    pub(crate) diagnostics: Vec<String>,
}

// The TUI reports `diagnostics` today; `source` and `resolve` are consumed by
// the switcher and web tasks that build on this loader.
#[allow(dead_code)]
impl WorkspaceList {
    /// Resolves an alias to its entry. Aliases are matched case-insensitively,
    /// the same way duplicates are detected.
    pub(crate) fn resolve(&self, alias: &str) -> Option<&WorkspaceEntry> {
        self.entries
            .iter()
            .find(|entry| entry.alias.eq_ignore_ascii_case(alias))
    }
}

/// Location of the user's workspace list, `None` when neither
/// `XDG_CONFIG_HOME` nor `HOME` is set.
pub(crate) fn workspace_list_path() -> Option<PathBuf> {
    super::user_config_dir().map(|dir| dir.join(WORKSPACE_LIST_FILE))
}

/// Loads the list from the user's config directory. A missing file is an empty
/// list, not a problem.
pub(crate) fn load() -> WorkspaceList {
    match workspace_list_path() {
        Some(path) => load_from(&path),
        None => WorkspaceList::default(),
    }
}

/// Loads the list from an explicit file.
pub(crate) fn load_from(path: &Path) -> WorkspaceList {
    let mut list = WorkspaceList {
        source: Some(path.to_path_buf()),
        ..WorkspaceList::default()
    };
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return list,
        Err(error) => {
            list.diagnostics.push(format!(
                "Workspace list {}: could not read it: {error}",
                path.display()
            ));
            return list;
        }
    };
    let table = match content.parse::<Table>() {
        Ok(table) => table,
        Err(error) => {
            list.diagnostics.push(format!(
                "Workspace list {}: invalid TOML: {}",
                path.display(),
                error.to_string().trim_end()
            ));
            return list;
        }
    };
    let file = path.display().to_string();
    for key in table.keys().filter(|key| key.as_str() != "workspace") {
        list.diagnostics.push(format!(
            "Workspace list {file}: unknown top-level key `{key}`; entries are `[[workspace]]` tables"
        ));
    }
    let entries = match table.get("workspace") {
        None => return list,
        Some(Value::Array(entries)) => entries,
        Some(_) => {
            list.diagnostics.push(format!(
                "Workspace list {file}: `workspace` must be an array of `[[workspace]]` tables"
            ));
            return list;
        }
    };
    // First occurrence wins; its position is kept for the message that
    // rejects a later duplicate.
    let mut aliases: BTreeMap<String, usize> = BTreeMap::new();
    let mut roots: BTreeMap<PathBuf, (usize, String)> = BTreeMap::new();
    for (index, value) in entries.iter().enumerate() {
        let number = index + 1;
        match read_entry(value) {
            Ok(entry) => {
                let label = format!("Workspace list {file}: entry {number} (`{}`)", entry.alias);
                if let Some(first) = aliases.get(&entry.alias.to_ascii_lowercase()) {
                    list.diagnostics.push(format!(
                        "{label}: duplicate alias; entry {first} already uses it"
                    ));
                } else if let Some((first, first_alias)) = roots.get(&entry.root) {
                    list.diagnostics.push(format!(
                        "{label}: duplicate workspace; entry {first} (`{first_alias}`) already resolves to {}",
                        entry.root.display()
                    ));
                } else {
                    aliases.insert(entry.alias.to_ascii_lowercase(), number);
                    roots.insert(entry.root.clone(), (number, entry.alias.clone()));
                    list.entries.push(entry);
                }
            }
            Err((alias, message)) => {
                let who = alias
                    .map(|alias| format!(" (`{alias}`)"))
                    .unwrap_or_default();
                list.diagnostics.push(format!(
                    "Workspace list {file}: entry {number}{who}: {message}"
                ));
            }
        }
    }
    list
}

/// Validates one entry. The error carries the alias when it could be read, so
/// the diagnostic can name the entry.
fn read_entry(value: &Value) -> Result<WorkspaceEntry, (Option<String>, String)> {
    let Value::Table(table) = value else {
        return Err((None, "must be a `[[workspace]]` table".to_string()));
    };
    let alias = read_alias(table).map_err(|message| (None, message))?;
    let fail = |message: String| (Some(alias.clone()), message);
    if let Some(key) = table.keys().find(|key| !ENTRY_KEYS.contains(&key.as_str())) {
        return Err(fail(format!(
            "unknown key `{key}`; an entry has only `alias` and `path`"
        )));
    }
    let path = match table.get("path") {
        None => return Err(fail("missing `path`".to_string())),
        Some(Value::String(path)) if !path.trim().is_empty() => path.trim(),
        Some(_) => return Err(fail("`path` must be a non-empty string".to_string())),
    };
    let root = resolve_root(path).map_err(fail)?;
    Ok(WorkspaceEntry { alias, root })
}

fn read_alias(table: &Table) -> Result<String, String> {
    let alias = match table.get("alias") {
        None => return Err("missing `alias`".to_string()),
        Some(Value::String(alias)) => alias.trim(),
        Some(_) => return Err("`alias` must be a string".to_string()),
    };
    let mut chars = alias.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !valid {
        return Err(format!(
            "invalid alias `{alias}`: use letters, digits, `.`, `_` or `-`, starting with a letter or digit"
        ));
    }
    Ok(alias.to_string())
}

/// Expands a leading `~`, requires an absolute path, and returns the canonical
/// directory when it directly contains a Tandem board.
fn resolve_root(path: &str) -> Result<PathBuf, String> {
    let expanded = if path == "~" || path.starts_with("~/") {
        let home = env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .ok_or_else(|| format!("`{path}` needs HOME, which is not set"))?;
        Path::new(&home).join(path[1..].trim_start_matches('/'))
    } else {
        PathBuf::from(path)
    };
    if !expanded.is_absolute() {
        return Err(format!("path `{path}` must be absolute or start with `~/`"));
    }
    let root = match fs::canonicalize(&expanded) {
        Ok(root) => root,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(format!("path {} does not exist", expanded.display()))
        }
        Err(error) => return Err(format!("could not resolve {}: {error}", expanded.display())),
    };
    if !root.is_dir() {
        return Err(format!("path {} is not a directory", root.display()));
    }
    if !root.join(".tandem").join("tandem.md").is_file() && !root.join("tandem.md").is_file() {
        return Err(format!(
            "{} is not a workspace root: no `.tandem/tandem.md` there (list the directory that contains the board, not a subdirectory or linked worktree)",
            root.display()
        ));
    }
    Ok(root)
}
