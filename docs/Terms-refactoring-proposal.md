# Terms refactoring proposal

Status: proposed. Start with Statute's human glossary and manual consistency review in both directions. Automated review remains the intended design and is tracked separately. Operational terms compaction and term retirement are accepted work for separate PRs. This document does not change the agreed terms, file formats, or CLI behavior.

## Purpose

Reduce the effort people need to understand Statute and what coding agents tell them, while preserving the precision needed for approvals, rule references, code discovery, and agent context.

These needs require different forms of explanation. People need a coherent introduction, examples, and connections between concepts. Coding agents need compact, precise definitions for the task at hand. Statute also needs stable units that rules and decisions can reference.

The proposal separates those responsibilities. The human learning model does not determine how many operational terms exist or where their approval boundaries fall.

## Two forms of the same vocabulary

| Material | Audience and purpose | Organization |
| --- | --- | --- |
| Statute's human glossary, `docs/Glossary.md` | Help people understand the product and the language agents use when communicating with them. | A few connected concepts, with explanations of their properties, states, roles, and workflow. Familiar words can be introduced in context. |
| `statute/terms.md` | Supply compact definitions to agents, provide reference targets for rules and decisions, and carry code names and naming distinctions. | Entries sized for precise retrieval, independent approval, and operational use. |

Both use the same names and meanings. The glossary can explain several operational terms in one section without presenting each as an independent concept a person must memorize.

For example, a Links section can explain Link, Code link, Test link, Fingerprint, Stale link, and Missing link together. Those operational terms can remain separately addressable when that is useful. The glossary supplies the connected explanation; it does not require merging their identities.

The [one-pager](One-pager.md) remains the introduction to what Statute does and why. The glossary explains its concepts in more depth. Procedures and detailed command behavior remain in the workflow and command documentation.

Start with Statute's own documentation. This does not introduce a required second vocabulary file in every repository using Statute. General support should be able to reference an existing human glossary at a chosen path.

## Meaning and authority

Agreed operational terms remain the authoritative definitions; proposed revisions remain explicitly proposed. A compact definition must retain the conditions and distinctions an agent needs to use it correctly. An essential exception cannot exist only in the human explanation.

The glossary explains those definitions through relationships, examples, and ordinary language. It must agree with them. If an edit to the glossary introduces a new qualification or changes meaning, that change needs to be reflected in the relevant operational definitions through a proposed decision. Moving a statement into the glossary must not bypass approval.

When asking a person to approve a meaning change, show the compact definition and the relevant glossary section together. The person should understand both the explanation they will read and the definition an agent will receive. A consistency review does not itself approve a proposed decision.

## First stage: a glossary and manual review

Draft `docs/Glossary.md` for Statute while keeping the existing operational terms and identities intact. Use ordinary Markdown links from the explanations to the relevant term entries. Add ordinary references from terms to their human explanations without introducing a new metadata field or changing the approved meaning. Several terms can reference one section, and one term can reference several sections.

Explain relationships, examples, and workflow in the glossary, and point to the authoritative term entries for exact definitions. It can use familiar words in context without turning them into additional operational terms. The glossary may not introduce a condition or exception that is absent from the relevant approved definition.

Review consistency manually as part of each PR that changes either representation. The author identifies the related entries and sections; the reviewer compares their meanings. Ordinary PR diffs and Markdown links provide the initial workflow. They do not automatically identify all counterparts or prove that review happened.

Start with sections broad enough to read coherently. Review one changed explanation against all its related compact definitions. Split sections when doing so materially improves reading or review precision. Use this work to learn the useful section boundaries and review cost before specifying the automation details.

## Manual consistency review in both directions

| Change | What needs review |
| --- | --- |
| A glossary section changes | Identify every operational term it explains, following its links and checking for omitted associations. Compare the explanation with each compact definition. |
| An operational definition changes | Follow its references and search the glossary for the term and its uses. Review every relevant explanation against the changed definition. |
| A reference changes, or a section disappears or becomes ambiguous | Check the mapping and its meaning. Repair the references or explain the new association before considering it reviewed. |

Changed text requests review; it does not prove that its counterpart must be edited. A clearer example or a wording change may leave the other representation correct.

The reviewer compares the relevant definitions and explanations, then either confirms that they remain consistent or updates them together. A change to an agreed meaning follows the existing decision and approval workflow. If a proposed meaning is awaiting approval, that state must remain visible; confirming consistency must not make it agreed.

Record the compared terms and glossary sections, the reviewed commit or diff, and the conclusion in the PR. Recheck relevant later edits before considering the review complete. This is review evidence in the PR, not a new CLI verdict or persistent consistency-record format.

For example, clarifying the glossary's explanation of a missing link can be reviewed without changing the operational definition. Changing the conditions under which a target is considered missing requires a proposed decision and aligned changes to both representations. An independently proposed change to Missing link also brings its glossary explanation up for review, even if nobody edited that document initially.

## Intended design: automated consistency review

Automation remains accepted follow-up work, tracked in [st-uzzw](../.beans/st-uzzw--automate-bidirectional-review-of-glossary-and-oper.md). The manual first stage informs its design; it does not replace the intended detection and review guarantees.

Automated references should identify all relevant counterparts when either a glossary section or an operational definition changes. Missing, ambiguous, removed, and retargeted sections must remain visible until addressed. A review record should identify the exact versions compared, and relevant later edits on either side must invalidate that confirmation. Shared sections can be grouped for readable output while retaining attribution to each term.

Fingerprints detect changed content; they cannot establish that two texts mean the same thing. The reviewer still confirms consistency or proposes aligned changes. Automated review must preserve the authority and approval boundary described above and remain separate from rule code/test completeness.

The reference syntax, stable section identity, versioned Markdown fingerprint normalization, review storage, CLI output, and pending-state behavior need specification before implementation. These belong in the automation feature, not in the first glossary draft. General support should reuse a repository's existing glossary where appropriate.

## Reviewing operational terms for compaction

The need to review and compact operational terms is accepted. Separating the human glossary does not imply that every current entry is well chosen. This is separate work, to be implemented by other coding agents in a separate PR after review of the audit and migration choices. Its criteria are operational; `terms.md` is not learning material.

For each possible merge or split, ask:

- Do these meanings need independent approval, or do they change together? Would merging them unnecessarily serialize unrelated proposals under the one-proposed-decision-per-term constraint?
- Do rules need to reference them separately? Would a broader reference obscure which meaning a rule depends on?
- Do agents need to retrieve them independently, with different code names, exclusions, or naming guidance?
- Can one compact definition preserve every important distinction without becoming broad, ambiguous, or expensive to include in agent context?
- Can existing rules, historical decisions, and lookup names continue to resolve after the change?

Merge entries when their separate identities add no useful operational precision. Keep or split them when independent approval, references, retrieval, or code associations require it. A familiar word may still deserve an operational entry, and a property or state may still need its own addressable definition.

There is no target term count. The glossary can reduce learning effort while the operational vocabulary remains more detailed. Consolidation must preserve historical references and follow the decision workflow for agreed definitions and affected rules. The audit can proceed now; migrations that remove or retire existing terms must coordinate with the separate retirement feature.

## Separate accepted feature: term retirement

Term retirement is useful independently of the human glossary and is accepted feature work for a separate PR by another coding agent. It must preserve immutable historical decisions and the identity of previously approved terms while defining how current rule references and future proposals treat retired terms.

The implementation plan is recorded in high-priority feature [st-9flm](../.beans/st-9flm--retire-operational-terms-without-breaking-historic.md) for later human review. This proposal does not implement retirement, retire any term, or ratify changes to the agreed vocabulary. Drafting the human glossary and performing the compaction audit do not depend on retirement being implemented first.

## Implementation boundaries

This is a proposal for a documentation relationship and workflow, not a feature already supported by the CLI. Current links belong to rules and have code or test roles; they do not represent consistency between a term and a glossary section. Markdown section fingerprinting is also deferred in the current spec. This relationship needs explicit support and must not affect a rule's implementation or verification completeness.

Current rule retrieval returns a term's first-paragraph definition. History checks compare that definition and its `code` and `not` metadata. Additional explanatory paragraphs are not a substitute for approved operational meaning. The new retrieval behavior should include glossary references alongside compact definitions, allowing an agent to fetch human context when needed without loading the whole glossary by default.

The delivery boundaries are explicit: this PR records the approach and follow-up work; the initial glossary work uses manual review; automation, operational terms compaction, and term retirement are separate implementation PRs. The need for those follow-ups is accepted, while detailed plans and specific changes to agreed meanings still receive their own review.

## Implications for coding agents

Keeping operational terms separate from the learning material preserves the ability to search and retrieve narrowly. Agents should use the glossary's human explanations when communicating with people, while consulting the compact definitions and their code associations to perform work.

Operational compaction can still change how agents search code, locate implementations, and identify affected areas. A term containing more meaning will be associated with more code identifiers and implementation locations. Combining those meanings can broaden search results and rule-reference results even when the human glossary stays unchanged. Preserve the associations and test retrieval precision before merging entries.

Term-to-code associations, rule code/test links, and term-to-glossary references serve different purposes. The design should keep their effects clear: finding the relevant implementation, detecting changes that require rule review, and checking that human and operational meanings remain consistent. Automatic affected-rule discovery from changes to term code names remains a separate deferred feature.
