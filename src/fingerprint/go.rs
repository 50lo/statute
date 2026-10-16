//! `go-ast-sha256-v1`: Go functions, types and methods.
//!
//! Symbols are `Func`, `Type` and `Type.Method`. A method's receiver type is named without `*` and
//! without type parameters. `#Type` covers the type's own spec, not its methods. Comments are dropped,
//! except `//go:` directives directly above the declaration and the file's `//go:build` line, which are
//! hashed with the symbol. A newline or `;` that ends a statement is formatting and is dropped; the
//! end of each statement and field is marked instead.

use super::Missing;
use super::syntax::{Input, Visit};
use std::collections::BTreeMap;
use tree_sitter::{Node, Parser};

/// The fingerprint of each symbol in one Go file, in order.
pub fn sign(source: &[u8], symbols: &[&str]) -> Vec<Result<String, Missing>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .expect("the Go grammar matches the tree-sitter version");
    let tree = match parser.parse(source, None) {
        Some(tree) if !tree.root_node().has_error() => tree,
        _ => return vec![Err(Missing::ParseError); symbols.len()],
    };
    let root = tree.root_node();

    let mut found: BTreeMap<String, Vec<(Node, Node)>> = BTreeMap::new();
    for i in 0..root.child_count() {
        let Some(decl) = root.child(i) else { continue };
        for (name, node) in declared(decl, source) {
            found.entry(name).or_default().push((decl, node));
        }
    }
    let build_line = build_line(root, source);

    symbols
        .iter()
        .map(|symbol| match found.get(*symbol).map(Vec::as_slice) {
            None | Some([]) => Err(Missing::NoSymbol),
            Some([(decl, node)]) => {
                let mut input = Input::new();
                if let Some(line) = build_line {
                    input.token(line);
                }
                for directive in directives(*decl, source) {
                    input.token(directive);
                }
                input.node(*node, source, &visit);
                Ok(input.finish())
            }
            Some(_) => Err(Missing::Ambiguous),
        })
        .collect()
}

/// The symbols a top-level declaration declares, with the node each one hashes.
fn declared<'t>(decl: Node<'t>, source: &[u8]) -> Vec<(String, Node<'t>)> {
    let name = |n: Node| text(n.child_by_field_name("name")?, source).map(str::to_string);
    match decl.kind() {
        "function_declaration" => name(decl).map(|n| vec![(n, decl)]).unwrap_or_default(),
        "method_declaration" => match (receiver(decl, source), name(decl)) {
            (Some(recv), Some(method)) => vec![(format!("{recv}.{method}"), decl)],
            _ => Vec::new(),
        },
        "type_declaration" => (0..decl.child_count())
            .filter_map(|i| decl.child(i))
            .filter(|spec| matches!(spec.kind(), "type_spec" | "type_alias"))
            .filter_map(|spec| Some((name(spec)?, spec)))
            .collect(),
        _ => Vec::new(),
    }
}

/// The receiver's type name: `Publisher` for `(p *Publisher[T])`.
fn receiver<'s>(method: Node, source: &'s [u8]) -> Option<&'s str> {
    let params = method.child_by_field_name("receiver")?;
    let param = (0..params.child_count())
        .filter_map(|i| params.child(i))
        .find(|c| c.kind() == "parameter_declaration")?;
    let mut ty = param.child_by_field_name("type")?;
    loop {
        match ty.kind() {
            "pointer_type" | "parenthesized_type" => {
                ty = (0..ty.child_count())
                    .filter_map(|i| ty.child(i))
                    .find(|c| c.is_named())?;
            }
            "generic_type" => ty = ty.child_by_field_name("type")?,
            _ => return text(ty, source),
        }
    }
}

/// `//go:` comments on the lines directly above a declaration, in source order.
fn directives<'s>(decl: Node, source: &'s [u8]) -> Vec<&'s [u8]> {
    let mut out = Vec::new();
    let mut next = decl;
    while let Some(prev) = next.prev_sibling() {
        if prev.kind() != "comment" || prev.end_position().row + 1 < next.start_position().row {
            break;
        }
        let comment = &source[prev.byte_range()];
        if comment.starts_with(b"//go:") {
            out.push(comment);
        }
        next = prev;
    }
    out.reverse();
    out
}

/// The `//go:build` line above the package clause.
fn build_line<'s>(root: Node, source: &'s [u8]) -> Option<&'s [u8]> {
    (0..root.child_count())
        .filter_map(|i| root.child(i))
        .take_while(|n| n.kind() != "package_clause")
        .map(|n| &source[n.byte_range()])
        .find(|t| t.starts_with(b"//go:build"))
}

/// Comments, and the newlines and semicolons that end statements, are not part of the hash input;
/// the end of each statement and field is marked instead.
fn visit(node: Node) -> Visit {
    let terminator = node.child_count() == 0
        && (node.kind() == "\n"
            || (node.kind() == ";" && node.parent().is_none_or(|p| p.kind() != "for_clause")));
    if terminator || node.kind() == "comment" {
        Visit::Skip
    } else if matches!(node.kind(), "statement_list" | "field_declaration_list") {
        Visit::List
    } else {
        Visit::Enter
    }
}

fn text<'s>(node: Node, source: &'s [u8]) -> Option<&'s str> {
    node.utf8_text(source).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! fixtures {
        ($($name:literal),*) => {
            /// A fixture file, compiled in so the test reads no files.
            fn fixture(name: &str) -> &'static [u8] {
                match name {
                    $($name => include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/tests/fixtures/fingerprint/go/",
                        $name
                    )),)*
                    _ => panic!("no fixture {name}"),
                }
            }
        };
    }

    fixtures!(
        "base.go",
        "same_comments.go",
        "same_formatting.go",
        "same_other_declarations.go",
        "changed.go",
        "changed_directive.go",
        "changed_build_line.go",
        "changed_statements.go",
        "parse_error.go",
        "golden.txt"
    );
    const SYMBOLS: &[&str] = &[
        "Publisher",
        "Publisher.Decide",
        "Publisher.Size",
        "Score",
        "ID",
        "Alias",
    ];

    fn hashes(file: &str) -> Vec<Result<String, Missing>> {
        sign(fixture(file), SYMBOLS)
    }

    /// Which symbols of each fixture must keep base.go's hash (`true`) or change it (`false`).
    #[test]
    fn edits_keep_or_change_the_hash() {
        let base = hashes("base.go");
        let cases: &[(&str, [bool; 6])] = &[
            ("same_comments.go", [true; 6]),
            ("same_formatting.go", [true; 6]),
            (
                "same_other_declarations.go",
                [true, true, false, true, true, true],
            ),
            ("changed.go", [false; 6]),
            (
                "changed_directive.go",
                [true, true, true, false, true, true],
            ),
            ("changed_build_line.go", [false; 6]),
            (
                "changed_statements.go",
                [true, true, true, false, true, true],
            ),
        ];
        for (file, expected) in cases {
            let other = hashes(file);
            for ((symbol, keep), (a, b)) in
                SYMBOLS.iter().zip(expected).zip(base.iter().zip(&other))
            {
                assert!(a.is_ok() && b.is_ok(), "{file} {symbol}: {a:?} {b:?}");
                assert_eq!(a == b, *keep, "{file} {symbol}");
            }
        }
    }

    /// The hash input is fixed: a change to any golden hash is a new mode version.
    #[test]
    fn golden_hashes() {
        let golden = String::from_utf8_lossy(fixture("golden.txt"));
        let actual: String = SYMBOLS
            .iter()
            .zip(hashes("base.go"))
            .map(|(s, h)| format!("{s} {}\n", h.unwrap()))
            .collect();
        assert_eq!(actual, golden);
    }

    #[test]
    fn missing_ambiguous_and_unparseable() {
        let results = sign(
            fixture("base.go"),
            &[
                "init",
                "Nope",
                "Publisher.Nope",
                "Size",
                "*Publisher.Decide",
            ],
        );
        assert_eq!(results[0], Err(Missing::Ambiguous));
        assert!(
            results[1..].iter().all(|r| *r == Err(Missing::NoSymbol)),
            "{results:?}"
        );

        assert_eq!(
            sign(fixture("parse_error.go"), &["Other"]),
            vec![Err(Missing::ParseError)]
        );
    }
}
