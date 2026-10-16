---
prefix: ATTR
---

# ATTR-001 Valid but duplicated

A user can open a chat about a Session.

```yaml
agreed: proposed
built: planned
terms: [Session]
```

# ATTR-001 The duplicate

Same ID.

```yaml
agreed: proposed
built: planned
```

## Subheading inside a rule

# Missing ID

No ID in the heading.

```yaml
agreed: proposed
built: planned
```

# BILL-009 Wrong prefix

Uses another file's prefix.

```yaml
agreed: proposed
built: planned
```

# ATTR-002 No block

Statement only.

# ATTR-003 Typo in a key

Statement.

```yaml
agreed: proposed
built: planned
protect: []
```

# ATTR-004 No statement

```yaml
agreed: proposed
built: planned
```

# ATTR-005 Agreed without cases or decision

Statement.

```yaml
agreed: agreed
built: planned
terms: [Sesion]
```

# ATTR-006 Established twice

Statement.

```yaml
agreed: proposed
built: planned
```
