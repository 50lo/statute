---
# st-jycx
title: Protect whole term sections and retired items in history checks
status: todo
type: bug
priority: high
tags:
    - statute
created_at: 2026-10-18T10:50:00Z
updated_at: 2026-10-18T12:09:14Z
blocking:
    - st-25bl
    - st-9flm
---

History checks leave two kinds of approved content unprotected. The behaviour to implement was chosen on 2026-10-18 by Anton Solo. Plan, steps and the decisions to propose: docs/Terms-refactoring-plan.md, "Choices made before item 1" and "Item 1".

## Term paragraphs after the definition

Observed (2026-10-18):

- `check::history::same_term` compares a term's first paragraph, `code` and `not`. Later paragraphs in the term's section can change without a decision.
- `statute show TERM` prints the whole section, later paragraphs included (`TermShow.text` in `src/commands/show.rs`). The earlier version of this Bean said `show` drops them; that is true only of the rule and file views.
- `statute show RULE` and `statute show FILE` list each rule's terms with only the first paragraph (`TermRef.definition` in `src/commands/show.rs`). An agent reading the rules for the code it will touch never sees a later paragraph.
- Statute's own Link term has such a paragraph, separating a tracker reference in `gaps.yml` from a rule's code and test links. It qualifies Link's meaning.

The human glossary proposal (docs/Terms-refactoring-proposal.md) forbids the glossary from introducing a condition absent from the approved definition, so what counts as approved must be unambiguous before the glossary is drafted (st-25bl).

Chosen: everything in a term section after the heading, except the `yaml` block, is the definition: paragraphs, lists, tables and any other code block. All of it is compared with whitespace collapsed, and the rule and file views of `show` print all of it. Do not compare the raw section span: `aliases`, `avoid`, `agreed` and YAML comments stay uncompared, as HIST-003's boundary case requires. Terms no longer have notes. Rejected alternative: treat later paragraphs as uncompared notes, like rule notes.

## Retired rules

Observed (2026-10-18):

- `agreed_rule` in `src/check/history.rs` selects only rules that read `agreed: agreed` in the base. A rule that is `agreed: retired` in the base can be edited or deleted without a finding, although spec 4.1 says it keeps its section, its ID must not be reused, and active decisions still reference it.
- `statute decide --changes RULE` accepts a retired rule and sets it to `agreed: proposed` (`src/commands/decide.rs`), which revives it.

Chosen: retirement is final. A rule retired in the base keeps its section, its compared fields and `agreed: retired`, whatever decision lists it. New behaviour gets a new rule ID. `decide` refuses `--changes` and `--retires` for a retired rule and writes nothing. Record this as a new history rule rather than widening HIST-002, because HIST-002 lets a decision carry a change and this rule lets none. Use a new finding code, not `rule.changed_without_decision`, whose message tells the reader to run `decide --changes`.

Retired terms (st-9flm) reuse this check later.

## Where

- docs/Statute-spec.md: 4.1, 4.3, and the `decide` row of the command table in 5.
- src/check/history.rs, src/commands/decide.rs, src/store/terms.rs, src/model/mod.rs (`Term`), src/commands/show.rs.
- statute/rules/history.md (HIST-003 change, new retired-rule rule), plugin/skills/ratify/SKILL.md (rejection restores a term's whole definition).
- tests/history_contract.rs, tests/decisions_contract.rs, and any tests/snapshots/ file whose `show` output changes.

## Acceptance

- [ ] Proposed decisions recorded with `statute decide` before code changes: one with `--changes HIST-003` (protected case for a later paragraph; its summary says that Link's paragraph after the `yaml` block, present when D261017-6x46f approved Link, now counts as part of Link's definition), one establishing the new retired-rule rule. Both stay proposed until a person explicitly approves them.
- [ ] Spec 4.3 states that everything in a term section after the heading except the `yaml` block is the definition and is compared; spec 4.1 states that a rule retired in the base keeps its section, compared fields and `agreed: retired`; spec 5 lists the new `decide` refusal.
- [ ] Changing any text of an agreed term outside its `yaml` block with no decision is `term.changed_without_decision`, including a paragraph or list item after the block. Editing `aliases`, `avoid` or a YAML comment gives no finding.
- [ ] For a rule retired in the base, deleting it, changing its statement or a compared field, or changing its `agreed` value is a finding, even when a decision lists it in `changes`. Editing its notes is not.
- [ ] `statute decide --changes RULE` and `--retires RULE` refuse a retired rule and write nothing.
- [ ] `statute show` on a rule that uses a term with a later paragraph prints that paragraph in the rule's term list, in text and `--json`.
- [ ] Link's text is unchanged; `cargo test` and `statute check --all` pass; the `change` skill ran and `statute check --md` is in the PR.
