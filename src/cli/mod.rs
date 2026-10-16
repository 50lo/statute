//! Argument parsing and rendering. Every command builds one report; `--json` prints it as is.

mod render;

use crate::commands;
use crate::commands::review::Verdict;
use crate::model::{Actor, DecisionDraft, LinkRole};
use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "statute",
    version,
    about = "Keeps product rules written down, linked to code, and checked"
)]
struct Cli {
    /// Print the report as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write statute.yml, the default layout and the AGENTS.md snippet
    Init,
    /// Work with rules
    Rule {
        #[command(subcommand)]
        command: RuleCommand,
    },
    /// Print rules, terms and decisions by rule ID, term name or decision ID, and the rules linked to a file or symbol
    Show {
        #[arg(required = true, value_name = "X")]
        names: Vec<String>,
    },
    /// Run every check, and report stale and missing links whose target changed since the base
    Check {
        /// Fingerprint every link, and look for term `code` and `not` names across the repository
        #[arg(long)]
        all: bool,
    },
    /// Link a rule to a code or test target with its current fingerprint, or remove a link
    Link {
        rule: String,
        /// A repo-relative file, or a symbol in one: `path#Symbol`
        target: String,
        /// The target is a test that shows the rule holds
        #[arg(long)]
        test: bool,
        /// Remove the link instead
        #[arg(long, conflicts_with = "test")]
        remove: bool,
    },
    /// Record a verdict on a rule whose links went stale, and refresh every stale link of the rule
    Review {
        rule: String,
        #[command(subcommand)]
        verdict: VerdictArg,
    },
    /// Append a proposed decision, set its changed rules and its terms to proposed, and print its ID
    Decide {
        /// One to three sentences
        #[arg(long, required_unless_present = "edit")]
        summary: Option<String>,
        /// Why this alternative, and what was rejected
        #[arg(long, required_unless_present = "edit")]
        rationale: Option<String>,
        /// New rules, comma-separated
        #[arg(long, value_delimiter = ',', value_name = "RULES")]
        establishes: Vec<String>,
        /// Rules whose text this decision changes
        #[arg(long, value_delimiter = ',', value_name = "RULES")]
        changes: Vec<String>,
        /// Rules this decision retires
        #[arg(long, value_delimiter = ',', value_name = "RULES")]
        retires: Vec<String>,
        /// Terms this decision introduces or redefines
        #[arg(long, value_delimiter = ',', value_name = "TERMS")]
        terms: Vec<String>,
        /// Who initiated the decision; informational only
        #[arg(long, value_enum, default_value = "agent")]
        actor: ActorArg,
        /// Open the decision in $VISUAL or $EDITOR before recording it
        #[arg(long)]
        edit: bool,
    },
    /// Per area: rules by agreed and built state, drafts, incomplete rules, stale and missing links
    Status {
        /// Show only this area
        #[arg(long)]
        area: Option<String>,
    },
}

#[derive(Subcommand)]
enum VerdictArg {
    /// The rule still holds
    Ok,
    /// The change is on purpose: the decision lists the rule in `changes` or `retires`
    Changed {
        #[arg(value_name = "DECISION")]
        decision: String,
    },
    /// The rule no longer holds: the rule has an open gap with this reference
    Gap {
        #[arg(value_name = "REF")]
        reference: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ActorArg {
    User,
    Agent,
}

#[derive(Subcommand)]
enum RuleCommand {
    /// Append a rule skeleton to statute/rules/<AREA>.md with the next free ID, and print the ID
    New { area: String, title: String },
}

/// Exit codes: 0 clean, 1 findings, 2 usage or file error.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    let result = match &cli.command {
        Command::Init => commands::init::run().map(|r| emit(json, &r, render::init(&r), 0)),
        Command::Rule {
            command: RuleCommand::New { area, title },
        } => commands::rule_new::run(area, title).map(|r| emit(json, &r, r.id.clone(), 0)),
        Command::Show { names } => {
            commands::show::run(names).map(|r| emit(json, &r, render::show(&r), 0))
        }
        Command::Check { all } => commands::check::run(*all)
            .map(|r| emit(json, &r, render::check(&r), u8::from(r.failed()))),
        Command::Link {
            rule,
            target,
            test,
            remove,
        } => {
            let report = if *remove {
                commands::link::remove(rule, target)
            } else {
                let role = if *test {
                    LinkRole::Test
                } else {
                    LinkRole::Code
                };
                commands::link::add(rule, target, role)
            };
            report.map(|r| emit(json, &r, render::link(&r), 0))
        }
        Command::Review { rule, verdict } => {
            let verdict = match verdict {
                VerdictArg::Ok => Verdict::Ok,
                VerdictArg::Changed { decision } => Verdict::Changed(decision.clone()),
                VerdictArg::Gap { reference } => Verdict::Gap(reference.clone()),
            };
            commands::review::run(rule, verdict).map(|r| emit(json, &r, render::review(&r), 0))
        }
        Command::Decide {
            summary,
            rationale,
            establishes,
            changes,
            retires,
            terms,
            actor,
            edit,
        } => {
            let draft = DecisionDraft {
                actor: match actor {
                    ActorArg::User => Actor::User,
                    ActorArg::Agent => Actor::Agent,
                },
                summary: summary.clone().unwrap_or_default(),
                rationale: rationale.clone().unwrap_or_default(),
                establishes: establishes.clone(),
                changes: changes.clone(),
                retires: retires.clone(),
                terms: terms.clone(),
            };
            let draft = if *edit { edit_draft(&draft) } else { Ok(draft) };
            draft
                .and_then(commands::decide::run)
                .map(|r| emit(json, &r, render::decide(&r), 0))
        }
        Command::Status { area } => {
            commands::status::run(area.as_deref()).map(|r| emit(json, &r, render::status(&r), 0))
        }
    };
    result.unwrap_or_else(|e| {
        if json {
            let report = serde_json::json!({"schema": commands::SCHEMA, "error": format!("{e:#}")});
            println!("{report:#}");
        } else {
            eprintln!("statute: {e:#}");
        }
        ExitCode::from(2)
    })
}

/// Lets a person edit the draft in their editor. `cli` launches the editor; `commands` writes and
/// reads the file.
fn edit_draft(draft: &DecisionDraft) -> Result<DecisionDraft> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    let path = commands::decide::draft_file(draft)?;
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("sh")
        .arg(&path)
        .status();
    let edited = commands::decide::read_draft_file(&path);
    match status {
        Ok(s) if s.success() => edited,
        Ok(s) => bail!("the editor `{editor}` exited with {s}; no decision was recorded"),
        Err(e) => bail!("cannot run the editor `{editor}`: {e}"),
    }
}

fn emit<R: Serialize>(json: bool, report: &R, text: String, code: u8) -> ExitCode {
    if json {
        match serde_json::to_string_pretty(report) {
            Ok(out) => println!("{out}"),
            Err(e) => {
                eprintln!("statute: cannot render the report: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        println!("{}", text.trim_end());
    }
    ExitCode::from(code)
}
