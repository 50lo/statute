---
name: change
description: Run after any code change in a repository that uses Statute (it has statute.yml), before declaring the task done. Reviews the rules whose linked code changed and records a verdict for each.
---

# Statute: change

Run this after changing code, and whenever `statute check` reports stale or missing links.

First run `statute --version`. If the command is missing, stop and tell the user to install the CLI with `cargo install --git https://github.com/50lo/statute`.

## Steps

1. Run `statute check --json`. Findings with code `link.stale` or `link.missing` name a `rule` and a `target`. Fix any other errors it reports first.
2. For each `link.missing` target: if the code moved or was renamed, find its new location, then run `statute link RULE NEW_TARGET` (add `--test` for a test link) and `statute link RULE OLD_TARGET --remove`. Relinking says the rule still holds, so check the rule as in step 3 before you relink. If the file no longer parses, fix the file.

   If the target was deleted and nothing replaces it, `review` refuses while the link is missing. Check the rule as in step 3 against what remains, then run `statute link RULE OLD_TARGET --remove` and record the outcome:
   - The behaviour still exists elsewhere: link the code or test that has it now.
   - The behaviour was removed on purpose: use the `decide` skill to propose changing or retiring the rule.
   - The behaviour was lost by mistake: open a gap (below).

   Removing a rule's last code or test link leaves a built rule incomplete, and `check` reports it until the rule is relinked, changed or retired. Leave that finding visible and tell the user.
3. For each rule with stale or missing links, run `statute show RULE` and read its statement, terms, `protects` and `boundary` cases. Read the changed code at every linked target. Try each protected and boundary case against the new code. Then decide which one applies:
   - **Holds**: every case still behaves as the rule says. Run `statute review RULE ok`.
   - **Changing on purpose**: the user asked for behaviour that the rule forbids. Name the rule to the user, use the `decide` skill to propose the change, then run `statute review RULE changed D…` with the new decision's ID.
   - **Broken**: the code no longer does what the rule says, and nobody asked for that. Open a gap (below), then run `statute review RULE gap REF`. Tell the user, and fix the code if that is in scope.
4. Reconcile tracker gaps. Run `statute status --json` and look at each rule's `open_gaps`. For each open gap that belongs to the issue tracker named in AGENTS.md (its ID is the tracker's, not `G…`), look up the issue with the tracker's tools. If the issue is closed, check the rule against the current code as in step 3, and run `statute gap close REF` only if the rule holds; otherwise tell the user the issue is closed but the rule still does not hold.
5. If you fixed the code for an open gap, check the rule as in step 3 and run `statute gap close REF`.
6. Run `statute check` again until it reports no stale or missing links.

## Opening a gap

- With an issue tracker named in AGENTS.md: open an issue with the tracker's own tools, tag it `statute` and `gap`, and run `statute gap open RULE --ref ISSUE_ID`.
- Without one: run `statute gap open RULE --note "what no longer holds"`. It prints the gap ID.

## Ends with

- One `statute review` per affected rule, plus any decision or gap it needed.
- `statute check --md` pasted into the PR description, as the "Rule impact" section. Add a sentence per affected rule if the reason is not obvious, written so a PM or designer can follow it.

Never edit `agreed` or `built`, never change a rule's text to match the code, and never review a rule you did not check.
