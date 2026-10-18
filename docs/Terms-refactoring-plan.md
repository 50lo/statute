# Terms refactoring: implementation plan

Status: in progress. Choices A and B below were made on 2026-10-18; choice C is open. This plan orders and details the work that [the terms refactoring proposal](Terms-refactoring-proposal.md) accepted. It changes no agreed terms, rules or CLI behaviour. Each work item makes its own changes in its own PR, through proposed decisions where agreed items change.

Work items 1 and 2 are planned in detail here. Work items 3, 4 and 5 already have their own plans or need a human review first, so this document gives only their order, dependencies and the constraints that come from items 1 and 2.

## Order of work

| # | Work item | Tracker | Starts after | Can run alongside |
| --- | --- | --- | --- | --- |
| 1 | Compare and show everything in a term section except its `yaml` block, and protect retired rules in history checks | st-jycx (bug, high) | Now | 3a |
| 2 | Draft Statute's human glossary and start the manual consistency review | st-25bl (feature, high) | 1 merged | 3a, 4 |
| 3a | Review the compaction audit and choose migrations | st-uvvi (task) | Now; documents only | 1, 2, 4 |
| 3b | Compaction migrations that keep every existing term | One PR per migration | 3a reviewed | 4 |
| 3c | Compaction migrations that remove or retire a term | One PR per migration | 3a reviewed and 4 merged | |
| 4 | Term retirement | st-9flm (feature, high) | 1 merged | 2, 3a |
| 5 | Automated glossary/term consistency review | st-uzzw (feature) | Evidence from item 2's review records | |

Reasons for this order:

- The glossary may not add a condition that the approved definition lacks. Item 1 settles what the approved definition is, so it comes before item 2.
- Items 1 and 4 both change `src/check/history.rs`, and st-9flm's item 8 builds on item 1's protection of retired rules. They run one after the other.
- Items 2 and 4 touch different files. Item 2 changes `docs/` and `AGENTS.md`; item 4 changes `src/`, `docs/Statute-spec.md` and `plugin/`. They can run in parallel.
- Item 3a needs no code and can start now. Item 3b has no hard ordering against item 2: whichever lands second does the manual review that item 2 sets up.
- Item 5 has no date. It starts when the evidence named in the proposal appears (see item 5 below).

## Choices made before item 1

Anton Solo made choices A and B on 2026-10-18. Choice C is still open.

### A. Everything in a term section except its `yaml` block is the approved definition

Current behaviour, checked against the source on 2026-10-18:

- The history check compares a term's first paragraph (its definition), `code` and `not` (`same_term` in `src/check/history.rs`). Later paragraphs can change without a decision.
- `statute show TERM` prints the whole section, later paragraphs included (`TermShow.text`, `src/commands/show.rs`).
- `statute show RULE` and `statute show FILE` print each rule's terms with only the first paragraph (`TermRef.definition`, built in the rule view of `src/commands/show.rs`). An agent reading rules for the code it will touch never sees a later paragraph.
- Link is the only term with a later paragraph today. It separates a tracker reference in `gaps.yml` from a rule's code and test links, which qualifies Link's meaning.

st-jycx originally said `show` does not return later paragraphs. `show TERM` does; the rule and file views do not.

Chosen: everything in a term section after the heading, except the `yaml` block, is the definition: paragraphs, lists, tables and any other code block. All of it is compared, and the rule and file views of `show` print all of it.

- Spec 4.3: everything in the section after the heading except the `yaml` block is the definition, and all of it is compared.
- Code: `store::terms::parse` collects everything in the section after the heading except the `yaml` block. `same_term` compares it with whitespace collapsed, and the rule and file views of `show` print it. Do not compare the raw section span: that would flag edits to `aliases`, `avoid`, `agreed` and YAML comments, which HIST-003's boundary case allows.
- Agreed items: `statute decide --changes HIST-003`, adding a protected case such as "Rewording a later paragraph of an agreed term with no decision is reported as an error."
- Plugin: in `plugin/skills/ratify/SKILL.md` step 6, rejection restores a term's whole definition. Terms no longer have notes.
- Effect: Link's paragraph becomes compared text without being edited, and agents see it in rule context. It was approved: it was in Link's section when D261017-6x46f, which lists Link, was ratified.

Reason: decision principle 1 in AGENTS.md prefers reporting a possible meaning change over treating it as formatting. Once the glossary exists, it is the place for explanation, so `terms.md` needs no unapproved prose.

Rejected alternative: treat later paragraphs as notes that are not compared, as rule notes are (spec 4.1), and move Link's tracker sentence into its definition through a decision. It was cheaper, but a note that qualifies the meaning would be caught only by review.

### B. Retirement of a rule is final

Spec 4.1 says a retired rule keeps its section and its ID is never reused. Nothing enforces this. The history check protects only rules that read `agreed: agreed` in the base (`agreed_rule` in `src/check/history.rs`), so a retired rule can be edited or deleted with no finding. Also, `statute decide --changes` accepts a retired rule and sets it to `agreed: proposed` (`src/commands/decide.rs`), which revives it.

Chosen: a rule retired in the base keeps its section, its compared fields and `agreed: retired`, whatever decision lists it. New behaviour gets a new rule ID. `decide` refuses `--changes` and `--retires` for a retired rule. This matches st-9flm's recommendation for terms (no revival).

### C. Where the compaction audit is

Open. PR #20 says a compaction audit was saved for later human review outside the repository. Item 3a starts from it and needs its location.

## Item 1: approved term text and retired rules (st-jycx)

**What.** Make everything in a term section except its `yaml` block the approved, compared definition, shown in rule context (choice A), and make retirement of a rule final in history checks and `decide` (choice B).

**Why.** The glossary rule "no condition absent from the approved definition" depends on knowing what the approved definition is. Retired rules are referenced by active decisions, and their IDs must not be reused, but today they can be edited or deleted with no finding.

**Where.**

- `docs/Statute-spec.md`: 4.1 (rule checks), 4.3 (term format and checks), and the `decide` row of the command table in 5, which lists what `decide` refuses.
- `src/check/history.rs` (`history`, `agreed_rule`, `same_term`) and `src/commands/decide.rs` (refusing retired rules).
- `src/store/terms.rs` (`parse`), `src/model/mod.rs` (`Term`), `src/commands/show.rs` (rule-view term list), `plugin/skills/ratify/SKILL.md`, and any snapshot in `tests/snapshots/` whose `show` output changes.
- `statute/rules/history.md`.
- Tests: `tests/history_contract.rs`, and `tests/decisions_contract.rs` for the `decide` refusal.

**Steps.**

1. Run `statute show` on every file above and read the rules and terms it returns: at least HIST-002, HIST-003, DEC-001, and the Term, Rule, Agreement status and Base terms.
2. Before changing code, record two proposed decisions with `statute decide`, so a person can approve them separately:
   - Term boundary: `--changes HIST-003`. Its summary says that Link's paragraph after the `yaml` block now counts as part of Link's definition.
   - Retired rules: draft a new rule with `statute rule new history "A retired rule never changes"` and `--establishes` it. Use a new rule, not a wider HIST-002, because the exemption differs: HIST-002 lets a decision carry a change, and the new rule lets none. Give it `protects` and `boundary` cases (see acceptance).
3. Update spec 4.1, 4.3 and the `decide` row in 5 to match the decisions.
4. Change the code. For retired rules, use a new finding code, for example `rule.retired_changed`, with a message that says a retired rule keeps its section and text and that new behaviour needs a new rule ID. Do not reuse `rule.changed_without_decision`, whose message tells the reader to run `decide --changes`.
5. Add tests, then link the new rule to its code and test with `statute link`. Run `statute review` for HIST-002 and HIST-003, whose linked code changed.
6. Run `cargo test`, `statute check --all` and the `change` skill. Put `statute check --md` output in the PR.
7. Ask a person to ratify the decisions with the `ratify` skill. Leave them proposed until a person explicitly approves.
8. Complete st-jycx after merge.

**Acceptance.**

- Spec 4.3 states that everything in a term section after the heading except the `yaml` block is the definition and is compared. Spec 4.1 states that a rule retired in the base keeps its section, compared fields and `agreed: retired`.
- In `tests/history_contract.rs`, against a committed base built from a fixture, as the existing `base()` helper does (the `valid` fixture already has a retired rule in `statute/rules/attribution.md`):
  - Changing any text of an agreed term outside its `yaml` block with no decision is `term.changed_without_decision`, including a paragraph or list item after the block.
  - Editing an agreed term's `aliases`, `avoid` or a YAML comment gives no finding (HIST-003's existing boundary).
  - For a rule retired in the base, deleting it, changing its statement or a compared field, or changing its `agreed` value is a finding, even when a decision lists it in `changes`. Editing its notes is not.
- `statute decide --changes RULE` and `--retires RULE` refuse a retired rule and write nothing.
- `statute show` on a rule that uses a term with a later paragraph prints that paragraph in the rule's term list, in text and `--json`.
- `statute check --all` passes on this repository, and Link's text is unchanged.
- The decisions stay proposed until a person approves them.

**Dependencies and constraints.** None; choices A and B are made. Blocks items 2 and 4. Follow the module layering in AGENTS.md: the comparison stays in `check`, parsing in `store`.

## Item 2: human glossary and manual review (st-25bl)

**What.** Write `docs/Glossary.md`, which explains Statute's concepts to people and links to the term entries in `statute/terms.md`. Add an AGENTS.md section that makes the manual consistency review part of every PR that changes the glossary or a term. Record the first review in this item's PR.

**Why.** See the proposal's "Purpose" and "First stage". The review records this item produces are also the evidence that decides when item 5 starts.

**Where.**

- `docs/Glossary.md` (new).
- `AGENTS.md`: add the glossary to the document list at the top, and add a `## Glossary` section after `## Decision-making principles`, before the `<!-- statute:begin -->` marker. The `## Statute` section between the markers is the snippet that `statute init` manages; keep the new section outside it. Leave the module layering table unchanged; `tests/layering.rs` parses it.
- `docs/One-pager.md`: one sentence linking to the glossary, for example after "Four concepts".
- `docs/Terms-refactoring-proposal.md`: update the status line.
- Not `plugin/agents-snippet.md`, the plugin skills or the `init` snippet. The proposal does not add a required second vocabulary file to adopting repositories; general support belongs to item 5.

**Starting outline.** Six sections that cover all 21 current terms. Each term has one main section and may be linked from others. The writer can merge or split sections when that reads better, and should record why in the PR.

| Section | Main terms | Also links |
| --- | --- | --- |
| Terms, rules and decisions | Term, Rule, Decision | |
| What a rule says | Protected case, Boundary case, Manual check | Rule, Test link |
| Agreement and approval | Agreement status, Draft rule, Proposed decision, Approval | Decision, Rule, Term |
| Links and fingerprints | Link, Code link, Test link, Fingerprint, Stale link, Missing link | Rule review |
| Build state and gaps | Build state, Gap | Code link, Test link, Manual check |
| Checking a change | Base, Affected rule, Rule review | Stale link, Missing link, Gap, Proposed decision |

**Writing constraints.**

- Link from the glossary to terms only, with Markdown links such as `[Code link](../statute/terms.md#code-link)`. GitHub anchors are the heading in lower case with spaces as hyphens. Do not link from `terms.md` to the glossary; the proposal's "First stage" explains why.
- End with an index that lists every term and the section that mainly explains it.
- Add no condition, exception, state or distinction that the agreed definition lacks. If the explanation needs one, either leave it out or propose a decision with `statute decide --terms` in the same PR and say in the glossary that the change is awaiting approval.
- Familiar words can be used in context without becoming terms. Do not use a term's `avoid` words; today that is "checksum" for Fingerprint.
- Explain concepts, relationships and examples. Link to the spec and skills for procedures instead of repeating them. Prefer examples from Statute's own rules, such as LINKS-004 for missing links.
- If an item 1 decision is still proposed, describe the agreed meaning and note the pending change.

**AGENTS.md `## Glossary` section.** It states:

1. Use `docs/Glossary.md` to explain Statute to people. Use the terms that `statute show` returns to do the work.
2. A PR that changes `docs/Glossary.md`, or the text, `code` or `not` of a term in `statute/terms.md`, gets the review in the proposal's "Manual consistency review in both directions", and the record below in its description.
3. When asking a person to ratify a decision that lists a term, show the term's definition together with every glossary section that links to it.
4. A glossary edit that changes a meaning needs a proposed decision. A consistency review approves nothing.
5. If either side changes after the commit named in the record, re-review the affected rows and update the commit before merge.
6. When a review finds a counterpart that an earlier review missed, or finds drift between the glossary and a term on the default branch, add it to st-uzzw as evidence.

**Review record.** Every PR that changes either side uses this format, so item 5's start condition can be judged from PR history:

```markdown
## Glossary consistency

Reviewed at: <commit SHA>

| Glossary section | Terms compared | Conclusion |
| --- | --- | --- |
| Links and fingerprints | Link, Code link, Test link, Fingerprint, Stale link, Missing link | Consistent |

Missed by an earlier review: none
```

The conclusion is one of: consistent; updated together (name the edits); decision D… proposed.

**Acceptance.**

- `docs/Glossary.md` links every one of the 21 terms in `statute/terms.md` at least once, and every link resolves to an existing `##` heading. Check by listing the link anchors and the term headings and comparing them.
- The PR description contains a review record covering every section and its terms, each with a conclusion.
- No term's text changed except through a proposed decision.
- `AGENTS.md`, `docs/One-pager.md` and the proposal's status line link to or mention the glossary as described above.
- `cargo test` passes, including `tests/layering.rs`. `statute check --all` passes. Run the `change` skill; a documentation-only change should affect no rules.

**Dependencies and constraints.** Starts after item 1 is merged, so the approved definition is settled. Changes no CLI behaviour.

## Item 3: operational term compaction (3a: st-uvvi)

**What.** 3a: review the existing compaction audit (choice C) and decide, for each candidate, whether to keep, merge, split or reword it. 3b and 3c: carry out the chosen migrations.

**Why and criteria.** See the proposal's "Reviewing operational terms for compaction". There is no target term count.

**Where.** `statute/terms.md`, the `terms` lists in `statute/rules/*.md`, `statute/decisions.db` through `statute decide`, and `docs/Glossary.md` links once item 2 has landed.

**Constraints.**

- For each candidate, the 3a record answers the five questions in the proposal and lists the rules that reference the terms (`statute show TERM`) and their `code` and `not` names.
- Before merging two entries, compare `statute show` output for the rules and files that use them, and `statute check --all` name hits, before and after. A merge that widens what these return is a cost to record in the review.
- Every change to an agreed term goes through a proposed decision and explicit approval.
- A migration that removes or retires a term waits for item 4 (3c).
- Once item 2 has landed, each migration PR includes a glossary review record.

## Item 4: term retirement (st-9flm)

The plan is in the Bean. Constraints from this plan:

- Start after item 1 is merged. Retired terms use the retired-item history check that item 1 adds for rules (st-9flm item 8); do not build a second mechanism.
- A retired term's protected text is the part that item 1 defines as approved.
- If item 2 has landed, retiring a term also updates the glossary sections that link to it, with a review record.

## Item 5: automated consistency review (st-uzzw)

The plan and its start condition are in the Bean. It starts when a review record shows a counterpart that an earlier review missed, when drift between the glossary and a term reaches the default branch, or when review records show review effort out of proportion to the edits. Item 2's AGENTS.md instruction routes that evidence to the Bean. Its design takes item 1's approved-text boundary and item 2's section structure as inputs.
