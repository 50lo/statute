//! The agent plugin: the Stop hook against real repositories, and the skills against the CLI.

mod common;

use common::TestRepo;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn plugin(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("plugin")
        .join(rel)
}

/// Runs each host's packaged Stop command with the built CLI first on the PATH.
fn stop_hook(dir: &Path, input: &str) -> String {
    let claude = run_stop_hook(
        dir,
        input,
        "hooks/hooks.json",
        "CLAUDE_PLUGIN_ROOT",
        &plugin(""),
    );
    let codex = run_stop_hook(dir, input, "hooks/codex.json", "PLUGIN_ROOT", &plugin(""));
    assert_eq!(
        claude, codex,
        "the hosts must receive the same review request"
    );
    codex
}

fn hook_command(config: &str) -> String {
    let hooks: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(plugin(config)).unwrap()).unwrap();
    hooks["hooks"]["Stop"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn run_stop_hook(dir: &Path, input: &str, config: &str, root_var: &str, root: &Path) -> String {
    let bin = Path::new(env!("CARGO_BIN_EXE_statute")).parent().unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let mut child = Command::new("sh")
        .args(["-c", &hook_command(config)])
        .current_dir(dir)
        .env("PATH", path)
        .env_remove("CLAUDE_PLUGIN_ROOT")
        .env_remove("PLUGIN_ROOT")
        .env(root_var, root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

const INPUT: &str = r#"{"hook_event_name": "Stop", "stop_hook_active": false}"#;

/// A clone of the valid fixture with BILL-001 linked, so the base is `origin/HEAD`.
fn linked_repo() -> TestRepo {
    let origin = TestRepo::fixture("valid");
    origin.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() int {\n\treturn 1\n}\n",
    );
    let run = origin.run(&["link", "BILL-001", "billing/refund.go#Approve"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    origin.commit("Initial");
    origin.clone_repo()
}

#[test]
fn stop_hook_asks_for_the_change_skill_once_when_links_are_stale() {
    let repo = linked_repo();
    assert_eq!(stop_hook(&repo.path("."), INPUT), "");

    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() int {\n\treturn 2\n}\n",
    );
    let out = stop_hook(&repo.path("billing"), INPUT);
    let reply: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(reply["decision"], "block");
    assert!(reply["reason"].as_str().unwrap().contains("`change` skill"));

    // The agent is already continuing because of the hook: it may stop.
    let again = r#"{"hook_event_name": "Stop", "stop_hook_active": true}"#;
    assert_eq!(stop_hook(&repo.path("."), again), "");

    let run = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(stop_hook(&repo.path("."), INPUT), "");
}

#[test]
fn stop_hook_is_quiet_outside_a_statute_repository() {
    let repo = TestRepo::empty();
    assert_eq!(stop_hook(&repo.path("."), INPUT), "");
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(stop_hook(dir.path(), INPUT), "");
}

#[test]
fn manifests_parse_and_the_hook_command_exists() {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin(".claude-plugin/plugin.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["name"], "statute");
    let hooks: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(plugin("hooks/hooks.json")).unwrap())
            .unwrap();
    let command = hooks["hooks"]["Stop"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    assert!(command.contains("hooks/stop.sh"), "{command}");

    let codex: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin(".codex-plugin/plugin.json")).unwrap(),
    )
    .unwrap();
    for key in ["name", "version", "description", "homepage"] {
        assert_eq!(codex[key], manifest[key], "{key}");
    }
    let skills = plugin(codex["skills"].as_str().unwrap());
    for skill in ["adopt", "change", "decide", "approve"] {
        assert!(skills.join(skill).join("SKILL.md").is_file());
    }
    let hooks = codex["hooks"].as_str().unwrap();
    assert!(plugin(hooks).is_file());
    assert!(hook_command(hooks).contains("${PLUGIN_ROOT}"));

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let marketplace: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(".agents/plugins/marketplace.json")).unwrap(),
    )
    .unwrap();
    let entry = &marketplace["plugins"][0];
    assert_eq!(entry["name"], codex["name"]);
    assert_eq!(entry["source"]["source"], "local");
    assert_eq!(entry["policy"]["installation"], "AVAILABLE");
    assert_eq!(
        root.join(entry["source"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        plugin("").canonicalize().unwrap()
    );
}

/// Both hosts run the hook command through a shell, so the command must work when the plugin is
/// installed under a path with spaces.
#[test]
fn hook_command_runs_from_a_path_with_spaces() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("plugin with spaces and 'quotes'");
    std::fs::create_dir_all(root.join("hooks")).unwrap();
    std::fs::copy(plugin("hooks/stop.sh"), root.join("hooks/stop.sh")).unwrap();
    let repo = linked_repo();
    repo.remove("billing/refund.go");
    for (config, root_var) in [
        ("hooks/hooks.json", "CLAUDE_PLUGIN_ROOT"),
        ("hooks/codex.json", "PLUGIN_ROOT"),
    ] {
        let output = run_stop_hook(&repo.path("."), INPUT, config, root_var, &root);
        let reply: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(reply["decision"], "block");
        let again = r#"{"hook_event_name":"Stop","stop_hook_active":true}"#;
        assert_eq!(
            run_stop_hook(&repo.path("."), again, config, root_var, &root),
            ""
        );
    }
}

/// A deleted target with no replacement: the `change` skill removes the link, and the commands it
/// names for the verdict then succeed.
#[test]
fn change_skill_handles_a_deleted_target() {
    let skill = std::fs::read_to_string(plugin("skills/change/SKILL.md")).unwrap();
    assert!(
        skill.contains("deleted") && skill.contains("`statute link RULE OLD_TARGET --remove`"),
        "the change skill has no path for a deleted target"
    );

    let repo = linked_repo();
    repo.remove("billing/refund.go");
    let check = repo.run(&["check", "--json"]);
    assert!(check.codes().contains(&"link.missing".to_string()));
    let gap = repo.run(&["gap", "open", "BILL-001", "--note", "Approve was deleted."]);
    assert_eq!(gap.code, 0, "{}", gap.stderr);
    let id = gap.stdout.trim();
    let review = repo.run(&["review", "BILL-001", "gap", id]);
    assert_eq!(review.code, 2, "review refuses while the link is missing");

    let remove = repo.run(&["link", "BILL-001", "billing/refund.go#Approve", "--remove"]);
    assert_eq!(remove.code, 0, "{}", remove.stderr);
    let review = repo.run(&["review", "BILL-001", "gap", id]);
    assert_eq!(review.code, 0, "{}", review.stderr);
    let check = repo.run(&["check", "--json"]);
    assert!(
        !check.codes().iter().any(|c| c.starts_with("link.")),
        "{}",
        check.stdout
    );
    assert_eq!(stop_hook(&repo.path("."), INPUT), "");
}

/// Each skill names itself as its directory does, and every `statute` command the plugin mentions
/// is one the CLI has.
#[test]
fn skills_use_only_commands_the_cli_has() {
    let help = Command::new(env!("CARGO_BIN_EXE_statute"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(help.stdout).unwrap();
    let commands: Vec<&str> = help
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next())
        .collect();

    let mut texts = Vec::new();
    for skill in ["adopt", "change", "decide", "approve"] {
        let text = std::fs::read_to_string(plugin(&format!("skills/{skill}/SKILL.md"))).unwrap();
        let front = text
            .strip_prefix("---\n")
            .unwrap()
            .split("\n---\n")
            .next()
            .unwrap();
        assert!(front.contains(&format!("name: {skill}\n")), "{skill}");
        assert!(front.contains("description: "), "{skill}");
        texts.push((skill.to_string(), text));
    }
    for file in ["README.md", "agents-snippet.md"] {
        texts.push((
            file.to_string(),
            std::fs::read_to_string(plugin(file)).unwrap(),
        ));
    }
    for (name, text) in &texts {
        for (i, _) in text.match_indices("`statute ") {
            let word = text[i + "`statute ".len()..]
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .next()
                .unwrap();
            if word.starts_with('-') {
                continue;
            }
            assert!(commands.contains(&word), "{name}: `statute {word}`");
        }
    }
}
