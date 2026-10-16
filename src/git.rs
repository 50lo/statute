//! Runs `git`. The only module that spawns processes.

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};

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

/// The files under `dir` at a revision, as paths from the repository root.
pub fn files_under(root: &Path, rev: &str, dir: &str) -> Result<Vec<String>> {
    let out = git(
        root,
        &[
            "ls-tree",
            "-z",
            "--name-only",
            rev,
            "--",
            &format!("{dir}/"),
        ],
    )?;
    Ok(out
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).into_owned())
        .collect())
}

/// The contents of each path at a revision, through one `git cat-file --batch`; `None` for a path the
/// revision does not have.
pub fn read_at(root: &Path, rev: &str, paths: &[&str]) -> Result<Vec<Option<Vec<u8>>>> {
    let mut child = Command::new("git")
        .args(["cat-file", "--batch"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot run git; is it installed?")?;
    let mut input = String::new();
    for path in paths {
        input.push_str(&format!("{rev}:{path}\n"));
    }
    let mut stdin = child.stdin.take().expect("stdin is piped");
    // Write from a thread so a large output cannot block the write.
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout is piped"));
    let mut out = Vec::new();
    for path in paths {
        let mut header = String::new();
        stdout
            .read_line(&mut header)
            .with_context(|| format!("cannot read {path} at {rev}"))?;
        let fields: Vec<&str> = header.split_whitespace().collect();
        match fields[..] {
            [_, "blob", size] => {
                let size: usize = size.parse().context("unexpected `git cat-file` output")?;
                let mut blob = vec![0; size + 1];
                stdout.read_exact(&mut blob)?;
                blob.pop();
                out.push(Some(blob));
            }
            [_, _, size] => {
                // Not a file, such as a directory; skip its contents.
                let size: usize = size.parse().context("unexpected `git cat-file` output")?;
                std::io::copy(
                    &mut (&mut stdout).take(size as u64 + 1),
                    &mut std::io::sink(),
                )?;
                out.push(None);
            }
            _ => out.push(None),
        }
    }
    writer
        .join()
        .expect("the writer does not panic")
        .context("cannot write to `git cat-file`")?;
    child.wait()?;
    Ok(out)
}

/// `user.name` and `user.email` from git config, when both are set.
pub fn user(root: &Path) -> Result<Option<(String, String)>> {
    let name = try_git(root, &["config", "user.name"])?.map(line);
    let email = try_git(root, &["config", "user.email"])?.map(line);
    Ok(match (name, email) {
        (Some(n), Some(e)) if !n.is_empty() && !e.is_empty() => Some((n, e)),
        _ => None,
    })
}
