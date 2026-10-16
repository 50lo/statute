//! `statute ratify`: activates decisions a person approved and updates their rules and terms.

use crate::model::Report;

pub fn run(_decisions: &[String]) -> anyhow::Result<Report> {
    Err(super::not_implemented("ratify", 3))
}
