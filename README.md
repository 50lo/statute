# Statute

Statute keeps a product's important rules written down, linked to code, and checked every time the code changes.

A rule says what must be true, why it matters, and where the code and tests make it true. When linked code changes, Statute flags the rule for review. If the code and a rule disagree, the disagreement stays visible until someone fixes the code or a person approves a decision to change the rule.

## Install

Install a [Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cargo install statute-cli --locked
```

The package is `statute-cli`; the installed command is `statute`. Git must also be available on your `PATH`.

Install the [Statute plugin for Claude Code or Codex](https://github.com/50lo/statute/blob/main/plugin/README.md). It gives your coding agent the workflows for capturing rules, reviewing changes, proposing decisions and recording your approval.

The CLI must be available on the agent's `PATH`.

## Get started

From an existing Git repository:

```sh
statute init
```

`init` creates `statute.yml`, the `statute/` directory, `statute.lock`, and adds instructions to `AGENTS.md`. Commit these files to your repository. Rules, terms, decisions and gaps are plain text files and easy to review.

Start with a few rules that are expensive to get wrong: money, privacy, permissions, or what data you keep. Each rule states one observable requirement, a case it protects, and a nearby case that shows where it stops.

## Capture your first rules

Ask your coding agent to use Statute's `adopt` skill. Give it a small area of the product and the documents that explain it:

> Use Statute's adopt skill for order cancellation. Read our order documentation and the cancellation code and tests. Propose the important rules and terms, link them to the implementation and tests, and show me what needs approval.

The agent drafts the rules and terms, checks their protected and boundary cases, links them to code and tests, and records proposed decisions. It identifies missing tests or behaviour that needs a manual check.

Review what it proposes. Are these the rules you intend? Do the examples cover the cases that matter? Ask the agent to revise anything that is wrong or unclear.

When you are ready, ask it to use the `ratify` skill to show the pending decisions for approval. Approve specific decision IDs; the agent records your approval through Statute. Until then, the new rules and terms remain *proposed*.

## Make changes with your agent

Continue asking your agent to build features and fix bugs as usual:

> Refactor order dispatch to use a queue.

The repository instructions tell the agent to read the linked rules and terms before editing. After editing, its `change` skill checks affected rules against the changed code and records the result:

- **The rule still holds:** the agent records its review and refreshes the links.
- **Your request needs the rule to change:** the agent names the conflict and proposes a decision.
- **The code broke the rule:** the agent records a gap and reports what needs fixing.

The agent includes rule impact in the pull request description. You review the product consequences alongside the code.

Statute detects changed links and records the review. The agent checks whether the behaviour still satisfies the rule.

## Change a product rule

When the intended behaviour changes, tell the agent what should happen:

> Customers can currently cancel until an order ships. We now need cancellation to stop when packing starts. Use Statute to propose that change and show me the affected rules, terms and cases.

The `decide` skill drafts the change and records why it is needed. The `ratify` skill presents it for your approval. Implementing the code and approving the rule are separate steps; the agent cannot approve its own proposal.

## CLI reference

Use `statute --help` and `statute COMMAND --help` for command details. The [specification](https://github.com/50lo/statute/blob/main/docs/Statute-spec.md) describes the file formats and workflow.

## More

- [Why Statute exists](https://github.com/50lo/statute/blob/main/docs/One-pager.md)
- [Behaviour and file formats](https://github.com/50lo/statute/blob/main/docs/Statute-spec.md)
- [Architecture](https://github.com/50lo/statute/blob/main/docs/Architecture.md)
- [Changelog](https://github.com/50lo/statute/blob/main/CHANGELOG.md)

Statute is under the [MIT license](https://github.com/50lo/statute/blob/main/LICENSE).
