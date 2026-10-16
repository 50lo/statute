# Sessions and commits

## Session

One conversation between a developer and a coding agent.

```yaml
agreed: agreed                   # proposed | agreed
code: [Session, SessionID]
not:
  - {name: sessions, means: the sign-in sessions table}
aliases: [agent session]
avoid: [chat]
```

## Commit

A git commit in a repository the collector watches.

```yaml
agreed: agreed
code: [Commit]
```
