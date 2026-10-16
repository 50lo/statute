use super::SCHEMA;
use crate::model::{Agreed, Built};
use crate::repo::Repo;
use crate::state;
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
}

/// Per area: every rule with its agreed and built state, and which rules are drafts.
pub fn run(area: Option<&str>) -> Result<StatusReport> {
    let model = Repo::discover()?.load()?;
    if let Some(area) = area
        && !model.rule_files.iter().any(|f| f.area == area)
    {
        bail!("no rule file for area `{area}`");
    }
    let establishers = state::establishers(&model);
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
                })
                .collect(),
        })
        .collect();
    Ok(StatusReport {
        schema: SCHEMA,
        areas,
    })
}
