//! One module per command: its logic, without argument parsing or rendering.
//! Command modules do not call each other; shared logic moves into a lower module.

pub mod check;
pub mod decide;
pub mod gap;
pub mod init;
pub mod link;
pub mod ratify;
pub mod review;
pub mod rule_new;
pub mod show;
pub mod status;

fn not_implemented(command: &str, milestone: u8) -> anyhow::Error {
    anyhow::anyhow!("`{command}` is not implemented yet (milestone {milestone})")
}
