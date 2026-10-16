use super::SCHEMA;
use crate::model::paths;
use crate::repo::Repo;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;

const SNIPPET: &str = include_str!("../../plugin/agents-snippet.md");

#[derive(Serialize)]
pub struct InitReport {
    pub schema: u32,
    pub root: String,
    /// Files and directories written, relative to the root.
    pub created: Vec<String>,
    /// Layout files that already existed and were left as they are.
    pub kept: Vec<String>,
    pub agents_md: AgentsMd,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentsMd {
    Created,
    Updated,
    Unchanged,
}

/// Writes `statute.yml`, the default layout and the AGENTS.md snippet. Existing files are kept.
pub fn run() -> Result<InitReport> {
    let repo = Repo::init_target()?;
    if repo.exists(paths::CONFIG) {
        bail!(
            "Statute is already set up in {}: {} exists",
            repo.root.display(),
            paths::CONFIG
        );
    }

    let mut txn = Txn::new(&repo);
    let (mut created, mut kept) = (Vec::new(), Vec::new());
    let files = [
        (paths::CONFIG, store::CONFIG_TEMPLATE),
        (paths::LOCK, store::LOCK_TEMPLATE),
        (paths::TERMS, ""),
        (paths::DECISIONS, store::EMPTY_LIST),
        (paths::GAPS, store::EMPTY_LIST),
    ];
    for (path, contents) in files {
        if repo.exists(path) {
            kept.push(path.to_string());
        } else {
            txn.write(path, contents.to_string());
            created.push(path.to_string());
        }
    }
    if !repo.exists(paths::RULES_DIR) {
        txn.create_dir(paths::RULES_DIR);
        created.push(format!("{}/", paths::RULES_DIR));
    }

    let existing = repo.read(paths::AGENTS)?;
    let agents_md = match store::with_agents_snippet(existing.as_deref(), SNIPPET) {
        None => AgentsMd::Unchanged,
        Some(text) => {
            txn.write(paths::AGENTS, text);
            if existing.is_some() {
                AgentsMd::Updated
            } else {
                created.push(paths::AGENTS.to_string());
                AgentsMd::Created
            }
        }
    };

    txn.commit()?;
    Ok(InitReport {
        schema: SCHEMA,
        root: repo.root.display().to_string(),
        created,
        kept,
        agents_md,
    })
}
