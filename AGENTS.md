# Statute

Statute is a Rust CLI and agent plugin that keeps a product's rules written down, linked to code, and checked on every change.

- What and why: [docs/One-pager.md](docs/One-pager.md)
- Behaviour, file formats, commands: [docs/Statute-spec.md](docs/Statute-spec.md) (source of truth)
- Code structure: [docs/Architecture.md](docs/Architecture.md)

## Module layering

Statute is one crate, so the compiler does not stop a module from importing any other. These rules keep the layers apart. Each module may import only the modules listed for it (and external crates). `tests/layering.rs` reads this table and fails on an import it does not allow, so keep its format.

| Module | May import | I/O |
| --- | --- | --- |
| `model` | nothing internal | none |
| `store` | `model` | none: text in, values or text out |
| `fingerprint` | `model` (the `Mode` type only) | none: bytes in, hashes out |
| `state` | `model`, `fingerprint` types | none |
| `check` | `model`, `state` | none |
| `impact` | `model`, `state` | none |
| `git` | nothing internal | runs `git`; the only module that spawns processes |
| `repo` | `model`, `store`, `git` | reads files; the only module that reads the filesystem |
| `txn` | `model`, `store`, `check`, `repo` | writes files; the only module that writes the filesystem |
| `commands` | everything above | through `repo`, `txn` and `git` only |
| `cli` | `commands`, `model` | prints output |
| `main.rs` | `cli` | sets the exit code |

Rules:

- Only `cli` imports `commands`, and only `main.rs` imports `cli`.
- No module outside `git`, `repo` and `txn` calls `std::fs`, `std::process` or reads environment variables. The one exception is `cli`, which launches `$EDITOR` for `decide --edit` and passes the result to `commands`. If a pure module needs data from outside, such as the current time or a random ID, its caller passes it in.
- Shared types belong in `model`, not in the module that first needed them. If two modules on the same layer need each other, move the shared part down.
- `commands` modules do not call each other. Logic used by two commands moves into a lower module.
- `cli` holds argument parsing and rendering only. Anything that decides an outcome or a finding goes in `commands` or below.

When a change needs an import these rules forbid, change the rules here first, with the reason, in the same PR.

## Extension

New fingerprint modes, link sources and impact reasons are new enum variants (docs/Architecture.md, section 8). Do not add traits, registries or plugin mechanisms for them.

Implementation is still in progress. This branch supports `init`, `rule new`, `show` for rule IDs, terms, decision IDs, files and symbols, `check [--all]`, whole-file and Go/Python symbol `link`, `review`, `check --md`, `decide`, `ratify`, and `status`. Gap commands and the `change` skill are not implemented yet; the snippet below describes the intended workflow. Check `statute --help` for the installed build.

Use `statute show` to inspect linked rules and terms, `statute decide` to record proposed decisions, and `statute ratify` to record a person's explicit approval of specific decisions. Report workflow steps that the current build cannot perform. Agreement and build states are set by the CLI.

<!-- statute:begin -->
## Statute

- Before changing code, run `statute show` on the files you will touch and read the rules and terms it returns.
- Change a rule or an agreed term only together with a proposed decision that lists it (`statute decide`), never to match the code. Never edit `agreed` or `built`; the CLI sets them.
- When a user instruction conflicts with an agreed rule, name the rule and propose a decision before changing the code.
- After changing code, run the `change` skill before declaring the task done.
- Anything you create is `proposed`. Run `statute ratify` only after a human in this session explicitly approves those specific decisions.
- Before asking a human to ratify a rule, check it: what valid case would it reject? What invalid case would it allow? Does its meaning change with actor or permission, missing data, lifecycle stage, time, retries, concurrency or partial failure? Add to the rule only the answers that change it.
- Gaps go to the issue tracker named in this file; if none is named, use `statute gap open --note`.
- When instructions conflict: explicit user instruction > current task > project principles > agreed rules, terms and ratified decisions > repo conventions > proposed decisions > agent judgment.
<!-- statute:end -->
