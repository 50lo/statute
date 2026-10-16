//! Enforces the module layering table in AGENTS.md: each module imports only the modules listed for it,
//! and only `git`, `repo` and `txn` touch the filesystem, processes or environment.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Module name -> modules it may import, from the AGENTS.md table.
fn allowed_imports() -> BTreeMap<String, BTreeSet<String>> {
    let agents = std::fs::read_to_string(Path::new(ROOT).join("AGENTS.md")).unwrap();
    let mut table = BTreeMap::new();
    let mut above: BTreeSet<String> = BTreeSet::new();
    let rows = agents
        .lines()
        .skip_while(|l| !l.starts_with("| Module |"))
        .skip(2)
        .take_while(|l| l.starts_with('|'));
    for row in rows {
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        let module = backticked(cells[1])
            .into_iter()
            .next()
            .expect("module name");
        let allowed: BTreeSet<String> = if cells[2].contains("everything above") {
            above.clone()
        } else {
            backticked(cells[2]).into_iter().collect()
        };
        above.insert(module.clone());
        table.insert(module, allowed);
    }
    assert!(
        table.len() > 5,
        "could not read the layering table from AGENTS.md"
    );
    table
}

fn backticked(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim_end_matches(".rs").to_string())
        .collect()
}

/// Source files of a module: `src/<m>.rs` or everything under `src/<m>/`.
fn sources(module: &str) -> Vec<PathBuf> {
    let src = Path::new(ROOT).join("src");
    let file = src.join(format!("{module}.rs"));
    let dir = src.join(module);
    let mut out = Vec::new();
    if file.is_file() {
        out.push(file);
    }
    if dir.is_dir() {
        collect_rs(&dir, &mut out);
    }
    out
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The source without `//` comments.
fn code(path: &Path) -> String {
    let text = std::fs::read_to_string(path).unwrap();
    text.lines()
        .map(|l| l.split("//").next().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Modules named after `prefix::` (such as `crate::`).
fn imports(code: &str, prefix: &str) -> BTreeSet<String> {
    code.match_indices(prefix)
        .map(|(i, _)| {
            let rest = &code[i + prefix.len()..];
            assert!(
                !rest.starts_with('{'),
                "use explicit paths, not `{prefix}{{...}}`"
            );
            rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        })
        .collect()
}

#[test]
fn modules_import_only_what_agents_md_allows() {
    let table = allowed_imports();
    let modules: BTreeSet<&String> = table.keys().collect();
    let mut violations = Vec::new();

    let src = Path::new(ROOT).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if name != "lib" && !modules.contains(&name) {
            violations.push(format!("src/{name} is not in the AGENTS.md layering table"));
        }
    }

    for (module, allowed) in &table {
        let (files, prefix) = match module.as_str() {
            "main" => (vec![src.join("main.rs")], "statute::"),
            m => (sources(m), "crate::"),
        };
        for file in files {
            for import in imports(&code(&file), prefix) {
                if import != *module && !allowed.contains(&import) {
                    violations.push(format!(
                        "{} imports `{import}`",
                        file.strip_prefix(ROOT).unwrap().display()
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "layering violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn only_git_repo_and_txn_touch_the_outside() {
    let mut violations = Vec::new();
    for module in allowed_imports().keys() {
        if ["git", "repo", "txn", "main"].contains(&module.as_str()) {
            continue;
        }
        // `cli` may launch `$EDITOR` for `decide --edit`.
        let forbidden: &[&str] = if module == "cli" {
            &["std::fs"]
        } else {
            &["std::fs", "std::env", "std::process::Command"]
        };
        for file in sources(module) {
            let code = code(&file);
            for f in forbidden.iter().filter(|f| code.contains(*f)) {
                violations.push(format!(
                    "{} uses {f}",
                    file.strip_prefix(ROOT).unwrap().display()
                ));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
