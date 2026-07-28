use std::io;

mod app;
mod cli;
mod project;
mod protocol;
mod tui;

// Exit code categories: 0 success, 1 runtime/data/write failure, 2 usage/argument failure.
#[derive(Debug)]
pub(crate) struct CliError {
    pub(crate) message: String,
    pub(crate) code: i32,
}

impl CliError {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
        }
    }
    pub(crate) fn user(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
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

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error.message);
        std::process::exit(error.code);
    }
}

fn run() -> Result<(), CliError> {
    match cli::run(std::env::args().skip(1).collect())? {
        cli::StartupRequest::Exit => Ok(()),
        cli::StartupRequest::Tui => tui::run_tui(app::project::open()?),
    }
}
