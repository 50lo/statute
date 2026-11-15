//! Automated, manual and mixed verification (bug st-8j2x, decision D261115-ywulh).
//!
//! A manual check is a standing requirement for a person. Test links never replace or complete it,
//! so `status`, `review` and the "Rule impact" section keep showing it when a rule also has test
//! links. Links and manual checks declare how a rule is verified; none of these reports claims that
//! a person did a manual check.

mod common;

use common::{Run, TestRepo};
use serde_json::{Value, json};

const RULES: &str = "statute/rules/billing.md";
const CODE: &str = "billing/refund.go";
const TEST: &str = "billing/refund_test.go";

const MANUAL: &str = "Finance compares the refund screen with the policy each release.";
const MIXED: &str = "A support lead reads the refund email and confirms its tone is calm.";
const PLANNED: &str = "A designer checks the refund button contrast.";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn rule(id: &str, title: &str, manual_check: Option<&str>) -> String {
    let manual = manual_check
        .map(|c| format!("manual_check: \"{c}\"\n"))
        .unwrap_or_default();
    format!(
        "# {id} {title}\n\nThe statement of {id}.\n\n```yaml\nagreed: proposed\nbuilt: planned\nprotects: []\nboundary: []\n{manual}```\n\n"
    )
}

/// Four rules, committed and cloned so the base is `origin/HEAD`:
/// - BILL-001 is automated only: a code link and a test link.
/// - BILL-002 is manual only: a code link and a manual check.
/// - BILL-003 is mixed: a code link, a test link and a manual check.
/// - BILL-004 is planned, with a manual check and no links.
fn repo() -> TestRepo {
    let origin = TestRepo::fixture("valid");
    origin.write(
        RULES,
        &format!(
            "---\nprefix: BILL\n---\n\n{}{}{}{}",
            rule("BILL-001", "Refunds are rounded to cents", None),
            rule(
                "BILL-002",
                "The refund screen matches the policy",
                Some(MANUAL)
            ),
            rule(
                "BILL-003",
                "Refund emails are sent and read calmly",
                Some(MIXED)
            ),
            rule(
                "BILL-004",
                "The refund button is easy to see",
                Some(PLANNED)
            ),
        ),
    );
    origin.write(CODE, &code(1));
    origin.write(
        TEST,
        "package billing\n\nfunc TestRound() {}\n\nfunc TestEmail() {}\n",
    );
    for (rule, target) in [
        ("BILL-001", "#Round"),
        ("BILL-002", "#Screen"),
        ("BILL-003", "#Email"),
    ] {
        ok(origin.run(&["link", rule, &format!("{CODE}{target}")]));
    }
    ok(origin.run(&["link", "BILL-001", &format!("{TEST}#TestRound"), "--test"]));
    ok(origin.run(&["link", "BILL-003", &format!("{TEST}#TestEmail"), "--test"]));
    origin.commit("Initial");
    origin.clone_repo()
}

/// Go code for the three built rules; `n` changes every function body.
fn code(n: u32) -> String {
    format!(
        "package billing\n\nfunc Round() int {{\n\treturn {n}\n}}\n\nfunc Screen() int {{\n\treturn {n}\n}}\n\nfunc Email() int {{\n\treturn {n}\n}}\n"
    )
}

fn status_rule(status: &Value, id: &str) -> Value {
    status["areas"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|a| a["rules"].as_array().unwrap())
        .find(|r| r["id"] == id)
        .unwrap_or_else(|| panic!("{id} not in status"))
        .clone()
}

fn impact_rule(report: &Value, id: &str) -> Value {
    report["impact"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["rule"] == id)
        .unwrap_or_else(|| panic!("{id} not in the impact"))
        .clone()
}

/// The `manual_check` values `status --json` reports, in rule order.
fn status_manual_checks(repo: &TestRepo) -> Vec<Value> {
    let status = ok(repo.run(&["status", "--json"])).json();
    ["BILL-001", "BILL-002", "BILL-003", "BILL-004"]
        .iter()
        .map(|id| status_rule(&status, id)["manual_check"].clone())
        .collect()
}

#[test]
fn automated_manual_and_mixed_rules_are_each_complete_and_built() {
    let repo = repo();
    let status = ok(repo.run(&["status", "--json"])).json();
    for id in ["BILL-001", "BILL-002", "BILL-003"] {
        let rule = status_rule(&status, id);
        assert_eq!(rule["built"], "built", "{id}");
        assert_eq!(rule["incomplete"], false, "{id}");
    }
    assert_eq!(status_rule(&status, "BILL-004")["built"], "planned");
    assert!(ok(repo.run(&["check", "--json"])).error_codes().is_empty());
}

#[test]
fn a_mixed_rule_without_a_code_link_is_incomplete_even_with_a_test_link_and_a_manual_check() {
    let repo = repo();
    ok(repo.run(&["link", "BILL-003", &format!("{CODE}#Email"), "--remove"]));
    let check = repo.run(&["check", "--json"]);
    assert_eq!(check.error_codes(), vec!["rule.incomplete"]);
    assert_eq!(check.json()["findings"][0]["rule"], "BILL-003");
}

#[test]
fn status_shows_every_manual_check_also_beside_test_links() {
    let repo = repo();
    assert_eq!(
        status_manual_checks(&repo),
        vec![json!(null), json!(MANUAL), json!(MIXED), json!(PLANNED)]
    );

    let text = ok(repo.run(&["status"])).stdout;
    let lines: Vec<&str> = text.lines().collect();
    let after = |id: &str| {
        let i = lines.iter().position(|l| l.contains(id)).unwrap();
        lines.get(i + 1).copied().unwrap_or_default()
    };
    assert!(!after("BILL-001").contains("manual check"), "{text}");
    for (id, check) in [
        ("BILL-002", MANUAL),
        ("BILL-003", MIXED),
        ("BILL-004", PLANNED),
    ] {
        assert!(
            after(id).trim_start() == format!("manual check, still required: {check}"),
            "{id}:\n{text}"
        );
    }
}

/// The mixed rule's linked test passes review, but its manual check stays required in every report.
/// Nothing in the reports turns the review into evidence that a person did the manual check.
#[test]
fn a_reviewed_mixed_rule_still_requires_its_manual_check() {
    let repo = repo();
    let before = status_manual_checks(&repo);
    repo.write(CODE, &code(2));
    for id in ["BILL-001", "BILL-002", "BILL-003"] {
        assert_eq!(repo.run(&["check", "--json"]).code, 1, "stale before {id}");
        let review = ok(repo.run(&["review", id, "ok", "--json"])).json();
        let expected = match id {
            "BILL-001" => json!(null),
            "BILL-002" => json!(MANUAL),
            _ => json!(MIXED),
        };
        assert_eq!(review["manual_check"], expected, "{id}");
    }
    assert_eq!(
        ok(repo.run(&["check", "--json"])).error_codes(),
        Vec::<String>::new()
    );
    assert_eq!(status_manual_checks(&repo), before);

    let report = ok(repo.run(&["check", "--md", "--json"])).json();
    for (id, expected) in [
        ("BILL-001", json!(null)),
        ("BILL-002", json!(MANUAL)),
        ("BILL-003", json!(MIXED)),
    ] {
        let rule = impact_rule(&report, id);
        assert_eq!(rule["verdict"], "holds", "{id}");
        assert_eq!(rule["manual_check"], expected, "{id}");
    }

    let md = ok(repo.run(&["check", "--md"])).stdout;
    let section = md
        .split("### Manual checks\n")
        .nth(1)
        .unwrap_or_else(|| panic!("no manual checks section:\n{md}"));
    assert!(section.contains(&format!(
        "- **BILL-002** The refund screen matches the policy: {MANUAL}"
    )));
    assert!(section.contains(&format!(
        "- **BILL-003** Refund emails are sent and read calmly: {MIXED}"
    )));
    assert!(!section.contains("BILL-001"), "{md}");
    // BILL-004 is not affected, so the PR section does not list it; `status` does.
    assert!(!section.contains("BILL-004"), "{md}");
}

#[test]
fn review_text_says_the_manual_check_is_still_required() {
    let repo = repo();
    repo.write(CODE, &code(3));
    let mixed = ok(repo.run(&["review", "BILL-003", "ok"])).stdout;
    assert_eq!(
        mixed,
        format!(
            "Refreshed 1 link of BILL-003:\n  {CODE}#Email\nBILL-003 also has a manual check. This verdict does not record it; a person must still do it:\n  {MIXED}\n"
        )
    );
    let automated = ok(repo.run(&["review", "BILL-001", "ok"])).stdout;
    assert_eq!(
        automated,
        format!("Refreshed 1 link of BILL-001:\n  {CODE}#Round\n")
    );
    // A review with nothing to refresh still names the manual check.
    let again = ok(repo.run(&["review", "BILL-003", "ok"])).stdout;
    assert!(again.starts_with("BILL-003 has no stale links; nothing to refresh\n"));
    assert!(again.contains(MIXED), "{again}");
}

#[test]
fn a_rule_impact_with_only_automated_rules_has_no_manual_checks_section() {
    let repo = repo();
    // Only `Round` changes, so only BILL-001 is affected.
    let only_round = code(1).replacen("return 1", "return 4", 1);
    repo.write(CODE, &only_round);
    ok(repo.run(&["review", "BILL-001", "ok"]));
    let md = ok(repo.run(&["check", "--md"])).stdout;
    assert!(md.contains("**BILL-001**"), "{md}");
    assert!(!md.contains("Manual checks"), "{md}");
}
