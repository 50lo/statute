---
# st-e5dc
title: Report an unreadable base instead of skipping history checks
status: todo
type: bug
priority: high
created_at: 2026-10-28T10:16:48Z
updated_at: 2026-10-28T10:16:48Z
---

When the base `statute/decisions.db` does not parse, `check` silently skips the history comparison of decisions, agreed rules and agreed terms. `check::history` returns early when `base.decisions` is `None` (src/check/history.rs), and `Repo::load_at` leaves it `None` on a parse error. Only the retired rule and term checks still run. `check` then reports no history findings, so an edited active decision or an agreed rule changed without a decision goes unnoticed. This conflicts with decision-making principle 1 (a missed behaviour change is worse than a false alarm).

Found in PR #26 (rename `ratified_by` to `approved_by`): the new parser rejected the old field in the base, and `check` reported no findings although eight agreed rules and two agreed terms had changed. A temporary build that accepted the old field showed the expected findings.

Expected: when the base exists but one of its Statute files does not parse, `check` reports a finding (for example `base.unreadable`, naming the file and revision) instead of passing silently. Decide whether it is an error or a warning, and cover it with a test.
