# Changelog

## Unreleased

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
