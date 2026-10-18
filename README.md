# Statute

Statute keeps a product's important rules written down, linked to code, and checked every time the code changes.

A rule says what must be true, why it matters, and where the code and tests make it true. When linked code changes, Statute flags the rule for review. If the code and a rule disagree, the disagreement stays visible until someone fixes the code or a person approves a decision to change the rule.

## Install

Install a [Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cargo install statute-cli --locked
```

The package is `statute-cli`; the installed command is `statute`. Git must also be available on your `PATH`.

## Get started

From an existing Git repository:

```sh
statute init
statute status
```

`init` creates `statute.yml`, the `statute/` directory, `statute.lock`, and instructions in `AGENTS.md`. Commit these files with your code. Rules, terms, decisions and gaps are plain files in the repository.

Start with a few rules that are expensive to get wrong: money, privacy, permissions, or what data you keep. Each rule states one observable requirement, a case it protects, and a nearby case that shows where it stops.

Create an area file such as `statute/rules/orders.md` with a prefix:

```yaml
---
prefix: ORD
---
```

Then add a rule skeleton and fill in its fields:

```sh
statute rule new orders "A cancelled order cannot ship"
```

Once the rule is written, link it to the code and a test that checks it. For example, if those files exist:

```sh
statute link ORD-001 src/orders.rs
statute link ORD-001 tests/orders.rs --test
statute check --all
```

Links can point to whole files or Go, Python and Rust symbols, such as `src/orders.rs#ship`. Agreement and implementation are recorded separately. New rules start as proposed; a person approves the decisions that make them agreed.

## Work with rules

```sh
statute show src/orders.rs   # Read the rules linked to a file before changing it
statute check               # Find problems and changed links since the base
statute check --all          # Check every link
statute check --md           # Show rule impact for a pull request
statute status              # See agreement, implementation, gaps and pending decisions
```

After a change, review each affected rule. Record that it still holds with `statute review RULE ok`, propose a decision if it must change, or open a gap if the code broke it. Statute tracks the review; the reviewer checks the behaviour. Add `--json` for machine-readable reports.

Use `statute --help` and `statute COMMAND --help` for command details. The [specification](https://github.com/50lo/statute/blob/main/docs/Statute-spec.md) describes the file formats and workflow.

## Agent plugin

The [agent plugin](https://github.com/50lo/statute/tree/main/plugin) provides `adopt`, `change`, `decide` and `ratify` skills for Claude Code and Codex. Claude Code also gets a Stop hook that asks for a review when links are stale or missing. Install the plugin separately from the CLI; see its [setup instructions](https://github.com/50lo/statute/blob/main/plugin/README.md).

## More

- [What Statute does and why](https://github.com/50lo/statute/blob/main/docs/One-pager.md)
- [Behaviour and file formats](https://github.com/50lo/statute/blob/main/docs/Statute-spec.md)
- [Architecture](https://github.com/50lo/statute/blob/main/docs/Architecture.md)
- [Changelog](https://github.com/50lo/statute/blob/main/CHANGELOG.md)

Statute is under the [MIT license](https://github.com/50lo/statute/blob/main/LICENSE).
