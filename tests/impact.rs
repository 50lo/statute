//! End-to-end tests for the PR "Rule impact" section, `check --md`.

mod common;

use common::{Run, TestRepo};

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

/// A clone of the valid fixture with linked code, so the base is `origin/HEAD`.
fn cloned_repo() -> TestRepo {
    let origin = TestRepo::fixture("valid");
    origin.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() int {\n\treturn 1\n}\n",
    );
    origin.write(
        "billing/refund_test.go",
        "package billing\n\nfunc TestApprove() {}\n",
    );
    origin.write("src/keep.go", "package src\n\nfunc Keep() {}\n");
    ok(origin.run(&["link", "BILL-001", "billing/refund.go#Approve"]));
    ok(origin.run(&[
        "link",
        "BILL-001",
        "billing/refund_test.go#TestApprove",
        "--test",
    ]));
    ok(origin.run(&["link", "ATTR-001", "src/keep.go#Keep"]));
    origin.commit("Initial");
    origin.clone_repo()
}

fn verdicts(repo: &TestRepo) -> Vec<(String, String)> {
    let report = repo.run(&["check", "--md", "--json"]).json();
    report["impact"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["rule"].as_str().unwrap().to_string(),
                r["verdict"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn pair(rule: &str, verdict: &str) -> (String, String) {
    (rule.to_string(), verdict.to_string())
}

/// From the appendix, "A PM adds a small feature": a stale link is unreviewed until `review RULE ok`,
/// and then the rule still holds (Architecture section 10).
#[test]
fn a_reviewed_rule_holds() {
    let repo = cloned_repo();
    let run = ok(repo.run(&["check", "--md"]));
    assert_eq!(
        run.stdout,
        "## Rule impact\n\nThis change affects no rules or terms.\n"
    );

    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() int {\n\treturn 2\n}\n",
    );
    assert_eq!(verdicts(&repo), [pair("BILL-001", "unreviewed")]);
    ok(repo.run(&["review", "BILL-001", "ok"]));
    assert_eq!(verdicts(&repo), [pair("BILL-001", "holds")]);
}

/// Every verdict, in the order the spec gives, and the section as a PM reads it.
#[test]
fn check_md_lists_affected_rules_and_terms_with_verdicts() {
    let repo = cloned_repo();
    // "A linked function is renamed": relinked.
    repo.write("src/keep.go", "package src\n\nfunc Kept() {}\n");
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Keep", "--remove"]));
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Kept"]));
    // A new rule and term, awaiting approval.
    let rule = ok(repo.run(&["rule", "new", "billing", "Refunds are logged"]));
    let rule = rule.stdout.trim();
    let id = ok(repo.run(&[
        "decide",
        "--establishes",
        rule,
        "--terms",
        "Commit",
        "--summary",
        "Log refunds.",
        "--rationale",
        "R.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // A stale link that is not reviewed.
    repo.write(
        "billing/refund.go",
        "package billing\n\nfunc Approve() int {\n\treturn 2\n}\n",
    );
    assert_eq!(
        verdicts(&repo),
        [
            pair("ATTR-001", "relinked"),
            pair("BILL-001", "unreviewed"),
            pair(rule, "new"),
        ]
    );

    // A new open gap: broken comes before relinked.
    let gaps = repo.read("statute/gaps.yml");
    repo.write(
        "statute/gaps.yml",
        &format!("{gaps}- id: G261016-abcde\n  rule: ATTR-001\n  status: open\n  opened_at: \"2026-10-16T10:00:00Z\"\n  note: \"Kept drops Sessions.\"\n"),
    );
    let run = repo.run(&["check", "--md"]);
    insta::assert_snapshot!(run.stdout.replace(&id, "D000000-xxxxx"));
}

/// A link whose role changes, such as a code link re-added with `--test`, is relinked.
#[test]
fn a_link_role_change_is_relinked() {
    let repo = cloned_repo();
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Keep", "--remove"]));
    ok(repo.run(&["link", "ATTR-001", "src/keep.go#Keep", "--test"]));
    assert_eq!(verdicts(&repo), [pair("ATTR-001", "relinked")]);
}

/// `status` lists every pending decision, including one already proposed in the base.
#[test]
fn status_lists_pending_decisions_from_the_base() {
    let repo = cloned_repo();
    let status = ok(repo.run(&["status", "--json"])).json();
    let pending: Vec<&str> = status["impact"]["awaiting_approval"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_str().unwrap())
        .collect();
    assert_eq!(pending, ["D261014-x4p9t"]);
    let run = ok(repo.run(&["status"]));
    assert!(
        run.stdout.contains(
            "### Awaiting approval\n\n- D261014-x4p9t: Context alone never links a Session to a commit. (ATTR-002)\n"
        ),
        "{}",
        run.stdout
    );
}
