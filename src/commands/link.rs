use super::SCHEMA;
use crate::fingerprint;
use crate::model::{Agreed, Built, Link, LinkRole, LinkSource, Mode, paths, split_target};
use crate::repo::Repo;
use crate::state;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Serialize)]
pub struct LinkReport {
    pub schema: u32,
    pub rule: String,
    pub target: String,
    pub role: LinkRole,
    pub mode: Mode,
    pub sig: String,
    /// The link was removed rather than added.
    pub removed: bool,
    /// This command set `built: built` on the rule, because the link completed it.
    pub built: bool,
}

/// Adds a link with the current fingerprint, and sets `built: built` when the rule becomes complete.
pub fn add(rule_id: &str, target: &str, role: LinkRole) -> Result<LinkReport> {
    let (file, symbol) = split_target(target);
    validate_path(file)?;
    if symbol.is_some_and(str::is_empty) {
        bail!("`{target}` has an empty symbol after `#`");
    }

    let repo = Repo::discover()?;
    let model = repo.load()?;
    let Some(mut links) = model.links.clone() else {
        bail!("{} does not parse; run `statute check`", paths::LOCK);
    };
    let Some(rule) = model.rules.iter().find(|r| r.id == rule_id) else {
        bail!("no rule {rule_id}");
    };
    if let Some(existing) = links
        .iter()
        .find(|l| l.rule() == rule_id && l.target == target)
    {
        bail!(
            "{rule_id} already links {target} (sig {}); a stale link is refreshed with `statute review`",
            existing.sig
        );
    }
    if let Some(link) = repo.symlink_in(file) {
        bail!("{link} is a symbolic link; link the file it points to, inside the repository");
    }
    if !repo.is_file(file) {
        if repo.exists(file) {
            bail!("{file} is not a file; a target is a file or a symbol in one, never a directory");
        }
        bail!("{file} does not exist");
    }
    let Some(mode) = fingerprint::mode_for(file, symbol) else {
        bail!("no fingerprint mode reads symbols in {file}; link the whole file");
    };
    let files = repo.read_files([file])?;
    let sig = match fingerprint::sign(&files, &[(mode, target)]).remove(0) {
        Ok(sig) => sig,
        Err(why) => bail!("cannot link {target}: {why}"),
    };

    links.push(Link {
        source: LinkSource::Rule(rule_id.to_string()),
        target: target.to_string(),
        role,
        mode,
        sig: sig.clone(),
    });
    let mut txn = Txn::new(&repo);
    txn.write(paths::LOCK, store::lock::render(&links));

    let becomes_built = rule
        .meta
        .as_ref()
        .is_some_and(|m| m.agreed != Agreed::Retired && m.built == Built::Planned)
        && state::is_complete(rule, &links);
    if becomes_built {
        let text = repo.read(&rule.file)?.unwrap_or_default();
        let Some(patched) = store::rules::set_built(&text, rule, Built::Built) else {
            bail!(
                "cannot set `built: built` on {rule_id} in {}: write its `built:` field as `built: planned` on one line, then link again",
                rule.file
            );
        };
        txn.write(&rule.file, patched);
    }
    txn.commit()?;

    Ok(LinkReport {
        schema: SCHEMA,
        rule: rule_id.to_string(),
        target: target.to_string(),
        role,
        mode,
        sig,
        removed: false,
        built: becomes_built,
    })
}

/// Removes a link. A link to a rule that no longer exists can be removed too.
pub fn remove(rule_id: &str, target: &str) -> Result<LinkReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let Some(mut links) = model.links else {
        bail!("{} does not parse; run `statute check`", paths::LOCK);
    };
    let Some(i) = links
        .iter()
        .position(|l| l.rule() == rule_id && l.target == target)
    else {
        bail!("{rule_id} has no link to {target}");
    };
    let removed = links.remove(i);
    let mut txn = Txn::new(&repo);
    txn.write(paths::LOCK, store::lock::render(&links));
    txn.commit()?;
    Ok(LinkReport {
        schema: SCHEMA,
        rule: rule_id.to_string(),
        target: removed.target,
        role: removed.role,
        mode: removed.mode,
        sig: removed.sig,
        removed: true,
        built: false,
    })
}

/// A repo-relative file path: no leading `/`, no `.` or `..` components, no empty components.
fn validate_path(file: &str) -> Result<()> {
    let valid = !file.is_empty()
        && !file.starts_with('/')
        && !file.contains('\\')
        && file
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..");
    if !valid {
        bail!(
            "`{file}` is not a repo-relative file path such as `src/billing/refund.go`; targets are relative to the repository root"
        );
    }
    Ok(())
}
