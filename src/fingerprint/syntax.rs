//! The hash input shared by the tree-sitter modes: the text of a node's tokens in source order, each
//! followed by a separator byte. Whitespace between tokens is not part of it, and neither are node
//! kind names, so a grammar update that renames kinds does not change hashes.

use sha2::{Digest, Sha256};
use tree_sitter::Node;

const SEPARATOR: u8 = 0;
/// Markers for structure that only dropped terminators or indentation show. None is source text.
const BLOCK_START: &[u8] = &[1];
const BLOCK_END: &[u8] = &[2];
const ITEM_END: &[u8] = &[3];

/// What to do with a node.
pub enum Visit {
    /// Leave it out of the hash input.
    Skip,
    /// Add its tokens.
    Enter,
    /// Add its tokens, marking the end of each named child: a statement or field list whose
    /// terminators are dropped.
    List,
    /// A `List` between block markers.
    Block,
}

/// Collects the hash input of a symbol.
pub struct Input {
    hasher: Sha256,
}

impl Input {
    pub fn new() -> Self {
        Input {
            hasher: Sha256::new(),
        }
    }

    /// Adds one token.
    pub fn token(&mut self, text: &[u8]) {
        self.hasher.update(text);
        self.hasher.update([SEPARATOR]);
    }

    /// Adds every token of `node`. `visit` sees each node before it is entered, so it can drop
    /// comments, docstrings or terminators whole, mark where each item of a list ends once its
    /// terminators are dropped, and mark where a block starts and ends when only indentation shows
    /// it. Text between children that no child covers is added too, trimmed, so no source text inside
    /// the node is lost to how the grammar splits it.
    pub fn node(&mut self, node: Node, source: &[u8], visit: &impl Fn(Node) -> Visit) {
        self.walk(node, source, visit, false);
    }

    fn walk(&mut self, node: Node, source: &[u8], visit: &impl Fn(Node) -> Visit, list: bool) {
        if node.child_count() == 0 {
            self.token(&source[node.byte_range()]);
            return;
        }
        let mut at = node.start_byte();
        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else { continue };
            self.gap(&source[at..child.start_byte()]);
            at = child.end_byte();
            match visit(child) {
                Visit::Skip => continue,
                Visit::Enter => self.walk(child, source, visit, false),
                Visit::List => self.walk(child, source, visit, true),
                Visit::Block => {
                    self.token(BLOCK_START);
                    self.walk(child, source, visit, true);
                    self.token(BLOCK_END);
                }
            }
            if list && child.is_named() {
                self.token(ITEM_END);
            }
        }
        self.gap(&source[at..node.end_byte()]);
    }

    fn gap(&mut self, text: &[u8]) {
        let text = text.trim_ascii();
        if !text.is_empty() {
            self.token(text);
        }
    }

    pub fn finish(self) -> String {
        self.hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
