//! DEC rules: one proposed decision per rule or term, and `ratify` activates exactly the named
//! decisions or nothing.

mod common;

use common::{Run, TestRepo};

const ATTRIBUTION: &str = "statute/rules/attribution.md";
const BILLING: &str = "statute/rules/billing.md";
const DECISIONS: &str = "statute/decisions.db";
const TERMS: &str = "statute/terms.md";
/// The fixture's proposed decision; it establishes ATTR-002.
const PROPOSED: &str = "D261014-x4p9t";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// Exit code 2 with the message in the text error or in the `--json` error.
fn refused(run: Run, message: &str) {
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stderr.contains(message) || run.stdout.contains(message),
        "{}{}",
        run.stdout,
        run.stderr
    );
}

fn base() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.commit("Base");
    repo
}

fn decide(repo: &TestRepo, args: &[&str]) -> Run {
    let mut all = vec!["decide", "--summary", "S.", "--rationale", "R.", "--json"];
    all.extend_from_slice(args);
    repo.run(&all)
}

fn new_decision(repo: &TestRepo, args: &[&str]) -> String {
    ok(decide(repo, args)).json()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn shown(repo: &TestRepo, name: &str) -> serde_json::Value {
    ok(repo.run(&["show", name, "--json"])).json()["items"][0].clone()
}

#[test]
fn dec_001_a_rule_or_term_is_in_at_most_one_proposed_decision() {
    let repo = base();
    refused(
        decide(&repo, &["--changes", "ATTR-002"]),
        "ATTR-002 is already in proposed decision D261014-x4p9t",
    );

    let first = new_decision(&repo, &["--terms", "Commit"]);
    refused(
        decide(&repo, &["--terms", "Commit"]),
        &format!("term `Commit` is already in proposed decision {first}"),
    );

    // Two branches can each propose the same rule; the merge result is reported.
    let decisions = repo.read(DECISIONS);
    repo.write(
        DECISIONS,
        &format!(
            "{decisions}- id: D261015-m3rg3
  timestamp: \"2026-10-15T09:00:00Z\"
  actor: agent
  status: proposed
  ratified_by: null
  summary: \"Another proposal for ATTR-002.\"
  rationale: \"R.\"
  changes: [ATTR-002]
"
        ),
    );
    let check = repo.run(&["check", "--json"]);
    assert!(
        check
            .error_codes()
            .contains(&"decision.overlap".to_string()),
        "{:?}",
        check.codes()
    );
}

#[test]
fn dec_001_a_rule_in_an_active_decision_can_be_proposed_again() {
    let repo = base();
    // D261001-a2b3c, which is active, establishes ATTR-001.
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    new_decision(&repo, &["--changes", "ATTR-001"]);
    let check = repo.run(&["check", "--json"]);
    assert_eq!((check.code, check.codes()), (0, vec![]));
}

#[test]
fn dec_002_ratify_activates_only_the_named_decisions() {
    let repo = base();
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    let other = new_decision(&repo, &["--changes", "ATTR-001"]);

    ok(repo.run(&["ratify", PROPOSED]));
    assert_eq!(shown(&repo, PROPOSED)["status"], "active");
    assert_eq!(shown(&repo, "ATTR-002")["agreed"], "agreed");
    assert_eq!(shown(&repo, &other)["status"], "proposed");
    assert_eq!(shown(&repo, "ATTR-001")["agreed"], "proposed");
}

#[test]
fn dec_002_ratify_can_activate_several_named_decisions_at_once() {
    let repo = base();
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("and stores it like any other Session.", "for 30 days."),
    );
    let other = new_decision(&repo, &["--changes", "ATTR-001"]);

    ok(repo.run(&["ratify", PROPOSED, &other]));
    for id in [PROPOSED, other.as_str()] {
        assert_eq!(shown(&repo, id)["status"], "active");
    }
    for id in ["ATTR-001", "ATTR-002"] {
        assert_eq!(shown(&repo, id)["agreed"], "agreed");
    }
}

#[test]
fn dec_003_ratify_writes_nothing_when_its_items_would_have_errors() {
    let repo = base();
    // BILL-001 has no protected or boundary case.
    let without_cases = new_decision(&repo, &["--establishes", "BILL-001"]);
    let (decisions, billing) = (repo.read(DECISIONS), repo.read(BILLING));
    refused(
        repo.run(&["ratify", &without_cases]),
        "nothing was ratified",
    );
    assert_eq!(repo.read(DECISIONS), decisions);
    assert_eq!(repo.read(BILLING), billing);

    // ATTR-002 uses Commit, which a separate proposed decision redefines.
    let terms = repo.read(TERMS);
    repo.write(
        TERMS,
        &terms.replace(
            "A git commit in a repository the collector watches.",
            "A git commit in any repository.",
        ),
    );
    new_decision(&repo, &["--terms", "Commit"]);
    let decisions = repo.read(DECISIONS);
    refused(
        repo.run(&["ratify", PROPOSED]),
        "ATTR-002 uses term `Commit`, which is not agreed",
    );
    assert_eq!(repo.read(DECISIONS), decisions);
    assert_eq!(shown(&repo, "ATTR-002")["agreed"], "proposed");
}

#[test]
fn dec_003_errors_about_other_items_do_not_block_ratify() {
    let repo = TestRepo::fixture("valid");
    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() {}\n",
    );
    ok(repo.run(&["link", "BILL-001", "billing/refund.go"]));
    repo.commit("Base");
    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() { panic(1) }\n",
    );
    assert_eq!(repo.run(&["check"]).code, 1, "BILL-001's link is stale");

    ok(repo.run(&["ratify", PROPOSED]));
    assert_eq!(shown(&repo, "ATTR-002")["agreed"], "agreed");
}

#[test]
fn dec_003_stale_and_missing_links_of_the_ratified_rules_do_not_block_ratify() {
    let repo = TestRepo::fixture("valid");
    repo.write(
        "attribution/collector.go",
        "package attribution\n\nfunc Keep() bool { return true }\n\nfunc Link() bool { return false }\n",
    );
    ok(repo.run(&["link", "ATTR-002", "attribution/collector.go#Link"]));
    ok(repo.run(&["link", "ATTR-002", "attribution/collector.go"]));
    ok(repo.run(&["link", "ATTR-001", "attribution/collector.go#Keep"]));
    repo.commit("Base");

    // The rule is approved while its code is mid-change: one link stale, one missing.
    repo.write(
        "attribution/collector.go",
        "package attribution\n\nfunc Keep() bool { return true }\n\nfunc Contribute() bool { return false }\n",
    );
    assert_eq!(
        repo.run(&["check", "--json"]).error_codes(),
        ["link.stale", "link.missing"]
    );
    ok(repo.run(&["ratify", PROPOSED]));
    assert_eq!(shown(&repo, "ATTR-002")["agreed"], "agreed");

    // Retiring a rule whose code was deleted: its links are missing, and ratify removes them.
    repo.remove("attribution/collector.go");
    let retire = new_decision(&repo, &["--retires", "ATTR-001"]);
    ok(repo.run(&["ratify", &retire]));
    let retired = shown(&repo, "ATTR-001");
    assert_eq!(retired["agreed"], "retired");
    assert!(!repo.read("statute.lock").contains("\"ATTR-001\""));
}
