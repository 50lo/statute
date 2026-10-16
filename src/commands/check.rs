//! `statute check`: every check from spec section 4, plus stale and missing links since the base.

use crate::model::Report;

pub struct Options {
    /// Re-fingerprint every link and search for term names across the repo.
    pub all: bool,
    /// Compute the "Rule impact" section.
    pub impact: bool,
}

pub fn run(_options: &Options) -> anyhow::Result<Report> {
    Err(super::not_implemented("check", 1))
}
