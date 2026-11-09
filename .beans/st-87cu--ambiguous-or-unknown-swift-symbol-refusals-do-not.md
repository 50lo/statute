---
# st-87cu
title: Ambiguous or unknown Swift symbol refusals do not name the candidates
status: todo
type: feature
priority: low
tags:
    - swift
    - link
created_at: 2026-11-09T00:00:57Z
updated_at: 2026-11-09T00:00:57Z
---

## What

When `statute link` refuses a Swift symbol target as ambiguous or not found, the message names neither the declarations that matched nor the accepted name forms. The person has to work out the right target from docs/Statute-spec.md, section 4.4.

## Why

The two cases came up in a real session linking review-map's rules. Each needed extra attempts or a trip to the spec:

- `Analyzer.run` is overloaded: `run(_:log:)` and `run(_:log:index:)`. The refusal didn't say which declarations matched, or that argument labels disambiguate. Finding `Analyzer.run(_:log:index:)` took trial and error.
- `GitRepo.mergeBase` is the only `mergeBase` in `Shell.swift`, but `Shell.swift#mergeBase` was refused as not found, because members need their type path (`#Type.member`). The message doesn't say that, or that a member with that name exists.

The spec defines both behaviors (section 4.4: "Matching multiple declarations is ambiguous"; members are named `#Type.member`), so this is about how clear the messages are, not about wrong results.

## Environment

- statute 0.3.2 on macOS.

## Reproduction

In review-map (https://github.com/50lo/review-map, branch `statute-core-rules`):

1. `statute link CORE-001 'Sources/ReviewMapCore/Analyzer.swift#Analyzer.run'`
2. `statute link CORE-001 'Sources/ReviewMapCore/Analyzer.swift#Analyzer.run' --json`
3. `statute link CORE-001 'Sources/ReviewMapCore/Shell.swift#mergeBase'`

## Observed

All three exit 2:

```
statute: cannot link Sources/ReviewMapCore/Analyzer.swift#Analyzer.run: more than one symbol matches
```

```json
{
  "error": "cannot link Sources/ReviewMapCore/Analyzer.swift#Analyzer.run: more than one symbol matches",
  "schema": 1
}
```

```
statute: cannot link Sources/ReviewMapCore/Shell.swift#mergeBase: the symbol is not found
```

## Expected

- **Ambiguous:** the refusal lists the matching declarations in a form that can be passed back to `link` (for example `Analyzer.run(_:log:)` and `Analyzer.run(_:log:index:)`). The JSON includes them as a field.
- **Not found:** when the file has a symbol with the same base name under a type path, the refusal names it (for example `GitRepo.mergeBase`).

## Acceptance criteria

- An ambiguous Swift symbol refusal lists every matching declaration as a target that `link` accepts, in human and `--json` output.
- A not-found Swift symbol refusal names the qualified symbols in the file whose last component equals the requested name, if there are any.
- Refusals still exit 2 and leave `statute.lock` unchanged.

## Where

- `statute link` refusals for Swift symbol targets. Naming rules are in docs/Statute-spec.md, section 4.4.

## Dependencies and constraints

- Low priority: the current behavior matches the spec.
- The same question may apply to Go, Python and Rust targets; I only checked Swift.
