//! Decides which Sessions contributed to a commit.

use std::fmt;

/// Decides which Sessions contributed to a commit.
#[derive(Debug, Clone)]
pub struct Publisher<T> { threshold: usize, items: Vec<T> }

impl<T: Clone> Publisher<T> {
    /// Reports whether the evidence is enough.
    pub fn decide(
        &self,
        evidence: &str,
    ) -> bool {
        evidence
            .contains(
                "git commit",
            )
    }
}

impl<T> fmt::Display for Publisher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{} items", self.items.len()) }
}

/// Something that can decide.
pub trait Decide {
    fn decide(&self, evidence: &str) -> bool;
    /// Ready by default.
    fn ready(&self) -> bool { true }
}

#[inline]
pub fn score(a: usize,b: usize)->usize{
    let s="a  b";
    a+b+s.len()
}

pub enum Kind { Code, Test }

pub type Id
    = String;

mod inner {
    pub fn helper(x: u8) -> (u8,) {
        let pair = (
            x,
            x,
        );
        (pair.0,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn decides() { assert!(score(1, 2) > 0); }
}
