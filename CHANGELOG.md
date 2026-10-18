# Changelog

## Unreleased

- Retire agreed terms with `decide --retires-terms`. A retired term keeps its section as a record of what earlier decisions meant, `show` labels it, and the name and wording checks skip it.

## 0.1.0 - 2026-10-19

First crates.io release, published as `statute-cli` with the `statute` command.

- Keep rules and terms in the repository, with separate agreement and implementation states.
- Link rules to whole files and Go, Python and Rust symbols; flag changed or missing targets for review.
- Propose decisions, record a person's approval, and track gaps between rules and code.
- Report rule impact as Markdown and command results as JSON.
- Provide agent skills for adoption, code changes, decisions and ratification, plus a Claude Code Stop hook.
