//! `statute gap open` and `statute gap close`.

use crate::model::Report;

/// Without `gap_ref`, writes a full entry to the gaps file and requires `note`.
pub fn open(_rule: &str, _gap_ref: Option<&str>, _note: Option<&str>) -> anyhow::Result<Report> {
    Err(super::not_implemented("gap open", 4))
}

pub fn close(_gap_ref: &str) -> anyhow::Result<Report> {
    Err(super::not_implemented("gap close", 4))
}
