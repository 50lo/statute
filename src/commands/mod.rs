//! One module per command: its logic, without argument parsing or rendering. Each returns a report.

pub mod check;
pub mod decide;
pub mod init;
pub mod link;
pub mod ratify;
pub mod review;
pub mod rule_new;
pub mod show;
pub mod status;

/// The `schema` every `--json` report starts with.
pub const SCHEMA: u32 = 1;
