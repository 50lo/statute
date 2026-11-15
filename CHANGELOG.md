# Changelog

## Unreleased

- A rule can have test links and a manual check together. Test links never replace or complete a manual check: it stays required. Decision D261115-ywulh records this and awaits approval.
- `status` shows each rule's manual check. `status --json` adds `manual_check` to each rule.
- `review` prints the rule's manual check as still required. `review --json` adds `manual_check`.
- The "Rule impact" section (`check --md`) lists the manual checks of affected rules under "Manual checks". Its JSON adds `manual_check` to each rule.
- The `rule.incomplete` message now reads "lacks a code link, or has neither a test link nor a `manual_check`".
- The `adopt`, `decide` and `change` skills keep manual checks beside test links and tell the user a person must still do them.

## 0.3.2 - 2026-11-08

- Fix release builds for Apple Silicon macOS.
- When a Swift file does not parse cleanly, `link`, `check` and `review` say near which line and column the parser first fails. `link --json` adds `parse_error_near`.

## 0.3.1 - 2026-11-08

- Fix plugins versioning.

## 0.3.0 - 2026-11-08

- Add Swift symbol links.
- Add a Pi package that reuses Statute's four skills and reminds the agent to run `change` when links are stale or missing.
- Add a Cursor plugin with the same four skills and a native stop hook, exposed via `.cursor-plugin/marketplace.json`.

## 0.2.0 - 2026-11-05

- `statute ratify` is now `statute approve`, the `ratify` skill is now `approve`, and the decision field `ratified_by` is now `approved_by`. Rename the field in an existing `decisions.db`.
- Retire agreed terms with `decide --retires-terms`. A retired term keeps its section as a record of what earlier decisions meant, `show` labels it, and the name and wording checks skip it.

## 0.1.0 - 2026-10-19

First crates.io release, published as `statute-cli` with the `statute` command.

- Keep rules and terms in the repository, with separate agreement and implementation states.
- Link rules to whole files and Go, Python and Rust symbols; flag changed or missing targets for review.
- Propose decisions, record a person's approval, and track gaps between rules and code.
- Report rule impact as Markdown and command results as JSON.
- Provide agent skills for adoption, code changes, decisions and ratification, plus a Claude Code Stop hook.
