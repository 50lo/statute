//! `swift-ast-sha256-v1`: Swift declarations and members.

use super::Missing;
use super::syntax::{Input, Visit};
use std::cell::RefCell;
use std::collections::BTreeMap;
use tree_sitter::{Node, Parser};

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new({
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_swift::LANGUAGE.into())
            .expect("the Swift grammar matches the tree-sitter version");
        parser
    });
}

/// The fingerprint of each symbol in one Swift file, in order.
pub fn sign(source: &[u8], symbols: &[&str]) -> Vec<Result<String, Missing>> {
    let tree = PARSER.with(|parser| parser.borrow_mut().parse(source, None));
    let tree = match tree {
        Some(tree) if !tree.root_node().has_error() => tree,
        _ => return vec![Err(Missing::ParseError); symbols.len()],
    };
    let mut found = BTreeMap::new();
    collect(tree.root_node(), "", false, source, &mut found);
    symbols
        .iter()
        .map(|symbol| match found.get(*symbol).map(Vec::as_slice) {
            None | Some([]) => Err(Missing::NoSymbol),
            Some([item]) => {
                let mut input = Input::new();
                input.node(*item, source, &visit);
                Ok(input.finish())
            }
            Some(_) => Err(Missing::Ambiguous),
        })
        .collect()
}

/// Collect declarations only in file and type bodies, never inside function bodies.
fn collect<'t>(
    scope: Node<'t>,
    prefix: &str,
    member: bool,
    source: &[u8],
    found: &mut BTreeMap<String, Vec<Node<'t>>>,
) {
    for i in 0..scope.named_child_count() {
        let Some(item) = scope.named_child(i as u32) else {
            continue;
        };
        match item.kind() {
            "class_declaration" | "protocol_declaration" => {
                let Some(name) = item.child_by_field_name("name") else {
                    continue;
                };
                let extension = (0..item.child_count())
                    .filter_map(|i| item.child(i))
                    .any(|n| n.kind() == "extension");
                let path = if extension {
                    // Extension names can contain nested types and generic arguments.
                    let mut parts = Vec::new();
                    for i in 0..name.named_child_count() {
                        if let Some(child) = name
                            .named_child(i as u32)
                            .filter(|n| n.kind() == "type_identifier")
                            && let Ok(part) = child.utf8_text(source)
                        {
                            parts.push(part);
                        }
                    }
                    format!("{prefix}{}", parts.join("."))
                } else if let Ok(name) = name.utf8_text(source) {
                    format!("{prefix}{name}")
                } else {
                    continue;
                };
                if !extension {
                    found.entry(path.clone()).or_default().push(item);
                }
                // Requirements are covered by the protocol, not separate symbols.
                if item.kind() != "protocol_declaration"
                    && let Some(body) = item.child_by_field_name("body")
                {
                    collect(body, &format!("{path}."), true, source, found);
                }
            }
            "typealias_declaration" => {
                if let Some(name) = item
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source).ok())
                {
                    found
                        .entry(format!("{prefix}{name}"))
                        .or_default()
                        .push(item);
                }
            }
            "function_declaration"
            | "init_declaration"
            | "deinit_declaration"
            | "subscript_declaration" => {
                if !member && item.kind() != "function_declaration" {
                    continue;
                }
                let name = match item.kind() {
                    "init_declaration" => Some("init"),
                    "deinit_declaration" => Some("deinit"),
                    "subscript_declaration" => Some("subscript"),
                    _ => (0..item.child_count())
                        .find(|&i| item.field_name_for_child(i) == Some("name"))
                        .and_then(|i| item.child(i))
                        .and_then(|n| n.utf8_text(source).ok()),
                };
                if let Some(name) = name {
                    let path = format!("{prefix}{name}");
                    found.entry(path.clone()).or_default().push(item);
                    if name != "deinit" {
                        let mut labels = String::new();
                        for i in 0..item.named_child_count() {
                            let Some(param) = item
                                .named_child(i as u32)
                                .filter(|n| n.kind() == "parameter")
                            else {
                                continue;
                            };
                            if let Some(label) = param
                                .child_by_field_name("external_name")
                                .or_else(|| param.child_by_field_name("name"))
                                .and_then(|n| n.utf8_text(source).ok())
                            {
                                labels.push_str(label);
                                labels.push(':');
                            }
                        }
                        found
                            .entry(format!("{path}({labels})"))
                            .or_default()
                            .push(item);
                    }
                }
            }
            "property_declaration" if member => {
                for i in 0..item.named_child_count() {
                    let Some(pattern) =
                        item.named_child(i as u32).filter(|n| n.kind() == "pattern")
                    else {
                        continue;
                    };
                    if let Some(name) = pattern
                        .child_by_field_name("bound_identifier")
                        .and_then(|n| n.utf8_text(source).ok())
                    {
                        found
                            .entry(format!("{prefix}{name}"))
                            .or_default()
                            .push(item);
                    }
                }
            }
            _ => {}
        }
    }
}

fn visit(node: Node) -> Visit {
    if is_comment(node) || is_trailing_comma(node) {
        Visit::Skip
    } else if node.kind() == "statements" {
        Visit::List
    } else {
        Visit::Enter
    }
}

fn is_comment(node: Node) -> bool {
    matches!(node.kind(), "comment" | "multiline_comment")
}

fn is_trailing_comma(node: Node) -> bool {
    if node.kind() != "," || node.child_count() != 0 {
        return false;
    }
    let mut ancestor = node.parent();
    while let Some(n) = ancestor {
        if matches!(n.kind(), "macro_invocation" | "attribute") {
            return false;
        }
        ancestor = n.parent();
    }
    let mut next = node.next_sibling();
    while let Some(n) = next.filter(|n| is_comment(*n)) {
        next = n.next_sibling();
    }
    next.is_some_and(|n| matches!(n.kind(), ")" | "]" | "}" | ">"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: &[u8] = include_bytes!("../../tests/fixtures/fingerprint/swift/base.swift");
    const GOLDEN: &str = include_str!("../../tests/fixtures/fingerprint/swift/golden.txt");
    const SYMBOLS: &[&str] = &[
        "Invoice",
        "Invoice.total",
        "Invoice.approve(name:)",
        "Invoice.Item.value()",
        "global(_:)",
        "P",
        "P.run()",
        "Invoice.move(to:)",
    ];

    fn one(source: &str, name: &str) -> Result<String, Missing> {
        sign(source.as_bytes(), &[name]).remove(0)
    }

    #[test]
    fn golden_hashes() {
        let actual: String = SYMBOLS
            .iter()
            .zip(sign(BASE, SYMBOLS))
            .map(|(s, h)| format!("{s} {}\n", h.unwrap()))
            .collect();
        assert_eq!(actual, GOLDEN);
    }

    #[test]
    fn names_and_ambiguity() {
        let source = r#"
struct Outer { struct Inner { func method() {} } }
extension Outer.Inner {
 func f(to x: Int) {}
 func f(by x: Int) {}
}
protocol P { func run() }
extension P { func run() {} }
struct A {
 var x = 1, y = 2
 init(name: String) {}
 init(id: Int) {}
 subscript(_ n: Int) -> Int { n }
}
#if DEBUG
func repeated() {}
#else
func repeated() {}
#endif
"#;
        for name in [
            "Outer.Inner.method",
            "Outer.Inner.f(to:)",
            "Outer.Inner.f(by:)",
            "P.run",
            "A.x",
            "A.y",
            "A.init(name:)",
            "A.init(id:)",
            "A.subscript(_:)",
        ] {
            assert!(one(source, name).is_ok(), "{name}: {:?}", one(source, name));
        }
        for name in ["Outer.Inner.f", "A.init", "repeated"] {
            assert_eq!(one(source, name), Err(Missing::Ambiguous), "{name}");
        }
        let operators = "func +(lhs: Int, rhs: Int) -> Int { lhs }";
        assert!(one(operators, "+").is_ok());
        assert!(one(operators, "+(lhs:rhs:)").is_ok());
        assert_eq!(one(operators, "Int"), Err(Missing::NoSymbol));
        for name in ["P.missing", "missing"] {
            assert_eq!(one(source, name), Err(Missing::NoSymbol));
        }
        assert_eq!(
            sign(
                include_bytes!("../../tests/fixtures/fingerprint/swift/parse_error.swift"),
                &["keep"]
            ),
            vec![Err(Missing::ParseError)]
        );
    }

    #[test]
    fn changes_and_formatting() {
        let base = "@MainActor public struct A { var n = 1; func f() -> Int { return n } }\nfunc other() {}";
        let equivalent = "func other() { print(0) }\n/// doc\n@MainActor\npublic struct A { /* comment */ var n = 1; func f() -> Int { return n } }";
        assert_eq!(one(base, "A"), one(equivalent, "A"));
        for changed in [
            base.replace("return n", "return 2"),
            base.replace("@MainActor", "@OtherActor"),
            base.replace("public", "private"),
            base.replace("func f()", "func f(label: Int)"),
        ] {
            assert_ne!(one(base, "A"), one(&changed, "A"));
        }
        assert_ne!(
            one("func f() { _ = g\n(1) }", "f"),
            one("func f() { _ = g (1) }", "f")
        );
        assert_ne!(
            one("func f() { a(); b() }", "f"),
            one("func f() { a()\nb() }", "f")
        );
        assert_eq!(
            one("func f() { let a = [1, 2] }", "f"),
            one("func f() { let a = [1, 2,] }", "f")
        );
        assert_eq!(
            one("func f() { let a = [1, 2] }", "f"),
            one("func f() { let a = [1, 2, /* comment */] }", "f")
        );
        assert_ne!(
            one("func f() { #foo(1, 2) }", "f"),
            one("func f() { #foo(1, 2,) }", "f")
        );
        assert_ne!(
            one("@MyMacro(a, b) struct A {}", "A"),
            one("@MyMacro(a, b,) struct A {}", "A")
        );
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;
    const BASE: &[u8] = include_bytes!("../../tests/fixtures/fingerprint/swift/base.swift");

    #[test]
    fn fixture_edits_preserve_unaffected_declarations() {
        let names = ["Invoice", "Invoice.approve(name:)", "Invoice.Item.value()"];
        let base = sign(BASE, &names);
        for other in [
            &include_bytes!("../../tests/fixtures/fingerprint/swift/same_comments.swift")[..],
            &include_bytes!("../../tests/fixtures/fingerprint/swift/same_formatting.swift")[..],
            &include_bytes!("../../tests/fixtures/fingerprint/swift/same_other_declarations.swift")
                [..],
        ] {
            assert_eq!(base, sign(other, &names));
        }
    }

    #[test]
    fn newer_syntax_samples_parse() {
        let samples = [
            "let x = #stringify(1 + 2)",
            "@Observable class Model { var name = \"\" }",
            "func f(_ x: borrowing String, _ y: consuming String) { let z = consume y; print(x, z) }",
            "func f<each T>(_ args: repeat each T) { }",
            "enum MyError: Error { case bad }; func f() throws(MyError) { }",
            "struct S: ~Copyable { }",
            "func f(_ x: sending Int) { }",
            "nonisolated(unsafe) var x = 0",
            "let regex = /a+/",
            "let s = #\"raw \\#(1)\"#",
            "let x = if true { 1 } else { 2 }",
            "let x = switch 1 { case 1: \"one\"; default: \"other\" }",
            "struct Foo {\n#if DEBUG\nfunc foo() {}\n#else\nfunc bar() {}\n#endif\n}",
        ];
        for sample in samples {
            assert_ne!(
                sign(sample.as_bytes(), &["not_a_symbol"]),
                vec![Err(Missing::ParseError)],
                "{sample}"
            );
        }
    }
}

#[cfg(test)]
#[test]
fn line_break_golden_hashes() {
    let separated = sign(
        include_bytes!("../../tests/fixtures/fingerprint/swift/line_break.swift"),
        &["f"],
    );
    let call = sign(
        include_bytes!("../../tests/fixtures/fingerprint/swift/line_call.swift"),
        &["f"],
    );
    let actual = format!(
        "separated {}\ncall {}\n",
        separated[0].as_ref().unwrap(),
        call[0].as_ref().unwrap()
    );
    assert_eq!(
        actual,
        include_str!("../../tests/fixtures/fingerprint/swift/line_golden.txt")
    );
}
