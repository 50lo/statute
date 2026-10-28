//! One function per check. Each adds findings with a stable code.

mod decisions;
mod gaps;
mod history;
mod links;
mod names;
mod rules;
mod terms;

use crate::model::{Finding, Model};

type Check = fn(&Model, &mut Vec<Finding>);

/// Every structural check, in report order.
const CHECKS: &[Check] = &[
    config_version,
    rules::prefixes,
    rules::ids,
    rules::statements,
    rules::terms_resolve,
    rules::retired_terms,
    rules::cases,
    rules::establishment,
    rules::avoid_words,
    links::retired,
    links::completeness,
    terms::unique,
    terms::retirement_recorded,
    decisions::ids,
    decisions::references,
    decisions::overlap,
    decisions::retired_terms,
    decisions::approved_by,
    decisions::summary_length,
    gaps::ids,
    gaps::rules_resolve,
    links::references,
];

/// Parse problems found while loading, then every check.
pub fn run(model: &Model) -> Vec<Finding> {
    let mut findings = model.problems.clone();
    for check in CHECKS {
        check(model, &mut findings);
    }
    findings
}

pub use history::history;
pub use links::freshness;
pub use names::NameSearch;

fn config_version(model: &Model, out: &mut Vec<Finding>) {
    if model.config.version != 1 {
        out.push(
            Finding::error(
                "config.version",
                format!(
                    "statute.yml has version {}; this statute supports version 1",
                    model.config.version
                ),
            )
            .at(crate::model::paths::CONFIG, None),
        );
    }
}
