---
name: decide
description: Use in a repository with statute.yml when a product choice passes the decision gates, or when a rule or an agreed term needs to change. Drafts the rules, terms and cases and records a proposed decision.
---

# Statute: decide

First run `statute --version`. If the command is missing, stop and tell the user to install the CLI with `cargo install --git https://github.com/50lo/statute`.

## When

- A rule or an agreed term has to change, or a new rule is needed.
- A choice passes the decision gates. If AGENTS.md has a decision policy, its gates apply. Otherwise a choice needs a decision when it changes behaviour that users or other teams rely on, is hard to reverse, or settles a question people disagreed about.

## Steps

1. Run `statute show` on the rules, terms and files involved, and read what it returns.
2. Write the decision's summary (one to three sentences) and rationale (why this option, and what was rejected).
3. Draft the rules and terms:
   - New rule: `statute rule new AREA "Title"` prints its ID. Write the statement, `scope`, `terms`, `protects` and `boundary` in the new section.
   - Changed rule: edit its statement and fields in place.
   - Retired rule: leave the text; the decision lists it in `retires`.
   - Term: add or edit its entry in `statute/terms.md`.
4. Check each rule before proposing it. What valid case would it reject? What invalid case would it allow? Does its meaning change with actor or permission, missing data, lifecycle stage, time, retries, concurrency or partial failure? Add only the answers that change the rule, as `protects` and `boundary` cases.
5. Propose a test for each rule that does not have one, and link it after writing it: `statute link RULE path/to/test#TestName --test`. Link the code that makes the rule true: `statute link RULE path/to/code#Symbol`. A rule an automated test cannot judge gets a `manual_check` instead of a test link.
6. Record the decision: `statute decide --summary "…" --rationale "…" [--establishes RULES] [--changes RULES] [--retires RULES] [--terms TERMS] [--actor user]`. Use `--actor user` when the user asked for the change. It prints the decision ID and sets the changed rules and terms to proposed.
7. Run `statute check` and fix what it reports.

## Ends with

A decision with `status: proposed` that lists its rules and terms. They stay proposed until a person approves the decision; use the `ratify` skill for that. Never run `statute ratify` on a decision you proposed without the person's explicit approval of that decision.
