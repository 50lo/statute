---
# st-97ld
title: Swift symbol links with a whole-file fallback
status: todo
type: epic
created_at: 2026-11-08T10:15:10Z
updated_at: 2026-11-08T10:15:10Z
---

## What

Let `statute link` link a rule to a symbol in a Swift file, such as `Sources/Billing/Invoice.swift#Invoice.approve`. Do this with a new tree-sitter fingerprint mode, `swift-ast-sha256-v1`. When the Swift grammar cannot parse a file cleanly, `link` falls back to a whole-file link (`bytes-sha256-v1`) instead of refusing.

## Why

Today a Swift file can only be linked as a whole file. Then every edit anywhere in the file, even a comment, makes the link stale and brings the rule up for review. Symbol links remove most of these false alarms. Go, Python and Rust already have them (docs/Statute-spec.md, section 4.4, "Fingerprint modes"). The spec already plans Swift as a later mode.

## Decisions already made (by Anton Solo, 2026-11-08)

- **Use the grammar that exists today.** Swift syntax is large and changes often, so no grammar will be perfect. Do not wait for a better one. Use the crate [`tree-sitter-swift`](https://crates.io/crates/tree-sitter-swift) 0.7.4 (MIT licence, community project by alex-pinkus). The grammar check in st-daf0 is not a go/no-go gate. It records what fails, so the fallback and the docs can deal with it.
- **Fall back to a whole-file link.** When the grammar cannot parse something in a Swift file, a whole-file link is a better experience than a refusal. It is also safe under project principle 1 in AGENTS.md ("A missed behaviour change is worse than a false alarm"): a whole-file link gives more false alarms, never fewer.

## Child tasks, in order

1. st-hqdq: specify Swift symbol names, the hash input and the fallback, then propose a Statute decision.
2. st-daf0: add the `swift-ast-sha256-v1` mode. Blocked by 1.
3. st-4dc0: add the whole-file fallback to `link`, and point `check` and `review` to it. Blocked by 1 and 2.

## Where

- Fingerprints: `src/fingerprint/` (`mod.rs`, a new `swift.rs`, shared `syntax.rs`). Design: docs/Architecture.md, section 6.
- Mode enum: `src/model/mod.rs` (`Mode`).
- Link command: `src/commands/link.rs`.
- Spec: docs/Statute-spec.md, sections 4.4 and 5.1.
- Rules: `statute/rules/links.md` (LINKS-001, LINKS-004).
- Reference for adding a mode: commit e355ce4 ("Add the rust-ast-sha256-v1 fingerprint mode").

## Acceptance criteria (epic)

- [ ] `statute link RULE File.swift#Symbol` records a `swift-ast-sha256-v1` link when the file parses cleanly.
- [ ] When the file does not parse cleanly, `link` records a whole-file `bytes-sha256-v1` link and says so in human and `--json` output.
- [ ] A proposed decision with the new rule(s) has been approved by a person and recorded with `statute approve`.
- [ ] The spec, the architecture doc and AGENTS.md list Swift as supported.
- [ ] All child tasks are completed.

## Constraints

- Module layering in AGENTS.md must hold. `tests/layering.rs` checks it.
- New modes are new enum variants. Do not add traits, registries or plugins (AGENTS.md, "Extension").
- Existing Go, Python and Rust behaviour and their golden hashes must not change.
- Follow the Statute workflow in AGENTS.md, section "Statute": run `statute show` before changing code, and run the `change` skill after.
