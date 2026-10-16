//! A temporary repository and a way to run the `statute` binary in it.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub struct TestRepo {
    dir: TempDir,
}

pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("not JSON ({e}): {}", self.stdout))
    }

    /// The finding codes of a `check --json` report, in order.
    pub fn codes(&self) -> Vec<String> {
        self.json()["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["code"].as_str().unwrap().to_string())
            .collect()
    }
}

impl TestRepo {
    /// An empty directory that git would treat as a repository root.
    pub fn empty() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        TestRepo { dir }
    }

    /// A copy of `tests/fixtures/<name>`.
    pub fn fixture(name: &str) -> Self {
        let repo = TestRepo::empty();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        copy_dir(&source, repo.dir.path());
        repo
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    pub fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
    }

    pub fn write(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    pub fn run(&self, args: &[&str]) -> Run {
        self.run_in(".", args)
    }

    /// Runs `statute` from a subdirectory of the repository.
    pub fn run_in(&self, subdir: &str, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_statute"))
            .args(args)
            .current_dir(self.path(subdir))
            .output()
            .unwrap();
        Run {
            code: out.status.code().unwrap(),
            stdout: String::from_utf8(out.stdout).unwrap(),
            stderr: String::from_utf8(out.stderr).unwrap(),
        }
    }
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
