---
# st-jycx
title: Protect whole term sections and retired items in history checks
status: todo
type: bug
priority: high
tags:
    - statute
created_at: 2026-10-18T10:50:00Z
updated_at: 2026-10-18T10:50:00Z
---

History checks leave two kinds of approved content unprotected.

## Term paragraphs after the definition

`check::history::same_term` compares a term's first paragraph, `code` and `not`. Later paragraphs in the term's section can change without a decision, and `show` does not return them (`src/commands/show.rs`, the term's `text` holds the heading, definition and block). Statute's own Link term has such a paragraph, separating a tracker reference from a rule's code and test links, which qualifies its meaning.

The human glossary proposal (docs/Terms-refactoring-proposal.md) forbids the glossary from introducing a condition absent from the approved definition, so what counts as approved must be unambiguous before the glossary is drafted.

Choose one and update docs/Statute-spec.md 4.3:

- Compare the whole term section (definition and later paragraphs, whitespace collapsed), and return the whole section from `show`.
- Or define later paragraphs as notes that may not qualify the meaning, and move any qualifying text into the definition, through a decision.

## Retired items

`agreed_rule` and `agreed_term` select only items that read `agreed: agreed` in the base. A rule that is `agreed: retired` in the base can be edited or deleted without a decision, although its ID must not be reused and active decisions still reference it. Extend the history check to rules retired in the base: their sections stay, and their compared fields do not change. Retired terms (st-9flm) get the same treatment.

## Acceptance

- [ ] Spec states which parts of a term section are compared.
- [ ] A change to the compared part of an agreed term without a decision is a finding; a change to anything defined as notes is not.
- [ ] Deleting a rule retired in the base, or changing its compared fields, is a finding.
- [ ] Existing Statute terms and rules still pass `statute check`; any term whose later paragraph qualifies its meaning is handled through a proposed decision.
