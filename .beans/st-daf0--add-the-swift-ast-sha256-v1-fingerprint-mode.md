---
# st-daf0
title: Add the swift-ast-sha256-v1 fingerprint mode
status: completed
type: task
priority: normal
created_at: 2026-11-08T10:15:10Z
updated_at: 2026-11-08T11:23:13Z
parent: st-97ld
blocked_by:
    - st-hqdq
---

## What

Add the `swift-ast-sha256-v1` fingerprint mode, so that `statute link RULE File.swift#Symbol` records a symbol link. Build it exactly as specified in st-hqdq: symbol names, what `#Type` covers, and the hash input. This task does not add the whole-file fallback. Until st-4dc0 is done, a Swift file that does not parse cleanly is refused, like Go, Python and Rust files.

## Why

See the epic. Symbol links remove the false alarms that whole-file links cause for every edit in a Swift file. Anton Solo decided (2026-11-08) to use `tree-sitter-swift` 0.7.4 as it is today and not wait for a better grammar.

## Where

Follow commit e355ce4 ("Add the rust-ast-sha256-v1 fingerprint mode"), which touched the same places:

- `Cargo.toml` and `Cargo.lock`: add `tree-sitter-swift = "=0.7.4"`, pinned exactly like the other grammars.
- `src/model/mod.rs`: new `Mode` variant.
- `src/fingerprint/mod.rs`: `mod swift;`, a match arm in `sign`, and `.swift` in `mode_for`.
- `src/fingerprint/swift.rs`: new. Use `src/fingerprint/rust.rs` and `syntax.rs` as models.
- `tests/fixtures/fingerprint/swift/`: golden fixtures.
- `tests/symbol_parse_contract.rs`: a Swift case.
- `statute.lock`: new links (see below).
- AGENTS.md: the line "whole-file and Go/Python/Rust symbol `link`" becomes "Go/Python/Rust/Swift".

## Contract

```rust
// src/model/mod.rs
pub enum Mode {
    // ...existing variants unchanged...
    #[serde(rename = "swift-ast-sha256-v1")]
    SwiftAstSha256V1,
}

// src/fingerprint/mod.rs: signatures do not change
pub fn sign(files: &BTreeMap<String, Option<Vec<u8>>>, targets: &[(Mode, &str)])
    -> Vec<Result<String, Missing>>;
pub fn mode_for(file: &str, symbol: Option<&str>) -> Option<Mode>; // Some("swift") => SwiftAstSha256V1

// src/fingerprint/swift.rs: same shape as go.rs, python.rs and rust.rs
pub fn sign(bytes: &[u8], names: &[&str]) -> Vec<Result<String, Missing>>;
```

- The parser is created once per thread and only when needed, like the other modes (docs/Architecture.md, section 6).
- A file whose tree has any error or missing node gives `Err(Missing::ParseError)` for every symbol in it (LINKS-001).
- A name that matches no declaration gives `NoSymbol`. A name that matches more than one gives `Ambiguous`.

## Grammar checks (record the results, they do not block)

- `tree-sitter-swift` 0.7.4 builds with `tree-sitter = "=0.27.1"` and `rust-version = 1.99.1`, with no network access at build time (the crate must ship its generated `parser.c`).
- `parser.set_language(&tree_sitter_swift::LANGUAGE.into())` succeeds at runtime, and a sample file parses without errors. The dependency on `tree-sitter-language` 0.1 alone does not prove that the parser's ABI version loads. Test it.
- Parse a set of real, newer Swift code and list the constructs that fail. Include: macros (`#stringify(...)`, `@Observable`), `consume`/`borrowing`/`consuming`, parameter packs (`each T`), typed throws (`throws(MyError)`), `~Copyable`, `sending`, `nonisolated(unsafe)`, regex literals (`/a+/`), raw strings (`#"..."#`), `if`/`switch` expressions, `#if` inside a member list. Add each failing construct as a `parse_error_*.swift` fixture that expects `ParseError`. Add a "Known gaps" note to the Swift item in docs/Architecture.md, section 6.1.
- Record the release binary size before and after, and put the numbers in the PR description.

## Golden fixtures

Follow the Go, Python and Rust fixtures (docs/Architecture.md, sections 6.1 and 10), with a `golden.txt` that pins the hashes:

- Same hash: comments and doc comments added; reformatting; a change to another declaration; the declaration moved to another place in the file.
- Different hash: a body change; a signature or argument-label change; an attribute or modifier change; the line-break case from st-hqdq, if it found one.
- Names: an overload picked by its labels; members added by an extension; a nested type; a protocol with a default implementation; an `Ambiguous` case.
- `parse_error.swift`: a real syntax error.

## Statute links

- Link `src/fingerprint/swift.rs#sign` as code to LINKS-001 and LINKS-004, as `go.rs#sign`, `python.rs#sign` and `rust.rs#sign` are linked now (see `statute show LINKS-001 LINKS-004`).
- `tests/symbol_parse_contract.rs` already backs LINKS-001. Add a Swift call to `syntax_error_outside_the_symbol`.

## Acceptance criteria

- [ ] `statute link RULE File.swift#Type.method --json` prints `"mode": "swift-ast-sha256-v1"` for a clean file.
- [ ] `statute link RULE Broken.swift#keep` still stops with `the file does not parse cleanly` (the fallback is st-4dc0).
- [ ] Golden fixtures cover every case listed above, and `cargo test` passes.
- [ ] Go, Python, Rust and bytes golden hashes are unchanged.
- [ ] Grammar check results and known gaps are recorded in Architecture 6.1 and the PR description.
- [ ] `tests/layering.rs` passes. `fingerprint` imports only `model` (the `Mode` type).
- [ ] The `change` skill has run, and LINKS-001 and LINKS-004 have a verdict for this change.
- [ ] AGENTS.md lists Swift symbol links.

## Dependencies

- Blocked by st-hqdq: the symbol names and hash input must be written in the spec first. You may start the grammar checks earlier.
- The decision from st-hqdq must be approved before this is merged.
