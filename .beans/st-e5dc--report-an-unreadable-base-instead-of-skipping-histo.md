---
# st-e5dc
title: Report an unreadable base instead of skipping history checks
status: completed
type: bug
priority: high
created_at: 2026-10-28T10:16:48Z
updated_at: 2026-11-06T10:13:31Z
---

When the base `statute/decisions.db` does not parse, `check` silently skips the history comparison of decisions, agreed rules and agreed terms. `check::history` returns early when `base.decisions` is `None` (src/check/history.rs), and `Repo::load_at` leaves it `None` on a parse error. Only the retired rule and term checks still run. `check` then reports no history findings, so an edited active decision or an agreed rule changed without a decision goes unnoticed. This conflicts with decision-making principle 1 (a missed behaviour change is worse than a false alarm).

Found in PR #26 (rename `ratified_by` to `approved_by`): the new parser rejected the old field in the base, and `check` reported no findings although eight agreed rules and two agreed terms had changed. A temporary build that accepted the old field showed the expected findings.

Expected: when the base exists but one of its Statute files does not parse, `check` reports a finding (for example `base.unreadable`, naming the file and revision) instead of passing silently. Decide whether it is an error or a warning, and cover it with a test.

## Summary of Changes

`check` now reports each base-load diagnostic as an error with code `base.unreadable`, preserving its file, line and item context and including the base revision and original diagnostic. It exits 1 because an incomplete comparison must not appear successful. Current-file checks and comparisons using readable base data continue. Markdown output includes the diagnostics before its potentially incomplete impact report.

Added regressions for the old `ratified_by` schema at the actual merge-base, malformed configuration/decisions/terms/rules/gaps/lock files, text/JSON/Markdown output, continued current and history findings, a base before Statute and a repository without commits. Documented the behavior in the specification.

Validation: reproduced failures before the fix; `cargo test --locked`, `cargo fmt --check`, and `cargo clippy --all-targets --locked -- -D warnings` pass. The built CLI's `check --all --json` reports no findings. Reviewed HIST-001 through HIST-005 and LINKS-002 against their protected and boundary cases and refreshed their links; no open tracker gaps were registered.
