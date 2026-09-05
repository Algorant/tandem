//! Shared application use cases for durable Tandem mutations.
//!
//! This layer coordinates canonical protocol validation with the concrete
//! [`crate::project::TandemProject`] boundary. It returns typed outcomes only;
//! CLI output and transient Ratatui state remain in their peer interfaces.

pub(crate) mod error;
#[allow(unused_imports)]
pub(crate) use error::{Error, ErrorKind};
pub(crate) mod accord;
pub(crate) mod assignment;
pub(crate) mod decisions;
pub(crate) mod dto;
pub(crate) mod project;
pub(crate) mod queries;
pub(crate) mod review;
#[cfg(test)]
mod review_contract_tests;
pub(crate) mod rules;
pub(crate) mod support;
pub(crate) mod tasks;
