//! `statute decide`: appends a proposed decision and marks its rules and terms proposed.

use crate::model::Report;

pub struct Options {
    pub summary: String,
    pub rationale: String,
    pub establishes: Vec<String>,
    pub changes: Vec<String>,
    pub retires: Vec<String>,
    pub terms: Vec<String>,
}

pub fn run(_options: &Options) -> anyhow::Result<Report> {
    Err(super::not_implemented("decide", 3))
}
