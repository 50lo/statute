//! LINKS-002 and LINKS-003: a changed target makes its link stale, and only a review verdict
//! refreshes it.

mod common;

use common::{Run, TestRepo};

const CODE: &str = "billing/refund.go";
const TEST: &str = "billing/refund_test.go";
const LOCK: &str = "statute.lock";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// The valid fixture with BILL-001 linked to the `Approve` symbol and the whole test file, committed.
fn linked() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(
        CODE,
        "package billing\n\nfunc Approve(amount int) bool {\n\treturn amount <= 500\n}\n",
    );
    repo.write(
        TEST,
        "package billing\n\nimport \"testing\"\n\nfunc TestLargeRefund(t *testing.T) {\n\tif Approve(501) {\n\t\tt.Fatal(\"approved\")\n\t}\n}\n",
    );
    ok(repo.run(&["link", "BILL-001", &format!("{CODE}#Approve")]));
    ok(repo.run(&["link", "BILL-001", TEST, "--test"]));
    repo.commit("Link BILL-001");
    repo
}

/// The targets of the stale links `check` reports.
fn stale(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "link.stale")
        .map(|f| f["target"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn links_002_a_changed_code_or_test_target_makes_its_link_stale() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\nfunc Approve(amount int) bool {\n\treturn amount <= 5000\n}\n",
    );
    assert_eq!(stale(&repo), [format!("{CODE}#Approve")]);

    // Weakening the linked test is a change too.
    repo.write(
        TEST,
        "package billing\n\nimport \"testing\"\n\nfunc TestLargeRefund(t *testing.T) {}\n",
    );
    assert_eq!(stale(&repo), [format!("{CODE}#Approve"), TEST.to_string()]);
    assert_eq!(repo.run(&["check"]).code, 1);
}

#[test]
fn links_002_comments_and_formatting_leave_a_symbol_link_fresh() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\n// Approve allows refunds up to the limit.\nfunc Approve(amount int) bool {\n\treturn amount <=\n\t\t500 // the limit\n}\n",
    );
    assert_eq!(stale(&repo), Vec::<String>::new());
    assert_eq!(repo.run(&["check"]).code, 0);
}

#[test]
fn links_003_a_stale_link_is_not_refreshed_without_a_valid_verdict() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\nfunc Approve(amount int) bool {\n\treturn true\n}\n",
    );
    let lock = repo.read(LOCK);

    let relink = repo.run(&["link", "BILL-001", &format!("{CODE}#Approve")]);
    assert_eq!(relink.code, 2, "{}", relink.stdout);

    // D261010-k7m2q retires ATTR-003 and does not list BILL-001.
    let changed = repo.run(&["review", "BILL-001", "changed", "D261010-k7m2q"]);
    assert_eq!(changed.code, 2, "{}", changed.stdout);
    assert!(
        changed
            .stderr
            .contains("D261010-k7m2q does not list BILL-001"),
        "{}",
        changed.stderr
    );

    let gap = repo.run(&["review", "BILL-001", "gap", "G261017-abcde"]);
    assert_eq!(gap.code, 2, "{}", gap.stdout);
    assert!(
        gap.stderr
            .contains("BILL-001 has no open gap G261017-abcde"),
        "{}",
        gap.stderr
    );

    assert_eq!(repo.read(LOCK), lock);
    assert_eq!(stale(&repo), [format!("{CODE}#Approve")]);
}

#[test]
fn links_003_one_valid_verdict_refreshes_every_stale_link_of_the_rule() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\nfunc Approve(amount int) bool {\n\treturn amount < 501\n}\n",
    );
    repo.write(
        TEST,
        "package billing\n\nimport \"testing\"\n\nfunc TestLargeRefund(t *testing.T) {\n\tif Approve(600) {\n\t\tt.Fatal(\"approved\")\n\t}\n}\n",
    );
    let report = ok(repo.run(&["review", "BILL-001", "ok", "--json"])).json();
    assert_eq!(
        report["refreshed"],
        serde_json::json!([format!("{CODE}#Approve"), TEST])
    );
    assert_eq!(stale(&repo), Vec::<String>::new());

    // A gap verdict for an open gap of the rule is valid too.
    repo.write(
        CODE,
        "package billing\n\nfunc Approve(amount int) bool {\n\treturn true\n}\n",
    );
    let gap = ok(repo.run(&[
        "gap",
        "open",
        "BILL-001",
        "--note",
        "Approve allows every refund.",
        "--json",
    ]))
    .json();
    let id = gap["id"].as_str().unwrap();
    ok(repo.run(&["review", "BILL-001", "gap", id]));
    assert_eq!(stale(&repo), Vec::<String>::new());
}
