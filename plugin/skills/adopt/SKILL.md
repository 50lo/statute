---
name: adopt
description: Use when setting Statute up in a repository. Selects durable rules and terms from existing docs, contracts, ADRs and decision logs, linked to code and tests, for a person to approve.
---

# Statute: adopt

First run `statute --version`. If the command is missing, stop and tell the user to install the CLI with `cargo install statute-cli --locked`.

## Steps

1. If the repository has no `statute.yml`, run `statute init`. It writes the layout and adds the Statute section to AGENTS.md.
2. Read the existing docs, contracts, ADRs and decision logs. Apply the rule and term criteria in the `decide` skill before drafting. For each area with qualifying rules and no rule file, create `statute/rules/<area>.md` with a `prefix` in its frontmatter:

   ```markdown
   ---
   prefix: BILL
   ---
   ```

3. Draft qualifying terms in `statute/terms.md` with their definitions, the code names that implement them (`code`) and the look-alike names that do not (`not`).
4. Draft the rules per area. For each rule run `statute rule new AREA "Title"`, then write one statement of one thing that must be true, its `scope`, its `terms`, and the `protects` and `boundary` cases. Keep each rule small enough for one test to check it.
5. Link each rule to the code that makes it true and to a test that checks it: `statute link RULE path#Symbol` and `statute link RULE path#TestName --test`. A rule that no automated test can judge gets a `manual_check` instead of a test link. Write missing tests, or list them for the person.
6. Record one decision per area with qualifying rules or terms. Run `statute decide --summary "Adopt the <area> rules and terms." --rationale "…"`, adding `--establishes RULES` for rules and `--terms TERMS` for terms when present.
7. Run `statute check` and fix what it reports until it passes.

## Ends with

One proposed decision per area with qualifying rules or terms, if any, `statute check` passing, and a list for the person to approve with the `approve` skill: per area, its rules and terms in one line each.
