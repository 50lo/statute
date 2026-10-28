//! Lifecycle guarantees the CLI can enforce. Human approval itself is supplied by the caller's
//! workflow; these tests do not treat a successful `approve` command as proof of consent.

mod common;

use common::{Run, TestRepo};
use serde_json::{Value, json};

const BILLING: &str = "statute/rules/billing.md";
const TERMS: &str = "statute/terms.md";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn shown(repo: &TestRepo, name: &str) -> Value {
    ok(repo.run(&["show", name, "--json"])).json()["items"][0].clone()
}

fn new_rule(repo: &TestRepo, title: &str, term: &str) -> String {
    let rule = ok(repo.run(&["rule", "new", "billing", title]));
    let id = rule.stdout.trim().to_string();
    let rules = repo.read(BILLING).replace(
        "terms: []\nexceptions: []\nprotects: []\nboundary: []",
        &format!(
            "terms: [{term}]\nexceptions: []\nprotects: [A large refund is logged.]\nboundary: [A small refund needs no log.]"
        ),
    );
    repo.write(BILLING, &rules);
    repo.write(
        TERMS,
        &format!(
            "{}\n## {term}\n\nThe person responsible for the refund log.\n",
            repo.read(TERMS)
        ),
    );
    id
}

fn propose(repo: &TestRepo, rule: &str, term: &str) -> String {
    ok(repo.run(&[
        "decide",
        "--actor",
        "agent",
        "--establishes",
        rule,
        "--terms",
        term,
        "--summary",
        "Record the refund logging requirement.",
        "--rationale",
        "Make the intended behavior reviewable.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn existing_implementation_and_test_links_do_not_agree_an_agent_proposal() {
    let repo = TestRepo::fixture("valid");
    // Both files exist before the agent records the rule or term.
    repo.write("billing/log.go", "package billing\n\nfunc Log() {}\n");
    repo.write(
        "billing/log_test.go",
        "package billing\n\nimport \"testing\"\n\nfunc TestLog(t *testing.T) { Log() }\n",
    );
    let rule = new_rule(&repo, "Large refunds are logged", "Log owner");
    let initial = shown(&repo, &rule);
    assert_eq!(initial["agreed"], "proposed");
    assert_eq!(initial["built"], "planned");

    let decision = propose(&repo, &rule, "Log owner");
    ok(repo.run(&["link", &rule, "billing/log.go#Log"]));
    ok(repo.run(&["link", &rule, "billing/log_test.go#TestLog", "--test"]));

    let linked = shown(&repo, &rule);
    assert_eq!(linked["agreed"], "proposed");
    assert_eq!(linked["built"], "built");
    let proposal = shown(&repo, &decision);
    assert_eq!(proposal["actor"], "agent");
    assert_eq!(proposal["status"], "proposed");
    assert_eq!(proposal["approved_by"], Value::Null);
    assert!(
        shown(&repo, "Log owner")["text"]
            .as_str()
            .unwrap()
            .contains("agreed: proposed")
    );

    // Invoke approve as the caller would after obtaining human approval. The CLI cannot verify that
    // approval happened; it can verify the named decision's resulting rule and term state.
    let report = ok(repo.run(&["approve", &decision, "--json"])).json();
    assert_eq!(report["agreed_rules"], json!([rule]));
    assert_eq!(report["agreed_terms"], json!(["Log owner"]));
    assert_eq!(report["built_rules"], json!([]));
    let approved = shown(&repo, &rule);
    assert_eq!(approved["agreed"], "agreed");
    assert_eq!(approved["built"], "built");
    assert!(
        shown(&repo, "Log owner")["text"]
            .as_str()
            .unwrap()
            .contains("agreed: agreed")
    );
    assert!(
        ok(repo.run(&["check", "--all", "--json"]))
            .error_codes()
            .is_empty()
    );
}

#[test]
fn approve_updates_only_named_decisions_rules_and_terms() {
    let repo = TestRepo::fixture("valid");
    let first_rule = new_rule(&repo, "Large refunds are logged", "Log owner");
    let first = propose(&repo, &first_rule, "Log owner");
    let second_rule = new_rule(&repo, "Small refunds are logged", "Audit owner");
    let second = propose(&repo, &second_rule, "Audit owner");
    let pending_decision = shown(&repo, &second);
    let pending_rule = shown(&repo, &second_rule);
    let pending_term = shown(&repo, "Audit owner");
    let earlier_pending = shown(&repo, "D261014-x4p9t");

    let report = ok(repo.run(&["approve", &first, "--json"])).json();
    assert_eq!(report["decisions"], json!([first]));
    assert_eq!(report["agreed_rules"], json!([first_rule]));
    assert_eq!(report["agreed_terms"], json!(["Log owner"]));
    assert_eq!(report["built_rules"], json!([]));
    assert_eq!(shown(&repo, &first)["status"], "active");
    let agreed_rule = shown(&repo, &first_rule);
    assert_eq!(agreed_rule["agreed"], "agreed");
    assert_eq!(agreed_rule["built"], "planned");
    assert_eq!(shown(&repo, &second), pending_decision);
    // Both rules and both terms share files: patching the selected entries must preserve the
    // unrelated proposal's content and state, even if its line numbers move.
    let remaining_rule = shown(&repo, &second_rule);
    assert_eq!(remaining_rule["text"], pending_rule["text"]);
    assert_eq!(remaining_rule["agreed"], "proposed");
    assert_eq!(remaining_rule["built"], "planned");
    assert_eq!(shown(&repo, "Audit owner")["text"], pending_term["text"]);
    assert_eq!(shown(&repo, "D261014-x4p9t"), earlier_pending);

    // The second proposal still requires its own explicit selection in a later invocation.
    ok(repo.run(&["approve", &second]));
    assert_eq!(shown(&repo, &second)["status"], "active");
    assert_eq!(shown(&repo, &second_rule)["agreed"], "agreed");
    assert!(ok(repo.run(&["check", "--json"])).error_codes().is_empty());
}

#[test]
fn proposing_a_change_preserves_build_state_without_reusing_the_previous_approval() {
    let repo = TestRepo::fixture("valid");
    let rule = new_rule(&repo, "Large refunds are logged", "Log owner");
    let first = propose(&repo, &rule, "Log owner");
    repo.write("billing/log.go", "package billing\n\nfunc Log() {}\n");
    repo.write(
        "billing/log_test.go",
        "package billing\n\nfunc TestLog() {}\n",
    );
    ok(repo.run(&["link", &rule, "billing/log.go#Log"]));
    ok(repo.run(&["link", &rule, "billing/log_test.go#TestLog", "--test"]));
    ok(repo.run(&["approve", &first]));
    let active = shown(&repo, &first);
    let links = repo.read("statute.lock");

    let change = ok(repo.run(&[
        "decide",
        "--changes",
        &rule,
        "--terms",
        "Log owner",
        "--summary",
        "Clarify who is responsible for refund logging.",
        "--rationale",
        "The clarification needs separate approval from the original requirement.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(shown(&repo, &rule)["agreed"], "proposed");
    assert_eq!(shown(&repo, &rule)["built"], "built");
    assert!(
        shown(&repo, "Log owner")["text"]
            .as_str()
            .unwrap()
            .contains("agreed: proposed")
    );
    let pending = shown(&repo, &change);
    assert_eq!(pending["status"], "proposed");
    assert_eq!(pending["approved_by"], Value::Null);
    assert_eq!(shown(&repo, &first), active);
    assert_eq!(repo.read("statute.lock"), links);

    // Neither a clean check nor displaying status is a new approval event.
    let before = (
        repo.read(BILLING),
        repo.read(TERMS),
        repo.read("statute/decisions.db"),
    );
    ok(repo.run(&["check", "--all", "--json"]));
    ok(repo.run(&["status", "--json"]));
    assert_eq!(
        (
            repo.read(BILLING),
            repo.read(TERMS),
            repo.read("statute/decisions.db")
        ),
        before
    );
    assert_eq!(shown(&repo, &change), pending);

    ok(repo.run(&["approve", &change]));
    assert_eq!(shown(&repo, &rule)["agreed"], "agreed");
    assert_eq!(shown(&repo, &rule)["built"], "built");
    assert_eq!(shown(&repo, &first), active);
}

#[test]
fn a_refused_batch_approval_changes_no_rules_terms_decisions_or_links() {
    for unknown_decision in [false, true] {
        let repo = TestRepo::fixture("valid");
        let first_rule = new_rule(&repo, "Large refunds are logged", "Log owner");
        let first = propose(&repo, &first_rule, "Log owner");
        let second_rule = new_rule(&repo, "Small refunds are logged", "Audit owner");
        let second = propose(&repo, &second_rule, "Audit owner");
        let complete = repo.read(BILLING);
        if !unknown_decision {
            // The first selected rule is valid. A later rule fails validation only after the
            // proposed batch has been assembled, so even its tentative edits must be discarded.
            let start = complete.find(&format!("# {second_rule} ")).unwrap();
            let (head, tail) = complete.split_at(start);
            repo.write(
                BILLING,
                &format!(
                    "{head}{}",
                    tail.replace("boundary: [A small refund needs no log.]", "boundary: []")
                ),
            );
        }
        let files = [BILLING, TERMS, "statute/decisions.db", "statute.lock"];
        let before = files.map(|file| repo.read(file));
        let selected = if unknown_decision {
            "D261231-zzzzz"
        } else {
            &second
        };
        let refused = repo.run(&["approve", &first, selected]);
        assert_eq!(refused.code, 2, "{}{}", refused.stdout, refused.stderr);
        assert!(
            refused.stderr.contains(if unknown_decision {
                "no decision D261231-zzzzz"
            } else {
                "rule.cases_missing"
            }),
            "{}",
            refused.stderr
        );
        assert_eq!(files.map(|file| repo.read(file)), before);
        for id in [&first, &second] {
            assert_eq!(shown(&repo, id)["status"], "proposed");
            assert_eq!(shown(&repo, id)["approved_by"], Value::Null);
        }
        for id in [&first_rule, &second_rule] {
            assert_eq!(shown(&repo, id)["agreed"], "proposed");
            assert_eq!(shown(&repo, id)["built"], "planned");
        }

        // Repair the invalid input and retry the intended batch; neither failure should poison it.
        repo.write(BILLING, &complete);
        ok(repo.run(&["approve", &first, &second]));
        assert_eq!(shown(&repo, &first_rule)["agreed"], "agreed");
        assert_eq!(shown(&repo, &second_rule)["agreed"], "agreed");
        assert!(ok(repo.run(&["check", "--json"])).error_codes().is_empty());
    }
}
