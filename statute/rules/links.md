---
prefix: LINKS
---

# LINKS-001 A symbol link requires a clean parse

A symbol link is missing when its file does not parse cleanly, even if the linked declaration itself is valid.

```yaml
agreed: proposed          # proposed | agreed | retired
built: built              # planned | built
scope: Links using a symbol fingerprint mode.
terms: [Link]
exceptions: []
protects:
  - A link to foo.py#bar is missing when foo.py has a syntax error elsewhere in the file, even if bar itself is valid.
boundary:
  - An existing file linked with bytes-sha256-v1 is not missing solely because its contents have a syntax error; that mode does not parse the file.
```
