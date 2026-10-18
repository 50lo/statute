---
prefix: DEC
---

# DEC-001 A rule or term is in at most one proposed decision

A rule or term must not be listed in more than one proposed decision. `decide` refuses a decision that lists a rule or term already in a proposed decision, and `check` reports an overlap that arrives through a merge.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Term, Proposed decision]
exceptions: []
protects:
  - "`decide --terms Commit` is refused while another proposed decision lists `Commit`."
  - Two proposed decisions listing the same rule after a merge are reported as an error.
boundary:
  - An agreed rule established by an active decision can be listed in a new proposed decision.
```

# DEC-002 Ratify activates only the decisions it names

`statute ratify` must make active only the decisions named on its command line, and change the agreement status only of the rules and terms those decisions list. Other proposed decisions and their rules and terms stay proposed.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Decision, Proposed decision, Agreement status, Approval]
exceptions: []
protects:
  - Ratifying one of two proposed decisions leaves the other decision and the rules it changes proposed.
boundary:
  - Several decisions named in one `ratify` call all become active together.
```

# DEC-003 Ratify writes nothing when its items would have errors

`statute ratify` must refuse and write nothing when `check` would report an error about the decisions it ratifies or their rules and terms afterwards, including a rule that would become agreed while using a term that is not agreed. Stale and missing links never block it, because agreement is recorded separately from whether the code still matches.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Decision, Rule, Term, Approval]
exceptions: []
protects:
  - Ratifying a decision that establishes a rule without protected or boundary cases is refused, and the decision and rule files are unchanged.
  - Ratifying a rule whose term is proposed in another decision is refused.
boundary:
  - A stale or missing link on a rule the decision lists does not block `ratify`, so a rule can be approved while its code is mid-change, and retired after its code was deleted.
  - An error about a rule the decision does not list does not block `ratify`.
```
