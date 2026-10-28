//! Shared types: the parsed Statute files and the findings about them.

mod finding;
pub mod ids;
mod impact;

pub use finding::{Finding, Severity};
pub use impact::{Impact, PendingDecision, Reason, RuleImpact, TermImpact, Verdict};

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
    /// `None` when `statute.lock` does not parse; a finding says why.
    pub links: Option<Vec<Link>>,
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
    /// Everything in the section after the heading except the `yaml` block, as written; `None` when
    /// that is blank.
    pub definition: Option<String>,
    /// Defaults when the term has no `yaml` block; `None` when the block does not match the schema.
    pub meta: Option<TermMeta>,
    pub span: Span,
}

impl Term {
    /// The `agreed` field; `None` when it is absent or the block does not parse.
    pub fn agreed(&self) -> Option<TermAgreed> {
        self.meta.as_ref().and_then(|m| m.agreed)
    }

    /// A retired term is a record of what earlier decisions meant, not current vocabulary.
    pub fn is_retired(&self) -> bool {
        self.agreed() == Some(TermAgreed::Retired)
    }
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
    Retired,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotName {
    pub name: String,
    pub means: String,
}

/// One entry of `decisions.db`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub id: String,
    pub timestamp: String,
    pub actor: Actor,
    pub status: DecisionStatus,
    #[serde(default)]
    pub approved_by: Option<ApprovedBy>,
    pub summary: String,
    pub rationale: String,
    #[serde(default)]
    pub establishes: Vec<String>,
    #[serde(default)]
    pub changes: Vec<String>,
    #[serde(default)]
    pub retires: Vec<String>,
    /// Terms this decision introduces or redefines.
    #[serde(default)]
    pub terms: Vec<String>,
    /// Terms this decision retires. Omitted when empty, so entries written before the field existed
    /// render unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retires_terms: Vec<String>,
}

impl Decision {
    /// Every rule ID the decision names.
    pub fn rules(&self) -> impl Iterator<Item = &String> {
        self.establishes
            .iter()
            .chain(&self.changes)
            .chain(&self.retires)
    }

    /// Every term the decision names: the ones it introduces or redefines, then the ones it retires.
    pub fn all_terms(&self) -> impl Iterator<Item = &String> {
        self.terms.iter().chain(&self.retires_terms)
    }
}

/// What `decide` is asked to record: a decision before it has an ID, a time and a status. People
/// edit it as YAML with `decide --edit`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionDraft {
    pub actor: Actor,
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
    #[serde(default)]
    pub retires_terms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    User,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionStatus {
    Proposed,
    Active,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovedBy {
    pub name: String,
    pub email: String,
    pub at: String,
}

/// One entry of `gaps.yml`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub id: String,
    pub rule: String,
    pub status: GapStatus,
    pub opened_at: String,
    /// Present for a gap kept in the gaps file; absent for a tracker issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GapStatus {
    Open,
    Closed,
}

/// One line of `statute.lock`: a rule linked to a code or test target, with its fingerprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub source: LinkSource,
    /// `path` or `path#symbol`, relative to the repository root.
    pub target: String,
    pub role: LinkRole,
    pub mode: Mode,
    pub sig: String,
}

impl Link {
    /// The rule this link belongs to.
    pub fn rule(&self) -> &str {
        match &self.source {
            LinkSource::Rule(id) => id,
        }
    }

    /// The file part of the target.
    pub fn file(&self) -> &str {
        split_target(&self.target).0
    }
}

/// What a link belongs to. Only rules in v1.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinkSource {
    Rule(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkRole {
    Code,
    Test,
}

/// How a link's target is fingerprinted. A change to a mode's hash input is a new variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Mode {
    #[serde(rename = "bytes-sha256-v1")]
    BytesSha256V1,
    #[serde(rename = "go-ast-sha256-v1")]
    GoAstSha256V1,
    #[serde(rename = "python-ast-sha256-v1")]
    PythonAstSha256V1,
    #[serde(rename = "rust-ast-sha256-v1")]
    RustAstSha256V1,
}

/// Splits a target at the first `#` into its file and symbol. The symbol is opaque; its mode reads it.
pub fn split_target(target: &str) -> (&str, Option<&str>) {
    match target.split_once('#') {
        Some((file, symbol)) => (file, Some(symbol)),
        None => (target, None),
    }
}
