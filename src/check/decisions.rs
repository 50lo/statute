use crate::model::ids::is_dated_id;
use crate::model::{Decision, DecisionStatus, Finding, Model, paths};
use std::collections::{BTreeMap, BTreeSet};

fn at(f: Finding, d: &Decision) -> Finding {
    f.decision(&d.id).at(paths::DECISIONS, None)
}

/// IDs have the form `DYYMMDD-xxxxx` and are unique.
pub fn ids(model: &Model, out: &mut Vec<Finding>) {
    let mut seen = BTreeSet::new();
    for d in model.decisions.iter().flatten() {
        if !is_dated_id(&d.id, 'D') {
            out.push(at(
                Finding::error(
                    "decision.id_invalid",
                    format!("`{}` is not a decision ID such as `D261015-k7m2q`", d.id),
                ),
                d,
            ));
        }
        if !seen.insert(&d.id) {
            out.push(at(
                Finding::error(
                    "decision.duplicate",
                    format!("{} appears more than once", d.id),
                ),
                d,
            ));
        }
    }
}

/// Every rule in `establishes`, `changes` and `retires` exists, and every entry in `terms` and
/// `retires_terms` is a term.
pub fn references(model: &Model, out: &mut Vec<Finding>) {
    let rules: BTreeSet<&str> = model.rules.iter().map(|r| r.id.as_str()).collect();
    let terms: BTreeSet<&str> = model.terms.iter().map(|t| t.name.as_str()).collect();
    for d in model.decisions.iter().flatten() {
        for rule in d.rules().filter(|r| !rules.contains(r.as_str())) {
            out.push(at(
                Finding::error(
                    "decision.rule_unknown",
                    format!("rule {rule} is not defined in any rule file"),
                )
                .rule(rule),
                d,
            ));
        }
        for term in d.all_terms().filter(|t| !terms.contains(t.as_str())) {
            out.push(at(
                Finding::error(
                    "decision.term_unknown",
                    format!("term `{term}` is not defined in terms.md"),
                )
                .term(term),
                d,
            ));
        }
    }
}

/// A rule or term appears in at most one proposed decision, whichever list names it.
pub fn overlap(model: &Model, out: &mut Vec<Finding>) {
    let mut rules: BTreeMap<&str, &str> = BTreeMap::new();
    let mut terms: BTreeMap<&str, &str> = BTreeMap::new();
    for d in model
        .decisions
        .iter()
        .flatten()
        .filter(|d| d.status == DecisionStatus::Proposed)
    {
        for rule in d.rules().collect::<BTreeSet<_>>() {
            if let Some(other) = rules.insert(rule, &d.id) {
                out.push(at(
                    Finding::error(
                        "decision.overlap",
                        format!("rule {rule} is also in proposed decision {other}; a rule is in at most one"),
                    )
                    .rule(rule),
                    d,
                ));
            }
        }
        for term in d.all_terms().collect::<BTreeSet<_>>() {
            if let Some(other) = terms.insert(term, &d.id) {
                out.push(at(
                    Finding::error(
                        "decision.overlap",
                        format!("term `{term}` is also in proposed decision {other}; a term is in at most one"),
                    )
                    .term(term),
                    d,
                ));
            }
        }
    }
}

/// A proposed decision names no retired term, in `terms` or `retires_terms`: a retired term never
/// changes. Active decisions keep naming the terms they retired or defined.
pub fn retired_terms(model: &Model, out: &mut Vec<Finding>) {
    let retired: BTreeSet<&str> = model
        .terms
        .iter()
        .filter(|t| t.is_retired())
        .map(|t| t.name.as_str())
        .collect();
    for d in model
        .decisions
        .iter()
        .flatten()
        .filter(|d| d.status == DecisionStatus::Proposed)
    {
        for term in d.all_terms().filter(|t| retired.contains(t.as_str())) {
            out.push(at(
                Finding::error(
                    "decision.term_retired",
                    format!("term `{term}` is retired, and a retired term never changes; name a new term for a new meaning"),
                )
                .term(term),
                d,
            ));
        }
    }
}

/// An active decision has `approved_by`.
pub fn approved_by(model: &Model, out: &mut Vec<Finding>) {
    for d in model.decisions.iter().flatten() {
        if d.status == DecisionStatus::Active && d.approved_by.is_none() {
            out.push(at(
                Finding::error(
                    "decision.approved_by_missing",
                    "an active decision needs `approved_by`",
                ),
                d,
            ));
        }
    }
}

/// Summaries stay within `summary_max` characters.
pub fn summary_length(model: &Model, out: &mut Vec<Finding>) {
    let max = model.config.summary_max;
    for d in model.decisions.iter().flatten() {
        let len = d.summary.chars().count();
        if len > max {
            out.push(at(
                Finding::error(
                    "decision.summary_too_long",
                    format!("summary has {len} characters; the cap is {max} (summary_max in statute.yml)"),
                ),
                d,
            ));
        }
    }
}
