//! Applies a command's edits: re-parses them in memory, then writes. The only module that writes files.
//!
//! Each file is written to a temp file next to it, and the temp files are renamed into place only
//! after all of them are written. Each rename is atomic; a command that writes several files is not.

use crate::model::{Finding, paths};
use crate::repo::Repo;
use crate::store;
use anyhow::{Context, Result, bail};
use std::path::PathBuf;

pub struct Txn<'a> {
    repo: &'a Repo,
    dirs: Vec<String>,
    writes: Vec<(String, String)>,
}

impl<'a> Txn<'a> {
    pub fn new(repo: &'a Repo) -> Self {
        Txn {
            repo,
            dirs: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn create_dir(&mut self, rel: &str) {
        self.dirs.push(rel.to_string());
    }

    /// Replaces the whole file, or creates it.
    pub fn write(&mut self, rel: &str, contents: String) {
        self.writes.push((rel.to_string(), contents));
    }

    pub fn commit(self) -> Result<()> {
        for (rel, contents) in &self.writes {
            let old = self.repo.read(rel)?;
            validate(rel, old.as_deref(), contents)?;
        }

        for dir in &self.dirs {
            std::fs::create_dir_all(self.repo.path(dir))
                .with_context(|| format!("cannot create {dir}"))?;
        }

        let mut temps: Vec<(PathBuf, PathBuf, &str)> = Vec::new();
        for (rel, contents) in &self.writes {
            let target = self.repo.path(rel);
            let written = temp_path(&target).and_then(|tmp| {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&tmp, contents)?;
                Ok(tmp)
            });
            match written {
                Ok(tmp) => temps.push((tmp, target, rel)),
                Err(e) => {
                    remove_temps(&temps);
                    return Err(e)
                        .with_context(|| format!("cannot write {rel}; no file was changed"));
                }
            }
        }

        let mut replaced = Vec::new();
        for (i, (tmp, target, rel)) in temps.iter().enumerate() {
            if let Err(e) = std::fs::rename(tmp, target) {
                remove_temps(&temps[i..]);
                let replaced = if replaced.is_empty() {
                    "none".to_string()
                } else {
                    replaced.join(", ")
                };
                bail!("cannot replace {rel}: {e}; files already replaced: {replaced}");
            }
            replaced.push(*rel);
        }
        Ok(())
    }
}

fn temp_path(target: &std::path::Path) -> std::io::Result<PathBuf> {
    let name = target
        .file_name()
        .ok_or_else(|| std::io::Error::other("not a file path"))?
        .to_string_lossy();
    Ok(target.with_file_name(format!(".{name}.statute-tmp")))
}

fn remove_temps(temps: &[(PathBuf, PathBuf, &str)]) {
    for (tmp, _, _) in temps {
        let _ = std::fs::remove_file(tmp);
    }
}

/// A command's output must parse at least as cleanly as the file it replaces; anything else is a bug.
fn validate(rel: &str, old: Option<&str>, new: &str) -> Result<()> {
    let key = |f: &Finding| {
        (
            f.code,
            f.rule.clone(),
            f.term.clone(),
            f.decision.clone(),
            f.gap.clone(),
            f.message.clone(),
        )
    };
    let mut before: Vec<_> = old
        .map(|t| problems(rel, t))
        .unwrap_or_default()
        .iter()
        .map(key)
        .collect();
    for problem in problems(rel, new) {
        match before.iter().position(|k| *k == key(&problem)) {
            Some(i) => {
                before.swap_remove(i);
            }
            None => bail!(
                "internal error: this command would leave {rel} unparseable ({}: {}); no file was changed",
                problem.code,
                problem.message
            ),
        }
    }
    Ok(())
}

fn problems(rel: &str, text: &str) -> Vec<Finding> {
    let rule_area = rel
        .strip_prefix(paths::RULES_DIR)
        .and_then(|r| r.strip_prefix('/'))
        .and_then(|r| r.strip_suffix(".md"))
        .filter(|area| !area.contains('/'));
    match rel {
        paths::CONFIG => store::parse_config(text).err().into_iter().collect(),
        paths::TERMS => store::terms::parse(text).problems,
        paths::DECISIONS => store::parse_decisions(text).err().into_iter().collect(),
        paths::GAPS => store::parse_gaps(text).err().into_iter().collect(),
        _ => match rule_area {
            Some(area) => store::rules::parse(rel, area, text).problems,
            None => Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn refuses_output_with_new_parse_problems() {
        let rules = "statute/rules/billing.md";
        let good = "---\nprefix: BILL\n---\n";
        assert!(validate(rules, None, good).is_ok());
        assert!(validate(rules, Some(good), "# BILL-001 T\n").is_err());
        // An existing problem is not the command's doing.
        assert!(validate(rules, Some("# X\n"), "# X\n\n# Y\n").is_err());
        assert!(validate(rules, Some("# X\n"), "# X\n").is_ok());
        assert!(validate("AGENTS.md", None, "anything").is_ok());
    }
}
