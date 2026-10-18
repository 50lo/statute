//! Append-only history: compares the model with the base.

use crate::model::{
    Agreed, Decision, DecisionStatus, Finding, Model, Rule, Term, TermAgreed, paths,
};

/// Active decisions in the base are unchanged and not deleted, rules retired in the base never change,
/// and the text of rules and terms agreed in the base changes only together with a decision that lists
/// them and is proposed or became active since the base.
pub fn history(model: &Model, base: &Model, out: &mut Vec<Finding>) {
    retired_rules(model, base, out);
    let (Some(decisions), Some(base_decisions)) = (&model.decisions, &base.decisions) else {
        return;
    };
    for old in base_decisions
        .iter()
        .filter(|d| d.status == DecisionStatus::Active)
    {
        let finding = match decisions.iter().find(|d| d.id == old.id) {
            None => Finding::error(
                "decision.history_deleted",
                format!(
                    "active decision {} was deleted; active decisions are never edited or deleted",
                    old.id
                ),
            ),
            Some(d) if d != old => Finding::error(
                "decision.history_edited",
                format!(
                    "active decision {} was edited; correct it with a new decision instead",
                    old.id
                ),
            ),
            Some(_) => continue,
        };
        out.push(finding.decision(&old.id).at(paths::DECISIONS, None));
    }

    // Decisions that may carry a change: proposed now, or active now but not in the base.
    let was_active = |id: &str| {
        base_decisions
            .iter()
            .any(|d| d.id == id && d.status == DecisionStatus::Active)
    };
    let carriers: Vec<&Decision> = decisions
        .iter()
        .filter(|d| d.status == DecisionStatus::Proposed || !was_active(&d.id))
        .collect();

    for old in base.rules.iter().filter(|r| agreed_rule(r)) {
        let now = model.rules.iter().find(|r| r.id == old.id);
        if now.is_some_and(|r| r.meta.is_none()) {
            continue; // A parse problem, reported on its own.
        }
        if now.is_some_and(|r| same_rule(r, old))
            || carriers.iter().any(|d| d.changes.contains(&old.id))
        {
            continue;
        }
        let what = if now.is_some() { "changed" } else { "removed" };
        let finding = Finding::error(
            "rule.changed_without_decision",
            format!("agreed rule {} was {what} without a decision that lists it in `changes`; run `statute decide --changes {}`", old.id, old.id),
        )
        .rule(&old.id);
        out.push(match now {
            Some(r) => finding.at(&r.file, Some(r.line)),
            None => finding.at(&old.file, None),
        });
    }

    for old in base.terms.iter().filter(|t| agreed_term(t)) {
        let now = model.terms.iter().find(|t| t.name == old.name);
        if now.is_some_and(|t| t.meta.is_none()) {
            continue;
        }
        if now.is_some_and(|t| same_term(t, old))
            || carriers.iter().any(|d| d.terms.contains(&old.name))
        {
            continue;
        }
        let what = if now.is_some() { "changed" } else { "removed" };
        out.push(
            Finding::error(
                "term.changed_without_decision",
                format!("agreed term `{}` was {what} without a decision that lists it in `terms`; run `statute decide --terms \"{}\"`", old.name, old.name),
            )
            .term(&old.name)
            .at(paths::TERMS, now.map(|t| t.line)),
        );
    }
}

/// A rule retired in the base keeps its section, its compared fields and `agreed: retired`. No decision
/// exempts it, so the finding names a new rule ID as the way forward rather than `decide`.
fn retired_rules(model: &Model, base: &Model, out: &mut Vec<Finding>) {
    for old in base.rules.iter().filter(|r| retired_rule(r)) {
        let now = model.rules.iter().find(|r| r.id == old.id);
        if now.is_some_and(|r| r.meta.is_none()) {
            continue; // A parse problem, reported on its own.
        }
        if now.is_some_and(|r| retired_rule(r) && same_rule(r, old)) {
            continue;
        }
        let what = if now.is_some() { "changed" } else { "removed" };
        let finding = Finding::error(
            "rule.retired_changed",
            format!("retired rule {} was {what}; a retired rule keeps its section, its text and `agreed: retired`, so give new behaviour a new rule ID with `statute rule new`", old.id),
        )
        .rule(&old.id);
        out.push(match now {
            Some(r) => finding.at(&r.file, Some(r.line)),
            None => finding.at(&old.file, None),
        });
    }
}

fn retired_rule(rule: &Rule) -> bool {
    rule.meta
        .as_ref()
        .is_some_and(|m| m.agreed == Agreed::Retired)
}

fn agreed_rule(rule: &Rule) -> bool {
    rule.meta
        .as_ref()
        .is_some_and(|m| m.agreed == Agreed::Agreed)
}

fn agreed_term(term: &Term) -> bool {
    term.meta
        .as_ref()
        .is_some_and(|m| m.agreed == Some(TermAgreed::Agreed))
}

/// The compared fields: the statement with whitespace collapsed, `scope`, `terms`, `exceptions`,
/// `protects`, `boundary` and `manual_check`. Notes, `agreed`, `built` and formatting are not compared.
fn same_rule(a: &Rule, b: &Rule) -> bool {
    let (Some(x), Some(y)) = (&a.meta, &b.meta) else {
        return false;
    };
    collapse(&a.statement) == collapse(&b.statement)
        && x.scope == y.scope
        && x.terms == y.terms
        && x.exceptions == y.exceptions
        && x.protects == y.protects
        && x.boundary == y.boundary
        && x.manual_check == y.manual_check
}

/// The definition (the section except its heading and `yaml` block) with whitespace collapsed, `code`
/// and `not`.
fn same_term(a: &Term, b: &Term) -> bool {
    let (Some(x), Some(y)) = (&a.meta, &b.meta) else {
        return false;
    };
    collapse(&a.definition) == collapse(&b.definition) && x.code == y.code && x.not == y.not
}

fn collapse(text: &Option<String>) -> Vec<&str> {
    text.as_deref()
        .map(|t| t.split_whitespace().collect())
        .unwrap_or_default()
}
