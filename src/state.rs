//! Derived state, computed from the model and never stored.

use crate::fingerprint::Missing;
use crate::model::{Agreed, Built, GapStatus, Link, LinkRole, Mode, Model, Rule};
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
        && rule.agreed() == Some(Agreed::Proposed)
        && !establishers.contains_key(rule.id.as_str())
}

/// Whether a link's target still has the fingerprint the lock records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Fresh,
    /// The current fingerprint differs from `sig`.
    Stale,
    /// The target is not found, matches more than one symbol, or its file does not parse cleanly.
    Missing(Missing),
}

/// Compares each link's `sig` with its current fingerprint, given in the same order.
pub fn health(links: &[&Link], current: &[Result<String, Missing>]) -> Vec<Health> {
    links
        .iter()
        .zip(current)
        .map(|(link, now)| match now {
            Ok(sig) if *sig == link.sig => Health::Fresh,
            Ok(_) => Health::Stale,
            Err(why) => Health::Missing(*why),
        })
        .collect()
}

/// A way to recover when the Swift grammar cannot read an existing symbol link.
/// `near` is where the parser first fails, as a 1-based line and column, when known.
pub fn swift_parse_hint(link: &Link, why: Missing, near: Option<(usize, usize)>) -> Option<String> {
    if link.mode != Mode::SwiftAstSha256V1 || why != Missing::ParseError {
        return None;
    }
    let target = link.target.replace('\'', "'\\''");
    let test = if link.role == LinkRole::Test {
        " --test"
    } else {
        ""
    };
    let near = near
        .map(|(line, column)| format!("; the parser first fails near line {line}, column {column}"))
        .unwrap_or_default();
    Some(format!(
        "{near}. If the Swift is valid, the parser may not support it yet: check the rule, then run `statute link --remove {} '{}'` and `statute link {} '{}'{}` to link the whole file.",
        link.rule(),
        target,
        link.rule(),
        target,
        test
    ))
}

/// A rule is complete when it has a code link, and a test link or a `manual_check`.
pub fn is_complete(rule: &Rule, links: &[Link]) -> bool {
    let has = |role| links.iter().any(|l| l.rule() == rule.id && l.role == role);
    let manual = rule
        .meta
        .as_ref()
        .and_then(|m| m.manual_check.as_deref())
        .is_some_and(|c| !c.trim().is_empty());
    has(LinkRole::Code) && (has(LinkRole::Test) || manual)
}

/// Incomplete: not retired, `built: built`, and not complete. Unknown while the lock does not parse.
pub fn is_incomplete(rule: &Rule, model: &Model) -> bool {
    let Some(links) = &model.links else {
        return false;
    };
    rule.meta
        .as_ref()
        .is_some_and(|m| m.agreed != Agreed::Retired && m.built == Built::Built)
        && !is_complete(rule, links)
}

/// Broken: not retired, `built: built`, and at least one open gap.
pub fn is_broken(rule: &Rule, model: &Model) -> bool {
    rule.meta
        .as_ref()
        .is_some_and(|m| m.agreed != Agreed::Retired && m.built == Built::Built)
        && model
            .gaps
            .iter()
            .flatten()
            .any(|g| g.rule == rule.id && g.status == GapStatus::Open)
}
