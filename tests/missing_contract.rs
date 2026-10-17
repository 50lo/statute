//! LINKS-004: a link whose target is gone or ambiguous is missing until it is fixed or relinked.

mod common;

use common::{Run, TestRepo};

const CODE: &str = "billing/refund.go";
const TARGET: &str = "billing/refund.go#Approve";
const LOCK: &str = "statute.lock";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// The valid fixture with BILL-001 linked to the `Approve` symbol, committed.
fn linked() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(
        CODE,
        "package billing\n\nfunc Approve() bool { return false }\n",
    );
    ok(repo.run(&["link", "BILL-001", TARGET]));
    repo.commit("Link BILL-001");
    repo
}

/// The messages of the missing links `check` reports.
fn missing(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "link.missing")
        .map(|f| f["message"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn links_004_a_renamed_ambiguous_or_deleted_target_is_missing() {
    let repo = linked();

    repo.write(
        CODE,
        "package billing\n\nfunc Allow() bool { return false }\n",
    );
    assert_eq!(
        missing(&repo),
        [format!("{TARGET} is missing: the symbol is not found")]
    );

    repo.write(
        CODE,
        "package billing\n\nfunc Approve() bool { return false }\n\nfunc Approve() bool { return true }\n",
    );
    assert_eq!(
        missing(&repo),
        [format!("{TARGET} is missing: more than one symbol matches")]
    );

    repo.remove(CODE);
    assert_eq!(
        missing(&repo),
        [format!("{TARGET} is missing: the file does not exist")]
    );
    assert_eq!(repo.run(&["check"]).code, 1);
}

#[test]
fn links_004_review_does_not_clear_a_missing_link() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\nfunc Allow() bool { return false }\n",
    );
    let lock = repo.read(LOCK);

    let review = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(review.code, 2, "{}", review.stdout);
    assert!(
        review
            .stderr
            .contains(&format!("BILL-001 has a missing link: {TARGET}")),
        "{}",
        review.stderr
    );
    assert_eq!(repo.read(LOCK), lock);
    assert_eq!(missing(&repo).len(), 1);
}

#[test]
fn links_004_relinking_the_new_location_clears_a_missing_link() {
    let repo = linked();
    repo.write(
        CODE,
        "package billing\n\nfunc Allow() bool { return false }\n",
    );

    ok(repo.run(&["link", "BILL-001", TARGET, "--remove"]));
    ok(repo.run(&["link", "BILL-001", "billing/refund.go#Allow"]));
    let check = repo.run(&["check", "--json"]);
    assert_eq!((check.code, check.codes()), (0, vec![]));
}
