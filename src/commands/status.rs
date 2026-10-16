use super::SCHEMA;
use crate::fingerprint;
use crate::model::{Agreed, Built, Link};
use crate::repo::Repo;
use crate::state::{self, Health};
use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Serialize)]
pub struct StatusReport {
    pub schema: u32,
    pub areas: Vec<AreaStatus>,
}

#[derive(Serialize)]
pub struct AreaStatus {
    pub area: String,
    pub file: String,
    pub prefix: Option<String>,
    pub rules: Vec<RuleStatus>,
}

#[derive(Serialize)]
pub struct RuleStatus {
    pub id: String,
    pub title: String,
    /// `None` when the rule's block does not parse; `check` reports it.
    pub agreed: Option<Agreed>,
    pub built: Option<Built>,
    /// Proposed, and no decision establishes it yet.
    pub draft: bool,
    /// Built, not retired, and lacking a code link, or both a test link and a `manual_check`.
    pub incomplete: bool,
    /// Targets of links whose fingerprint changed.
    pub stale: Vec<String>,
    /// Targets of links that are not found, ambiguous, or in a file that does not parse.
    pub missing: Vec<String>,
}

/// Per area: every rule with its agreed and built state, drafts, incomplete rules, and stale and
/// missing links. Unlike `check`, every link is fingerprinted, as with `check --all`.
pub fn run(area: Option<&str>) -> Result<StatusReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    if let Some(area) = area
        && !model.rule_files.iter().any(|f| f.area == area)
    {
        bail!("no rule file for area `{area}`");
    }
    let establishers = state::establishers(&model);
    let links: Vec<&Link> = model.links.iter().flatten().collect();
    let files = repo.read_files(links.iter().map(|l| l.file()))?;
    let targets: Vec<_> = links.iter().map(|l| (l.mode, l.target.as_str())).collect();
    let health = state::health(&links, &fingerprint::sign(&files, &targets));
    let unhealthy = |rule: &str, wanted: fn(&Health) -> bool| -> Vec<String> {
        links
            .iter()
            .zip(&health)
            .filter(|(l, h)| l.rule() == rule && wanted(h))
            .map(|(l, _)| l.target.clone())
            .collect()
    };
    let areas = model
        .rule_files
        .iter()
        .filter(|f| area.is_none_or(|a| a == f.area))
        .map(|file| AreaStatus {
            area: file.area.clone(),
            file: file.path.clone(),
            prefix: file.prefix.clone(),
            rules: model
                .rules
                .iter()
                .filter(|r| r.area == file.area)
                .map(|rule| RuleStatus {
                    id: rule.id.clone(),
                    title: rule.title.clone(),
                    agreed: rule.meta.as_ref().map(|m| m.agreed),
                    built: rule.meta.as_ref().map(|m| m.built),
                    draft: state::is_draft(rule, &model, &establishers),
                    incomplete: state::is_incomplete(rule, &model),
                    stale: unhealthy(&rule.id, |h| *h == Health::Stale),
                    missing: unhealthy(&rule.id, |h| matches!(h, Health::Missing(_))),
                })
                .collect(),
        })
        .collect();
    Ok(StatusReport {
        schema: SCHEMA,
        areas,
    })
}
