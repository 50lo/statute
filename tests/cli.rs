use assert_cmd::Command;

fn statute() -> Command {
    Command::cargo_bin("statute").unwrap()
}

#[test]
fn help_lists_every_command() {
    let output = statute().arg("--help").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for command in [
        "init", "rule", "show", "check", "link", "review", "decide", "ratify", "gap", "status",
    ] {
        assert!(help.contains(command), "help does not list `{command}`");
    }
}

#[test]
fn unimplemented_command_exits_with_2() {
    let output = statute()
        .args(["review", "ATTR-003", "ok"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not implemented"), "{stderr}");
}

#[test]
fn json_error_is_machine_readable() {
    let output = statute().args(["status", "--json"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], 1);
    assert!(value["error"].is_string());
}

#[test]
fn md_and_json_conflict() {
    let output = statute()
        .args(["check", "--md", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}
