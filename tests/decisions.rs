//! End-to-end tests for decisions: `decide`, `approve`, and the history checks in `check`.

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
    assert_eq!(entry["approved_by"], serde_json::Value::Null);
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

/// A draft that cannot be recorded is kept, and the error names where.
#[test]
fn decide_edit_keeps_a_draft_it_cannot_record() {
    let repo = TestRepo::fixture("valid");
    let before = repo.read(DECISIONS);
    let kept = |editor: &str, message: &str| {
        let run = repo.run_env(
            &[("EDITOR", editor)],
            &[
                "decide",
                "--edit",
                "--summary",
                "Mine.",
                "--rationale",
                "R.",
            ],
        );
        assert_eq!(run.code, 2, "{}", run.stdout);
        assert!(run.stderr.contains(message), "{}", run.stderr);
        let path = run.stderr.split("draft is kept in ").nth(1).unwrap().trim();
        let draft = std::fs::read_to_string(path).unwrap();
        assert!(draft.contains("Mine."), "{draft}");
        std::fs::remove_file(path).unwrap();
    };
    // A YAML typo.
    kept(
        r#"f() { echo 'notes: "unclosed' >> "$1"; }; f"#,
        "does not parse",
    );
    // A valid draft naming an unknown rule.
    kept(
        r#"f() { sed 's/^retires: \[\]/retires: [ATTR-009]/' "$1" > "$1.new" && mv "$1.new" "$1"; }; f"#,
        "no rule ATTR-009",
    );
    assert_eq!(repo.read(DECISIONS), before);
}

const BILLING: &str = "statute/rules/billing.md";

fn refused(run: Run, message: &str) {
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.stderr.contains(message), "{}", run.stderr);
}

fn errors(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).error_codes()
}

/// From the appendix, "A PM adds a feature that needs a new rule": the rule is agreed before any code
/// is written, and built once both links exist.
#[test]
fn approve_agrees_a_new_rule_and_term_before_the_code() {
    let repo = TestRepo::fixture("valid");
    repo.commit("Initial");
    let rule = ok(repo.run(&["rule", "new", "billing", "Refunds over $1,000 are logged"]));
    let rule = rule.stdout.trim().to_string();
    let text = repo.read(BILLING).replace(
        "terms: []\nexceptions: []\nprotects: []\nboundary: []",
        "terms: [Approver]\nexceptions: []\nprotects: [A $1,200 refund is logged.]\nboundary: [A $900 refund is not logged.]",
    );
    repo.write(BILLING, &text);
    repo.write(
        TERMS,
        &format!(
            "{}\n## Approver\n\nThe second person who approves a large refund.\n",
            repo.read(TERMS)
        ),
    );
    let id = ok(repo.run(&[
        "decide",
        "--establishes",
        &rule,
        "--terms",
        "Approver",
        "--summary",
        "Log large refunds.",
        "--rationale",
        "Audit.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let report = ok(repo.run(&["approve", &id, "--json"])).json();
    assert_eq!(report["agreed_rules"], serde_json::json!([rule]));
    assert_eq!(report["agreed_terms"], serde_json::json!(["Approver"]));
    assert_eq!(report["built_rules"], serde_json::json!([]));
    assert_eq!(report["approved_by"]["name"], "Pat Example");

    let entry = decision(&repo, &id);
    assert_eq!(entry["status"], "active");
    assert_eq!(entry["approved_by"]["email"], "pat@example.com");
    assert!(repo.read(TERMS).ends_with("```yaml\nagreed: agreed\n```\n"));
    let rules = repo.read(BILLING);
    let section = &rules[rules.find(&rule).unwrap()..];
    assert!(section.contains("agreed: agreed "), "{section}");
    assert!(errors(&repo).is_empty(), "{:?}", errors(&repo));

    // The code and the test come after; the second link completes the rule.
    repo.write("billing/log.go", "package billing\n\nfunc Log() {}\n");
    repo.write(
        "billing/log_test.go",
        "package billing\n\nfunc TestLog() {}\n",
    );
    ok(repo.run(&["link", &rule, "billing/log.go#Log"]));
    let run = ok(repo.run(&[
        "link",
        &rule,
        "billing/log_test.go#TestLog",
        "--test",
        "--json",
    ]));
    assert_eq!(run.json()["built"], true);

    let run = repo.run(&["approve", &id]);
    refused(run, "is already active");
}

/// `approve` refuses when the result would have errors about what it approves, and changes nothing.
#[test]
fn approve_refuses_a_rule_without_cases_and_writes_nothing() {
    let repo = TestRepo::fixture("valid");
    let id = ok(repo.run(&[
        "decide",
        "--establishes",
        "BILL-001",
        "--summary",
        "S.",
        "--rationale",
        "R.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let before = (repo.read(DECISIONS), repo.read(BILLING));
    let run = repo.run(&["approve", &id]);
    refused(run, "rule.cases");
    assert_eq!((repo.read(DECISIONS), repo.read(BILLING)), before);

    refused(
        repo.run(&["approve", "D261231-zzzzz"]),
        "no decision D261231-zzzzz",
    );
    // Errors about other items do not block: D261014-x4p9t is still approved with BILL-001 broken.
    ok(repo.run(&["approve", "D261014-x4p9t"]));
}

/// From the appendix, "A rule needs to change", ending with a retirement: `approve` retires the rule and
/// removes its links.
#[test]
fn approve_retires_a_rule_and_removes_its_links() {
    let repo = TestRepo::fixture("valid");
    repo.write("src/keep.go", "package src\n\nfunc Keep() {}\n");
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Keep"]));
    let id = ok(repo.run(&[
        "decide",
        "--retires",
        "ATTR-001",
        "--summary",
        "S.",
        "--rationale",
        "R.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let report = ok(repo.run(&["approve", &id, "--json"])).json();
    assert_eq!(report["retired_rules"], serde_json::json!(["ATTR-001"]));
    assert_eq!(report["removed_links"], 1);
    assert_eq!(repo.read("statute.lock"), "{\"version\": 1}\n");
    assert!(
        repo.read(ATTRIBUTION)
            .contains("agreed: retired           #")
    );
}

#[test]
fn approve_needs_the_approver_from_git() {
    let repo = TestRepo::fixture("valid");
    repo.git(&["config", "--unset", "user.email"]);
    let run = repo.run_env(
        &[
            ("GIT_CONFIG_GLOBAL", "/dev/null"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
        ],
        &["approve", "D261014-x4p9t"],
    );
    refused(run, "user.email");
}

/// Spec 4.1 and 4.2: an agreed rule or term edited without a decision, and an active decision edited
/// or deleted, are errors against the base.
#[test]
fn check_reports_history_changes_against_the_base() {
    let repo = TestRepo::fixture("valid");
    repo.commit("Initial");
    assert!(errors(&repo).is_empty());

    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace(
            "and stores it like any other Session.",
            "and stores it   like any\nother Session.\n\nMore notes.",
        ),
    );
    assert!(
        errors(&repo).is_empty(),
        "whitespace and notes are not compared"
    );
    repo.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    let terms = repo.read(TERMS);
    repo.write(
        TERMS,
        &terms.replace("code: [Commit]", "code: [Commit, Sha]"),
    );
    assert_eq!(
        errors(&repo),
        [
            "rule.changed_without_decision",
            "term.changed_without_decision"
        ]
    );

    // A proposed decision that lists the rule and the term carries the change.
    ok(repo.run(&[
        "decide",
        "--changes",
        "ATTR-001",
        "--terms",
        "Commit",
        "--summary",
        "S.",
        "--rationale",
        "R.",
    ]));
    assert!(errors(&repo).is_empty(), "{:?}", errors(&repo));

    let decisions = repo.read(DECISIONS);
    repo.write(
        DECISIONS,
        &decisions
            .replace("Sessions no longer expire.", "Sessions never expire.")
            .replace("D261001-a2b3c", "D261001-zzzzz"),
    );
    let codes = errors(&repo);
    assert!(
        codes.contains(&"decision.history_deleted".to_string()),
        "{codes:?}"
    );
    assert!(
        codes.contains(&"decision.history_edited".to_string()),
        "{codes:?}"
    );
}

/// `approve` after the proposal has already reached the default branch.
#[test]
fn approve_works_when_the_proposal_is_already_on_the_default_branch() {
    let origin = TestRepo::fixture("valid");
    let rules = origin.read(ATTRIBUTION);
    origin.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    ok(origin.run(&[
        "decide",
        "--changes",
        "ATTR-001",
        "--summary",
        "S.",
        "--rationale",
        "R.",
    ]));
    origin.commit("Propose a change to ATTR-001");

    let clone = origin.clone_repo();
    let id = decision_ids(&clone).pop().unwrap();
    ok(clone.run(&["approve", &id]));
    assert!(errors(&clone).is_empty(), "{:?}", errors(&clone));
    clone.commit("Approve");
    assert!(errors(&clone).is_empty(), "{:?}", errors(&clone));
}

fn decision_ids(repo: &TestRepo) -> Vec<String> {
    repo.read(DECISIONS)
        .lines()
        .filter_map(|l| l.strip_prefix("- id: "))
        .map(str::to_string)
        .collect()
}

/// `approve` sets `built: built` on a rule it agrees that is complete.
#[test]
fn approve_builds_a_complete_rule() {
    let repo = TestRepo::fixture("valid");
    repo.write("src/link.go", "package src\n\nfunc Link() {}\n");
    ok(repo.run(&["link", "ATTR-002", "src/link.go#Link"]));
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace(
            "agreed: proposed\nbuilt: planned\nterms: [Session, Commit]",
            "agreed: proposed\nbuilt: planned\nmanual_check: \"Someone reads the commit log.\"\nterms: [Session, Commit]",
        ),
    );
    let report = ok(repo.run(&["approve", "D261014-x4p9t", "--json"])).json();
    assert_eq!(report["built_rules"], serde_json::json!(["ATTR-002"]));
    assert!(
        repo.read(ATTRIBUTION)
            .contains("agreed: agreed\nbuilt: built\nmanual_check")
    );
    assert!(errors(&repo).is_empty(), "{:?}", errors(&repo));
}

/// Spec 4.2: `approve` refuses a rule that uses a term that is neither agreed nor approved with it.
#[test]
fn approve_refuses_a_rule_using_an_unapproved_term() {
    let repo = TestRepo::fixture("valid");
    repo.write(
        TERMS,
        &format!(
            "{}\n## Approver\n\nThe second person who approves a large refund.\n",
            repo.read(TERMS)
        ),
    );
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace(
            "terms: [Session, Commit]\nprotects:",
            "terms: [Approver]\nprotects:",
        ),
    );
    let before = repo.read(DECISIONS);
    refused(
        repo.run(&["approve", "D261014-x4p9t"]),
        "uses term `Approver`, which is not agreed",
    );
    assert_eq!(repo.read(DECISIONS), before);

    // Proposed in a separate decision that is not approved with it: still refused.
    let id = ok(repo.run(&[
        "decide",
        "--terms",
        "Approver",
        "--summary",
        "S.",
        "--rationale",
        "R.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();
    refused(
        repo.run(&["approve", "D261014-x4p9t"]),
        "uses term `Approver`, which is not agreed",
    );
    // Approved together, the term is approved with the rule.
    ok(repo.run(&["approve", "D261014-x4p9t", &id]));
}
