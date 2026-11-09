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

/// The `schema` every `--json` report starts with.
pub const SCHEMA: u32 = 1;

/// Each link's current fingerprint, computed from the files in the working tree, in link order.
fn fingerprints(repo: &Repo, links: &[&Link]) -> Result<Vec<Result<String, Missing>>> {
    let files = repo.read_files(links.iter().map(|l| l.file()))?;
    let targets: Vec<_> = links.iter().map(|l| (l.mode, l.target.as_str())).collect();
    Ok(fingerprint::sign(&files, &targets))
}

/// Where the Swift parser first fails, for each link that is missing because of a Swift parse
/// error, in link order. Only those files are read again; every other entry is `None`.
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
    Ok(links
        .iter()
        .zip(health)
        .map(|pair| {
            if !broken(pair) {
                return None;
            }
            let bytes = files.get(pair.0.file())?.as_deref()?;
            fingerprint::swift_parse_error_at(bytes)
        })
        .collect())
}
