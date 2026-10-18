---
name: ratify
description: Use in a repository with statute.yml when proposed decisions are pending or a person asks to approve rules, terms or decisions. Shows each pending decision for approval and records the person's answer.
---

# Statute: ratify

First run `statute --version`. If the command is missing, stop and tell the user to install the CLI with `cargo install statute-cli --locked`.

Only a person approves. Run `statute ratify` only for decisions the person in this session explicitly approved, one by one.

## Steps

1. Run `statute status --json` and read `impact.awaiting_approval`: each pending decision with its rules and terms. Run `statute show D…` for each.
2. For each pending decision, give the person:
   - two lines: what it decides and why, the rules and terms it lists, and the main risk if it is wrong;
   - each changed rule and term as a diff against its agreed text, and each one that was never agreed as an addition.
3. The agreed text comes from git. For a rule or term, walk back through the commits that changed its file (`git log --format=%H -- FILE`, newest first) and take its text from the most recent commit where it read `agreed: agreed` (`git show COMMIT:FILE`). That commit can be older than the branch, because a proposal can already be on the default branch.
4. Ask the person to approve, reject or skip each decision.
5. Approved decisions: `statute ratify D… [D…]`. The CLI sets the decisions active, records the person from git `user.name` and `user.email`, and updates the rules and terms. If it refuses, show the errors to the person; nothing was ratified.
6. Rejected decisions, for each one:
   - Delete its entry from `statute/decisions.db`.
   - For each rule in its `changes` that was agreed before: restore the statement and the compared fields (`scope`, `terms`, `exceptions`, `protects`, `boundary`, `manual_check`) to the agreed text, and set `agreed: agreed`. Leave `built`, notes and links as they are.
   - For each term in its `terms` that was agreed before: restore its whole definition (everything in its section except the heading and the `yaml` block; terms have no notes), `code` and `not` to the agreed text, and set `agreed: agreed`.
   - For each rule in its `establishes`: remove its links with `statute link RULE TARGET --remove`, then remove the rule's section. Remove each term the decision introduced.
   - Run `statute check` until no reference to the removed items remains.
7. Skipped decisions stay proposed and can be reworked with the `decide` skill.

## Ends with

`statute ratify` for the approved decisions, the rejected ones removed, and `statute check` passing.
