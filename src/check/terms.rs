use crate::model::{Finding, Model, Term, paths};
use std::collections::{BTreeMap, BTreeSet};

/// Term names are unique.
pub fn unique(model: &Model, out: &mut Vec<Finding>) {
    let mut seen: BTreeMap<&str, &Term> = BTreeMap::new();
    for term in &model.terms {
        if let Some(first) = seen.insert(&term.name, term) {
            out.push(
                Finding::error(
                    "term.duplicate",
                    format!(
                        "term `{}` is also defined on line {}",
                        term.name, first.line
                    ),
                )
                .term(&term.name)
                .at(paths::TERMS, Some(term.line)),
            );
        }
    }
}

/// A retired term is named in some decision's `retires_terms`, as an agreed or retired rule is
/// established by a decision.
pub fn retirement_recorded(model: &Model, out: &mut Vec<Finding>) {
    let Some(decisions) = &model.decisions else {
        return;
    };
    let retired_by: BTreeSet<&str> = decisions
        .iter()
        .flat_map(|d| &d.retires_terms)
        .map(String::as_str)
        .collect();
    for term in model.terms.iter().filter(|t| t.is_retired()) {
        if !retired_by.contains(term.name.as_str()) {
            out.push(
                Finding::error(
                    "term.retired_without_decision",
                    format!(
                        "term `{}` is retired, but no decision lists it in `retires_terms`",
                        term.name
                    ),
                )
                .term(&term.name)
                .at(paths::TERMS, Some(term.line)),
            );
        }
    }
}
