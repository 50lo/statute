//! Statute keeps a product's rules written down, linked to code, and checked on every change.
//!
//! Module layering and I/O boundaries are set in AGENTS.md; `tests/layering.rs` enforces them.

pub mod check;
pub mod cli;
pub mod commands;
pub mod fingerprint;
pub mod git;
pub mod impact;
pub mod model;
pub mod repo;
pub mod state;
pub mod store;
pub mod txn;
