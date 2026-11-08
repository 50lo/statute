---
# st-hqdq
title: Specify Swift symbol links and the whole-file fallback, and propose a decision
status: completed
type: task
priority: normal
created_at: 2026-11-08T10:15:10Z
updated_at: 2026-11-08T11:48:13Z
parent: st-97ld
---

## What

Write down how Swift symbol links work, before any code is written:

1. Which Swift declarations are symbols, and how a target names them.
2. What the `swift-ast-sha256-v1` hash input includes and drops.
3. When `link` falls back to a whole-file link, and what `check` and `review` say about Swift links that stop parsing.

Then update the spec and the architecture doc, and propose a Statute decision with `statute decide` for the new or changed rules. A person must approve that decision before st-daf0 and st-4dc0 are merged.

## Why

The hash input decides how many false alarms and missed changes a mode produces (docs/Architecture.md, section 6.1). It cannot be changed later without a new mode version. The fallback changes how `link` behaves today, and agreed rules cover that behaviour. So both need a decision first.

Background: the epic records two decisions by Anton Solo (2026-11-08). We use `tree-sitter-swift` 0.7.4 as it is today, and we fall back to a whole-file link when the grammar cannot parse a file.

## Where

- docs/Statute-spec.md: section 4.4 (the "Fingerprint modes" table and the text after it, including "A parse error is fixed in the file"), section 5.1 (dependency table).
- docs/Architecture.md: section 6 (`Mode` enum) and 6.1 (add a "Swift symbols" item).
- `statute/rules/links.md`: LINKS-001 and LINKS-004. Use `statute show LINKS-001 LINKS-004`.
- `src/commands/link.rs`: today `link` stops with `cannot link <target>: the file does not parse cleanly`.

## Step 1: look at the grammar output

Before you choose names, parse a few real Swift files with `tree-sitter-swift` 0.7.4. Note the node kinds for declarations: class, struct, enum, actor, protocol, extension, func, init, deinit, subscript, var/let, typealias. A throwaway program is fine. Do not commit it.

## Step 2: questions to answer

Each question has a recommendation. You may choose differently, but give the reason in the decision.

**Symbol names**

- *Overloads.* Swift often has several functions with the same base name, for example `init(name:)` and `init(id:)`, or `move(to:)` and `move(by:)`. With base names only, many links would be `Ambiguous` and `link` would refuse them.
  Recommendation: accept the base name (`Invoice.approve`) when exactly one declaration has it. Also accept Swift's own name with argument labels (`Invoice.init(name:)`, `move(to:)`, `f(_:)`), the form DocC uses. Document that the shell needs quotes around targets with parentheses.
- *Extensions.* Recommendation: members of `extension Foo` are `Foo.member`, like Rust impl methods. An extension of a nested type `extension Outer.Inner` adds to `Outer.Inner`. Generic arguments and `where` clauses are not part of the name.
- *`#Type`.* Recommendation: it covers the type's primary declaration, including all members declared inside its body. It does not cover extensions. This matches Rust (`#Type` does not cover impls). Members declared in the body are also symbols on their own (`Type.member`).
- *Nested types.* Recommendation: dotted paths, `Outer.Inner.method`.
- *Protocols.* A protocol requirement `protocol P { func f() }` and a default implementation `extension P { func f() {} }` would both be `P.f`. Recommendation: `#P` covers the whole protocol declaration. Requirements inside it are not separate symbols, so `P.f` names only the default implementation.
- *Properties and subscripts.* Computed properties often hold logic. Recommendation: `var` and `let` members (stored and computed) are symbols (`Type.name`), and so is `subscript` (`Type.subscript(_:)`). Top-level `let`/`var` are not symbols, like Rust constants.
- *`#if` blocks.* The same declaration can appear in two `#if` branches. Recommendation: treat that as `Ambiguous`, and document it.

**Hash input**

- Use the shared token hash in `src/fingerprint/syntax.rs` (leaf tokens in source order, no node kind names).
- Recommendation: comments and doc comments (`///`, `/** */`) are dropped. Attributes (`@MainActor`, `@available`, property wrappers such as `@Published`) and modifiers (`public`, `final`, `nonisolated`) count.
- *Line breaks (important).* Swift ends statements at line breaks. Today's hash ignores all whitespace. Check whether two token streams that differ only in line breaks can mean different code. Possible cases: a `(` or `[` at the start of a new line, `return` at the end of a line, `;` between statements. If such a case exists, the hash would miss a behaviour change. That breaks principle 1 in AGENTS.md. Recommendation: if you find such a case, add a statement separator to the hash input (for example, one separator byte at the end of each statement node). Add a golden fixture that proves the hash changes.
- *Trailing commas.* Swift 6.1 allows them in more lists. Recommendation: drop a trailing comma before a closing delimiter, as the Rust mode does. Do not drop it inside macro arguments.

**Fallback (see st-4dc0)**

- Recommendation: the fallback happens only in `link`, only for the Swift mode, and only for `Missing::ParseError`. `Missing::NoSymbol` and `Missing::Ambiguous` are still refused, because a typo in a symbol name must not quietly become a whole-file link. Go, Python and Rust still refuse a file with a parse error. Their grammars are mature, so a parse error there is a real syntax error that the author can fix.
- `check` does not change: an existing Swift symbol link in a file that stops parsing is still missing. `check` never writes the lock, and only `review` refreshes links (LINKS-003). LINKS-001 and LINKS-004 stay true.
- The `check` and `review` messages for a missing Swift link with a parse error name the way out: check the rule, run `link --remove`, then run `link` again, which falls back.
- If the rule already links the whole file, `link` writes nothing. Recommendation: exit 0 with a message that the rule already links the whole file, so that agents can retry safely. The alternative is exit 2, like the current "already links" error.
- *Lost intent.* A fallback link forgets which symbol the rule was about. Later nothing shows which whole-file Swift links could be narrowed when the grammar gets better. Options: (a) do not record it; (b) add an optional lock field such as `"wanted": "Invoice.approve"`, absent by default, using the same pattern as Architecture 8.1, so the lock version stays 1. Recommendation: (a) for now, and open a follow-up bean if people ask for it. This question does not block the work.

## Step 3: write it down

- Update docs/Statute-spec.md, section 4.4:
  - add a `swift-ast-sha256-v1` row to the "Fingerprint modes" table, with targets and notes;
  - describe the fallback;
  - change "A parse error is fixed in the file" so it covers the Swift fallback;
  - remove Swift from "Later modes (Swift, TypeScript)".
- Update section 5.1: add `tree-sitter-swift` and remove Swift from "later".
- Update docs/Architecture.md, section 6: add `SwiftAstSha256V1` to the `Mode` enum. In section 6.1, add a "Swift symbols" item and the Swift comment and line-break rules.
- Run `statute decide` to propose a decision. It adds a new LINKS rule for the fallback, for example "link falls back to a whole-file link for a Swift file that does not parse cleanly", with `protects` and `boundary` cases. Before asking for approval, check the rule as AGENTS.md says ("Before asking a human to approve a rule…"). List any changed rule in `changes`. If a term changes, follow the glossary steps in AGENTS.md.
- Ask a person to approve. Run `statute approve` only after they explicitly approve this decision.

## Acceptance criteria

- [x] Each question in step 2 has an answer in the spec or architecture doc.
- [x] The line-break question has a written conclusion: "no case found", with the cases tried, or "separator added", with the case.
- [x] Spec 4.4 and 5.1 and Architecture 6 and 6.1 describe the Swift mode and the fallback.
- [x] A proposed decision lists the new fallback rule and any changed rule. `statute check` passes.
- [x] A person has approved the decision, and `statute approve` has recorded it. Approval may come after st-daf0 starts, but before st-daf0 and st-4dc0 are merged.

## Constraints

- Do not edit `agreed` or `built` by hand. The CLI sets them.
- Do not change LINKS-001 or LINKS-004 unless the decision lists them.
- Follow the plain-English style of the existing docs.
