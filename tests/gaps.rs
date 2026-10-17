//! End-to-end tests for `gap open`, `gap close` and the derived broken state.

mod common;

use common::{Run, TestRepo};

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn refused(run: Run, message: &str) {
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    assert!(run.stderr.contains(message), "{}", run.stderr);
}

/// The valid fixture with ATTR-001 linked to code and a test, so it is built. Its tracker gap
/// FOOT-412 is open.
fn built_repo() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write("src/keep.go", "package src\n\nfunc Keep() {}\n");
    repo.write("src/keep_test.go", "package src\n\nfunc TestKeep() {}\n");
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Keep"]));
    ok(repo.run(&["link", "ATTR-001", "src/keep_test.go#TestKeep", "--test"]));
    repo
}

fn attr_001(repo: &TestRepo) -> serde_json::Value {
    let status = ok(repo.run(&["status", "--area", "attribution", "--json"])).json();
    status["areas"][0]["rules"][0].clone()
}

/// From the appendix, "A rule is found broken": without a tracker, `gap open --note` prints the
/// gap ID, `review` takes it, and the rule shows as broken until `gap close`.
#[test]
fn a_rule_found_broken_stays_broken_until_its_gap_closes() {
    let repo = built_repo();
    let rule = attr_001(&repo);
    assert_eq!(rule["built"], "built");
    assert_eq!(rule["broken"], true);
    assert_eq!(rule["open_gaps"], serde_json::json!(["FOOT-412"]));
    ok(repo.run(&["gap", "close", "FOOT-412"]));
    assert_eq!(attr_001(&repo)["broken"], false);

    repo.write("src/keep.go", "package src\n\nfunc Keep() { drop() }\n");
    let id = ok(repo.run(&[
        "gap",
        "open",
        "ATTR-001",
        "--note",
        "Keep drops Sessions without a Repository.",
    ]))
    .stdout
    .trim()
    .to_string();
    assert!(
        id.starts_with('G') && id.len() == "G261015-x4p9t".len(),
        "{id}"
    );
    ok(repo.run(&["review", "ATTR-001", "gap", &id]));
    let rule = attr_001(&repo);
    assert_eq!(rule["broken"], true);
    assert_eq!(rule["open_gaps"], serde_json::json!([id]));
    let run = ok(repo.run(&["status"]));
    assert!(
        run.stdout.contains(&format!("(broken; open gaps: {id})")),
        "{}",
        run.stdout
    );
    assert!(
        repo.read("statute/gaps.yml")
            .contains("note: Keep drops Sessions without a Repository.")
    );
    ok(repo.run(&["check"]));

    ok(repo.run(&["gap", "close", &id]));
    assert_eq!(attr_001(&repo)["broken"], false);
    // Closing again changes nothing.
    let before = repo.read("statute/gaps.yml");
    ok(repo.run(&["gap", "close", &id]));
    assert_eq!(repo.read("statute/gaps.yml"), before);
}

/// With a tracker, the entry is only an index: ID, rule, status and when it opened.
#[test]
fn a_tracker_gap_is_an_index_entry() {
    let repo = built_repo();
    ok(repo.run(&["gap", "close", "FOOT-412"]));
    let run = ok(repo.run(&["gap", "open", "ATTR-001", "--ref", "FOOT-500", "--json"])).json();
    assert_eq!(run["id"], "FOOT-500");
    assert_eq!(run["status"], "open");
    let gaps = repo.read("statute/gaps.yml");
    let entry = &gaps[gaps.find("- id: FOOT-500").unwrap()..];
    assert!(
        entry.contains("rule: ATTR-001\n  status: open\n  opened_at: "),
        "{entry}"
    );
    assert!(!entry.contains("note"), "{entry}");
    assert_eq!(attr_001(&repo)["broken"], true);
    ok(repo.run(&["check"]));
}

/// A planned or retired rule with an open gap is not broken.
#[test]
fn only_built_rules_are_broken() {
    let repo = TestRepo::fixture("valid");
    let rule = attr_001(&repo);
    assert_eq!(rule["built"], "planned");
    assert_eq!(rule["open_gaps"], serde_json::json!(["FOOT-412"]));
    assert_eq!(rule["broken"], false);
}

#[test]
fn gap_open_and_close_refusals() {
    let repo = built_repo();
    refused(
        repo.run(&["gap", "open", "NOPE-001", "--note", "x"]),
        "no rule NOPE-001",
    );
    refused(repo.run(&["gap", "open", "ATTR-001"]), "--note");
    refused(
        repo.run(&["gap", "open", "ATTR-001", "--note", " "]),
        "needs `--note`",
    );
    refused(
        repo.run(&["gap", "open", "ATTR-001", "--ref", "FOOT-1", "--note", "x"]),
        "cannot be used with",
    );
    refused(
        repo.run(&["gap", "open", "ATTR-001", "--ref", "FOOT-412"]),
        "FOOT-412 is already in statute/gaps.yml",
    );
    refused(repo.run(&["gap", "close", "FOOT-999"]), "no gap FOOT-999");
}
