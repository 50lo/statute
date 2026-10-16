use super::markdown::{self, Block};
use super::yaml_finding;
use crate::model::{Finding, Term, TermMeta, paths};
use pulldown_cmark::HeadingLevel;

pub struct ParsedTerms {
    pub terms: Vec<Term>,
    pub problems: Vec<Finding>,
}

/// Parses `statute/terms.md`: `#` headings group terms, each `##` heading is a term.
pub fn parse(text: &str) -> ParsedTerms {
    let path = paths::TERMS;
    let doc = markdown::scan(text, HeadingLevel::H2);
    let mut problems = Vec::new();

    for line in &doc.subheadings {
        problems.push(
            Finding::error(
                "term.subheading",
                "terms have no subheadings: `#` headings group terms and each `##` heading is a term",
            )
            .at(path, Some(*line)),
        );
    }

    let mut terms = Vec::new();
    for section in doc.sections {
        let meta = match &section.block {
            None => Some(TermMeta::default()),
            Some(Block { text, line }) => match yaml_serde::from_str::<Option<TermMeta>>(text) {
                Ok(meta) => Some(meta.unwrap_or_default()),
                Err(e) => {
                    problems.push(
                        yaml_finding("term.schema", "term block", &e, path, *line)
                            .term(&section.heading),
                    );
                    None
                }
            },
        };
        terms.push(Term {
            name: section.heading,
            line: section.line,
            definition: section.statement,
            meta,
            span: section.span,
        });
    }
    ParsedTerms { terms, problems }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TermAgreed;

    #[test]
    fn parses_terms_under_groups() {
        let text = "# Sessions\n\n## Session\n\nOne conversation.\n\n```yaml\nagreed: agreed\ncode: [Session]\nnot:\n  - {name: sessions, means: sign-in sessions}\n```\n\n## Commit\n\nA git commit.\n";
        let parsed = parse(text);
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let [session, commit] = &parsed.terms[..] else {
            panic!();
        };
        let meta = session.meta.as_ref().unwrap();
        assert_eq!(meta.agreed, Some(TermAgreed::Agreed));
        assert_eq!(meta.not[0].name, "sessions");
        assert_eq!(commit.definition.as_deref(), Some("A git commit."));
        assert_eq!(commit.meta, Some(TermMeta::default()));
    }

    #[test]
    fn schema_errors_name_the_term() {
        let parsed = parse("## Session\n\nD.\n\n```yaml\nagreed: retired\n```\n");
        assert_eq!(parsed.problems[0].code, "term.schema");
        assert_eq!(parsed.problems[0].term.as_deref(), Some("Session"));
        assert!(parsed.terms[0].meta.is_none());
    }
}
