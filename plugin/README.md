# Statute agent plugin

Skills and a hook that let a coding agent keep a repository's Statute rules current. They need the `statute` CLI on the `PATH`:

```sh
cargo install statute-cli --locked
```

| Part | What it does |
| --- | --- |
| `skills/adopt` | Sets Statute up in a repository from its existing docs |
| `skills/change` | After a code change: reviews the rules whose linked code changed |
| `skills/decide` | Proposes a decision when a rule or term needs to change |
| `skills/ratify` | Shows pending decisions to a person and records the approval |
| `hooks/stop.sh` | Shared Stop hook: asks the agent to run `change` when links are stale or missing |
| `agents-snippet.md` | The AGENTS.md section that `statute init` writes |

## Claude Code

Install this directory as a plugin; it brings the four skills and the Stop hook. The hook runs `statute check --json` at the end of each turn and, when links are stale or missing, asks the agent once to run the `change` skill. It does nothing outside a repository with `statute.yml`.

## Codex

This directory is also a Codex plugin: `.codex-plugin/plugin.json` packages the same four skills and Stop hook. The repository's `.agents/plugins/marketplace.json` exposes it as **Statute**.

1. Register this checkout as a local marketplace: `codex plugin marketplace add /absolute/path/to/statute`. Restart Codex desktop to load it.
2. Open the Plugins Directory, select **Statute**, and install the `statute` plugin.
3. Review and trust its Stop hook when prompted. Installing the plugin alone does not trust hooks.
4. Start a new chat and ask Codex to use the Statute `adopt`, `change`, `decide` or `ratify` skill. The CLI must be on the execution environment's `PATH`.

The hook runs `statute check --json` at the end of a turn. When links are stale or missing it asks Codex once to run `change`; it stays quiet outside a Statute repository and when the CLI is unavailable. The AGENTS.md section written by `statute init` also tells Codex to run `change` after code changes. Only a person's explicit approval allows the `ratify` skill to activate a decision.

Plugin lifecycle hooks currently require a manual installation in Codex desktop. For a client without plugin hooks, copy or symlink the four directories under `skills/` into `.agents/skills/` in the target repository; the AGENTS.md instructions provide the after-change workflow there.

Registering the marketplace makes it available from other projects too. After editing the package, refresh the installed plugin and restart the app to load the updated files.

Packaging and hook trust follow the [Codex plugin documentation](https://developers.openai.com/plugins/build/plugins) and [hook documentation](https://learn.chatgpt.com/docs/hooks).
