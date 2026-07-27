//! Shared application use cases for durable Tandem mutations.
//!
//! This layer coordinates canonical protocol validation with the concrete
//! [`crate::project::TandemProject`] boundary. It returns typed outcomes only;
//! CLI output and transient Ratatui state remain in their peer interfaces.

pub(crate) mod accord;
pub(crate) mod tasks;
