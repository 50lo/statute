//! End-to-end tests for decisions: `decide`.

mod common;

use common::{Run, TestRepo};

const DECISIONS: &str = "statute/decisions.db";
const TERMS: &str = "statute/terms.md";
const ATTRIBUTION: &str = "statute/rules/attribution.md";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn is_decision_id(id: &str) -> bool {
    let b = id.as_bytes();
    id.len() == 13
        && b[0] == b'D'
        && b[1..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..]
            .iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// A decision's entry, through `show`.
fn decision(repo: &TestRepo, id: &str) -> serde_json::Value {
    let shown = ok(repo.run(&["show", id, "--json"])).json();
    shown["items"][0].clone()
}

/// From the appendix, "A rule needs to change": `decide --changes` sets the rule back to proposed.
#[test]
fn decide_records_a_proposed_decision_and_proposes_what_it_changes() {
    let repo = TestRepo::fixture("valid");
    let rules_before = repo.read(ATTRIBUTION);
    let terms_before = repo.read(TERMS);

    let run = ok(repo.run(&[
        "decide",
        "--changes",
        "ATTR-001",
        "--terms",
        "Session",
        "--summary",
        "Keep Sessions without a repository for 30 days.",
        "--rationale",
        "Storage: they are rarely looked at.",
        "--json",
    ]));
    let report = run.json();
    let id = report["id"].as_str().unwrap();
    assert!(is_decision_id(id), "{id}");
    assert_eq!(report["proposed_rules"], serde_json::json!(["ATTR-001"]));
    assert_eq!(report["proposed_terms"], serde_json::json!(["Session"]));

    let entry = decision(&repo, id);
    assert_eq!(entry["status"], "proposed");
    assert_eq!(entry["actor"], "agent");
    assert_eq!(entry["ratified_by"], serde_json::Value::Null);
    assert_eq!(entry["changes"], serde_json::json!(["ATTR-001"]));
    assert_eq!(entry["terms"], serde_json::json!(["Session"]));
    assert!(entry["timestamp"].as_str().unwrap().ends_with('Z'));

    // Only the `agreed:` lines change.
    assert_eq!(
        repo.read(ATTRIBUTION),
        rules_before.replacen(
            "agreed: agreed            #",
            "agreed: proposed          #",
            1
        )
    );
    assert_eq!(
        repo.read(TERMS),
        terms_before.replacen(
            "agreed: agreed                   #",
            "agreed: proposed                 #",
            1
        )
    );
    // The earlier entries survive the rewrite.
    assert_eq!(
        decision(&repo, "D261001-a2b3c")["establishes"],
        serde_json::json!(["ATTR-001", "ATTR-003"])
    );
    assert!(ok(repo.run(&["check", "--json"])).error_codes().is_empty());

    // The text output is the ID, then what became proposed.
    let run = ok(repo.run(&[
        "decide",
        "--retires",
        "BILL-001",
        "--summary",
        "S.",
        "--rationale",
        "R.",
    ]));
    assert!(is_decision_id(run.stdout.trim()), "{}", run.stdout);
}

/// From the appendix, "The feature needs a new term": a new term gets `agreed: proposed`, and a
/// `yaml` block when it has none.
#[test]
fn decide_proposes_a_new_term_and_establishes_a_new_rule() {
    let repo = TestRepo::fixture("valid");
    let id = ok(repo.run(&["rule", "new", "billing", "Refunds are logged"]));
    let rule = id.stdout.trim();
    repo.write(
        TERMS,
        &format!(
            "{}\n## Approver\n\nThe second person who approves a large refund.\n",
            repo.read(TERMS)
        ),
    );

    let run = ok(repo.run(&[
        "decide",
        "--establishes",
        rule,
        "--terms",
        "Approver",
        "--summary",
        "S.",
        "--rationale",
        "R.",
        "--json",
    ]));
    assert_eq!(run.json()["proposed_rules"], serde_json::json!([]));
    assert!(repo.read(TERMS).ends_with(
        "## Approver\n\nThe second person who approves a large refund.\n\n```yaml\nagreed: proposed\n```\n"
    ));
    let entry = decision(&repo, run.json()["id"].as_str().unwrap());
    assert_eq!(entry["establishes"], serde_json::json!([rule]));
}

#[test]
fn decide_refuses_pending_and_unknown_rules_and_terms() {
    let repo = TestRepo::fixture("valid");
    let before = repo.read(DECISIONS);
    let refused = |args: &[&str], message: &str| {
        let mut all = vec!["decide", "--summary", "S.", "--rationale", "R."];
        all.extend_from_slice(args);
        let run = repo.run(&all);
        assert_eq!(run.code, 2, "{args:?}: {}", run.stdout);
        assert!(run.stderr.contains(message), "{args:?}: {}", run.stderr);
    };
    // ATTR-002 is in the proposed decision D261014-x4p9t.
    refused(
        &["--changes", "ATTR-002"],
        "already in proposed decision D261014-x4p9t",
    );
    refused(&["--establishes", "ATTR-009"], "no rule ATTR-009");
    refused(&["--terms", "Approver"], "no term `Approver`");
    assert_eq!(repo.read(DECISIONS), before);

    let run = repo.run(&["decide", "--summary", "S."]);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("--rationale"), "{}", run.stderr);
}

/// `--edit` opens the draft in the editor, and records what the person saved.
#[test]
fn decide_edit_records_the_edited_draft() {
    let repo = TestRepo::fixture("valid");
    let editor =
        r#"f() { sed 's/Draft summary/Edited summary/' "$1" > "$1.new" && mv "$1.new" "$1"; }; f"#;
    let run = ok(repo.run_env(
        &[("EDITOR", editor)],
        &[
            "decide",
            "--edit",
            "--actor",
            "user",
            "--summary",
            "Draft summary",
            "--rationale",
            "R.",
            "--json",
        ],
    ));
    let entry = decision(&repo, run.json()["id"].as_str().unwrap());
    assert_eq!(entry["summary"], "Edited summary");
    assert_eq!(entry["actor"], "user");

    // Saving an empty file records nothing.
    let before = repo.read(DECISIONS);
    let run = repo.run_env(
        &[("EDITOR", r#"f() { : > "$1"; }; f"#)],
        &["decide", "--edit"],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr.contains("no decision was recorded"),
        "{}",
        run.stderr
    );
    assert_eq!(repo.read(DECISIONS), before);
}
