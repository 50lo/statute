//! `statute link`: adds a link with its current fingerprint, or removes one.

use crate::model::Report;

pub struct Options {
    pub rule: String,
    pub target: String,
    pub test: bool,
    pub remove: bool,
}

pub fn run(_options: &Options) -> anyhow::Result<Report> {
    Err(super::not_implemented("link", 2))
}
