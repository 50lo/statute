//! Enforces the "Module layering" rules in AGENTS.md: the table there is the source of truth.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Modules allowed to touch the filesystem, processes or the environment.
const IO_MODULES: [&str; 5] = ["git", "repo", "txn", "cli", "main.rs"];
const IO_PATTERNS: [&str; 3] = ["std::fs", "std::process", "std::env"];

#[test]
fn modules_import_only_what_agents_md_allows() {
    let allowed = allowed_imports();
    let mut violations = Vec::new();
    for file in source_files() {
        let Some(module) = module_of(&file) else {
            continue;
        };
        let Some(may_import) = allowed.get(&module) else {
            violations.push(format!(
                "{}: module `{module}` is not in the AGENTS.md table",
                file.display()
            ));
            continue;
        };
        let text = fs::read_to_string(&file).unwrap();
        let prefix = if module == "main.rs" {
            "statute::"
        } else {
            "crate::"
        };
        for import in imports(&text, prefix) {
            if import != module && !may_import.contains(&import) {
                violations.push(format!(
                    "{}: `{module}` may not import `{import}`",
                    file.display()
                ));
            }
        }
        // `std::process::ExitCode` is not I/O; main.rs and cli use it.
        if !IO_MODULES.contains(&module.as_str()) {
            for pattern in IO_PATTERNS.iter().filter(|p| text.contains(*p)) {
                violations.push(format!(
                    "{}: `{module}` may not use `{pattern}`",
                    file.display()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "layering violations:\n{}",
        violations.join("\n")
    );
}

/// Module name -> modules it may import, from the "Module layering" table.
fn allowed_imports() -> BTreeMap<String, BTreeSet<String>> {
    let text = fs::read_to_string(root().join("AGENTS.md")).unwrap();
    let section = text
        .split("## Module layering")
        .nth(1)
        .and_then(|rest| rest.split("\n## ").next())
        .expect("AGENTS.md has a Module layering section");
    let mut table = BTreeMap::new();
    let mut above = BTreeSet::new();
    for line in section.lines().filter(|l| l.starts_with("| `")) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let module = backticked(cells[1]).remove(0);
        let may_import = if cells[2].contains("everything above") {
            above.clone()
        } else {
            backticked(cells[2]).into_iter().collect()
        };
        above.insert(module.clone());
        table.insert(module, may_import);
    }
    table
}

fn backticked(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(String::from)
        .collect()
}

/// First path segments after `prefix`, including `prefix{a, b::c}` groups.
fn imports(text: &str, prefix: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for (start, _) in text.match_indices(prefix) {
        let rest = &text[start + prefix.len()..];
        let items: Vec<&str> = match rest.strip_prefix('{') {
            Some(inner) => inner[..inner.find('}').unwrap_or(inner.len())]
                .split(',')
                .collect(),
            None => vec![rest],
        };
        found.extend(
            items
                .into_iter()
                .map(first_segment)
                .filter(|n| !n.is_empty() && n != "self"),
        );
    }
    found
}

fn first_segment(path: &str) -> String {
    path.trim()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// `src/x.rs` and `src/x/...` belong to `x`; `main.rs` is its own row; `lib.rs` is skipped.
fn module_of(file: &Path) -> Option<String> {
    let relative = file.strip_prefix(root().join("src")).unwrap();
    let first = relative.components().next()?.as_os_str().to_str()?;
    match first {
        "lib.rs" => None,
        "main.rs" => Some(first.to_string()),
        _ => Some(first.trim_end_matches(".rs").to_string()),
    }
}

fn source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![root().join("src")];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    files
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
