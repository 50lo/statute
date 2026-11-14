//! Derived state, computed from the model and never stored.

use crate::fingerprint::Missing;
use crate::model::{
    Agreed, Built, GapStatus, Link, LinkRole, Mode, Model, Rule, Term, TermAction, TermAgreed,
    TermRefusal,
};
use std::collections::BTreeMap;

/// The terms a decision names, each with what the decision does to it: `defines` is its `terms` list
/// and `retires` its `retires_terms` list, in that order.
pub fn term_actions<'a>(
    defines: &'a [String],
    retires: &'a [String],
) -> impl Iterator<Item = (&'a String, TermAction)> {
    let defines = defines.iter().map(|name| (name, TermAction::Define));
    let retires = retires.iter().map(|name| (name, TermAction::Retire));
    defines.chain(retires)
}

/// Why a decision cannot take `action` on a term, judged by the term's own state; `None` when it can.
/// A retired term takes no action, and only an agreed term can be retired. `term` is `None` when no
/// term has the name: that does not stop a decision from defining it, but a term that does not exist
/// is not agreed, so it cannot be retired. Whether one decision, or one `approve` call, both defines
/// and retires a term is the caller's check.
pub fn term_refusal(term: Option<&Term>, action: TermAction) -> Option<TermRefusal> {
    if term.is_some_and(Term::is_retired) {
        return Some(TermRefusal::Retired);
    }
    match action {
        TermAction::Define => None,
        TermAction::Retire if term.is_some_and(|t| t.meta.is_none()) => {
            Some(TermRefusal::Unreadable)
        }
        TermAction::Retire if term.and_then(Term::agreed) != Some(TermAgreed::Agreed) => {
            Some(TermRefusal::NotAgreed)
        }
        TermAction::Retire => None,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Span, TermMeta};

    fn term(meta: Option<TermMeta>) -> Term {
        Term {
            name: "Session".to_string(),
            line: 1,
            definition: None,
            meta,
            span: Span {
                heading: 0..0,
                block: None,
                end: 0,
            },
        }
    }

    fn with(agreed: Option<TermAgreed>) -> Term {
        term(Some(TermMeta {
            agreed,
            ..TermMeta::default()
        }))
    }

    #[test]
    fn a_term_takes_an_action_only_from_the_states_that_allow_it() {
        use TermAction::{Define, Retire};
        use TermRefusal::{NotAgreed, Retired, Unreadable};
        let cases = [
            (None, None, Some(NotAgreed)),
            (Some(term(None)), None, Some(Unreadable)),
            (Some(with(None)), None, Some(NotAgreed)),
            (
                Some(with(Some(TermAgreed::Proposed))),
                None,
                Some(NotAgreed),
            ),
            (Some(with(Some(TermAgreed::Agreed))), None, None),
            (
                Some(with(Some(TermAgreed::Retired))),
                Some(Retired),
                Some(Retired),
            ),
        ];
        for (term, define, retire) in cases {
            assert_eq!(term_refusal(term.as_ref(), Define), define, "{term:?}");
            assert_eq!(term_refusal(term.as_ref(), Retire), retire, "{term:?}");
        }
    }

    #[test]
    fn term_actions_lists_defined_terms_before_retired_ones() {
        let defines = ["Commit".to_string()];
        let retires = ["Session".to_string()];
        let actions: Vec<_> = term_actions(&defines, &retires)
            .map(|(name, action)| (name.as_str(), action))
            .collect();
        assert_eq!(
            actions,
            [
                ("Commit", TermAction::Define),
                ("Session", TermAction::Retire)
            ]
        );
    }
}
