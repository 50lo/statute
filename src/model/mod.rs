//! Shared types: the parsed Statute files and the findings about them.

mod finding;
pub mod ids;

pub use finding::{Finding, Severity};

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Fixed paths of the Statute layout, relative to the repository root.
pub mod paths {
    pub const CONFIG: &str = "statute.yml";
    pub const LOCK: &str = "statute.lock";
    pub const TERMS: &str = "statute/terms.md";
    pub const RULES_DIR: &str = "statute/rules";
    pub const DECISIONS: &str = "statute/decisions.db";
    pub const GAPS: &str = "statute/gaps.yml";
    pub const AGENTS: &str = "AGENTS.md";

    /// The rule file of an area: `statute/rules/<area>.md`.
    pub fn rule_file(area: &str) -> String {
        format!("{RULES_DIR}/{area}.md")
    }
}

/// Everything Statute knows about a repository, parsed from its files.
#[derive(Debug, Default)]
pub struct Model {
    pub config: Config,
    pub rule_files: Vec<RuleFile>,
    pub rules: Vec<Rule>,
    pub terms: Vec<Term>,
    /// `None` when `decisions.db` does not parse; a finding says why.
    pub decisions: Option<Vec<Decision>>,
    /// `None` when `gaps.yml` does not parse; a finding says why.
    pub gaps: Option<Vec<Gap>>,
    /// Problems found while parsing, reported by `check` with the other findings.
    pub problems: Vec<Finding>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    #[serde(default = "default_summary_max")]
    pub summary_max: usize,
}

fn default_summary_max() -> usize {
    600
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            summary_max: default_summary_max(),
        }
    }
}

/// One file under `statute/rules/`.
#[derive(Debug, Clone)]
pub struct RuleFile {
    pub path: String,
    /// The file name without `.md`.
    pub area: String,
    /// From the frontmatter; `None` when missing or invalid.
    pub prefix: Option<String>,
}

/// A `#` section of a rule file.
#[derive(Debug, Clone)]
pub struct Rule {
    /// The first word of the heading, as written. `check` validates it.
    pub id: String,
    pub title: String,
    pub file: String,
    pub area: String,
    pub line: usize,
    /// The first paragraph, as written in the source.
    pub statement: Option<String>,
    /// `None` when the block is missing or does not match the schema.
    pub meta: Option<RuleMeta>,
    #[expect(
        dead_code,
        reason = "read by the commands that patch rule blocks, from milestone 2"
    )]
    pub span: Span,
}

/// Byte ranges of a section, for patches.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub heading: Range<usize>,
    /// The content of the `yaml` block, between the fences.
    pub block: Option<Range<usize>>,
    /// The section runs from `heading.start` to `end`.
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleMeta {
    pub agreed: Agreed,
    pub built: Built,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub terms: Vec<String>,
    #[serde(default)]
    pub exceptions: Vec<String>,
    #[serde(default)]
    pub protects: Vec<String>,
    #[serde(default)]
    pub boundary: Vec<String>,
    #[serde(default)]
    pub manual_check: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[expect(
    clippy::enum_variant_names,
    reason = "the values of the `agreed` field: `agreed: agreed`"
)]
pub enum Agreed {
    Proposed,
    Agreed,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Built {
    Planned,
    Built,
}

/// A `##` section of `terms.md`.
#[derive(Debug, Clone)]
pub struct Term {
    pub name: String,
    pub line: usize,
    #[cfg_attr(not(test), expect(dead_code, reason = "read by `show`"))]
    pub definition: Option<String>,
    /// Defaults when the term has no `yaml` block; `None` when the block does not match the schema.
    pub meta: Option<TermMeta>,
    #[expect(
        dead_code,
        reason = "read by `decide` and `ratify`, which patch the `agreed:` line"
    )]
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermMeta {
    /// Absent until `decide` first lists the term.
    #[serde(default)]
    pub agreed: Option<TermAgreed>,
    #[serde(default)]
    pub code: Vec<String>,
    #[serde(default)]
    pub not: Vec<NotName>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub avoid: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TermAgreed {
    Proposed,
    Agreed,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotName {
    pub name: String,
    pub means: String,
}

/// One entry of `decisions.db`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub id: String,
    pub timestamp: String,
    pub actor: Actor,
    pub status: DecisionStatus,
    #[serde(default)]
    pub ratified_by: Option<RatifiedBy>,
    pub summary: String,
    pub rationale: String,
    #[serde(default)]
    pub establishes: Vec<String>,
    #[serde(default)]
    pub changes: Vec<String>,
    #[serde(default)]
    pub retires: Vec<String>,
    #[serde(default)]
    pub terms: Vec<String>,
}

impl Decision {
    /// Every rule ID the decision names.
    pub fn rules(&self) -> impl Iterator<Item = &String> {
        self.establishes
            .iter()
            .chain(&self.changes)
            .chain(&self.retires)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    User,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionStatus {
    Proposed,
    Active,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RatifiedBy {
    pub name: String,
    pub email: String,
    pub at: String,
}

/// One entry of `gaps.yml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub id: String,
    pub rule: String,
    pub status: GapStatus,
    pub opened_at: String,
    /// Present for a gap kept in the gaps file; absent for a tracker issue.
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GapStatus {
    Open,
    Closed,
}
