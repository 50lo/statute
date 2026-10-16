use super::SCHEMA;
use crate::check;
use crate::fingerprint;
use crate::git;
use crate::impact;
use crate::model::{Finding, Impact, Link, Model, Severity};
use crate::repo::Repo;
use crate::state;
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct CheckReport {
    pub schema: u32,
    pub errors: usize,
    pub warnings: usize,
    pub findings: Vec<Finding>,
    /// The rules and terms the branch affects; only with `--md`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<Impact>,
}

impl CheckReport {
    pub fn failed(&self) -> bool {
        self.errors > 0
    }
}

/// Runs every check on the Statute files, and reports stale and missing links whose target file
/// changed since the base. With `all`, every link is fingerprinted, and term `code` and `not` names
/// are searched for across the repository.
/// With `md`, it also computes the "Rule impact" section against the base.
pub fn run(all: bool, md: bool) -> Result<CheckReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let mut findings = check::run(&model);
    let rev = git::base(&repo.root)?;
    let base = match &rev {
        Some(rev) => {
            let base = repo.load_at(rev)?;
            check::history(&model, &base, &mut findings);
            base
        }
        // Before the first commit, everything is new.
        None => Model {
            decisions: Some(Vec::new()),
            gaps: Some(Vec::new()),
            links: Some(Vec::new()),
            ..Model::default()
        },
    };
    let mut link_health = Vec::new();

    if let Some(links) = &model.links {
        let selected: Vec<&Link> = if all {
            links.iter().collect()
        } else {
            let changed = git::changed_files(&repo.root, rev.as_deref())?;
            links
                .iter()
                .filter(|l| changed.contains(l.file()))
                .collect()
        };
        let files = repo.read_files(selected.iter().map(|l| l.file()))?;
        let targets: Vec<_> = selected
            .iter()
            .map(|l| (l.mode, l.target.as_str()))
            .collect();
        let health = state::health(&selected, &fingerprint::sign(&files, &targets));
        findings.extend(check::freshness(&selected, &health));
        link_health = selected
            .iter()
            .zip(&health)
            .map(|(l, h)| (l.rule(), l.target.as_str(), *h))
            .collect();
    }

    if all {
        let mut search = check::NameSearch::new(&model);
        repo.walk(check::NameSearch::searches, |bytes| search.scan(bytes))?;
        findings.extend(search.findings(&model));
    }

    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let impact = md.then(|| impact::compute(&model, &base, &link_health));
    Ok(CheckReport {
        schema: SCHEMA,
        errors,
        warnings: findings.len() - errors,
        findings,
        impact,
    })
}
