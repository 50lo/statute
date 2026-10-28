use super::SCHEMA;
use crate::check;
use crate::git;
use crate::model::{
    Agreed, ApprovedBy, Built, Decision, DecisionStatus, Finding, Model, Rule, Severity,
    TermAgreed, paths,
};
use crate::repo::Repo;
use crate::state;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

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
        at: jiff::Timestamp::now()
            .strftime("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
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

    let mut edits: BTreeMap<String, String> = BTreeMap::new();
    for id in &agreed_rules {
        patch_rule(
            &repo,
            &model,
            id,
            &mut edits,
            &|t, r| store::rules::set_agreed(t, r, Agreed::Agreed),
            "agreed: agreed",
        )?;
    }
    for id in &retired_rules {
        patch_rule(
            &repo,
            &model,
            id,
            &mut edits,
            &|t, r| store::rules::set_agreed(t, r, Agreed::Retired),
            "agreed: retired",
        )?;
    }

    // Links of retired rules go; the rules that are complete with the remaining links become built.
    let kept: Vec<_> = links
        .iter()
        .filter(|l| !retired_rules.iter().any(|r| r == l.rule()))
        .cloned()
        .collect();
    let removed_links = links.len() - kept.len();
    if removed_links > 0 {
        edits.insert(paths::LOCK.to_string(), store::lock::render(&kept));
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
            patch_rule(
                &repo,
                &model,
                id,
                &mut edits,
                &|t, r| store::rules::set_built(t, r, Built::Built),
                "built: built",
            )?;
            built_rules.push(id.clone());
        }
    }

    let mut terms_text = None;
    let term_states = agreed_terms
        .iter()
        .map(|t| (t, TermAgreed::Agreed))
        .chain(retired_terms.iter().map(|t| (t, TermAgreed::Retired)));
    for (name, agreed) in term_states {
        let text = match terms_text.take() {
            Some(text) => text,
            None => repo.read(paths::TERMS)?.unwrap_or_default(),
        };
        let parsed = store::terms::parse(&text);
        let Some(term) = parsed.terms.iter().find(|t| &t.name == name) else {
            bail!("no term `{name}`; run `statute check`");
        };
        let Some(patched) = store::terms::set_agreed(&text, term, agreed) else {
            let value = match agreed {
                TermAgreed::Retired => "retired",
                _ => "agreed",
            };
            bail!(
                "cannot set `agreed: {value}` on term `{name}`: write the field on one line in a block mapping, then approve again"
            );
        };
        terms_text = Some(patched);
    }
    if let Some(text) = terms_text {
        edits.insert(paths::TERMS.to_string(), text);
    }

    let mut approved = decisions.clone();
    for d in approved.iter_mut().filter(|d| ids.contains(&&d.id)) {
        d.status = DecisionStatus::Active;
        d.approved_by = Some(approved_by.clone());
    }
    edits.insert(
        paths::DECISIONS.to_string(),
        store::render_decisions(&approved),
    );

    // The checks `check` runs, on the result. Only errors about what is being approved block it.
    let after = repo.load_with(&edits)?;
    let mut findings = check::run(&after);
    if let Some(rev) = git::base(&repo.root)? {
        check::history(&after, &repo.load_at(&rev)?, &mut findings);
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
            let agreed = after.terms.iter().any(|t| {
                &t.name == name
                    && t.meta.as_ref().and_then(|m| m.agreed) == Some(TermAgreed::Agreed)
            });
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

    let mut txn = Txn::new(&repo);
    for (file, text) in edits {
        txn.write(&file, text);
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

/// Patches one rule in its file, applying the patch to the text earlier patches produced.
fn patch_rule(
    repo: &Repo,
    model: &Model,
    id: &str,
    edits: &mut BTreeMap<String, String>,
    patch: &dyn Fn(&str, &Rule) -> Option<String>,
    what: &str,
) -> Result<()> {
    let Some(rule) = model.rules.iter().find(|r| r.id == id) else {
        bail!("no rule {id}; run `statute check`");
    };
    let text = match edits.get(&rule.file) {
        Some(text) => text.clone(),
        None => repo.read(&rule.file)?.unwrap_or_default(),
    };
    // Earlier patches in the same file move this rule's byte offsets.
    let parsed = store::rules::parse(&rule.file, &rule.area, &text);
    let current = parsed.rules.iter().find(|r| r.id == id);
    let Some(patched) = current.and_then(|r| patch(&text, r)) else {
        bail!(
            "cannot set `{what}` on {id} in {}: write the field on one line in a block mapping, then approve again",
            rule.file
        );
    };
    edits.insert(rule.file.clone(), patched);
    Ok(())
}
