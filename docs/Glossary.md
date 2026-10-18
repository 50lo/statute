# Statute glossary

This glossary explains the words that Statute uses. It is for people: product managers, designers and engineers who use Statute, or who read what a coding agent says about it.

Every term has an exact definition in [statute/terms.md](../statute/terms.md). Coding agents read those definitions when they work, and people approve them. This glossary explains the same meanings in more words. It gives examples and shows how the terms connect.

This glossary does not change any meaning:

- If this glossary and `terms.md` seem to disagree, the definition in `terms.md` counts. Please report the difference, so that the glossary can be fixed.
- To change what a term means, change its definition in `terms.md` together with a proposed decision. Editing this glossary does not approve anything.

Each section starts with a list of the terms it explains. Each term name in that list, and in the text, is a Markdown link to the term's definition in `terms.md`. The [index](#index) at the end lists every term and the section that explains it.

This glossary explains ideas, not procedures. For commands and file formats, see the [spec](Statute-spec.md). For the steps that an agent follows, see the skills in [plugin/skills](../plugin/skills/). For why Statute exists, see the [one-pager](One-pager.md).

Sections:

1. [Terms, rules and decisions](#terms-rules-and-decisions)
2. [What a rule says](#what-a-rule-says)
3. [Agreement and approval](#agreement-and-approval)
4. [Links and fingerprints](#links-and-fingerprints)
5. [Build state and gaps](#build-state-and-gaps)
6. [Checking a change](#checking-a-change)
7. [Keeping this glossary correct](#keeping-this-glossary-correct)
8. [Index](#index)

## Terms, rules and decisions

Main terms: [Term](../statute/terms.md#term), [Rule](../statute/terms.md#rule), [Decision](../statute/terms.md#decision). Also mentions: [Agreement status](../statute/terms.md#agreement-status).

This section explains Statute's three basic records: the words that the product uses, the things that must be true, and the choices behind them. Links, which connect rules to code and tests, are explained in [Links and fingerprints](#links-and-fingerprints).

A **[term](../statute/terms.md#term)** is a word or phrase used in the product, with a definition. A term also records the names used for it in code. It can also list names that look alike but mean something different.

Example from [spec 4.3](Statute-spec.md#43-terms): "Session" means one conversation between a developer and a coding agent. In code it is `Session`. The `sessions` table looks similar, but it holds sign-in sessions. So the term lists `sessions` as a name that means something else.

Statute's own terms are in `statute/terms.md`. Each `##` heading in that file is one term. Spec 4.3 describes the format.

A **[rule](../statute/terms.md#rule)** states one thing that must be true about the product, and when it must be true. A person or a test can check whether the product follows it. Each rule has an ID and lives in a file in `statute/rules/`. For example, rule CORE-005 says: "Only a person can approve a proposed decision."

A **[decision](../statute/terms.md#decision)** records a choice about rules or terms, and the reason for it. It starts as a proposal. When a person approves it, it becomes active, and it records who approved it. A decision lists what it does. For example, it can establish a new rule, change a rule, or introduce a term. Decisions are in `statute/decisions.db`, and each one has an ID such as `D261018-m21p6`. [Spec 4.2](Statute-spec.md#42-decisions) lists all the fields.

How the three fit together:

- Rules use terms. Each rule lists the terms it uses. When an agent reads a rule with `statute show`, it also gets the definitions of those terms.
- Decisions explain why the rules and terms are the way they are.
- An agreed rule or term is never changed just to match the code. It changes only together with a decision that lists it (rules HIST-002 and HIST-003). [Agreement and approval](#agreement-and-approval) explains how such a change becomes agreed.
- An active decision is never edited or deleted (rule HIST-001). To correct one, record a new decision.

## What a rule says

Main terms: [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case), [Manual check](../statute/terms.md#manual-check). Also mentions: [Rule](../statute/terms.md#rule), [Test link](../statute/terms.md#test-link), [Code link](../statute/terms.md#code-link).

The statement of a [rule](../statute/terms.md#rule) says what must be true. But people can read the same words in different ways. So a rule also carries concrete cases.

A **[protected case](../statute/terms.md#protected-case)** is a concrete case that shows behavior the rule must keep. When a person or an agent checks whether the rule still holds, this is a case they can try.

A **[boundary case](../statute/terms.md#boundary-case)** is a nearby case that shows where the rule stops applying. It makes sure that the rule does not reject valid behavior outside the rule's intended scope.

Example: rule LINKS-004 says that a link is missing when its target is gone or ambiguous.

- Protected case: "A link to a Go function that was renamed is reported as missing."
- Boundary case: "Removing the missing link and linking the new location clears the finding."

In a rule file, these cases are the `protects` and `boundary` lists. Rule CORE-001 says that an agreed rule must have at least one of each. [Spec 4.1](Statute-spec.md#41-rules) maps the cases onto tests: one test that fails when the protected case breaks, and one test that keeps the boundary case allowed. Such a test is a [test link](../statute/terms.md#test-link) of the rule.

A **[manual check](../statute/terms.md#manual-check)** is a written procedure that a person follows to verify a rule. It is for a rule whose behavior an automated test cannot judge. A manual check replaces only the test requirement. The rule still needs a [code link](../statute/terms.md#code-link).

Example: rule CORE-005 says that only a person can approve a decision. A test cannot see whether a person really approved. So CORE-005 has a manual check: before each ratification, a person checks the conversation for an explicit approval of every decision that the agent will ratify.

## Agreement and approval

Main terms: [Agreement status](../statute/terms.md#agreement-status), [Draft rule](../statute/terms.md#draft-rule), [Proposed decision](../statute/terms.md#proposed-decision), [Approval](../statute/terms.md#approval). Also mentions: [Decision](../statute/terms.md#decision), [Rule](../statute/terms.md#rule), [Term](../statute/terms.md#term), [Build state](../statute/terms.md#build-state).

Statute records two separate facts about a rule: did people agree to it, and does the code do it? This section is about the first fact. The second fact is the [build state](../statute/terms.md#build-state), explained in [Build state and gaps](#build-state-and-gaps).

### Agreement status

The **[agreement status](../statute/terms.md#agreement-status)** of a [rule](../statute/terms.md#rule) or a [term](../statute/terms.md#term) is one of three values:

- **proposed**: suggested, but not accepted yet.
- **agreed**: accepted.
- **retired**: no longer used, but kept as a record. See [Retired rules and terms](#retired-rules-and-terms) below.

Agreement status records acceptance only. It is separate from whether the code implements the rule (rule CORE-002). A rule can be agreed before anyone writes its code.

Nobody edits the agreement status by hand. The CLI sets it. New rules and terms that an agent writes start as proposed (rule CORE-004). `statute ratify` makes them agreed or retired.

### From draft to agreed

Here is how a new rule becomes agreed:

1. An agent writes a new rule with `statute rule new`. The rule is proposed. No [decision](../statute/terms.md#decision) establishes it yet, so it is a **[draft rule](../statute/terms.md#draft-rule)**.
2. The agent records a decision that establishes the rule, with `statute decide --establishes`. The decision is a **[proposed decision](../statute/terms.md#proposed-decision)**. The rule is no longer a draft, but it is still proposed.
3. A person reads the decision and approves it. This is **[approval](../statute/terms.md#approval)**.
4. The agent runs `statute ratify` with the decision's ID. The decision becomes active, and the rule becomes agreed.

A change to an agreed rule or term follows steps 2 to 4. The decision lists the rule in `changes`, or the term in `terms`. Until the decision is ratified, the changed rule or term is proposed again.

A draft rule is a pending item. `statute status` lists it. It is not a finding: `statute check` does not report it as a problem.

### Proposed decisions

A proposed decision is a decision that a person has not approved yet. It is also called a pending decision. While it waits for approval:

- The rules it establishes or changes stay proposed. The terms it introduces or redefines also stay proposed.
- A rule or term that it retires stays agreed, and stays in force, until the decision is ratified.
- The decision can still be edited or deleted.
- It never blocks a merge. A branch with proposed decisions can be merged. `statute status` and the "Rule impact" section of the PR show what still waits for approval.

A rule or term can be in only one proposed decision at a time (rule DEC-001).

### Approval

Approval is a person's explicit acceptance of one specific proposed decision. Explicit means that the person says it directly. The agent does not assume it.

- Only a person can approve. An agent can prepare the proposal and run the checks, but it cannot approve its own proposal (rule CORE-005).
- Approval covers a specific decision. Approval of one decision does not cover another one (rule CORE-005).
- After the person approves, the agent runs `statute ratify`. This records the decision as active and updates its rules and terms.

The [ratify skill](../plugin/skills/ratify/SKILL.md) describes how the agent presents each decision to the person. In this repository, AGENTS.md also asks the agent to show each term that a decision lists: the term's definition together with every glossary section that links to it.

### Retired rules and terms

A decision can retire a rule or a term that nobody should use any more. After ratification, its agreement status is retired. A retired rule or term keeps its section in the file, because active decisions still name it. It is a record of what those decisions meant, not something to use in new work.

The spec and the rules add these points:

- A retired rule or term never changes, and its ID or name is never used again (spec [4.1](Statute-spec.md#41-rules) and [4.3](Statute-spec.md#43-terms), rules HIST-004 and HIST-005). New behavior gets a new rule ID. A new meaning gets a new term.
- Only retired rules and active decisions may name a retired term (rule DEC-005).

## Links and fingerprints

Main terms: [Link](../statute/terms.md#link), [Code link](../statute/terms.md#code-link), [Test link](../statute/terms.md#test-link), [Fingerprint](../statute/terms.md#fingerprint), [Stale link](../statute/terms.md#stale-link), [Missing link](../statute/terms.md#missing-link). Also mentions: [Affected rule](../statute/terms.md#affected-rule), [Rule review](../statute/terms.md#rule-review), [Build state](../statute/terms.md#build-state), [Manual check](../statute/terms.md#manual-check), [Gap](../statute/terms.md#gap).

### Links

A **[link](../statute/terms.md#link)** connects a rule to a target in the code or in the tests. A target is a file, or a symbol in a file, such as a function, a type or a method ([spec 4.4](Statute-spec.md#44-links)). Each link also stores a fingerprint of its target, so that Statute can see when the target changes. Links are stored in `statute.lock`.

There are two kinds of link. They do different jobs.

A **[code link](../statute/terms.md#code-link)** connects the rule to the implementation that makes the rule true. When that code changes, the rule must be reviewed. The code link also tells the reviewer where to look.

A **[test link](../statute/terms.md#test-link)** connects the rule to a test that checks the rule's behavior. The test can catch a violation in code that the rule does not link to. Example: someone adds a new filter in some other function, and the filter breaks the rule. No code link of the rule sees this change, but the linked test fails. The project's CI runs the test. Statute does not run tests.

Example from this repository: rule LINKS-004 has code links to symbols such as `src/state.rs#health` and `src/commands/review.rs#run`. It has a test link to the whole file `tests/missing_contract.rs`.

A rule that is [built](../statute/terms.md#build-state) needs both kinds, or a [manual check](../statute/terms.md#manual-check) in place of the test link. See [Build state and gaps](#build-state-and-gaps).

### Fingerprints

A **[fingerprint](../statute/terms.md#fingerprint)** is a hash of a link target. The link's mode computes it, and `statute.lock` stores it as `sig`. A mode is the method that computes the hash. It depends on the kind of target: for example, a Rust function, a Go function, or any whole file.

The symbol modes, used for functions, types and methods, ignore comments and formatting. Any other change to the target's code changes the fingerprint, even when the behavior stays the same. [Spec 4.4](Statute-spec.md#44-links) lists the modes and what each one ignores.

This is on purpose: a missed behavior change is worse than a false alarm (AGENTS.md, decision-making principle 1).

A fingerprint shows only that the code changed. It cannot show whether the rule still holds. That is the job of a [rule review](../statute/terms.md#rule-review).

### Stale and missing links

A **[stale link](../statute/terms.md#stale-link)** is a link whose target now has a different fingerprint from the one recorded in `statute.lock`. A stale link brings its rule up for review: the rule becomes an [affected rule](../statute/terms.md#affected-rule). Only a review verdict refreshes a stale link (rule LINKS-003). See [Checking a change](#checking-a-change).

A **[missing link](../statute/terms.md#missing-link)** is a link where one of these is true:

- The target is not found. For example, the linked function was renamed, moved or deleted.
- The target matches more than one symbol.
- The target is in a file that does not parse cleanly, for example because of a syntax error. This is true even when the linked function itself is fine (rule LINKS-001).

A missing link also brings its rule up for review. `statute review` refuses a rule while one of its links is missing (rule LINKS-004). If the target moved, the agent first checks the rule, and then links the target at its new location.

Example from the [spec's usage scenarios](Statute-spec.md#appendix-usage-scenarios): an engineer renames the linked function `Publisher.Decide` to `Publisher.Choose`. The link to `Publisher.Decide` is now missing. The agent checks the rule, removes the old link, and links `Publisher.Choose`.

### What is not a link

A [gap](../statute/terms.md#gap) can refer to an issue in an external issue tracker. That reference is recorded in `statute/gaps.yml`. It is separate from the rule's code and test links. Gaps are explained in [Build state and gaps](#build-state-and-gaps).

## Build state and gaps

Main terms: [Build state](../statute/terms.md#build-state), [Gap](../statute/terms.md#gap). Also mentions: [Code link](../statute/terms.md#code-link), [Test link](../statute/terms.md#test-link), [Manual check](../statute/terms.md#manual-check), [Agreement status](../statute/terms.md#agreement-status), [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case).

### Build state

The [agreement status](../statute/terms.md#agreement-status) says whether people accepted a rule. The **[build state](../statute/terms.md#build-state)** says whether the rule is planned or built. An open gap on the rule shows broken behavior:

- **planned**: the rule is not built yet.
- **built**: the rule is built. A built rule needs a [code link](../statute/terms.md#code-link), and either a [test link](../statute/terms.md#test-link) or a [manual check](../statute/terms.md#manual-check) (rule CORE-003).
- **broken**: rule GAPS-001 reports a rule as broken when it is not retired, is built, and has at least one open gap. Statute computes "broken". It is never stored in the rule.

Nobody edits the build state by hand. `statute link` and `statute ratify` mark a rule built when it has everything that a built rule needs ([spec 4.1](Statute-spec.md#41-rules)).

Example from this repository: rule CORE-005 is agreed, but it is still planned. It has a manual check, but no code link yet.

These records do not guarantee that the rule holds. Code, documents and tests can all agree with each other and still be wrong. The [protected](../statute/terms.md#protected-case) and [boundary](../statute/terms.md#boundary-case) cases that people write for each rule protect against this ([spec 4.1](Statute-spec.md#41-rules)).

### Gaps

A **[gap](../statute/terms.md#gap)** is a known mismatch between a rule and the implementation. It is tracked as an issue until it is resolved. A person decides what to do: fix the implementation, or change the rule. To change the rule, the agent proposes a decision, as for any change to an agreed rule.

Example: suppose a change makes `statute check` stop reporting a renamed function as missing. Rule LINKS-004 no longer holds. The agent opens a gap for LINKS-004. Until the gap is closed, `statute status` shows LINKS-004 as broken.

Where gaps are kept ([spec 4.5](Statute-spec.md#45-gaps)):

- `statute/gaps.yml` lists every gap.
- In this repository, the gap itself is a Bean tagged `statute` and `gap` (see the "Issue tracker" section of AGENTS.md). The entry in `gaps.yml` is only an index: ID, rule and status.
- In a repository without an issue tracker, the entry in `gaps.yml` is the whole gap.

## Checking a change

Main terms: [Base](../statute/terms.md#base), [Affected rule](../statute/terms.md#affected-rule), [Rule review](../statute/terms.md#rule-review). Also mentions: [Stale link](../statute/terms.md#stale-link), [Missing link](../statute/terms.md#missing-link), [Protected case](../statute/terms.md#protected-case), [Boundary case](../statute/terms.md#boundary-case), [Gap](../statute/terms.md#gap), [Proposed decision](../statute/terms.md#proposed-decision), [Manual check](../statute/terms.md#manual-check).

Every change goes through the same loop:

1. Before coding, the agent runs `statute show` on the files it will change. It reads the rules and terms that come back.
2. After coding, `statute check` compares the work with the base and reports the affected rules.
3. The agent reviews each affected rule and records the result (a verdict).
4. `statute check --md` writes the "Rule impact" section for the PR.

The [change skill](../plugin/skills/change/SKILL.md) has the exact steps.

### Base

The **[base](../statute/terms.md#base)** is the commit that `statute check` compares against. It is the merge-base of `HEAD` with the default branch: the newest commit that your branch and the default branch have in common. If there is no default branch, the base is `HEAD`.

Changes since the base include:

- the commits on your branch,
- files that you changed but did not commit,
- untracked files: new files that are not in git yet.

### Affected rule

An **[affected rule](../statute/terms.md#affected-rule)** is a rule that a change brings up for review. For example, a [stale link](../statute/terms.md#stale-link) or a [missing link](../statute/terms.md#missing-link) brings its rule up for review.

Affected does not mean broken. It means that the rule needs attention: someone must look at it. [Spec 5](Statute-spec.md#5-cli) lists all the reasons that the "Rule impact" section uses to mark a rule as affected.

### Rule review

A **[rule review](../statute/terms.md#rule-review)** is the agent's assessment of whether an affected rule still holds after a change. The agent reads the rule and its implementation. It tries the rule's [protected](../statute/terms.md#protected-case) and [boundary](../statute/terms.md#boundary-case) cases. Then it records one of three results. The change skill uses these commands:

| Result | What happens next | Command |
| --- | --- | --- |
| The rule holds | No other step. | `statute review RULE ok` |
| The rule changes on purpose | The agent proposes a decision that changes or retires the rule (a [proposed decision](../statute/terms.md#proposed-decision)). | `statute review RULE changed D…` |
| The rule is broken | The agent opens a [gap](../statute/terms.md#gap). | `statute review RULE gap REF` |

When `statute review` accepts the verdict, it refreshes all stale links of the rule (rule LINKS-003).

Who does what in a rule review:

- The tool finds changed links and runs the mechanical checks. It does not judge the behavior.
- The agent does the review.
- A person does any [manual check](../statute/terms.md#manual-check), and approves any decision that changes the rule.

Example from this repository: a change edits the code of the function `health` in `src/state.rs`, not only its comments. Rules LINKS-001, LINKS-002 and LINKS-004 link to it. Their links to `health` become stale, so the three rules are affected. The agent reviews each rule. If all three still hold, it runs `statute review` with `ok` for each one, and the links are refreshed.

## Keeping this glossary correct

A PR that changes this glossary, or the text, `code` or `not` of a term in `terms.md`, includes a glossary consistency review. The reviewer compares each changed glossary section with the definitions of the terms it explains, and each changed definition with the glossary sections that link to it. The "Glossary" section of AGENTS.md describes the review and its record. A consistency review is not a rule review, and it approves nothing.

## Index

| Term | Explained in |
| --- | --- |
| [Affected rule](../statute/terms.md#affected-rule) | [Checking a change](#checking-a-change) |
| [Agreement status](../statute/terms.md#agreement-status) | [Agreement and approval](#agreement-and-approval) |
| [Approval](../statute/terms.md#approval) | [Agreement and approval](#agreement-and-approval) |
| [Base](../statute/terms.md#base) | [Checking a change](#checking-a-change) |
| [Boundary case](../statute/terms.md#boundary-case) | [What a rule says](#what-a-rule-says) |
| [Build state](../statute/terms.md#build-state) | [Build state and gaps](#build-state-and-gaps) |
| [Code link](../statute/terms.md#code-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Decision](../statute/terms.md#decision) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Draft rule](../statute/terms.md#draft-rule) | [Agreement and approval](#agreement-and-approval) |
| [Fingerprint](../statute/terms.md#fingerprint) | [Links and fingerprints](#links-and-fingerprints) |
| [Gap](../statute/terms.md#gap) | [Build state and gaps](#build-state-and-gaps) |
| [Link](../statute/terms.md#link) | [Links and fingerprints](#links-and-fingerprints) |
| [Manual check](../statute/terms.md#manual-check) | [What a rule says](#what-a-rule-says) |
| [Missing link](../statute/terms.md#missing-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Proposed decision](../statute/terms.md#proposed-decision) | [Agreement and approval](#agreement-and-approval) |
| [Protected case](../statute/terms.md#protected-case) | [What a rule says](#what-a-rule-says) |
| [Rule](../statute/terms.md#rule) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Rule review](../statute/terms.md#rule-review) | [Checking a change](#checking-a-change) |
| [Stale link](../statute/terms.md#stale-link) | [Links and fingerprints](#links-and-fingerprints) |
| [Term](../statute/terms.md#term) | [Terms, rules and decisions](#terms-rules-and-decisions) |
| [Test link](../statute/terms.md#test-link) | [Links and fingerprints](#links-and-fingerprints) |
