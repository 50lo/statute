use super::SCHEMA;
use crate::model::{Gap, GapStatus, ids, paths};
use crate::repo::Repo;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use rand::RngExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct GapReport {
    pub schema: u32,
    pub id: String,
    pub rule: String,
    pub status: GapStatus,
}

/// Adds an open gap to `gaps.yml`. With `reference`, an index entry for a tracker issue; without it,
/// a full entry with a generated ID and the note.
pub fn open(rule: &str, reference: Option<&str>, note: Option<&str>) -> Result<GapReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    if !model.rules.iter().any(|r| r.id == rule) {
        bail!("no rule {rule}");
    }
    let Some(mut gaps) = model.gaps.clone() else {
        bail!("{} does not parse; run `statute check`", paths::GAPS);
    };
    let now = jiff::Timestamp::now();
    let (id, note) = match (reference, note.map(str::trim)) {
        (Some(_), Some(_)) => bail!("a tracker gap keeps its notes in the tracker; drop `--note`"),
        (Some(reference), None) => {
            let reference = reference.trim();
            if reference.is_empty() || reference.contains(char::is_whitespace) {
                bail!("the reference must be the tracker's issue ID or URL, without spaces");
            }
            if gaps.iter().any(|g| g.id == reference) {
                bail!("{reference} is already in {}", paths::GAPS);
            }
            (reference.to_string(), None)
        }
        (None, Some(note)) if !note.is_empty() => {
            let mut rng = rand::rng();
            let id = ids::new_dated_id(
                'G',
                &now.strftime("%y%m%d").to_string(),
                |id| gaps.iter().any(|g| g.id == id),
                |n| rng.random_range(0..n),
            );
            (id, Some(note.to_string()))
        }
        (None, _) => bail!("a gap without `--ref` needs `--note` saying what does not hold"),
    };
    gaps.push(Gap {
        id: id.clone(),
        rule: rule.to_string(),
        status: GapStatus::Open,
        opened_at: now.strftime("%Y-%m-%dT%H:%M:%SZ").to_string(),
        note,
    });
    let mut txn = Txn::new(&repo);
    txn.write(paths::GAPS, store::render_gaps(&gaps));
    txn.commit()?;
    Ok(GapReport {
        schema: SCHEMA,
        id,
        rule: rule.to_string(),
        status: GapStatus::Open,
    })
}

/// Marks a gap closed. A gap that is already closed stays as it is.
pub fn close(reference: &str) -> Result<GapReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let Some(mut gaps) = model.gaps.clone() else {
        bail!("{} does not parse; run `statute check`", paths::GAPS);
    };
    let Some(gap) = gaps.iter_mut().find(|g| g.id == reference) else {
        bail!("no gap {reference} in {}", paths::GAPS);
    };
    let report = GapReport {
        schema: SCHEMA,
        id: gap.id.clone(),
        rule: gap.rule.clone(),
        status: GapStatus::Closed,
    };
    if gap.status == GapStatus::Open {
        gap.status = GapStatus::Closed;
        let mut txn = Txn::new(&repo);
        txn.write(paths::GAPS, store::render_gaps(&gaps));
        txn.commit()?;
    }
    Ok(report)
}
