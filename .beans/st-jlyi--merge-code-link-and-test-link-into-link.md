---
# st-jlyi
title: Merge Code link and Test link into Link
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

Merge the agreed terms Code link and Test link into Link through one proposed decision, and retire the two role terms.

- Reword Link's definition so it states both roles (the sentences of Code link and Test link, kept as they are). Keep Link's tracker paragraph unchanged.
- Link's `code` becomes `[Link, LinkLine, "LinkRole::Code", "LinkRole::Test"]`.
- CORE-003 `terms`: `[Rule, Build state, Code link, Test link, Manual check]` becomes `[Rule, Build state, Link, Manual check]`.
- LINKS-002 `terms`: `[Link, Stale link, Code link, Test link, Base]` becomes `[Link, Stale link, Base]`.
- Leave the Code link and Test link sections as they are; the decision lists them in `retires_terms`.
- Command: `statute decide --actor user --terms Link --retires-terms "Code link,Test link" --changes CORE-003,LINKS-002`.

The full analysis, proposed wording and measured effect are in st-uvvi, section "Review record (2026-11-15)", part "Code link and Test link into Link".

## Why

Both role terms are used by exactly the same two rules, have not changed since adoption, and have one code name each. Their separate identities add no operational precision. The owner approved this choice on 2026-11-15.

## Where

`statute/terms.md`, `statute/rules/core.md` (CORE-003), `statute/rules/links.md` (LINKS-002), `statute/decisions.db` through `statute decide` and `statute approve`.

## Acceptance

- [x] One proposed decision lists Link in `terms`, Code link and Test link in `retires_terms`, and CORE-003 and LINKS-002 in `changes`.
- [x] `statute check --all` reports no findings, before and after approval.
- [x] A person explicitly approves the decision; then `statute approve` retires Code link and Test link and agrees Link, CORE-003 and LINKS-002.
- [x] `statute show CORE-003` prints Link with both roles; `statute show "Code link"` shows a retired term with no rules.

## Dependencies and constraints

- No other proposed decision may list CORE-003, LINKS-002, Link, Code link or Test link (DEC-001).
- Retirement contract: `docs/Statute-spec.md` section 4.3, rules DEC-004, DEC-005, HIST-005. `approve` refuses while a live rule still lists a retiring term, so the rule changes go in the same decision.
