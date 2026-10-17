//! Decides which Sessions contributed to a commit.

use std::collections::BTreeMap;
use std::fmt;

const LIMIT: usize = 3;

/// Decides which Sessions contributed to a commit.
#[derive(Debug, Clone)]
pub struct Publisher<T> {
    threshold: usize,
    items: Vec<T>,
}

impl<T: Clone> Publisher<T> {
    pub fn new() -> Self {
        Publisher { threshold: LIMIT, items: Vec::new() }
    }

    /// Reports whether the evidence is enough.
    pub fn decide(&self, evidence: &str) -> bool {
        evidence.contains("git commit") // the only evidence that counts
    }
}

impl<T> fmt::Display for Publisher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} items", self.items.len())
    }
}

impl<T> Drop for Publisher<T> {
    fn drop(&mut self) {}
}

/// Something that can decide.
pub trait Decide {
    fn decide(&self, evidence: &str) -> bool;

    /// Ready by default.
    fn ready(&self) -> bool {
        true
    }
}

#[inline]
pub fn score(a: usize, b: usize) -> usize {
    let s = "a  b"; // whitespace inside a string is content
    a + b + s.len()
}

pub fn index() -> BTreeMap<String, usize> {
    BTreeMap::new()
}

pub enum Kind {
    Code,
    Test,
}

pub type Id = String;

mod inner {
    /* a block comment */
    pub fn helper(x: u8) -> (u8,) {
        let pair = (x, x);
        (pair.0,)
    }

    pub fn other() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decides() {
        assert!(score(1, 2) > 1);
    }

    #[test]
    fn indexes() {
        assert!(index().is_empty());
    }
}
