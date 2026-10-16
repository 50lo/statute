//! `check --all`: term `code` and `not` names that no longer appear anywhere in the repository.

use crate::model::{Finding, Model, paths};
use aho_corasick::AhoCorasick;

/// Searches files for every term's `code` and `not` names as whole identifiers.
pub struct NameSearch {
    names: Vec<String>,
    automaton: Option<AhoCorasick>,
    found: Vec<bool>,
}

impl NameSearch {
    pub fn new(model: &Model) -> Self {
        let mut names: Vec<String> = model
            .terms
            .iter()
            .filter_map(|t| t.meta.as_ref())
            .flat_map(|m| m.code.iter().chain(m.not.iter().map(|n| &n.name)))
            .filter(|n| !n.is_empty())
            .cloned()
            .collect();
        names.sort();
        names.dedup();
        let automaton =
            (!names.is_empty()).then(|| AhoCorasick::new(&names).expect("names are plain strings"));
        let found = vec![false; names.len()];
        NameSearch {
            names,
            automaton,
            found,
        }
    }

    /// Whether a file is searched: Statute's own files and Markdown are not, because docs lag behind
    /// code and would hide a renamed name.
    pub fn searches(rel: &str) -> bool {
        rel != paths::CONFIG
            && rel != paths::LOCK
            && !rel.starts_with("statute/")
            && !rel.ends_with(".md")
    }

    pub fn scan(&mut self, bytes: &[u8]) {
        let Some(automaton) = &self.automaton else {
            return;
        };
        let is_ident = |b: Option<&u8>| b.is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
        for m in automaton.find_overlapping_iter(bytes) {
            let before = m.start().checked_sub(1).and_then(|i| bytes.get(i));
            if !is_ident(before) && !is_ident(bytes.get(m.end())) {
                self.found[m.pattern().as_usize()] = true;
            }
        }
    }

    /// A warning for each name of each term that was not found.
    pub fn findings(&self, model: &Model) -> Vec<Finding> {
        let missing = |name: &String| {
            self.names
                .iter()
                .position(|n| n == name)
                .is_some_and(|i| !self.found[i])
        };
        let mut out = Vec::new();
        for term in &model.terms {
            let Some(meta) = &term.meta else { continue };
            let names = meta
                .code
                .iter()
                .map(|n| ("code", n))
                .chain(meta.not.iter().map(|n| ("not", &n.name)));
            for (field, name) in names.filter(|(_, n)| missing(n)) {
                out.push(
                    Finding::warning(
                        "term.name_not_found",
                        format!(
                            "`{name}` in `{field}` of term {} does not appear as an identifier in any file outside Statute's files and Markdown",
                            term.name
                        ),
                    )
                    .term(&term.name)
                    .at(paths::TERMS, Some(term.line)),
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NotName, Span, Term, TermMeta};

    fn model(code: &[&str], not: &[&str]) -> Model {
        Model {
            terms: vec![Term {
                name: "Session".into(),
                line: 3,
                definition: None,
                meta: Some(TermMeta {
                    code: code.iter().map(|s| s.to_string()).collect(),
                    not: not
                        .iter()
                        .map(|n| NotName {
                            name: n.to_string(),
                            means: "m".into(),
                        })
                        .collect(),
                    ..TermMeta::default()
                }),
                span: Span {
                    heading: 0..0,
                    block: None,
                    end: 0,
                },
            }],
            ..Model::default()
        }
    }

    #[test]
    fn names_count_only_as_whole_identifiers() {
        let model = model(&["Session", "SessionID", "commit_log"], &["sessions"]);
        let mut search = NameSearch::new(&model);
        search.scan(b"type SessionID string\nvar x = sessions_table\n");
        search.scan(b"commit_log(); // MySession");
        let missing: Vec<_> = search
            .findings(&model)
            .iter()
            .map(|f| f.message.split('`').nth(1).unwrap().to_string())
            .collect();
        assert_eq!(missing, ["Session", "sessions"]);
    }

    #[test]
    fn statute_files_and_markdown_are_not_searched() {
        assert!(NameSearch::searches("src/session.go"));
        assert!(NameSearch::searches("statute.go"));
        for skipped in [
            "statute.yml",
            "statute.lock",
            "statute/terms.md",
            "README.md",
        ] {
            assert!(!NameSearch::searches(skipped), "{skipped}");
        }
    }
}
