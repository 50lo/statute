//! Decides which Sessions contributed to a commit, and why.
//! A second line of module docs.

use std::fmt;

// A plain comment above the docs.
/// Decides which Sessions contributed to a commit.
/// It keeps every item it has seen.
#[derive(Debug, Clone)]
pub struct Publisher<T> {
    /// How many items are enough.
    threshold: usize,
    items: Vec<T>, // the items
}

/** Methods. */
impl<T: Clone> Publisher<T> {
    /// Reports whether the evidence is enough to decide.
    pub fn decide(&self, evidence: &str) -> bool {
        // Only one kind of evidence counts.
        evidence.contains("git commit")
    }
}

impl<T> fmt::Display for Publisher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} items", /* count */ self.items.len())
    }
}

/// Something that can decide, given evidence.
pub trait Decide {
    /// Decides.
    fn decide(&self, evidence: &str) -> bool;

    fn ready(&self) -> bool {
        true // always
    }
}

#[inline] // keep it fast
pub fn score(a: usize, b: usize) -> usize {
    let s = "a  b";
    a + b + s.len()
}

/// The role of a link.
pub enum Kind {
    /// Production code.
    Code,
    Test,
}

/// An identifier.
pub type Id = String;

/// Helpers.
mod inner {
    //! Inner docs.
    pub fn helper(x: u8) -> (u8,) {
        let pair = (x, x); // a pair
        (pair.0,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test.
    #[test]
    // between the attribute and the function
    fn decides() {
        assert!(score(1, 2) > 0);
    }
}
