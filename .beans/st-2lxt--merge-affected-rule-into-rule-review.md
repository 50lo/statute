---
# st-2lxt
title: Merge Affected rule into Rule review
status: completed
type: task
priority: normal
tags:
    - statute
    - planning
created_at: 2026-11-15T03:02:15Z
updated_at: 2026-11-15T03:03:19Z
parent: st-uvvi
---

## What

Merge the agreed term Affected rule into Rule review through one proposed decision, and retire Affected rule.

- Add to Rule review, after its first sentence: "A rule is affected when a change brings it up for review; being affected means the rule needs attention and does not itself establish that it is broken."
- Rule review's `code` becomes `["review::Verdict", ReviewReport, RuleImpact]`.
- No rule lists Affected rule, so no rule changes.
- Leave the Affected rule section as it is; the decision lists it in `retires_terms`.
- Command: `statute decide --actor user --terms "Rule review" --retires-terms "Affected rule"`.

The full analysis is in st-uvvi, section "Review record (2026-11-15)", part "Affected rule into Rule review".

## Why

No rule lists Affected rule, and it always changed together with Rule review. Rule review's definition already uses the phrase "affected rule", but rule views do not expand prose, so LINKS-003 and LINKS-004 use it without a definition. The merge makes Rule review self-contained. The owner approved this choice on 2026-11-15.

## Where

`statute/terms.md`, `statute/decisions.db` through `statute decide` and `statute approve`.

## Acceptance

- [x] One proposed decision lists Rule review in `terms` and Affected rule in `retires_terms`.
- [x] `statute check --all` reports no findings, before and after approval; `RuleImpact` is still searched, now under Rule review.
- [x] A person explicitly approves the decision; then `statute approve` retires Affected rule and agrees Rule review.

## Dependencies and constraints

- No other proposed decision may list Rule review or Affected rule (DEC-001). It shares no term or rule with the Link merge, so both can be pending together.
- Retirement contract: `docs/Statute-spec.md` section 4.3, rules DEC-004, DEC-005, HIST-005.
