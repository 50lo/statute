//! End-to-end tests: run the binary in a temporary repository and inspect files and reports.

mod common;

use common::TestRepo;

#[test]
fn init_writes_the_layout_and_check_passes() {
    let repo = TestRepo::empty();
    let run = repo.run(&["init", "--json"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(
        run.stdout.starts_with("{\n  \"schema\": 1,"),
        "{}",
        run.stdout
    );
    let report = run.json();
    assert_eq!(report["agents_md"], "created");
    assert_eq!(
        report["created"],
        serde_json::json!([
            "statute.yml",
            "statute.lock",
            "statute/terms.md",
            "statute/decisions.db",
            "statute/gaps.yml",
            "statute/rules/",
            "AGENTS.md"
        ])
    );
    assert_eq!(
        repo.read("statute.yml"),
        "version: 1\nsummary_max: 600      # decision summary cap, in characters\n"
    );
    assert_eq!(repo.read("statute.lock"), "{\"version\": 1}\n");
    assert!(repo.path("statute/rules").is_dir());
    assert!(
        repo.read("AGENTS.md")
            .contains("Run `statute ratify` only after a human")
    );

    let check = repo.run(&["check", "--json"]);
    assert_eq!((check.code, check.codes()), (0, vec![]));
}

#[test]
fn init_appends_the_snippet_once_and_refuses_a_second_run() {
    let repo = TestRepo::empty();
    repo.write("AGENTS.md", "# Project\n\nOur conventions.\n");
    let run = repo.run(&["init", "--json"]);
    assert_eq!(run.json()["agents_md"], "updated");
    let agents = repo.read("AGENTS.md");
    assert!(
        agents.starts_with("# Project\n\nOur conventions.\n\n<!-- statute:begin -->\n## Statute\n")
    );

    let again = repo.run(&["init"]);
    assert_eq!(again.code, 2);
    assert!(again.stderr.contains("already set up"), "{}", again.stderr);
    assert_eq!(repo.read("AGENTS.md"), agents);
}

#[test]
fn init_from_a_subdirectory_sets_up_the_git_root() {
    let repo = TestRepo::empty();
    repo.write("src/main.go", "package main\n");
    assert_eq!(repo.run_in("src", &["init"]).code, 0);
    assert!(repo.path("statute.yml").is_file());
    assert!(!repo.path("src/statute.yml").exists());
    // Other commands find the root from below it too.
    assert_eq!(repo.run_in("src", &["check"]).code, 0);
}

#[test]
fn rule_new_appends_a_skeleton_with_the_next_id_and_keeps_existing_bytes() {
    let repo = TestRepo::fixture("valid");
    let before = repo.read("statute/rules/attribution.md");

    let run = repo.run(&[
        "rule",
        "new",
        "attribution",
        "Commits without a Session are ignored",
    ]);
    assert_eq!(
        (run.code, run.stdout.as_str()),
        (0, "ATTR-004\n"),
        "{}",
        run.stderr
    );
    let after = repo.read("statute/rules/attribution.md");
    assert!(after.starts_with(&before), "existing bytes changed");
    insta::assert_snapshot!(&after[before.len()..]);

    let json = repo
        .run(&["rule", "new", "attribution", "Another", "--json"])
        .json();
    assert_eq!(json["id"], "ATTR-005");
    assert_eq!(json["file"], "statute/rules/attribution.md");

    // New rules are drafts: listed by `status`, not reported by `check`.
    assert_eq!(repo.run(&["check", "--json"]).codes(), Vec::<String>::new());
    let status = repo
        .run(&["status", "--area", "attribution", "--json"])
        .json();
    let rules = status["areas"][0]["rules"].as_array().unwrap();
    let draft = |id: &str| rules.iter().find(|r| r["id"] == id).unwrap()["draft"].clone();
    assert_eq!(
        (draft("ATTR-004"), draft("ATTR-005"), draft("ATTR-002")),
        (true.into(), true.into(), false.into())
    );
}

#[test]
fn rule_new_needs_a_rule_file_with_a_prefix() {
    let repo = TestRepo::empty();
    repo.run(&["init"]);

    let missing = repo.run(&["rule", "new", "billing", "Refunds", "--json"]);
    assert_eq!(missing.code, 2);
    assert!(
        missing.json()["error"]
            .as_str()
            .unwrap()
            .contains("statute/rules/billing.md does not exist")
    );

    repo.write("statute/rules/billing.md", "Prose without frontmatter.\n");
    let no_prefix = repo.run(&["rule", "new", "billing", "Refunds"]);
    assert_eq!(no_prefix.code, 2);
    assert!(
        no_prefix.stderr.contains("needs a valid `prefix:`"),
        "{}",
        no_prefix.stderr
    );
    assert_eq!(
        repo.read("statute/rules/billing.md"),
        "Prose without frontmatter.\n"
    );

    assert_eq!(repo.run(&["rule", "new", "../billing", "Refunds"]).code, 2);

    repo.write("statute/rules/billing.md", "---\nprefix: BILL\n---\n");
    assert_eq!(
        repo.run(&["rule", "new", "billing", "Refunds"]).stdout,
        "BILL-001\n"
    );
}

#[test]
fn rule_new_refuses_to_append_inside_an_unclosed_code_block() {
    let repo = TestRepo::fixture("valid");
    let path = "statute/rules/billing.md";
    let text = format!(
        "{}\nNotes with an unclosed fence:\n\n```text\nunclosed\n",
        repo.read(path)
    );
    repo.write(path, &text);

    let run = repo.run(&["rule", "new", "billing", "Hidden rule"]);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("unclosed block"), "{}", run.stderr);
    assert_eq!(repo.read(path), text);
}

#[test]
fn missing_layout_files_are_file_errors() {
    for path in [
        "statute/terms.md",
        "statute/decisions.db",
        "statute/gaps.yml",
    ] {
        let repo = TestRepo::fixture("valid");
        std::fs::remove_file(repo.path(path)).unwrap();
        for args in [
            &["check"][..],
            &["status"],
            &["rule", "new", "billing", "X"],
        ] {
            let run = repo.run(args);
            assert_eq!(run.code, 2, "{path} {args:?}");
            assert!(
                run.stderr.contains(&format!("{path} is missing")),
                "{}",
                run.stderr
            );
        }
    }
    // Empty files are fine, and so is a missing rules directory, which git does not keep.
    let repo = TestRepo::fixture("valid");
    for path in [
        "statute/terms.md",
        "statute/decisions.db",
        "statute/gaps.yml",
    ] {
        repo.write(path, "");
    }
    std::fs::remove_dir_all(repo.path("statute/rules")).unwrap();
    assert_eq!(repo.run(&["check"]).code, 0);
}

#[test]
fn the_valid_fixture_has_no_findings() {
    let run = TestRepo::fixture("valid").run(&["check"]);
    assert_eq!((run.code, run.stdout.as_str()), (0, "No findings.\n"));
}

#[test]
fn check_reports_every_problem_in_the_broken_fixture() {
    let run = TestRepo::fixture("broken").run(&["check", "--json"]);
    assert_eq!(run.code, 1);
    insta::assert_json_snapshot!(run.json());
}

#[test]
fn check_text_output() {
    let run = TestRepo::fixture("broken").run(&["check"]);
    assert_eq!(run.code, 1);
    insta::assert_snapshot!(run.stdout);
}

#[test]
fn warnings_alone_exit_0() {
    let repo = TestRepo::fixture("valid");
    let path = "statute/rules/billing.md";
    let text = repo.read(path).replace(
        "second person approves it",
        "second person approves it in chat",
    );
    repo.write(path, &text);
    let run = repo.run(&["check", "--json"]);
    assert_eq!(
        (run.code, run.codes()),
        (0, vec!["rule.avoid_word".to_string()])
    );
    assert_eq!(run.json()["warnings"], 1);
}

#[test]
fn unknown_metadata_keys_are_errors() {
    let repo = TestRepo::fixture("valid");
    let path = "statute/rules/billing.md";
    repo.write(
        path,
        &repo.read(path).replace("protects: []", "protect: []"),
    );
    let run = repo.run(&["check", "--json"]);
    assert_eq!(
        (run.code, run.codes()),
        (1, vec!["rule.schema".to_string()])
    );
    assert_eq!(run.json()["findings"][0]["line"], 12);
}

#[test]
fn an_unparseable_decisions_file_hides_the_checks_that_need_it() {
    let repo = TestRepo::fixture("valid");
    repo.write("statute/decisions.db", "- id: [unclosed\n");
    let run = repo.run(&["check", "--json"]);
    // No cascade of "not established" errors for the agreed and retired rules.
    assert_eq!(
        (run.code, run.codes()),
        (1, vec!["decision.parse".to_string()])
    );
    // Nothing is a draft while establishment is unknown.
    let status = repo.run(&["status", "--json"]).json();
    assert!(!status.to_string().contains("\"draft\":true"));
}

#[test]
fn status_lists_rules_by_area() {
    let repo = TestRepo::fixture("valid");
    let run = repo.run(&["status"]);
    assert_eq!(run.code, 0);
    insta::assert_snapshot!(run.stdout);
    insta::assert_json_snapshot!(repo.run(&["status", "--area", "billing", "--json"]).json());

    let unknown = repo.run(&["status", "--area", "nope"]);
    assert_eq!(unknown.code, 2);
    assert!(unknown.stderr.contains("no rule file for area `nope`"));
}

#[test]
fn commands_outside_a_statute_repository_exit_2() {
    let repo = TestRepo::empty();
    for args in [&["check"][..], &["status"], &["rule", "new", "a", "b"]] {
        let run = repo.run(args);
        assert_eq!(run.code, 2, "{args:?}");
        assert!(run.stderr.contains("run `statute init`"), "{}", run.stderr);
    }
}

#[test]
fn usage_errors_exit_2() {
    let repo = TestRepo::empty();
    assert_eq!(repo.run(&["rule", "new", "billing"]).code, 2);
    assert_eq!(repo.run(&["check", "--all"]).code, 2);
}

#[test]
fn show_prints_rules_terms_and_decisions_once() {
    let repo = TestRepo::fixture("valid");
    let run = repo.run(&["show", "ATTR-001", "Session", "D261001-a2b3c", "ATTR-001"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    insta::assert_snapshot!(run.stdout);
    insta::assert_json_snapshot!(repo.run(&["show", "ATTR-003", "--json"]).json());

    let json = repo.run(&["show", "Session", "Session", "--json"]).json();
    assert_eq!(json["items"].as_array().unwrap().len(), 1);
}

#[test]
fn show_refuses_an_unknown_name() {
    let repo = TestRepo::fixture("valid");
    let run = repo.run(&["show", "ATTR-001", "ATTR-999"]);
    assert_eq!((run.code, run.stdout.as_str()), (2, ""));
    assert!(
        run.stderr
            .contains("no rule, term or decision named `ATTR-999`"),
        "{}",
        run.stderr
    );
    assert_eq!(repo.run(&["show"]).code, 2);
}

#[test]
fn show_works_when_decisions_and_gaps_do_not_parse() {
    let repo = TestRepo::fixture("valid");
    repo.write("statute/decisions.db", "- id: [unclosed\n");
    repo.write("statute/gaps.yml", "- id: [unclosed\n");
    let run = repo.run(&["show", "ATTR-001", "--json"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let rule = &run.json()["items"][0];
    assert_eq!(
        (&rule["decisions"], &rule["gaps"]),
        (&serde_json::Value::Null, &serde_json::Value::Null)
    );
    let text = repo.run(&["show", "ATTR-001"]).stdout;
    assert!(text.contains("decisions.db does not parse"), "{text}");

    let missing = repo.run(&["show", "D261001-a2b3c"]);
    assert_eq!(missing.code, 2);
    assert!(
        missing
            .stderr
            .contains("statute/decisions.db does not parse"),
        "{}",
        missing.stderr
    );
}
