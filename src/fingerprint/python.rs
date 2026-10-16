//! `python-ast-sha256-v1`: Python functions, classes and methods.
//!
//! Symbols are `func`, `Class` and `Class.method`, with nested classes as dotted paths such as
//! `Outer.Inner.method`. Decorators are part of the symbol, and `#Class` covers the whole class,
//! methods included. Comments and docstrings are dropped, and so are `;` separators. Because
//! indentation carries meaning, every block is marked where it starts and ends, and so is the end of
//! each statement in it.

use super::Missing;
use super::syntax::{Input, Visit};
use std::collections::BTreeMap;
use tree_sitter::{Node, Parser};

/// The fingerprint of each symbol in one Python file, in order.
pub fn sign(source: &[u8], symbols: &[&str]) -> Vec<Result<String, Missing>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("the Python grammar matches the tree-sitter version");
    let tree = match parser.parse(source, None) {
        Some(tree) if !tree.root_node().has_error() => tree,
        _ => return vec![Err(Missing::ParseError); symbols.len()],
    };

    let mut found: BTreeMap<String, Vec<Node>> = BTreeMap::new();
    collect(tree.root_node(), "", source, &mut found);

    symbols
        .iter()
        .map(|symbol| match found.get(*symbol).map(Vec::as_slice) {
            None | Some([]) => Err(Missing::NoSymbol),
            Some([node]) => {
                let mut input = Input::new();
                input.node(*node, source, &visit);
                Ok(input.finish())
            }
            Some(_) => Err(Missing::Ambiguous),
        })
        .collect()
}

/// Adds the definitions directly in `scope` (a module or a class body) under `prefix`, and recurses
/// into classes. Functions are symbols only at module level or directly in a class.
fn collect<'t>(
    scope: Node<'t>,
    prefix: &str,
    source: &[u8],
    found: &mut BTreeMap<String, Vec<Node<'t>>>,
) {
    for i in 0..scope.child_count() {
        let Some(node) = scope.child(i) else { continue };
        let definition = match node.kind() {
            "decorated_definition" => match node.child_by_field_name("definition") {
                Some(d) => d,
                None => continue,
            },
            "function_definition" | "class_definition" => node,
            _ => continue,
        };
        let Some(name) = definition
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
        else {
            continue;
        };
        let path = format!("{prefix}{name}");
        found.entry(path.clone()).or_default().push(node);
        if definition.kind() == "class_definition"
            && let Some(body) = definition.child_by_field_name("body")
        {
            collect(body, &format!("{path}."), source, found);
        }
    }
}

/// Comments, docstrings and `;` are not part of the hash input; blocks and their statements are
/// marked.
fn visit(node: Node) -> Visit {
    match node.kind() {
        "comment" => Visit::Skip,
        ";" if node.child_count() == 0 => Visit::Skip,
        "block" => Visit::Block,
        _ if is_docstring(node) => Visit::Skip,
        _ => Visit::Enter,
    }
}

/// A string literal that is the first statement of a function or class body.
fn is_docstring(node: Node) -> bool {
    let is_string = |n: Node| matches!(n.kind(), "string" | "concatenated_string");
    node.kind() == "expression_statement"
        && node.named_child_count() == 1
        && node.named_child(0).is_some_and(is_string)
        && node.parent().is_some_and(|block| {
            block.kind() == "block"
                && block.named_child(0) == Some(node)
                && block
                    .parent()
                    .is_some_and(|d| matches!(d.kind(), "function_definition" | "class_definition"))
        })
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
                        "/tests/fixtures/fingerprint/python/",
                        $name
                    )),)*
                    _ => panic!("no fixture {name}"),
                }
            }
        };
    }

    fixtures!(
        "base.py",
        "same_comments.py",
        "same_formatting.py",
        "same_other_declarations.py",
        "changed.py",
        "changed_indentation.py",
        "changed_decorator.py",
        "changed_statements.py",
        "parse_error.py",
        "golden.txt"
    );

    const SYMBOLS: &[&str] = &[
        "load",
        "Store",
        "Store.get",
        "Store.Meta",
        "Store.Meta.version",
        "helper",
    ];

    fn hashes(file: &str) -> Vec<Result<String, Missing>> {
        sign(fixture(file), SYMBOLS)
    }

    /// Which symbols of each fixture must keep base.py's hash (`true`) or change it (`false`).
    #[test]
    fn edits_keep_or_change_the_hash() {
        let base = hashes("base.py");
        let cases: &[(&str, [bool; 6])] = &[
            ("same_comments.py", [true; 6]),
            ("same_formatting.py", [true; 6]),
            (
                "same_other_declarations.py",
                [true, true, true, true, true, false],
            ),
            ("changed.py", [false; 6]),
            (
                "changed_indentation.py",
                [false, true, true, true, true, true],
            ),
            (
                "changed_decorator.py",
                [false, true, true, true, true, true],
            ),
            (
                "changed_statements.py",
                [true, true, true, true, true, false],
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
            .zip(hashes("base.py"))
            .map(|(s, h)| format!("{s} {}\n", h.unwrap()))
            .collect();
        assert_eq!(actual, golden);
    }

    #[test]
    fn missing_ambiguous_and_unparseable() {
        let source = b"class A:\n    @property\n    def x(self): return 1\n\n    @x.setter\n    def x(self, v): pass\n\ndef outer():\n    def inner(): pass\n";
        let results = sign(source, &["A.x", "outer.inner", "B", "A.y"]);
        assert_eq!(results[0], Err(Missing::Ambiguous));
        assert!(
            results[1..].iter().all(|r| *r == Err(Missing::NoSymbol)),
            "{results:?}"
        );
        assert_eq!(
            sign(fixture("parse_error.py"), &["other"]),
            vec![Err(Missing::ParseError)]
        );
    }
}
