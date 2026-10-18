# Terms refactoring proposal

Status: proposed. This document proposes a human glossary linked to compact operational terms, with consistency review in both directions. It does not change the agreed terms, file formats, or CLI behavior.

## Purpose

Reduce the effort people need to understand Statute and what coding agents tell them, while preserving the precision needed for approvals, rule references, code discovery, and agent context.

These needs require different forms of explanation. People need a coherent introduction, examples, and connections between concepts. Coding agents need compact, precise definitions for the task at hand. Statute also needs stable units that rules and decisions can reference.

The proposal separates those responsibilities. The human learning model does not determine how many operational terms exist or where their approval boundaries fall.

## Two forms of the same vocabulary

| Material | Audience and purpose | Organization |
| --- | --- | --- |
| A human glossary, for example `docs/Glossary.md` | Help people understand the product and the language agents use when communicating with them. | A few connected concepts, with explanations of their properties, states, roles, and workflow. Familiar words can be introduced in context. |
| `statute/terms.md` | Supply compact definitions to agents, provide reference targets for rules and decisions, and carry code names and naming distinctions. | Entries sized for precise retrieval, independent approval, and operational use. |

Both use the same names and meanings. The glossary can explain several operational terms in one section without presenting each as an independent concept a person must memorize.

For example, a Links section can explain Link, Code link, Test link, Fingerprint, Stale link, and Missing link together. Those operational terms can remain separately addressable when that is useful. The glossary supplies the connected explanation; it does not require merging their identities.

The [one-pager](One-pager.md) remains the introduction to what Statute does and why. The glossary explains its concepts in more depth. Procedures and detailed command behavior remain in the workflow and command documentation.

## Meaning and authority

Agreed operational terms remain the authoritative definitions; proposed revisions remain explicitly proposed. A compact definition must retain the conditions and distinctions an agent needs to use it correctly. An essential exception cannot exist only in the human explanation.

The glossary explains those definitions through relationships, examples, and ordinary language. It must agree with them. If an edit to the glossary introduces a new qualification or changes meaning, that change needs to be reflected in the relevant operational definitions through a proposed decision. Moving a statement into the glossary must not bypass approval.

When asking a person to approve a meaning change, show the compact definition and the relevant glossary section together. The person should understand both the explanation they will read and the definition an agent will receive. A consistency review does not itself approve a proposed decision.

## References between terms and the glossary

Each operational term references the glossary section that explains it. Several terms may reference the same section. A term whose explanation spans sections may need more than one reference.

The reference identifies the section, and a recorded fingerprint detects when its contents change. Missing or ambiguous sections require attention. Changing a reference also requires review, so a term cannot silently acquire a different explanation by pointing elsewhere.

Start with sections broad enough to read coherently. If six terms reference Links, an edit to that section may bring all six up for consistency review. The report can group them under the changed section so the agent reviews the explanation once against the relevant definitions. Narrower sections are useful when they improve precision enough to justify their maintenance cost.

The exact reference syntax, section identity, fingerprint normalization, and review-record format need specification before implementation. This document describes the relationship and its required behavior rather than introducing a new schema.

## Consistency review in both directions

| Change | What needs review |
| --- | --- |
| A glossary section changes | Every operational term referencing that section. Check whether each compact definition still expresses the intended meaning. |
| An operational definition changes | Every glossary section referenced by that term. Check whether the human explanation still agrees with the definition. |
| A reference changes, or a section disappears or becomes ambiguous | The mapping and its meaning. Repair the reference or explain the new association before considering it reviewed. |

Changed text requests review; it does not prove that its counterpart must be edited. A clearer example or a wording change may leave the other representation correct.

The reviewer compares the relevant definitions and explanations, then either confirms that they remain consistent or updates them together. A change to an agreed meaning follows the existing decision and approval workflow. If a proposed meaning is awaiting approval, that state must remain visible; refreshing a consistency record must not make it agreed.

Record the exact versions reviewed. A later edit to either side must bring the relationship up for review again. A fingerprint establishes which contents were checked; it cannot establish that two texts mean the same thing.

For example, clarifying the glossary's explanation of a missing link can be reviewed without changing the operational definition. Changing the conditions under which a target is considered missing requires a proposed decision and aligned changes to both representations. An independently proposed change to Missing link also brings its glossary explanation up for review, even if nobody edited that document initially.

## Reviewing operational terms for compaction

The existing terms still need review. Separating the human glossary does not imply that every current entry is well chosen. That review uses operational criteria rather than treating `terms.md` as learning material.

For each possible merge or split, ask:

- Do these meanings need independent approval, or do they change together? Would merging them unnecessarily serialize unrelated proposals under the one-proposed-decision-per-term constraint?
- Do rules need to reference them separately? Would a broader reference obscure which meaning a rule depends on?
- Do agents need to retrieve them independently, with different code names, exclusions, or naming guidance?
- Can one compact definition preserve every important distinction without becoming broad, ambiguous, or expensive to include in agent context?
- Can existing rules, historical decisions, and lookup names continue to resolve after the change?

Merge entries when their separate identities add no useful operational precision. Keep or split them when independent approval, references, retrieval, or code associations require it. A familiar word may still deserve an operational entry, and a property or state may still need its own addressable definition.

There is no target term count. The glossary can reduce learning effort while the operational vocabulary remains more detailed. Any eventual consolidation must preserve historical references and follow the decision workflow for agreed definitions and affected rules. Term retirement or another migration mechanism should be designed for the consolidations actually chosen.

## Implementation boundaries

This is a proposal for a documentation relationship and workflow, not a feature already supported by the CLI. Current links belong to rules and have code or test roles; they do not represent consistency between a term and a glossary section. Markdown section fingerprinting is also deferred in the current spec. This relationship needs explicit support and must not affect a rule's implementation or verification completeness.

Current rule retrieval returns a term's first-paragraph definition. History checks compare that definition and its `code` and `not` metadata. Additional explanatory paragraphs are not a substitute for approved operational meaning. The new retrieval behavior should include glossary references alongside compact definitions, allowing an agent to fetch human context when needed without loading the whole glossary by default.

The glossary can be drafted while existing operational identities remain intact. Automated consistency checking and any later term consolidation need separate implementation and migration work. Neither a fixed reduction of the operational vocabulary nor term retirement is a prerequisite for drafting the human glossary.

## Implications for coding agents

Keeping operational terms separate from the learning material preserves the ability to search and retrieve narrowly. Agents should use the glossary's human explanations when communicating with people, while consulting the compact definitions and their code associations to perform work.

Operational compaction can still change how agents search code, locate implementations, and identify affected areas. A term containing more meaning will be associated with more code identifiers and implementation locations. Combining those meanings can broaden search results and rule-reference results even when the human glossary stays unchanged. Preserve the associations and test retrieval precision before merging entries.

Term-to-code associations, rule code/test links, and term-to-glossary references serve different purposes. The design should keep their effects clear: finding the relevant implementation, detecting changes that require rule review, and checking that human and operational meanings remain consistent. Automatic affected-rule discovery from changes to term code names remains a separate deferred feature.
