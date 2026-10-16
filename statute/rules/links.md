---
prefix: LINKS
---

# LINKS-001 Link target must parse cleanly

Link target must parse cleanly, otherwise link is considered missing.

```yaml
agreed: agreed          # proposed | agreed | retired
built: planned            # planned | built
scope: ""
terms: [Link]
exceptions: []
protects:
  - Link targeting foo.py with correct Python syntax is not missing
boundary:
  - Links targeting bar.py with JSON content instead of Python is a missing link
```
