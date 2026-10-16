use clap::{Args, Parser, Subcommand};

/// Keep a product's rules written down, linked to code, and checked on every change.
#[derive(Parser)]
#[command(name = "statute", version)]
pub struct Cli {
    /// Print the report as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Write statute.yml, the default layout and the AGENTS.md snippet.
    Init,

    /// Create rules.
    #[command(subcommand)]
    Rule(RuleCommand),

    /// Show the rules for files, symbols, rule IDs, terms or decision IDs.
    Show {
        #[arg(required = true, value_name = "ITEM")]
        items: Vec<String>,
    },

    /// Run every check and report stale and missing links since the base.
    Check(CheckArgs),

    /// Add a link from a rule to code or a test, or remove one.
    Link(LinkArgs),

    /// Record one verdict for a rule and refresh its stale links.
    Review {
        rule: String,
        #[command(subcommand)]
        verdict: VerdictArg,
    },

    /// Append a proposed decision.
    Decide(DecideArgs),

    /// Activate decisions a person approved.
    Ratify {
        #[arg(required = true, value_name = "DECISION")]
        decisions: Vec<String>,
    },

    /// Open or close a gap.
    #[command(subcommand)]
    Gap(GapCommand),

    /// Summarise each area: rules, pending decisions, open gaps and manual checks.
    Status {
        #[arg(long)]
        area: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum RuleCommand {
    /// Append a rule skeleton to rules/<AREA>.md and print its ID.
    New { area: String, title: String },
}

#[derive(Args)]
pub struct CheckArgs {
    /// Re-fingerprint every link and look for term names across the repo.
    #[arg(long)]
    pub all: bool,

    /// Print the PR "Rule impact" section.
    #[arg(long, conflicts_with = "json")]
    pub md: bool,
}

#[derive(Args)]
pub struct LinkArgs {
    pub rule: String,
    /// `path` or `path#symbol`.
    pub target: String,
    /// The target is a test.
    #[arg(long)]
    pub test: bool,
    /// Remove the link instead of adding it.
    #[arg(long)]
    pub remove: bool,
}

#[derive(Subcommand)]
pub enum VerdictArg {
    /// The rule still holds.
    Ok,
    /// The rule changes on purpose, through this decision.
    Changed { decision: String },
    /// The rule is broken; an open gap records it.
    Gap {
        #[arg(value_name = "REF")]
        gap_ref: String,
    },
}

#[derive(Args)]
pub struct DecideArgs {
    #[arg(long)]
    pub summary: String,
    #[arg(long)]
    pub rationale: String,
    /// Rules this decision establishes.
    #[arg(long, value_delimiter = ',', value_name = "RULE")]
    pub establishes: Vec<String>,
    /// Rules whose text this decision changes.
    #[arg(long, value_delimiter = ',', value_name = "RULE")]
    pub changes: Vec<String>,
    /// Rules this decision retires.
    #[arg(long, value_delimiter = ',', value_name = "RULE")]
    pub retires: Vec<String>,
    /// Terms this decision introduces or redefines.
    #[arg(long, value_delimiter = ',', value_name = "TERM")]
    pub terms: Vec<String>,
    /// Open the new decision in $EDITOR.
    #[arg(long)]
    pub edit: bool,
}

#[derive(Subcommand)]
pub enum GapCommand {
    /// Add an open gap and print its ID.
    Open {
        rule: String,
        /// Tracker issue for the gap.
        #[arg(long = "ref", value_name = "REF")]
        gap_ref: Option<String>,
        /// What is wrong; required without --ref.
        #[arg(long)]
        note: Option<String>,
    },
    /// Close a gap.
    Close {
        #[arg(value_name = "REF")]
        gap_ref: String,
    },
}
