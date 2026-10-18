---
# st-25bl
title: Draft Statute's human glossary and start manual consistency review
status: todo
type: feature
priority: high
tags:
    - statute
    - glossary
created_at: 2026-10-18T12:06:59Z
updated_at: 2026-10-18T12:09:14Z
blocked_by:
    - st-jycx
---

## What

Write `docs/Glossary.md`: an explanation of Statute's concepts for people, organised in a few connected sections, with ordinary Markdown links from each section to the term entries it explains in `statute/terms.md`. Make the manual consistency review between the glossary and the terms a standing instruction in AGENTS.md, and record the first review in this Bean's PR.

This is the first stage of docs/Terms-refactoring-proposal.md ("First stage" and "Manual consistency review in both directions"). The detailed plan, including a starting outline that covers all 21 current terms, the writing constraints, the AGENTS.md instructions and the review-record format, is "Item 2" in docs/Terms-refactoring-plan.md.

## Why

People need connected explanations and examples; `statute/terms.md` holds compact definitions sized for agents, rule references and approval. Today Statute has only the one-pager and the term list. The review records this work produces are also the evidence that decides when st-uzzw (automated review) starts.

## Where

- `docs/Glossary.md` (new).
- `AGENTS.md`: add the glossary to the document list at the top; add a `## Glossary` section after `## Decision-making principles` and before `<!-- statute:begin -->`. The `## Statute` section between the markers is managed by `statute init`. Do not touch the module layering table; `tests/layering.rs` parses it.
- `docs/One-pager.md`: one sentence linking to the glossary.
- `docs/Terms-refactoring-proposal.md`: update its status line.
- Not `plugin/agents-snippet.md`, the plugin skills or the `init` snippet: adopting repositories get no required second vocabulary file. General support belongs to st-uzzw.

## Constraints

- Links run one way, from the glossary to `../statute/terms.md#<anchor>` (lower-case heading, spaces as hyphens). Do not link from `terms.md` to the glossary.
- The glossary adds no condition, exception, state or distinction that the agreed definition lacks. After st-jycx, the agreed definition is everything in a term's section after the heading except its `yaml` block. If the explanation needs more, leave it out or propose a decision with `statute decide --terms` and say in the glossary that the change awaits approval. A consistency review approves nothing.
- Do not use a term's `avoid` words (today "checksum" for Fingerprint).
- Explain concepts, relationships and examples; link to the spec and skills for procedures.

## Acceptance

- [ ] `docs/Glossary.md` links every term in `statute/terms.md` at least once, every link resolves to an existing `##` heading (compare the link anchors with the term headings), and it ends with an index of each term and the section that mainly explains it.
- [ ] AGENTS.md has the `## Glossary` section with the six instructions in the plan's Item 2, including: show a term's definition with every glossary section that links to it when asking for ratification; re-review rows changed after the recorded commit; route missed counterparts and drift to st-uzzw.
- [ ] The PR description has a "Glossary consistency" record in the plan's format, covering every section and its terms, each with a conclusion.
- [ ] No term's text changed except through a proposed decision.
- [ ] One-pager and proposal status line updated.
- [ ] `cargo test` (including `tests/layering.rs`) and `statute check --all` pass; the `change` skill reports no affected rules.

## Dependencies

Blocked by st-jycx, which settles what the approved definition is. Can run in parallel with st-9flm (term retirement) and the compaction audit review; whichever of this and a compaction migration lands second does the manual review.
