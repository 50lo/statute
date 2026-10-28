# Statute

Statute is a Rust CLI and agent plugin that keeps a product's rules written down, linked to code, and checked on every change.

- What and why: [docs/One-pager.md](docs/One-pager.md)
- Behaviour, file formats, commands: [docs/Statute-spec.md](docs/Statute-spec.md) (source of truth)
- Code structure: [docs/Architecture.md](docs/Architecture.md)
- Statute's terms explained for people: [docs/Glossary.md](docs/Glossary.md)

## Issue tracker

This project uses Beans (`beans`) as its issue tracker. Keep gap details in a Bean
tagged `statute` and `gap`, then record its ID with `statute gap open RULE --ref st-xxxx`.
After verifying that the rule holds, close the gap with `statute gap close st-xxxx`
and complete the Bean.

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

Implementation is still in progress. This branch supports `init`, `rule new`, `show` for rule IDs, terms, decision IDs, files and symbols, `check [--all]`, whole-file and Go/Python/Rust symbol `link`, `review`, `check --md`, `decide`, `approve`, `gap open`, `gap close`, and `status`. The agent plugin, with the `adopt`, `change`, `decide` and `approve` skills, is in `plugin/`. Check `statute --help` for the installed build.

Use `statute show` to inspect linked rules and terms, `statute decide` to record proposed decisions, and `statute approve` to record a person's explicit approval of specific decisions. Report workflow steps that the current build cannot perform. Agreement and build states are set by the CLI.

## Decision-making principles

Apply these when a design choice trades one risk against another.

1. A missed behaviour change is worse than a false alarm. When a change might alter behaviour, report it (for example, make the link stale) rather than treat it as formatting.

## Glossary

[docs/Glossary.md](docs/Glossary.md) explains Statute's terms to people, in sections that each explain several terms. [statute/terms.md](statute/terms.md) holds the exact definitions, which people approve. Agreed terms are authoritative; the glossary must agree with them and adds no condition, exception, state or distinction that a term's definition lacks. A term's definition is everything in its section except the heading and the `yaml` block. Each glossary section starts with "Main terms" and "Also mentions" lines that link to the terms it covers. Links run one way, from the glossary to `statute/terms.md`.

1. Use `docs/Glossary.md` to explain Statute to people. Use the terms that `statute show` returns to do the work.
2. A PR that changes `docs/Glossary.md`, or the text, `code` or `not` of a term in `statute/terms.md`, gets a consistency review, and the record below in its description. The author lists every term that each changed glossary section explains (its "Main terms" and "Also mentions" lines and other uses), and every glossary section that links to or uses each changed term. The reviewer compares each pair and either confirms that they agree or updates them together. [docs/Terms-refactoring-proposal.md](docs/Terms-refactoring-proposal.md), "Manual consistency review in both directions", has the full method.
3. When asking a person to approve a decision that lists a term, show the term's definition together with every glossary section that links to it.
4. A glossary edit that changes a meaning needs a proposed decision (`statute decide --terms`) that changes the term's definition, and the glossary says that the change awaits approval. A consistency review approves nothing.
5. If the glossary or a compared term changes after the commit named in the record, re-review the affected rows and update the commit before merge.
6. When a review finds a counterpart that an earlier review missed, or finds drift between the glossary and a term on the default branch, add it to Bean st-uzzw (automated consistency review) as evidence.

Record format, one row per glossary section that changed or that links to a changed term:

```markdown
## Glossary consistency

Reviewed at: <commit SHA>

| Glossary section | Terms compared | Conclusion |
| --- | --- | --- |
| Links and fingerprints | Link, Code link, Test link, Fingerprint, Stale link, Missing link | Consistent |

Missed by an earlier review: none
```

The conclusion is one of: consistent; updated together (name the edits); decision D… proposed.

<!-- statute:begin -->
## Statute

- Before changing code, run `statute show` on the files you will touch and read the rules and terms it returns.
- Change a rule or an agreed term only together with a proposed decision that lists it (`statute decide`), never to match the code. Never edit `agreed` or `built`; the CLI sets them.
- When a user instruction conflicts with an agreed rule, name the rule and propose a decision before changing the code.
- After changing code, run the `change` skill before declaring the task done.
- Anything you create is `proposed`. Run `statute approve` only after a human in this session explicitly approves those specific decisions.
- Before asking a human to approve a rule, check it: what valid case would it reject? What invalid case would it allow? Does its meaning change with actor or permission, missing data, lifecycle stage, time, retries, concurrency or partial failure? Add to the rule only the answers that change it.
- Gaps go to the issue tracker named in this file; if none is named, use `statute gap open --note`.
- When instructions conflict: explicit user instruction > current task > project principles > agreed rules, terms and approved decisions > repo conventions > proposed decisions > agent judgment.
<!-- statute:end -->
