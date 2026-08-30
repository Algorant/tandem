//! Process-boundary parsing helpers for the clap command model.
#![allow(dead_code)]
use clap::{error::ErrorKind, Parser};

use super::model::Cli;

/// Extract the globally reserved JSON switch before clap sees prose values.
/// Exact tokens are reserved; callers can still pass literal `--json` with
/// clap's `--body=--json` form.
pub(crate) fn json_requested(argv: &[String]) -> bool {
    argv.iter()
        .any(|arg| matches!(arg.as_str(), "-j" | "--json"))
}

pub(crate) fn parse(argv: &[String]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("tandem".to_string()).chain(argv.iter().cloned()))
}

pub(crate) fn is_help(error: &clap::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn accepts_global_json_before_and_after_subcommands() {
        assert!(parse(&args(&["-j", "list"])).is_ok());
        assert!(parse(&args(&["list", "-j"])).is_ok());
        assert!(json_requested(&args(&["list", "-j"])));
    }

    #[test]
    fn generated_help_and_version_are_display_errors() {
        let help = parse(&args(&["--help"])).unwrap_err();
        assert!(is_help(&help));
        let version = parse(&args(&["--version"])).unwrap_err();
        assert!(is_help(&version));
    }

    #[test]
    fn rules_edit_accepts_generic_clear_source() {
        assert!(parse(&args(&[
            "rules",
            "edit",
            "always-12",
            "revised",
            "--clear",
            "source",
        ]))
        .is_ok());
    }

    #[test]
    fn prose_accepts_leading_hyphens_and_scalar_repetition_is_rejected() {
        assert!(parse(&args(&[
            "add",
            "task",
            "- title",
            "--acceptance",
            "- criterion"
        ]))
        .is_ok());
        assert!(parse(&args(&[
            "add",
            "task",
            "Title",
            "--acceptance",
            "criterion",
            "--priority",
            "low",
            "--priority",
            "high",
        ]))
        .is_err());
    }
}
