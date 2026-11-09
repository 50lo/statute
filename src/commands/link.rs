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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<&'static str>,
    /// Where the Swift parser first fails, with a `parse-error` fallback. Tree-sitter recovers
    /// from errors, so this is near the cause, not always at it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error_near: Option<Position>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub already_linked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_role: Option<LinkRole>,
}

/// A 1-based line and character column in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
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
    let (target, role, mode, sig, requested, parse_error_near) =
        match fingerprint::sign(&files, &[(mode, target)]).remove(0) {
            Ok(sig) => (target.to_string(), role, mode, sig, None, None),
            Err(fingerprint::Missing::ParseError) if mode == Mode::SwiftAstSha256V1 => {
                let requested = Some(target.to_string());
                let parse_error_near = files
                    .get(file)
                    .and_then(Option::as_deref)
                    .and_then(fingerprint::swift_parse_error_at)
                    .map(|(line, column)| Position { line, column });
                if let Some(existing) = links
                    .iter()
                    .find(|l| l.rule() == rule_id && l.target == file)
                {
                    return Ok(LinkReport {
                        schema: SCHEMA,
                        rule: rule_id.to_string(),
                        target: existing.target.clone(),
                        role: existing.role,
                        mode: existing.mode,
                        sig: existing.sig.clone(),
                        removed: false,
                        built: false,
                        requested,
                        fallback: Some("parse-error"),
                        parse_error_near,
                        already_linked: true,
                        requested_role: (role != existing.role).then_some(role),
                    });
                }
                let mode = Mode::BytesSha256V1;
                let sig = fingerprint::sign(&files, &[(mode, file)])
                    .remove(0)
                    .map_err(|why| anyhow::anyhow!("cannot link {file}: {why}"))?;
                (
                    file.to_string(),
                    role,
                    mode,
                    sig,
                    requested,
                    parse_error_near,
                )
            }
            Err(why) => bail!("cannot link {target}: {why}"),
        };

    links.push(Link {
        source: LinkSource::Rule(rule_id.to_string()),
        target: target.clone(),
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
        target,
        role,
        mode,
        sig,
        removed: false,
        built: becomes_built,
        fallback: requested.as_ref().map(|_| "parse-error"),
        requested,
        parse_error_near,
        already_linked: false,
        requested_role: None,
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
        requested: None,
        fallback: None,
        parse_error_near: None,
        already_linked: false,
        requested_role: None,
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
