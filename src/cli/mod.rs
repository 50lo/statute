//! Argument parsing and rendering. Every command builds one report; `--json` prints it as is.

mod render;

use crate::commands;
use crate::model::LinkRole;
use clap::{Parser, Subcommand};
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
    /// Print rules, terms and decisions by rule ID, term name or decision ID
    Show {
        #[arg(required = true, value_name = "X")]
        names: Vec<String>,
    },
    /// Run every check, and report stale and missing links whose target changed since the base
    Check {
        /// Fingerprint every link, not only those whose target file changed
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
    /// Per area: rules by agreed and built state, and draft rules
    Status {
        /// Show only this area
        #[arg(long)]
        area: Option<String>,
    },
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
