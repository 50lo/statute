//! `statute review`: records one verdict for a rule and refreshes its stale links.

use crate::model::Report;

pub enum Verdict {
    Ok,
    Changed { decision: String },
    Gap { gap_ref: String },
}

pub fn run(_rule: &str, _verdict: &Verdict) -> anyhow::Result<Report> {
    Err(super::not_implemented("review", 2))
}
