//! `statute status`: per area, rules by state, pending decisions, open gaps and manual checks.

use crate::model::Report;

pub fn run(_area: Option<&str>) -> anyhow::Result<Report> {
    Err(super::not_implemented("status", 1))
}
