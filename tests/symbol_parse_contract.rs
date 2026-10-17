//! A valid linked declaration does not excuse a syntax error elsewhere in its file.

mod common;

use common::{Run, TestRepo};

fn ok(run: Run) -> Run {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    run
}

fn syntax_error_outside_the_symbol(file: &str, clean: &str, broken: &str, mode: &str) {
    let repo = TestRepo::fixture("valid");
    let target = format!("{file}#keep");
    repo.write(file, clean);
    let symbol = ok(repo.run(&["link", "BILL-001", &target, "--json"])).json();
    assert_eq!(symbol["mode"], mode);
    let bytes = ok(repo.run(&["link", "BILL-001", file, "--json"])).json();
    assert_eq!(bytes["mode"], "bytes-sha256-v1");
    repo.commit("Link a valid symbol and its whole file");
    let before = repo.read("statute.lock");

    // The linked declaration is unchanged. Only an unrelated declaration is malformed.
    assert!(broken.starts_with(clean));
    repo.write(file, broken);
    let check = repo.run(&["check", "--all", "--json"]);
    assert_eq!(check.code, 1, "{}{}", check.stdout, check.stderr);
    let report = check.json();
    let findings: Vec<_> = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["severity"] == "error")
        .collect();
    assert_eq!(findings.len(), 2, "{findings:?}");
    let symbol_finding = findings.iter().find(|f| f["target"] == target).unwrap();
    assert_eq!(symbol_finding["code"], "link.missing");
    assert_eq!(
        symbol_finding["message"],
        format!("{target} is missing: the file does not parse cleanly")
    );
    let bytes_finding = findings.iter().find(|f| f["target"] == file).unwrap();
    assert_eq!(bytes_finding["code"], "link.stale");

    let review = repo.run(&["review", "BILL-001", "ok"]);
    assert_eq!(review.code, 2);
    assert!(
        review.stderr.contains(&format!(
            "BILL-001 has a missing link: {target} (the file does not parse cleanly)"
        )),
        "{}",
        review.stderr
    );
    assert_eq!(repo.read("statute.lock"), before);

    // Whole-file byte links can still be reviewed against these exact malformed contents.
    ok(repo.run(&["link", "BILL-001", &target, "--remove"]));
    let stale = repo.run(&["check", "--all", "--json"]);
    assert_eq!(
        (stale.code, stale.error_codes()),
        (1, vec!["link.stale".to_string()])
    );
    let refreshed = ok(repo.run(&["review", "BILL-001", "ok", "--json"])).json();
    assert_eq!(refreshed["refreshed"], serde_json::json!([file]));
    let fresh = repo.run(&["check", "--all", "--json"]);
    assert_eq!((fresh.code, fresh.error_codes()), (0, vec![]));
}

#[test]
fn python_symbol_is_missing_for_a_syntax_error_elsewhere_but_bytes_still_work() {
    syntax_error_outside_the_symbol(
        "src/example.py",
        "def keep():\n    return 1\n",
        "def keep():\n    return 1\n\ndef broken(:\n    pass\n",
        "python-ast-sha256-v1",
    );
}

#[test]
fn go_symbol_is_missing_for_a_syntax_error_elsewhere_but_bytes_still_work() {
    syntax_error_outside_the_symbol(
        "src/example.go",
        "package example\n\nfunc keep() int { return 1 }\n",
        "package example\n\nfunc keep() int { return 1 }\n\nfunc broken( {\n}\n",
        "go-ast-sha256-v1",
    );
}
