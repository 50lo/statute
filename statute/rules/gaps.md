---
prefix: GAPS
---

# GAPS-001 A built rule with an open gap is broken

A rule that is not retired, has `built: built` and has at least one open gap must be reported as broken. Broken is derived from the gaps file and never stored in the rule.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Rule, Gap, Build state]
exceptions: []
protects:
  - Opening a gap on a built rule makes `status` report it as broken while `built` stays `built`.
boundary:
  - A rule whose gaps are all closed is not broken.
  - A planned rule with an open gap is not broken.
```
