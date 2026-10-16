---
# st-2o30
title: 'rule new: add --prefix to create or initialise the rule file'
status: todo
type: feature
priority: low
created_at: 2026-10-16T10:03:51Z
updated_at: 2026-10-16T10:03:51Z
---

## What

Add a `--prefix <PREFIX>` option to `statute rule new AREA "TITLE"` so a user can start a new rule area in one command instead of hand-writing the rule file's frontmatter first.

## Why

Today `rule new` on a missing file stops with:

```
$ statute rule new links "Target must parse cleanly"
statute: statute/rules/links.md does not exist; create it with `prefix: <PREFIX>` in its frontmatter first
```

The user has to open an editor, learn the frontmatter format (docs/Statute-spec.md, rule file section) and rerun the command.

## Behaviour with `--prefix`

| State of `statute/rules/<AREA>.md` | Result |
| --- | --- |
| Does not exist | Create it with frontmatter containing `prefix: <PREFIX>`, then append the rule skeleton |
| Exists, no frontmatter | Add frontmatter with `prefix: <PREFIX>` at the top, keep the existing content, append the skeleton |
| Exists, frontmatter without `prefix:` | Add `prefix: <PREFIX>` to it, keep other keys, append the skeleton |
| Exists, frontmatter already has `prefix:` | Error, file unchanged |

Also reject, with the file unchanged:
- a `<PREFIX>` that fails the prefix format (`model::ids::is_prefix`; spec, rule file section);
- a `<PREFIX>` already declared by another rule file (spec: "No two rule files share a prefix").

Without `--prefix`, behaviour stays as it is. The "does not exist" error should mention `--prefix` as the fix.

Open question for the implementer to settle with the user: should `--prefix X` on a file that already declares `prefix: X` be an error (as specified above) or a no-op?

## Where

- `src/commands/rule_new.rs`: `run(area, title)` gains the prefix argument; the file is written through `Txn` as now.
- `src/store/rules.rs`: frontmatter parsing lives here; adding/inserting the `prefix:` line belongs here too (pure text in, text out).
- `src/cli/mod.rs`: the clap definition of `rule new`. `src/cli/render.rs` lines ~22 and ~66 print "create statute/rules/<area>.md with `prefix: <PREFIX>`" hints that could point to `--prefix`.
- `docs/Statute-spec.md`: the `rule new` row of the command table and the "Starting Statute in a new repo" scenario need updating; the spec is the source of truth.
- Layering and Statute workflow rules: AGENTS.md.

## Related

- st-ta1f (invalid-prefix error message in `rule new`) touches the same code in `rule_new.rs`; do them together or rebase one on the other.

## Acceptance criteria

- [ ] Each row of the behaviour table and both reject cases are covered by tests in `tests/cli.rs`, including that rejected runs exit with code 2 and leave the file unchanged, in text and `--json` output.
- [ ] Created or edited files pass `statute check` with no new findings.
- [ ] `statute rule new --help` documents `--prefix`.
- [ ] The spec describes the option.
- [ ] `cargo test` passes, including `tests/layering.rs`.
