use super::SCHEMA;
use crate::model::{
    Agreed, Decision, DecisionDraft, DecisionStatus, Model, TermAgreed, ids, paths,
};
use crate::repo::Repo;
use crate::store;
use crate::txn::{self, Txn};
use anyhow::{Result, anyhow, bail};
use rand::RngExt;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct DecideReport {
    pub schema: u32,
    pub id: String,
    /// Rules in `changes` this command set to `agreed: proposed`.
    pub proposed_rules: Vec<String>,
    /// Terms this command set to `agreed: proposed`.
    pub proposed_terms: Vec<String>,
}

/// Appends a proposed decision, and sets `agreed: proposed` on the rules it changes and the terms it
/// lists.
pub fn run(draft: DecisionDraft) -> Result<DecideReport> {
    if draft.summary.trim().is_empty() || draft.rationale.trim().is_empty() {
        bail!("a decision needs a summary and a rationale");
    }
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let Some(mut decisions) = model.decisions.clone() else {
        bail!("{} does not parse; run `statute check`", paths::DECISIONS);
    };
    refuse_unknown_or_pending(&model, &draft)?;

    let mut txn = Txn::new(&repo);
    let mut proposed_rules = Vec::new();
    let mut rule_files: BTreeMap<&str, String> = BTreeMap::new();
    for id in &draft.changes {
        let Some(rule) = model.rules.iter().find(|r| &r.id == id) else {
            continue;
        };
        let Some(meta) = &rule.meta else {
            bail!(
                "{id} has no readable `yaml` block in {}; run `statute check`",
                rule.file
            );
        };
        if meta.agreed == Agreed::Proposed {
            continue;
        }
        let text = match rule_files.remove(rule.file.as_str()) {
            Some(text) => text,
            None => repo.read(&rule.file)?.unwrap_or_default(),
        };
        // Earlier patches in the same file move this rule's byte offsets.
        let parsed = store::rules::parse(&rule.file, &rule.area, &text);
        let current = parsed.rules.iter().find(|r| &r.id == id).unwrap_or(rule);
        let Some(patched) = store::rules::set_agreed(&text, current, Agreed::Proposed) else {
            bail!(
                "cannot set `agreed: proposed` on {id} in {}: write its `agreed:` field on one line, then decide again",
                rule.file
            );
        };
        rule_files.insert(&rule.file, patched);
        proposed_rules.push(id.clone());
    }
    for (file, text) in rule_files {
        txn.write(file, text);
    }

    let mut proposed_terms = Vec::new();
    let mut terms_text = None;
    for name in &draft.terms {
        let Some(term) = model.terms.iter().find(|t| &t.name == name) else {
            continue;
        };
        let Some(meta) = &term.meta else {
            bail!("term `{name}` has no readable `yaml` block; run `statute check`");
        };
        if meta.agreed == Some(TermAgreed::Proposed) {
            continue;
        }
        let text = match terms_text.take() {
            Some(text) => text,
            None => repo.read(paths::TERMS)?.unwrap_or_default(),
        };
        let parsed = store::terms::parse(&text);
        let current = parsed
            .terms
            .iter()
            .find(|t| &t.name == name)
            .unwrap_or(term);
        let Some(patched) = store::terms::set_agreed(&text, current, TermAgreed::Proposed) else {
            bail!(
                "cannot set `agreed: proposed` on term `{name}`: write its `agreed:` field on one line, then decide again"
            );
        };
        terms_text = Some(patched);
        proposed_terms.push(name.clone());
    }
    if let Some(text) = terms_text {
        txn.write(paths::TERMS, text);
    }

    let now = jiff::Timestamp::now();
    let mut rng = rand::rng();
    let id = ids::new_dated_id(
        'D',
        &now.strftime("%y%m%d").to_string(),
        |id| decisions.iter().any(|d| d.id == id),
        |n| rng.random_range(0..n),
    );
    decisions.push(Decision {
        id: id.clone(),
        timestamp: now.strftime("%Y-%m-%dT%H:%M:%SZ").to_string(),
        actor: draft.actor,
        status: DecisionStatus::Proposed,
        ratified_by: None,
        summary: draft.summary,
        rationale: draft.rationale,
        establishes: draft.establishes,
        changes: draft.changes,
        retires: draft.retires,
        terms: draft.terms,
    });
    txn.write(paths::DECISIONS, store::render_decisions(&decisions));
    txn.commit()?;

    Ok(DecideReport {
        schema: SCHEMA,
        id,
        proposed_rules,
        proposed_terms,
    })
}

/// Writes the draft as YAML to a scratch file for a person to edit, and returns its path.
pub fn draft_file(draft: &DecisionDraft) -> Result<PathBuf> {
    txn::write_scratch("decision.yml", &store::render_draft(draft))
}

/// Records the draft a person edited in the file `draft_file` wrote, and removes the file. A draft
/// that cannot be recorded is kept, and the error says where; an empty summary discards it.
pub fn record_draft_file(path: &Path) -> Result<DecideReport> {
    let kept = || {
        format!(
            "no decision was recorded; your draft is kept in {}",
            path.display()
        )
    };
    let draft = match store::parse_draft(&Repo::read_scratch(path)?) {
        Ok(draft) if draft.summary.trim().is_empty() => {
            txn::remove_scratch(path);
            bail!("the summary is empty; no decision was recorded")
        }
        Ok(draft) => draft,
        Err(e) => bail!("the edited decision does not parse: {e}; {}", kept()),
    };
    let report = run(draft).map_err(|e| anyhow!("{e:#}; {}", kept()))?;
    txn::remove_scratch(path);
    Ok(report)
}

/// Removes the file `draft_file` wrote, when the person gave up editing it.
pub fn discard_draft_file(path: &Path) {
    txn::remove_scratch(path);
}

/// Every rule and term the draft names exists, and none is in another proposed decision.
fn refuse_unknown_or_pending(model: &Model, draft: &DecisionDraft) -> Result<()> {
    let rules = draft
        .establishes
        .iter()
        .chain(&draft.changes)
        .chain(&draft.retires);
    for id in rules.clone() {
        if !model.rules.iter().any(|r| &r.id == id) {
            bail!("no rule {id}");
        }
    }
    for name in &draft.terms {
        if !model.terms.iter().any(|t| &t.name == name) {
            bail!("no term `{name}` in {}", paths::TERMS);
        }
    }
    let pending = model
        .decisions
        .iter()
        .flatten()
        .filter(|d| d.status == DecisionStatus::Proposed);
    for d in pending {
        if let Some(id) = rules.clone().find(|id| d.rules().any(|r| r == *id)) {
            bail!(
                "{id} is already in proposed decision {}; a rule is in at most one proposed decision",
                d.id
            );
        }
        if let Some(name) = draft.terms.iter().find(|t| d.terms.contains(t)) {
            bail!(
                "term `{name}` is already in proposed decision {}; a term is in at most one proposed decision",
                d.id
            );
        }
    }
    Ok(())
}
