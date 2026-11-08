//! Swift parse errors can be linked conservatively as whole files.
mod common;

use common::{Run, TestRepo};

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

#[test]
fn existing_symbol_can_be_relinked_after_a_parse_error() {
    let repo = TestRepo::fixture("valid");
    let file = "src/example.swift";
    let target = "src/example.swift#keep";
    repo.write(file, "func keep() -> Int { 1 }\n");
    let linked = ok(repo.run(&["link", "BILL-001", target, "--json"])).json();
    assert_eq!(linked["mode"], "swift-ast-sha256-v1");
    repo.commit("Link Swift symbol");
    repo.write(file, "func keep() -> Int { 1 }\nfunc broken( {\n");
    let before = repo.read("statute.lock");

    let check = repo.run(&["check", "--all", "--json"]);
    let report = check.json();
    let finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["target"] == target)
        .unwrap();
    assert_eq!(finding["code"], "link.missing");
    assert!(
        finding["message"]
            .as_str()
            .unwrap()
            .contains("statute link --remove BILL-001")
    );
    let review = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(review.code, 2);
    assert!(review.stderr.contains("parser may not support it yet"));
    assert_eq!(repo.read("statute.lock"), before);

    ok(repo.run(&["link", "--remove", "BILL-001", target]));
    let result = ok(repo.run(&["link", "BILL-001", target, "--json"])).json();
    assert_eq!(result["target"], file);
    assert_eq!(result["requested"], target);
    assert_eq!(result["fallback"], "parse-error");
    assert_eq!(result["mode"], "bytes-sha256-v1");
    let lock = repo.read("statute.lock");
    assert!(lock.contains("bytes-sha256-v1"));
    assert!(!lock.contains(target));
    ok(repo.run(&["review", "BILL-001", "ok"]));
    ok(repo.run(&["check", "--all"]));
}

#[test]
fn fallback_reports_human_output_and_preserves_existing_link() {
    let repo = TestRepo::fixture("valid");
    let file = "src/example.swift";
    let target = "src/example.swift#keep";
    repo.write(file, "func broken( {\n");
    let result = ok(repo.run(&["link", "BILL-001", target]));
    assert!(result.stdout.contains("whole file"));
    assert!(result.stdout.contains("cannot read the file cleanly"));
    let before = repo.read("statute.lock");
    let retry = ok(repo.run(&["link", "BILL-001", target, "--test", "--json"])).json();
    assert_eq!(retry["role"], "code");
    assert_eq!(retry["target"], file);
    assert_eq!(retry["requested"], target);
    assert_eq!(retry["fallback"], "parse-error");
    assert_eq!(repo.read("statute.lock"), before);
}

#[test]
fn clean_missing_and_ambiguous_swift_symbols_are_refused() {
    let repo = TestRepo::fixture("valid");
    repo.write(
        "src/example.swift",
        "func keep() {}\nfunc keep(_ a: Int) {}\n",
    );
    let before = repo.read("statute.lock");
    for target in ["src/example.swift#absent", "src/example.swift#keep"] {
        let result = repo.run(&["link", "BILL-001", target]);
        assert_eq!(result.code, 2, "{}{}", result.stdout, result.stderr);
        assert_eq!(repo.read("statute.lock"), before);
    }
}

#[test]
fn other_language_parse_errors_are_refused() {
    let repo = TestRepo::fixture("valid");
    for (file, contents) in [
        ("src/example.go", "package p\nfunc broken( {\n"),
        ("src/example.py", "def broken(:\n"),
        ("src/example.rs", "fn broken( {\n"),
    ] {
        repo.write(file, contents);
        let before = repo.read("statute.lock");
        let result = repo.run(&["link", "BILL-001", &format!("{file}#keep")]);
        assert_eq!(result.code, 2, "{}{}", result.stdout, result.stderr);
        assert!(result.stderr.contains("does not parse cleanly"));
        assert_eq!(repo.read("statute.lock"), before);
    }
}

#[test]
fn existing_requested_symbol_is_still_refused_before_fallback() {
    let repo = TestRepo::fixture("valid");
    let file = "src/example.swift";
    let target = "src/example.swift#keep";
    repo.write(file, "func keep() {}\n");
    ok(repo.run(&["link", "BILL-001", target]));
    ok(repo.run(&["link", "BILL-001", file]));
    repo.write(file, "func broken( {\n");
    let before = repo.read("statute.lock");
    let result = repo.run(&["link", "BILL-001", target]);
    assert_eq!(result.code, 2);
    assert!(result.stderr.contains("already links"));
    assert_eq!(repo.read("statute.lock"), before);
}

#[test]
fn recovery_hint_preserves_test_role() {
    let repo = TestRepo::fixture("valid");
    let file = "src/example.swift";
    let target = "src/example.swift#keep";
    repo.write(file, "func keep() {}\n");
    ok(repo.run(&["link", "BILL-001", target, "--test"]));
    repo.commit("Link a Swift test");
    repo.write(file, "func broken( {\n");
    let review = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(review.code, 2);
    assert!(review.stderr.contains("'src/example.swift#keep' --test"));
}
