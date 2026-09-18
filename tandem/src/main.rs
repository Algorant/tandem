#![allow(dead_code)]

use std::io;

mod app;
mod cli;
mod project;
mod protocol;
mod tui;
mod web;

// Exit code categories: 0 success, 1 runtime/data/write failure, 2 usage/argument failure.
#[derive(Debug)]
pub(crate) struct CliError {
    pub(crate) message: String,
    pub(crate) code: i32,
    pub(crate) json: bool,
    /// Pre-built `{"ok":false,"error":...}` envelope for errors whose JSON
    /// contract differs from the generic usage/io shape. `None` uses the
    /// generic renderer; human mode always uses `message`.
    pub(crate) envelope: Option<serde_json::Value>,
}

impl CliError {
    pub(crate) fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
            json: false,
            envelope: None,
        }
    }
    pub(crate) fn user(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
            json: false,
            envelope: None,
        }
    }

    /// A standalone checkpoint failure. There is no successful record write to
    /// preserve, so this fails closed with exit code 1 and a checkpoint-specific
    /// JSON envelope instead of the generic lifecycle `status: failed` data.
    pub(crate) fn checkpoint_failure(message: impl Into<String>) -> Self {
        let message = message.into();
        let checkpoint = serde_json::json!({
            "status": "failed",
            "commit": serde_json::Value::Null,
            "amended": false,
            "consolidated": 0,
            "error": message.as_str(),
        });
        Self {
            envelope: Some(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "checkpoint",
                    "message": message.as_str(),
                    "details": { "checkpoint": checkpoint },
                }
            })),
            message,
            code: 1,
            json: false,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::user(error.to_string())
    }
}

impl From<protocol::diagnostic::Diagnostic> for CliError {
    fn from(diagnostic: protocol::diagnostic::Diagnostic) -> Self {
        CliError::user(diagnostic.message)
    }
}

impl From<app::Error> for CliError {
    fn from(error: app::Error) -> Self {
        if error.kind == app::ErrorKind::Usage {
            CliError::usage(error.message)
        } else {
            CliError::user(error.message)
        }
    }
}

fn main() {
    if let Err(error) = run() {
        if error.json {
            match &error.envelope {
                Some(envelope) => println!("{envelope}"),
                None => println!(
                    "{}",
                    serde_json::json!({"ok": false, "error": {"code": if error.code == 2 { "usage" } else { "io" }, "message": error.message, "details": {}}})
                ),
            }
        } else {
            eprintln!("Error: {}", error.message);
        }
        std::process::exit(error.code);
    }
}

fn run() -> Result<(), CliError> {
    match cli::run(std::env::args().skip(1).collect())? {
        cli::StartupRequest::Exit => Ok(()),
        cli::StartupRequest::Tui => tui::run_tui(app::project::open()?),
        cli::StartupRequest::Web(options) => web::run(app::project::open()?, options),
    }
}
