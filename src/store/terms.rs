use super::markdown::{self, Block, Section};
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
        let definition = definition(text, &section);
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
            definition,
            meta,
            span: section.span,
        });
    }
    ParsedTerms { terms, problems }
}

/// Everything in the section after the heading except the `yaml` block, with blank lines around the
/// parts removed, or `None` when nothing is left. Terms have no notes: a paragraph, list, table or other
/// code block after the `yaml` block is part of the definition.
fn definition(text: &str, section: &Section) -> Option<String> {
    let body = section.span.heading.end..section.span.end;
    let parts = match &section.fence {
        Some(fence) => [&text[body.start..fence.start], &text[fence.end..body.end]],
        None => [&text[body], ""],
    };
    let parts: Vec<&str> = parts
        .into_iter()
        .map(trim_blank_lines)
        .filter(|p| !p.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

/// The text without leading blank lines and trailing whitespace. The first line keeps its indentation.
fn trim_blank_lines(text: &str) -> &str {
    let text = text.trim_end();
    let first = text.len() - text.trim_start().len();
    let line_start = text[..first].rfind('\n').map_or(0, |i| i + 1);
    &text[line_start..]
}

/// `terms.md` with `agreed:` set on one term, adding the field, or a `yaml` block after the term's
/// text, when the term has none. `None` when the result would change anything else the term says.
pub fn set_agreed(text: &str, term: &Term, agreed: TermAgreed) -> Option<String> {
    let value = match agreed {
        TermAgreed::Proposed => "proposed",
        TermAgreed::Agreed => "agreed",
        TermAgreed::Retired => "retired",
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
    fn the_definition_is_everything_but_the_heading_and_the_yaml_block() {
        let text = "## A\n\nFirst\nparagraph.\n\n```yaml\ncode: [A]\n```\n\nLater.\n\n- an item\n\n| x | y |\n| - | - |\n\n```text\nkept\n```\n\n# Group\n\n## B\n\n```yaml\ncode: [B]\n```\n\nOnly after.\n\n## C\n\n```yaml\n```\n";
        let parsed = parse(text);
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        let definitions: Vec<_> = parsed
            .terms
            .iter()
            .map(|t| t.definition.as_deref())
            .collect();
        assert_eq!(
            definitions,
            [
                Some(
                    "First\nparagraph.\n\nLater.\n\n- an item\n\n| x | y |\n| - | - |\n\n```text\nkept\n```"
                ),
                Some("Only after."),
                None,
            ]
        );
    }

    #[test]
    fn set_agreed_keeps_a_whole_definition_without_a_block() {
        let text = "## A\n\nD.\n\n- an item\n\n## B\n\nE.\n";
        let patched = set_agreed(text, &parse(text).terms[0], TermAgreed::Proposed).unwrap();
        assert_eq!(
            patched,
            "## A\n\nD.\n\n- an item\n\n```yaml\nagreed: proposed\n```\n\n## B\n\nE.\n"
        );
        assert_eq!(
            parse(&patched).terms[0].definition.as_deref(),
            Some("D.\n\n- an item")
        );
    }

    #[test]
    fn set_agreed_retires_a_term_and_keeps_its_whole_section() {
        let text = "## A\n\nD.\n\n```yaml\nagreed: agreed   # proposed | agreed | retired\ncode: [A]\naliases: [a]\n```\n\nLater.\n\n## B\n\nE.\n";
        let patched = set_agreed(text, &parse(text).terms[0], TermAgreed::Retired).unwrap();
        assert_eq!(
            patched,
            text.replace("agreed: agreed   #", "agreed: retired  #")
        );
        let retired = &parse(&patched).terms[0];
        assert!(retired.is_retired());
        assert_eq!(retired.definition.as_deref(), Some("D.\n\nLater."));
    }

    #[test]
    fn schema_errors_name_the_term() {
        let parsed = parse("## Session\n\nD.\n\n```yaml\nagreed: dropped\n```\n");
        assert_eq!(parsed.problems[0].code, "term.schema");
        assert_eq!(parsed.problems[0].term.as_deref(), Some("Session"));
        assert!(parsed.terms[0].meta.is_none());
    }
}
