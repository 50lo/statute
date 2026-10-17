//! HIST rules: active decisions, agreed rules and agreed terms change only through new decisions.

mod common;

use common::{Run, TestRepo};

const ATTRIBUTION: &str = "statute/rules/attribution.md";
const DECISIONS: &str = "statute/decisions.db";
const TERMS: &str = "statute/terms.md";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn errors(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).error_codes()
}

fn base() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.commit("Base");
    assert_eq!(errors(&repo), Vec::<String>::new());
    repo
}

#[test]
fn hist_001_an_active_decision_is_never_edited_or_deleted() {
    let repo = base();
    let decisions = repo.read(DECISIONS);

    repo.write(
        DECISIONS,
        &decisions.replace("Sessions no longer expire.", "Sessions never expire."),
    );
    assert_eq!(errors(&repo), ["decision.history_edited"]);

    let without_first = decisions
        .split_once("- id: D261010-k7m2q")
        .map(|(_, rest)| format!("- id: D261010-k7m2q{rest}"))
        .unwrap();
    repo.write(DECISIONS, &without_first);
    assert!(
        errors(&repo).contains(&"decision.history_deleted".to_string()),
        "{:?}",
        errors(&repo)
    );
}

#[test]
fn hist_001_a_proposed_decision_can_be_edited_or_deleted() {
    let repo = base();
    let decisions = repo.read(DECISIONS);

    repo.write(
        DECISIONS,
        &decisions.replace(
            "Context alone never links a Session to a commit.",
            "Context alone never creates a contribution.",
        ),
    );
    assert_eq!(errors(&repo), Vec::<String>::new());

    let (active, _) = decisions.split_once("- id: D261014-x4p9t").unwrap();
    repo.write(DECISIONS, active);
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn hist_002_an_agreed_rule_changes_only_with_a_decision_that_lists_it() {
    let repo = base();
    let rules = repo.read(ATTRIBUTION);
    let reworded = rules.replace("and stores it like any other Session.", "for 30 days.");

    repo.write(ATTRIBUTION, &reworded);
    assert_eq!(errors(&repo), ["rule.changed_without_decision"]);

    repo.write(
        ATTRIBUTION,
        &rules.replace(
            "scope: Session collection",
            "scope: Session collection and export",
        ),
    );
    assert_eq!(errors(&repo), ["rule.changed_without_decision"]);

    // The rewording is accepted together with a proposed decision that lists the rule.
    repo.write(ATTRIBUTION, &reworded);
    ok(repo.run(&[
        "decide",
        "--changes",
        "ATTR-001",
        "--summary",
        "Keep Sessions for 30 days.",
        "--rationale",
        "R.",
    ]));
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn hist_002_a_decision_active_in_the_base_does_not_carry_a_new_change() {
    let repo = TestRepo::fixture("valid");
    let decisions = repo.read(DECISIONS);
    repo.write(
        DECISIONS,
        &format!(
            "{decisions}- id: D261011-c4ng3
  timestamp: \"2026-10-11T09:00:00Z\"
  actor: user
  status: active
  ratified_by: {{name: \"Pat Example\", email: \"pat@example.com\", at: \"2026-10-11T09:30:00Z\"}}
  summary: \"Clarify how Sessions without a repository are stored.\"
  rationale: \"R.\"
  changes: [ATTR-001]
"
        ),
    );
    repo.commit("Base with an active decision that changed ATTR-001");
    assert_eq!(errors(&repo), Vec::<String>::new());

    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    assert_eq!(errors(&repo), ["rule.changed_without_decision"]);
}

#[test]
fn hist_002_notes_and_formatting_of_an_agreed_rule_need_no_decision() {
    let repo = base();
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules
            .replace(
                "and stores it like any other Session.",
                "and stores it   like any\nother Session.",
            )
            .replace(
                "Notes: this came out of the collector filter incident.",
                "Notes: see the collector filter incident report.",
            ),
    );
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn hist_003_an_agreed_term_changes_only_with_a_decision_that_lists_it() {
    let repo = base();
    let terms = repo.read(TERMS);

    repo.write(
        TERMS,
        &terms.replace("code: [Commit]", "code: [Commit, Sha]"),
    );
    assert_eq!(errors(&repo), ["term.changed_without_decision"]);

    repo.write(
        TERMS,
        &terms.replace(
            "One conversation between a developer and a coding agent.",
            "One conversation between a person and a coding agent.",
        ),
    );
    assert_eq!(errors(&repo), ["term.changed_without_decision"]);

    ok(repo.run(&[
        "decide",
        "--terms",
        "Session",
        "--summary",
        "Sessions belong to any person.",
        "--rationale",
        "R.",
    ]));
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn hist_003_aliases_and_avoid_words_of_an_agreed_term_need_no_decision() {
    let repo = base();
    let terms = repo.read(TERMS);
    repo.write(
        TERMS,
        &terms
            .replace("aliases: [agent session]", "aliases: [agent session, run]")
            .replace("avoid: [chat]", "avoid: [chat, thread]"),
    );
    assert_eq!(errors(&repo), Vec::<String>::new());
}
