//! Executable implementation of the Tandem protocol.
//!
//! The normative protocol remains the repository Markdown specification in
//! [`protocol/plan/spec.md`](../../../protocol/plan/spec.md); this module
//! implements that specification and does not replace it. Concrete project
//! discovery, filesystem access, and raw-source patching remain outside this
//! boundary.

pub(crate) mod config;
pub(crate) mod document;
pub(crate) mod hierarchy;
pub(crate) mod ids;
