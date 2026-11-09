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
use crate::model::{Link, Mode};
use crate::repo::Repo;
use crate::state::Health;
use anyhow::Result;
use std::collections::BTreeMap;

/// The `schema` every `--json` report starts with.
pub const SCHEMA: u32 = 1;

/// Each link's current fingerprint, computed from the files in the working tree, in link order.
fn fingerprints(repo: &Repo, links: &[&Link]) -> Result<Vec<Result<String, Missing>>> {
    let files = repo.read_files(links.iter().map(|l| l.file()))?;
    let targets: Vec<_> = links.iter().map(|l| (l.mode, l.target.as_str())).collect();
    Ok(fingerprint::sign(&files, &targets))
}

/// Where the Swift parser first fails, for each link that is missing because of a Swift parse
/// error, in link order. Only those files are read again, and each is parsed once; every other
/// entry is `None`.
fn swift_parse_errors(
    repo: &Repo,
    links: &[&Link],
    health: &[Health],
) -> Result<Vec<Option<(usize, usize)>>> {
    let broken = |(link, health): (&&Link, &Health)| {
        link.mode == Mode::SwiftAstSha256V1 && *health == Health::Missing(Missing::ParseError)
    };
    let files = repo.read_files(
        links
            .iter()
            .zip(health)
            .filter(|pair| broken(*pair))
            .map(|(l, _)| l.file()),
    )?;
    let near: BTreeMap<&str, Option<(usize, usize)>> = files
        .iter()
        .map(|(file, bytes)| {
            let at = bytes.as_deref().and_then(fingerprint::swift_parse_error_at);
            (file.as_str(), at)
        })
        .collect();
    Ok(links
        .iter()
        .zip(health)
        .map(|pair| {
            if !broken(pair) {
                return None;
            }
            near.get(pair.0.file()).copied().flatten()
        })
        .collect())
}
