# Statute: v1 architecture

Status: proposal for review. It implements [the spec](Statute-spec.md) and adds no behaviour; where the spec leaves a choice open, the open choices are listed under "Decisions to confirm".

## 1. Principles

- **One crate, plain modules.** A single Cargo package with a library and a thin binary, so publishing is one package with one version. The compiler does not enforce layering inside a crate, so the import rules in [AGENTS.md](../AGENTS.md) do.
- **Extend by adding a variant, not by a plugin framework.** Fingerprint modes, link sources and impact triggers are closed Rust enums matched in one place. A new idea from "Ideas for v2" adds a variant and its match arms; nothing else moves.
- **Pure core, effects at the edges.** Parsing, fingerprinting, checks, derived state and impact are pure functions over in-memory data. Only three modules touch the outside: `repo` reads files, `txn` writes them, `git` runs `git`.
- **The JSON report is the contract.** Every command builds one serializable report; text, `--json` and `--md` are renderings of it. Skills and the Stop hook depend only on `--json` and stable finding codes.
- **Patch Markdown, rewrite YAML.** The CLI never re-renders a Markdown file. It patches byte ranges (a `yaml` block line, an appended section), so people's prose, comments and formatting survive.

## 2. Repository layout

```text
Cargo.toml
src/
  main.rs            parse args, call the command, render, set the exit code
  lib.rs
  cli/               clap argument definitions; renderers (text, json, md)
  repo.rs            find the root, read statute.yml, fixed paths; the only module that reads files
  model/             Rule, Term, Decision, Gap, Link, IDs, enums for agreed/built/status
  store/             parse each file kind from text and render edits back to text (below); no I/O
  git.rs             the only module that runs `git`
  fingerprint/       Mode enum; bytes, go, python (tree-sitter); takes bytes, no I/O
  check/             one function per check; Finding with a stable code
  state.rs           derived state: in force, incomplete, broken, stale, missing
  impact.rs          affected rules and terms against the base; verdicts for "Rule impact"
  commands/          one module per command: its logic, without argument parsing or rendering
  txn.rs             plan edits, validate in memory, write atomically; the only module that writes files
plugin/
  .claude-plugin/    Claude Code plugin manifest
  skills/            adopt, change, decide, ratify (SKILL.md each)
  hooks/             Stop hook: runs `statute check --json`
  agents-snippet.md  the AGENTS.md snippet; compiled into the binary for `init`
tests/
  fixtures/          small fixture repos (Go and Python files, Statute files)
  *.rs               end-to-end tests: build a temp git repo, run the binary, compare JSON
docs/
```

The plugin lives in the same repository so a CLI change and the skill that relies on it land in one PR. `init` embeds `plugin/agents-snippet.md` with `include_str!`, so the snippet has one source.

## 3. Data flow

### 3.1 Loading

`Repo::open` finds the root (the directory holding `statute.yml`, worktrees included) and reads the config. `Model::load` parses every Statute file; they are small, so there is no lazy loading. The model holds:

- rules and terms, each with its source span (file, heading range, `yaml` block range) for later patches;
- decisions and gaps as parsed entries;
- links from `statute.lock`, indexed by target file.

`BaseModel` is the same structure built from the base versions of the Statute files, read through one `git cat-file --batch` process. It is loaded only by commands that compare with the base (`check`, `status`).

### 3.2 `check`

```text
Repo::open
  -> git: base = merge-base(HEAD, origin/HEAD); ChangeSet = diff(base) + working tree + untracked
  -> Model::load, BaseModel::load
  -> check::structural(model)            IDs, schemas, references, completeness, terms
  -> check::history(model, base)         active decisions unchanged
  -> fingerprint links whose target file is in the ChangeSet (all links with --all)
       group by file; parse each file once; rayon across files
  -> state::derive                       stale, missing, incomplete, broken
  -> impact::compute(model, base, link states)   only for --md and status
  -> Report -> render -> exit code
```

`check --all` adds the repository walk (`ignore`) and the term-name search (`aho-corasick`).

### 3.3 Mutating commands

Every command that writes (`init`, `rule new`, `link`, `review`, `decide`, `ratify`, `gap open|close`) goes through a transaction:

1. Load the model.
2. Plan edits: byte-range patches for Markdown, whole-file writes for YAML and the lock.
3. Apply the edits in memory, re-parse the touched files, and run the structural checks on the result.
4. Refuse if the result has an error that the current state does not have. Otherwise write each file to a temp file and rename it into place.

This gives `ratify` its "refuses when the result would fail `check`" for free, and holds every other command to the same standard: a command never makes `check` worse.

## 4. Storage

| File | Read | Write |
| --- | --- | --- |
| `statute.yml` | `yaml_serde` into a typed struct | `init` only |
| `rules/<area>.md` | `pulldown-cmark` events with byte offsets: frontmatter, `#` headings, first paragraph, first `yaml` block, notes | append a section (`rule new`); patch the `agreed:` or `built:` line inside the block |
| `terms.md` | same, with `#` groups and `##` terms | patch or insert the `agreed:` line |
| `decisions.db` | `yaml_serde` into a list of entries | full rewrite |
| `gaps.yml` | same | full rewrite |
| `statute.lock` | `serde_json`, one line at a time | full rewrite: version line, then links sorted by source and target |

Metadata blocks parse into typed structs that reject unknown keys, so a typo such as `protect:` is an error rather than a silently ignored field. Field changes made by the CLI patch one line, which keeps comments such as `# proposed | agreed | retired`.

A full rewrite means the CLI parses the whole file, changes the data, and writes the whole file back in one fixed format: fixed key order, quoting, indentation and entry order, like `gofmt` or `rustfmt` output. The same data always produces the same bytes, so an entry the command did not touch never shows up in a diff. The cost is that comments and hand formatting in these files are lost on the next CLI write. The append-only check compares parsed entries, not text, so reformatting cannot trip it.

## 5. Git

`git.rs` is the only place that runs `git`. It provides:

- `base()`: merge-base of `HEAD` and `origin/HEAD`. With no remote, the base is `HEAD`, so only working-tree changes count.
- `changed_files(base)`: `git diff -z --name-status base` plus `git ls-files --others --exclude-standard -z`, with renames reported as delete plus add.
- `read_at(base, paths)`: one `git cat-file --batch` process for all base files.
- `user()`: `user.name` and `user.email`, for `ratified_by`.

The `ChangeSet` type exposes the changed paths. It is the place to add diff hunks later (see 8.2) without changing callers.

## 6. Fingerprints

```rust
enum Mode { BytesSha256V1, GoAstSha256V1, PythonAstSha256V1 }
```

- A target is `path` or `path#symbol`. The core splits at the first `#` and treats the symbol as opaque; the mode interprets it.
- `link` picks the mode from the file extension when a symbol is given, and `bytes-sha256-v1` otherwise. The chosen mode is stored per link and used for every later fingerprint of that link, so modes can be added or revised (as `-v2`) without touching existing links.
- Each tree-sitter mode turns one parsed file into a map from symbol to hash, in one pass, so all links into a file cost one parse. Missing covers: no such symbol, more than one match, or a parse with errors.
- One tree-sitter parser per rayon thread, created lazily per language.

Swift and TypeScript (milestone 7) are new variants with the same shape.

## 7. Checks, findings and reports

A finding has a stable code, a severity and the item it is about:

```json
{"code": "link.stale", "severity": "error", "rule": "ATTR-003", "target": "internal/x.go#F", "message": "..."}
```

- Severities: `error` (exit 1) and `warning` (reported, exit 0). Pending proposals are not findings; `status` lists them.
- Codes are grouped by concept: `rule.*`, `term.*`, `decision.*`, `link.*`, `gap.*`, `config.*`. Skills match on codes, never on message text.
- Each check is a plain function `fn(&Model, &mut Findings)` (or with `&BaseModel` for history). The list of checks is one array in `check/mod.rs`.
- Every `--json` report starts with `"schema": 1`.

`impact.rs` produces, for each affected rule and term, the reasons it is affected and its verdict, following the order in spec section 5. `check --md` and `status` render it.

## 8. Ideas for v2: where each one fits

None of them needs a v1 design change. These are the v1 choices that keep them additive.

### 8.1 Guide links

A guide is a doc linked to code so that a code change flags the doc for an update.

- v1 choice: `Link.source` is an enum, `LinkSource::Rule(RuleId)`. Fingerprinting, the lock index and stale or missing detection work on links without knowing the source kind; only rule-specific checks and `review` match on it.
- v2 work: add `LinkSource::Guide(path)`, an optional `"kind"` field in lock lines (absent means rule, so the lock version stays 1), a verdict command for guides, and a "Guides to update" list in reports.

### 8.2 Term matching

A change that touches a term's code names brings the rules using that term up for review.

- v1 choice: impact reasons are an enum (`LinkStale`, `LinkMissing`, `LinkAddedOrRemoved`, `DecisionChanged`, `GapChanged`). Terms already parse their `code` and `not` names, and `aho-corasick` is already a dependency for `check --all`.
- v2 work: add `ChangeSet::hunks(path)`, a `TermMatched` reason, and a decision on how a term-triggered review is recorded, since there is no stale link for `review` to refresh.

### 8.3 Markdown section fingerprints

- v1 choice: modes are a versioned enum, the symbol part of a target is opaque to the core, and `pulldown-cmark` is already a dependency.
- v2 work: add `MdSectionSha256V1` for `file.md#Heading`, hashing the section's normalised text up to the next heading of the same or higher level.

## 9. Dependencies

The crates in spec section 5.1, plus:

- `anyhow` for command errors (usage and file errors map to exit 2);
- dev only: `assert_cmd`, `tempfile`, `insta` for JSON snapshots of end-to-end runs.

## 10. Testing

- Unit tests next to each parser, check and fingerprint mode.
- End-to-end tests build a temporary git repo from a fixture, make a commit, apply an edit, run the binary and snapshot the `--json` report. Each spec scenario in the appendix becomes one test.
- From milestone 2, a `hyperfine` run of `check` on Footprints against the 100 ms budget.

## 11. Milestones to modules

| Milestone | Modules |
| --- | --- |
| 1. Model and checks | `repo`, `model`, `store` (all but lock), `check` (structural), `txn`, `commands` and `cli` for `init`, `rule new`, `show`, `check`, `status` |
| 2. Links | `git`, `fingerprint`, lock in `store`, `state` (stale, missing, incomplete), `commands::link`, `commands::review`, `check --all` |
| 3. Decisions | `commands::decide`, `commands::ratify`, `check::history`, `impact`, `--md` renderer |
| 4. Gaps | `commands::gap`, `state` (broken) |
| 5. Plugin | `plugin/` |

## 12. Decisions to confirm

1. **Full rewrite for `decisions.db`, `gaps.yml` and the lock; byte-range patches for Markdown.** YAML comments in `decisions.db` would not survive a CLI write. Recommended, because the CLI is the main writer of those files.
2. **Every mutating command refuses to introduce a new `check` error**, not only `ratify`. Recommended; it is the same code path.
3. **No remote means base = `HEAD`.** The spec assumes `origin/HEAD`; this covers a fresh local repo.
4. **Plugin in the same repository** under `plugin/`. Recommended.
