# Statute glossary

This glossary is for engineers, product managers, and designers learning Statute or reading what a coding agent says about it. It explains the words Statute uses, with examples of how they fit together.

Start with terms, rules and decisions, then follow a rule through approval, implementation and review. The [index](#index) helps you look up a particular word. Links on term names take you to their exact definitions in [statute/terms.md](../statute/terms.md).

For why Statute exists, see the [one-pager](One-pager.md). For commands and file formats, see the [spec](Statute-spec.md). Statute comes with instructions for coding agents, called skills. They explain how to adopt Statute, check rules after a code change, and prepare decisions for a person’s approval. You can read them in [plugin/skills](../plugin/skills/).

Sections:

1. [Terms, rules and decisions](#terms-rules-and-decisions)
2. [What a rule protects and allows](#what-a-rule-protects-and-allows)
3. [Agreement and approval](#agreement-and-approval)
4. [Links and fingerprints](#links-and-fingerprints)
5. [Build state and gaps](#build-state-and-gaps)
6. [Checking a change](#checking-a-change)
7. [Keeping this glossary correct](#keeping-this-glossary-correct)
8. [Index](#index)

## Terms, rules and decisions

Main terms: [Term](../statute/terms.md#term), [Rule](../statute/terms.md#rule), [Decision](../statute/terms.md#decision). Also mentions: [Agreement status](../statute/terms.md#agreement-status), [Approval](../statute/terms.md#approval), [Proposed decision](../statute/terms.md#proposed-decision), [Link](../statute/terms.md#link).

Statute keeps three kinds of record: terms for the words a product uses, rules for what must be true, and decisions for why people chose those rules and meanings. [Links](#links-and-fingerprints) connect the rules to code and tests.

A [term](../statute/terms.md#term) defines a word or phrase used in the product. It records the names used for it in code and distinguishes names that look alike but mean something different. For example, "Session" means one conversation between a developer and a coding agent. In code it is `Session`. The `sessions` table looks similar, but it holds sign-in sessions. So the term lists `sessions` as a name that means something else. Statute's own terms are in `statute/terms.md`.

A [rule](../statute/terms.md#rule) states one thing that must be true about the product, and when it must be true. A person or a test can check whether the product follows it. For example, rule CORE-005 says: "Only a person can approve a proposed decision." Each rule has an ID and lives in a file in `statute/rules/`.

A [decision](../statute/terms.md#decision) records a choice about rules or terms, and the reason for it. For example, it can establish a new rule, change a rule, or introduce a term. It starts as a proposal. After a person approves it, the agent records that approval with `statute approve`. The decision becomes active and records who approved it. Each decision has an ID such as `D261018-m21p6`; they are kept in `statute/decisions.db`.

How these three fit together:

- Rules use terms to describe reqirement precisely. Each rule lists the terms it uses, so when a coding agent reads a rule with `statute show`, it also gets the definitions of those terms.
- Decisions explain why the rules and terms are the way they are.
- An agreed rule or term is never changed just to match the code. It changes only through a decision that records the change (rules HIST-002 and HIST-003). [Agreement and approval](#agreement-and-approval) section explains the process.
- An active decision is never edited or deleted (rule HIST-001). To correct one, record a new decision.

## What a rule protects and allows

Main terms: [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case), [Manual check](../statute/terms.md#manual-check). Also mentions: [Rule](../statute/terms.md#rule), [Test link](../statute/terms.md#test-link), [Code link](../statute/terms.md#code-link), [Link](../statute/terms.md#link), [Decision](../statute/terms.md#decision), [Approval](../statute/terms.md#approval), [Proposed decision](../statute/terms.md#proposed-decision).

People can read the same [rule](../statute/terms.md#rule) in different ways. Concrete cases help them check whether they mean the same thing.

A [protected case](../statute/terms.md#protected-case) shows behavior the rule must preserve. It gives a person or agent a case to try when checking the rule.

A [boundary case](../statute/terms.md#boundary-case) shows where the rule stops applying. It describes nearby behavior that the rule should still allow.

For example, CORE-005 says that only a person can approve a proposed decision:

- Protected case: an agent's confirmation that tests pass does not count as human approval.
- Boundary case: after a person explicitly approves a specific decision, the agent may run `statute approve` to record that approval.

In a rule file, these cases are the `protects` and `boundary` lists. An agreed rule needs at least one of each (CORE-001). Where automated tests can judge the behavior, the tests check both cases, and [test links](../statute/terms.md#test-link) connect them to the rule ([spec 4.1](Statute-spec.md#41-rules)).

A [manual check](../statute/terms.md#manual-check) is a written procedure for a person to verify behavior that an automated test cannot judge. It replaces the test requirement. The rule still needs a [code link](../statute/terms.md#code-link).

CORE-005 uses a manual check because a test cannot see whether a person really approved. Before each approval, a person checks the conversation for explicit approval of every decision that the agent will approve.

## Agreement and approval

Main terms: [Agreement status](../statute/terms.md#agreement-status), [Draft rule](../statute/terms.md#draft-rule), [Proposed decision](../statute/terms.md#proposed-decision), [Approval](../statute/terms.md#approval). Also mentions: [Decision](../statute/terms.md#decision), [Rule](../statute/terms.md#rule), [Term](../statute/terms.md#term), [Build state](../statute/terms.md#build-state).

People can agree to a rule before anyone writes the code for it. Statute records that agreement separately from the [build state](../statute/terms.md#build-state), explained in [Build state and gaps](#build-state-and-gaps).

### Agreement status

The [agreement status](../statute/terms.md#agreement-status) of a [rule](../statute/terms.md#rule) or [term](../statute/terms.md#term) records whether people have accepted it:

- **proposed**: suggested, but not accepted yet.
- **agreed**: accepted.
- **retired**: no longer used, but kept as a record. See [Retired rules and terms](#retired-rules-and-terms) below.

New rules and terms that an agent writes start as proposed (CORE-004). The CLI sets their status; `statute approve` makes them agreed or retired ([spec 4.1](Statute-spec.md#41-rules)). If a person rejects a proposal, the agent follows the [approve skill](../plugin/skills/approve/SKILL.md) to restore the earlier agreed text and status of the rules and terms it changed.

### From draft to agreed

Here is how a new rule becomes agreed:

1. An agent writes a new rule. No [decision](../statute/terms.md#decision) establishes it yet, so it is a [draft rule](../statute/terms.md#draft-rule).
2. The agent records a [proposed decision](../statute/terms.md#proposed-decision) that establishes the rule. The rule is no longer a draft, but it is still proposed.
3. A person reads the decision and approves it. This is [approval](../statute/terms.md#approval).
4. The agent runs `statute approve` with the decision's ID. The decision becomes active, and the rule becomes agreed.

A change to an agreed rule or term follows steps 2 to 4. The decision names what it changes, and the changed rule or term is proposed again until approval. A proposal to retire it works differently: it stays agreed and in force until approval.

`statute status` lists draft rules as pending work. `statute check` lets you keep a draft without reporting a problem.

### Proposed decisions

A proposed decision is a decision that a person has not approved yet. It is also called a pending decision. While it waits for approval:

- The rules it establishes or changes stay proposed. The terms it introduces or redefines also stay proposed.
- A rule or term that it retires stays agreed, and stays in force, until the decision is approved.
- The decision can still be edited or deleted.
- It never blocks a merge. A branch with proposed decisions can be merged. `statute status` and the "Rule impact" section of the PR show what still waits for approval.

A rule or term can be in only one proposed decision at a time (rule DEC-001).

### Approval

Approval is a person's explicit acceptance of a specific proposed decision. The person must say directly which decision they accept.

- An agent can prepare the proposal and run the checks. Only a person can approve it (CORE-005).
- Approval of one decision does not cover another one (CORE-005).
- After the person approves, the agent runs `statute approve`. This records the decision as active and updates its rules and terms.

The [approve skill](../plugin/skills/approve/SKILL.md) describes how the agent presents each decision for approval.

### Retired rules and terms

A decision can retire a rule or term that people should stop using. After approval, its status is retired. It stays in the file so people can still understand the active decisions that name it.

Retirement is final ([spec 4.1](Statute-spec.md#41-rules) and [4.3](Statute-spec.md#43-terms)):

- A retired rule or term never changes, and its ID or name is never used again (HIST-004 and HIST-005). New behavior gets a new rule ID. A new meaning gets a new term.
- Only retired rules and active decisions may name a retired term (rule DEC-005).

## Links and fingerprints

Main terms: [Link](../statute/terms.md#link), [Code link](../statute/terms.md#code-link), [Test link](../statute/terms.md#test-link), [Fingerprint](../statute/terms.md#fingerprint), [Stale link](../statute/terms.md#stale-link), [Missing link](../statute/terms.md#missing-link). Also mentions: [Rule](../statute/terms.md#rule), [Affected rule](../statute/terms.md#affected-rule), [Rule review](../statute/terms.md#rule-review), [Build state](../statute/terms.md#build-state), [Manual check](../statute/terms.md#manual-check), [Gap](../statute/terms.md#gap).

### Links

A [link](../statute/terms.md#link) connects a rule to the code that implements it or a test that checks it. The linked part is called the target. It can be a whole file or a named part of a file, such as a function, type or method (a "symbol" in [spec 4.4](Statute-spec.md#44-links)). Statute records links and their fingerprints in `statute.lock` so it can detect changes to the targets.

A [code link](../statute/terms.md#code-link) points to the implementation that makes the rule true. When that code changes, the rule comes up for review, and the link tells the reviewer where to look.

A [test link](../statute/terms.md#test-link) points to a test that checks the rule's behavior. The test can catch a violation outside the linked implementation. For example, someone adds a filter in another function and breaks the rule. That change leaves the code links untouched, but the linked test fails. The project's automated checks (CI) run the test; Statute records the connection between the test and the rule.

Example from this repository: rule LINKS-004 has code links to symbols such as `src/state.rs#health` and `src/commands/review.rs#run`. It has a test link to the whole file `tests/missing_contract.rs`.

A rule that is [built](../statute/terms.md#build-state) needs both kinds, or a [manual check](../statute/terms.md#manual-check) in place of the test link. See [Build state and gaps](#build-state-and-gaps).

### Fingerprints

A [fingerprint](../statute/terms.md#fingerprint) is a hash: a value computed from the linked code or test. Statute saves it in `statute.lock` as `sig` and compares it with a newly computed value to detect changes. The link's "mode" is the method used to compute the hash for that kind of target, such as a Rust function or a whole file.

The symbol modes, used for functions, types and methods, ignore comments and formatting. Any other change to the target's code changes the fingerprint, even when the behavior stays the same. [Spec 4.4](Statute-spec.md#44-links) lists the modes and what each one ignores.

A changed fingerprint tells the agent to check the rule. The agent then assesses whether the rule still holds in a [rule review](../statute/terms.md#rule-review).

### Stale and missing links

A [stale link](../statute/terms.md#stale-link) means the target's fingerprint has changed since Statute recorded it. The rule becomes an [affected rule](../statute/terms.md#affected-rule) and needs review. Only a review verdict refreshes the saved fingerprint (LINKS-003). See [Checking a change](#checking-a-change).

A [missing link](../statute/terms.md#missing-link) means Statute cannot find or reliably identify the target:

- The target is not found. For example, the linked function was renamed, moved or deleted.
- The target matches more than one symbol.
- The target is in a file that does not parse cleanly, for example because of a syntax error. This is true even when the linked function itself is fine (rule LINKS-001).

A missing link also brings its rule up for review. `statute review` refuses a rule while one of its links is missing (rule LINKS-004). If the target moved, the agent first checks the rule, and then links the target at its new location.

Example from the [spec's usage scenarios](Statute-spec.md#appendix-usage-scenarios): an engineer renames the linked function `Publisher.Decide` to `Publisher.Choose`. The link to `Publisher.Decide` is now missing. The agent checks the rule, removes the old link, and links `Publisher.Choose`.

A [gap](../statute/terms.md#gap) can also refer to an issue in an external tracker. Statute keeps that reference in `statute/gaps.yml`, separately from code and test links. Gaps are explained next.

## Build state and gaps

Main terms: [Build state](../statute/terms.md#build-state), [Gap](../statute/terms.md#gap). Also mentions: [Code link](../statute/terms.md#code-link), [Test link](../statute/terms.md#test-link), [Manual check](../statute/terms.md#manual-check), [Agreement status](../statute/terms.md#agreement-status), [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case), [Rule](../statute/terms.md#rule), [Link](../statute/terms.md#link), [Missing link](../statute/terms.md#missing-link), [Proposed decision](../statute/terms.md#proposed-decision).

### Build state

The [agreement status](../statute/terms.md#agreement-status) says whether people accepted a rule. The [build state](../statute/terms.md#build-state) says how far its implementation has got:

- **planned**: the rule is not built yet. People may already have agreed to it.
- **built**: the implementation is recorded as built. It needs a [code link](../statute/terms.md#code-link) and either a [test link](../statute/terms.md#test-link) or a [manual check](../statute/terms.md#manual-check) (CORE-003).

Statute reports a built rule as **broken** while it has an open gap, unless the rule is retired (GAPS-001). It computes this status from the gap; "broken" is never stored in the rule itself.

Nobody edits the build state by hand. `statute link` and `statute approve` mark a rule built when it has everything that a built rule needs ([spec 4.1](Statute-spec.md#41-rules)).

Example from this repository: rule CORE-005 is agreed, but it is still planned. It has a manual check, but no code link yet.

Code, documents and tests can all agree with each other and still be wrong. A built rule still needs review against its [protected](../statute/terms.md#protected-case) and [boundary](../statute/terms.md#boundary-case) cases ([spec 4.1](Statute-spec.md#41-rules)).

### Gaps

A [gap](../statute/terms.md#gap) records a known mismatch between a rule and the implementation. It stays open as an issue until resolved. A person decides whether to fix the implementation or change the rule. Changing an agreed rule needs a proposed decision.

Example: suppose a change makes `statute check` stop reporting a renamed function as missing. Rule LINKS-004 no longer holds. The agent opens a gap for LINKS-004. Until the gap is closed, `statute status` shows LINKS-004 as broken.

Where gaps are kept ([spec 4.5](Statute-spec.md#45-gaps)):

- `statute/gaps.yml` lists every gap.
- In this repository, the gap itself is a Bean tagged `statute` and `gap` (see the "Issue tracker" section of AGENTS.md). The entry in `gaps.yml` is only an index: ID, rule and status.
- In a repository without an issue tracker, the entry in `gaps.yml` is the whole gap.

## Checking a change

Main terms: [Base](../statute/terms.md#base), [Affected rule](../statute/terms.md#affected-rule), [Rule review](../statute/terms.md#rule-review). Also mentions: [Stale link](../statute/terms.md#stale-link), [Missing link](../statute/terms.md#missing-link), [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case), [Gap](../statute/terms.md#gap), [Proposed decision](../statute/terms.md#proposed-decision), [Manual check](../statute/terms.md#manual-check), [Rule](../statute/terms.md#rule), [Link](../statute/terms.md#link), [Decision](../statute/terms.md#decision), [Approval](../statute/terms.md#approval), [Term](../statute/terms.md#term).

Every change goes through the same loop:

1. Before coding, the agent runs `statute show` on the files it will change. It reads the rules and terms that come back.
2. After coding, `statute check` compares the work with the base and reports stale and missing links. A rule with such a link is an affected rule.
3. The agent reviews each affected rule and records the result (a verdict).
4. `statute check --md` writes the "Rule impact" section for the PR.

The [change skill](../plugin/skills/change/SKILL.md) has the exact steps.

### Base

The [base](../statute/terms.md#base) is the saved version of the project (a git commit) that `statute check` compares your work against. It is the most recent commit shared by your branch and the default branch, such as `main`. Git calls this the merge-base. If there is no default branch, Statute uses your current commit, `HEAD`.

Changes since the base include:

- the commits on your branch,
- files that you changed but did not commit,
- untracked files: new files that are not in git yet.

### Affected rule

An [affected rule](../statute/terms.md#affected-rule) is a rule that a change brings up for review. For example, Statute flags a rule when one of its links becomes [stale](../statute/terms.md#stale-link) or [missing](../statute/terms.md#missing-link).

Review determines whether the affected rule still holds. [Spec 5](Statute-spec.md#5-cli) lists all the reasons that the "Rule impact" section uses to mark a rule as affected.

### Rule review

A [rule review](../statute/terms.md#rule-review) is the agent's assessment of whether an affected rule still holds after a change. The agent reads the rule and its implementation, tries its [protected](../statute/terms.md#protected-case) and [boundary](../statute/terms.md#boundary-case) cases, and records one of three results:

| Result | What the agent records |
| --- | --- |
| The rule holds | Confirmation that the cases still behave as the rule says. |
| The rule changes on purpose | A [proposed decision](../statute/terms.md#proposed-decision) to change or retire the rule. |
| The rule is broken | A [gap](../statute/terms.md#gap) describing the mismatch. |

The [change skill](../plugin/skills/change/SKILL.md) gives the commands for each result. When `statute review` accepts the verdict, it refreshes all stale links of the rule ([spec 4.4](Statute-spec.md#44-links)).

Who does what in a rule review:

- The tool finds changed links and checks the records and fingerprints.
- The agent does the review.
- A person does any [manual check](../statute/terms.md#manual-check), and approves any decision that changes the rule.

For example, the function `health` in `src/state.rs` has links from three rules: LINKS-001, LINKS-002 and LINKS-004. A code change that alters its fingerprint makes all three links stale. The agent reviews each rule. If all three still hold, it records that result for each one, and Statute refreshes the links.

## Keeping this glossary correct

Main terms: none. Also mentions: [Term](../statute/terms.md#term), [Proposed decision](../statute/terms.md#proposed-decision), [Rule review](../statute/terms.md#rule-review).

People approve the exact definitions in [statute/terms.md](../statute/terms.md), and coding agents read them when they work. If this glossary and an agreed definition disagree, follow the agreed definition and report the difference so the glossary can be fixed.

Changing a term's meaning needs a [proposed decision](../statute/terms.md#proposed-decision) together with a change to its definition. Editing this glossary approves nothing.

A PR that changes this glossary or a term's definition includes a consistency review. The reviewer compares each changed explanation with the definitions it uses, and each changed definition with the explanations that use it. This checks that the texts agree; a [rule review](../statute/terms.md#rule-review) checks whether the implementation follows a rule. The "Glossary" section of AGENTS.md gives the full review requirements and record format.

## Index

| Term | Explained in |
| --- | --- |
| [Affected rule](../statute/terms.md#affected-rule) | [Checking a change](#checking-a-change) |
| [Agreement status](../statute/terms.md#agreement-status) | [Agreement and approval](#agreement-and-approval) |
| [Approval](../statute/terms.md#approval) | [Agreement and approval](#agreement-and-approval) |
| [Base](../statute/terms.md#base) | [Checking a change](#checking-a-change) |
| [Boundary case](../statute/terms.md#boundary-case) | [What a rule protects and allows](#what-a-rule-protects-and-allows) |
| [Build state](../statute/terms.md#build-state) | [Build state and gaps](#build-state-and-gaps) |
| [Code link](../statute/terms.md#code-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Decision](../statute/terms.md#decision) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Draft rule](../statute/terms.md#draft-rule) | [Agreement and approval](#agreement-and-approval) |
| [Fingerprint](../statute/terms.md#fingerprint) | [Links and fingerprints](#links-and-fingerprints) |
| [Gap](../statute/terms.md#gap) | [Build state and gaps](#build-state-and-gaps) |
| [Link](../statute/terms.md#link) | [Links and fingerprints](#links-and-fingerprints) |
| [Manual check](../statute/terms.md#manual-check) | [What a rule protects and allows](#what-a-rule-protects-and-allows) |
| [Missing link](../statute/terms.md#missing-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Proposed decision](../statute/terms.md#proposed-decision) | [Agreement and approval](#agreement-and-approval) |
| [Protected case](../statute/terms.md#protected-case) | [What a rule protects and allows](#what-a-rule-protects-and-allows) |
| [Rule](../statute/terms.md#rule) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Rule review](../statute/terms.md#rule-review) | [Checking a change](#checking-a-change) |
| [Stale link](../statute/terms.md#stale-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Term](../statute/terms.md#term) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Test link](../statute/terms.md#test-link) | [Links and fingerprints](#links-and-fingerprints) |
