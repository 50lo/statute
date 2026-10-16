---
# st-ta1f
title: 'rule new: explain required prefix format when prefix is invalid'
status: todo
type: bug
priority: low
created_at: 2026-10-16T09:32:12Z
updated_at: 2026-10-16T09:32:12Z
---

## What

`statute rule new AREA "TITLE"` prints the same error whether the rule file's `prefix:` is missing or present but malformed. When it is malformed, the message gives no hint of the required format, so the user cannot fix it without running `statute check`. A new user may not know that command exists yet.

Make `rule new` say what is wrong with a malformed prefix and what format is required, using the same wording as the `rule.prefix_invalid` finding from `statute check`.

## Observed behaviour

```
$ cat statute/rules/links.md
---
prefix: links
---
$ statute rule new links "Link target must parse cleanly"
statute: statute/rules/links.md needs a valid `prefix:` in its frontmatter, such as `prefix: BILL`
$ statute check
error[rule.prefix_invalid] prefix `links` must be uppercase letters and digits, starting with a letter
  --> statute/rules/links.md
1 error, 0 warnings
```

## Expected behaviour

- **Prefix present but invalid** (here `links`): the error names the bad value and the required format, e.g.
  `statute/rules/links.md: prefix `links` must be uppercase letters and digits, starting with a letter, such as `prefix: LINKS``
  When the uppercased value is itself a valid prefix (as `LINKS` is), suggesting it is a nice touch but not required.
- **Prefix missing** (no frontmatter, or frontmatter without `prefix:`): keep the current message, `<file> needs a valid `prefix:` in its frontmatter, such as `prefix: BILL``. Optionally state the format there too.
- Both cases still exit with code 2 and leave the rule file unchanged (that is the current behaviour; `tests/cli.rs` checks it).

## Reproduction

1. `statute init` in an empty git repo.
2. Create `statute/rules/links.md` containing only the frontmatter `---
prefix: links
---
`.
3. Run `statute rule new links "Link target must parse cleanly"`.

## Where

- `src/commands/rule_new.rs` (around line 35–44): looks up the rule file's prefix with `.and_then(|f| f.prefix.as_deref()).filter(|p| ids::is_prefix(p))`, which merges "missing" and "invalid" into one `None` and so one `bail!`. Split the two cases.
- `src/check/rules.rs` (`prefixes`, around line 18–22): the source of the `rule.prefix_invalid` text, "prefix `{prefix}` must be uppercase letters and digits, starting with a letter".
- `src/model/ids.rs`: `is_prefix` defines the format, `^[A-Z][A-Z0-9]*$`.
- Spec: `docs/Statute-spec.md`, the rule file frontmatter section (around line 56) and the `rule new` row of the command table (around line 294).
- Tests: `tests/cli.rs`, `rule_new_needs_a_rule_file_with_a_prefix` (around line 115). It covers the missing-prefix case only.

## Constraints

- Module layering (AGENTS.md, enforced by `tests/layering.rs`): `commands` may import `model` and `check`. To avoid two copies of the format wording drifting apart, a sensible option is a shared description in `model::ids` (e.g. a `PREFIX_FORMAT` const or a helper next to `is_prefix`) that both `check::rules::prefixes` and `commands::rule_new` use. Do not make `commands` call into another command.
- If the `rule.prefix_invalid` wording changes, update the snapshots `tests/snapshots/cli__check_text_output.snap` and `tests/snapshots/cli__check_reports_every_problem_in_the_broken_fixture.snap` on purpose. Keeping the wording unchanged is preferred.
- Before changing code, run `statute show` on the files you touch, and run the `change` skill before calling the task done (AGENTS.md, "Statute" section).

## Acceptance criteria

- [ ] `rule new` on a file whose `prefix:` is present but fails `is_prefix` prints an error that includes the offending value and the required format (uppercase letters and digits, starting with a letter).
- [ ] `rule new` on a file with no `prefix:` still prints a "needs a valid `prefix:`" message.
- [ ] Both cases exit with code 2 and leave the file unchanged, in both text and `--json` output.
- [ ] The format wording is defined once and shared with the `rule.prefix_invalid` finding, or the two messages match exactly.
- [ ] `tests/cli.rs` gains a case for the invalid-prefix path (e.g. `prefix: links`) asserting the new message; the existing missing-prefix assertion still passes.
- [ ] `cargo test` passes, including `tests/layering.rs`.
