use crate::model::{Agreed, Finding, Link, Model, paths};
use crate::state::{self, Health};
use std::collections::{BTreeMap, BTreeSet};

fn at(f: Finding, link: &Link) -> Finding {
    f.rule(link.rule())
        .target(&link.target)
        .at(paths::LOCK, None)
}

/// Every link names an existing rule, and no rule links one target twice.
pub fn references(model: &Model, out: &mut Vec<Finding>) {
    let rules: BTreeSet<&str> = model.rules.iter().map(|r| r.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    for link in model.links.iter().flatten() {
        if !rules.contains(link.rule()) {
            out.push(at(
                Finding::error(
                    "link.rule_unknown",
                    format!("rule {} is not defined in any rule file", link.rule()),
                ),
                link,
            ));
        }
        if !seen.insert((link.rule(), link.target.as_str())) {
            out.push(at(
                Finding::error(
                    "link.duplicate",
                    format!("{} links {} more than once", link.rule(), link.target),
                ),
                link,
            ));
        }
    }
}

/// A retired rule has no links.
pub fn retired(model: &Model, out: &mut Vec<Finding>) {
    let retired: BTreeMap<&str, _> = model
        .rules
        .iter()
        .filter(|r| r.meta.as_ref().is_some_and(|m| m.agreed == Agreed::Retired))
        .map(|r| (r.id.as_str(), r))
        .collect();
    for link in model.links.iter().flatten() {
        if let Some(rule) = retired.get(link.rule()) {
            out.push(
                Finding::error(
                    "rule.retired_linked",
                    format!("{} is retired but still links {}", rule.id, link.target),
                )
                .rule(&rule.id)
                .target(&link.target)
                .at(&rule.file, Some(rule.line)),
            );
        }
    }
}

/// A rule that is not retired and is `built: built` has a code link, and a test link or a `manual_check`.
pub fn completeness(model: &Model, out: &mut Vec<Finding>) {
    for rule in model
        .rules
        .iter()
        .filter(|r| state::is_incomplete(r, model))
    {
        out.push(
            Finding::error(
                "rule.incomplete",
                format!(
                    "{} is built but lacks a code link, or both a test link and a `manual_check`",
                    rule.id
                ),
            )
            .rule(&rule.id)
            .at(&rule.file, Some(rule.line)),
        );
    }
}

/// A finding for each stale or missing link.
pub fn freshness(links: &[&Link], health: &[Health]) -> Vec<Finding> {
    let mut out = Vec::new();
    for (link, health) in links.iter().zip(health) {
        let finding = match health {
            Health::Fresh => continue,
            Health::Stale => Finding::error(
                "link.stale",
                format!(
                    "{} changed since it was linked; check that the rule still holds, then run `statute review {}`",
                    link.target,
                    link.rule()
                ),
            ),
            Health::Missing(why) => {
                Finding::error("link.missing", format!("{} is missing: {why}", link.target))
            }
        };
        out.push(
            finding
                .rule(link.rule())
                .target(&link.target)
                .at(link.file(), None),
        );
    }
    out
}
