use super::SCHEMA;
use crate::check;
use crate::fingerprint;
use crate::git;
use crate::model::{Finding, Link, Severity};
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
}

impl CheckReport {
    pub fn failed(&self) -> bool {
        self.errors > 0
    }
}

/// Runs every check on the Statute files, and reports stale and missing links whose target file
/// changed since the base. With `all`, every link is fingerprinted, and term `code` and `not` names
/// are searched for across the repository.
pub fn run(all: bool) -> Result<CheckReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let mut findings = check::run(&model);

    if let Some(links) = &model.links {
        let selected: Vec<&Link> = if all {
            links.iter().collect()
        } else {
            let base = git::base(&repo.root)?;
            let changed = git::changed_files(&repo.root, base.as_deref())?;
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
    Ok(CheckReport {
        schema: SCHEMA,
        errors,
        warnings: findings.len() - errors,
        findings,
    })
}
