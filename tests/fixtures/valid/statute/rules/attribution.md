---
prefix: ATTR
---

Rules for linking Sessions to commits.

# ATTR-001 A Session without a repository is still kept

The collector keeps a Session that has no Repository, and stores it like any other Session.

```yaml
agreed: agreed            # proposed | agreed | retired
built: planned            # planned | built   (broken is derived, see 4.6)
scope: Session collection
terms: [Session]
exceptions: []
protects:
  - A Session started outside any git repository is stored and listed.
boundary:
  - A Session the developer deleted is not kept; deletion still applies.
```

Notes: this came out of the collector filter incident.

# ATTR-002 Context alone creates no contribution

HEAD, branch, time, path overlap and collector score never create a Session–commit contribution link.

```yaml
agreed: proposed
built: planned
terms: [Session, Commit]
protects:
  - A Session active on the same branch an hour before abc123, with no other evidence, does not contribute to abc123.
boundary:
  - A Session that ran `git commit` for abc123 may still contribute to abc123.
```

# ATTR-003 Sessions expire after a day

A Session with no activity for 24 hours is closed.

```yaml
agreed: retired
built: planned
terms: [Session]
protects:
  - A Session idle for 25 hours is closed.
boundary:
  - A Session idle for 23 hours stays open.
```
