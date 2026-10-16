use super::SCHEMA;
use crate::check;
use crate::model::{Finding, Severity};
use crate::repo::Repo;
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

/// Runs every check on the Statute files.
pub fn run() -> Result<CheckReport> {
    let model = Repo::discover()?.load()?;
    let findings = check::run(&model);
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
