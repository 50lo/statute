//! Statute keeps a product's rules written down, linked to code, and checked on every change.
//!
//! Module layering is set in AGENTS.md and checked by `tests/layering.rs`.

pub mod cli;

mod check;
mod commands;
mod model;
mod repo;
mod state;
mod store;
mod txn;
