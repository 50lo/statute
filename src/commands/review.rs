use super::SCHEMA;
use crate::fingerprint;
use crate::model::{GapStatus, Link, paths};
use crate::repo::Repo;
use crate::state::{self, Health};
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;

/// The verdict on a rule whose links went stale.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "verdict", content = "ref", rename_all = "lowercase")]
pub enum Verdict {
    /// The rule still holds.
    Ok,
    /// The change is on purpose: this decision changes or retires the rule.
    Changed(String),
    /// The rule no longer holds: this open gap records it.
    Gap(String),
}

#[derive(Serialize)]
pub struct ReviewReport {
    pub schema: u32,
    pub rule: String,
    #[serde(flatten)]
    pub verdict: Verdict,
    /// Targets whose fingerprint this review refreshed.
    pub refreshed: Vec<String>,
}

/// Records one verdict for a rule: checks it against the decisions and gaps, then refreshes every
/// stale link of the rule.
pub fn run(rule_id: &str, verdict: Verdict) -> Result<ReviewReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    if !model.rules.iter().any(|r| r.id == rule_id) {
        bail!("no rule {rule_id}");
    }
    let Some(mut links) = model.links.clone() else {
        bail!("{} does not parse; run `statute check`", paths::LOCK);
    };

    match &verdict {
        Verdict::Ok => {}
        Verdict::Changed(id) => {
            let Some(decisions) = &model.decisions else {
                bail!("{} does not parse; run `statute check`", paths::DECISIONS);
            };
            let Some(d) = decisions.iter().find(|d| d.id == *id) else {
                bail!("no decision {id}");
            };
            if !d.changes.iter().chain(&d.retires).any(|r| r == rule_id) {
                bail!("{id} does not list {rule_id} in `changes` or `retires`");
            }
        }
        Verdict::Gap(reference) => {
            let Some(gaps) = &model.gaps else {
                bail!("{} does not parse; run `statute check`", paths::GAPS);
            };
            let open = gaps
                .iter()
                .any(|g| g.id == *reference && g.rule == rule_id && g.status == GapStatus::Open);
            if !open {
                bail!(
                    "{rule_id} has no open gap {reference}; open it first with `statute gap open {rule_id}`"
                );
            }
        }
    }

    let own: Vec<&Link> = links.iter().filter(|l| l.rule() == rule_id).collect();
    let files = repo.read_files(own.iter().map(|l| l.file()))?;
    let targets: Vec<_> = own.iter().map(|l| (l.mode, l.target.as_str())).collect();
    let current = fingerprint::sign(&files, &targets);
    let health = state::health(&own, &current);
    if let Some((link, Health::Missing(why))) = own
        .iter()
        .zip(&health)
        .find(|(_, h)| matches!(h, Health::Missing(_)))
    {
        bail!(
            "{rule_id} has a missing link: {} ({why}); fix the file, or relink the target with `statute link {rule_id} {} --remove` and `statute link`",
            link.target,
            link.target
        );
    }
    // Missing links were refused above, so every stale link has a current fingerprint.
    let fresh: Vec<(String, String)> = own
        .iter()
        .zip(&health)
        .zip(current)
        .filter(|((_, h), _)| **h == Health::Stale)
        .filter_map(|((l, _), now)| Some((l.target.clone(), now.ok()?)))
        .collect();

    for link in links.iter_mut().filter(|l| l.rule() == rule_id) {
        if let Some((_, sig)) = fresh.iter().find(|(t, _)| *t == link.target) {
            link.sig = sig.clone();
        }
    }
    if !fresh.is_empty() {
        let mut txn = Txn::new(&repo);
        txn.write(paths::LOCK, store::lock::render(&links));
        txn.commit()?;
    }
    Ok(ReviewReport {
        schema: SCHEMA,
        rule: rule_id.to_string(),
        verdict,
        refreshed: fresh.into_iter().map(|(t, _)| t).collect(),
    })
}
