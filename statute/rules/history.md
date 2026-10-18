---
prefix: HIST
---

# HIST-001 Active decisions are never edited or deleted

`check` must report an error when a decision that is active in the base is edited or deleted. A wrong active decision is corrected by a new decision that changes or retires its rules.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Decisions that are active in the base.
terms: [Decision, Base]
exceptions: []
protects:
  - Rewording the summary of a decision that is active in the base is reported as an error.
  - Deleting a decision that is active in the base is reported as an error.
boundary:
  - A proposed decision can be edited or deleted without a finding; deleting it is how a rejected proposal is removed.
```

# HIST-002 An agreed rule changes only with a decision that lists it

When a rule is agreed in the base, `check` must report an error if its statement, `scope`, `terms`, `exceptions`, `protects`, `boundary` or `manual_check` changes, or the rule is removed, unless a decision that is proposed or became active since the base lists the rule in `changes`.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Rules that are agreed in the base.
terms: [Rule, Decision, Agreement status, Base]
exceptions: []
protects:
  - Rewording an agreed rule's statement with no decision that lists it is reported as an error.
  - A change carried only by a decision that was already active in the base is reported as an error.
boundary:
  - Changing an agreed rule's notes or the whitespace of its statement needs no decision.
```

# HIST-003 An agreed term changes only with a decision that lists it

When a term is agreed in the base, `check` must report an error if its definition, `code` or `not` changes, or the term is removed, unless a decision that is proposed or became active since the base lists the term in `terms`. A term's definition is everything in its section after the heading except the `yaml` block, compared with whitespace collapsed.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Terms that are agreed in the base.
terms: [Term, Decision, Agreement status, Base]
exceptions: []
protects:
  - Adding a code name to an agreed term with no decision that lists it is reported as an error.
  - Rewording a paragraph or list item after an agreed term's `yaml` block with no decision that lists the term is reported as an error.
boundary:
  - Changing an agreed term's `aliases` or `avoid` words needs no decision.
  - Changing the whitespace of an agreed term's definition, or a comment in its `yaml` block, needs no decision.
```

# HIST-004 A retired rule never changes

When a rule is retired in the base, `check` must report an error if its statement, `scope`, `terms`, `exceptions`, `protects`, `boundary` or `manual_check` changes, its `agreed` value is no longer `retired`, or the rule is removed, even when a decision lists it. `decide` refuses a decision that lists a retired rule in `changes` or `retires`, and writes nothing.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Rules that are retired in the base, and decisions drafted with `decide`.
terms: [Rule, Decision, Agreement status, Base]
exceptions: []
protects:
  - Rewording a retired rule's statement is reported as an error, even when a proposed decision lists the rule in `changes`.
  - "Setting a retired rule back to `agreed: proposed`, or deleting its section, is reported as an error."
  - "`decide --changes ATTR-003` is refused while ATTR-003 is retired, and nothing is written."
boundary:
  - Changing a retired rule's notes, the whitespace of its statement, or its `built` value gives no finding.
  - A rule that a proposed decision retires is still agreed until the decision is ratified, so HIST-002 governs its changes until then.
```
