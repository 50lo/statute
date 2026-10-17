//! `rust-ast-sha256-v1`: Rust functions, types, traits and methods.
//!
//! Symbols are `func`, `Type` (struct, enum, union, trait or type alias) and `Type.method`, with
//! inline modules as dotted paths such as `tests.decides`. A method is named by the type its `impl`
//! block is for, without generics, `&` or a path, so methods of inherent and trait impls share one
//! namespace. Trait items are `Trait.method`. `#Type` covers the type's own item, not its impl
//! blocks; `#Trait` covers the whole trait. Items inside functions are not symbols. Outer attributes
//! are part of the item. Comments, doc comments and trailing commas are dropped.

use super::Missing;
use super::syntax::{Input, Visit};
use std::collections::BTreeMap;
use tree_sitter::{Node, Parser};

/// The fingerprint of each symbol in one Rust file, in order.
pub fn sign(source: &[u8], symbols: &[&str]) -> Vec<Result<String, Missing>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("the Rust grammar matches the tree-sitter version");
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
            Some([item]) => {
                let mut input = Input::new();
                for attribute in attributes(*item) {
                    input.node(attribute, source, &visit);
                }
                input.node(*item, source, &visit);
                Ok(input.finish())
            }
            Some(_) => Err(Missing::Ambiguous),
        })
        .collect()
}

/// Adds the items of a module body under `prefix`. Recurses into inline modules, and into impl and
/// trait bodies for their functions.
fn collect<'t>(
    scope: Node<'t>,
    prefix: &str,
    source: &[u8],
    found: &mut BTreeMap<String, Vec<Node<'t>>>,
) {
    let name = |n: Node| n.child_by_field_name("name")?.utf8_text(source).ok();
    for i in 0..scope.child_count() {
        let Some(item) = scope.child(i) else { continue };
        match item.kind() {
            "function_item" | "struct_item" | "enum_item" | "union_item" | "type_item" => {
                if let Some(name) = name(item) {
                    found
                        .entry(format!("{prefix}{name}"))
                        .or_default()
                        .push(item);
                }
            }
            "trait_item" => {
                if let Some(name) = name(item) {
                    let path = format!("{prefix}{name}");
                    found.entry(path.clone()).or_default().push(item);
                    functions(item, &format!("{path}."), source, found);
                }
            }
            "impl_item" => {
                if let Some(ty) = item
                    .child_by_field_name("type")
                    .and_then(|t| type_name(t, source))
                {
                    functions(item, &format!("{prefix}{ty}."), source, found);
                }
            }
            "mod_item" => {
                if let (Some(name), Some(body)) = (name(item), item.child_by_field_name("body")) {
                    collect(body, &format!("{prefix}{name}."), source, found);
                }
            }
            _ => {}
        }
    }
}

/// Adds the functions of an impl or trait body under `prefix`.
fn functions<'t>(
    item: Node<'t>,
    prefix: &str,
    source: &[u8],
    found: &mut BTreeMap<String, Vec<Node<'t>>>,
) {
    let Some(body) = item.child_by_field_name("body") else {
        return;
    };
    for i in 0..body.child_count() {
        let Some(f) = body.child(i) else { continue };
        if matches!(f.kind(), "function_item" | "function_signature_item")
            && let Some(name) = f
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
        {
            found.entry(format!("{prefix}{name}")).or_default().push(f);
        }
    }
}

/// The name of the type an impl block is for: `Publisher` for `&mut billing::Publisher<T>`.
fn type_name<'s>(mut ty: Node, source: &'s [u8]) -> Option<&'s str> {
    loop {
        match ty.kind() {
            "generic_type" | "reference_type" | "pointer_type" => {
                ty = ty.child_by_field_name("type")?
            }
            "scoped_type_identifier" => ty = ty.child_by_field_name("name")?,
            "type_identifier" | "primitive_type" => return ty.utf8_text(source).ok(),
            _ => return None,
        }
    }
}

/// The outer attributes directly above an item, in source order. Comments between them are skipped.
fn attributes(item: Node) -> Vec<Node> {
    let mut out = Vec::new();
    let mut prev = item.prev_sibling();
    while let Some(node) = prev {
        match node.kind() {
            "attribute_item" => out.push(node),
            _ if is_comment(node) => {}
            _ => break,
        }
        prev = node.prev_sibling();
    }
    out.reverse();
    out
}

/// Comments and trailing commas are not part of the hash input.
fn visit(node: Node) -> Visit {
    if is_comment(node) || is_trailing_comma(node) {
        Visit::Skip
    } else {
        Visit::Enter
    }
}

fn is_comment(node: Node) -> bool {
    matches!(node.kind(), "line_comment" | "block_comment")
}

/// A `,` before a closing delimiter or at the end of its list, which rustfmt adds or removes when it
/// reflows lines. The comma of a one-element tuple, `(x,)`, is code; so is a lone comma in a macro's
/// tokens, where tuples cannot be told apart.
fn is_trailing_comma(node: Node) -> bool {
    if node.kind() != "," || node.child_count() != 0 {
        return false;
    }
    let mut next = node.next_sibling();
    while let Some(n) = next.filter(|n| is_comment(*n)) {
        next = n.next_sibling();
    }
    let closes = next.is_none_or(|n| matches!(n.kind(), ")" | "]" | "}" | ">" | "|"));
    let one_tuple = node.parent().is_some_and(|p| {
        matches!(
            p.kind(),
            "tuple_expression" | "tuple_type" | "tuple_pattern" | "token_tree"
        ) && p.named_child_count() == 1
    });
    closes && !one_tuple
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
                        "/tests/fixtures/fingerprint/rust/",
                        $name
                    )),)*
                    _ => panic!("no fixture {name}"),
                }
            }
        };
    }

    fixtures!(
        "base.rs",
        "same_comments.rs",
        "same_formatting.rs",
        "same_other_declarations.rs",
        "changed.rs",
        "changed_attributes.rs",
        "changed_statements.rs",
        "parse_error.rs",
        "golden.txt"
    );

    const SYMBOLS: &[&str] = &[
        "Publisher",
        "Publisher.decide",
        "Publisher.fmt",
        "Decide",
        "Decide.ready",
        "score",
        "Kind",
        "Id",
        "inner.helper",
        "tests.decides",
    ];

    fn hashes(file: &str) -> Vec<Result<String, Missing>> {
        sign(fixture(file), SYMBOLS)
    }

    /// Which symbols of each fixture must keep base.rs's hash (`true`) or change it (`false`).
    #[test]
    fn edits_keep_or_change_the_hash() {
        let base = hashes("base.rs");
        let (t, f) = (true, false);
        let cases: &[(&str, [bool; 10])] = &[
            ("same_comments.rs", [t; 10]),
            ("same_formatting.rs", [t; 10]),
            ("same_other_declarations.rs", [t, t, t, t, t, t, t, t, t, f]),
            ("changed.rs", [f; 10]),
            ("changed_attributes.rs", [f, t, t, t, t, f, t, t, t, f]),
            ("changed_statements.rs", [t, t, t, t, t, f, t, t, f, t]),
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
            .zip(hashes("base.rs"))
            .map(|(s, h)| format!("{s} {}\n", h.unwrap()))
            .collect();
        assert_eq!(actual, golden);
    }

    #[test]
    fn missing_ambiguous_and_unparseable() {
        let source = b"struct A;\nimpl A { fn new() {} }\nimpl Clone for A { fn clone(&self) -> A { A } }\nimpl<T> From<T> for A { fn from(_: T) -> A { A } }\nimpl<T> From<Vec<T>> for A { fn from(_: Vec<T>) -> A { A } }\nfn outer() { fn inner() {} }\nmod m {}\n";
        let results = sign(
            source,
            &["A.from", "A.new", "A.clone", "outer.inner", "m", "B", "new"],
        );
        assert_eq!(results[0], Err(Missing::Ambiguous));
        assert!(results[1].is_ok() && results[2].is_ok(), "{results:?}");
        assert!(
            results[3..].iter().all(|r| *r == Err(Missing::NoSymbol)),
            "{results:?}"
        );
        assert_eq!(
            sign(fixture("parse_error.rs"), &["keep"]),
            vec![Err(Missing::ParseError)]
        );
    }

    #[test]
    fn impl_types_are_named_without_generics_references_or_paths() {
        let source = b"impl<'a> outer::inner::Wrapper<'a> { fn get(&self) {} }\nimpl Show for &mut Other { fn show(&self) {} }\n";
        let results = sign(source, &["Wrapper.get", "Other.show"]);
        assert!(results.iter().all(Result::is_ok), "{results:?}");
    }
}
