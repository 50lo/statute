---
# st-rgyl
title: Merge Proposed decision into Decision
status: completed
type: task
priority: normal
tags:
    - statute
    - planning
created_at: 2026-11-15T03:37:05Z
updated_at: 2026-11-15T03:39:43Z
parent: st-uvvi
---

## What

Merge the agreed term Proposed decision into Decision through one proposed decision, and retire Proposed decision.

- Decision's definition becomes: "A recorded choice about rules or terms, with the reason for it. It starts as a proposed decision, which a person has not approved yet: the rules it establishes or changes and the terms it introduces or redefines stay proposed, while a rule or term it retires stays agreed and in force. A proposed decision can be edited or deleted, and it never blocks a merge. Once a person approves it, it becomes active and records who approved it."
- Decision's `code` becomes `[Decision, DecisionDraft, "DecisionStatus::Proposed"]`, and it gets `aliases: [pending decision]`.
- Only the `terms` lists of four rules change; statements and cases stay as they are:
  - DEC-001: `[Rule, Term, Proposed decision]` becomes `[Rule, Term, Decision]`.
  - DEC-002: `[Decision, Proposed decision, Agreement status, Approval]` becomes `[Decision, Agreement status, Approval]`.
  - DEC-004: `[Decision, Term, Agreement status, Proposed decision]` becomes `[Decision, Term, Agreement status]`.
  - DEC-005: `[Rule, Term, Decision, Proposed decision, Agreement status]` becomes `[Rule, Term, Decision, Agreement status]`.
- Leave the Proposed decision section as it is; the decision lists it in `retires_terms`.
- Command: `statute decide --actor user --terms Decision --retires-terms "Proposed decision" --changes DEC-001,DEC-002,DEC-004,DEC-005`.

## Why

The st-uvvi review record first recommended keeping both terms, because Proposed decision changed without Decision (D261018-m21p6). On 2026-11-15 the owner judged that such changes are rare, so the DEC-001 contention cost is small. The merge also gives CORE-005, HIST-001, HIST-004 and HIST-005, whose text already relies on proposal semantics, the definition in their rule views. Cost: Decision grows from 26 to 79 words in 12 rule views, and DEC-003, HIST-002, HIST-003 and LINKS-003 print text they do not need. Term definitions have no length limit. The owner approved this choice on 2026-11-15.

## Where

`statute/terms.md`, `statute/rules/decisions.md`, `statute/decisions.db` through `statute decide` and `statute approve`.

## Acceptance

- [x] One proposed decision lists Decision in `terms`, Proposed decision in `retires_terms`, and DEC-001, DEC-002, DEC-004 and DEC-005 in `changes`.
- [x] `statute check --all` reports no findings, before and after approval; `DecisionStatus::Proposed` is still searched, now under Decision.
- [x] A person explicitly approves the decision; then `statute approve` retires Proposed decision and agrees Decision and the four rules.

## Dependencies and constraints

- No other proposed decision may list Decision, Proposed decision or the four rules (DEC-001).
- Retirement contract: `docs/Statute-spec.md` section 4.3, rules DEC-004, DEC-005, HIST-005.
