---
# st-uvvi
title: Review the operational terms compaction audit and choose migrations
status: todo
type: task
priority: normal
tags:
    - statute
    - planning
created_at: 2026-10-18T12:07:14Z
updated_at: 2026-10-18T13:48:39Z
---

## What

Review the existing audit of Statute's operational terms (`statute/terms.md`, 21 terms on 2026-10-18) and, for each candidate it raises, choose keep, merge, split or reword, with the reasoning and the migration it implies. The output is a reviewed record of migration choices for a person to approve. This Bean changes no term, rule or decision; each chosen migration is its own later PR.

Input: the complete compaction audit is embedded below under "Recorded compaction audit". It was produced on 2026-10-18 against commit `5a7d6fe3eaccb380ea8c9a58181f97464d1deb73` while PR #20 was prepared. Use this preserved snapshot as the starting evidence; the task no longer depends on locating the original temporary file or asking for it.

Plan: "Item 3" in docs/Terms-refactoring-plan.md. Criteria: "Reviewing operational terms for compaction" in docs/Terms-refactoring-proposal.md. There is no target term count, and `terms.md` is judged as operational vocabulary, not learning material.

## Why

Separating the human glossary (st-25bl) from the operational terms does not mean every current entry is well chosen. Entries whose separate identities add no operational precision cost agents context and people review effort; entries that combine meanings needing independent approval or retrieval should be split.

## Where

- Read: `statute/terms.md`, the `terms` lists in `statute/rules/*.md`, `statute show TERM` for each candidate (its rules), and `statute check --all` for code-name hits.
- Write: a review record, as a document in `docs/` or in this Bean, whichever the reviewer prefers. No changes to `statute/` files.

## Acceptance

- [ ] Every candidate in the audit has a choice (keep, merge, split, reword) answered against the five questions in the proposal's compaction section.
- [ ] For each merge or split: the rules that reference the affected terms, their `code` and `not` names, and the expected effect on `statute show` and `statute check --all` results (does the merge widen what they return?).
- [ ] Each migration is classed as keeping every existing term (can proceed after review, through proposed decisions) or removing/retiring a term (waits for st-9flm).
- [ ] A person has reviewed the record; follow-up Beans are filed for the approved migrations.

## Dependencies and constraints

- Can start now; it needs no code and does not wait for st-jycx, st-25bl or st-9flm.
- Migrations that follow: every change to an agreed term goes through `statute decide` and explicit human approval; a migration that removes or retires a term waits for st-9flm; once st-25bl has landed, each migration PR includes a glossary consistency record.

## Audit results and review boundary

The audit covers all 21 agreed terms at the audited commit. Its findings and proposed definitions are recommendations for this task to assess, not human-approved migration choices. Accepting the need for compaction did not approve any specific merge, wording, or retirement. Keep this Bean open for the review and approval work described above.

- First-batch candidates: Proposed decision into Decision; Code link and Test link into Link. The report supplies proposed compact definitions, metadata to preserve, exact rule-reference remappings, and the wider lookup/approval scope to evaluate.
- Later candidates requiring wording and retrieval trials: Protected case and Boundary case into Rule; Affected rule into Rule review.
- Keep initially: the core entries and specialized meanings including Agreement status, Build state, Approval, Manual check, Stale link, Missing link, Fingerprint, Base, and Draft rule. The report gives an operational reason for each.
- There is no target term count. The first candidates would remove three active identities if chosen; that count is a consequence, not a goal. Retirement may retain historical headings.
- Reuse the documented inventory and experiments. Add new evidence only where the current head differs or an open question needs testing. Account explicitly for `code`, `not`, `aliases`, and `avoid`, lookup behavior, historical identities, and independent approval.

The report describes behavior at its audited commit. In particular, its first-paragraph definition/history and retrieval observations are baseline facts, not a requirement to retain that policy: st-jycx plans whole-section protection and rule-context retrieval. Revalidate these and retirement/lookup assumptions on the migration PR's actual head, including any implemented st-9flm behavior. The original local copy at `/private/tmp/terms_compaction_audit.md` is optional; all audit content is preserved here.

## Recorded compaction audit

The following report is reproduced unchanged from the original audit. Its proposed work and wording remain subject to human review.

# Statute operational terms compaction audit

Date: 18 October 2026, Australia/Sydney.

Audited repository: `~/code/statute`, commit `5a7d6fe3eaccb380ea8c9a58181f97464d1deb73` (PR #20 before its next proposal revision). The audit reads the current operational terms, every current rule and decision, the one-pager, the proposal, the specification, and the relevant parser, history, reference, retrieval, impact and name-checking code. This is an audit for later human review. It changes no repository files and approves no definitions or decisions.

The user has accepted that terms compaction and term retirement need work. This report proposes the compaction scope; the feature enabling retirement is a separate implementation. Neither task is part of the PR #20 documentation proposal. The coordinating agent created high-priority retirement feature `st-9flm` and glossary automation feature `st-uzzw` during this run; their implementation is outside this audit.

## Recommendation

Keep the human glossary and operational vocabulary at different levels of detail. Evaluate operational terms as units of approval, references, retrieval and code associations. Do not minimize their number as a proxy for learning effort.

There are useful consolidation candidates in the existing 21 agreed entries. The strongest are **Proposed decision into Decision**, and **Code link and Test link into Link**. Both can retain their distinctions explicitly in the parent's compact definition. Two further candidates need a retrieval and wording comparison: **Protected case and Boundary case into Rule**, and **Affected rule into Rule review**. These are recommendations for assessment and proposed changes, not approval to remove the old identities.

Keep independent entries for Agreement status, Build state, Approval, Manual check, Stale link, Missing link, Fingerprint, Base and Draft rule for the first compaction pass. Their meanings are specialized, have distinct subjects or remedies, or need narrow code discovery. A state or property can earn a separate entry; its grammatical form does not settle the question.

A successful first batch would remove three active identities, if the prototypes pass the criteria below. That would produce 18 active entries, but **18 is a consequence of those particular choices, not a target**. Later candidates should be judged separately. Retained historical entries may still physically exist after retirement, so physical heading count is not a useful success measure.

## What the tool does today

These are current behavior, not proposals:

- **Identity is the exact heading name.** Each `##` section is a term. `#` headings only group entries. `TermMeta` has `agreed`, `code`, `not`, `aliases` and `avoid`; unknown fields are rejected. There is no retired term status. See `src/store/terms.rs:11`, `src/model/mod.rs:141` and `docs/Statute-spec.md:165`.
- **The first paragraph carries the operational definition.** Normal rule retrieval embeds only that paragraph; direct term retrieval includes the whole source section. History compares the definition with whitespace collapsed, plus `code` and `not`. Extra paragraphs, aliases and avoid words are outside that history comparison. See `src/commands/show.rs:189`, `src/check/history.rs:120` and HIST-003.
- **References are exact and direct.** `show TERM` returns rules whose `terms` list names exactly that term. It does not expand a parent/child taxonomy or follow references through another definition. Rule retrieval does not recursively load terms mentioned in definition prose. See `src/commands/show.rs:108`, `:189`, and `:260`.
- **Aliases are not redirects.** An alias neither resolves a rule's `terms` reference nor makes `show ALIAS` work. `show "pending decision" --json` returned exit 2 in a disposable checkout, although Proposed decision has that alias.
- **Approval boundaries are per term.** `decide` rejects a second pending decision naming the same term; merged overlaps are findings. Merging entries can serialize independently meaningful changes. It does not block arbitrary code edits or imply that pending proposals block merges. See `src/commands/decide.rs:171`, `src/check/decisions.rs:62` and DEC-001.
- **Code associations are names, not dependency coverage.** `check --all` checks `code` and `not` strings as whole identifiers in non-Markdown files outside Statute's data. It does not prove the named implementation implements the definition. Current `show` does not search code from term metadata. Code-name-driven affected-rule discovery remains deferred. See `src/check/names.rs`, `src/impact.rs:57` and `docs/Statute-spec.md:417`.
- **Deletion is not a supported migration.** All decision term references, including active history, must resolve. Active decisions cannot be rewritten. `decide` refuses a deleted term. In a disposable checkout, deleting Code link caused unknown-term findings for CORE-003, LINKS-002 and historical D261016-f5ieq, plus `term.changed_without_decision`.

These boundaries rule out a few shortcuts. Putting the important distinctions in a second paragraph loses normal rule context and history protection. Combining metadata preserves name existence checks, but its flat list loses an explicit association between an individual submeaning and its names. Adding an alias does not preserve old operational references.

## Assessment criteria

For each candidate, compare the current and proposed behavior on these questions:

1. Would distinct meaning changes reasonably need separate proposals and approval? Existing shared introduction in one decision is evidence of coupling, not proof that future approval should be shared.
2. Do rules need a narrower reference than the proposed parent? Compare the actual dependent rule sets, not only term spelling.
3. Can an agent retrieve the useful definition without loading substantially unrelated meaning? A compact parent must be self-contained because retrieval is not recursive.
4. Are code names and exclusions still attributable to the right meaning? Unioning metadata is possible; it is not free of precision costs.
5. Does the definition preserve every condition that changes outcomes? Ordinary words can encode specialized behavior, such as Draft rule or Missing link.
6. Can historical names, decisions, rule references and useful lookup names remain intelligible? This requires a migration policy, not retrospective editing of history.

No entry should split just because the implementation uses two enum values. No entries should merge merely because one describes a field of another.

## Inventory and recommendations

Rule references below were counted from the actual current rule metadata and checked with `statute show` for the main candidates. Zero references does not mean an entry is unused: definitions, status output, agent instructions and code discovery are separate consumers.

| Current term | Current direct rule references | Recommendation and operational reason |
| --- | --- | --- |
| **Term** | CORE-004, DEC-001, DEC-003, HIST-003 | **Keep.** The reference and approval object is central, and distinguishes a product meaning from an arbitrary code identifier. Do not put the human glossary's instructional purpose into this definition. |
| **Rule** | CORE-001–004, DEC-001, DEC-003, GAPS-001, HIST-002 | **Keep.** Candidate receiver for protected/boundary case semantics, but avoid absorbing every lifecycle, verification and review concept. Its present definition is only 25 words. |
| **Decision** | CORE-005, DEC-002–003, HIST-001–003, LINKS-003 | **Keep and enrich in a proposed change.** It can own proposal/active lifecycle and immutability without introducing Active decision. Preserve the separate human authority boundary in Approval. |
| **Link** | LINKS-001–004 | **Keep; strongest receiver for Code link and Test link.** The current role entries have identical direct dependent rules and map naturally to one two-value role property. Preserve both jobs and the CI distinction in the definition. Keep health and fingerprint details separately addressable initially. |
| **Code link** | CORE-003, LINKS-002 | **Candidate merge into Link, first batch.** No current rule depends on this entry without also depending on Test link. `LinkRole::Code` can remain explicitly associated in definition prose and preserved in Link's metadata. Separate future approval of code-role semantics is possible but not yet demonstrated as valuable enough to retain this identity. |
| **Test link** | CORE-003, LINKS-002 | **Candidate merge into Link, first batch.** Same reference set as Code link. Preserve that the linked test can catch violations outside linked implementation and is run by project CI; Statute fingerprinting does not execute it. |
| **Manual check** | CORE-003 | **Keep.** Distinct human procedure and verification substitution: it replaces the test requirement, never the code requirement. Independent policy is changing in existing mixed automated/manual verification work (`st-8j2x`, referenced in the rule notes). Folding it into Link would blur the distinction between a procedure and a target connection. |
| **Protected case** | CORE-001 | **Candidate merge into Rule, second batch after a prototype.** It and Boundary case are paired fields of the same rule obligation with identical direct dependencies. Keep their meanings explicit in the first paragraph if merged; preserve `protects`. The concern is increased context in all Rule consumers, not the existence of a distinct field. |
| **Boundary case** | CORE-001 | **Same second-batch candidate.** Preserve its specific purpose of identifying nearby valid behavior outside scope; do not reduce both fields to generic examples. Preserve `boundary`. If the full Rule paragraph becomes too broad, keeping both is preferable to inventing a new umbrella term merely to lower the count. |
| **Gap** | GAPS-001, LINKS-003 | **Keep.** A tracked mismatch has its own lifecycle and remediation, distinct from a mechanical stale/missing condition or a request for review. A glossary can teach these together without merging them. |
| **Agreement status** | CORE-002, CORE-004, DEC-002, HIST-002–003 | **Keep.** Shared property of rules and terms with deliberately different allowed states. It changes independently of implementation. Folding it into Rule and Term duplicates meaning; folding it into Build state conflates facts CORE-002 separates. Preserve both `Agreed` and `TermAgreed`. Retirement will require an intentional future revision of its term-state description. |
| **Build state** | CORE-002–003, GAPS-001 | **Keep.** Distinct completeness and broken-state semantics, already separately referenced from agreement. `Built` has planned/built values, whereas broken is derived from gaps. Preserve that distinction. |
| **Affected rule** | None | **Candidate merge into Rule review, second batch.** Its critical distinction is “needs review” rather than “broken.” Rule review already describes what happens to affected rules. Prototype one definition that includes the trigger distinction and the assessment responsibilities without exhaustively repeating impact algorithms. No metadata is lost, but exact lookup compatibility still needs a migration policy. |
| **Rule review** | LINKS-003–004 | **Keep.** It owns behavioral assessment and an agent/tool/person responsibility boundary. Candidate receiver for Affected rule. Do not hide machine-versus-human judgment in the glossary. Its current paragraph is already 68 words, so assess duplication and retrieval cost before adding to it. |
| **Approval** | CORE-002, CORE-005, DEC-002–003 | **Keep.** Explicit human acceptance of a specific decision is a separate authority event from an agent recording it. It can explain ratification within the same entry; a Ratification entry is not required merely because the CLI has that command. |
| **Base** | HIST-001–003, LINKS-002 | **Keep.** Specialized comparison semantics determine checks and historical protection. A familiar Git word is not enough to infer the default-branch fallback and inclusion of uncommitted/untracked changes. Preserve `git::base`; do not treat its `merge-base` alias as an exact universal substitute because the fallback can be HEAD. |
| **Stale link** | LINKS-002–003 | **Keep.** Narrow fingerprint-comparison condition and review-to-refresh remedy, independently referenced and associated with `Health::Stale`. Merging it into Link would broaden every role-related reference to include refresh policy. |
| **Missing link** | LINKS-004 | **Keep.** Missing includes nonexistent, ambiguous and unparsable targets, and review cannot clear it. The repair differs from stale-link review. `Health::Missing` is a useful precise code association. A Link health parent would reduce count but broaden retrieval and approval with no demonstrated gain. |
| **Fingerprint** | None | **Keep initially.** Mode-dependent normalization and the distinction between changed hash input and changed behavior are independently important to linking/fingerprinting work. Preserve `fingerprint::sign`, `sig` and `avoid: [checksum]`. No direct rule currently imports this definition; a separate decision should consider whether LINKS-002 should name it. Avoid warnings would still function if metadata moved, so that alone does not justify a separate entry. |
| **Draft rule** | None | **Keep initially.** A draft is not every proposed rule; it has no establishing decision and is pending rather than a finding. `is_draft` provides a narrow code association. It could fold into Rule later, but combining that with case semantics would grow Rule's definition for eight dependent rules to save a short specialized entry. |
| **Proposed decision** | DEC-001–002 | **Candidate merge into Decision, first batch.** Lifecycle can be compactly owned by the parent. Preserve retirement staying in force pending approval, edit/delete permissions, merge behavior and `DecisionStatus::Proposed`. Its `pending decision` alias is descriptive metadata, not a functioning redirect today. |

No split is recommended immediately. Agreement status and Rule review deserve scrutiny during wording work because they contain multiple subjects or actor responsibilities, but those are connected meanings rather than demonstrated independent concepts. Ratification, Finding, Link target and Guide were previously discussed additions; this audit does not recommend creating them before a concrete operational reference, approval or retrieval need is shown. Explain them in the appropriate glossary/workflow section meanwhile.

## First-batch prototypes and exact mapping

The following wording is **proposed audit material**, not approved replacement text. All substantive meaning must fit in the first paragraph under today's parser and history policy.

### Decision with its lifecycle

> A recorded choice about rules or terms, with its reason and, once active, who approved it. It starts proposed: its introduced or changed rules and listed terms stay proposed, while rules it will retire remain agreed until ratification. A proposed decision can be edited or deleted and never blocks a merge. Ratification records a person's explicit approval and makes the decision active and immutable; later corrections require another decision.

Preserve metadata `code: [DecisionStatus::Proposed]` and the descriptive alias `pending decision` from Proposed decision. Add other identifiers only after a separate code-association check; `Decision`, `DecisionStatus` or `DecisionStatus::Active` are possible future improvements, not required for this merge.

Rule remapping: replace Proposed decision with Decision in DEC-001; remove the duplicate reference in DEC-002 because it already names Decision. Other Decision references remain. Historical decisions keep the original names and need retirement support to remain meaningful.

Tradeoff: every Decision consumer receives lifecycle detail, even if it only needs immutable history or a review's decision reference. Compare output size and task usefulness against the current pair. Parent/child vocabulary alone is not proof that this tradeoff wins. The definition is about 75 words, in the range of the existing Rule review paragraph, with concrete specialized distinctions preserved.

### Link with its two roles

> A connection from a rule to an implementation or test target, with a fingerprint used to detect target changes. A code-role link points to the implementation that makes the rule true and shows reviewers where to look when it changes. A test-role link points to a test of the rule's behavior, run by the project's CI, which can catch violations introduced outside the linked implementation. Changes to either role's target can require rule review.

Preserve `code: [LinkLine, "LinkRole::Code", "LinkRole::Test"]`. Do not silently replace `LinkLine` with the Rust `Link` type as part of this task: the existing string exists in the repository and represents an already approved association. Any enrichment or correction should be explicit. Preserve Link's tracker-reference distinction; if judged a material qualification rather than explanation, include that distinction in the first paragraph under a decision rather than leaving it solely in later prose.

Rule remapping: CORE-003 replaces Code link and Test link with Link; LINKS-002 already names Link, so remove its two redundant references. No other current rules reference the two roles separately.

Tradeoff: `show "Code link"` and `show "Test link"` currently return exactly CORE-003 and LINKS-002. A future redirect to Link would return those plus LINKS-001, LINKS-003 and LINKS-004. That is a real retrieval broadening even though the two role entries have identical dependents. Approve this explicitly or retain the role entries if task trials show the broader result is harmful. Do not claim that ordinary aliases preserve narrow lookup.

### Second-batch trials

- For Protected case/Boundary case, add a compact sentence to Rule explaining protected behavior and a nearby valid case outside scope; preserve both field-name mappings. CORE-001 removes its separate references; its statement and case metadata still keep the two obligations distinct. Then inspect rule output for all eight existing Rule consumers.
- For Affected rule, begin Rule review with “An affected rule is one a change brings up for review; this does not itself mean it is broken,” then state assessment, outcomes and actor responsibilities once. No current rule references need remapping, but old historical and direct-lookup names remain a migration concern.

## Separate compaction from tooling changes

Semantic compaction changes approved definitions and rule references. These operations already need proposed decisions and human approval; familiar words and editorial intent do not exempt them.

Format and retrieval work is separate. Parent/child definitions, recursive expansion, protected whole-body prose, aliases as lookup redirects, field-scoped code-name metadata and glossary reference fields all require specifications and implementation. None should be smuggled into a terms-only cleanup. In particular, today's YAML rejects an invented `glossary` field, and today's extra paragraphs cannot carry hidden authoritative conditions.

Retirement needs its own feature PR and lifecycle tests. It should preserve historical term identities and explain replacements without allowing a retired definition to silently satisfy a live agreed rule. Whether old direct lookups show a retained definition with a replacement pointer or redirect to a parent is a design choice for that feature. If they redirect, the change in rule-result scope must be visible. Do not rewrite historical decisions to remove retired names.

The audit, proposed wording, code-name inventory, glossary drafting and retrieval comparisons can proceed before retirement exists. Final removal of active identities, new live-reference validation and safe migration depend on the feature's chosen contract. Keeping two active entries with the same newly merged definition is not a useful interim “compaction”: it retains proposal contention and creates duplicate sources of truth.

## Proposed work sequence

1. **Keep PR #20 as the design proposal.** It can introduce Statute's own glossary and manual consistency review in both directions before automated review exists. Do not edit approved operational definitions as incidental documentation cleanup.
2. **Review this audit and choose the first mappings.** Confirm whether the broader Decision and Link approval/retrieval units are acceptable. The decision should be about precise proposed wording and mappings, not an entry count.
3. **Prepare retirement independently in `st-9flm`.** Define history preservation, allowed live references, proposal/ratification/rejection transitions, status/show behavior, any replacement mapping actually needed, and branch/merge cases. The current feature direction retains sections and proposes a `retires_terms` action without an initial redirect subsystem; exact old-name retrieval can therefore retain historical context without redirecting. Its usefulness extends beyond these candidates.
4. **In a separate compaction PR, prototype the approved mappings in a disposable checkout using the implemented lifecycle.** Enumerate every definition and metadata field being retained or intentionally changed, every live rule reference remapped, every historical identity retained, and the expected direct lookup results.
5. **Create scoped proposed decisions for terms and changed rules.** For the first candidates, list the surviving and retired term identities as the retirement interface requires; name DEC-001, DEC-002, CORE-003 and LINKS-002 where their term metadata changes. If proposals overlap, coordinate them rather than bypassing DEC-001. Never ratify on the strength of this audit or acceptance of the need for compaction.
6. **Validate semantic preservation and retrieval, then request approval.** Use the scenarios below, `statute show` output comparisons, both Statute checks and any tests required for the retirement implementation. A fresh lock signature or clean structural check alone does not prove semantic preservation.
7. **Only then assess second-batch candidates.** Keep or abandon them based on actual definition density and task retrieval, including whether added parent detail slows unrelated work. Record any remaining choices for other agents rather than forcing all work into the first PR.

## Acceptance cases for compaction

These are useful behavioral or retrieval checks; do not add tests that merely assert the spelling of the new paragraphs.

- An agent given CORE-003 still sees both the implementation and verification roles, knows CI runs linked tests, and knows a manual check does not replace a code link.
- An agent given LINKS-002 understands that fingerprint changes may be behavior-preserving and that stale reporting applies to both code and test roles. If Fingerprint is needed in that context, a rule must reference it explicitly or the parent must include the needed condition; prose mention does not expand it automatically.
- An agent given DEC-001 understands proposal overlap, while an agent given HIST-001 understands active-history immutability. A pending retirement does not make an in-force rule disappear prematurely.
- A protected case and boundary case still serve different purposes after any second-batch merge; ordinary happy-path examples cannot stand in for nearby valid cases outside scope.
- A draft remains distinct from a rule already established by a pending decision. Affected, stale, missing, incomplete and broken are not collapsed into one generic bad state.
- The union of preserved code/not names still passes name checks. Alter a mapped name in a scratch copy and verify its warning is attributed to the intended surviving term. Preserve avoid-word behavior, including `checksum` if its owning term changes.
- Old active decisions remain immutable and resolvable. Old term lookup behavior follows the retirement contract. Live rules cannot accidentally depend on obsolete semantics or a replacement that means something broader.
- A proposal that changes or retires one term cannot silently agree another, and rejection/restoration preserves the last agreed definitions and live references. Concurrent migrations or proposals surface meaningful overlap instead of silently discarding mappings.
- Show the glossary section and compact definition together for human review; compare both directions manually. Confirmation that they remain consistent is not approval of a proposed meaning change.

## Evidence and limits

Read-only validation against the audited checkout: the installed `statute check --all --json` returned zero errors, zero warnings and no findings. `statute show` confirmed the exact role/case dependent-rule sets and the zero direct references for Affected rule, Fingerprint and Draft rule. A scripted Markdown inventory counted all 21 terms and first-paragraph lengths; all are currently `agreed: agreed`.

A disposable tracked-file copy at `/private/var/folders/kt/l619hns12m96t4y17yv0k8lm0000gp/T/statute-compaction-audit-4xc7dodu` reproduced alias lookup failure and the four Code link deletion findings. The scratch copy has its own Git baseline; it does not alter the shared checkout.

Important source anchors:

- `statute/terms.md`: the audited 21 entries, code mappings and current paragraphs.
- `statute/rules/core.md`, `decisions.md`, `gaps.md`, `history.md`, `links.md`: direct reference sets and operational guarantees.
- `statute/decisions.db`: active decision D261016-f5ieq preserves the original role/case names; later decisions preserve the specialized health, fingerprint and lifecycle terms.
- `src/commands/show.rs:96`, `:189`, `:260`: exact name dispatch, embedded definitions and direct dependent rules.
- `src/check/history.rs:71`, `:120`: removal protection and protected term fields.
- `src/check/rules.rs:95`, `:171`: exact rule references and avoid warnings.
- `src/check/decisions.rs:34`, `:62`: historical term-reference resolution and proposal overlap.
- `src/commands/decide.rs:171`: unknown names and pending proposal refusal.
- `src/check/names.rs`: flat identifier associations, file exclusions and name warnings.
- `src/model/mod.rs:154`, `:170`, `:274`, `:305`: term schema/status and link roles.
- `src/state.rs:18`, `:52`, `:74`: draft, completeness and derived broken state.
- `src/impact.rs:57` and `docs/Statute-spec.md:417`: current term impact comes from changed decisions; code-name impact remains deferred.
- Existing contracts: `tests/history_contract.rs`, `tests/decisions_contract.rs`, `tests/core_lifecycle.rs`, `tests/core_rule_cases.rs`, `tests/cli.rs`, `tests/impact.rs`, `tests/review_contract.rs` and `tests/missing_contract.rs`.

The merge recommendations infer likely future editing and retrieval needs from today's references and semantics. There is no usage telemetry showing how often agents retrieve each term, no measured cognitive-load study, and no claim that current identical reference sets guarantee future coupling. The audit does not validate proposed definitions as ratified behavior or run a speculative retirement implementation. Source line numbers and dependent-rule sets should be rechecked on the implementing PR's head.
