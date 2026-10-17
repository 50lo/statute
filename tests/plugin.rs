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

/// Runs the Stop hook in `dir` with the built `statute` first on the `PATH`.
fn stop_hook(dir: &Path, input: &str) -> String {
    let bin = Path::new(env!("CARGO_BIN_EXE_statute")).parent().unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let mut child = Command::new("sh")
        .arg(plugin("hooks/stop.sh"))
        .current_dir(dir)
        .env("PATH", path)
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
    let script = command.strip_prefix("${CLAUDE_PLUGIN_ROOT}/").unwrap();
    assert!(plugin(script).is_file(), "{script}");
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
    for skill in ["adopt", "change", "decide", "ratify"] {
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
