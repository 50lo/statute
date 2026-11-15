## Term

A word or phrase used in the product, with a definition and the names used for it in code. A term also distinguishes names that look alike but mean different things.

```yaml
agreed: agreed
code: [Term, TermMeta]
```

## Rule

A statement of one thing that must be true about the product, and when. A person or test can check whether the product follows it.

```yaml
agreed: agreed
code: [Rule, RuleMeta]
```

## Decision

A recorded choice about rules or terms, with the reason for it. It starts as a proposed decision, which a person has not approved yet: the rules it establishes or changes and the terms it introduces or redefines stay proposed, while a rule or term it retires stays agreed and in force. A proposed decision can be edited or deleted, and it never blocks a merge. Once a person approves it, it becomes active and records who approved it.

```yaml
agreed: agreed
code: [Decision, DecisionDraft, "DecisionStatus::Proposed"]
aliases: [pending decision]
```

## Link

A connection from a rule to a code or test target, with a fingerprint used to detect changes to that target. A code link points to the implementation that makes the rule true; a change there brings the rule up for review and tells the reviewer where to look. A test link points to a test that checks the rule's behavior; the test can catch a violation introduced outside the linked implementation, and the project's CI runs it.

```yaml
agreed: agreed                   # proposed | agreed | retired
code: [Link, LinkLine, "LinkRole::Code", "LinkRole::Test"]
```

A reference to an issue in an external tracker, recorded in gaps.yml, is separate from a rule's code or test links.

## Code link

A link to the implementation that makes a rule true. A change there brings the rule up for review and tells the reviewer where to look.

```yaml
agreed: retired
code: ["LinkRole::Code"]
```

## Test link

A link to a test that checks a rule's behavior. The test can catch a violation introduced outside the linked implementation. The project's CI runs the test.

```yaml
agreed: retired
code: ["LinkRole::Test"]
```

## Manual check

A stated procedure for a person to verify the behavior of a rule that an automated test cannot judge. It can be the rule's only verification or sit beside test links. A test link never replaces or completes it, so it stays required while it is stated. The rule still needs a code link. Statute records the procedure, not whether a person has followed it.

```yaml
agreed: agreed
code: [manual_check]
```

## Protected case

A concrete case showing behavior the rule must preserve. It gives a person or agent a case to try when checking whether the rule still holds.

```yaml
agreed: agreed
code: [protects]
```

## Boundary case

A nearby case showing where the rule stops applying, so it does not reject valid behavior outside its intended scope.

```yaml
agreed: agreed
code: [boundary]
```

## Gap

A known mismatch between a rule and the implementation, tracked as an issue until it is resolved. A person decides whether to fix the implementation or change the rule.

```yaml
agreed: agreed
code: [Gap, GapStatus]
```

## Agreement status

Whether a rule or term is proposed, agreed or retired. Agreement records acceptance separately from whether the code implements the rule.

```yaml
agreed: agreed
code: [Agreed, TermAgreed]
```

## Build state

Whether a rule is planned or built, with an open gap indicating broken behavior. A built rule needs a code link, and a test link, a manual check or both. These records do not guarantee that the rule holds, and do not show that a manual check was done.

```yaml
agreed: agreed
code: [Built]
```

## Affected rule

A rule that a change brings up for review. Being affected means the rule needs attention; it does not itself establish that the rule is broken.

```yaml
agreed: retired
code: [RuleImpact]
```

## Rule review

An assessment by the agent of whether an affected rule still holds after a change. A rule is affected when a change brings it up for review; being affected means the rule needs attention and does not itself establish that it is broken. The agent reads the rule and implementation, tries the protected and boundary cases, and records whether the rule holds, changes on purpose, or is broken. The tool finds changed links and performs mechanical checks; it does not judge the behavior. A person performs any manual check and approves decisions to change the rule.

```yaml
agreed: agreed
code: ["review::Verdict", ReviewReport, RuleImpact]
```

## Approval

A person's explicit acceptance of a specific proposed decision. An agent can prepare the proposal and perform the checks, but cannot approve its own proposal. Once a person approves, the agent runs `statute approve` to record the decision as active and update its rules and terms.

```yaml
agreed: agreed
code: [ApprovedBy, approved_by]
```

## Base

The commit that `check` compares against: the merge-base of `HEAD` with the default branch, or `HEAD` when there is no default branch. Changes since the base include the branch's commits and the uncommitted and untracked files in the working tree.

```yaml
agreed: agreed
code: [git::base]
aliases: [merge-base]
```

## Stale link

A link whose target's current fingerprint differs from the one recorded in `statute.lock`. It brings its rule up for review, and only a review verdict refreshes it.

```yaml
agreed: agreed
code: [Health::Stale]
```

## Missing link

A link whose target is not found, matches more than one symbol, or sits in a file that does not parse cleanly. It brings its rule up for review; a moved target is relinked at its new location after the rule is checked.

```yaml
agreed: agreed
code: [Health::Missing]
```

## Fingerprint

A hash of a link target, computed by the link's mode, recorded in `statute.lock` as `sig`. Symbol modes ignore comments and formatting; any other change to the target's code alters the fingerprint, even when behavior stays the same.

```yaml
agreed: agreed
code: [fingerprint::sign, sig]
avoid: [checksum]
```

## Draft rule

A proposed rule that no decision establishes yet. It is a pending item that `status` lists, not a finding.

```yaml
agreed: agreed
code: [is_draft]
```

## Proposed decision

A decision that a person has not approved yet. The rules it establishes or changes and the terms it introduces or redefines stay proposed until it is approved, while a rule or term it retires stays agreed and in force until then. It can be edited or deleted, and it never blocks a merge.

```yaml
agreed: retired
code: [DecisionStatus::Proposed]
aliases: [pending decision]
```
