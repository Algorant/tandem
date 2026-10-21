//! Existing instruction-file pointers for fresh initialization. Never replace files.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::CliError;

const POINTER: &[u8] =
    b"Read the current project rules with `tandem rules list` before starting work.";

#[derive(Debug, Serialize)]
pub(crate) struct PointerOutcome {
    pub(crate) file: String,
    pub(crate) status: &'static str,
}

#[derive(Debug)]
struct Target {
    path: PathBuf,
    names: Vec<PathBuf>,
    file: File,
    metadata: Metadata,
    bytes: Vec<u8>,
    append: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct PointerPlan {
    targets: Vec<Target>,
    outcomes: Vec<PointerOutcome>,
}

fn same_file(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev() && a.ino() == b.ino()
}

fn changed(path: &Path) -> CliError {
    CliError::user(format!(
        "instruction file changed or disappeared: {}",
        path.display()
    ))
}

impl PointerPlan {
    /// Resolve and inspect both names before any board files are created.
    pub(crate) fn preflight(root: &Path) -> Result<Self, CliError> {
        let mut targets: Vec<Target> = Vec::new();
        let mut outcomes = Vec::new();
        for name in ["AGENTS.md", "CLAUDE.md"] {
            let original = root.join(name);
            let path = match fs::canonicalize(&original) {
                Ok(path) => path,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    outcomes.push(PointerOutcome {
                        file: name.to_string(),
                        status: "absent",
                    });
                    continue;
                }
                Err(error) => {
                    return Err(CliError::user(format!(
                        "cannot resolve {}: {error}",
                        original.display()
                    )))
                }
            };
            let metadata = fs::metadata(&path)?;
            if !metadata.is_file() {
                return Err(CliError::user(format!(
                    "instruction target is not a regular file: {}",
                    original.display()
                )));
            }
            if let Some(target) = targets
                .iter_mut()
                .find(|target| same_file(&metadata, &target.metadata))
            {
                target.names.push(original);
                outcomes.push(PointerOutcome {
                    file: name.to_string(),
                    status: "same-target",
                });
                continue;
            }
            let mut file = File::open(&path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let present = bytes
                .split(|byte| *byte == b'\n')
                .any(|line| line.strip_suffix(b"\r").unwrap_or(line) == POINTER);
            let mut append = Vec::new();
            if !present {
                // Append to the resolved inode in place. Neither open creates a missing file.
                file = OpenOptions::new().read(true).append(true).open(&path)?;
                let newline: &[u8] = if bytes.windows(2).any(|pair| pair == b"\r\n") {
                    b"\r\n"
                } else {
                    b"\n"
                };
                if !bytes.is_empty() && !bytes.ends_with(b"\n") {
                    append.extend_from_slice(newline);
                }
                append.extend_from_slice(POINTER);
                append.extend_from_slice(newline);
            }
            let target = Target {
                path,
                names: vec![original],
                file,
                metadata,
                bytes,
                append,
            };
            target.verify_names()?;
            if !same_file(&target.metadata, &target.file.metadata()?) {
                return Err(changed(&target.path));
            }
            outcomes.push(PointerOutcome {
                file: name.to_string(),
                status: if present { "already-present" } else { "added" },
            });
            targets.push(target);
        }
        let mut plan = Self { targets, outcomes };
        plan.verify()?;
        Ok(plan)
    }

    fn verify(&mut self) -> Result<(), CliError> {
        for target in &mut self.targets {
            target.verify()?;
        }
        Ok(())
    }

    pub(crate) fn apply(mut self) -> Result<Vec<PointerOutcome>, CliError> {
        self.verify().map_err(|error| {
            CliError::user(format!(
                "No instruction pointers written: {}",
                error.message
            ))
        })?;
        let mut completed = Vec::new();
        for target in &mut self.targets {
            let result = (|| -> Result<(), CliError> {
                target.verify()?;
                if !target.append.is_empty() {
                    target.file.write_all(&target.append)?;
                    target.file.flush()?;
                    target.verify_names()?;
                    let mut expected = target.bytes.clone();
                    expected.extend_from_slice(&target.append);
                    target.file.seek(SeekFrom::Start(0))?;
                    let mut actual = Vec::new();
                    target.file.read_to_end(&mut actual)?;
                    if actual != expected
                        || target.file.metadata()?.mode() != target.metadata.mode()
                    {
                        return Err(changed(&target.path));
                    }
                }
                Ok(())
            })();
            result.map_err(|error| CliError::user(format!(
                "instruction pointer failed at {} (this target may be partially written); completed targets: {:?}; {}",
                target.path.display(), completed, error.message
            )))?;
            completed.push(target.path.display().to_string());
        }
        Ok(self.outcomes)
    }
}

impl Target {
    fn verify(&mut self) -> Result<(), CliError> {
        self.verify_names()?;
        let current = self.file.metadata()?;
        if !same_file(&self.metadata, &current)
            || self.metadata.len() != current.len()
            || self.metadata.modified()? != current.modified()?
            || self.metadata.mode() != current.mode()
        {
            return Err(changed(&self.path));
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        self.file.read_to_end(&mut bytes)?;
        if bytes != self.bytes {
            return Err(changed(&self.path));
        }
        Ok(())
    }

    fn verify_names(&self) -> Result<(), CliError> {
        for name in &self.names {
            let resolved = fs::canonicalize(name).map_err(|_| changed(name))?;
            let metadata = fs::metadata(&resolved).map_err(|_| changed(name))?;
            if !metadata.is_file() || !same_file(&self.metadata, &metadata) {
                return Err(changed(name));
            }
        }
        Ok(())
    }
}
