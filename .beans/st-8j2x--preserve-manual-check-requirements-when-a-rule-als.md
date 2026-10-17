---
# st-8j2x
title: Preserve manual check requirements when a rule also has automated tests
status: todo
type: bug
priority: high
created_at: 2026-10-17T00:30:58Z
updated_at: 2026-10-17T00:30:58Z
---

A rule can contain behavior that automated tests can verify and behavior that needs a person to judge. Linking its mechanical tests must not hide or discharge its manual check requirement.

Statute currently describes a manual check as a replacement for a test link (docs/Statute-spec.md, CORE-003 and the Manual check term). src/state.rs::is_complete uses code && (test || manual_check). The current status report also does not expose manual check instructions. This leaves mixed verification unclear and makes partial test links easy to mistake for coverage of the whole rule.

Examples from Statute's own rules:
- CORE-001: tests can require protected and boundary case metadata; a person must judge whether the rule is one meaningful, checkable requirement.
- CORE-004: tests can verify proposed defaults and lifecycle transitions; a person must check the agent's edits and how it presents them.
- CORE-005: tests can verify that ratify affects only selected decisions; a person must confirm explicit human approval of those particular decisions.

Partial test links for these rules are deferred until this bug is fixed. The tests remain available in tests/core_rule_cases.rs and tests/core_lifecycle.rs.

Acceptance criteria:
- [ ] Define mixed automated/manual verification in a proposed Statute decision and update the affected spec, rules and terms before changing their meaning.
- [ ] A rule with both test links and manual_check still visibly requires its stated manual check. Automated checks must not imply that the entire rule has been verified.
- [ ] Show the manual instructions in status and carry the manual requirement through change-review output, including when automated tests are linked.
- [ ] Add regressions for automated-only, manual-only and mixed rules, including a mixed rule whose automated tests pass while manual review is still needed. Preserve the distinction between declared verification coverage and evidence that behavior holds.
- [ ] Link the deferred mechanical tests to CORE-001, CORE-004 and CORE-005 once mixed verification is represented correctly; do not mark their human judgment as automated coverage.

Keep the existing requirement for a code link. This bug does not ask the CLI to infer human consent from Git identity or from a passing test suite.
