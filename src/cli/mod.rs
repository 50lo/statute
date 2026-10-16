//! Argument parsing and rendering. Decides nothing: every outcome comes from `commands`.

mod args;
mod render;

use std::process::ExitCode;

use clap::Parser;

use crate::commands::{self, review::Verdict};
use crate::model::Report;
use args::{Cli, Command, GapCommand, RuleCommand, VerdictArg};
use render::Format;

/// Exit codes from spec section 5: 0 clean, 1 findings, 2 usage or file error.
/// Clap exits with 2 on its own for usage errors.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let format = match &cli.command {
        _ if cli.json => Format::Json,
        Command::Check(args) if args.md => Format::Markdown,
        _ => Format::Text,
    };
    match dispatch(cli.command) {
        Ok(report) => {
            render::report(&report, format);
            if report.has_errors() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            render::error(&err, format);
            ExitCode::from(2)
        }
    }
}

fn dispatch(command: Command) -> anyhow::Result<Report> {
    match command {
        Command::Init => commands::init::run(),
        Command::Rule(RuleCommand::New { area, title }) => commands::rule_new::run(&area, &title),
        Command::Show { items } => commands::show::run(&items),
        Command::Check(args) => commands::check::run(&commands::check::Options {
            all: args.all,
            impact: args.md,
        }),
        Command::Link(args) => commands::link::run(&commands::link::Options {
            rule: args.rule,
            target: args.target,
            test: args.test,
            remove: args.remove,
        }),
        Command::Review { rule, verdict } => {
            let verdict = match verdict {
                VerdictArg::Ok => Verdict::Ok,
                VerdictArg::Changed { decision } => Verdict::Changed { decision },
                VerdictArg::Gap { gap_ref } => Verdict::Gap { gap_ref },
            };
            commands::review::run(&rule, &verdict)
        }
        // `--edit` is handled here once decide exists: cli launches $EDITOR (AGENTS.md).
        Command::Decide(args) => commands::decide::run(&commands::decide::Options {
            summary: args.summary,
            rationale: args.rationale,
            establishes: args.establishes,
            changes: args.changes,
            retires: args.retires,
            terms: args.terms,
        }),
        Command::Ratify { decisions } => commands::ratify::run(&decisions),
        Command::Gap(GapCommand::Open {
            rule,
            gap_ref,
            note,
        }) => commands::gap::open(&rule, gap_ref.as_deref(), note.as_deref()),
        Command::Gap(GapCommand::Close { gap_ref }) => commands::gap::close(&gap_ref),
        Command::Status { area } => commands::status::run(area.as_deref()),
    }
}
