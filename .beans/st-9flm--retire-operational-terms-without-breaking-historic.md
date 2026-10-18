---
# st-9flm
title: Retire operational terms without breaking historical references
status: todo
type: feature
priority: high
tags:
    - statute
    - planning
created_at: 2026-10-18T09:53:27Z
updated_at: 2026-10-18T09:53:27Z
---

# Term retirement with preserved historical references

## Purpose and status

Implement retirement of operational terms in a separate PR. The need for this capability is accepted by the user; the semantics below are implementation recommendations for human review, not ratified product decisions. It is useful independently of the human-glossary proposal in PR #20 and must not depend on glossary automation or on a chosen compaction audit outcome. Do not consolidate Statute's vocabulary as part of this feature.

## Current behavior and the problem

Checked the current source and spec on 2026-10-18:

- `TermAgreed` supports only `proposed` and `agreed`; `store::terms::parse` rejects `agreed: retired`.
- A term name is its reference identity. Rule `terms` and every decision's `terms` must resolve to a present heading. Active decisions cannot be changed or deleted (`check::history`). Removing an old term therefore breaks immutable history as well as current rule references.
- `decide --terms` introduces or redefines a term and marks it proposed. Decisions have `retires` for rules, but no term-retirement action. Proposal overlap currently considers only decision `terms` for terms.
- `ratify` validates an in-memory result before writing. It scopes blocking findings to selected items, and checks agreement only for rules becoming agreed. Retirement must additionally catch existing live rules using the retiring term, even if those rules were not selected.
- `show TERM` looks up the exact heading and returns its text and referencing rules; aliases are not lookup redirects. `check --all` searches identifiers for every term, and `avoid` checking also visits every term.
- `Txn` prepares all files before replacement but only each rename is atomic. A rename failure can leave several files replaced and reports them. Do not promise crash-atomic multi-file retirement.

Evidence: `src/model/mod.rs`, `src/store/terms.rs`, `src/check/{terms,rules,decisions,history,names}.rs`, `src/commands/{decide,ratify,show,status}.rs`, `src/impact.rs`, `src/model/impact.rs`, `src/txn.rs`, `docs/Statute-spec.md` sections 4.2, 4.3, 5 and 6.

## Recommended lifecycle and scope

1. Retain a retired term's heading, definition and metadata as a historical record. Add `TermAgreed::Retired`, rendered as `agreed: retired`. Do not delete or reuse its name. No automatic aliasing, renaming, revival or rewriting of historical decisions.
2. Add an explicit decision/draft action `retires_terms: ["Old term"]` and a matching `decide --retires-terms "Old term"` option; leave existing `retires` strictly for rule IDs. Keep `terms` meaning introduces/redefines.
3. An agreed term with a pending retirement stays agreed until ratification, matching proposed rule retirement. The pending action is visible in `show`, `status` and `check --md`. Cancelling the retirement deletes only the proposed decision; it does not require recovering the old definition or agreement field.
4. Only existing agreed terms can be proposed for retirement. Refuse unknown, never-agreed draft/proposed, or already retired terms. Reject one decision naming the same term in both `terms` and `retires_terms`, and reject overlapping proposals across both action lists, including merge-created overlap. A term cannot be changed and retired in the same action: preserve the historical definition, and revise any successor separately.
5. Historical active decisions and retired rules can continue referencing the retained term. Non-retired rules must not use a retired term. Proposed decisions introducing/redefining a retired term are invalid; historical active decisions that previously introduced/redefined it remain valid. Pending retirement can coexist with current references while the migration is prepared; the resulting ratification must remove them or retire their rules.
6. On ratification, validate the complete resultant model across the selected batch. Existing non-retired rules retaining the old term block retirement, even if not listed in a selected decision; attribute the finding to both the retiring term and offending rule so scoped validation includes it. Same-batch rule updates, new/redefined successor terms and rule retirements are allowed when the resultant references and approval state are valid. Do not turn unrelated errors or stale/missing code links into global ratification blockers.
7. Retired terms remain explicitly retrievable by exact name, visibly labelled retired. Rule output referring to a retired term must identify that state rather than presenting it as current advice. No silent replacement. Exclude retired terms from current `code`/`not` existence checks and `avoid` wording guidance, while keeping their metadata for historical inspection. Current terms retain existing checking behavior.
8. Protect retained identity and compared meaning (`definition`, `code`, `not`) after retirement, including deletion and unapproved resurrection. Check agreement-state transitions against a valid newly active retirement decision; a hand-edited `agreed: retired` must not bypass ratification. A pending decision alone cannot authorize an early retired state. Determine retirement provenance from decisions rather than adding a second editable authority field.

## Replacement mapping: deliberately small first version

Recommendation: do not require a machine-readable replacement field in the first implementation. A term may disappear without a replacement, split into several meanings, or merge with another. Record the reason and any successor names in the retirement decision's summary/rationale, and update current rule references explicitly through decisions. This is enough to preserve history and unblock compaction without adding redirect chains, cycles or search semantics.

Before implementation, settle whether a structured replacement list is necessary for the intended first callers. If accepted, keep it explicit and non-redirecting: allow none/one/many existing successor terms, require agreed successors in the resultant batch, forbid self references and retired successors, preserve mapping history, and define validation/lookup output. Do not infer an alias or transfer `code`/`not` metadata automatically. Otherwise defer this independently useful enhancement.

## Implementation work

- [ ] Settle the decision field/flag spelling and replacement-mapping choice; update `docs/Statute-spec.md` before implementation. Record proposed changes to agreed project rules/terms with `statute decide`; likely affected definitions include Term and Agreement status, and contracts HIST-001/003 and DEC-001/002/003. Run `statute show` on each source file before modifying it. Do not ratify without explicit human approval of specific decisions.
- [ ] Extend model enums, decision/draft schema and typed reports. Default missing `retires_terms` to empty so existing repos parse unchanged. Serialize empty new action lists with omission where practical. Old active decisions must compare semantically unchanged after the additive default, and their stored fields must not be rewritten to invent new events. Keep strict unknown-field validation.
- [ ] Extend term state patching with parse-after-patch verification. Preserve heading, definition, metadata, comments and surrounding layout. Cover existing mapping forms, leading comments and YAML markers; avoid a new generic document rewrite subsystem.
- [ ] Add pure validation for retirement actions, current-versus-historical references, proposal overlap, retired-name retention and state/provenance history. Update the existing checks rather than layering a parallel lifecycle engine. Trace checks against branches whose base already contains retired terms, and proposals already merged to the default branch.
- [ ] Extend CLI argument parsing, `--edit` drafts, `decide` reports and refusal messages. Validate every requested action before writing. Reparse command output through existing `Txn` boundaries.
- [ ] Extend `ratify` to accumulate term retirements, patch retained sections, activate exactly the named decisions, run resultant-model and history validation, and write only after successful preflight. Include affected external rule references in scoped refusal; maintain existing behavior for unrelated findings and stale/missing links.
- [ ] Update `show` text/JSON, status pending actions, term impact/reasons and Markdown PR reporting. Distinguish retiring terms from redefining terms; do not announce retired while approval is pending. Preserve compact context behavior and label historical terms. Update name/avoid search filters.
- [ ] Document lifecycle examples, cancellation, errors, version compatibility and recovery. Update plugin `decide`, `ratify`, `change` and `adopt` guidance where they handle term references or proposed decisions. Existing ratify rejection instructions must distinguish `terms` from `retires_terms` so rejecting a retirement cannot delete an agreed term. Update architecture/layering guidance only if actually needed; keep filesystem access and Git access in their current modules.
- [ ] Add meaningful contract tests below, run the relevant suites then full Rust checks, run `statute check --all`, and complete the project's `change` skill. Leave any changed agreed meanings proposed until specifically approved.

## Compatibility, retries and partial failure

Existing term files and decisions require no bulk migration. New binaries read the old schema; existing active decisions keep resolving to retained records. Older binaries will reject new retirement state/action fields due to strict schemas: document the minimum supporting release before using the feature. Inspect the current `statute.yml` format version and JSON-report schema policy and make an explicit versioning decision rather than silently claiming backward compatibility with old readers. Update fixtures only where feature semantics require it.

Keep validation refusals write-free. A retry after a successful command currently reports that the decision is already active; preserve that convention and guarantee no duplicate retirement/activation. Do not introduce an unrelated generic idempotency subsystem. On a failed rename or interrupted multi-file update, report actual changed paths, run checks, and restore the touched files to a known consistent snapshot before retrying; never advise rerunning an already partially activated decision blindly. Test the in-memory no-write guarantee separately from the existing filesystem transaction limit. Preserve the single-writer command assumption; concurrent branch proposals are detected after merge. Do not claim simultaneous CLI writers are serialized without adding and testing such support in separately scoped work.

## Acceptance tests

- Old-format fixture parses and checks unchanged; old active decision equality/history remains valid; new retirement fields round-trip; invalid state/action metadata identifies its item.
- Propose retirement: term remains agreed, old references still resolve, pending views name its action. Reject/delete proposal: term remains unchanged. Unknown, draft/proposed and already retired targets refuse without writes.
- Cross-action duplication, mixed rule/term names, and overlapping proposals refuse; merge-created overlaps are findings. Retired names cannot be reused or resurrected through `--terms` or manual state edits.
- Ratify unreferenced term: exact selected event activates, term becomes retired, historical decisions still check, exact-name show labels it retired, current identifier/avoid checks ignore it, unrelated pending items remain unchanged.
- Referenced term: existing non-retired rule outside the selected batch blocks retirement, with actionable attribution and byte-identical files. References from retired rules remain valid. Proposed rules must be updated too; pending is not an exemption from an invalid resultant retired reference.
- In one batch, update rules to agreed successors and retire old terms, or retire their rules; all succeed only when resultant references and successor approval are valid. A successor pending elsewhere, malformed term or invalid selected rule refuses the whole batch without writes. Unrelated broken rules and stale/missing code links retain existing scoped semantics.
- Base contains retired term: deletion, definition/identifier change, name reuse and resurrection are rejected. Active historical events stay immutable. Pending retirement on the default branch remains visible and can later be ratified against an older agreed baseline.
- `show`, status, text/JSON output and `check --md` distinguish retirement proposed versus completed; selected-batch reporting includes all term actions, and ordinary term redefinition behavior remains intact.
- CLI examples include spaces in term names, `--edit` parse/validation failure preserving the scratch draft, retries after success, refusal with no writes, and documented partial-write recovery. Reuse existing transaction tests; add failure injection only where needed to substantiate a new guarantee.

## Non-goals

No actual glossary migration or term compaction, glossary-section fingerprints, bidirectional glossary consistency records, automatic affected-rule discovery from code names, aliases as redirects, historical decision rewriting, generalized plugin/registry architecture, or broad transaction redesign.
