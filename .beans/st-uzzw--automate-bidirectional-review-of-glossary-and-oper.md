---
# st-uzzw
title: Automate bidirectional review of glossary and operational terms
status: todo
type: feature
priority: normal
tags:
    - statute
    - glossary
created_at: 2026-10-18T09:52:23Z
updated_at: 2026-10-18T12:09:14Z
blocked_by:
    - st-25bl
---

## Purpose

Implement automated consistency review between human glossary sections and compact operational terms. The need is accepted; PR #20 starts with Statute's own glossary and manual review in both directions. Automation remains the intended design and will be delivered separately after the reference and review semantics are specified.

Proposal: docs/Terms-refactoring-proposal.md. PR: https://github.com/50lo/statute/pull/20.

## Required behavior

- Operational terms reference the human glossary sections that explain them. Several terms can reference one section, and one term may reference several sections. Support a chosen existing glossary path rather than requiring every adopting repository to create a particular new file.
- A glossary section change identifies every referencing term for consistency review. An operational definition change identifies its glossary sections. Missing, ambiguous, removed or retargeted section references remain visible until addressed.
- A changed counterpart requires review, not an automatic text edit. The reviewer can confirm consistency or propose aligned changes. Content fingerprints do not establish semantic equivalence.
- Record the exact reviewed versions on both sides. Further relevant edits invalidate that review. Review must be idempotent for unchanged contents and must not refresh against contents different from those actually reviewed.
- Preserve the existing authority boundary: agreed operational definitions govern meaning; proposed revisions remain proposed. Consistency review does not ratify decisions or approve meaning changes. Approval presents both the compact definition and relevant human explanation.
- Include documentation references with compact term context in show output. Full human explanations can be retrieved explicitly rather than injected into every task. Group review output for shared sections without erasing the individual affected terms.
- Keep term/glossary consistency separate from rule code/test completeness and term-to-code matching. Do not count documentation references as implementation or verification links.

## When to start

Start when the manual stage gives evidence for automation: drift between the glossary and a term definition reaches the default branch, or the review records in PRs that edit either side show missed counterparts or review effort out of proportion to the edit. Until then, the manual stage stays the workflow.

## Design work before implementation

Specify the reference syntax and owner, stable section identity, heading rename/move and duplicate handling, versioned Markdown fingerprint normalization, storage of review state, CLI review command/refusals, JSON findings, rule-impact presentation, and behavior for pending decisions. st-jycx settles first that everything in a term section except its `yaml` block is the compared definition; terms have no uncompared notes.

Use existing closed enum extensions and module layering; introduce no registry or generic plugin framework. Scope the implementation to this relationship. General guide links and automatic code-name-to-rule impact matching remain separate features. Term retirement and operational term compaction are also separate PRs and are not prerequisites for drafting or manually reviewing Statute's glossary.

## Acceptance and validation

- [ ] Spec and implementation plan settle the design choices above and identify any agreed definitions/rules needing a proposed decision.
- [ ] A term-only edit and a glossary-only edit each identify the correct counterparts; an unrelated section edit does not flag other sections.
- [ ] Many-to-one and one-to-many references retain precise per-term attribution with readable grouped output.
- [ ] Missing/ambiguous targets, retargeting, section rename/move, and duplicate identities are handled explicitly without silently accepting a different explanation.
- [ ] Confirmation refreshes only the reviewed pair(s); a subsequent edit to either side makes them need review again. Refused review leaves stored records unchanged.
- [ ] Meaning changes still require a decision and explicit human approval; consistency confirmation never sets agreement state.
- [ ] show returns compact definitions and usable references, with a way to fetch the human context.
- [ ] Existing rule completeness, stale/missing code links, historical decisions, and current repository formats retain their documented behavior or receive an explicit migration.
- [ ] Focused integration tests cover these cases; layering checks, relevant regression tests, and Statute checks pass.

The manual glossary work supplies evidence about useful section granularity and review cost. It is an initial delivery stage, not a substitute for the accepted automation work. No implementation or ratification is requested by creation of this feature.
