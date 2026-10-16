use crate::model::ids::is_dated_id;
use crate::model::{Finding, Gap, Model, paths};
use std::collections::BTreeSet;

fn at(f: Finding, g: &Gap) -> Finding {
    f.gap(&g.id).at(paths::GAPS, None)
}

/// IDs are unique; a gap kept in the gaps file (one with a `note`) has an ID such as `G261015-x4p9t`.
/// A tracker gap uses the tracker's own ID.
pub fn ids(model: &Model, out: &mut Vec<Finding>) {
    let mut seen = BTreeSet::new();
    for g in model.gaps.iter().flatten() {
        if g.note.is_some() && !is_dated_id(&g.id, 'G') {
            out.push(at(
                Finding::error(
                    "gap.id_invalid",
                    format!("`{}` is not a gap ID such as `G261015-x4p9t`", g.id),
                ),
                g,
            ));
        }
        if !seen.insert(&g.id) {
            out.push(at(
                Finding::error("gap.duplicate", format!("{} appears more than once", g.id)),
                g,
            ));
        }
    }
}

/// Every gap names an existing rule.
pub fn rules_resolve(model: &Model, out: &mut Vec<Finding>) {
    let rules: BTreeSet<&str> = model.rules.iter().map(|r| r.id.as_str()).collect();
    for g in model.gaps.iter().flatten() {
        if !rules.contains(g.rule.as_str()) {
            out.push(at(
                Finding::error(
                    "gap.rule_unknown",
                    format!("rule {} is not defined in any rule file", g.rule),
                )
                .rule(&g.rule),
                g,
            ));
        }
    }
}
