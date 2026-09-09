//! Stable operational errors shared by application use cases.
#![allow(dead_code)]
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) message: String,
    pub(crate) details: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    Usage,
    NotFound,
    Validation,
    Conflict,
    Io,
    Internal,
}

impl Error {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Usage, message)
    }

    pub(crate) fn user(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
        }
    }

    pub(crate) fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub(crate) fn code(&self) -> &'static str {
        match self.kind {
            ErrorKind::Usage => "usage",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Validation => "validation",
            ErrorKind::Conflict => "conflict",
            ErrorKind::Io => "io",
            ErrorKind::Internal => "internal",
        }
    }
}

impl From<crate::CliError> for Error {
    fn from(error: crate::CliError) -> Self {
        if error.code == 2 {
            Self::usage(error.message)
        } else {
            Self::user(error.message)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, error.to_string())
    }
}

impl From<crate::protocol::diagnostic::Diagnostic> for Error {
    fn from(error: crate::protocol::diagnostic::Diagnostic) -> Self {
        Self::new(ErrorKind::Validation, error.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_stable_operational_codes() {
        assert_eq!(Error::usage("invalid").code(), "usage");
        assert_eq!(
            Error::new(ErrorKind::NotFound, "missing").code(),
            "not_found"
        );
        assert_eq!(
            Error::new(ErrorKind::Validation, "invalid").code(),
            "validation"
        );
        assert_eq!(
            Error::new(ErrorKind::Conflict, "conflict").code(),
            "conflict"
        );
        assert_eq!(Error::new(ErrorKind::Io, "io").code(), "io");
    }
}
