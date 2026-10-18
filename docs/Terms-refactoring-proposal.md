# Terms refactoring proposal

Status: proposed. This document proposes how to organize Statute's vocabulary. The specific grouping below is illustrative; it does not change the agreed glossary or product behavior.

## Purpose

Reduce the vocabulary someone must learn to understand Statute. A glossary should explain the concepts that make the system work, while letting readers use familiar words in their ordinary sense.

The goal is lower cognitive load. Counting headings is only one way to assess that. Replacing several small entries with one dense entry would make the glossary shorter without making it easier to learn.

The intended learning path is: learn a few core concepts, then learn how they interact in the change workflow. Supporting details should appear where the reader needs them.

## What deserves a separate term

Give a concept its own glossary entry when Statute assigns it a specific meaning that readers need to refer to repeatedly. Its definition should resolve an ambiguity or explain a distinction that changes how someone uses the system.

A word appearing in the product does not by itself justify an entry. In particular:

- Familiar words can be explained in context when their ordinary meaning is sufficient.
- Properties, roles, states, and parts of a concept can be defined within that concept's entry.
- Operations and their outcomes can be explained in the workflow where they occur.
- Technical details can live in command or developer documentation when they are unnecessary for understanding the product.

This applies beyond lifecycle states. For example, a decision's proposed and active states belong with Decision. A link's code and test roles belong with Link. Adding a separate Link type entry would still ask readers to learn a concept that can be explained naturally as a property of a link.

The test is whether a separate entry helps someone understand or act. A qualified phrase can remain useful in prose and searches without becoming another glossary entry to memorize.

## Preserve the distinctions

Consolidation must preserve meaning. Code and test links do different jobs; approval and recording that approval have different actors; a changed target and a missing target need different responses. Those distinctions should stay explicit in the entries and workflow that explain them.

Each entry should have a short definition, followed by a compact explanation of its important properties. Keep detailed procedures in the workflow and command documentation. Give each explanation one primary home, and refer to it from elsewhere so overlapping definitions do not drift apart.

For example, a consolidated Link entry could explain:

> A link connects a rule to the implementation that makes it true or to a test that checks it.
>
> Its role is code or test. A code link identifies where the rule is implemented; a change there brings the rule up for review. A test link identifies a test of the rule's behavior. Running that test can catch violations elsewhere, including in new code with no link.
>
> A link points to a whole file or a particular symbol, such as a function. Statute records a fingerprint of that target to detect relevant changes. A symbol's fingerprint covers that declaration; dependencies need their own links when they matter.
>
> A link is stale when its fingerprint differs from the recorded value. It is missing when its target cannot be found unambiguously or its file cannot be parsed by the selected symbol mode. Both require attention; review refuses a missing target until it is fixed or relinked.

This keeps the useful meanings of code link, test link, target, fingerprint, stale, and missing together. Readers can understand those phrases through Link without learning each as an independent concept.

## An illustrative organization

The four concepts in the [one-pager](One-pager.md), together with Gap, provide a starting point:

| Concept | What belongs with it |
| --- | --- |
| Term | An agreed meaning for a product word, its names in code, and similar names that mean something else. |
| Rule | An observable obligation, its applicability and exceptions, protected and boundary cases, agreement and implementation states, and automated or manual verification. |
| Decision | A recorded choice and its rationale, the rules or terms it affects, proposal and approval, ratification, and the immutability of the approved record. |
| Link | The connection to implementation or a test, its role and target, the extent of its coverage, and detection of changed or missing targets. |
| Gap | A known mismatch between a rule and the implementation, tracked until the implementation is fixed or the rule is changed through an approved decision. |

The change workflow can explain how the tool identifies affected rules, how the agent reviews them, and what happens when a rule holds, changes intentionally, or is broken. Words such as review, verdict, finding, and approval can be introduced there with the precision needed for the task.

The one-pager can explain that guides describe the implementation while rules prescribe required behavior. Command documentation can explain technical details such as the comparison base and fingerprint modes.

Five entries are a starting point, not a fixed limit. Keep or add an entry when experience shows that its meaning needs an independent definition. Judge the result by whether readers can explain the system and perform its workflow with less effort.

## Implications for coding agents

This refactoring also affects how coding agents search code, locate implementations, and identify areas a change may affect. A term that contains more meaning will be associated with more code identifiers and implementation locations. For example, a consolidated Link concept spans link creation, code and test roles, target selection, fingerprinting, and stale or missing detection.

Fewer glossary entries therefore do not imply fewer code associations. Preserve the useful subordinate phrases and their mappings to code so an agent can start from the broader concept, identify the relevant property or operation, and narrow its search to the right code points. The association between a term and code identifiers is distinct from a rule's code and test links.

Broader terms may produce broader search results and more candidate areas to inspect. The refactoring must account for that tradeoff: reduce the vocabulary people must learn while preserving the precision agents need to find affected behavior and review it against the right rules. Assess those effects before changing the glossary or any discovery behavior that relies on it.
