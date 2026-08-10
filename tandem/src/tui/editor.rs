use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::project::StoredDocument as Document;
use crate::protocol::hierarchy::DocumentLocation;
use crate::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EditorTarget {
    pub(super) id: String,
    pub(super) path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EditorCommand {
    program: String,
    args: Vec<String>,
    pub(super) source: &'static str,
}

impl EditorCommand {
    pub(super) fn from_value(value: &str, source: &'static str) -> Result<Self, CliError> {
        let words = split_editor_command(value).map_err(|message| {
            CliError::user(format!(
                "could not parse {source} value `{value}`: {message}"
            ))
        })?;
        let Some((program, args)) = words.split_first() else {
            return Err(CliError::user(format!("{source} is empty")));
        };
        Ok(Self {
            program: program.clone(),
            args: args.to_vec(),
            source,
        })
    }

    pub(super) fn display_label(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

pub(super) fn editor_target_for_doc(doc: &Document) -> Result<EditorTarget, String> {
    if doc.location != DocumentLocation::Board {
        return Err("Only active board documents are editable in $EDITOR for now.".to_string());
    }
    if doc.doc_type() != "task" {
        return Err(format!(
            "Only active task documents open in $EDITOR for now; {} is type `{}` and is deferred.",
            doc.id(),
            doc.doc_type()
        ));
    }
    Ok(EditorTarget {
        id: doc.id().to_string(),
        path: doc.path.clone(),
    })
}

pub(super) fn editor_command_from_env() -> Result<EditorCommand, CliError> {
    for (name, source) in [("EDITOR", "$EDITOR"), ("VISUAL", "$VISUAL")] {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                return EditorCommand::from_value(&value, source);
            }
        }
    }

    EditorCommand::from_value(default_editor_program(), "default editor")
}

fn default_editor_program() -> &'static str {
    if cfg!(windows) {
        "notepad"
    } else {
        "vi"
    }
}

pub(super) fn run_editor_command(command: &EditorCommand, path: &Path) -> io::Result<ExitStatus> {
    Command::new(&command.program)
        .args(&command.args)
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub(super) fn split_editor_command(value: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut word_started = false;

    for ch in value.chars() {
        if escaped {
            current.push(ch);
            word_started = true;
            escaped = false;
            continue;
        }

        if ch == '\\' && quote != Some('\'') {
            escaped = true;
            word_started = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                current.push(ch);
            }
            word_started = true;
            continue;
        }

        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            word_started = true;
        } else if ch.is_whitespace() {
            if word_started {
                words.push(current.clone());
                current.clear();
                word_started = false;
            }
        } else {
            current.push(ch);
            word_started = true;
        }
    }

    if escaped {
        current.push('\\');
    }
    if let Some(active_quote) = quote {
        return Err(format!("unterminated {active_quote} quote"));
    }
    if word_started {
        words.push(current);
    }
    if words.first().map(|word| word.is_empty()).unwrap_or(false) {
        return Err("editor command program is empty".to_string());
    }
    Ok(words)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn split_command_supports_arguments_and_quotes() {
        assert_eq!(
            split_editor_command("code --wait 'two words.md'").unwrap(),
            vec!["code", "--wait", "two words.md"]
        );
        assert_eq!(
            split_editor_command("\"/tmp/my editor\" --flag").unwrap(),
            vec!["/tmp/my editor", "--flag"]
        );
        assert!(split_editor_command("vim '")
            .unwrap_err()
            .contains("unterminated"));
    }

    #[cfg(unix)]
    #[test]
    fn run_command_smoke_appends_to_document() {
        use std::os::unix::fs::PermissionsExt;

        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!(
            "tandem-editor-smoke-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let script = root.join("editor-smoke.sh");
        let doc = root.join("task-1.md");
        fs::write(
            &script,
            "#!/bin/sh\nprintf '\\nsmoke editor touched %s\\n' \"$1\" >> \"$1\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).unwrap();
        fs::write(
            &doc,
            "---\nid: task-1\ntype: task\ntitle: Test\nstate: todo\n---\n",
        )
        .unwrap();

        let editor = format!("/bin/sh {}", script.display());
        let command = EditorCommand::from_value(&editor, "test editor").unwrap();
        let status = run_editor_command(&command, &doc).unwrap();
        assert!(status.success());
        assert!(fs::read_to_string(&doc)
            .unwrap()
            .contains("smoke editor touched"));
        fs::remove_dir_all(root).unwrap();
    }
}
