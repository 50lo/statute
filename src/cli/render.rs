//! Text renderings of the reports.

use crate::commands::check::CheckReport;
use crate::commands::init::{AgentsMd, InitReport};
use crate::commands::link::LinkReport;
use crate::commands::show::{Item, Role, ShowReport};
use crate::commands::status::StatusReport;
use crate::model::{Agreed, Built, DecisionStatus, GapStatus, LinkRole, Severity};
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

pub fn link(r: &LinkReport) -> String {
    if r.removed {
        return format!("Removed the link from {} to {}\n", r.rule, r.target);
    }
    let role = match r.role {
        LinkRole::Code => "code",
        LinkRole::Test => "test",
    };
    let mut out = format!("Linked {} to {} ({role})\n", r.rule, r.target);
    if r.built {
        let _ = writeln!(out, "{} is complete and now `built: built`", r.rule);
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

pub fn show(r: &ShowReport) -> String {
    let mut blocks = Vec::new();
    for item in &r.items {
        let mut out = String::new();
        match item {
            Item::Rule(rule) => {
                let _ = writeln!(out, "{}:{}\n{}", rule.file, rule.line, rule.text);
                out.push_str("\nTerms:\n");
                if rule.terms.is_empty() {
                    out.push_str("  none\n");
                }
                for t in &rule.terms {
                    let definition = t.definition.as_deref().unwrap_or("(no definition)");
                    let _ = writeln!(out, "  {}: {}", t.name, one_line(definition));
                }
                out.push_str("\nDecisions:\n");
                match &rule.decisions {
                    None => out.push_str("  (decisions.db does not parse; run `statute check`)\n"),
                    Some(ds) if ds.is_empty() => out.push_str("  none\n"),
                    Some(ds) => {
                        for d in ds {
                            let role = match d.role {
                                Role::Establishes => "establishes",
                                Role::Changes => "changes",
                                Role::Retires => "retires",
                            };
                            let _ = writeln!(
                                out,
                                "  {} {role} ({}): {}",
                                d.id,
                                decision_status(d.status),
                                one_line(&d.summary)
                            );
                        }
                    }
                }
                out.push_str("\nGaps:\n");
                match &rule.gaps {
                    None => out.push_str("  (gaps.yml does not parse; run `statute check`)\n"),
                    Some(gaps) if gaps.is_empty() => out.push_str("  none\n"),
                    Some(gaps) => {
                        for g in gaps {
                            let status = match g.status {
                                GapStatus::Open => "open",
                                GapStatus::Closed => "closed",
                            };
                            let note = g
                                .note
                                .as_deref()
                                .map(|n| format!(": {}", one_line(n)))
                                .unwrap_or_default();
                            let _ =
                                writeln!(out, "  {} {status}, opened {}{note}", g.id, g.opened_at);
                        }
                    }
                }
            }
            Item::Term(term) => {
                let _ = writeln!(out, "{}:{}\n{}", term.file, term.line, term.text);
                out.push_str("\nRules:\n");
                if term.rules.is_empty() {
                    out.push_str("  none\n");
                }
                for rule in &term.rules {
                    let _ = writeln!(out, "  {} {}", rule.id, rule.title);
                }
            }
            Item::Decision(d) => {
                // The entry as it would read in decisions.db.
                match yaml_serde::to_string(std::slice::from_ref(d)) {
                    Ok(yaml) => out.push_str(&yaml),
                    Err(e) => {
                        let _ = writeln!(out, "{}: cannot render the entry: {e}", d.id);
                    }
                }
            }
        }
        blocks.push(out.trim_end().to_string());
    }
    blocks.join("\n\n")
}

fn decision_status(s: DecisionStatus) -> &'static str {
    match s {
        DecisionStatus::Proposed => "proposed",
        DecisionStatus::Active => "active",
    }
}

/// Collapses whitespace, so a multi-line value fits on one line of a list.
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}
