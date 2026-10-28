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

# DEC-002 Approve activates only the decisions it names

`statute approve` must make active only the decisions named on its command line, and change the agreement status only of the rules and terms those decisions list. Other proposed decisions and their rules and terms stay proposed.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Decision, Proposed decision, Agreement status, Approval]
exceptions: []
protects:
  - Approving one of two proposed decisions leaves the other decision and the rules it changes proposed.
boundary:
  - Several decisions named in one `approve` call all become active together.
```

# DEC-003 Approve writes nothing when its items would have errors

`statute approve` must refuse and write nothing when `check` would report an error about the decisions it approves or their rules and terms afterwards, including a rule that would become agreed while using a term that is not agreed, and any rule that is not retired and still uses a term the decisions retire. Stale and missing links never block it, because agreement is recorded separately from whether the code still matches.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Decision, Rule, Term, Approval]
exceptions: []
protects:
  - Approving a decision that establishes a rule without protected or boundary cases is refused, and the decision and rule files are unchanged.
  - Approving a rule whose term is proposed in another decision is refused.
  - Retiring Session is refused while ATTR-001, which the approved decisions do not list, still uses Session, and no file changes.
boundary:
  - A stale or missing link on a rule the decision lists does not block `approve`, so a rule can be approved while its code is mid-change, and retired after its code was deleted.
  - An error about a rule the decision does not list does not block `approve`, unless the rule uses a term the decision retires.
  - Retiring Session succeeds when the same `approve` call changes every rule that uses it to a successor term that is agreed or approved in that call.
```

# DEC-004 A decision retires only an agreed term that it does not redefine

`decide` and `approve` must refuse, and write nothing, when a term listed in `retires_terms` is not agreed, or is also listed in `terms` by the same decision or, for `approve`, by another decision in the same call.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Decisions drafted with `decide`, and decisions named to `approve`.
terms: [Decision, Term, Agreement status, Proposed decision]
exceptions: []
protects:
  - "`decide --retires-terms Repository` is refused while Repository is proposed and was never agreed, and nothing is written."
  - "`decide --terms Session --retires-terms Session` is refused, and nothing is written."
  - Approving, in one call, a decision that redefines Session and another that retires it is refused, and nothing is written.
boundary:
  - "`decide --retires-terms Session` is accepted while Session is agreed and rules still use it, and Session stays `agreed: agreed` until the decision is approved."
```

# DEC-005 Only retired rules and active decisions name a retired term

`check` must report an error when a rule that is not retired lists a retired term in `terms`, or a proposed decision lists a retired term in `terms` or `retires_terms`, and `approve` must refuse a decision that names a retired term.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Term, Decision, Proposed decision, Agreement status]
exceptions: []
protects:
  - A proposed rule that lists a retired term in `terms` is reported as an error.
  - A proposed decision that lists a retired term in `terms` is reported as an error, and `approve` refuses it.
boundary:
  - A retired rule or an active decision that names a retired term gives no finding.
  - A rule can keep using a term while a proposed decision retires it; the error appears once the term is retired.
```
