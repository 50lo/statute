//! End-to-end tests for links: `link`, `review`, `show` and `status` for links, and stale and missing
//! links in `check`.

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
/// rule, so `link` sets `built: built`. (`decide` and `approve` arrive with milestone 3.)
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
    assert_eq!(
        (all.code, all.error_codes()),
        (1, vec!["link.stale".to_string()])
    );
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

/// PR #6 review, finding 1: a symlink target would be fingerprinted through the link while git reports
/// changes under the real path, so `check` missed the edit.
#[cfg(unix)]
#[test]
fn link_refuses_symbolic_links_and_check_treats_them_as_missing() {
    use std::os::unix::fs::symlink;
    let repo = billing_repo();
    symlink("refund.go", repo.path("billing/alias.go")).unwrap();
    symlink("billing", repo.path("payments")).unwrap();
    symlink("/etc/hosts", repo.path("hosts")).unwrap();
    for (target, link) in [
        ("billing/alias.go", "billing/alias.go"),
        ("payments/refund.go", "payments"),
        ("hosts", "hosts"),
    ] {
        let run = repo.run(&["link", "BILL-001", target]);
        assert_eq!(run.code, 2, "{target}");
        assert!(
            run.stderr.contains(&format!("{link} is a symbolic link")),
            "{target}: {}",
            run.stderr
        );
    }
    assert_eq!(repo.read("statute.lock"), "{\"version\": 1}\n");

    // A link to a symlink written by hand is missing, so an edit behind it is never silently fresh.
    ok(repo.run(&["link", "BILL-001", CODE]));
    let lock = repo.read("statute.lock").replace(CODE, "billing/alias.go");
    repo.write("statute.lock", &lock);
    let run = repo.run(&["check", "--all", "--json"]);
    assert_eq!(
        (run.code, run.error_codes()),
        (1, vec!["link.missing".to_string()])
    );
}

/// PR #6 review, finding 2: `link` must set `built` in any block layout `check` accepts.
#[test]
fn the_link_that_completes_a_rule_sets_built_in_any_block_layout() {
    let layouts = [
        (
            "  agreed: proposed\n  built: planned\n  protects: []\n  boundary: []\n",
            "  built: built\n",
        ),
        (
            "agreed: proposed\n\"built\": planned\nprotects: []\nboundary: []\n",
            "\"built\": built\n",
        ),
        (
            "agreed: proposed\nbuilt: >-\n  planned\nprotects: []\nboundary: []\n",
            "\nbuilt: built\nprotects",
        ),
    ];
    for (block, patched) in layouts {
        let repo = billing_repo();
        let rules = "statute/rules/billing.md";
        let original = "agreed: proposed\nbuilt: planned\nprotects: []\nboundary: []\n";
        let text = repo.read(rules).replace(original, block);
        repo.write(rules, &text);
        assert_eq!(repo.run(&["check"]).code, 0, "{block:?}");

        ok(repo.run(&["link", "BILL-001", CODE]));
        let run = repo.run(&["link", "BILL-001", TEST, "--test", "--json"]);
        assert_eq!(run.code, 0, "{block:?}: {}", run.stderr);
        assert_eq!(run.json()["built"], true);
        let after = repo.read(rules);
        assert!(after.contains(patched), "{after}");
        assert!(!after.contains("planned"), "{after}");
        let check = repo.run(&["check", "--json"]);
        assert_eq!((check.code, check.codes()), (0, vec![]), "{block:?}");
    }
}

const EDITED: &str = "package billing\n\nfunc Approve() { pay() }\n";

/// From the appendix, "A PM adds a small feature": `check` reports the stale link, and
/// `review RULE ok` refreshes it.
#[test]
fn review_ok_refreshes_the_stale_links_of_a_rule() {
    let repo = linked_repo();
    repo.write(CODE, EDITED);
    assert_eq!(repo.run(&["check"]).code, 1);

    let run = ok(repo.run(&["review", "BILL-001", "ok", "--json"])).json();
    assert_eq!(run["verdict"], "ok");
    assert_eq!(run["refreshed"], serde_json::json!([CODE]));
    let check = repo.run(&["check", "--json"]);
    assert_eq!((check.code, check.codes()), (0, vec![]));

    // Nothing stale: nothing to refresh, and the lock is unchanged.
    let lock = repo.read("statute.lock");
    let again = ok(repo.run(&["review", "BILL-001", "ok"]));
    assert_eq!(
        again.stdout,
        "BILL-001 has no stale links; nothing to refresh\n"
    );
    assert_eq!(repo.read("statute.lock"), lock);
}

/// From the appendix, "An engineer changes a function many rules depend on": each rule gets one
/// review, which refreshes all of that rule's stale links and no other rule's.
#[test]
fn one_review_per_rule_refreshes_all_of_its_links_and_only_its_own() {
    let repo = linked_repo();
    let rules = "statute/rules/billing.md";
    let second = "\n# BILL-002 Refunds are logged\n\nEvery refund is logged.\n\n```yaml\nagreed: proposed\nbuilt: planned\n```\n";
    repo.write(rules, &format!("{}{second}", repo.read(rules)));
    ok(repo.run(&["link", "BILL-002", CODE]));
    repo.commit("Link BILL-002");

    repo.write(CODE, EDITED);
    repo.write(TEST, "package billing\n\nfunc TestLargeRefund() { t() }\n");
    let check = repo.run(&["check", "--json"]);
    let stale: Vec<_> = check.json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            format!(
                "{} {}",
                f["rule"].as_str().unwrap(),
                f["target"].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(
        stale,
        [
            "BILL-001 billing/refund.go",
            "BILL-001 billing/refund_test.go",
            "BILL-002 billing/refund.go"
        ]
    );

    let run = ok(repo.run(&["review", "BILL-001", "ok", "--json"])).json();
    assert_eq!(run["refreshed"], serde_json::json!([CODE, TEST]));
    assert_eq!(
        repo.run(&["check", "--json"]).codes(),
        ["link.stale".to_string()]
    );
    ok(repo.run(&["review", "BILL-002", "ok"]));
    assert_eq!(repo.run(&["check"]).code, 0);
}

/// From the appendix, "A rule needs to change" and "A rule is found broken": the verdict must be
/// backed by a decision that changes the rule, or by an open gap on it.
#[test]
fn review_checks_changed_and_gap_verdicts_against_decisions_and_gaps() {
    let repo = linked_repo();
    let decisions = format!(
        "{}- id: D261016-b4c5d\n  timestamp: \"2026-10-16T09:00:00Z\"\n  actor: agent\n  status: proposed\n  summary: \"Raise the threshold.\"\n  rationale: \"R.\"\n  changes: [BILL-001]\n",
        repo.read("statute/decisions.db")
    );
    repo.write("statute/decisions.db", &decisions);
    let gaps = format!(
        "{}- id: FOOT-500\n  rule: BILL-001\n  status: open\n  opened_at: \"2026-10-16T09:00:00Z\"\n",
        repo.read("statute/gaps.yml")
    );
    repo.write("statute/gaps.yml", &gaps);
    repo.write(CODE, EDITED);
    let lock = repo.read("statute.lock");

    let refusals: &[(&[&str], &str)] = &[
        (&["review", "BILL-404", "ok"], "no rule BILL-404"),
        (
            &["review", "BILL-001", "changed", "D261099-zzzzz"],
            "no decision D261099-zzzzz",
        ),
        (
            &["review", "BILL-001", "changed", "D261014-x4p9t"],
            "D261014-x4p9t does not list BILL-001 in `changes` or `retires`",
        ),
        (
            &["review", "BILL-001", "gap", "FOOT-412"],
            "BILL-001 has no open gap FOOT-412",
        ),
        (
            &["review", "BILL-001", "gap", "G261015-x4p9t"],
            "no open gap G261015-x4p9t",
        ),
        (&["review", "BILL-001"], ""),
    ];
    for (args, message) in refusals {
        let run = repo.run(args);
        assert_eq!(run.code, 2, "{args:?}");
        assert!(run.stderr.contains(message), "{args:?}: {}", run.stderr);
    }
    assert_eq!(repo.read("statute.lock"), lock);

    let changed =
        ok(repo.run(&["review", "BILL-001", "changed", "D261016-b4c5d", "--json"])).json();
    assert_eq!(
        (&changed["verdict"], &changed["ref"]),
        (&"changed".into(), &"D261016-b4c5d".into())
    );
    repo.write(CODE, "package billing\n\nfunc Approve() { pay(); log() }\n");
    let gap = ok(repo.run(&["review", "BILL-001", "gap", "FOOT-500", "--json"])).json();
    assert_eq!(gap["refreshed"], serde_json::json!([CODE]));
}

#[test]
fn review_refuses_a_rule_with_a_missing_link() {
    let repo = linked_repo();
    repo.remove(CODE);
    let run = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(run.code, 2);
    assert!(
        run.stderr
            .contains("BILL-001 has a missing link: billing/refund.go (the file does not exist)"),
        "{}",
        run.stderr
    );
}

/// From the appendix, "A PM adds a feature that needs a new rule": `show` on files with no rules
/// is not an error, and `show` on a linked file or a symbol in it returns the rule.
#[test]
fn show_prints_the_rules_linked_to_a_file_or_symbol() {
    let repo = linked_repo();
    repo.write("billing/other.go", "package billing\n");

    let run = ok(repo.run(&["show", CODE, "billing/refund.go#Approve", TEST, "--json"])).json();
    let items = run["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "BILL-001");
    assert_eq!(
        items[0]["links"],
        serde_json::json!([
            {"target": CODE, "role": "code"},
            {"target": TEST, "role": "test"}
        ])
    );

    let none = ok(repo.run(&["show", "billing/other.go"]));
    assert_eq!(none.stdout, "No rules are linked to billing/other.go.\n");
    let missing = repo.run(&["show", "billing/nope.go"]);
    assert_eq!(missing.code, 2);
    assert!(
        missing
            .stderr
            .contains("no rule, term, decision or file named `billing/nope.go`")
    );
}

#[test]
fn status_shows_stale_missing_and_incomplete_rules() {
    let repo = linked_repo();
    repo.write(CODE, EDITED);
    ok(repo.run(&["link", "BILL-001", TEST, "--remove"]));
    let run = ok(repo.run(&["status", "--area", "billing"]));
    insta::assert_snapshot!(run.stdout);

    repo.remove(CODE);
    let json = ok(repo.run(&["status", "--area", "billing", "--json"])).json();
    let rule = &json["areas"][0]["rules"][0];
    assert_eq!(
        (&rule["incomplete"], &rule["stale"], &rule["missing"]),
        (
            &true.into(),
            &serde_json::json!([]),
            &serde_json::json!([CODE])
        )
    );
}

/// From the appendix, "The feature needs a new term": `check --all` warns about a term's code name
/// that appears nowhere outside Statute's files and Markdown.
#[test]
fn check_all_warns_about_term_names_that_no_longer_appear() {
    let repo = TestRepo::fixture("valid");
    repo.write(
        "collector/session.go",
        "type Session struct{ ID SessionID }\n// sessions\n",
    );
    repo.write("docs/notes.md", "A Commit is a commit.\n");
    repo.write("ignored/commit.go", "type Commit struct{}\n");
    repo.write(".gitignore", "ignored/\n");
    let run = repo.run(&["check", "--all", "--json"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.codes(), ["term.name_not_found"]);
    let finding = &run.json()["findings"][0];
    assert_eq!(
        (&finding["term"], &finding["line"]),
        (&"Commit".into(), &16.into())
    );

    // Plain `check` does not search.
    assert_eq!(repo.run(&["check", "--json"]).codes(), Vec::<String>::new());
}

const PUBLISHER: &str = "internal/sessioncommit/publisher.go";
const PUBLISHER_SRC: &str = "package sessioncommit\n\ntype Publisher struct{}\n\n// Decide decides.\nfunc (p *Publisher) Decide(e string) bool {\n\treturn e != \"\"\n}\n\nfunc init() {}\n\nfunc init() {}\n";

/// From the appendix, "A linked function is renamed": a Go symbol link ignores comments, goes stale
/// when the body changes, and is missing after a rename until it is relinked.
#[test]
fn go_symbol_links_follow_the_symbol_not_the_file() {
    let repo = billing_repo();
    repo.write(PUBLISHER, PUBLISHER_SRC);
    let target = format!("{PUBLISHER}#Publisher.Decide");
    let linked = ok(repo.run(&["link", "BILL-001", &target, "--json"])).json();
    assert_eq!(linked["mode"], "go-ast-sha256-v1");
    repo.commit("Link Decide");

    // A comment and an unrelated declaration change nothing.
    repo.write(
        PUBLISHER,
        &PUBLISHER_SRC
            .replace("// Decide decides.", "// Decide decides, reworded.")
            .replace("type Publisher struct{}", "type Publisher struct{ n int }"),
    );
    assert_eq!(repo.run(&["check", "--json"]).codes(), Vec::<String>::new());

    repo.write(PUBLISHER, &PUBLISHER_SRC.replace("e != \"\"", "len(e) > 3"));
    assert_eq!(repo.run(&["check", "--json"]).codes(), ["link.stale"]);

    repo.write(PUBLISHER, &PUBLISHER_SRC.replace("Decide(", "Choose("));
    let run = repo.run(&["check", "--json"]);
    assert_eq!(run.codes(), ["link.missing"]);
    assert!(
        run.json()["findings"][0]["message"]
            .as_str()
            .unwrap()
            .ends_with("the symbol is not found")
    );
    ok(repo.run(&["link", "BILL-001", &target, "--remove"]));
    ok(repo.run(&["link", "BILL-001", &format!("{PUBLISHER}#Publisher.Choose")]));
    assert_eq!(repo.run(&["check", "--json"]).codes(), Vec::<String>::new());
}

#[test]
fn link_refuses_an_ambiguous_unknown_or_unparseable_go_symbol() {
    let repo = billing_repo();
    repo.write(PUBLISHER, PUBLISHER_SRC);
    repo.write("internal/broken.go", "package x\n\nfunc F( {\n");
    let cases = [
        (format!("{PUBLISHER}#init"), "more than one symbol matches"),
        (format!("{PUBLISHER}#Nope"), "the symbol is not found"),
        (
            "internal/broken.go#F".to_string(),
            "the file does not parse cleanly",
        ),
    ];
    for (target, message) in cases {
        let run = repo.run(&["link", "BILL-001", &target]);
        assert_eq!(run.code, 2, "{target}");
        assert!(run.stderr.contains(message), "{target}: {}", run.stderr);
    }
    assert_eq!(repo.read("statute.lock"), "{\"version\": 1}\n");
}
