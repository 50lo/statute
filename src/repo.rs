//! Finds the repository root and reads Statute files. The only module that reads the filesystem.

use crate::model::{Model, paths};
use crate::store;
use anyhow::{Context, Result, bail};
use std::io::ErrorKind;
use std::path::PathBuf;

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    /// The nearest directory at or above the current one that holds `statute.yml`.
    pub fn discover() -> Result<Repo> {
        let cwd = std::env::current_dir().context("cannot read the current directory")?;
        for dir in cwd.ancestors() {
            if dir.join(paths::CONFIG).is_file() {
                return Ok(Repo {
                    root: dir.to_path_buf(),
                });
            }
        }
        bail!(
            "no statute.yml in this directory or any parent; run `statute init` at the repository root"
        )
    }

    /// Where `init` sets up Statute: the nearest directory holding `.git` (a directory, or a file in a
    /// worktree), or the current directory outside git.
    pub fn init_target() -> Result<Repo> {
        let cwd = std::env::current_dir().context("cannot read the current directory")?;
        let root = cwd
            .ancestors()
            .find(|d| d.join(".git").exists())
            .unwrap_or(&cwd);
        Ok(Repo {
            root: root.to_path_buf(),
        })
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    /// A file's text, or `None` when it does not exist.
    pub fn read(&self, rel: &str) -> Result<Option<String>> {
        match std::fs::read_to_string(self.path(rel)) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("cannot read {rel}")),
        }
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.path(rel).exists()
    }

    /// Areas with a rule file, sorted.
    pub fn areas(&self) -> Result<Vec<String>> {
        let dir = self.path(paths::RULES_DIR);
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e).with_context(|| format!("cannot read {}", paths::RULES_DIR)),
        };
        let mut areas = Vec::new();
        for entry in entries {
            let path = entry
                .with_context(|| format!("cannot read {}", paths::RULES_DIR))?
                .path();
            if path.is_file()
                && path.extension().is_some_and(|e| e == "md")
                && let Some(area) = path.file_stem().and_then(|s| s.to_str())
            {
                areas.push(area.to_string());
            }
        }
        areas.sort();
        Ok(areas)
    }

    /// Reads and parses every Statute file. Parse problems go into `Model::problems`.
    pub fn load(&self) -> Result<Model> {
        let mut model = Model::default();

        if let Some(text) = self.read(paths::CONFIG)? {
            match store::parse_config(&text) {
                Ok(config) => model.config = config,
                Err(problem) => model.problems.push(problem),
            }
        }

        for area in self.areas()? {
            let path = paths::rule_file(&area);
            let text = self.read(&path)?.unwrap_or_default();
            let parsed = store::rules::parse(&path, &area, &text);
            model.rule_files.push(parsed.file);
            model.rules.extend(parsed.rules);
            model.problems.extend(parsed.problems);
        }

        let parsed = store::terms::parse(&self.read(paths::TERMS)?.unwrap_or_default());
        model.terms = parsed.terms;
        model.problems.extend(parsed.problems);

        let text = self.read(paths::DECISIONS)?.unwrap_or_default();
        model.decisions = store::parse_decisions(&text)
            .map_err(|p| model.problems.push(p))
            .ok();

        let text = self.read(paths::GAPS)?.unwrap_or_default();
        model.gaps = store::parse_gaps(&text)
            .map_err(|p| model.problems.push(p))
            .ok();

        Ok(model)
    }
}
