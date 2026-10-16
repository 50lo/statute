//! End-to-end tests for links: `link`, and stale and missing links in `check`.

mod common;

use common::TestRepo;

const CODE: &str = "billing/refund.go";
const TEST: &str = "billing/refund_test.go";

/// The valid fixture with refund code and its test, committed.
fn billing_repo() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(CODE, "package billing\n\nfunc Approve() {}\n");
    repo.write(TEST, "package billing\n\nfunc TestLargeRefund() {}\n");
    repo.commit("Initial");
    repo
}

/// `billing_repo` with BILL-001 linked to its code and test, committed.
fn linked_repo() -> TestRepo {
    let repo = billing_repo();
    assert_eq!(repo.run(&["link", "BILL-001", CODE]).code, 0);
    assert_eq!(repo.run(&["link", "BILL-001", TEST, "--test"]).code, 0);
    repo.commit("Link BILL-001");
    repo
}

fn ok(run: common::Run) -> common::Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// From the appendix, "A PM adds a feature that needs a new rule": the second link completes the
/// rule, so `link` sets `built: built`. (`decide` and `ratify` arrive with milestone 3.)
#[test]
fn the_link_that_completes_a_rule_sets_built() {
    let repo = billing_repo();
    let rules = "statute/rules/billing.md";
    let before = repo.read(rules);

    let code = ok(repo.run(&["link", "BILL-001", CODE, "--json"])).json();
    assert_eq!(
        (&code["role"], &code["built"]),
        (&"code".into(), &false.into())
    );
    assert_eq!(code["mode"], "bytes-sha256-v1");
    assert_eq!(repo.read(rules), before);

    let test = ok(repo.run(&["link", "BILL-001", TEST, "--test"]));
    assert_eq!(
        test.stdout,
        "Linked BILL-001 to billing/refund_test.go (test)\nBILL-001 is complete and now `built: built`\n"
    );
    // Only the `built:` line changed.
    assert_eq!(
        repo.read(rules),
        before.replace("built: planned", "built: built")
    );
    insta::assert_snapshot!(repo.read("statute.lock"));

    let check = repo.run(&["check", "--json"]);
    assert_eq!((check.code, check.codes()), (0, vec![]));
}

#[test]
fn a_manual_check_completes_a_rule_without_a_test_link() {
    let repo = billing_repo();
    let rules = "statute/rules/billing.md";
    let text = repo.read(rules).replace(
        "boundary: []\n",
        "boundary: []\nmanual_check: \"Finance reviews refunds monthly.\"\n",
    );
    repo.write(rules, &text);
    let run = ok(repo.run(&["link", "BILL-001", CODE, "--json"]));
    assert_eq!(run.json()["built"], true);
    assert!(repo.read(rules).contains("built: built"));
}

#[test]
fn check_reports_a_link_whose_target_changed_since_the_base() {
    let repo = linked_repo();
    let clean = repo.run(&["check", "--json"]);
    assert_eq!((clean.code, clean.codes()), (0, vec![]));

    // An uncommitted edit makes the code link stale; the test link is untouched.
    repo.write(CODE, "package billing\n\nfunc Approve() { pay() }\n");
    let run = repo.run(&["check", "--json"]);
    assert_eq!((run.code, run.codes()), (1, vec!["link.stale".to_string()]));
    let finding = &run.json()["findings"][0];
    assert_eq!(
        (&finding["rule"], &finding["target"], &finding["file"]),
        (&"BILL-001".into(), &CODE.into(), &CODE.into())
    );

    // Without a remote the base is HEAD, so once the edit is committed only `--all` sees it.
    repo.commit("Pay on approval");
    assert_eq!(repo.run(&["check"]).code, 0);
    let all = repo.run(&["check", "--all", "--json"]);
    assert_eq!((all.code, all.codes()), (1, vec!["link.stale".to_string()]));
}

#[test]
fn the_base_is_the_merge_base_with_the_default_branch() {
    let origin = linked_repo();
    let clone = origin.clone_repo();
    clone.git(&["checkout", "--quiet", "-b", "feature"]);
    clone.write(CODE, "package billing\n\nfunc Approve() { pay() }\n");
    clone.commit("Pay on approval");

    // The commit is on the branch, after the merge-base with origin/HEAD.
    let run = clone.run(&["check", "--json"]);
    assert_eq!((run.code, run.codes()), (1, vec!["link.stale".to_string()]));
}

/// From the appendix, "A linked function is renamed", at file level: the old target is missing until
/// the agent removes the link and links the new location.
#[test]
fn a_moved_target_is_missing_until_it_is_relinked() {
    let repo = linked_repo();
    let moved = "billing/approve.go";
    repo.git(&["mv", CODE, moved]);

    let run = repo.run(&["check", "--json"]);
    assert_eq!(
        (run.code, run.codes()),
        (1, vec!["link.missing".to_string()])
    );
    assert_eq!(
        run.json()["findings"][0]["message"],
        "billing/refund.go is missing: the file does not exist"
    );

    let removed = ok(repo.run(&["link", "BILL-001", CODE, "--remove", "--json"])).json();
    assert_eq!(
        (&removed["removed"], &removed["role"]),
        (&true.into(), &"code".into())
    );
    // Between the two commands the rule is built but has no code link.
    let between = repo.run(&["check", "--json"]);
    assert_eq!(
        (between.code, between.codes()),
        (1, vec!["rule.incomplete".to_string()])
    );

    ok(repo.run(&["link", "BILL-001", moved]));
    let after = repo.run(&["check", "--json"]);
    assert_eq!((after.code, after.codes()), (0, vec![]));
}

#[test]
fn link_refuses_what_the_spec_lists_and_bad_targets() {
    let repo = linked_repo();
    repo.write("billing/notes", "");
    let lock = repo.read("statute.lock");
    let cases: &[(&[&str], &str)] = &[
        (&["link", "BILL-404", CODE], "no rule BILL-404"),
        (
            &["link", "BILL-001", CODE, "--test"],
            "already links billing/refund.go",
        ),
        (&["link", "BILL-001", "billing"], "never a directory"),
        (
            &["link", "BILL-001", "billing/nope.go"],
            "billing/nope.go does not exist",
        ),
        (
            &["link", "BILL-001", "../outside.go"],
            "not a repo-relative file path",
        ),
        (
            &["link", "BILL-001", "/etc/hosts"],
            "not a repo-relative file path",
        ),
        (
            &["link", "BILL-001", "./billing/notes"],
            "not a repo-relative file path",
        ),
        (&["link", "BILL-001", "billing/notes#"], "empty symbol"),
        (
            &["link", "BILL-001", "billing/notes#X"],
            "no fingerprint mode reads symbols",
        ),
        (
            &["link", "BILL-001", "billing/nope.go", "--remove"],
            "has no link to billing/nope.go",
        ),
    ];
    for (args, message) in cases {
        let run = repo.run(args);
        assert_eq!(run.code, 2, "{args:?}");
        assert!(run.stderr.contains(message), "{args:?}: {}", run.stderr);
    }
    assert_eq!(repo.read("statute.lock"), lock);

    repo.write("statute.lock", "not json\n");
    let run = repo.run(&["link", "BILL-001", "billing/notes"]);
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("statute.lock does not parse"),
        "{}",
        run.stderr
    );
    let check = repo.run(&["check", "--json"]);
    assert_eq!(
        (check.code, check.codes()),
        (1, vec!["link.parse".to_string()])
    );
}

#[test]
fn a_link_to_a_rule_that_no_longer_exists_can_be_removed() {
    let repo = linked_repo();
    let lock = repo.read("statute.lock").replace("BILL-001", "BILL-404");
    repo.write("statute.lock", &lock);
    assert_eq!(
        repo.run(&["check", "--json"]).codes()[..2],
        ["rule.incomplete", "link.rule_unknown"]
    );
    ok(repo.run(&["link", "BILL-404", CODE, "--remove"]));
    ok(repo.run(&["link", "BILL-404", TEST, "--remove"]));
    assert_eq!(repo.read("statute.lock"), "{\"version\": 1}\n");
}

#[test]
fn check_needs_git() {
    let repo = TestRepo::fixture("valid");
    std::fs::remove_dir_all(repo.path(".git")).unwrap();
    let run = repo.run(&["check"]);
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("not a git repository"),
        "{}",
        run.stderr
    );
    // `--all` fingerprints every link and needs no base.
    assert_eq!(repo.run(&["check", "--all"]).code, 0);
}
