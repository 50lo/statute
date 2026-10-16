//! `statute rule new`: appends a rule skeleton to `rules/<area>.md` with the next free ID.

use crate::model::Report;

pub fn run(_area: &str, _title: &str) -> anyhow::Result<Report> {
    Err(super::not_implemented("rule new", 1))
}
