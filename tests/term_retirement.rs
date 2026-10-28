//! Retiring a term end to end: proposing, reporting, approving and reading a retired term (spec 4.2,
//! 4.3 and 5).

mod common;

use common::{Run, TestRepo};

const ATTRIBUTION: &str = "statute/rules/attribution.md";
const DECISIONS: &str = "statute/decisions.db";
const TERMS: &str = "statute/terms.md";
/// The fixture's proposed decision; it establishes ATTR-002.
const PROPOSED: &str = "D261014-x4p9t";

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn refused(run: Run, message: &str) {
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stderr.contains(message) || run.stdout.contains(message),
        "{}{}",
        run.stdout,
        run.stderr
    );
}

/// The valid fixture with Repository, an agreed term no rule uses. Its code name appears nowhere, and
/// ATTR-002's statement uses its `avoid` word.
fn origin() -> TestRepo {
    let repo = TestRepo::fixture("valid");
    repo.write(
        TERMS,
        &format!(
            "{}\n## Repository\n\nA git repository the collector watches.\n\n```yaml\nagreed: agreed\ncode: [RepoID]\navoid: [project]\n```\n",
            repo.read(TERMS)
        ),
    );
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("and collector score never", "and project score never"),
    );
    repo
}

fn retire_repository(repo: &TestRepo) -> String {
    ok(repo.run(&[
        "decide",
        "--retires-terms",
        "Repository",
        "--summary",
        "Stop calling repositories a term.",
        "--rationale",
        "No rule uses it.",
        "--json",
    ]))
    .json()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn term(repo: &TestRepo, name: &str) -> serde_json::Value {
    ok(repo.run(&["show", name, "--json"])).json()["items"][0].clone()
}

/// The codes of the findings about Repository. The fixture's other terms have code names that appear
/// nowhere, so `check --all` warns about them too.
fn about_repository(repo: &TestRepo, args: &[&str]) -> Vec<String> {
    let mut all = vec!["check", "--json"];
    all.extend_from_slice(args);
    let report = repo.run(&all).json();
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["term"] == "Repository")
        .map(|f| f["code"].as_str().unwrap().to_string())
        .collect()
}

fn errors(repo: &TestRepo) -> Vec<String> {
    repo.run(&["check", "--json"]).error_codes()
}

fn impact(repo: &TestRepo) -> serde_json::Value {
    repo.run(&["check", "--md", "--json"]).json()["impact"].clone()
}

#[test]
fn retiring_an_unused_term() {
    let repo = origin();
    repo.commit("Base");
    assert_eq!(
        about_repository(&repo, &["--all"]),
        ["rule.avoid_word", "term.name_not_found"]
    );
    let terms = repo.read(TERMS);

    // Proposed: the term stays agreed, and every view names the pending retirement.
    let id = retire_repository(&repo);
    assert_eq!(repo.read(TERMS), terms);
    let shown = term(&repo, "Repository");
    assert_eq!(shown["agreed"], "agreed");
    assert_eq!(shown["decisions"][0]["id"], id.as_str());
    assert_eq!(shown["decisions"][0]["role"], "retires");
    assert_eq!(shown["decisions"][0]["status"], "proposed");
    let text = ok(repo.run(&["show", "Repository"])).stdout;
    assert!(!text.contains("Retired:"), "{text}");
    assert!(text.contains(&format!("{id} retires (proposed)")), "{text}");

    let status = ok(repo.run(&["status", "--json"])).json();
    let pending = status["impact"]["awaiting_approval"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == id.as_str())
        .unwrap()
        .clone();
    assert_eq!(pending["retires_terms"], serde_json::json!(["Repository"]));
    assert_eq!(pending["terms"], serde_json::json!([]));
    let md = ok(repo.run(&["check", "--md"])).stdout;
    assert!(
        md.contains(&format!(
            "| Repository | decision {id} proposes retiring it; it stays agreed until then |"
        )),
        "{md}"
    );
    assert!(md.contains("(retires term Repository)"), "{md}");

    // Approved: retired, labelled, out of the current checks, and the other proposal is untouched.
    let run = ok(repo.run(&["approve", &id, "--json"]));
    assert_eq!(
        run.json()["retired_terms"],
        serde_json::json!(["Repository"])
    );
    assert_eq!(run.json()["agreed_terms"], serde_json::json!([]));
    assert_eq!(term(&repo, "Repository")["agreed"], "retired");
    assert_eq!(term(&repo, PROPOSED)["status"], "proposed");
    let text = ok(repo.run(&["show", "Repository"])).stdout;
    assert!(text.contains("agreed: retired"), "{text}");
    assert!(text.contains("Retired: kept as a record"), "{text}");
    assert_eq!(about_repository(&repo, &["--all"]), Vec::<String>::new());
    assert_eq!(errors(&repo), Vec::<String>::new());
    assert_eq!(
        impact(&repo)["terms"][0]["reasons"][0],
        serde_json::json!({"reason": "retired", "decision": id})
    );
    let md = ok(repo.run(&["check", "--md"])).stdout;
    assert!(
        md.contains(&format!("| Repository | retired by decision {id} |")),
        "{md}"
    );

    // Retries: the decision is already active, and the term cannot be named again.
    refused(repo.run(&["approve", &id]), "is already active");
    refused(
        repo.run(&[
            "decide",
            "--retires-terms",
            "Repository",
            "--summary",
            "S.",
            "--rationale",
            "R.",
        ]),
        "term `Repository` is retired",
    );
}

#[test]
fn rejecting_a_retirement_deletes_only_the_decision() {
    let repo = origin();
    repo.commit("Base");
    let (terms, decisions) = (repo.read(TERMS), repo.read(DECISIONS));
    retire_repository(&repo);
    assert_eq!(repo.read(TERMS), terms);

    repo.write(DECISIONS, &decisions);
    assert_eq!(repo.read(TERMS), terms);
    assert_eq!(errors(&repo), Vec::<String>::new());
    assert_eq!(term(&repo, "Repository")["agreed"], "agreed");
}

#[test]
fn a_retirement_proposed_on_the_default_branch_is_approved_later() {
    let origin = origin();
    let id = retire_repository(&origin);
    origin.commit("Propose retiring Repository");
    let repo = origin.clone_repo();
    assert_eq!(errors(&repo), Vec::<String>::new());
    let status = ok(repo.run(&["status", "--json"])).json();
    assert!(
        status["impact"]["awaiting_approval"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["id"] == id.as_str() && d["retires_terms"][0] == "Repository"),
        "{status}"
    );

    ok(repo.run(&["approve", &id]));
    assert_eq!(term(&repo, "Repository")["agreed"], "retired");
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn a_rule_that_keeps_a_retired_term_shows_it_as_retired() {
    let repo = origin();
    let rules = repo.read(ATTRIBUTION);
    let (before, retired) = rules.split_once("# ATTR-003").unwrap();
    repo.write(
        ATTRIBUTION,
        &format!(
            "{before}# ATTR-003{}",
            retired.replace("terms: [Session]", "terms: [Session, Repository]")
        ),
    );
    repo.commit("Base");
    let id = retire_repository(&repo);
    ok(repo.run(&["approve", &id]));

    let rule = term(&repo, "ATTR-003");
    assert_eq!(rule["terms"][1]["name"], "Repository");
    assert_eq!(rule["terms"][1]["agreed"], "retired");
    let text = ok(repo.run(&["show", "ATTR-003"])).stdout;
    assert!(
        text.contains("  Repository (retired): A git repository the collector watches."),
        "{text}"
    );
    assert!(text.contains("  Session: One conversation"), "{text}");
}

#[test]
fn decide_edit_records_retired_terms_and_keeps_a_draft_it_cannot_record() {
    let repo = origin();
    let set = |names: &str| {
        format!(
            r#"f() {{ sed 's/^retires_terms: \[\]/retires_terms: [{names}]/' "$1" > "$1.new" && mv "$1.new" "$1"; }}; f"#
        )
    };
    let before = repo.read(DECISIONS);
    let run = repo.run_env(
        &[("EDITOR", &set("Nothing"))],
        &[
            "decide",
            "--edit",
            "--summary",
            "Mine.",
            "--rationale",
            "R.",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.stderr.contains("no term `Nothing`"), "{}", run.stderr);
    let path = run.stderr.split("draft is kept in ").nth(1).unwrap().trim();
    assert!(std::fs::read_to_string(path).unwrap().contains("Mine."));
    std::fs::remove_file(path).unwrap();
    assert_eq!(repo.read(DECISIONS), before);

    let run = ok(repo.run_env(
        &[("EDITOR", &set("Repository"))],
        &[
            "decide",
            "--edit",
            "--summary",
            "Mine.",
            "--rationale",
            "R.",
            "--json",
        ],
    ));
    let id = run.json()["id"].as_str().unwrap().to_string();
    assert_eq!(
        term(&repo, &id)["retires_terms"],
        serde_json::json!(["Repository"])
    );
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

#[test]
fn one_call_can_retire_a_term_together_with_the_rules_that_use_it() {
    let repo = TestRepo::fixture("valid");
    repo.commit("Base");
    let retire_rule = new_decision(&repo, &["--retires", "ATTR-001"]);
    // ATTR-002 is proposed, so its terms change without a decision; PROPOSED establishes it.
    let rules = repo.read(ATTRIBUTION);
    repo.write(
        ATTRIBUTION,
        &rules.replace("terms: [Session, Commit]", "terms: [Commit]"),
    );
    let retire_term = new_decision(&repo, &["--retires-terms", "Session"]);

    ok(repo.run(&["approve", &retire_rule, &retire_term, PROPOSED]));
    assert_eq!(term(&repo, "ATTR-001")["agreed"], "retired");
    assert_eq!(term(&repo, "ATTR-002")["agreed"], "agreed");
    assert_eq!(term(&repo, "Session")["agreed"], "retired");
    // Retired ATTR-001 and ATTR-003 keep naming Session.
    assert_eq!(term(&repo, "ATTR-003")["terms"][0]["agreed"], "retired");
    assert_eq!(errors(&repo), Vec::<String>::new());
}

#[test]
fn a_term_whose_block_does_not_parse_is_not_retired() {
    let repo = TestRepo::fixture("valid");
    let block =
        "## Old approver\n\nThe second person on a refund.\n\n```yaml\nagreed: agreed\n```\n";
    repo.write(TERMS, &format!("{}\n{block}", repo.read(TERMS)));
    repo.commit("Base");
    let (terms, decisions) = (repo.read(TERMS), repo.read(DECISIONS));
    let id = new_decision(&repo, &["--retires-terms", "Old approver"]);
    let proposed = repo.read(DECISIONS);

    let broken = terms.replace(block, &block.replace("agreed: agreed", "agreed: [agreed"));
    assert_ne!(broken, terms);
    repo.write(TERMS, &broken);
    refused(
        repo.run(&["approve", &id]),
        "term `Old approver` has no readable `yaml` block",
    );
    assert_eq!(
        (repo.read(TERMS), repo.read(DECISIONS)),
        (broken.clone(), proposed.clone())
    );

    repo.write(DECISIONS, &decisions);
    refused(
        decide(&repo, &["--retires-terms", "Old approver"]),
        "term `Old approver` has no readable `yaml` block",
    );
    assert_eq!(repo.read(DECISIONS), decisions);

    // Repaired, the term retires; its name has a space.
    repo.write(TERMS, &terms);
    repo.write(DECISIONS, &proposed);
    ok(repo.run(&["approve", &id]));
    assert_eq!(term(&repo, "Old approver")["agreed"], "retired");
    assert_eq!(errors(&repo), Vec::<String>::new());
}
