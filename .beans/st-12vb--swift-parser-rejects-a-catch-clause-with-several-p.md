---
# st-12vb
title: Swift parser rejects a catch clause with several patterns
status: todo
type: bug
priority: normal
tags:
    - swift
    - parser
created_at: 2026-11-09T00:00:43Z
updated_at: 2026-11-09T00:00:43Z
---

## What

statute's Swift parser cannot read a file that contains a `catch` clause with more than one pattern, such as `catch let A(...), let B(...)`. This syntax is valid Swift (multi-pattern catch clauses, Swift 5.3, SE-0276), and `swiftc -parse` accepts the file. Because the whole file counts as not parsing cleanly, `statute link` cannot link any symbol in it. Every symbol link to that file falls back to a whole-file link, including links to declarations that are nowhere near the `catch` clause.

## Why

Swift projects that use this syntax lose symbol-level links for the whole file. In review-map, `Sources/ReviewMapCore/Annotator.swift` (about 790 lines, about ten types) had one such clause in `Annotator.parseJSON`. Rules that needed `Annotator.run` could only link the whole file, so any edit anywhere in the file would mark them stale. review-map worked around it by splitting the clause in two (PR 50lo/review-map#35, commit 2c14b06); without that workaround, the file stays whole-file only.

## Environment

- statute 0.3.2 on macOS (Darwin 27.0.0).
- `Cargo.toml` pins `tree-sitter-swift = "=0.7.4"`.
- The Swift toolchain that accepts the file is Xcode 26's (Swift 6.2).

## Reproduction

1. In a repository that uses statute, with any rule (here `CORE-001`), add `repro-tmp/MultiCatch.swift`:

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

2. `xcrun swiftc -parse repro-tmp/MultiCatch.swift` exits 0.
3. Run `statute link CORE-001 'repro-tmp/MultiCatch.swift#Decode.describe'`.

## Observed

The command exits 0 and records a whole-file `bytes-sha256-v1` link:

```
Linked CORE-001 to repro-tmp/MultiCatch.swift (whole file, code): repro-tmp/MultiCatch.swift#Decode.describe could not be linked because the Swift parser cannot read the file cleanly
```

With `--json`: `"target": "repro-tmp/MultiCatch.swift"`, `"mode": "bytes-sha256-v1"`, `"requested": "repro-tmp/MultiCatch.swift#Decode.describe"`, `"fallback": "parse-error"`.

## Expected

The file parses cleanly, and `Decode.describe` is linked as a symbol with `swift-ast-sha256-v1`, as it is for an equivalent file.

## Control cases

Each case below was checked separately, in a file of its own:

- The same function with the clause split into two single-pattern clauses (`catch let DecodingError.typeMismatch(_, ctx) { … } catch let DecodingError.valueNotFound(_, ctx) { … }`) links as a symbol: `Linked CORE-001 to repro-tmp/SingleCatch.swift#Decode.describe (code)`.
- A single-pattern `catch let DecodingError.keyNotFound(key, ctx)` parses.
- A single-pattern `catch let DecodingError.typeMismatch(_, ctx)` parses.
- An argument label named `as` (`func f<T>(_ x: Int, as: T.Type)`) parses.
- A multi-line string literal with an interpolated ternary, or with a `\` line continuation, parses.

## Acceptance criteria

- A Swift file whose only unusual construct is a multi-pattern `catch` clause parses cleanly, and its symbols can be linked with `swift-ast-sha256-v1`.
- If the construct cannot be supported, statute documents that it is unsupported (for example in docs/Statute-spec.md, near the Swift symbol rules in section 4.4), so users know why a file falls back.

## Where

- Swift symbol parsing and the `swift-ast-sha256-v1` mode: docs/Statute-spec.md, section 4.4 (the `swift-ast-sha256-v1` row and the paragraphs after it), and the "Symbol fingerprints" row that lists `tree-sitter-swift`.
- Related completed work: st-daf0 (Swift mode), st-4dc0 (whole-file fallback).

## Dependencies and constraints

- The whole-file fallback itself works as specified (st-4dc0) and must keep working for constructs that stay unsupported.
