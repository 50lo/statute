---
prefix: CORE
---

# CORE-001 An agreed rule describes one checkable requirement

An agreed rule must describe one requirement that a person or test can check. It must include a protected case showing behavior to preserve and a boundary case showing where the requirement stops applying.

```yaml
agreed: agreed          # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Protected case, Boundary case]
exceptions: []
manual_check: >-
  Before approving a new or changed rule, a person reads its statement and cases.
  Confirm that it states one requirement, that someone can describe behavior that
  satisfies it and behavior that violates it, and that its protected and boundary
  cases match its intended scope. Ask for a rewrite or split before approval if
  the requirement is vague, combines unrelated requirements, or has misleading cases.
protects:
  - An agreed rule without a protected case or a boundary case is reported as invalid.
  - A rule that combines unrelated requirements must be split before it becomes agreed.
boundary:
  - A draft can remain proposed while its statement and cases are being written.
```

Partial automated coverage is in `tests/core_rule_cases.rs`. Its test link is
deferred until Beans bug `st-8j2x` supports rules that need both automated tests
and a manual check.

# CORE-002 Agreement and implementation are recorded separately

Agreement status and build state must record separate facts. Approval alone must not mark a rule built, and implementation alone must not mark it agreed.

```yaml
agreed: agreed          # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Agreement status, Build state, Approval]
exceptions: []
protects:
  - Approving a rule with no implementation or verification links leaves it planned.
  - Adding all required links to a proposed rule does not make it agreed.
boundary:
  - A rule can be agreed but planned, or proposed but built.
  - A rule can become agreed and built in the same operation when the separate conditions for both are met.
```

# CORE-003 A built rule has implementation and verification links

A built rule must have at least one code link and at least one test link. A stated manual check can replace the test link when an automated test cannot judge the behavior.

```yaml
agreed: agreed          # proposed | agreed | retired
built: built              # planned | built
scope: Rules that are not retired.
terms: [Rule, Build state, Code link, Test link, Manual check]
exceptions: []
protects:
  - A built rule with test links but no code link is incomplete.
  - A built rule with a code link but neither a test link nor a manual check is incomplete.
boundary:
  - A rule with a code link and a stated manual check can be built without a test link when an automated test cannot judge its behavior.
  - A planned rule can have no links while its implementation is being prepared.
```

# CORE-004 Agent additions start as proposed

New rules and terms written by an agent must start as proposed. The agent must not present its own additions as already agreed.

```yaml
agreed: proposed        # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Term, Agreement status]
exceptions: []
manual_check: >-
  When changing the agent instructions for adding rules or terms, ask the agent
  to add a rule and a term for behavior that already has implementation and tests.
  Before giving approval, a person inspects the additions, the proposed decision
  and the agent's response. Confirm that the additions are proposed and that the
  agent does not call them agreed. After approving that specific decision, confirm
  that approval agrees only the rules and terms named by the approved decisions.
protects:
  - A rule or term created by an agent starts as proposed even when the implementation and tests already exist.
boundary:
  - A proposed rule or term can become agreed after a person explicitly approves the decision that names it.
```

Partial automated coverage is in `tests/core_lifecycle.rs`. Its test link for
this rule is deferred until Beans bug `st-8j2x` is fixed.

# CORE-005 Only a person can approve a decision

Only a person can approve a proposed decision. An agent may record that approval through the tool only after the person explicitly approves that specific decision.

```yaml
agreed: proposed        # proposed | agreed | retired
built: planned            # planned | built
scope: ""
terms: [Decision, Approval]
exceptions: []
manual_check: >-
  Before each approval, a person checks the conversation for explicit human
  approval of every decision the agent intends to approve. Match the approval to
  each decision's ID or unambiguous proposed contents. Passing tests, the agent's
  own assessment and approval of another decision do not count. If approval is
  missing or unclear for any selected decision, obtain it before approving that decision.
protects:
  - An agent's confirmation that tests pass does not count as human approval of a decision.
  - Approval of one decision does not authorize the agent to activate another decision.
boundary:
  - After a person explicitly approves a specific decision, the agent may run statute approve for that decision to record the approval.
```

Partial automated coverage is in `tests/core_lifecycle.rs`. Its test link for
this rule is deferred until Beans bug `st-8j2x` is fixed.
