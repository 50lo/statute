//! Which rules and terms a branch affects, why, and the verdict on each rule (spec section 5).

use crate::model::{
    Decision, DecisionStatus, Gap, GapStatus, Impact, Link, Model, PendingDecision, Reason,
    RuleImpact, TermImpact, Verdict,
};
use crate::state::Health;
use std::collections::{BTreeMap, BTreeSet};

/// Reasons by rule ID or term name.
type Reasons<'a> = BTreeMap<&'a str, BTreeSet<Reason>>;

/// Compares the model with the base. `health` is the current health of the links `check` looked at.
pub fn compute<'a>(
    model: &'a Model,
    base: &'a Model,
    health: &[(&'a str, &str, Health)],
) -> Impact {
    let mut rules = Reasons::new();
    let mut terms = Reasons::new();
    link_reasons(model, base, health, &mut rules);
    let decisions = model.decisions.as_deref().unwrap_or_default();
    let base_decisions = base.decisions.as_deref().unwrap_or_default();
    let changed: Vec<_> = decisions
        .iter()
        .filter(|d| !base_decisions.contains(d))
        .collect();
    decision_reasons(&changed, &mut rules, &mut terms);
    let new_open = gap_reasons(model, base, &mut rules);

    let title = |id: &str| {
        model
            .rules
            .iter()
            .chain(&base.rules)
            .find(|r| r.id == id)
            .map(|r| r.title.clone())
            .unwrap_or_default()
    };
    let rule_impacts: Vec<RuleImpact> = rules
        .iter()
        .map(|(rule, reasons)| RuleImpact {
            rule: rule.to_string(),
            title: title(rule),
            verdict: verdict(rule, reasons, &changed, &new_open),
            reasons: reasons.iter().cloned().collect(),
        })
        .collect();
    let term_impacts: Vec<TermImpact> = terms
        .iter()
        .map(|(term, reasons)| TermImpact {
            term: term.to_string(),
            reasons: reasons.iter().cloned().collect(),
        })
        .collect();

    let awaiting_approval = decisions
        .iter()
        .filter(|d| d.status == DecisionStatus::Proposed)
        .filter(|d| {
            d.rules().any(|r| rules.contains_key(r.as_str()))
                || d.all_terms().any(|t| terms.contains_key(t.as_str()))
        })
        .map(pending_decision)
        .collect();

    Impact {
        rules: rule_impacts,
        terms: term_impacts,
        awaiting_approval,
    }
}

/// Stale and missing links, and links added, removed or refreshed since the base.
fn link_reasons<'a>(
    model: &'a Model,
    base: &'a Model,
    health: &[(&'a str, &str, Health)],
    rules: &mut Reasons<'a>,
) {
    for (rule, target, h) in health {
        let target = target.to_string();
        let reason = match h {
            Health::Fresh => continue,
            Health::Stale => Reason::LinkStale { target },
            Health::Missing(_) => Reason::LinkMissing { target },
        };
        rules.entry(rule).or_default().insert(reason);
    }

    let links = model.links.as_deref().unwrap_or_default();
    let base_links = base.links.as_deref().unwrap_or_default();
    // A link is the same link only with the same rule, target, role and mode; any other change is
    // a removal and an addition, so the rule is relinked.
    let same = |a: &Link, b: &Link| {
        a.source == b.source && a.target == b.target && a.role == b.role && a.mode == b.mode
    };
    for link in links {
        let old = base_links.iter().find(|l| same(l, link));
        let target = link.target.clone();
        let reason = match old {
            None => Reason::LinkAdded { target },
            Some(old) if old.sig != link.sig => Reason::LinkRefreshed { target },
            Some(_) => continue,
        };
        rules.entry(link.rule()).or_default().insert(reason);
    }
    for old in base_links {
        if !links.iter().any(|l| same(l, old)) {
            let reason = Reason::LinkRemoved {
                target: old.target.clone(),
            };
            rules.entry(old.rule()).or_default().insert(reason);
        }
    }
}

/// The rules and terms that decisions added or changed since the base name.
fn decision_reasons<'a>(
    changed: &[&'a Decision],
    rules: &mut Reasons<'a>,
    terms: &mut Reasons<'a>,
) {
    for d in changed {
        let reason = || Reason::DecisionChanged {
            decision: d.id.clone(),
        };
        for rule in d.rules() {
            rules.entry(rule).or_default().insert(reason());
        }
        for term in &d.terms {
            terms.entry(term).or_default().insert(reason());
        }
        for term in &d.retires_terms {
            let decision = d.id.clone();
            let reason = match d.status {
                DecisionStatus::Proposed => Reason::RetirementProposed { decision },
                DecisionStatus::Active => Reason::Retired { decision },
            };
            terms.entry(term).or_default().insert(reason);
        }
    }
}

/// Adds a reason for every gap added or changed since the base, and returns the gaps that are open
/// now and were not open in the base.
fn gap_reasons<'a>(model: &'a Model, base: &Model, rules: &mut Reasons<'a>) -> Vec<&'a Gap> {
    let gaps = model.gaps.as_deref().unwrap_or_default();
    let base_gaps = base.gaps.as_deref().unwrap_or_default();
    gaps.iter()
        .filter(|g| !base_gaps.contains(g))
        .inspect(|g| {
            let reason = Reason::GapChanged { gap: g.id.clone() };
            rules.entry(&g.rule).or_default().insert(reason);
        })
        .filter(|g| {
            g.status == GapStatus::Open
                && !base_gaps
                    .iter()
                    .any(|b| b.id == g.id && b.status == GapStatus::Open)
        })
        .collect()
}

/// The verdict on an affected rule, in the order spec section 5 gives.
fn verdict(
    rule: &str,
    reasons: &BTreeSet<Reason>,
    changed: &[&Decision],
    new_open: &[&Gap],
) -> Verdict {
    let lists = |pick: fn(&Decision) -> &Vec<String>| {
        changed.iter().any(|d| pick(d).iter().any(|r| r == rule))
    };
    if reasons
        .iter()
        .any(|r| matches!(r, Reason::LinkStale { .. } | Reason::LinkMissing { .. }))
    {
        Verdict::Unreviewed
    } else if new_open.iter().any(|g| g.rule == rule) {
        Verdict::Broken
    } else if lists(|d| &d.establishes) {
        Verdict::New
    } else if lists(|d| &d.changes) {
        Verdict::Changed
    } else if lists(|d| &d.retires) {
        Verdict::Retired
    } else if reasons
        .iter()
        .any(|r| matches!(r, Reason::LinkAdded { .. } | Reason::LinkRemoved { .. }))
    {
        Verdict::Relinked
    } else {
        Verdict::Holds
    }
}

/// Every proposed decision, whether or not the branch changed it: what `status` lists as awaiting
/// approval.
pub fn pending(model: &Model) -> Vec<PendingDecision> {
    model
        .decisions
        .iter()
        .flatten()
        .filter(|d| d.status == DecisionStatus::Proposed)
        .map(pending_decision)
        .collect()
}

fn pending_decision(d: &Decision) -> PendingDecision {
    PendingDecision {
        id: d.id.clone(),
        summary: d.summary.clone(),
        rules: d.rules().cloned().collect(),
        terms: d.terms.clone(),
        retires_terms: d.retires_terms.clone(),
    }
}
