//! The YAML files: `statute.yml`, `decisions.db` and `gaps.yml`.

#![expect(
    clippy::result_large_err,
    reason = "a parse error is rare and becomes a finding"
)]

use super::yaml_finding;
use crate::model::{Config, Decision, Finding, Gap, paths};
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
