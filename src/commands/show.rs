//! `statute show`: rules for files, symbols, rule IDs, terms or decision IDs.

use crate::model::Report;

pub fn run(_items: &[String]) -> anyhow::Result<Report> {
    Err(super::not_implemented("show", 1))
}
