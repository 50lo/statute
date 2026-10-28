//! CORE-001's mechanical case requirements. Whether a rule is one checkable requirement still
//! needs human or agent judgment; these tests do not claim to assess the meaning of its text.

mod common;

use common::{Run, TestRepo};

const RULES: &str = "statute/rules/billing.md";
const DECISIONS: &str = "statute/decisions.db";
const PROTECTED: &str = "protects: [A refund over $500 needs a second approval.]";
const BOUNDARY: &str = "boundary: [A refund of $500 does not need a second approval.]";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}\n{}", run.stdout, run.stderr);
    run
}

fn propose(repo: &TestRepo) -> String {
    ok(repo.run(&[
        "decide",
        "--establishes",
        "BILL-001",
        "--summary",
        "Require a second approval for refunds over $500.",
        "--rationale",
        "A second person checks large refunds.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn complete_cases(repo: &TestRepo) {
    let text = repo
        .read(RULES)
        .replace("protects: []", PROTECTED)
        .replace("boundary: []", BOUNDARY);
    repo.write(RULES, &text);
}

fn assert_agreed_case_is_required(present: &str, missing: &str) {
    let repo = TestRepo::fixture("valid");
    complete_cases(&repo);
    let decision = propose(&repo);
    ok(repo.run(&["approve", &decision]));

    let agreed = repo.read(RULES);
    assert!(agreed.contains("agreed: agreed"));
    repo.write(RULES, &agreed.replace(present, missing));

    let check = repo.run(&["check", "--json"]);
    assert_eq!(check.code, 1, "{}\n{}", check.stdout, check.stderr);
    assert_eq!(check.codes(), ["rule.cases_missing"]);
    let report = check.json();
    assert_eq!(report["findings"][0]["rule"], "BILL-001");
    assert_eq!(report["findings"][0]["severity"], "error");
    assert_eq!(report["findings"][0]["file"], RULES);
}

#[test]
fn an_agreed_rule_without_a_protected_case_is_invalid_even_with_a_boundary_case() {
    assert_agreed_case_is_required(PROTECTED, "protects: []");
}

#[test]
fn an_agreed_rule_without_a_boundary_case_is_invalid_even_with_a_protected_case() {
    assert_agreed_case_is_required(BOUNDARY, "boundary: []");
}

#[test]
fn approval_requires_each_kind_of_case_and_can_retry_after_the_missing_case_is_written() {
    for (present, missing) in [(PROTECTED, "protects: []"), (BOUNDARY, "boundary: []")] {
        let repo = TestRepo::fixture("valid");
        complete_cases(&repo);
        repo.write(RULES, &repo.read(RULES).replace(present, missing));
        let decision = propose(&repo);

        // The unfinished cases are valid while the rule remains proposed.
        assert!(ok(repo.run(&["check", "--json"])).codes().is_empty());
        let before = (
            repo.read(RULES),
            repo.read(DECISIONS),
            repo.read("statute.lock"),
        );
        let refused = repo.run(&["approve", &decision]);
        assert_eq!(refused.code, 2, "{}", refused.stdout);
        assert!(
            refused.stderr.contains("rule.cases_missing"),
            "{}",
            refused.stderr
        );
        assert_eq!(
            (
                repo.read(RULES),
                repo.read(DECISIONS),
                repo.read("statute.lock")
            ),
            before,
            "refused approval must leave the rule, decision and links unchanged"
        );

        repo.write(RULES, &repo.read(RULES).replace(missing, present));
        let approved = ok(repo.run(&["approve", &decision, "--json"])).json();
        assert_eq!(approved["agreed_rules"], serde_json::json!(["BILL-001"]));
        assert!(repo.read(RULES).contains("agreed: agreed"));
        assert!(ok(repo.run(&["check", "--json"])).codes().is_empty());
    }
}

#[test]
fn a_proposed_draft_can_keep_empty_cases_while_its_statement_is_being_written() {
    let repo = TestRepo::fixture("valid");
    let created = ok(repo.run(&["rule", "new", "billing", "Refund receipts", "--json"])).json();
    let id = created["id"].as_str().unwrap();
    let text = repo.read(RULES).replace(
        "State the obligation in one or two sentences: what must or must not happen, to what, and under which conditions.",
        "Draft: describe when a refund receipt is sent.",
    );
    repo.write(RULES, &text);

    assert!(ok(repo.run(&["check", "--json"])).codes().is_empty());
    let status = ok(repo.run(&["status", "--area", "billing", "--json"])).json();
    let draft = status["areas"][0]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rule| rule["id"] == id)
        .unwrap();
    assert_eq!(draft["draft"], true);
    assert_eq!(draft["agreed"], "proposed");
}
