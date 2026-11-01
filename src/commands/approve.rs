use super::SCHEMA;
use crate::check;
use crate::git;
use crate::model::{
    Agreed, ApprovedBy, Built, Decision, DecisionStatus, Finding, Model, Rule, Severity,
    TermAgreed, ids, paths,
};
use crate::repo::Repo;
use crate::state;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct ApproveReport {
    pub schema: u32,
    pub decisions: Vec<String>,
    pub approved_by: ApprovedBy,
    /// Rules that became `agreed: agreed`.
    pub agreed_rules: Vec<String>,
    pub retired_rules: Vec<String>,
    pub agreed_terms: Vec<String>,
    pub retired_terms: Vec<String>,
    /// Rules this command set to `built: built`, because they are complete.
    pub built_rules: Vec<String>,
    /// Links removed from retired rules.
    pub removed_links: usize,
}

/// Makes the decisions active, and updates their rules and terms. Refuses when `check` would report an
/// error about the decisions, their rules or their terms afterwards.
pub fn run(ids: &[String]) -> Result<ApproveReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let Some(decisions) = &model.decisions else {
        bail!("{} does not parse; run `statute check`", paths::DECISIONS);
    };
    let Some(links) = &model.links else {
        bail!("{} does not parse; run `statute check`", paths::LOCK);
    };
    let ids: Vec<&String> = ids.iter().collect::<BTreeSet<_>>().into_iter().collect();
    for id in &ids {
        match decisions.iter().find(|d| &&d.id == id) {
            None => bail!("no decision {id}"),
            Some(d) if d.status == DecisionStatus::Active => bail!("{id} is already active"),
            Some(_) => {}
        }
    }
    let Some((name, email)) = git::user(&repo.root)? else {
        bail!(
            "git `user.name` and `user.email` must be set; `approve` records who approved the decisions"
        );
    };
    let approved_by = ApprovedBy {
        name,
        email,
        at: ids::timestamp(jiff::Timestamp::now()),
    };

    let chosen: Vec<_> = decisions.iter().filter(|d| ids.contains(&&d.id)).collect();
    let mut agreed_rules = Vec::new();
    let mut retired_rules = Vec::new();
    let mut agreed_terms = Vec::new();
    let mut retired_terms = Vec::new();
    for d in &chosen {
        agreed_rules.extend(d.establishes.iter().chain(&d.changes).cloned());
        retired_rules.extend(d.retires.iter().cloned());
        agreed_terms.extend(d.terms.iter().cloned());
        retired_terms.extend(d.retires_terms.iter().cloned());
    }
    refuse_invalid_term_actions(&model, &chosen)?;

    let mut txn = Txn::new(&repo);
    for id in &agreed_rules {
        patch_rule(&mut txn, &model, id, "agreed: agreed", |t, r| {
            store::rules::set_agreed(t, r, Agreed::Agreed)
        })?;
    }
    for id in &retired_rules {
        patch_rule(&mut txn, &model, id, "agreed: retired", |t, r| {
            store::rules::set_agreed(t, r, Agreed::Retired)
        })?;
    }

    // Links of retired rules go; the rules that are complete with the remaining links become built.
    let kept: Vec<_> = links
        .iter()
        .filter(|l| !retired_rules.iter().any(|r| r == l.rule()))
        .cloned()
        .collect();
    let removed_links = links.len() - kept.len();
    if removed_links > 0 {
        txn.write(paths::LOCK, store::lock::render(&kept));
    }
    let mut built_rules = Vec::new();
    for id in &agreed_rules {
        let Some(rule) = model.rules.iter().find(|r| &r.id == id) else {
            continue;
        };
        let planned = rule
            .meta
            .as_ref()
            .is_some_and(|m| m.built == Built::Planned);
        if planned && state::is_complete(rule, &kept) {
            patch_rule(&mut txn, &model, id, "built: built", |t, r| {
                store::rules::set_built(t, r, Built::Built)
            })?;
            built_rules.push(id.clone());
        }
    }

    let term_states = agreed_terms
        .iter()
        .map(|t| (t, TermAgreed::Agreed))
        .chain(retired_terms.iter().map(|t| (t, TermAgreed::Retired)));
    for (name, agreed) in term_states {
        let Some(term) = model.terms.iter().find(|t| &t.name == name) else {
            bail!("no term `{name}`; run `statute check`");
        };
        let text = txn.text(paths::TERMS)?;
        let Some(patched) = store::terms::set_agreed(&text, term, agreed) else {
            let value = match agreed {
                TermAgreed::Retired => "retired",
                _ => "agreed",
            };
            bail!(
                "cannot set `agreed: {value}` on term `{name}`: write the field on one line in a block mapping, then approve again"
            );
        };
        txn.write(paths::TERMS, patched);
    }

    let mut approved = decisions.clone();
    for d in approved.iter_mut().filter(|d| ids.contains(&&d.id)) {
        d.status = DecisionStatus::Active;
        d.approved_by = Some(approved_by.clone());
    }
    txn.write(paths::DECISIONS, store::render_decisions(&approved));

    // The checks `check` runs, on the result. Only errors about what is being approved block it.
    let after = repo.load_with(txn.writes())?;
    let mut findings = check::run(&after);
    let (rev, base) = repo.load_base()?;
    if rev.is_some() {
        check::history(&after, &base, &mut findings);
    }
    let about = |f: &Finding| {
        f.decision.as_ref().is_some_and(|d| ids.contains(&d))
            || f.rule
                .as_ref()
                .is_some_and(|r| agreed_rules.contains(r) || retired_rules.contains(r))
            || f.term
                .as_ref()
                .is_some_and(|t| agreed_terms.contains(t) || retired_terms.contains(t))
    };
    let mut blocking: Vec<String> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && about(f))
        .map(|f| format!("  {}: {}", f.code, f.message))
        .collect();
    // A rule becomes agreed only with agreed terms: agreed before, or approved in this batch. A retired
    // term is reported above as `rule.term_retired`.
    for rule in after.rules.iter().filter(|r| agreed_rules.contains(&r.id)) {
        for name in rule.meta.iter().flat_map(|m| &m.terms) {
            let agreed = after
                .terms
                .iter()
                .any(|t| &t.name == name && t.agreed() == Some(TermAgreed::Agreed));
            let known = after
                .terms
                .iter()
                .any(|t| &t.name == name && !t.is_retired());
            if known && !agreed {
                blocking.push(format!(
                    "  rule.term_not_agreed: {} uses term `{name}`, which is not agreed; approve the decision that lists it at the same time",
                    rule.id
                ));
            }
        }
    }
    if !blocking.is_empty() {
        bail!(
            "nothing was approved; `check` would report these errors afterwards:\n{}",
            blocking.join("\n")
        );
    }

    txn.commit()?;

    Ok(ApproveReport {
        schema: SCHEMA,
        decisions: ids.into_iter().cloned().collect(),
        approved_by,
        agreed_rules,
        retired_rules,
        agreed_terms,
        retired_terms,
        built_rules,
        removed_links,
    })
}

/// Checks the term actions of the chosen decisions against the terms as they are before approving,
/// because afterwards the decisions are active and the terms already patched: every term they retire
/// is agreed, no term they name is retired, and no term is both defined and retired in the batch.
fn refuse_invalid_term_actions(model: &Model, chosen: &[&Decision]) -> Result<()> {
    let term = |name: &str| model.terms.iter().find(|t| t.name == name);
    for d in chosen {
        for name in d.all_terms() {
            if term(name).is_some_and(|t| t.is_retired()) {
                bail!(
                    "nothing was approved; {} names term `{name}`, which is retired, and a retired term never changes",
                    d.id
                );
            }
        }
        for name in &d.retires_terms {
            if term(name).is_some_and(|t| t.meta.is_none()) {
                bail!(
                    "nothing was approved; term `{name}` has no readable `yaml` block; run `statute check`"
                );
            }
            if term(name).and_then(|t| t.agreed()) != Some(TermAgreed::Agreed) {
                bail!(
                    "nothing was approved; {} retires term `{name}`, which is not agreed; only an agreed term can be retired",
                    d.id
                );
            }
        }
    }
    for d in chosen {
        for name in &d.retires_terms {
            if let Some(other) = chosen.iter().find(|o| o.terms.contains(name)) {
                bail!(
                    "nothing was approved; {} retires term `{name}` and {} redefines it; a term cannot be redefined and retired at once",
                    d.id,
                    other.id
                );
            }
        }
    }
    Ok(())
}

/// Patches one rule in its file, on top of the edits already in `txn`.
fn patch_rule(
    txn: &mut Txn,
    model: &Model,
    id: &str,
    what: &str,
    patch: impl Fn(&str, &Rule) -> Option<String>,
) -> Result<()> {
    let Some(rule) = model.rules.iter().find(|r| r.id == id) else {
        bail!("no rule {id}; run `statute check`");
    };
    let Some(patched) = patch(&txn.text(&rule.file)?, rule) else {
        bail!(
            "cannot set `{what}` on {id} in {}: write the field on one line in a block mapping, then approve again",
            rule.file
        );
    };
    txn.write(&rule.file, patched);
    Ok(())
}
