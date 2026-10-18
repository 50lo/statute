//! The YAML files: `statute.yml`, `decisions.db` and `gaps.yml`.

#![expect(
    clippy::result_large_err,
    reason = "a parse error is rare and becomes a finding"
)]

use super::yaml_finding;
use crate::model::{Config, Decision, DecisionDraft, Finding, Gap, paths};
use serde::de::DeserializeOwned;

pub fn parse_config(text: &str) -> Result<Config, Finding> {
    yaml_serde::from_str(text)
        .map_err(|e| yaml_finding("config.parse", "statute.yml", &e, paths::CONFIG, 1))
}

pub fn parse_decisions(text: &str) -> Result<Vec<Decision>, Finding> {
    parse_list(text, "decision.parse", paths::DECISIONS)
}

pub fn parse_gaps(text: &str) -> Result<Vec<Gap>, Finding> {
    parse_list(text, "gap.parse", paths::GAPS)
}

/// An empty file is an empty list.
fn parse_list<T: DeserializeOwned>(
    text: &str,
    code: &'static str,
    path: &str,
) -> Result<Vec<T>, Finding> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    yaml_serde::from_str::<Option<Vec<T>>>(text)
        .map(Option::unwrap_or_default)
        .map_err(|e| yaml_finding(code, path, &e, path, 1))
}

/// `decisions.db` in its one fixed format. The CLI rewrites the whole file, so comments are not kept.
pub fn render_decisions(decisions: &[Decision]) -> String {
    yaml_serde::to_string(decisions).expect("decisions serialize")
}

/// `gaps.yml` in its one fixed format. The CLI rewrites the whole file, so comments are not kept.
pub fn render_gaps(gaps: &[Gap]) -> String {
    yaml_serde::to_string(gaps).expect("gaps serialize")
}

const DRAFT_HEADER: &str = "# The decision `statute decide` will record. Edit it, save and close.\n# An empty summary records nothing.\n";

/// A draft as YAML for `decide --edit`.
pub fn render_draft(draft: &DecisionDraft) -> String {
    format!(
        "{DRAFT_HEADER}{}",
        yaml_serde::to_string(draft).expect("drafts serialize")
    )
}

pub fn parse_draft(text: &str) -> Result<DecisionDraft, String> {
    yaml_serde::from_str(text).map_err(|e| e.to_string())
}

pub const CONFIG_TEMPLATE: &str =
    "version: 1\nsummary_max: 600      # decision summary cap, in characters\n";
pub const EMPTY_LIST: &str = "[]\n";
pub const LOCK_TEMPLATE: &str = "{\"version\": 1}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DecisionStatus;

    #[test]
    fn templates_parse() {
        assert_eq!(parse_config(CONFIG_TEMPLATE).unwrap(), Config::default());
        assert!(parse_decisions(EMPTY_LIST).unwrap().is_empty());
        assert!(parse_gaps("").unwrap().is_empty());
    }

    #[test]
    fn parses_a_decision() {
        let text = "- id: D261014-k7m2q\n  timestamp: \"2026-10-14T10:26:12Z\"\n  actor: agent\n  status: proposed\n  ratified_by: null\n  summary: \"S.\"\n  rationale: \"R.\"\n  establishes: [ATTR-001]\n";
        let decisions = parse_decisions(text).unwrap();
        assert_eq!(decisions[0].status, DecisionStatus::Proposed);
        assert_eq!(decisions[0].establishes, ["ATTR-001"]);
    }

    #[test]
    fn rendered_decisions_parse_back() {
        let text = "- id: D261014-k7m2q\n  timestamp: \"2026-10-14T10:26:12Z\"\n  actor: agent\n  status: active\n  ratified_by: {name: \"Pat: Example\", email: p@example.com, at: \"2026-10-15T09:00:00Z\"}\n  summary: \"yes\"\n  rationale: \"Line one.\\nLine two: with # and 'quotes'.\"\n  establishes: [ATTR-001, \"true\"]\n  terms: [Active customer]\n";
        let decisions = parse_decisions(text).unwrap();
        let rendered = render_decisions(&decisions);
        assert_eq!(parse_decisions(&rendered).unwrap(), decisions);
        assert_eq!(render_decisions(&[]), EMPTY_LIST);
    }

    #[test]
    fn drafts_parse_back() {
        let draft = DecisionDraft {
            actor: crate::model::Actor::User,
            summary: "S: s.".into(),
            rationale: "R.\nMore.".into(),
            establishes: vec!["BILL-001".into()],
            changes: vec![],
            retires: vec![],
            terms: vec!["Approver".into()],
            retires_terms: vec!["Old approver".into()],
        };
        assert_eq!(parse_draft(&render_draft(&draft)).unwrap(), draft);
        let empty = DecisionDraft {
            retires_terms: vec![],
            ..draft
        };
        assert!(render_draft(&empty).contains("retires_terms: []"));
    }

    #[test]
    fn entries_without_retired_terms_render_as_before() {
        let text = "- id: D261014-k7m2q\n  timestamp: 2026-10-14T10:26:12Z\n  actor: agent\n  status: proposed\n  ratified_by: null\n  summary: S.\n  rationale: R.\n  establishes: []\n  changes: []\n  retires: []\n  terms:\n  - Session\n";
        let decisions = parse_decisions(text).unwrap();
        assert!(decisions[0].retires_terms.is_empty());
        assert_eq!(render_decisions(&decisions), text);

        let retiring = text.replace(
            "  - Session\n",
            "  - Session\n  retires_terms:\n  - Old session\n",
        );
        let decisions = parse_decisions(&retiring).unwrap();
        assert_eq!(decisions[0].retires_terms, ["Old session"]);
        assert_eq!(render_decisions(&decisions), retiring);
    }

    #[test]
    fn a_misspelled_retired_terms_field_is_an_error() {
        let err = parse_decisions("- id: D261014-k7m2q\n  timestamp: \"2026-10-14T10:26:12Z\"\n  actor: agent\n  status: proposed\n  summary: S.\n  rationale: R.\n  retire_terms: [Session]\n").unwrap_err();
        assert_eq!(err.code, "decision.parse");
        assert_eq!(err.line, Some(7));
    }

    #[test]
    fn errors_carry_the_line() {
        let err = parse_decisions("- id: D261014-k7m2q\n  stauts: proposed\n").unwrap_err();
        assert_eq!(err.code, "decision.parse");
        assert_eq!(err.line, Some(2));
    }

    #[test]
    fn unknown_config_keys_are_errors() {
        assert_eq!(
            parse_config("version: 1\nsumary_max: 5\n")
                .unwrap_err()
                .code,
            "config.parse"
        );
    }
}
