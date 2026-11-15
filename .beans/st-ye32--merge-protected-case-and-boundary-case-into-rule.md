---
# st-ye32
title: Merge Protected case and Boundary case into Rule
status: in-progress
type: task
tags:
    - statute
    - planning
created_at: 2026-11-15T03:49:48Z
updated_at: 2026-11-15T03:49:48Z
parent: st-uvvi
---

## What

Merge the agreed terms Protected case and Boundary case into Rule through one proposed decision, and retire the two case terms.

- Rule's definition becomes: "A statement of one thing that must be true about the product, and when. A person or test can check whether the product follows it. Its protected cases are concrete cases showing behavior it must preserve; they give a person or agent cases to try when checking whether it still holds. Its boundary cases are nearby cases showing where it stops applying, so it does not reject valid behavior outside its intended scope."
- Rule's `code` becomes `[Rule, RuleMeta, protects, boundary]`.
- CORE-001 `terms`: `[Rule, Protected case, Boundary case]` becomes `[Rule]`. Its statement and cases stay as they are.
- Leave the Protected case and Boundary case sections as they are; the decision lists them in `retires_terms`.
- Command: `statute decide --actor user --terms Rule --retires-terms "Protected case,Boundary case" --changes CORE-001`.

## Why

The st-uvvi review record first recommended keeping both terms, because only CORE-001 lists them and the merge adds text to Rule in 10 rule views. On 2026-11-15 the owner chose to merge: both case terms describe fields of a rule, have one code name each, and have not changed since adoption. Cost: Rule grows from 25 to 73 words in the views of CORE-001 to CORE-004, DEC-001, DEC-003, DEC-005, GAPS-001, HIST-002 and HIST-004. Term definitions have no length limit.

## Where

`statute/terms.md`, `statute/rules/core.md` (CORE-001), `statute/decisions.db` through `statute decide` and `statute approve`.

## Acceptance

- [ ] One proposed decision lists Rule in `terms`, Protected case and Boundary case in `retires_terms`, and CORE-001 in `changes`.
- [ ] `statute check --all` reports no findings, before and after approval; `protects` and `boundary` are still searched, now under Rule.
- [ ] A person explicitly approves the decision; then `statute approve` retires Protected case and Boundary case and agrees Rule and CORE-001.
- [ ] `statute show CORE-001` prints Rule with both case meanings, and they stay distinct: a protected case is behavior to preserve, a boundary case is nearby valid behavior outside scope.

## Dependencies and constraints

- No other proposed decision may list Rule, CORE-001, Protected case or Boundary case (DEC-001).
- Retirement contract: `docs/Statute-spec.md` section 4.3, rules DEC-004, DEC-005, HIST-005.
