---
prefix: LINKS
---

# LINKS-001 A symbol link requires a clean parse

A symbol link is missing when its file does not parse cleanly, even if the linked declaration itself is valid.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Links using a symbol fingerprint mode.
terms: [Link]
exceptions: []
protects:
  - A link to foo.py#bar is missing when foo.py has a syntax error elsewhere in the file, even if bar itself is valid.
boundary:
  - An existing file linked with bytes-sha256-v1 is not missing solely because its contents have a syntax error; that mode does not parse the file.
```

# LINKS-002 A changed link target makes its link stale

`check` must report a link as stale when the current fingerprint of its target differs from the fingerprint recorded in `statute.lock`. This applies to code links and test links alike.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Links whose target changed since the base, or every link with `check --all`.
terms: [Link, Stale link, Code link, Test link, Base]
exceptions: []
protects:
  - Changing the body of a linked Go function makes its code link stale.
  - Weakening a linked test makes its test link stale.
boundary:
  - A comment or formatting change in a function linked with a symbol mode leaves its link fresh.
```

# LINKS-003 Only a review verdict refreshes a stale link

The recorded fingerprint of an existing link must change only through `statute review` with a verdict that `review` accepts: `ok`, `changed D…` when the decision lists the rule in `changes` or `retires`, or `gap REF` when the rule has that open gap. A refused verdict leaves `statute.lock` unchanged.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: ""
terms: [Link, Stale link, Rule review, Decision, Gap]
exceptions: []
protects:
  - Linking an already linked target again is refused, so `link` cannot refresh a stale link.
  - "`review RULE changed D…` is refused when the decision does not list the rule, and the link stays stale."
boundary:
  - "`review RULE ok` refreshes every stale link of that rule at once."
```

# LINKS-004 A link whose target is gone or ambiguous is missing

`check` must report a link as missing when its target file does not exist, its symbol is not found, or more than one symbol matches it. `review` must refuse a rule while any of its links is missing, so the link stays reported until the file is fixed or the target is relinked.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built
scope: Links whose target changed since the base, or every link with `check --all`.
terms: [Link, Missing link, Rule review]
exceptions: []
protects:
  - A link to a Go function that was renamed is reported as missing.
  - "`review RULE ok` is refused while one of the rule's links is missing, and `statute.lock` is unchanged."
boundary:
  - Removing the missing link and linking the new location clears the finding.
```
