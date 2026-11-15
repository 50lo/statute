---
# st-4c69
title: Swift parse-error fallback does not say where the parser failed
status: completed
type: feature
priority: normal
tags:
    - swift
    - parser
created_at: 2026-11-09T00:00:56Z
updated_at: 2026-11-09T00:51:30Z
---

## What

When a Swift file does not parse cleanly, `statute link` reports only that "the Swift parser cannot read the file cleanly". The human output and the `--json` output do not say where in the file the parser failed (a line, a column, or the construct). The same holds for the `link.missing` explanation in `check` and `review` (docs/Statute-spec.md, section 4.4).

## Why

Without a location, a person or an agent cannot tell which construct caused the fallback, so they can't tell whether it can be avoided or worked around. In review-map, `Sources/ReviewMapCore/Annotator.swift` is about 790 lines. Finding the cause, a multi-pattern `catch` clause (reported in st-12vb), took a manual bisection: about 11 probe files, one per top-level type, then 10 more, one per member of the failing type, then 4 single-construct files. Each probe had to be linked and then unlinked again, because a successful probe writes a link to `statute.lock`.

## Environment

- statute 0.3.2 on macOS.

## Reproduction

1. In a repository that uses statute, with any rule (here `CORE-001`), add `repro-tmp/MultiCatch.swift`, a valid Swift file that the parser cannot read (see st-12vb):

   ```swift
   enum Decode {
       static func describe(_ data: Data) -> String {
           do {
               _ = try JSONDecoder().decode([String].self, from: data)
               return "ok"
           } catch let DecodingError.typeMismatch(_, ctx), let DecodingError.valueNotFound(_, ctx) {
               return ctx.debugDescription
           } catch {
               return "other"
           }
       }
   }
   ```

2. Run `statute link CORE-001 'repro-tmp/MultiCatch.swift#Decode.describe'`, with and without `--json`.

## Observed

Human output:

```
Linked CORE-001 to repro-tmp/MultiCatch.swift (whole file, code): repro-tmp/MultiCatch.swift#Decode.describe could not be linked because the Swift parser cannot read the file cleanly
```

JSON output has `schema`, `rule`, `target`, `role`, `mode`, `sig`, `removed`, `built`, `requested` and `fallback: "parse-error"`. Neither output gives a position or a construct.

## Expected

The fallback report (human and JSON) says where the parser first failed, at least a line, so people can find the construct without bisecting the file.

## Acceptance criteria

- `link`'s parse-error fallback report includes the location of at least the first parse error, in both human and `--json` output.
- If `check` or `review` explain a Swift parse error, they include the same location.
- Existing JSON fields keep their names and meaning.

## Where

- `statute link` fallback reporting, and the Swift parse-error explanation in `check` and `review`. The behavior is specified in docs/Statute-spec.md, section 4.4. The fallback came from st-4dc0.

## Dependencies and constraints

- Normal priority: the fallback is safe, but without a location every unsupported Swift construct needs a manual bisection of the file (about 25 probe files in review-map), and it helps with any future grammar gap, not only st-12vb.
