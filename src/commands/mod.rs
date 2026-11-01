//! One module per command: its logic, without argument parsing or rendering. Each returns a report.

pub mod approve;
pub mod check;
pub mod decide;
pub mod gap;
pub mod init;
pub mod link;
pub mod review;
pub mod rule_new;
pub mod show;
pub mod status;

use crate::fingerprint::{self, Missing};
use crate::model::Link;
use crate::repo::Repo;
use anyhow::Result;

/// The `schema` every `--json` report starts with.
pub const SCHEMA: u32 = 1;

/// Each link's current fingerprint, computed from the files in the working tree, in link order.
fn fingerprints(repo: &Repo, links: &[&Link]) -> Result<Vec<Result<String, Missing>>> {
    let files = repo.read_files(links.iter().map(|l| l.file()))?;
    let targets: Vec<_> = links.iter().map(|l| (l.mode, l.target.as_str())).collect();
    Ok(fingerprint::sign(&files, &targets))
}
