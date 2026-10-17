# Statute agent plugin

Skills and a hook that let a coding agent keep a repository's Statute rules current. They need the `statute` CLI on the `PATH`:

```sh
cargo install --git https://github.com/50lo/statute
```

| Part | What it does |
| --- | --- |
| `skills/adopt` | Sets Statute up in a repository from its existing docs |
| `skills/change` | After a code change: reviews the rules whose linked code changed |
| `skills/decide` | Proposes a decision when a rule or term needs to change |
| `skills/ratify` | Shows pending decisions to a person and records the approval |
| `hooks/stop.sh` | Claude Code Stop hook: asks the agent to run `change` when links are stale or missing |
| `agents-snippet.md` | The AGENTS.md section that `statute init` writes |

## Claude Code

Install this directory as a plugin; it brings the four skills and the Stop hook. The hook runs `statute check --json` at the end of each turn and, when links are stale or missing, asks the agent once to run the `change` skill. It does nothing outside a repository with `statute.yml`.

## Codex

Codex uses the same skills without the hook. Copy or symlink each directory under `skills/` into the folder Codex loads skills from. The AGENTS.md section tells Codex to run the `change` skill after changing code.
