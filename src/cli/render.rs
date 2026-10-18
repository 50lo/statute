//! Text renderings of the reports.

use crate::commands::check::CheckReport;
use crate::commands::decide::DecideReport;
use crate::commands::init::{AgentsMd, InitReport};
use crate::commands::link::LinkReport;
use crate::commands::ratify::RatifyReport;
use crate::commands::review::ReviewReport;
use crate::commands::show::{Item, Role, ShowReport};
use crate::commands::status::StatusReport;
use crate::model::{
    Agreed, Built, DecisionStatus, GapStatus, Impact, LinkRole, Reason, Severity, Verdict,
};
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

/// The PR "Rule impact" section, in Markdown a PM or designer can follow.
pub fn impact(r: &Impact) -> String {
    let mut out = String::from("## Rule impact\n\n");
    if r.rules.is_empty() && r.terms.is_empty() {
        out.push_str("This change affects no rules or terms.\n");
    }
    if !r.rules.is_empty() {
        out.push_str("| Rule | Verdict | Why |\n| --- | --- | --- |\n");
        for rule in &r.rules {
            let verdict = match rule.verdict {
                Verdict::Unreviewed => "Not reviewed yet",
                Verdict::Broken => "Broken: a gap is open",
                Verdict::New => "New",
                Verdict::Changed => "Changed",
                Verdict::Retired => "Retired",
                Verdict::Relinked => "Relinked",
                Verdict::Holds => "Still holds",
            };
            let _ = writeln!(
                out,
                "| **{}** {} | {verdict} | {} |",
                rule.rule,
                cell(&rule.title),
                reasons(&rule.reasons)
            );
        }
    }
    if !r.terms.is_empty() {
        if !r.rules.is_empty() {
            out.push('\n');
        }
        out.push_str("| Term | Why |\n| --- | --- |\n");
        for term in &r.terms {
            let _ = writeln!(out, "| {} | {} |", cell(&term.term), reasons(&term.reasons));
        }
    }
    if !r.awaiting_approval.is_empty() {
        out.push_str("\n### Awaiting approval\n\n");
        for d in &r.awaiting_approval {
            let mut names = d.rules.clone();
            names.extend(d.terms.iter().map(|t| format!("term {t}")));
            let _ = writeln!(
                out,
                "- {}: {} ({})",
                d.id,
                d.summary.trim(),
                names.join(", ")
            );
        }
    }
    out
}

fn reasons(reasons: &[Reason]) -> String {
    let text: Vec<String> = reasons
        .iter()
        .map(|r| match r {
            Reason::LinkStale { target } => format!("`{target}` changed and is not reviewed yet"),
            Reason::LinkMissing { target } => format!("`{target}` is missing"),
            Reason::LinkAdded { target } => format!("now linked to `{target}`"),
            Reason::LinkRemoved { target } => format!("no longer linked to `{target}`"),
            Reason::LinkRefreshed { target } => format!("`{target}` changed and was reviewed"),
            Reason::DecisionChanged { decision } => format!("decision {decision}"),
            Reason::GapChanged { gap } => format!("gap {gap}"),
        })
        .collect();
    cell(&text.join("; "))
}

/// Text safe inside a Markdown table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

pub fn decide(r: &DecideReport) -> String {
    let mut out = r.id.clone();
    for rule in &r.proposed_rules {
        out.push_str(&format!("\n{rule} is proposed until {} is ratified", r.id));
    }
    for term in &r.proposed_terms {
        out.push_str(&format!(
            "\nterm `{term}` is proposed until {} is ratified",
            r.id
        ));
    }
    out
}

pub fn ratify(r: &RatifyReport) -> String {
    let mut out = format!(
        "Ratified {} by {} <{}>",
        r.decisions.join(", "),
        r.ratified_by.name,
        r.ratified_by.email
    );
    let mut line = |label: &str, items: &[String]| {
        if !items.is_empty() {
            out.push_str(&format!("\n{label}: {}", items.join(", ")));
        }
    };
    line("Agreed rules", &r.agreed_rules);
    line("Retired rules", &r.retired_rules);
    line("Agreed terms", &r.agreed_terms);
    line("Built rules", &r.built_rules);
    if r.removed_links > 0 {
        out.push_str(&format!(
            "\nRemoved {} links of retired rules",
            r.removed_links
        ));
    }
    out
}

pub fn link(r: &LinkReport) -> String {
    if r.removed {
        return format!("Removed the link from {} to {}\n", r.rule, r.target);
    }
    let mut out = format!("Linked {} to {} ({})\n", r.rule, r.target, role(r.role));
    if r.built {
        let _ = writeln!(out, "{} is complete and now `built: built`", r.rule);
    }
    out
}

pub fn review(r: &ReviewReport) -> String {
    if r.refreshed.is_empty() {
        return format!("{} has no stale links; nothing to refresh\n", r.rule);
    }
    let mut out = format!(
        "Refreshed {} {} of {}:\n",
        r.refreshed.len(),
        plural(r.refreshed.len(), "link"),
        r.rule
    );
    for target in &r.refreshed {
        let _ = writeln!(out, "  {target}");
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
            let mut notes = Vec::new();
            if rule.draft {
                notes.push("draft".to_string());
            }
            if rule.incomplete {
                notes.push("incomplete".to_string());
            }
            if rule.broken {
                notes.push("broken".to_string());
            }
            if !rule.open_gaps.is_empty() {
                notes.push(format!("open gaps: {}", rule.open_gaps.join(", ")));
            }
            for (targets, what) in [(&rule.stale, "stale"), (&rule.missing, "missing")] {
                if !targets.is_empty() {
                    notes.push(format!("{what}: {}", targets.join(", ")));
                }
            }
            let notes = if notes.is_empty() {
                String::new()
            } else {
                format!("  ({})", notes.join("; "))
            };
            let _ = writeln!(
                out,
                "  {:width$}  {agreed:<8}  {built:<7}  {}{notes}",
                rule.id, rule.title
            );
        }
    }
    out.push('\n');
    out.push_str(&impact(&r.impact));
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
                    term_definition(&mut out, &t.name, definition);
                }
                out.push_str("\nLinks:\n");
                match &rule.links {
                    None => out.push_str("  (statute.lock does not parse; run `statute check`)\n"),
                    Some(links) if links.is_empty() => out.push_str("  none\n"),
                    Some(links) => {
                        for l in links {
                            let _ = writeln!(out, "  {} {}", role(l.role), l.target);
                        }
                    }
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
    for name in &r.unlinked {
        blocks.push(format!("No rules are linked to {name}."));
    }
    blocks.join("\n\n")
}

fn role(r: LinkRole) -> &'static str {
    match r {
        LinkRole::Code => "code",
        LinkRole::Test => "test",
    }
}

fn decision_status(s: DecisionStatus) -> &'static str {
    match s {
        DecisionStatus::Proposed => "proposed",
        DecisionStatus::Active => "active",
    }
}

/// Collapses whitespace, so a multi-line value fits on one line of a list.
/// The first paragraph on the term's line, then the rest of the definition indented below it, line by
/// line so lists, tables and code keep their shape.
fn term_definition(out: &mut String, name: &str, definition: &str) {
    let (first, rest) = definition.split_once("\n\n").unwrap_or((definition, ""));
    let _ = writeln!(out, "  {name}: {}", one_line(first));
    for line in rest.lines().filter(|l| !l.trim().is_empty()) {
        let _ = writeln!(out, "    {}", line.trim_end());
    }
}

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
