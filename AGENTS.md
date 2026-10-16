# Statute

Statute is a Rust CLI and agent plugin that keeps a product's rules written down, linked to code, and checked on every change.

- What and why: [docs/One-pager.md](docs/One-pager.md)
- Behaviour, file formats, commands: [docs/Statute-spec.md](docs/Statute-spec.md) (source of truth)
- Code structure: [docs/Architecture.md](docs/Architecture.md)

## Module layering

Statute is one crate, so the compiler does not stop a module from importing any other. These rules keep the layers apart. Each module may import only the modules listed for it (and external crates).

| Module | May import | I/O |
| --- | --- | --- |
| `model` | nothing internal | none |
| `store` | `model` | none: text in, values or text out |
| `fingerprint` | nothing internal | none: bytes in, hashes out |
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

`tests/layering.rs` reads the table above and fails on any import it does not allow, and on `std::fs`, `std::process` or `std::env` outside `git`, `repo`, `txn`, `cli` and `main.rs`.

## Extension

New fingerprint modes, link sources and impact reasons are new enum variants (docs/Architecture.md, section 8). Do not add traits, registries or plugin mechanisms for them.

## Before pushing

Run what CI runs:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
