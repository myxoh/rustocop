//! RuboCop 1.87.0 compatibility layer.
//!
//! Modules in this tree intentionally mirror RuboCop's implementation
//! boundaries and terminology. They are not used by cops until a separate,
//! explicit migration step.

#![allow(dead_code)]

pub(crate) mod ast;
pub(crate) mod cop;
pub(crate) mod dependencies;

#[cfg(test)]
mod translation_spec;
