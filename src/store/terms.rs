use super::markdown::{self, Block};
use super::yaml_finding;
use crate::model::{Finding, Term, TermAgreed, TermMeta, paths};
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

/// `terms.md` with `agreed:` set on one term, adding the field, or a `yaml` block after the term's
/// text, when the term has none. `None` when the result would change anything else the term says.
pub fn set_agreed(text: &str, term: &Term, agreed: TermAgreed) -> Option<String> {
    let value = match agreed {
        TermAgreed::Proposed => "proposed",
        TermAgreed::Agreed => "agreed",
    };
    let expected = TermMeta {
        agreed: Some(agreed),
        ..term.meta.clone()?
    };
    let patched = match term.span.block.clone() {
        Some(block) => super::set_field(text, block.clone(), "agreed", value)
            .unwrap_or_else(|| super::insert_field(text, block, "agreed", value)),
        None => {
            let (before, after) = text.split_at(term.span.end);
            let gap = if before.ends_with("\n\n") {
                ""
            } else if before.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let next = if after.is_empty() { "" } else { "\n" };
            format!("{before}{gap}```yaml\nagreed: {value}\n```\n{next}{after}")
        }
    };
    let reparsed = parse(&patched);
    let same = reparsed
        .terms
        .iter()
        .find(|t| t.span.heading == term.span.heading)
        .is_some_and(|t| t.meta.as_ref() == Some(&expected) && t.definition == term.definition)
        && reparsed.problems.len() <= parse(text).problems.len();
    same.then_some(patched)
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
    fn set_agreed_replaces_inserts_or_adds_the_field() {
        let set = |text: &str, i: usize| {
            set_agreed(text, &parse(text).terms[i], TermAgreed::Proposed).unwrap()
        };
        let with = "## A\n\nD.\n\n```yaml\nagreed: agreed   # note\ncode: [A]\n```\n";
        assert_eq!(
            set(with, 0),
            with.replace("agreed: agreed   #", "agreed: proposed #")
        );
        let without = "## A\n\nD.\n\n```yaml\n  code: [A]\n```\n";
        assert_eq!(
            set(without, 0),
            "## A\n\nD.\n\n```yaml\n  agreed: proposed\n  code: [A]\n```\n"
        );
        let no_block = "# G\n\n## A\n\nD.\n\n## B\n\nE.";
        assert_eq!(
            set(no_block, 0),
            "# G\n\n## A\n\nD.\n\n```yaml\nagreed: proposed\n```\n\n## B\n\nE."
        );
        assert_eq!(
            set(no_block, 1),
            "# G\n\n## A\n\nD.\n\n## B\n\nE.\n\n```yaml\nagreed: proposed\n```\n"
        );
    }

    #[test]
    fn set_agreed_inserts_at_the_mapping_not_before_comments_or_markers() {
        let set = |block: &str| {
            let text = format!("## A\n\nD.\n\n```yaml\n{block}```\n");
            set_agreed(&text, &parse(&text).terms[0], TermAgreed::Proposed).map(|t| {
                t.replace("## A\n\nD.\n\n```yaml\n", "")
                    .replace("```\n", "")
            })
        };
        assert_eq!(
            set("# identifier names\n  code: [A]\n").as_deref(),
            Some("# identifier names\n  agreed: proposed\n  code: [A]\n")
        );
        assert_eq!(
            set("---\ncode: [A]\n").as_deref(),
            Some("---\nagreed: proposed\ncode: [A]\n")
        );
        assert_eq!(
            set("{code: [A]}\n").as_deref(),
            Some("{agreed: proposed, code: [A]}\n")
        );
        assert_eq!(set("{ }\n").as_deref(), Some("{agreed: proposed}\n"));
    }

    #[test]
    fn schema_errors_name_the_term() {
        let parsed = parse("## Session\n\nD.\n\n```yaml\nagreed: retired\n```\n");
        assert_eq!(parsed.problems[0].code, "term.schema");
        assert_eq!(parsed.problems[0].term.as_deref(), Some("Session"));
        assert!(parsed.terms[0].meta.is_none());
    }
}
