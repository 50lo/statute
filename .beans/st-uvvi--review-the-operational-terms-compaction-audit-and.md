---
# st-uvvi
title: Review the operational terms compaction audit and choose migrations
status: todo
type: task
priority: normal
tags:
    - statute
    - planning
created_at: 2026-10-18T12:07:14Z
updated_at: 2026-10-18T12:07:14Z
---

## What

Review the existing audit of Statute's operational terms (`statute/terms.md`, 21 terms on 2026-10-18) and, for each candidate it raises, choose keep, merge, split or reword, with the reasoning and the migration it implies. The output is a reviewed record of migration choices for a person to approve. This Bean changes no term, rule or decision; each chosen migration is its own later PR.

Input: an audit saved for later human review outside the repository while PR #20 was prepared (https://github.com/50lo/statute/pull/20). Its location is not recorded; ask Anton Solo for it before starting rather than redoing the audit.

Plan: "Item 3" in docs/Terms-refactoring-plan.md. Criteria: "Reviewing operational terms for compaction" in docs/Terms-refactoring-proposal.md. There is no target term count, and `terms.md` is judged as operational vocabulary, not learning material.

## Why

Separating the human glossary (st-25bl) from the operational terms does not mean every current entry is well chosen. Entries whose separate identities add no operational precision cost agents context and people review effort; entries that combine meanings needing independent approval or retrieval should be split.

## Where

- Read: `statute/terms.md`, the `terms` lists in `statute/rules/*.md`, `statute show TERM` for each candidate (its rules), and `statute check --all` for code-name hits.
- Write: a review record, as a document in `docs/` or in this Bean, whichever the reviewer prefers. No changes to `statute/` files.

## Acceptance

- [ ] Every candidate in the audit has a choice (keep, merge, split, reword) answered against the five questions in the proposal's compaction section.
- [ ] For each merge or split: the rules that reference the affected terms, their `code` and `not` names, and the expected effect on `statute show` and `statute check --all` results (does the merge widen what they return?).
- [ ] Each migration is classed as keeping every existing term (can proceed after review, through proposed decisions) or removing/retiring a term (waits for st-9flm).
- [ ] A person has reviewed the record; follow-up Beans are filed for the approved migrations.

## Dependencies and constraints

- Can start now; it needs no code and does not wait for st-jycx, st-25bl or st-9flm.
- Migrations that follow: every change to an agreed term goes through `statute decide` and explicit human approval; a migration that removes or retires a term waits for st-9flm; once st-25bl has landed, each migration PR includes a glossary consistency record.
