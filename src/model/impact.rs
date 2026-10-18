//! The PR "Rule impact" section: what a branch did to rules and terms, compared with the base.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Impact {
    pub rules: Vec<RuleImpact>,
    pub terms: Vec<TermImpact>,
    /// Proposed decisions that name an affected rule or term.
    pub awaiting_approval: Vec<PendingDecision>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuleImpact {
    pub rule: String,
    pub title: String,
    pub verdict: Verdict,
    pub reasons: Vec<Reason>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TermImpact {
    pub term: String,
    pub reasons: Vec<Reason>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PendingDecision {
    pub id: String,
    pub summary: String,
    pub rules: Vec<String>,
    /// Terms the decision introduces or redefines.
    pub terms: Vec<String>,
    /// Terms the decision retires; they stay agreed until it is ratified.
    pub retires_terms: Vec<String>,
}

/// Why a rule or term is affected. A new kind of reason is a new variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Reason {
    /// A linked target changed and the rule is not reviewed yet.
    LinkStale {
        target: String,
    },
    /// A linked target is not found, is ambiguous, or does not parse.
    LinkMissing {
        target: String,
    },
    LinkAdded {
        target: String,
    },
    LinkRemoved {
        target: String,
    },
    /// `review` refreshed the link's fingerprint.
    LinkRefreshed {
        target: String,
    },
    /// A decision that names it was added or changed. For a term, one that introduces or redefines it.
    DecisionChanged {
        decision: String,
    },
    /// A proposed decision that retires the term was added or changed. The term stays agreed until
    /// the decision is ratified.
    RetirementProposed {
        decision: String,
    },
    /// A decision that retires the term became active since the base.
    Retired {
        decision: String,
    },
    /// A gap on the rule was added or changed.
    GapChanged {
        gap: String,
    },
}

/// The verdict on an affected rule, decided in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// A link is still stale or missing.
    Unreviewed,
    /// A new open gap.
    Broken,
    /// A new decision establishes it.
    New,
    /// A new decision changes it.
    Changed,
    /// A new decision retires it.
    Retired,
    /// A link was added or removed.
    Relinked,
    Holds,
}
