//! Splits a Markdown file into heading sections, keeping byte offsets for patches.
//!
//! A section starts at a heading of the section level and runs to the next heading of the same or a
//! higher level. Its statement is the first top-level paragraph before its `yaml` block; its block is
//! the first top-level fenced block whose info string is `yaml`. Everything after the block is notes.

use crate::model::Span;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;

pub struct Document {
    pub frontmatter: Option<Frontmatter>,
    pub sections: Vec<Section>,
    /// Lines of headings deeper than the section level.
    pub subheadings: Vec<usize>,
}

pub struct Frontmatter {
    pub text: String,
    pub line: usize,
}

pub struct Section {
    pub heading: String,
    pub line: usize,
    pub statement: Option<String>,
    pub block: Option<Block>,
    pub span: Span,
}

pub struct Block {
    pub text: String,
    /// Line of the first content line, after the opening fence.
    pub line: usize,
}

pub fn scan(text: &str, level: HeadingLevel) -> Document {
    let lines = LineIndex::new(text);
    let mut doc = Document {
        frontmatter: None,
        sections: Vec::new(),
        subheadings: Vec::new(),
    };
    let mut current: Option<Section> = None;
    let mut depth = 0usize;
    let mut heading: Option<(HeadingLevel, Range<usize>, String)> = None;
    let mut frontmatter: Option<(usize, String)> = None;
    let mut block: Option<(Range<usize>, Option<Range<usize>>, String)> = None;

    let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                let top = depth == 0;
                depth += 1;
                if !top {
                    continue;
                }
                match tag {
                    Tag::MetadataBlock(_) => frontmatter = Some((range.start, String::new())),
                    Tag::Heading { level: l, .. } => heading = Some((l, range, String::new())),
                    Tag::Paragraph => {
                        if let Some(s) = current.as_mut()
                            && s.statement.is_none()
                            && s.block.is_none()
                        {
                            s.statement = Some(text[range].trim().to_string());
                        }
                    }
                    Tag::CodeBlock(CodeBlockKind::Fenced(info)) => {
                        let is_yaml = info.split_whitespace().next() == Some("yaml");
                        if is_yaml && current.as_ref().is_some_and(|s| s.block.is_none()) {
                            block = Some((range, None, String::new()));
                        }
                    }
                    _ => {}
                }
            }
            Event::End(end) => {
                depth -= 1;
                if depth != 0 {
                    continue;
                }
                match end {
                    TagEnd::MetadataBlock(_) => {
                        if let Some((start, text)) = frontmatter.take() {
                            doc.frontmatter = Some(Frontmatter {
                                text,
                                line: lines.line(start),
                            });
                        }
                    }
                    TagEnd::Heading(_) => {
                        let Some((l, range, title)) = heading.take() else {
                            continue;
                        };
                        if l > level {
                            doc.subheadings.push(lines.line(range.start));
                            continue;
                        }
                        if let Some(mut s) = current.take() {
                            s.span.end = range.start;
                            doc.sections.push(s);
                        }
                        if l == level {
                            current = Some(Section {
                                heading: title.trim().to_string(),
                                line: lines.line(range.start),
                                statement: None,
                                block: None,
                                span: Span {
                                    heading: range,
                                    block: None,
                                    end: text.len(),
                                },
                            });
                        }
                    }
                    TagEnd::CodeBlock => {
                        let (Some((range, content, body)), Some(s)) =
                            (block.take(), current.as_mut())
                        else {
                            continue;
                        };
                        let content = content.unwrap_or_else(|| {
                            let after_fence = text[range.clone()]
                                .find('\n')
                                .map_or(range.end, |i| range.start + i + 1);
                            after_fence..after_fence
                        });
                        s.block = Some(Block {
                            text: body,
                            line: lines.line(content.start),
                        });
                        s.span.block = Some(content);
                    }
                    _ => {}
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, _, title)) = heading.as_mut() {
                    title.push_str(&t);
                } else if let Some((_, body)) = frontmatter.as_mut() {
                    body.push_str(&t);
                } else if let Some((_, content, body)) = block.as_mut() {
                    body.push_str(&t);
                    *content = Some(match content.take() {
                        Some(c) => c.start..range.end,
                        None => range,
                    });
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, _, title)) = heading.as_mut() {
                    title.push(' ');
                }
            }
            _ => {}
        }
    }
    if let Some(s) = current {
        doc.sections.push(s);
    }
    doc
}

/// Maps byte offsets to 1-based line numbers.
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        LineIndex { starts }
    }

    pub fn line(&self, offset: usize) -> usize {
        self.starts.partition_point(|&s| s <= offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "---\nprefix: ATTR\n---\n\nIntro prose.\n\n# ATTR-001 First rule\n\nThe statement\nspans two lines.\n\n```yaml\nagreed: proposed\n```\n\nNotes.\n\n## A subheading\n\n# ATTR-002 Second\n\n- a list item, not a statement\n\n```yaml\n```\n";

    #[test]
    fn splits_sections_and_finds_statement_and_block() {
        let doc = scan(RULES, HeadingLevel::H1);
        let fm = doc.frontmatter.unwrap();
        assert_eq!(fm.text, "prefix: ATTR\n");
        assert_eq!(fm.line, 1);
        assert_eq!(doc.subheadings, vec![18]);

        let [first, second] = &doc.sections[..] else {
            panic!("expected two sections");
        };
        assert_eq!(first.heading, "ATTR-001 First rule");
        assert_eq!(first.line, 7);
        assert_eq!(
            first.statement.as_deref(),
            Some("The statement\nspans two lines.")
        );
        let block = first.block.as_ref().unwrap();
        assert_eq!(block.text, "agreed: proposed\n");
        assert_eq!(block.line, 13);
        let content = first.span.block.clone().unwrap();
        assert_eq!(&RULES[content], "agreed: proposed\n");
        assert_eq!(&RULES[first.span.end..][..10], "# ATTR-002");

        assert_eq!(second.statement, None);
        let block = second.block.as_ref().unwrap();
        assert_eq!(block.text, "");
        assert_eq!(second.span.block.clone().unwrap().len(), 0);
        assert_eq!(second.span.end, RULES.len());
    }

    #[test]
    fn a_paragraph_after_the_block_is_not_the_statement() {
        let doc = scan(
            "# X-001 T\n\n```yaml\na: 1\n```\n\nLate.\n",
            HeadingLevel::H1,
        );
        assert_eq!(doc.sections[0].statement, None);
    }

    #[test]
    fn higher_headings_end_a_section_without_starting_one() {
        let text = "# Group\n\n## Session\n\nDefinition.\n\n# Other group\n\nProse.\n\n## Commit\n\nOne.\n";
        let doc = scan(text, HeadingLevel::H2);
        let names: Vec<_> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(names, ["Session", "Commit"]);
        assert_eq!(&text[doc.sections[0].span.end..][..13], "# Other group");
    }

    #[test]
    fn only_yaml_fences_count_as_blocks() {
        let doc = scan(
            "# X-001 T\n\nS.\n\n```json\n{}\n```\n\n```yaml\nb: 2\n```\n",
            HeadingLevel::H1,
        );
        assert_eq!(doc.sections[0].block.as_ref().unwrap().text, "b: 2\n");
    }
}
