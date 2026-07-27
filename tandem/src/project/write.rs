//! Locking, snapshots, and atomic concrete Tandem-project writes.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::project::{display_path, TandemProject};
use crate::CliError;

const MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS: usize = 1000;
static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Serializes cooperative hierarchy snapshots and mutations on config inode.
pub(crate) struct HierarchyLock {
    file: File,
}

impl HierarchyLock {
    pub(crate) fn acquire(project: &TandemProject) -> Result<Self, CliError> {
        let path = project.config_path.clone();
        let file = OpenOptions::new().read(true).open(&path).map_err(|error| {
            CliError::user(format!(
                "Write failure: could not open hierarchy lock {}: {error}",
                display_path(&path)
            ))
        })?;
        file.lock().map_err(|error| {
            CliError::user(format!(
                "Write failure: could not lock hierarchy snapshot {}: {error}",
                display_path(&path)
            ))
        })?;
        Ok(Self { file })
    }
}

impl Drop for HierarchyLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileSnapshot {
    signature: FileSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileSignature {
    len: u64,
    modified: Option<SystemTime>,
}

pub(crate) fn read_file_snapshot(path: &Path) -> Result<(String, FileSnapshot), CliError> {
    let before = file_signature(path)?;
    let content = fs::read_to_string(path)?;
    let after = file_signature(path)?;
    if before != after {
        return Err(CliError::user(format!(
            "Write conflict: {} changed while the command was reading it. No files were updated; rerun the command.",
            display_path(path)
        )));
    }
    Ok((content, FileSnapshot { signature: after }))
}

pub(crate) fn ensure_file_unchanged(path: &Path, expected: &FileSnapshot) -> Result<(), CliError> {
    if file_signature(path)? == expected.signature {
        Ok(())
    } else {
        Err(CliError::user(format!(
            "Write conflict: {} changed while the command was preparing its update. No files were updated; rerun the command.",
            display_path(path)
        )))
    }
}

pub(crate) fn write_atomic(path: &Path, content: &str) -> Result<(), CliError> {
    let temp_path = write_temp_file_for(path, content)?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(CliError::user(format!(
            "Write failure: could not replace {}: {error}",
            display_path(path)
        )));
    }
    Ok(())
}

pub(crate) fn write_new_atomic(path: &Path, content: &str) -> Result<bool, CliError> {
    let temp_path = write_temp_file_for(path, content)?;
    match fs::hard_link(&temp_path, path) {
        Ok(()) => {
            let _ = fs::remove_file(&temp_path);
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temp_path);
            Ok(false)
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(CliError::user(format!(
                "Write failure: could not create {}: {error}",
                display_path(path)
            )))
        }
    }
}

pub(crate) fn create_new_sequential_document_after<F>(
    project: &TandemProject,
    prefix: &str,
    last_allocated: usize,
    mut content_for_id: F,
) -> Result<CreatedDocument, CliError>
where
    F: FnMut(&str) -> String,
{
    let mut next_number = last_allocated.checked_add(1).ok_or_else(|| {
        CliError::user(format!("ID allocation failure: {prefix} sequence overflow"))
    })?;
    for _ in 0..MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS {
        let id = format!("{prefix}-{next_number}");
        let path = project.board_dir.join(format!("{id}.md"));
        if write_new_atomic(&path, &content_for_id(&id))? {
            return Ok(CreatedDocument { id, path });
        }
        next_number = next_number.checked_add(1).ok_or_else(|| {
            CliError::user(format!("ID allocation failure: {prefix} sequence overflow"))
        })?;
    }
    Err(CliError::user(format!(
        "ID allocation failure: could not reserve a new {prefix} document after {MAX_SEQUENTIAL_ID_ALLOCATION_ATTEMPTS} attempts; concurrent writers may be too active, rerun the command"
    )))
}

#[derive(Debug)]
pub(crate) struct CreatedDocument {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn archive_board_document(
    project: &TandemProject,
    board_path: &Path,
    snapshot: &FileSnapshot,
    content: &str,
    action: &str,
) -> Result<PathBuf, CliError> {
    let file_name = board_path.file_name().map(PathBuf::from).ok_or_else(|| {
        CliError::user(format!(
            "cannot determine file name for {}",
            board_path.display()
        ))
    })?;
    let log_path = project.logs_dir.join(file_name);
    if log_path.exists() {
        return Err(CliError::user(format!(
            "Validation failed: log document already exists: {}",
            display_path(&log_path)
        )));
    }
    ensure_file_unchanged(board_path, snapshot)?;
    write_atomic(&log_path, content)?;
    if let Err(error) = fs::remove_file(board_path) {
        let rollback = fs::remove_file(&log_path);
        let rollback_detail = match rollback {
            Ok(()) => "the new log was rolled back".to_string(),
            Err(rollback_error) => format!(
                "the new log could not be rolled back ({rollback_error}); inspect both files"
            ),
        };
        return Err(CliError::user(format!(
            "Write failure: could not remove active document {} after writing {action} log {}: {error}; {rollback_detail}",
            display_path(board_path),
            display_path(&log_path)
        )));
    }
    Ok(log_path)
}

pub(crate) fn file_signature(path: &Path) -> Result<FileSignature, CliError> {
    let metadata = fs::metadata(path)?;
    Ok(FileSignature {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn write_temp_file_for(path: &Path, content: &str) -> Result<PathBuf, CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = temporary_path_for(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|error| {
            CliError::user(format!(
                "Write failure: could not create temp file {} for {}: {error}",
                display_path(&temp_path),
                display_path(path)
            ))
        })?;
    if let Err(error) = file
        .write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
    {
        let _ = fs::remove_file(&temp_path);
        return Err(CliError::user(format!(
            "Write failure: could not write {}: {error}",
            display_path(path)
        )));
    }
    drop(file);
    Ok(temp_path)
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.md");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(
        ".{file_name}.tmp.{}.{}.{}",
        std::process::id(),
        nanos,
        counter
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tandem-project-write-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn stale_snapshot_rejects_replacement_without_overwriting_newer_content() {
        let root = project_root("conflict");
        let path = root.join("document.md");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, "before").unwrap();
        let (_, snapshot) = read_file_snapshot(&path).unwrap();
        fs::write(&path, "newer content").unwrap();
        assert!(ensure_file_unchanged(&path, &snapshot).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "newer content");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_moves_patched_document_to_logs_without_losing_its_body() {
        let root = project_root("archive");
        let data_dir = root.join(".tandem");
        let project =
            TandemProject::with_paths(root.clone(), data_dir.clone(), data_dir.join("tandem.md"));
        fs::create_dir_all(&project.board_dir).unwrap();
        fs::write(&project.config_path, "---\n---\n").unwrap();
        let board_path = project.board_dir.join("task-1.md");
        fs::write(&board_path, "---\nid: task-1\n---\n# retained\n").unwrap();
        let (_, snapshot) = read_file_snapshot(&board_path).unwrap();
        let log_path = archive_board_document(
            &project,
            &board_path,
            &snapshot,
            "---\nid: task-1\ncompletedAt: now\n---\n# retained\n",
            "completed",
        )
        .unwrap();
        assert!(!board_path.exists());
        assert_eq!(
            fs::read_to_string(log_path).unwrap(),
            "---\nid: task-1\ncompletedAt: now\n---\n# retained\n"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
