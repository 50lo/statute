---
# st-2o30
title: 'rule new: add --prefix to create the rule file'
status: completed
type: feature
priority: low
created_at: 2026-10-16T10:03:51Z
updated_at: 2026-11-06T10:08:57Z
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

`--prefix` only creates a new rule file. It never edits an existing one.

| State of `statute/rules/<AREA>.md` | Result |
| --- | --- |
| Does not exist | Create it with frontmatter containing `prefix: <PREFIX>`, then append the rule skeleton |
| Exists, in any form | Error, file unchanged. This covers a file with a prefix (including the same prefix), with frontmatter but no prefix, with no frontmatter, and an empty file |

An existing file with no prefix is fixed by hand: add `prefix:` to its frontmatter, then run `rule new` without `--prefix`.

Also reject, with the file unchanged:
- a `<PREFIX>` that fails the prefix format (`model::ids::is_prefix`; spec, rule file section);
- a `<PREFIX>` already declared by another rule file (spec: "No two rule files share a prefix").

Without `--prefix`, behaviour stays as it is: the file must already exist and declare a valid prefix, and no duplicate check is added. The "does not exist" error should mention `--prefix` as the fix.

## Where

- `src/commands/rule_new.rs`: `run(area, title)` gains the prefix argument; the file is written through `Txn` as now.
- `src/store/rules.rs`: frontmatter parsing lives here; building the new file's frontmatter belongs here too (pure text in, text out).
- `src/cli/mod.rs`: the clap definition of `rule new`. `src/cli/render.rs` lines ~22 and ~66 print "create statute/rules/<area>.md with `prefix: <PREFIX>`" hints that could point to `--prefix`.
- `docs/Statute-spec.md`: the `rule new` row of the command table and the "Starting Statute in a new repo" scenario need updating; the spec is the source of truth.
- Layering and Statute workflow rules: AGENTS.md.

## Related

- st-ta1f (invalid-prefix error message in `rule new`) touches the same code in `rule_new.rs`; do them together or rebase one on the other.

## Acceptance criteria

- [x] Each row of the behaviour table and both reject cases are covered by tests in `tests/cli.rs`, including that rejected runs exit with code 2 and leave the file unchanged, in text and `--json` output.
- [x] Created files pass `statute check` with no new findings.
- [x] `statute rule new --help` documents `--prefix`.
- [x] The spec describes the option.
- [x] `cargo test` passes, including `tests/layering.rs`.

## Summary of Changes

Implemented in [PR #32](https://github.com/50lo/statute/pull/32), merged as `492476c`. `rule new --prefix` creates a new rule file and its first proposed rule, refuses every existing-file form and invalid or duplicate prefixes without writes, and leaves the existing no-option behavior intact. Help, onboarding hints and the specification describe the option.

Verified on 6 November 2026: `cargo test --locked` passes, including creation, refusal, clean-check, help and layering coverage.
