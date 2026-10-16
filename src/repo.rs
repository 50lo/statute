//! Finds the repository root and reads Statute files. The only module that reads the filesystem.

use crate::model::{Model, paths};
use crate::store;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
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

    /// A layout file's text. A missing file is an error, so that it is not mistaken for an empty one.
    /// `statute/rules/` is not required: git does not keep an empty directory.
    fn read_required(&self, rel: &str) -> Result<String> {
        match self.read(rel)? {
            Some(text) => Ok(text),
            None => bail!(
                "{rel} is missing; restore it from git, or create it empty if there is nothing to record yet"
            ),
        }
    }

    /// The bytes of each file, `None` for one that does not exist, is not a regular file, or is reached
    /// through a symbolic link: a link's target is the path git reports changes for.
    pub fn read_files<'p>(
        &self,
        rels: impl IntoIterator<Item = &'p str>,
    ) -> Result<BTreeMap<String, Option<Vec<u8>>>> {
        let mut files = BTreeMap::new();
        for rel in rels {
            if files.contains_key(rel) {
                continue;
            }
            let path = self.path(rel);
            let bytes = if path.is_file() && self.symlink_in(rel).is_none() {
                Some(std::fs::read(&path).with_context(|| format!("cannot read {rel}"))?)
            } else {
                None
            };
            files.insert(rel.to_string(), bytes);
        }
        Ok(files)
    }

    /// Calls `visit` with the bytes of every file in the repository that `wanted` accepts, by
    /// repo-relative path. Files that git ignores, hidden files and symbolic links are skipped.
    pub fn walk(&self, wanted: impl Fn(&str) -> bool, mut visit: impl FnMut(&[u8])) -> Result<()> {
        for entry in ignore::WalkBuilder::new(&self.root).build() {
            let entry = entry.context("cannot walk the repository")?;
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let Ok(rel) = entry.path().strip_prefix(&self.root) else {
                continue;
            };
            let rel = rel.to_string_lossy().replace('\\', "/");
            if wanted(&rel) {
                let bytes =
                    std::fs::read(entry.path()).with_context(|| format!("cannot read {rel}"))?;
                visit(&bytes);
            }
        }
        Ok(())
    }

    /// The first component of a repo-relative path that is a symbolic link, such as `src` in
    /// `src/a.go` when `src` is a link.
    pub fn symlink_in(&self, rel: &str) -> Option<String> {
        let mut prefix = PathBuf::new();
        for part in rel.split('/') {
            prefix.push(part);
            let meta = std::fs::symlink_metadata(self.root.join(&prefix)).ok()?;
            if meta.file_type().is_symlink() {
                return Some(prefix.to_string_lossy().into_owned());
            }
        }
        None
    }

    pub fn is_file(&self, rel: &str) -> bool {
        self.path(rel).is_file()
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

        let parsed = store::terms::parse(&self.read_required(paths::TERMS)?);
        model.terms = parsed.terms;
        model.problems.extend(parsed.problems);

        let text = self.read_required(paths::DECISIONS)?;
        model.decisions = store::parse_decisions(&text)
            .map_err(|p| model.problems.push(p))
            .ok();

        let text = self.read_required(paths::GAPS)?;
        model.gaps = store::parse_gaps(&text)
            .map_err(|p| model.problems.push(p))
            .ok();

        let text = self.read_required(paths::LOCK)?;
        model.links = store::lock::parse(&text)
            .map_err(|p| model.problems.push(p))
            .ok();

        Ok(model)
    }
}
