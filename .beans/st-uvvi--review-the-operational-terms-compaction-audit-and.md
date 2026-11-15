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
updated_at: 2026-11-15T00:00:00Z
---

## What

Review the existing audit of Statute's operational terms (`statute/terms.md`, 21 terms on 2026-10-18) and, for each candidate it raises, choose keep, merge, split or reword, with the reasoning and the migration it implies. The output is a reviewed record of migration choices for a person to approve. This Bean changes no term, rule or decision; each chosen migration is its own later PR.

Input: the complete compaction audit is embedded below under "Recorded compaction audit". It was produced on 2026-10-18 against commit `5a7d6fe3eaccb380ea8c9a58181f97464d1deb73` while PR #20 was prepared. Use this preserved snapshot as the starting evidence; the task no longer depends on locating the original temporary file or asking for it.

Plan: "Item 3" in docs/Terms-refactoring-plan.md. Criteria: "Reviewing operational terms for compaction" in docs/Terms-refactoring-proposal.md. There is no target term count, and `terms.md` is judged as operational vocabulary, not learning material.

The terms refactoring plan and proposal were removed from `docs/` before the public release. Read them with `git show ba873fd:docs/Terms-refactoring-plan.md` and `git show ba873fd:docs/Terms-refactoring-proposal.md`.

## Why

Not every current entry in the operational terms is well chosen. Entries whose separate identities add no operational precision cost agents context and people review effort; entries that combine meanings needing independent approval or retrieval should be split.

## Where

- Read: `statute/terms.md`, the `terms` lists in `statute/rules/*.md`, `statute show TERM` for each candidate (its rules), and `statute check --all` for code-name hits.
- Write: the review record, in this Bean (section "Review record (2026-11-15)"). No document under `docs/`, and no changes to `statute/` files.

## Acceptance

- [x] Every candidate in the audit has a choice (keep, merge, split, reword) answered against the five questions in the proposal's compaction section.
- [x] For each merge or split: the rules that reference the affected terms, their `code` and `not` names, and the expected effect on `statute show` and `statute check --all` results (does the merge widen what they return?).
- [x] Each migration is classed as keeping every existing term (can proceed after review, through proposed decisions) or retiring a term (through `retires_terms`, under the retirement contract in `docs/Statute-spec.md` section 4.3 and rules DEC-004, DEC-005 and HIST-005), with the retirement steps it needs.
- [ ] A person has reviewed the record; follow-up Beans are filed for the approved migrations.

## Dependencies and constraints

- Can start now; it needs no code. st-jycx (whole-section protection) and st-9flm (term retirement) are completed, so nothing blocks it.
- Migrations that follow: every change to an agreed term goes through `statute decide` and explicit human approval. A migration that retires a term uses `decide --retires-terms` and must change, in the same `approve` call, every rule that still uses the term (see the review record).

## Audit results and review boundary

The audit covers all 21 agreed terms at the audited commit. Its findings and proposed definitions are recommendations for this task to assess, not human-approved migration choices. Accepting the need for compaction did not approve any specific merge, wording, or retirement. Keep this Bean open for the review and approval work described above.

The human glossary (`docs/Glossary.md`), its consistency review and its automation Bean `st-uzzw` were removed or scrapped on 2026-11-15 (PR #49). The audit below still mentions them. Ignore those parts; they ask for no work. The audit's "ratify" command is now `statute approve`.

- First-batch candidates: Proposed decision into Decision; Code link and Test link into Link. The report supplies proposed compact definitions, metadata to preserve, exact rule-reference remappings, and the wider lookup/approval scope to evaluate.
- Later candidates requiring wording and retrieval trials: Protected case and Boundary case into Rule; Affected rule into Rule review.
- Keep initially: the core entries and specialized meanings including Agreement status, Build state, Approval, Manual check, Stale link, Missing link, Fingerprint, Base, and Draft rule. The report gives an operational reason for each.
- There is no target term count. The first candidates would remove three active identities if chosen; that count is a consequence, not a goal. Retirement keeps the retired term's heading.
- Reuse the documented inventory and experiments. Add new evidence only where the current head differs or an open question needs testing. Account explicitly for `code`, `not`, `aliases`, and `avoid`, lookup behavior, historical identities, and independent approval.

The report describes behavior at its audited commit. Its first-paragraph definition and retrieval observations are out of date: st-jycx made the whole section (except the heading and the `yaml` block) the compared definition, and rule views print all of it. st-9flm implemented term retirement. The review record below lists what changed. The original local copy at `/private/tmp/terms_compaction_audit.md` is optional; all audit content is preserved here.

## Review record (2026-11-15)

This record is a recommendation for the owner. It approves no migration, wording or retirement. Each migration needs its own proposed decision and a person's explicit approval.

Checked against commit `b8df322` (main after PR #48 and PR #49). All 21 terms are still in `statute/terms.md`, all `agreed: agreed`. `statute check --all` reports no findings.

### Summary of choices

| Candidate | Audit said | Choice | Class |
| --- | --- | --- | --- |
| Code link and Test link into Link | Merge, first batch | **Merge** | Retires two terms |
| Proposed decision into Decision | Merge, first batch | **Keep** both | Keeps every term |
| Affected rule into Rule review | Merge, second batch | **Merge**, lower priority | Retires one term |
| Protected case and Boundary case into Rule | Merge, second batch | **Keep** all three | Keeps every term |
| The 15 other terms | Keep | **Keep** | Keeps every term |

No split and no stand-alone reword is recommended. The Link and Rule review merges reword the surviving term as part of the merge.

### What changed since the audit

The audit's section "What the tool does today" is wrong on these points at `b8df322`:

| Audit claim | Now |
| --- | --- |
| The first paragraph is the definition. Rule views embed only it. History compares only it. | The definition is the whole section except the heading and the `yaml` block (`definition` in `src/store/terms.rs`). History compares all of it with whitespace collapsed, plus `code` and `not` (`same_term` in `src/check/history.rs`). Rule and file views of `show` print all of it. So "all meaning must fit in the first paragraph" no longer applies. The new cost: every added sentence prints in every rule view that lists the term. |
| There is no retired term status. | `TermAgreed::Retired` exists (`src/model/mod.rs`). |
| Deleting a term is not a supported migration. | Deletion is still refused. Retirement is the supported way to remove a term's active identity (see "Retirement steps" below). |
| The command is `ratify`. | It is `statute approve`. |
| Proposed decision is used by DEC-001 and DEC-002. Decision has 7 rules; Rule has 8. | Proposed decision: DEC-001, DEC-002, DEC-004, DEC-005. Decision: 11 rules. Rule: 10 rules. Link: 5 rules (LINKS-005 is new). |
| Link's `code` is `[LinkLine]`. | `[Link, LinkLine]` (decision D261108-4ssd6). |
| Glossary steps (consistency records, showing glossary sections). | The glossary was removed. These steps are void. |

These audit claims still hold:

- A term's identity is its exact heading. `show` matches the exact name only: `statute show "code link"` and `statute show "pending decision"` both exit 2.
- Aliases are not redirects. `show` does not resolve them, and a rule that lists `pending decision` in `terms` gets `rule.term_unknown` (re-run in the disposable clone).
- References are direct. `show TERM` returns only rules that list that exact term. A rule view prints only the terms the rule lists; it does not follow terms named in a definition's prose.
- Approval boundaries are per term: a rule or term is in at most one proposed decision (DEC-001).
- `check --all` searches `code` and `not` names as whole identifiers. It skips retired terms, and their `avoid` words give no warnings.
- Impact from code names is not implemented. "Term matching" is listed under "Ideas for v2" in `docs/Statute-spec.md` section 7. A term is affected only when a decision that lists it is added or changed.

PR #48 (Bean st-8j2x, decision D261115-ywulh) is merged and approved. It changed CORE-003 and the terms Manual check and Build state. No proposed decision holds them now, so DEC-001 does not block any migration below. Its effect on wording is covered under "Code link and Test link into Link".

### Retirement steps (current contract)

Source: `docs/Statute-spec.md` section 4.3, rules DEC-004, DEC-005 and HIST-005.

1. Record one decision with `--terms SURVIVOR --retires-terms OLD --changes RULES`. RULES are every rule that is not retired and lists OLD. Only an agreed term can be retired, and the decision cannot also list it in `--terms`.
2. Edit the survivor's definition and `code`, and replace OLD with SURVIVOR in each rule's `terms`. Leave OLD's section unchanged. OLD stays `agreed: agreed`, and its references stay valid, until approval.
3. A person approves. `statute approve` sets OLD to `agreed: retired`. It refuses while any rule that is not retired still lists OLD, so the rule changes must be in the same `approve` call.
4. After approval: OLD keeps its heading, definition, `code`, `not` and `agreed: retired` for ever. Its name is never reused. `show OLD` still finds it, labels it retired, and lists no rules. `check --all` no longer searches its `code` and `not` names, so any name that still matters must move to the survivor. There is no replacement field; the decision's summary or rationale names the successor.

Prototype: in a disposable clone of `b8df322` (scratchpad, never pushed), recording `--retires-terms "Code link,Test link"` without changing CORE-003 and LINKS-002 was accepted by `decide`, and then `approve` refused with four `rule.term_retired` errors and wrote nothing. This confirms step 3. All `approve` runs in this record are simulations in that clone, not human approvals.

### Code link and Test link into Link: merge

The five questions from the proposal:

1. **Independent approval?** No evidence for it. Code link and Test link were defined once, in D261016-f5ieq, and never changed. Link changed twice without them (D261017-6x46f, D261108-4ssd6). The recent change to verification policy (D261115-ywulh) went into Manual check and Build state, not into the two roles. Merging does not serialize real work.
2. **Separate rule references?** No. Both are used by exactly CORE-003 and LINKS-002. Neither rule needs one role without the other.
3. **Independent retrieval, code names, naming guidance?** Each role has one code name. Neither has `not`, `aliases` or `avoid`. One flat `code` list keeps both names searched.
4. **One compact definition without losing distinctions?** Yes. The prototype definition below keeps every sentence of both roles. Cost: Link grows from 42 to 99 words, and LINKS-001, LINKS-003, LINKS-004 and LINKS-005 print the role text, which they do not need.
5. **Do references keep resolving?** Yes, through retirement. D261016-f5ieq keeps naming both retired terms, which DEC-005 allows.

Facts today:

| Term | Rules | `code` | `not` | `aliases` | `avoid` | Decisions |
| --- | --- | --- | --- | --- | --- | --- |
| Link | LINKS-001 to LINKS-005 | `Link`, `LinkLine` | none | none | none | f5ieq, 6x46f, 4ssd6 |
| Code link | CORE-003, LINKS-002 | `LinkRole::Code` | none | none | none | f5ieq |
| Test link | CORE-003, LINKS-002 | `LinkRole::Test` | none | none | none | f5ieq |

Proposed surviving definition (prototyped; the tracker paragraph is unchanged):

> A connection from a rule to a code or test target, with a fingerprint used to detect changes to that target. A code link points to the implementation that makes the rule true; a change there brings the rule up for review and tells the reviewer where to look. A test link points to a test that checks the rule's behavior; the test can catch a violation introduced outside the linked implementation, and the project's CI runs it.
>
> A reference to an issue in an external tracker, recorded in gaps.yml, is separate from a rule's code or test links.

`code: [Link, LinkLine, "LinkRole::Code", "LinkRole::Test"]`. No `not`, `aliases` or `avoid`. The retired sections keep their own `code` as a record.

Rule remapping:

| Rule | `terms` today | `terms` after |
| --- | --- | --- |
| CORE-003 | Rule, Build state, Code link, Test link, Manual check | Rule, Build state, Link, Manual check |
| LINKS-002 | Link, Stale link, Code link, Test link, Base | Link, Stale link, Base |

Decision: `statute decide --terms Link --retires-terms "Code link,Test link" --changes CORE-003,LINKS-002`.

Effect (measured in the prototype):

| Command | Before | After |
| --- | --- | --- |
| `show Link` | 5 rules | 6 rules (adds CORE-003) |
| `show "Code link"`, `show "Test link"` | CORE-003, LINKS-002 | Retired section, labelled retired, no rules. No redirect to Link. |
| `show CORE-003` | 529 words | 580 words |
| `show LINKS-002` | 355 words | 362 words |
| `show LINKS-003` (and LINKS-001, 004, 005) | Link at 42 words | Link at 99 words (+57 each) |
| `show src/state.rs` | 2,987 words | 3,218 words |
| `check --all` | No findings | No findings. `LinkRole::Code` and `LinkRole::Test` are still searched, now under Link. |

Wording interaction with PR #48: CORE-003's statement and cases, and the definitions of Manual check and Build state, say "code link" and "test link" in plain prose. After the merge, Link's definition defines both phrases, and CORE-003 lists Link, so `show CORE-003` still explains them. `show "Manual check"` and `show "Build state"` alone do not print Link, but that was already true for Code link and Test link, because prose is not expanded. No rule or term needs rewording for this merge.

Acceptance checks for the migration PR: an agent given CORE-003 still sees both roles, knows CI runs linked tests, and knows a manual check never replaces a code link. An agent given LINKS-002 sees that stale applies to both roles.

### Proposed decision into Decision: keep both

This reverses the audit's first-batch recommendation. The evidence changed after the audit.

1. **Independent approval?** Yes, and this question decides the choice. Proposed decision was introduced in D261017-rlpdd and changed in its meaning once without Decision: D261018-m21p6 added that a retiring term stays agreed and in force until approval. D261028-qy5et also changed it, but only to rename "ratify" to "approve", so it is weak evidence. Decision changed only once after adoption, for code names (D261108-4ssd6). Merged, each future change to the proposal lifecycle would hold Decision in a proposed decision, blocking any other change to it (DEC-001), and every term-impact report would name Decision.
2. **Separate rule references?** Partly. DEC-001, DEC-002, DEC-004 and DEC-005 list Proposed decision. Four rules that list only Decision also use proposal semantics in their prose or cases: CORE-005 ("approve a proposed decision"), HIST-001 (a proposed decision can be edited or deleted), HIST-004 and HIST-005 (a rule or term that a proposed decision retires stays agreed until approval). DEC-003, HIST-002, HIST-003 and LINKS-003 do not depend on the lifecycle.
3. **Independent retrieval?** Mixed. The prototype Decision grows from 26 to 79 words and prints in 12 rule views. For the four rules named in question 2 the merge would help, because prose is not expanded (the same reason given for the Affected rule merge). For DEC-003, HIST-002, HIST-003 and LINKS-003 it adds text they do not need. Questions 2 and 3 do not settle the choice on their own.
4. **One compact definition?** Possible (prototyped below), but it bundles the most-changed lifecycle text into a central term.
5. **References keep resolving?** Yes, through retirement; this is not the blocker.

Facts today:

| Term | Rules | `code` | `not` | `aliases` | `avoid` |
| --- | --- | --- | --- | --- | --- |
| Decision | CORE-005, DEC-002 to DEC-005, HIST-001 to HIST-005, LINKS-003 (11) | `Decision`, `DecisionDraft` | none | none | none |
| Proposed decision | DEC-001, DEC-002, DEC-004, DEC-005 | `DecisionStatus::Proposed` | none | `pending decision` | none |

Prototype, for the record only (the tool accepted it and `check --all` stayed clean): `decide --terms Decision --retires-terms "Proposed decision" --changes DEC-001,DEC-002,DEC-004,DEC-005`, with Proposed decision replaced by Decision in DEC-001 and removed from DEC-002, DEC-004 and DEC-005. The decision changes the retirement rules DEC-004 and DEC-005 themselves. `show Decision` widened from 11 to 12 rules. `show DEC-001` grew from 346 to 377 words.

Class: keeps every term. No follow-up needed for the keep choice.

Option for the owner (not a recommendation): keep both terms and add Proposed decision to the `terms` of CORE-005, HIST-001, HIST-004 and HIST-005, through one decision with `--changes` for those four rules. This gives those rule views the lifecycle definition without widening Decision. It keeps every term. File a Bean for it only if the owner wants it.

### Affected rule into Rule review: merge, lower priority

1. **Independent approval?** No. Both were defined in D261016-f5ieq and changed together in D261108-4ssd6.
2. **Separate rule references?** No rule lists Affected rule.
3. **Independent retrieval?** Rule review's definition already starts with "whether an affected rule still holds", but LINKS-003 and LINKS-004 do not list Affected rule, so their views use the phrase without defining it. The merge makes Rule review self-contained. Cost: +28 words in two rule views.
4. **Code names?** `RuleImpact` moves to Rule review. It names the impact result, not the review, so attribution gets a little looser. It is still searched.
5. **References keep resolving?** Yes. D261016-f5ieq and D261108-4ssd6 keep naming the retired term.

| Term | Rules | `code` | `not` | `aliases` | `avoid` |
| --- | --- | --- | --- | --- | --- |
| Rule review | LINKS-003, LINKS-004 | `review::Verdict`, `ReviewReport` | none | none | none |
| Affected rule | none | `RuleImpact` | none | none | none |

Proposed surviving definition (prototyped):

> An assessment by the agent of whether an affected rule still holds after a change. A rule is affected when a change brings it up for review; being affected means the rule needs attention and does not itself establish that it is broken. The agent reads the rule and implementation, tries the protected and boundary cases, and records whether the rule holds, changes on purpose, or is broken. The tool finds changed links and performs mechanical checks; it does not judge the behavior. A person performs any manual check and approves decisions to change the rule.

`code: ["review::Verdict", ReviewReport, RuleImpact]`. Rule remapping: none.

Decision: `statute decide --terms "Rule review" --retires-terms "Affected rule"`. In the prototype `approve` accepted it with no rule changes, and `check --all` stayed clean.

Effect: `show "Rule review"` still returns LINKS-003 and LINKS-004. `show "Affected rule"` returns the retired section with no rules (it had none before). Rule review grows from 68 to 96 words.

Risk: if a later rule is about impact only, it would have to list Rule review and carry the review text. Re-check this choice if such a rule is proposed.

### Protected case and Boundary case into Rule: keep

1. **Independent approval?** They changed only at adoption. No evidence either way.
2. **Separate rule references?** Only CORE-001 lists them. Rule is listed by 10 rules.
3. **Independent retrieval?** No gain. Merging adds about 30 to 46 words to Rule (the two definitions are 46 words; a compact version is about 30) in 10 rule views to remove two entries used by one rule.
4. **Code names?** `protects` and `boundary` are distinct field names. Each now maps to one meaning; a merged list would lose that.
5. **References keep resolving?** It would work through retirement, but the cost in question 3 decides it.

The audit itself said that keeping both is better than a too-broad Rule. Class: keeps every term.

### Terms kept as they are

| Term | Reason to keep |
| --- | --- |
| Term, Rule | Core reference and approval objects. |
| Decision | Kept apart from Proposed decision (see above). |
| Manual check | A human procedure, not a link target. Its policy changed on its own in D261115-ywulh. |
| Gap | Own lifecycle and remedy. |
| Agreement status | Shared by rules and terms; changes apart from build state (CORE-002). Now also covers retired terms. |
| Build state | Separate from agreement; changed on its own in D261115-ywulh. |
| Approval | Human authority event, apart from the agent recording it. |
| Base | Specific fallback and working-tree rules a plain Git word does not carry. |
| Stale link, Missing link | Different conditions and remedies; precise code names. |
| Fingerprint | Mode-dependent hashing; owns `avoid: [checksum]`. |
| Draft rule | Not every proposed rule; pending, not a finding. |

### Order and interaction with PR #48

1. **Link merge** first. PR #48 is merged, so CORE-003 is free. Do not run it while another proposed decision lists CORE-003, LINKS-002, Link, Code link or Test link; `decide` would refuse it (DEC-001).
2. **Rule review merge** second, or at the same time. It touches no rule and shares no term with the Link merge, so the two decisions do not overlap.
3. No step for Proposed decision, Protected case or Boundary case.

### Follow-up Beans to file after approval

Do not file these until the owner approves the choices above.

- **Merge Code link and Test link into Link:** one proposed decision that rewords Link, adds both `LinkRole` names to its `code`, retires Code link and Test link, and changes CORE-003 and LINKS-002, with the before/after `show` comparison in the PR.
- **Merge Affected rule into Rule review:** one proposed decision that adds the affected-rule sentence to Rule review, moves `RuleImpact` to its `code`, and retires Affected rule.

### Owner decisions (2026-11-15)

- Approved and done: the Link merge (Bean st-jlyi, decision D261115-8tnxv) and the Rule review merge (Bean st-2lxt, decision D261115-hy140). Code link, Test link and Affected rule are now retired.
- Still to discuss: keeping Proposed decision, keeping Protected case and Boundary case, and the optional reference addition (items 2, 4 and 5 below).

### What the owner needs to decide

1. Accept the Link merge, including that `show Link` widens to CORE-003 and that four LINKS rules print the role text.
2. Accept keeping Proposed decision, against the audit's first-batch recommendation.
3. Accept the lower-priority Rule review merge, or keep Affected rule.
4. Accept keeping Protected case and Boundary case.
5. Optional: whether to add Proposed decision to the `terms` of CORE-005, HIST-001, HIST-004 and HIST-005 (see the Proposed decision section). If yes, file a Bean for it.

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
