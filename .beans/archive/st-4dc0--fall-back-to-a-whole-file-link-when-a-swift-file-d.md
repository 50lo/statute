---
# st-4dc0
title: Fall back to a whole-file link when a Swift file does not parse cleanly
status: completed
type: task
priority: normal
created_at: 2026-11-08T10:15:10Z
updated_at: 2026-11-08T11:23:13Z
parent: st-97ld
blocked_by:
    - st-hqdq
    - st-daf0
---

## What

When `statute link RULE File.swift#Symbol` finds that the Swift file does not parse cleanly, record a whole-file link instead of refusing. The whole-file link is `File.swift` with mode `bytes-sha256-v1`. Also make `check` and `review` tell people how to move an existing Swift symbol link to a whole-file link when its file stops parsing.

Build the exact behaviour that st-hqdq wrote in docs/Statute-spec.md, section 4.4, and in the approved decision. The text below shows the recommended behaviour. If the decision differs, follow the decision.

## Why

The Swift grammar (`tree-sitter-swift` 0.7.4) does not understand every Swift construct, and Swift keeps changing. Valid Swift code can therefore fail to parse. Today `link` stops with `cannot link <target>: the file does not parse cleanly`, and the spec says "A parse error is fixed in the file". That is not possible when the code is valid and the grammar is behind. Anton Solo decided (2026-11-08) that a whole-file fallback gives a better experience than a refusal. It is also safe: a whole-file link raises more false alarms, never fewer (AGENTS.md, principle 1).

## Observed behaviour today (with st-daf0 done)

1. A Swift file uses a construct the grammar cannot parse (one of the `parse_error_*.swift` fixtures from st-daf0).
2. `statute link BILL-001 'Invoice.swift#Invoice.approve'` exits with code 2: `cannot link Invoice.swift#Invoice.approve: the file does not parse cleanly`.
3. The person has no way to link that code except to remember to link the whole file by hand.

The second case: an existing link `Invoice.swift#Invoice.approve` goes missing after someone adds a construct the grammar cannot parse. `statute check` reports `link.missing ... the file does not parse cleanly`. `statute review BILL-001 ok` is refused (LINKS-004). The way out exists (`link --remove`, then `link` the file), but nothing tells the person about it.

## Expected behaviour

**`link`**, in `src/commands/link.rs`:

- The fallback happens only when the mode is `SwiftAstSha256V1` and the result is `Err(Missing::ParseError)`. Then `link` records `{source: RULE, target: "Invoice.swift", role, mode: "bytes-sha256-v1", sig: <sha256 of the file>}`. It exits with code 0.
- `NoSymbol` and `Ambiguous` are still refused, for Swift too. A typo must not quietly become a whole-file link.
- Go, Python and Rust are not affected. They still refuse a file with a parse error.
- Human output says clearly that the link is whole-file and why. For example: `linked BILL-001 to Invoice.swift (whole file): Invoice.swift#Invoice.approve could not be linked because the Swift parser cannot read the file cleanly`.
- `--json` output tells agents what happened. Keep today's fields, and set `target` and `mode` to what was recorded. Add:
  - `"requested": "Invoice.swift#Invoice.approve"`
  - `"fallback": "parse-error"` (absent or `null` when there was no fallback).
  Write the exact field names into the spec.
- If the rule already links `Invoice.swift` as a whole file, write nothing. Follow the decision from st-hqdq. The recommendation is exit 0 with a message that the rule already links the whole file.
- The rest of `link` does not change: it refuses to replace an existing link, and it refuses directories and symbolic links.

**`check` and `review`**:

- An existing Swift symbol link in a file that does not parse cleanly is still `link.missing`. This is LINKS-001, and it does not change. `check` never writes the lock.
- For a Swift link only, the message adds the way out. For example: `Invoice.swift#Invoice.approve is missing: the file does not parse cleanly. If the Swift is valid, the parser may not support it yet: check the rule, then run statute link --remove BILL-001 Invoice.swift#Invoice.approve and statute link BILL-001 Invoice.swift#Invoice.approve to link the whole file`. The `review` refusal message gets the same hint.
- Messages for Go, Python and Rust do not change. Existing snapshot and contract tests must still pass unchanged.
- The decision part ("is this a Swift parse error?") belongs in `commands` or below, not in `cli` (AGENTS.md, module layering).

## Where

- `src/commands/link.rs` (`add`).
- The code that builds `link.missing` findings and the `review` refusal: start from `src/check/links.rs#freshness`, `src/state.rs#health` and `src/commands/review.rs#run`.
- Rendering in `src/cli/` only if a new field must be printed.
- docs/Statute-spec.md, section 4.4. st-hqdq should already have updated it. Check that the text matches the result.
- Tests: a new end-to-end test next to `tests/symbol_parse_contract.rs`, using `tests/common` (`TestRepo`).

## Acceptance criteria

- [x] `link` on a Swift symbol in a file with a parse error records a `bytes-sha256-v1` link to the whole file, exits 0, and prints the reason. `--json` includes `requested` and `fallback`.
- [x] `link` on a missing Swift symbol (`NoSymbol`) or an ambiguous one in a clean file is still refused with exit 2, and `statute.lock` is unchanged.
- [x] `link` on a Go, Python or Rust symbol in a file with a parse error is still refused. The existing `tests/symbol_parse_contract.rs` cases pass unchanged.
- [x] End-to-end test of the full path: link `File.swift#keep` while the file parses → commit → add a construct the grammar cannot parse → `check` reports `link.missing` with the hint → `review RULE ok` is refused with the hint → `link --remove` and `link` again → the lock has a whole-file `bytes-sha256-v1` link → `review` and `check` pass.
- [x] A test for the case where the rule already links the whole file.
- [x] The rule added by the st-hqdq decision has a code link to `src/commands/link.rs#add` (and to any helper that decides the fallback) and a test link to the new end-to-end test.
- [x] The `change` skill has run, with verdicts for LINKS-001, LINKS-004, CORE-002, CORE-003, LINKS-003 and the new rule.

## Dependencies

- Blocked by st-hqdq: the fallback rule and the exact behaviour must be in the spec and the decision.
- Blocked by st-daf0: the Swift mode must exist.
- The decision must be approved before this is merged.

## Summary of Changes

Added the Swift-only whole-file fallback, clear human and JSON reporting, and check/review recovery guidance. End-to-end tests cover retries, missing and ambiguous symbols, other languages and relinking after a parse error.
