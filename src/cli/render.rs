//! Text renderings of the reports.

use crate::commands::check::CheckReport;
use crate::commands::init::{AgentsMd, InitReport};
use crate::commands::status::StatusReport;
use crate::model::{Agreed, Built, Severity};
use std::fmt::Write;

pub fn init(r: &InitReport) -> String {
    let mut out = format!("Set up Statute in {}\n", r.root);
    for path in &r.created {
        let _ = writeln!(out, "  created {path}");
    }
    for path in &r.kept {
        let _ = writeln!(out, "  kept    {path}");
    }
    if matches!(r.agents_md, AgentsMd::Updated) {
        out.push_str("  added the Statute snippet to AGENTS.md\n");
    }
    out.push_str(
        "Next: create statute/rules/<area>.md with `prefix: <PREFIX>` in its frontmatter.\n",
    );
    out
}

pub fn check(r: &CheckReport) -> String {
    let mut out = String::new();
    for f in &r.findings {
        let severity = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        // The item the finding is about comes first; others it names follow in the message.
        let own = match f.code.split('.').next() {
            Some("term") => &f.term,
            Some("decision") => &f.decision,
            Some("gap") => &f.gap,
            _ => &f.rule,
        };
        let subject = own.as_ref().or(f.rule.as_ref()).or(f.term.as_ref());
        let subject = subject.map(|s| format!(" {s}:")).unwrap_or_default();
        let _ = writeln!(out, "{severity}[{}]{subject} {}", f.code, f.message);
        if let Some(file) = &f.file {
            let line = f.line.map(|l| format!(":{l}")).unwrap_or_default();
            let _ = writeln!(out, "  --> {file}{line}");
        }
    }
    if r.findings.is_empty() {
        out.push_str("No findings.\n");
    } else {
        let _ = writeln!(
            out,
            "{} {}, {} {}",
            r.errors,
            plural(r.errors, "error"),
            r.warnings,
            plural(r.warnings, "warning")
        );
    }
    out
}

pub fn status(r: &StatusReport) -> String {
    if r.areas.is_empty() {
        return "No rule files yet: create statute/rules/<area>.md with `prefix: <PREFIX>` in its frontmatter.\n".into();
    }
    let mut out = String::new();
    for area in &r.areas {
        let prefix = area
            .prefix
            .as_deref()
            .map(|p| format!(" ({p})"))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{}{prefix}: {} {}",
            area.area,
            area.rules.len(),
            plural(area.rules.len(), "rule")
        );
        let width = area.rules.iter().map(|r| r.id.len()).max().unwrap_or(0);
        for rule in &area.rules {
            let agreed = match rule.agreed {
                Some(Agreed::Proposed) => "proposed",
                Some(Agreed::Agreed) => "agreed",
                Some(Agreed::Retired) => "retired",
                None => "?",
            };
            let built = match rule.built {
                Some(Built::Planned) => "planned",
                Some(Built::Built) => "built",
                None => "?",
            };
            let draft = if rule.draft { "  (draft)" } else { "" };
            let _ = writeln!(
                out,
                "  {:width$}  {agreed:<8}  {built:<7}  {}{draft}",
                rule.id, rule.title
            );
        }
    }
    out
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}
