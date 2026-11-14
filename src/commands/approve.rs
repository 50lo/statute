use super::SCHEMA;
use crate::check;
use crate::git;
use crate::model::{
    Agreed, ApprovedBy, Built, Decision, DecisionStatus, Finding, Link, Model, Rule, Severity,
    TermAgreed, ids, paths,
};
use crate::repo::Repo;
use crate::state::{self, TermAction, TermRefusal};
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
    let changes = Changes::of(&chosen);
    refuse_invalid_term_actions(&model, &chosen)?;

    let mut txn = Txn::new(&repo);
    let (built_rules, removed_links) = patch_rules(&mut txn, &model, links, &changes)?;
    patch_terms(&mut txn, &model, &changes)?;
    let mut approved = decisions.clone();
    for d in approved.iter_mut().filter(|d| ids.contains(&&d.id)) {
        d.status = DecisionStatus::Active;
        d.approved_by = Some(approved_by.clone());
    }
    txn.write(paths::DECISIONS, store::render_decisions(&approved));

    let after = repo.load_with(txn.writes())?;
    let (rev, base) = repo.load_base()?;
    let blocking = blocking_errors(&after, rev.map(|_| &base), &ids, &changes);
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
        agreed_rules: changes.agreed_rules,
        retired_rules: changes.retired_rules,
        agreed_terms: changes.agreed_terms,
        retired_terms: changes.retired_terms,
        built_rules,
        removed_links,
    })
}

/// The rules and terms the chosen decisions change, by the state they get.
struct Changes {
    agreed_rules: Vec<String>,
    retired_rules: Vec<String>,
    agreed_terms: Vec<String>,
    retired_terms: Vec<String>,
}

impl Changes {
    fn of(chosen: &[&Decision]) -> Self {
        let mut changes = Changes {
            agreed_rules: Vec::new(),
            retired_rules: Vec::new(),
            agreed_terms: Vec::new(),
            retired_terms: Vec::new(),
        };
        for d in chosen {
            let agreed = d.establishes.iter().chain(&d.changes).cloned();
            changes.agreed_rules.extend(agreed);
            changes.retired_rules.extend(d.retires.iter().cloned());
            changes.agreed_terms.extend(d.terms.iter().cloned());
            changes
                .retired_terms
                .extend(d.retires_terms.iter().cloned());
        }
        changes
    }

    fn names_rule(&self, id: &String) -> bool {
        self.agreed_rules.contains(id) || self.retired_rules.contains(id)
    }

    fn names_term(&self, name: &String) -> bool {
        self.agreed_terms.contains(name) || self.retired_terms.contains(name)
    }
}

/// Sets `agreed` on the rules, removes the links of retired rules, and sets `built: built` on the
/// agreed rules that are complete with the remaining links. Returns those rules and the number of
/// links removed.
fn patch_rules(
    txn: &mut Txn,
    model: &Model,
    links: &[Link],
    changes: &Changes,
) -> Result<(Vec<String>, usize)> {
    for id in &changes.agreed_rules {
        patch_rule(txn, model, id, "agreed: agreed", |t, r| {
            store::rules::set_agreed(t, r, Agreed::Agreed)
        })?;
    }
    for id in &changes.retired_rules {
        patch_rule(txn, model, id, "agreed: retired", |t, r| {
            store::rules::set_agreed(t, r, Agreed::Retired)
        })?;
    }

    let kept: Vec<_> = links
        .iter()
        .filter(|l| !changes.retired_rules.iter().any(|r| r == l.rule()))
        .cloned()
        .collect();
    let removed_links = links.len() - kept.len();
    if removed_links > 0 {
        txn.write(paths::LOCK, store::lock::render(&kept));
    }
    let mut built_rules = Vec::new();
    for id in &changes.agreed_rules {
        let Some(rule) = model.rules.iter().find(|r| &r.id == id) else {
            continue;
        };
        let planned = rule
            .meta
            .as_ref()
            .is_some_and(|m| m.built == Built::Planned);
        if planned && state::is_complete(rule, &kept) {
            patch_rule(txn, model, id, "built: built", |t, r| {
                store::rules::set_built(t, r, Built::Built)
            })?;
            built_rules.push(id.clone());
        }
    }
    Ok((built_rules, removed_links))
}

/// Sets `agreed: agreed` or `agreed: retired` on the terms.
fn patch_terms(txn: &mut Txn, model: &Model, changes: &Changes) -> Result<()> {
    let term_states = changes
        .agreed_terms
        .iter()
        .map(|t| (t, TermAgreed::Agreed))
        .chain(
            changes
                .retired_terms
                .iter()
                .map(|t| (t, TermAgreed::Retired)),
        );
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
    Ok(())
}

/// The errors `check` would report on the result about what is being approved, which block it. Errors
/// about anything else do not. `base` is `None` before the first commit.
fn blocking_errors(
    after: &Model,
    base: Option<&Model>,
    ids: &[&String],
    changes: &Changes,
) -> Vec<String> {
    let mut findings = check::run(after);
    if let Some(base) = base {
        check::history(after, base, &mut findings);
    }
    let about = |f: &Finding| {
        f.decision.as_ref().is_some_and(|d| ids.contains(&d))
            || f.rule.as_ref().is_some_and(|r| changes.names_rule(r))
            || f.term.as_ref().is_some_and(|t| changes.names_term(t))
    };
    let mut blocking: Vec<String> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && about(f))
        .map(|f| format!("  {}: {}", f.code, f.message))
        .collect();
    // A rule becomes agreed only with agreed terms: agreed before, or approved in this batch. A retired
    // term is reported above as `rule.term_retired`.
    let agreed_rules = after
        .rules
        .iter()
        .filter(|r| changes.agreed_rules.contains(&r.id));
    for rule in agreed_rules {
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
    blocking
}

/// Checks the term actions of the chosen decisions against the terms as they are before approving,
/// because afterwards the decisions are active and the terms already patched: every term they retire
/// is agreed, no term they name is retired, and no term is both defined and retired in the batch.
fn refuse_invalid_term_actions(model: &Model, chosen: &[&Decision]) -> Result<()> {
    let term = |name: &str| model.terms.iter().find(|t| t.name == name);
    let refused = |d: &Decision, name: &str, why: TermRefusal| match why {
        TermRefusal::Retired => format!(
            "nothing was approved; {} names term `{name}`, which is retired, and a retired term never changes",
            d.id
        ),
        TermRefusal::Unreadable => format!(
            "nothing was approved; term `{name}` has no readable `yaml` block; run `statute check`"
        ),
        TermRefusal::NotAgreed => format!(
            "nothing was approved; {} retires term `{name}`, which is not agreed; only an agreed term can be retired",
            d.id
        ),
    };
    for d in chosen {
        for (name, action) in state::term_actions(&d.terms, &d.retires_terms) {
            if let Some(why @ TermRefusal::Retired) = state::term_refusal(term(name), action) {
                bail!("{}", refused(d, name, why));
            }
        }
        for name in &d.retires_terms {
            if let Some(why) = state::term_refusal(term(name), TermAction::Retire) {
                bail!("{}", refused(d, name, why));
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
