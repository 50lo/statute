//! GAPS-001: a built rule with an open gap is broken, and only while the gap is open.

mod common;

use common::{Run, TestRepo};

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// The `status --json` entry for a rule.
fn status(repo: &TestRepo, id: &str) -> serde_json::Value {
    let report = ok(repo.run(&["status", "--json"])).json();
    report["areas"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|a| a["rules"].as_array().unwrap().clone())
        .find(|r| r["id"] == id)
        .unwrap()
}

/// The valid fixture with BILL-001 built from a code link and a test link, committed.
fn built() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() {}\n",
    );
    repo.write(
        "billing/refund_test.go",
        "package billing\n\nfunc TestApprove() {}\n",
    );
    ok(repo.run(&["link", "BILL-001", "billing/refund.go#Approve"]));
    ok(repo.run(&["link", "BILL-001", "billing/refund_test.go", "--test"]));
    repo.commit("Build BILL-001");
    assert_eq!(status(&repo, "BILL-001")["built"], "built");
    repo
}

fn open_gap(repo: &TestRepo, rule: &str) -> String {
    ok(repo.run(&[
        "gap",
        "open",
        rule,
        "--note",
        "The rule does not hold.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn gaps_001_a_built_rule_with_an_open_gap_is_broken() {
    let repo = built();
    assert_eq!(status(&repo, "BILL-001")["broken"], false);

    let gap = open_gap(&repo, "BILL-001");
    let rule = status(&repo, "BILL-001");
    assert_eq!(rule["broken"], true);
    assert_eq!(rule["open_gaps"], serde_json::json!([gap]));
    // Being broken changes no stored field.
    assert_eq!(rule["built"], "built");
}

#[test]
fn gaps_001_a_closed_gap_or_a_planned_rule_is_not_broken() {
    let repo = built();
    let gap = open_gap(&repo, "BILL-001");
    ok(repo.run(&["gap", "close", &gap]));
    assert_eq!(status(&repo, "BILL-001")["broken"], false);

    // ATTR-001 is planned, so its open gaps do not make it broken.
    open_gap(&repo, "ATTR-001");
    let planned = status(&repo, "ATTR-001");
    assert_eq!(planned["built"], "planned");
    assert_eq!(planned["broken"], false);
}
