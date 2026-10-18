//! DEC rules: one proposed decision per rule or term, `ratify` activates exactly the named decisions
//! or nothing, and a term is retired only from agreed and is then named only by history.

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

fn errors(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).error_codes()
}

/// The terms, decisions and rule files, to compare before and after a refusal.
fn files(repo: &TestRepo) -> [String; 4] {
    [TERMS, DECISIONS, ATTRIBUTION, BILLING].map(|f| repo.read(f))
}

#[test]
fn dec_003_retiring_a_term_a_rule_still_uses_is_refused() {
    let repo = base();
    let retire = new_decision(&repo, &["--retires-terms", "Session"]);
    let before = files(&repo);

    // ATTR-001 is agreed and ATTR-002 proposed; neither is in the ratified decision.
    let run = repo.run(&["ratify", &retire]);
    refused(run, "nothing was ratified");
    let run = repo.run(&["ratify", &retire]);
    for rule in ["ATTR-001", "ATTR-002"] {
        assert!(
            run.stderr
                .contains(&format!("{rule} uses term `Session`, which is retired")),
            "{}",
            run.stderr
        );
    }
    assert!(!run.stderr.contains("ATTR-003"), "{}", run.stderr);
    assert_eq!(files(&repo), before);
}

#[test]
fn dec_003_retiring_a_term_succeeds_when_the_same_call_moves_its_rules_to_a_successor() {
    let repo = base();
    let terms = repo.read(TERMS);
    repo.write(
        TERMS,
        &format!(
            "{terms}\n## Coding session\n\nOne conversation between a developer and a coding agent, in one repository.\n"
        ),
    );
    let rules = repo.read(ATTRIBUTION);
    let moved = rules.replace(
        "scope: Session collection\nterms: [Session]",
        "scope: Session collection\nterms: [Coding session]",
    );
    assert_ne!(moved, rules);
    // ATTR-002 is proposed, so its terms change without a decision; PROPOSED establishes it.
    let moved = moved.replace(
        "terms: [Session, Commit]",
        "terms: [Coding session, Commit]",
    );
    repo.write(ATTRIBUTION, &moved);
    let change = new_decision(
        &repo,
        &["--changes", "ATTR-001", "--terms", "Coding session"],
    );
    let retire = new_decision(&repo, &["--retires-terms", "Session"]);
    assert_eq!(errors(&repo), Vec::<String>::new());

    // Without the decision that approves the successor, nothing is ratified.
    let before = files(&repo);
    refused(
        repo.run(&["ratify", &retire, PROPOSED]),
        "nothing was ratified",
    );
    assert_eq!(files(&repo), before);

    let run = ok(repo.run(&["ratify", &retire, &change, PROPOSED, "--json"]));
    assert_eq!(run.json()["retired_terms"], serde_json::json!(["Session"]));
    assert_eq!(shown(&repo, "Session")["agreed"], "retired");
    assert_eq!(shown(&repo, "Coding session")["agreed"], "agreed");
    assert_eq!(shown(&repo, "ATTR-001")["agreed"], "agreed");
    // ATTR-003 is retired and keeps naming Session.
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn dec_004_decide_retires_only_an_agreed_term_it_does_not_redefine() {
    let repo = base();
    repo.write(
        TERMS,
        &format!(
            "{}\n## Repository\n\nA git repository the collector watches.\n\n```yaml\nagreed: proposed\n```\n",
            repo.read(TERMS)
        ),
    );
    let before = files(&repo);
    let cases: [(&[&str], &str); 5] = [
        (
            &["--retires-terms", "Repository"],
            "term `Repository` is not agreed",
        ),
        (
            &["--terms", "Session", "--retires-terms", "Session"],
            "term `Session` is in both `terms` and `retires_terms`",
        ),
        (&["--retires-terms", "Nothing"], "no term `Nothing`"),
        (&["--retires-terms", "ATTR-001"], "no term `ATTR-001`"),
        (&["--retires", "Session"], "no rule Session"),
    ];
    for (args, message) in cases {
        refused(decide(&repo, args), message);
        assert_eq!(files(&repo), before, "{args:?}");
    }
}

#[test]
fn dec_004_a_retirement_can_be_proposed_while_rules_still_use_the_term() {
    let repo = base();
    let terms = repo.read(TERMS);
    let run = ok(decide(&repo, &["--retires-terms", "Session"]));
    assert_eq!(run.json()["retiring_terms"], serde_json::json!(["Session"]));
    assert_eq!(run.json()["proposed_terms"], serde_json::json!([]));
    assert_eq!(repo.read(TERMS), terms);
    assert_eq!(shown(&repo, "Session")["agreed"], "agreed");
    assert_eq!(errors(&repo), Vec::<String>::new());

    // A second proposal for the same term is refused, whichever list names it.
    for flag in ["--terms", "--retires-terms"] {
        refused(
            decide(&repo, &[flag, "Session"]),
            "term `Session` is already in proposed decision",
        );
    }
}

/// Two proposed decisions, as a merge could bring them in: one redefines Commit and one retires it.
/// `decide` refuses to write the second, and the first leaves Commit agreed because it was not written
/// by `decide`.
const REDEFINES_COMMIT: &str = "- id: D261015-r3d3f
  timestamp: \"2026-10-15T09:00:00Z\"
  actor: agent
  status: proposed
  ratified_by: null
  summary: \"Commits in any repository count.\"
  rationale: \"R.\"
  terms: [Commit]
";
const RETIRES_COMMIT: &str = "- id: D261015-r3t1r
  timestamp: \"2026-10-15T10:00:00Z\"
  actor: agent
  status: proposed
  ratified_by: null
  summary: \"Stop calling commits a term.\"
  rationale: \"R.\"
  retires_terms: [Commit]
";

#[test]
fn dec_004_ratify_refuses_to_redefine_and_retire_a_term_in_one_call() {
    let repo = base();
    let decisions = repo.read(DECISIONS);
    repo.write(
        DECISIONS,
        &format!("{decisions}{REDEFINES_COMMIT}{RETIRES_COMMIT}"),
    );
    assert!(
        errors(&repo).contains(&"decision.overlap".to_string()),
        "{:?}",
        errors(&repo)
    );
    let before = files(&repo);
    refused(
        repo.run(&["ratify", "D261015-r3d3f", "D261015-r3t1r"]),
        "D261015-r3t1r retires term `Commit` and D261015-r3d3f redefines it",
    );
    assert_eq!(files(&repo), before);

    // A term `decide` set to proposed is not agreed, so it cannot be retired either.
    repo.write(DECISIONS, &format!("{decisions}{RETIRES_COMMIT}"));
    repo.write(
        TERMS,
        &repo.read(TERMS).replace(
            "agreed: agreed\ncode: [Commit]",
            "agreed: proposed\ncode: [Commit]",
        ),
    );
    let before = files(&repo);
    refused(
        repo.run(&["ratify", "D261015-r3t1r"]),
        "D261015-r3t1r retires term `Commit`, which is not agreed",
    );
    assert_eq!(files(&repo), before);
}

/// The valid fixture with Repository retired by an active decision, and named by retired ATTR-003.
fn base_with_a_retired_term() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(
        TERMS,
        &format!(
            "{}\n## Repository\n\nA git repository the collector watches.\n\n```yaml\nagreed: retired\n```\n",
            repo.read(TERMS)
        ),
    );
    repo.write(
        DECISIONS,
        &format!(
            "{}- id: D261012-r3p0s
  timestamp: \"2026-10-12T09:00:00Z\"
  actor: user
  status: active
  ratified_by: {{name: \"Pat Example\", email: \"pat@example.com\", at: \"2026-10-12T09:30:00Z\"}}
  summary: \"Stop calling repositories a term.\"
  rationale: \"R.\"
  retires_terms: [Repository]
",
            repo.read(DECISIONS)
        ),
    );
    let rules = repo.read(ATTRIBUTION);
    let (before, retired) = rules.split_once("# ATTR-003").unwrap();
    repo.write(
        ATTRIBUTION,
        &format!(
            "{before}# ATTR-003{}",
            retired.replace("terms: [Session]", "terms: [Session, Repository]")
        ),
    );
    repo.commit("Base with a retired term");
    repo
}

#[test]
fn dec_005_only_retired_rules_and_active_decisions_name_a_retired_term() {
    let repo = base_with_a_retired_term();
    // Retired ATTR-003 and active D261012-r3p0s name Repository.
    assert_eq!(errors(&repo), Vec::<String>::new());

    // A proposed rule is not exempt.
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace(
            "terms: [Session, Commit]",
            "terms: [Session, Commit, Repository]",
        ),
    );
    assert_eq!(errors(&repo), ["rule.term_retired"]);
    repo.write(ATTRIBUTION, &rules);

    // A proposed decision that redefines it, as a merge could bring in.
    let decisions = repo.read(DECISIONS);
    repo.write(
        DECISIONS,
        &format!(
            "{decisions}- id: D261015-d3f1n
  timestamp: \"2026-10-15T09:00:00Z\"
  actor: agent
  status: proposed
  ratified_by: null
  summary: \"Repositories include mirrors.\"
  rationale: \"R.\"
  terms: [Repository]
"
        ),
    );
    assert_eq!(errors(&repo), ["decision.term_retired"]);
    let before = files(&repo);
    refused(
        repo.run(&["ratify", "D261015-d3f1n"]),
        "D261015-d3f1n names term `Repository`, which is retired",
    );
    assert_eq!(files(&repo), before);
}
