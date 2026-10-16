use crate::model::{Finding, Model, Term, paths};
use std::collections::BTreeMap;

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
