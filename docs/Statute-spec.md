# Statute: CLI and agent plugin spec

Status: draft for implementation. Concepts, vocabulary and the change loop are defined in the one-pager, [Statute: keeping product rules true](One-pager.md). This spec turns them into file formats, commands and agent behaviour, and does not repeat the rationale.

## 1. Scope

Goals:

- One tool that owns the four concepts (Terms, Rules, Decisions, Links), the gap state of rules, and their checks.
- Deterministic checks that run locally without any agent.
- An agent plugin that carries the judgment steps: adopting, checking, deciding, getting human approval.
- Usable by PMs and designers who contribute through coding agents: approval happens in the agent session, and output meant for the PR is written for a non-engineer reader.
- One native format for every repo, set up with `statute init`; existing material is converted into it once.

Non-goals:

- Proving rules correct, or executing rules as code.
- Running tests. Tests are enforced by the project's CI; Statute tracks changes to linked tests.
- CI integration.
- A full specification or requirements-management system.
- A UI. Output is terminal text.

## 2. Architecture

| Layer | Form | Responsibility |
| --- | --- | --- |
| Core | `statute` CLI, single binary, no runtime dependencies | File formats, checks, link fingerprints, status |
| Agent | Skills, an AGENTS.md snippet, and a Stop hook in Claude Code | Judgment: reading rules and terms before coding, checking affected rules, proposing, asking a human to approve |
| Repo | `statute.yml` + content files + `statute.lock` | The repo's own terms, rules, decisions and links |

Rule of separation: anything that can be decided mechanically lives in the CLI. Anything that needs judgment lives in the plugin and always ends with a CLI command that records the result.

## 3. Repository layout and config

Default layout created by `statute init`:

```text
statute.yml           config (below)
statute.lock          links and their fingerprints (generated, committed)
statute/
  terms.md            Terms
  rules/<area>.md     Rules, one file per area
  decisions.db        Decisions (append-only) in YAML format, with a `.db` extension to discourage agents from reading the whole file into their context window
  gaps.yml            Gaps index: every gap and its status; the full gap when AGENTS.md names no issue tracker, otherwise a reference to the issue
```

Every repo uses this layout, so there are no paths to configure. `statute.yml` holds settings only:

```yaml
version: 1
summary_max: 600      # decision summary cap, in characters
```

Merge conflicts in Statute files are resolved like any other conflict, by a person or their agent. `check` then validates the result: files parse, keys and IDs are unique, and active history is unchanged.

Each rule file starts with YAML frontmatter that declares the ID prefix for its rules. The area is the file name: `rules/attribution.md` is the area `attribution`.

```markdown
---
prefix: ATTR
---
```

All paths are repo-relative. The lock lives at the repository root (worktrees included).

## 4. Data model

### 4.1 Rules

A rule is a `#` heading section inside a rule file; there is no other rule format. Every `#` heading in a rule file is a rule. The heading carries the ID and a short title, and the rule runs to the next heading, so rule prose has no subheadings. The first paragraph is the statement; a fenced `yaml` block carries metadata. Everything after the block is free prose (notes).

The metadata block is the first `yaml` fenced block after the heading. Statute parses it as YAML and checks it against the rule schema. Readers can call it the rule block; the fence stays plain `yaml` so editors render and highlight it correctly. Terms work the same way (4.3).

````markdown
# ATTR-003 Context alone creates no contribution

HEAD, branch, time, path overlap and collector score never create a Session–commit contribution link.

```yaml
agreed: agreed            # proposed | agreed | retired
built: built              # planned | built   (broken is derived, see 4.6)
scope: Session–commit attribution
terms: [Session, Commit]
exceptions: []            # optional: qualifications that change how the rule is read
protects:
  - A Session active on the same branch an hour before abc123, with no other evidence, does not contribute to abc123.
boundary:
  - A Session that ran `git commit` for abc123 may still contribute to abc123; this rule does not block evidence-based links.
```
````

Each rule states one observable obligation: what must or must not happen, to what, and under which conditions, so that a violation can be told apart from permitted behaviour. The statement names an implementation constraint only when that constraint matters in itself. The rule fields:

- Statement: the obligation, in one or two sentences.
- `scope` and `exceptions`: only qualifications that could change someone's reading. Keep "this alone must not cause rejection" apart from "this must always be accepted".
- `protects`: one case the rule protects. `boundary`: one nearby case it permits or excludes. Add more only when they expose a different boundary.
- Rationale lives in the decisions that establish and change the rule, and in its code and tests, and is not repeated in the rule.

The protected and boundary cases map onto the required tests: one test that fails when the protected case breaks, and one that keeps the boundary case permitted. `statute rule new` writes this skeleton.

A rule whose behaviour needs a person to observe or judge it carries a manual check in place of a test link. For example, "The cancel link is as easy to find as the upgrade link":

```yaml
manual_check: "A designer compares the cancel and upgrade links in each release review."
```

Use `manual_check` only for such rules. A metric's computation, for example, is tested against its agreed definition; choosing the definition is the decision.

How fields change:

- `agreed` is set by the CLI: `rule new` writes `proposed`, `decide` sets `proposed` on the rules a decision changes, and `ratify` sets `agreed` or `retired` (4.2).
- `built` is set by `link` and `ratify` when the rule becomes complete: at least one code link, and at least one test link or a `manual_check`. Nobody edits `built` by hand.
- The statement, `scope`, `terms`, `exceptions`, `protects`, `boundary` and `manual_check` change only together with a proposed decision that lists the rule in `changes`. `decide` sets the rule back to `agreed: proposed` until that decision is ratified. Pending items never block a merge, so the default branch can carry the proposed text; `status` and the PR "Rule impact" section show the rule as awaiting approval.
- Retirement is final. A retired rule keeps its section, its statement and the fields above, and `agreed: retired`, whatever decision lists it. `decide` refuses to list it in `changes` or `retires`; new behaviour gets a new rule ID.

Checks (`statute check`):

- ID matches `^[A-Z][A-Z0-9]*-\d{3}$`, starts with the file's `prefix`, is unique across all rule files, and is never reused after retirement. No two rule files share a prefix.
- `agreed` and `built` hold allowed values.
- Every entry in `terms` resolves to a term (4.3).
- `agreed: agreed` requires at least one `protects` and one `boundary` case. They are the safeguard against code, docs and tests that agree with each other and are all wrong.
- A rule that is not retired and has `built: built` requires at least one `code` link and at least one `test` link, or a non-empty `manual_check` in place of the test link.
- Every agreed or retired rule is established by exactly one decision. A proposed rule that no decision establishes yet is a draft: it is a pending item, not a finding, and `status` lists it.
- The statement, `scope`, `terms`, `exceptions`, `protects`, `boundary` and `manual_check` of a rule that is agreed in the base change only together with a decision that lists the rule in `changes` and is proposed or became active since the base. Notes and formatting are not compared.
- A retired rule keeps its section and has no links.
- A rule that is retired in the base keeps its section, its statement and its `scope`, `terms`, `exceptions`, `protects`, `boundary` and `manual_check`, and stays `agreed: retired`, even when a decision lists it. Notes, `built` and formatting are not compared.

Rule IDs use counters because they are readable. Two branches can create the same ID. `rule new` appends to the end of the rule file, so their rule files conflict on merge, and whoever resolves the conflict renumbers the rule that is not yet on the default branch, together with its links, gaps and decisions. `check` reports any duplicate that slips through.

### 4.2 Decisions

Decisions are append-only events in `statute/decisions.db` (YAML).

```yaml
- id: D261014-k7m2q
  timestamp: "2026-10-14T10:26:12Z"
  actor: agent                         # user | agent: who initiated the decision; informational only
  status: proposed                     # proposed | active
  ratified_by: null                    # set by `statute ratify`; required when status is active
  summary: "One to three sentences."
  rationale: "Why this alternative, what was rejected."
  establishes: [ATTR-001, ATTR-002]    # new rules
  changes: [ATTR-003]                  # rules whose text this decision changes
  retires: []
  terms: [Approver]                    # terms this decision introduces or redefines
```

`ratified_by` records a person and a time: `{name: "Anton Solo", email: "…", at: "2026-10-15T09:00:00Z"}`. Any human can approve; there are no rule owners.

`statute ratify D…` makes each decision active:

- It sets `status: active` and `ratified_by`, taking the person from git `user.name` and `user.email`.
- It sets `agreed: agreed` on the rules the decision establishes or changes and on the terms it lists, and `agreed: retired` on the rules it retires. It removes the links of retired rules.
- It sets `built: built` on every rule it touches that is complete (4.1).
- It refuses when `check` would report an error about the ratified decisions, their rules or their terms after ratification: for example, a rule becoming agreed without `protects` and `boundary` cases, or a rule using a term that is neither agreed nor approved in the same decision. Errors about other items and stale or missing links do not block it.

Checks:

- IDs are unique and have the form `DYYMMDD-xxxxx`: the marker `D`, the UTC date, and five random lowercase letters or digits, for example `D261015-k7m2q`. The random part lets agents in parallel worktrees add decisions without conflicting IDs.
- Every rule ID in `establishes`, `changes`, `retires` exists, and every entry in `terms` resolves to a term.
- A rule appears in the `establishes` of at most one decision (exactly one once it is agreed or retired, 4.1).
- A rule or term appears in at most one proposed decision. `decide` refuses a second one, and `check` reports an overlap that arrives through a merge.
- `status: active` requires `ratified_by`.
- History is immutable from `active` on. `check` compares `decisions.db` with the merge-base and fails when an active entry was edited or deleted. Proposed entries may be edited or deleted; a rejected proposal is deleted, and git keeps its history. A wrong decision is corrected by a new decision that changes or retires its rules.
- Summary length is capped (`summary_max` in `statute.yml`, default 600 characters). A long summary usually means several rules hiding in one decision.

### 4.3 Terms

Terms live in `statute/terms.md`. `#` headings group terms for readability and carry no meaning. Each `##` heading is a term and runs to the next heading. A fenced `yaml` block carries metadata. Everything else in the section after the heading is the definition: paragraphs, lists, tables and any other code block, before or after the `yaml` block. Terms have no notes.

````markdown
# Sessions and commits

## Session

One conversation between a developer and a coding agent.

```yaml
agreed: agreed                   # proposed | agreed
code: [Session, SessionID, sessioncommit]
not:
  - {name: sessions, means: the sign-in sessions table}
aliases: [agent session]
avoid: [chat]
```
````

Term names are unique across the repo. A word that means different things in two areas becomes two terms with distinct names, such as "Billing active customer" and "Support active customer".

- `code`: identifiers, types, packages and tables that implement the term.
- `not`: code names that look like the term but mean something else, with what they mean.
- `aliases`: other words that mean the same thing. `avoid`: words that should not be used for it.

Terms are approved like rules, because a term's wording steers how agents name and structure code, and a wrong term can cost a feature rewrite. A decision lists the terms it introduces or redefines in `terms`. `decide` sets `agreed: proposed` on them, adding the field to a new term, and `ratify` makes them agreed together with the decision's rules. An agreed term's definition, `code` or `not` changes only together with a proposed decision that lists it, so the term stays `proposed` until that decision is ratified.

Checks:

- Term headings are unique.
- The definition, `code` and `not` of a term that is agreed in the base change only together with a decision that lists the term in `terms` and is proposed or became active since the base. The definition is compared with whitespace collapsed; `aliases`, `avoid`, `agreed` and comments in the `yaml` block are not compared.
- `avoid` words in rule statements produce a warning.
- `check --all` warns when a `code` or `not` name no longer appears as a whole identifier in any file outside Statute's own files (`statute.yml`, `statute.lock`, `statute/`) and Markdown files.

Terms are vocabulary for people and agents. `check` does not match them against diffs.

### 4.4 Links

A link connects a rule to a code or test target, with a fingerprint. `statute.lock` is JSON Lines: the first line holds the format version, then one link per line, sorted by source and target, so merge conflicts stay local and readable.

```json
{"version": 1}
{"source": "ATTR-003", "target": "internal/sessioncommit/publisher.go#Publisher.Decide", "role": "code", "mode": "go-ast-sha256-v1", "sig": "…"}
{"source": "ATTR-003", "target": "internal/sessioncommit/publisher_test.go#TestDecideIgnoresContext", "role": "test", "mode": "go-ast-sha256-v1", "sig": "…"}
```

- `source`: the rule ID.
- `role`: `code` or `test`. Both are first class and do different jobs. A code link shows where the rule is made true; a change there brings the rule up for review. A test link shows the rule is true; the project's CI runs the test, and a change to the test brings the rule up for review, because weakening a test is a quiet way to break a rule.
- Test links target a specific test (`file_test.go#TestName`), so a change to that test is detected precisely.
- `link` records the current fingerprint and refuses to replace an existing link. A stale link is refreshed only through `review`.
- A link is missing when its target is not found (renamed, moved or deleted), matches more than one symbol, or sits in a file that does not parse cleanly. A partial parse never counts as unchanged. `link` refuses a target that matches more than one symbol, and `review` refuses a rule with a missing link.
- A parse error is fixed in the file. For a target that moved, the agent checks the rule as for a stale link, then removes the link with `link --remove` and links the new location. Relinking is the agent's statement that the rule still holds, like `review RULE ok`, and the PR "Rule impact" section lists the rule as relinked.

Fingerprint modes:

| Mode | Target | Notes |
| --- | --- | --- |
| `bytes-sha256-v1` | any file | Fallback for every language |
| `go-ast-sha256-v1` | `file.go#Func`, `#Type`, `#Type.Method` | Ignores comments and formatting, except `//go:` directives, including the file's `//go:build` line |
| `python-ast-sha256-v1` | `file.py#func`, `#Class`, `#Class.method` | Ignores comments, docstrings and formatting. Ported from nodrift |
| `rust-ast-sha256-v1` | `file.rs#func`, `#Type`, `#Type.method`, `#module.func` | Ignores comments, doc comments, trailing commas and formatting; attributes count |

The symbol modes are built on tree-sitter (5.1). Each mode's hash input is fixed by golden fixtures; a change that alters a fingerprint is a new mode version. Later modes (Swift, TypeScript) use tree-sitter behind the same interface. A symbol fingerprint does not cover its dependencies; link those separately when they matter, and rely on test links for new code.

Targets are files or symbols, never directories. `check` re-fingerprints only links whose target file changed since the merge-base with the default branch or in the working tree, so its cost grows with the size of the change, not the size of the repo. `check --all` re-fingerprints every link, which catches links made stale by changes that reached the default branch without Statute.

Reviews. A stale link brings its rule up for review, and the verdict is given once per rule: `statute review RULE ok|changed D…|gap REF`. `review` checks the verdict against the files, then refreshes every stale link of that rule. The lock stores no review records, because each verdict is recorded elsewhere:

- `ok`: the refreshed fingerprint is the record, and git history shows when it changed.
- `changed D…`: the decision must list the rule in `changes` or `retires`. The decision is the record.
- `gap REF`: the rule must have an open gap with that reference. The gap is the record.

### 4.5 Gaps

A gap is a known mismatch between a rule and the code. It is a state of the rule, backed by an issue.

- Statute keeps every gap in the gaps file (`statute/gaps.yml`), because `broken` (4.6) is derived from it and the CLI cannot query an issue tracker. Without a tracker, the entry is the gap. With the tracker named in AGENTS.md, the issue is the gap and the entry is only an index: ID, rule, status.
- With a tracker, the agent opens the issue with its usual tools and records the reference: `statute gap open RULE --ref <url-or-id>`. When using a tracker, always add `statute` and `gap` tags to the issue.
- Without a tracker, `statute gap open RULE --note …` writes an entry to the gaps file:

```yaml
- id: G261015-x4p9t
  rule: ATTR-003
  status: open          # open | closed
  opened_at: "2026-10-15T01:20:00Z"
  note: "New filter in collector drops Sessions without a Repository."
```

Gap IDs in the gaps file follow the decision format with the marker `G`, for example `G261015-x4p9t`. Gaps in an external tracker use the tracker's own ID.

A tracker entry holds only what derived state needs; its notes live in the tracker:

```yaml
- id: FOOT-412            # the tracker's own ID
  rule: ATTR-003
  status: open
  opened_at: "2026-10-15T01:20:00Z"
```

A gap is closed by whoever resolves it: in the PR with the code fix, or after ratifying the decision that changes or retires the rule. Only the agent can reach the tracker, so it keeps the index current. For an open tracker gap whose issue is closed, the `change` skill checks the rule against the current code and closes the gap only if the rule holds; otherwise it reports the gap. A stale entry errs on the safe side: the rule shows as broken until someone closes it.

- `statute gap close REF` marks a gap closed, for either kind.
- `check` verifies that references to the gaps file resolve. References to an external tracker are recorded, not verified.

### 4.6 Derived state

Computed, never stored:

Rules:

- **in force**: `agreed: agreed`.
- **incomplete**: a rule that is not retired, has `built: built`, and lacks a code link, or lacks both a test link and a `manual_check`. Reported as an error.
- **broken**: a rule that is not retired, has `built: built`, and has at least one open gap.

Links:

- **stale**: a link whose current fingerprint differs from `sig`. A stale link brings its rule up for review.
- **missing**: a link whose target is not found, matches more than one symbol, or sits in a file that does not parse cleanly (4.4). A missing link brings its rule up for review.

## 5. CLI

Every command supports `--json`. Exit codes: 0 = clean, 1 = findings (stale or missing links, failed checks, incomplete rules), 2 = usage or file error. Pending proposals are not findings; `status` lists them. No command blocks a merge; exit codes are for the agent and local scripts.

The base for `check` is the merge-base of `HEAD` with the default branch (`origin/HEAD`), plus uncommitted changes in the working tree, including untracked files.

| Command | Does |
| --- | --- |
| `init` | Writes `statute.yml`, the default layout and the AGENTS.md snippet (6.1) |
| `rule new AREA "TITLE"` | Appends a rule skeleton (statement, `scope`, `exceptions`, `protects`, `boundary`) to `rules/<area>.md` with the next free ID for the file's `prefix`, `agreed: proposed` and `built: planned`; prints the ID |
| `show X...` | Takes one or more files, symbols, rule IDs, terms or decision IDs, and prints each rule once. A file or symbol shows the rules linked to it, with their text and terms. A rule shows its text, links, terms, the decisions that established and changed it, and its gaps. A term shows its definition and the rules that use it. A decision shows its entry |
| `check [--all] [--md]` | Runs every check in section 4, including append-only history against the base, and reports stale and missing links whose targets changed since the base. `--all` re-fingerprints every link and looks for term `code` and `not` names across the repo. `--md` prints the PR "Rule impact" section |
| `link RULE TARGET [--test] [--remove]` | Adds a link with the current fingerprint, or removes one. Refuses an unknown rule and an existing link. Sets `built: built` when the rule becomes complete |
| `review RULE ok\|changed D…\|gap REF` | Records one verdict for the rule (4.4) and refreshes every stale link of that rule |
| `decide --summary … --rationale … [--establishes …] [--changes …] [--retires …] [--terms …] [--actor user\|agent] [--edit]` | Appends a decision with `status: proposed`, sets `agreed: proposed` on the rules in `--changes` and the terms in `--terms`, and prints its ID. Refuses an unknown rule or term, one already in a proposed decision, and a retired rule in `--changes` or `--retires`. `--actor` records who initiated it (default `agent`); `--edit` opens it in `$EDITOR` for people |
| `ratify D...` | Activates the decisions and updates their rules and terms (4.2) |
| `gap open RULE [--ref REF] [--note …]` | Adds an open gap and prints its ID. Without `--ref` it writes a full entry and requires `--note`; with `--ref` it records a tracker issue (4.5) |
| `gap close REF` | Closes a gap |
| `status [--area]` | Per area: rules by agreed/built, broken, stale, incomplete, draft rules, pending decisions with the rules and terms awaiting approval, open gaps, and every manual check |

The "Rule impact" section (`check --md`) lists every rule and term the branch affects, why, and the verdict, in plain language a PM or designer can follow. A rule is affected when one of its links went stale or missing, was added or removed, or was refreshed by `review` (its fingerprint in the lock differs from the base) since the base, or when a decision or gap that names it was added or changed since the base. A term is affected when a decision that lists it was added or changed. Verdicts are derived by comparing with the base, in this order: a link still stale or missing means unreviewed; a new open gap means broken; a new decision that establishes, changes or retires the rule means new, changed or retired; a link added or removed means relinked; otherwise the rule holds. Rules and terms awaiting approval are listed with their pending decisions.

Approval: `ratify` is run by the agent in the session, only after a human has explicitly approved the specific decisions. This is a convention, not an identity check. Pending items never block a merge, so contributors in other timezones are not held up.

### 5.1 Implementation

The CLI is written in Rust and ships as one static binary per platform. Every dependency is a maintained crate; nothing needs a runtime on the user's machine.

| Need | Crate | Notes |
| --- | --- | --- |
| Command line | `clap` | |
| YAML: metadata blocks, frontmatter, decisions, gaps, `statute.yml` | `yaml_serde` | The YAML organization's maintained fork of `serde_yaml` |
| JSON: `statute.lock` (JSON Lines), `--json` | `serde_json` | |
| Markdown: rule and term sections, frontmatter | `pulldown-cmark` | Byte offsets for headings and fenced blocks |
| Symbol fingerprints | `tree-sitter` with `tree-sitter-go`, `tree-sitter-python`, `tree-sitter-rust`; later Swift and TypeScript | Grammars are C, compiled into the binary |
| Hashing | `sha2` | |
| Changed files, base versions of Statute files, approver name | the `git` command | `git diff -z`, plus `git ls-files --others --exclude-standard -z` for untracked files; base files through one `git cat-file --batch` process |
| Repository walk for `check --all` | `ignore` | ripgrep's parallel walker; respects `.gitignore` |
| Finding term names across the repo in `check --all` | `aho-corasick` | One automaton for all `code` and `not` names |
| Parallel work | `rayon` | |
| IDs and timestamps | `rand`, `jiff` | |
| Release | `cargo-dist` | macOS and Linux binaries, Homebrew tap |

Speed. `check` runs in the Stop hook at the end of every agent turn, so its cost must follow the size of the change:

- Load `statute.lock` once and index links by target file. Read only code files that appear in the diff.
- Parse each changed code file once, in parallel, with one tree-sitter parser per thread, and compute every fingerprint that file needs in that pass.
- Statute's own files are small: parse all of them, and the base versions of `decisions.db`, `gaps.yml` and `statute.lock`, on every run.
- `check --all` is the only full-repository pass. It walks files in parallel, re-fingerprints every link, and searches for term names with one automaton.
- Budget: `check` finishes in under 100 ms on Footprints, measured with `hyperfine` from milestone 2 onward.

## 6. Agent plugin

Packaging in the first version:

- Claude Code: a plugin with the skills and the Stop hook.
- Codex: the same skills and the AGENTS.md snippet, without the hook.

The CLI is a prerequisite; each skill checks for it and stops with install instructions if it is missing.

### 6.1 AGENTS.md snippet (installed by `statute init`)

- Before changing code, run `statute show` on the files you will touch and read the rules and terms it returns.
- Change a rule or an agreed term only together with a proposed decision that lists it (`statute decide`), never to match the code. Never edit `agreed` or `built`; the CLI sets them.
- When a user instruction conflicts with an agreed rule, name the rule and propose a decision before changing the code.
- After changing code, run the `change` skill before declaring the task done.
- Anything you create is `proposed`. Run `statute ratify` only after a human in this session explicitly approves those specific decisions.
- Before asking a human to ratify a rule, check it: what valid case would it reject? What invalid case would it allow? Does its meaning change with actor or permission, missing data, lifecycle stage, time, retries, concurrency or partial failure? Add to the rule only the answers that change it.
- Gaps go to the issue tracker named in this file; if none is named, use `statute gap open --note`.
- When instructions conflict: explicit user instruction > current task > project principles > agreed rules, terms and ratified decisions > repo conventions > proposed decisions > agent judgment.

### 6.2 Skills

| Skill | Trigger | Steps | Ends with |
| --- | --- | --- | --- |
| `adopt` | Setting Statute up in a repo | Read existing docs, contracts, ADRs and decision logs and convert them into the Statute format; draft terms with `code` and `not` entries; draft rules per area with `rule new` (statement, terms, protected and boundary cases), each with a code link and a test link | One decision per area that establishes its rules and lists its terms, `statute check` passing, a list for human approval |
| `change` | After any code change in the repo | `statute check`; relink missing targets at their new location; for each rule with stale or missing links: read the rule and code, try the protected and boundary cases, decide holds / changing on purpose / broken; for each open tracker gap whose issue is closed, check the rule against the current code and close the gap only if it holds | `statute review RULE …` per rule, `decide` or a gap as needed, `statute check --md` in the PR, written so a PM or designer can follow it |
| `decide` | A choice passes the decision gates (kept from Footprints AGENTS.md), or a rule or term needs to change | Write summary and rationale; draft new or changed rule sections and terms; ask the review questions from 6.1 and generate cases (retries, concurrency, partial failure, missing data, valid cases the rule would reject); turn them into proposed `protects` and `boundary` cases and proposed tests linked to the rule | Decision with `status: proposed` listing its rules and terms, which stay `proposed` |
| `ratify` | Pending decisions exist, or a human asks to approve | Summarise each pending decision in two lines with its rules, terms and risk; show each changed rule and term as a diff against its agreed text, taken from git: its version in the most recent commit where it read `agreed: agreed`, which may predate the base when the proposal is already on the default branch; show a rule or term that was never agreed as an addition; ask the human to approve, reject or skip each (a skipped decision stays proposed and can be reworked) | `statute ratify D...` for the approved decisions. For each rejected decision: delete its entry; for each rule in its `changes` and each term in its `terms` that was agreed before, restore the compared fields (4.1, 4.3) to the agreed text and set `agreed: agreed`, leaving `built`, notes and links as they are; remove each rule in its `establishes` together with its links (`link --remove`), and each term it introduced; run `statute check` until no reference to the removed items remains |

### 6.3 Hook (Claude Code)

One Stop/end-of-turn hook: `statute check --json`, against the merge-base with the default branch plus the working tree, so commits made during the session are covered. If links are stale or missing, it returns a message telling the agent to run the `change` skill. It never blocks the human and never reviews on its own.

## 7. Footprints migration

Footprints is converted into the Statute format once, with the `adopt` skill and a disposable conversion script that is not part of the CLI. Afterwards it keeps no files in the old formats.

| Today | Statute |
| --- | --- |
| `docs/UBIQUITOUS_LANGUAGE.md` | Converted to `statute/terms.md`; add `code` and `not` entries, starting with "Session" vs the `sessions` table. The terms are approved through the area adopt decisions |
| `agent_rules.yml` | Rewritten as a short "Project principles" section in AGENTS.md, in plain language; the file is removed. These are the "project principles" in the AGENTS.md precedence |
| `decisions_log.yml` | Converted to `statute/decisions.db`; add `establishes`, `changes` and `retires`; convert IDs to the D format and update references to them. Each converted decision is active, with `ratified_by` set to Anton and its original date |
| Contract docs | Converted to `statute/rules/<area>.md` with a `prefix` in the frontmatter; split prose into ID'd rule sections, starting with `session_commit_attribution_contract.md`. Rules that no converted decision establishes are established by the area's adopt decision |
| `no-drift.lock` | Converted to `statute.lock` by the conversion script: run `nodrift check` first and require it to be clean, then recompute every fingerprint with Statute. Bindings to guides are not converted, and the SQL ownership doc stays an ordinary doc |
| Decision Policy section in AGENTS.md | Kept as the `decide` skill's gates; AGENTS.md gets the snippet from 6.1 |
| Beans | Named in AGENTS.md as the issue tracker for gaps |

## 8. Milestones

1. **Model and checks.** Rust CLI skeleton; parsers for terms, rules (with frontmatter prefixes), decisions and the gaps file; `init`, `rule new`, `show` for rules, terms and decisions, `check` for files, `status`. In this milestone `status` reports agreed and built states only; stale, incomplete and broken arrive with milestones 2 to 4. Convert Footprints terms, rules and decisions with a disposable script.
2. **Links.** Reimplement nodrift's fingerprinting in the CLI: `bytes`, and the `go-ast` and `python-ast` modes on tree-sitter; code and test links with the completeness check; `link` (including setting `built`), `review`, `show` for files and symbols, `check` with stale and missing links against the base, and `check --all`. Convert Footprints' `no-drift.lock` with the same script.
3. **Decisions.** `decide`, `ratify`, append-only checks, `check --md`.
4. **Gaps.** `gap open`, `gap close`; the gaps index; derived `broken` state.
5. **Plugin.** AGENTS.md snippet and the four skills for Claude Code and Codex, including tracker reconciliation in `change`; Stop hook for Claude Code.
6. **Seeded violations.** Write the violation set for the pilot rules (section 9).
7. **Second repo.** Port to a deliberately different repo (Swift) by adding a tree-sitter Swift mode; record what else had to change.

Milestones 1–6 are enough to run the 3–4 week pilot on the Footprints attribution contract.

## 9. Pilot measures

Recorded from decisions, gaps and the PR "Rule impact" sections, so measurement costs nothing extra:

- Real problems caught: gaps and rule-changing decisions that a human confirms were real.
- False alarms, per trigger: stale code links and stale test links whose rule held with no relevant change. A high rate means links are too coarse.
- Cost: minutes per PR spent on review and decisions (self-reported in the PR).
- Understanding: can someone who did not write the rules explain the attribution workflow from the rule file alone?
- Contributors: how many changes came from PMs or designers, and whether the loop worked for them without engineer help.

Seeded violations measure detection directly, because real rule breaks may be rare during the pilot:

- For each pilot rule, write several plausible rule-breaking changes of different kinds: an edit inside linked code, new code elsewhere (a new filter, a new caller), a changed default or config, a weakened or deleted test.
- Apply each on a branch, run the loop and the project's CI, and record whether it was caught and by what: code link, test link, failing test in CI, or nothing.
- Report what each trigger caught and its false alarms. This shows which mechanisms earn their place.

## 10. Ideas for v2

Ideas worth keeping that v1 leaves out to stay simple. Each goes in only when the pilot shows a need.

- **Guide links.** Link descriptive docs, such as code maps and ownership docs, to code, so a change to the code flags the doc for an update.
- **Term matching.** When a change touches a term's code names, the rules that use the term come up for review.
- **Markdown section fingerprints.** For rules whose code lives in Markdown, such as agent prompt files.

## 11. Open questions

- Implementation language: Rust (5.1), unless the milestone 1 skeleton shows a problem.

## Appendix. Usage scenarios

Informational. These scenarios show how the commands and skills fit together in everyday work; they add no requirements.

**Starting Statute in a new repo.** A person runs `statute init`, which writes the layout and the AGENTS.md snippet. They create `statute/rules/billing.md` with `prefix: BILL` in its frontmatter. There is nothing to adopt, so rules arrive one feature at a time, as in "A PM adds a feature that needs a new rule".

**Adopting an area in an existing repo.** The `adopt` skill reads the area's docs and contracts, drafts terms, and drafts rules with `rule new`. It links each rule's code and test with `link`, which sets `built: built` as each rule becomes complete. It then runs `decide --establishes ATTR-001,… --terms Session,… --summary "Adopt existing rules" --rationale …`. After a person approves, the agent runs `ratify D…`, and the rules and terms become agreed.

**A PM adds a small feature.** The agent runs `show` on the files it will touch and reads the rules and terms it gets back. It writes the code. At the end of the turn the Stop hook runs `check`, which reports two stale links on ATTR-003, and the agent runs the `change` skill. It reads the rule, tries the protected and boundary cases against the new code, and runs `review ATTR-003 ok`. It runs `check --md` and pastes the "Rule impact" section into the PR. The agent runs three commands; the PM runs none.

**A PM adds a feature that needs a new rule.** Before coding, the agent runs `show` on the files it plans to touch and finds no rules there. The PM says refunds over $500 need a second approval. The `decide` skill drafts the rule with `rule new billing "Large refunds need a second approval"`, which writes BILL-004 as proposed and planned. It works through the review questions, fills in the protected and boundary cases, and runs `decide --establishes BILL-004 --summary … --rationale …`. The PM approves, and the agent runs `ratify D…`. The agent then writes the code and the test, and runs `link BILL-004 billing/refund.go#Approve` and `link BILL-004 billing/refund_test.go#TestLargeRefundNeedsSecondApproval --test`. The second link completes the rule, so `link` sets `built: built`. The rule is agreed before any code is written, and built once both links exist.

**The feature needs a new term.** The new code introduces "Approver". The agent adds an `Approver` entry to `terms.md` with its definition and `code: [Approver]`, lists it in BILL-004's `terms`, and adds `--terms Approver` to the same `decide` call, which marks the term proposed. When the PM approves and the agent runs `ratify`, the rule and the term become agreed together. `check` confirms that the rule's reference resolves, and `check --all` later confirms that the code name exists in the repo.

**A rule that cannot have an automated test.** A designer's rule in `subscriptions.md` says the cancel link is as easy to find as the upgrade link. No test can judge that, so the agent adds `manual_check: "A designer compares the cancel and upgrade links in each release review"` to the rule and links the code that renders both links, and `link` sets `built: built`. The manual check is part of the rule the decision establishes, so the designer approves both together, and `status` lists it with the area's manual checks.

**A rule needs to change.** The PM asks to raise the second-approval threshold to $1,000, which conflicts with BILL-004. The agent names the rule, edits it, and runs `decide --changes BILL-004 --summary … --rationale …`, which sets the rule back to proposed. It updates the code and the test; at the end of the turn `check` reports BILL-004's links as stale, and the agent runs `review BILL-004 changed D…`. The PM approves, and the agent runs `ratify D…`. Until then, `status` and the PR's "Rule impact" section show BILL-004 as awaiting approval.

**A rule is found broken.** An agent changes a function linked to ATTR-003, and the `change` skill finds that the rule no longer holds. With a tracker named in AGENTS.md, the agent opens an issue with the tracker's tools, runs `gap open ATTR-003 --ref FOOT-412`, then `review ATTR-003 gap FOOT-412`. Without a tracker, it runs `gap open ATTR-003 --note …` and passes the gap ID it prints to `review`. Until the gap is closed, `status` shows the rule as broken, and a person decides whether to fix the code or change the rule. The PR that fixes the code runs `gap close FOOT-412`.

**A break caught only by a test.** An agent adds a filter, in code no rule links to, that drops sessions without a repository. No link is stale, but the test linked to ATTR-005 fails in CI. The agent runs `show` on the test file, finds ATTR-005 ("a session without a repository is still kept"), confirms the break, and opens a gap as above. No `review` is needed, because no link changed.

**An engineer changes a function many rules depend on.** An engineer changes a helper that 12 rules link to. `check` lists the 12 rules with their stale links. The agent checks each rule and runs one `review` per rule, which refreshes all of that rule's stale links at once.

**A linked function is renamed.** An engineer renames `Publisher.Decide` to `Publisher.Choose`. `check` reports the links to `Publisher.Decide` as missing. The agent checks each affected rule as for a stale link, then runs `link RULE internal/sessioncommit/publisher.go#Publisher.Decide --remove` and `link RULE internal/sessioncommit/publisher.go#Publisher.Choose`. The PR "Rule impact" section lists those rules as relinked.

**A periodic health check.** A person or agent runs `check --all` and `status`. `check --all` re-fingerprints every link, which catches links made stale or missing by changes that reached the default branch without Statute, and warns about term `code` and `not` names that no longer exist. `status` shows each area's pending decisions, open gaps and manual checks.
