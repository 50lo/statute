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
  repo.rs            find the root, read statute.yml and the Statute files, load models; the only module that reads files
  model/             Rule, Term, Decision, Gap, Link, Mode, IDs, enums for agreed/built/status
  store/             parse each file kind from text and render edits back to text (below); no I/O
  git.rs             the only module that runs `git`
  fingerprint/       bytes, go, python, rust (tree-sitter); takes bytes, no I/O
  check/             one function per check; Finding with a stable code
  state.rs           derived state: in force, draft, incomplete, broken, stale, missing
  impact.rs          affected rules and terms against the base; verdicts for "Rule impact"
  commands/          one module per command: its logic, without argument parsing or rendering
  txn.rs             plan edits, re-parse them in memory, write files; the only module that writes files
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

`Repo::open` finds the root (the directory holding `statute.yml`, worktrees included) and reads the config. `repo` then reads every Statute file and passes the text to `store`, which parses it into a `Model`; the files are small, so there is no lazy loading. The model holds:

- rules and terms, each with its source span (file, heading range, `yaml` block range) for later patches;
- decisions and gaps as parsed entries;
- links from `statute.lock`, indexed by target file.

`BaseModel` is the same structure built from the base versions of the Statute files, which `repo` reads through one `git cat-file --batch` process. It is loaded only by commands that compare with the base (`check`, `status`, `ratify`).

### 3.2 `check`

```text
Repo::open
  -> git: base = merge-base(HEAD, origin/HEAD); ChangeSet = diff(base) + working tree + untracked
  -> repo: load Model and BaseModel
  -> check::structural(model)            IDs, schemas, references, completeness, terms
  -> check::history(model, base)         active decisions unchanged; agreed rules and terms
                                         changed only with a decision (7.1)
  -> fingerprint links whose target file is in the ChangeSet (all links with --all)
       group by file; parse each file once; rayon across files
  -> state::derive                       stale, missing, incomplete, broken
  -> impact::compute(model, base, link states)   only for --md and status (7.2)
  -> Report -> render -> exit code
```

`check --all` adds the repository walk (`ignore`) and the term-name search (`aho-corasick`). The search skips Statute's own files (`statute.yml`, `statute.lock`, everything under `statute/`) and Markdown files, because docs lag behind code and would hide a renamed name. A match counts only as a whole identifier: the bytes on either side are not letters, digits or `_`, so `Session` is not found inside `SessionID`.

### 3.3 Mutating commands

Every command that writes (`init`, `rule new`, `link`, `review`, `decide`, `ratify`, `gap open|close`) follows the same steps:

1. Load the model.
2. Check the command's own preconditions: the refusals the spec lists for it. For example, `link` refuses an unknown rule, an existing link and a target that matches more than one symbol; `review` refuses a rule with a missing link; `decide` refuses an unknown rule or term, one that is already in a proposed decision, and a retired rule in `changes` or `retires`.
3. Plan edits in `txn`: byte-range patches for Markdown, full rewrites for YAML and the lock.
4. Apply the edits in memory and re-parse the touched files. A file that no longer parses is a bug in the command, which fails without writing.
5. Write every touched file to a temp file next to it, then rename each into place.

Commands other than `ratify` do not run `check` on their result, because the spec's workflows pass through states that `check` reports: `link --remove` before relinking a moved target leaves the rule incomplete until the new link is added. `check` reports whatever remains at the end.

`ratify` runs the structural and history checks on its result, using the same check list as `check`, and refuses when any error is about the decisions it ratifies, their rules or their terms. Errors about other items and link freshness do not block it: approving one decision does not wait on unrelated work.

Each rename is atomic; a command that writes several files is not. All temp files are written before any rename, so most failures (disk full, permissions) happen before a file is replaced. If a rename fails, the command fails and names the files it already replaced; git shows the partial write.

## 4. Storage

| File | Read | Write |
| --- | --- | --- |
| `statute.yml` | `yaml_serde` into a typed struct | `init` only |
| `rules/<area>.md` | `pulldown-cmark` events with byte offsets: frontmatter, `#` headings, first paragraph, first `yaml` block, notes | append a section (`rule new`); patch the `agreed:` or `built:` line inside the block |
| `terms.md` | same, with `#` groups and `##` terms; a term's definition is its whole section except the heading and the `yaml` block | patch or insert the `agreed:` line |
| `decisions.db` | `yaml_serde` into a list of entries | full rewrite in one fixed format; YAML comments are not kept |
| `gaps.yml` | same | full rewrite |
| `statute.lock` | `serde_json`, one line at a time | full rewrite: version line, then links sorted by source and target |

Metadata blocks parse into typed structs that reject unknown keys, so a typo such as `protect:` is an error rather than a silently ignored field. Field changes made by the CLI patch one line, which keeps comments such as `# proposed | agreed | retired`.

A full rewrite means the CLI parses the whole file, changes the data, and writes the whole file back in one fixed format: fixed key order, quoting, indentation and entry order, like `gofmt` or `rustfmt` output. The same data always produces the same bytes, so once a file is in this format, an entry the command did not touch never shows up in a diff. The first CLI write after a hand edit or a merge resolution can reformat other entries, and comments and hand formatting in these files are lost then. The append-only check compares parsed entries, not text, so reformatting cannot trip it.

## 5. Git

`git.rs` is the only place that runs `git`. It provides:

- `base()`: merge-base of `HEAD` and `origin/HEAD`. With no remote, the base is `HEAD`, so only working-tree changes count.
- `changed_files(base)`: `git diff -z --name-status base` plus `git ls-files --others --exclude-standard -z`, with renames reported as delete plus add.
- `read_at(base, paths)`: one `git cat-file --batch` process for all base files.
- `user()`: `user.name` and `user.email`, for `ratified_by`.

The `ChangeSet` type exposes the changed paths. It is the place to add diff hunks later (see 8.2) without changing callers.

## 6. Fingerprints

```rust
enum Mode { BytesSha256V1, GoAstSha256V1, PythonAstSha256V1, RustAstSha256V1 }
```

- `Mode` lives in `model`, because every link stores its mode; `fingerprint` imports it from there.
- A target is `path` or `path#symbol`. The core splits at the first `#` and treats the symbol as opaque; the mode interprets it.
- `link` picks the mode from the file extension when a symbol is given, and `bytes-sha256-v1` otherwise. The chosen mode is stored per link and used for every later fingerprint of that link, so modes can be added or revised (as `-v2`) without touching existing links.
- Each tree-sitter mode turns one parsed file into a map from symbol to hash, in one pass, so all links into a file cost one parse. Missing covers: no such symbol, more than one match, or a parse with errors.
- One tree-sitter parser per rayon thread, created lazily per language.

Swift and TypeScript (milestone 7) are new variants with the same shape.

### 6.1 Hash input

The hash input decides how many false alarms and missed changes links produce, so each mode fixes it:

- **Tokens, not text.** A symbol's hash covers the text of its leaf tokens in source order, each followed by a separator byte. Whitespace between tokens is not part of the input, so formatting does not change the hash.
- **Comments.** Comment tokens are dropped. In Go, `//go:` directives directly above the declaration and the file's `//go:build` line are hashed with the symbol. In Python, a docstring (a string literal that is the first statement of a function or class body) is dropped like a comment. In Rust, doc comments are comments and are dropped; `#[doc = …]` attributes are kept.
- **No node kinds.** Tree-sitter node kind names are not part of the input, so a grammar update that renames node kinds does not change hashes.
- **Go symbols.** `Func`, `Type` and `Type.Method`. A method's receiver type is named without `*` and without type parameters, so `func (p *Publisher[T]) Decide` is `Publisher.Decide`. `#Type` covers the type declaration only, not its methods.
- **Python symbols.** `func`, `Class` and `Class.method`, with nested classes as dotted paths. Decorators are part of the symbol. `#Class` covers the whole class, methods included.
- **Rust symbols.** Functions, and types (struct, enum, union, trait, type alias) as `func` and `Type`; functions in `impl` and trait bodies as `Type.method`; items of inline modules as dotted paths, such as `tests.decides`. An impl's type is named without generics, `&` or a path, so `impl<T> fmt::Display for &billing::Publisher<T>` adds methods to `Publisher`; inherent and trait impls share that namespace, and a name defined in two impls is ambiguous. `#Type` covers the type's own item, not its impl blocks; `#Trait` covers the whole trait. Constants, statics, macros, modules themselves and items inside functions are not symbols. Outer attributes directly above an item are part of it. A trailing comma before a closing delimiter is formatting and is dropped, except in a one-element tuple and in macro and attribute tokens, where a macro can match on it.

Grammar crate versions are pinned. Each mode has golden fixtures: edits that must keep the hash (a comment, reformatting, a change to another declaration) and edits that must change it (a body or signature change, a Go directive). A change to the hash input that alters any golden hash is a new mode version, never an edit to an existing one.

## 7. Checks, findings and reports

A finding has a stable code, a severity and the item it is about:

```json
{"code": "link.stale", "severity": "error", "rule": "ATTR-003", "target": "internal/x.go#F", "message": "..."}
```

- Severities: `error` (exit 1) and `warning` (reported, exit 0). Pending proposals are not findings; `status` lists them.
- Codes are grouped by concept: `rule.*`, `term.*`, `decision.*`, `link.*`, `gap.*`, `config.*`. Skills match on codes, never on message text.
- Each check is a plain function `fn(&Model, &mut Findings)` (or with `&BaseModel` for history). The list of checks is one array in `check/mod.rs`.
- Every `--json` report starts with `"schema": 1`.

### 7.1 History checks

`check::history` compares the model with the base:

- Active decisions are unchanged: an entry that is active in the base has the same parsed fields and is not deleted.
- Agreed rules and terms change only with a decision. For each rule that is agreed in the base, it compares the statement (with whitespace collapsed) and the parsed `scope`, `terms`, `exceptions`, `protects`, `boundary` and `manual_check`; for each term that is agreed in the base, the definition (its whole section except the heading and the `yaml` block, with whitespace collapsed), `code` and `not`. Notes, `agreed`, `built` and formatting are not compared. A difference requires a decision that lists the item (`changes` for a rule, `terms` for a term) and is either proposed or became active since the base. Rules and terms that are proposed in the base are not compared: their text is a proposal, and restoring a rejected proposal must not trip the check.
- Retired rules never change. For each rule that is retired in the base, it compares the same fields as for an agreed rule and requires `agreed: retired`, and no decision exempts it. The finding is `rule.retired_changed`, not `rule.changed_without_decision`, because `decide` cannot carry the change.

### 7.2 Impact

`impact.rs` produces, for each affected rule and term, the reasons it is affected and its verdict, following the order in spec section 5. `check --md` and `status` render it. It takes three kinds of link input:

- current link health: stale or missing now (`LinkStale`, `LinkMissing`);
- links added or removed since the base, matched by source and target (`LinkAddedOrRemoved`);
- links whose recorded `sig` differs from the base lock, which `review` refreshed (`LinkRefreshed`).

A rule with only `LinkRefreshed` reasons falls through the verdict order to "holds". This is how a rule reviewed with `review RULE ok` stays in the report without a separate review record.

## 8. Ideas for v2: where each one fits

None of them needs a v1 design change. These are the v1 choices that keep them additive.

### 8.1 Guide links

A guide is a doc linked to code so that a code change flags the doc for an update.

- v1 choice: `Link.source` is an enum, `LinkSource::Rule(RuleId)`. Fingerprinting, the lock index and stale or missing detection work on links without knowing the source kind; only rule-specific checks and `review` match on it.
- v2 work: add `LinkSource::Guide(path)`, an optional `"kind"` field in lock lines (absent means rule, so the lock version stays 1), a verdict command for guides, and a "Guides to update" list in reports.

### 8.2 Term matching

A change that touches a term's code names brings the rules using that term up for review.

- v1 choice: impact reasons are an enum (`LinkStale`, `LinkMissing`, `LinkAddedOrRemoved`, `LinkRefreshed`, `DecisionChanged`, `GapChanged`). Terms already parse their `code` and `not` names, and `aho-corasick` is already a dependency for `check --all`.
- v2 work: add `ChangeSet::hunks(path)`, a `TermMatched` reason, and a decision on how a term-triggered review is recorded, since there is no stale link for `review` to refresh.

### 8.3 Markdown section fingerprints

- v1 choice: modes are a versioned enum, the symbol part of a target is opaque to the core, and `pulldown-cmark` is already a dependency.
- v2 work: add `MdSectionSha256V1` for `file.md#Heading`, hashing the section's normalised text up to the next heading of the same or higher level.

## 9. Dependencies

The crates in spec section 5.1, plus:

- `anyhow` for command errors (usage and file errors map to exit 2);
- dev only: `tempfile`, and `insta` for snapshots of end-to-end runs. Tests run the built binary through `CARGO_BIN_EXE_statute`.

## 10. Testing

- Unit tests next to each parser, check and fingerprint mode.
- Golden fixtures for each fingerprint mode (6.1).
- End-to-end tests build a temporary git repo from a fixture, make a commit, apply an edit, run the binary and snapshot the `--json` report. Each spec scenario in the appendix becomes one test. Beyond the scenarios:
  - intermediate states: `rule new` and `link` before `decide`; `link --remove` then `link` for a moved target;
  - a rule reviewed with `review RULE ok` appears in `check --md` as holding;
  - `ratify` after the proposal has already reached the default branch;
  - an agreed rule or term edited without a decision is an error.
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

1. **No remote means base = `HEAD`.** The spec assumes `origin/HEAD`; this covers a fresh local repo.
2. **Plugin in the same repository** under `plugin/`. Recommended.
