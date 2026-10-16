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

    /// The codes of the error findings of a `check --json` report, in order.
    pub fn error_codes(&self) -> Vec<String> {
        self.json()["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["severity"] == "error")
            .map(|f| f["code"].as_str().unwrap().to_string())
            .collect()
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
    /// An empty git repository with no commits.
    pub fn empty() -> Self {
        let repo = TestRepo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "--quiet", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Pat Example"]);
        repo.git(&["config", "user.email", "pat@example.com"]);
        repo
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

    /// Runs git in the repository and returns its output; panics when git fails.
    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.dir.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// Commits every file in the working tree.
    pub fn commit(&self, message: &str) {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--quiet", "--message", message]);
    }

    /// A clone of this repository, so `origin/HEAD` points at this one's branch.
    pub fn clone_repo(&self) -> TestRepo {
        let clone = TestRepo {
            dir: tempfile::tempdir().unwrap(),
        };
        let source = self.dir.path().to_str().unwrap();
        clone.git(&["clone", "--quiet", source, "."]);
        clone.git(&["config", "user.name", "Pat Example"]);
        clone.git(&["config", "user.email", "pat@example.com"]);
        clone
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path(rel)).unwrap();
    }

    pub fn run(&self, args: &[&str]) -> Run {
        self.run_in(".", args)
    }

    /// Runs `statute` from a subdirectory of the repository.
    pub fn run_in(&self, subdir: &str, args: &[&str]) -> Run {
        self.run_with(subdir, &[], args)
    }

    /// Runs `statute` with extra environment variables.
    pub fn run_env(&self, env: &[(&str, &str)], args: &[&str]) -> Run {
        self.run_with(".", env, args)
    }

    fn run_with(&self, subdir: &str, env: &[(&str, &str)], args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_statute"))
            .args(args)
            .envs(env.iter().copied())
            .env_remove("VISUAL")
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
