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
| `ops` | everything above | through `repo`, `txn` and `git` only |
| `cli` | `ops`, `model` | prints output |
| `main.rs` | `cli` | sets the exit code |

Rules:

- Only `cli` imports `ops`, and only `main.rs` imports `cli`.
- No module outside `git`, `repo` and `txn` calls `std::fs`, `std::process` or reads environment variables. If a pure module needs data from outside, its caller passes it in.
- Shared types belong in `model`, not in the module that first needed them. If two modules on the same layer need each other, move the shared part down.
- `ops` modules do not call each other. Logic used by two commands moves into a lower module.
- `cli` holds argument parsing and rendering only. Anything that decides an outcome or a finding goes in `ops` or below.

When a change needs an import these rules forbid, change the rules here first, with the reason, in the same PR.

## Extension

New fingerprint modes, link sources and impact reasons are new enum variants (docs/Architecture.md, section 8). Do not add traits, registries or plugin mechanisms for them.
