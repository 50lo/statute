//! Runs `git`. The only module that spawns processes.

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

/// Paths changed since the base, relative to the directory git ran in.
#[derive(Debug, Default)]
pub struct ChangeSet {
    paths: BTreeSet<String>,
}

impl ChangeSet {
    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }
}

fn run(dir: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .context("cannot run git; is it installed?")
}

/// The output of a command whose failure is an answer, such as "no such revision".
fn try_git(dir: &Path, args: &[&str]) -> Result<Option<Vec<u8>>> {
    let out = run(dir, args)?;
    Ok(out.status.success().then_some(out.stdout))
}

fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = run(dir, args)?;
    if !out.status.success() {
        bail!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

fn line(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes).trim().to_string()
}

/// The merge-base of `HEAD` and `origin/HEAD`; `HEAD` when there is no such remote branch; `None`
/// before the first commit, or outside a repository, where `changed_files` then fails.
pub fn base(dir: &Path) -> Result<Option<String>> {
    let Some(head) = try_git(dir, &["rev-parse", "--verify", "--quiet", "HEAD"])? else {
        return Ok(None);
    };
    let base = try_git(dir, &["merge-base", "HEAD", "origin/HEAD"])?.unwrap_or(head);
    Ok(Some(line(base)))
}

/// Files that differ between the base and the working tree, plus untracked files. Renames count as a
/// deletion and an addition. Without a base, every tracked and untracked file counts.
pub fn changed_files(dir: &Path, base: Option<&str>) -> Result<ChangeSet> {
    let mut paths = BTreeSet::new();
    let mut add = |out: Vec<u8>| {
        for p in out.split(|b| *b == 0).filter(|p| !p.is_empty()) {
            paths.insert(String::from_utf8_lossy(p).into_owned());
        }
    };
    match base {
        Some(base) => add(git(
            dir,
            &[
                "diff",
                "-z",
                "--name-only",
                "--no-renames",
                "--relative",
                base,
                "--",
            ],
        )?),
        None => add(git(dir, &["ls-files", "-z", "--cached"])?),
    }
    add(git(
        dir,
        &["ls-files", "-z", "--others", "--exclude-standard"],
    )?);
    Ok(ChangeSet { paths })
}
