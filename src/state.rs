//! Derived state, computed from the model and never stored.

use crate::model::{Agreed, Model, Rule};
use std::collections::BTreeMap;

/// For each rule ID, the decisions whose `establishes` lists it.
pub fn establishers(model: &Model) -> BTreeMap<&str, Vec<&str>> {
    let mut map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for d in model.decisions.iter().flatten() {
        for rule in &d.establishes {
            map.entry(rule.as_str()).or_default().push(d.id.as_str());
        }
    }
    map
}

/// A draft is a proposed rule that no decision establishes yet. It is a pending item, not a finding.
/// Nothing is a draft while `decisions.db` does not parse, because establishment is unknown.
pub fn is_draft(rule: &Rule, model: &Model, establishers: &BTreeMap<&str, Vec<&str>>) -> bool {
    model.decisions.is_some()
        && rule
            .meta
            .as_ref()
            .is_some_and(|m| m.agreed == Agreed::Proposed)
        && !establishers.contains_key(rule.id.as_str())
}
