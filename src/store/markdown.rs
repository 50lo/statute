//! Splits a Markdown file into heading sections, keeping byte offsets for patches.
//!
//! A section starts at a heading of the section level and runs to the next heading of the same or a
//! higher level. Its statement is the first top-level paragraph before its `yaml` block; its block is
//! the first top-level fenced block whose info string is `yaml`. In a rule, everything after the block
//! is notes; in a term, everything but the block is the definition.

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
    /// The whole `yaml` block, fences included.
    pub fence: Option<Range<usize>>,
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
                                fence: None,
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
                        s.fence = Some(range.clone());
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

/// The text with the value of a top-level `key` inside a `yaml` block replaced, or `None` when the
/// block has no such key. Top-level keys sit at the indentation of the block's first content line, and
/// the key may be quoted. A value that continues on more-indented lines, such as a folded scalar, is
/// replaced whole. An inline value keeps its trailing comment, at the same column where there is room.
/// The caller re-parses the result: this reads YAML's layout, not its full grammar.
pub fn set_field(text: &str, block: Range<usize>, key: &str, value: &str) -> Option<String> {
    let content = |l: &'_ str| -> usize { l.trim_end_matches(['\n', '\r']).len() };
    let indent = |l: &str| l.len() - l.trim_start_matches(' ').len();
    let mut lines = Vec::new();
    let mut at = block.start;
    for line in text[block].split_inclusive('\n') {
        lines.push((at, &line[..content(line)]));
        at += line.len();
    }
    let top = lines
        .iter()
        .map(|(_, l)| *l)
        .find(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with("---")
        })
        .map(indent)?;
    let keys = [key.to_string(), format!("\"{key}\""), format!("'{key}'")];

    for (i, &(start, line)) in lines.iter().enumerate() {
        if indent(line) != top {
            continue;
        }
        let Some(after_colon) = keys
            .iter()
            .find_map(|k| line[top..].strip_prefix(k.as_str()))
            .and_then(|r| r.trim_start_matches([' ', '\t']).strip_prefix(':'))
            .filter(|r| r.is_empty() || r.starts_with([' ', '\t']))
        else {
            continue;
        };
        let colon_end = start + line.len() - after_colon.len();

        // A value that continues on later lines: replace from the colon to the end of its last line.
        let last = lines[i + 1..]
            .iter()
            .filter(|(_, l)| !l.trim().is_empty())
            .take_while(|(_, l)| indent(l) > top)
            .last();
        if let Some(&(last_start, last_line)) = last {
            let end = last_start + last_line.len();
            return Some(format!("{} {value}{}", &text[..colon_end], &text[end..]));
        }

        let value_start = colon_end + after_colon.len() - after_colon.trim_start().len();
        let inline = &text[value_start..start + line.len()];
        let old_len = match inline.chars().next() {
            Some(q @ ('"' | '\'')) => inline[1..].find(q).map_or(inline.len(), |e| e + 2),
            Some('#') | None => 0,
            Some(_) => {
                let end = [" #", "\t#"]
                    .iter()
                    .filter_map(|c| inline.find(c))
                    .min()
                    .unwrap_or(inline.len());
                inline[..end].trim_end().len()
            }
        };
        let after = &inline[old_len..];
        let gap = after.len() - after.trim_start_matches([' ', '\t']).len();
        let (end, padding) = if after[gap..].starts_with('#') {
            let width = (old_len + gap).saturating_sub(value.len()).max(1);
            (value_start + old_len + gap, " ".repeat(width))
        } else {
            (value_start + old_len, String::new())
        };
        let space = if value_start == colon_end { " " } else { "" };
        return Some(format!(
            "{}{space}{value}{padding}{}",
            &text[..value_start],
            &text[end..]
        ));
    }
    None
}

/// The text with `key: value` added to the mapping in a `yaml` block: as the first entry of a flow
/// mapping (`{…}`), or as a line before the first key of a block mapping, at that key's indentation.
/// Comments and a `---` marker before the mapping stay in front of it. The caller re-parses the result.
pub fn insert_field(text: &str, block: Range<usize>, key: &str, value: &str) -> String {
    let mut at = block.start;
    let mut first = None;
    for line in text[block.clone()].split_inclusive('\n') {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') && t != "---" {
            first = Some(line);
            break;
        }
        at += line.len();
    }
    let Some(line) = first else {
        return format!("{}{key}: {value}\n{}", &text[..at], &text[at..]);
    };
    let indent = line.len() - line.trim_start_matches(' ').len();
    if line[indent..].starts_with('{') {
        let open = at + indent + 1;
        let empty = text[open..].trim_start().starts_with('}');
        let sep = if empty { "" } else { ", " };
        return format!(
            "{}{key}: {value}{sep}{}",
            &text[..open],
            text[open..].trim_start_matches(' ')
        );
    }
    format!(
        "{}{}{key}: {value}\n{}",
        &text[..at],
        " ".repeat(indent),
        &text[at..]
    )
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
        let fence = first.fence.clone().unwrap();
        assert_eq!(&RULES[fence], "```yaml\nagreed: proposed\n```");
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
    fn set_field_replaces_one_value_and_keeps_the_comment_column() {
        let text = "# X-001 T\n\nS.\n\n```yaml\nagreed: proposed\nbuilt: planned            # planned | built\nscope: built\n```\n";
        let block = scan(text, HeadingLevel::H1).sections[0]
            .span
            .block
            .clone()
            .unwrap();
        let out = set_field(text, block.clone(), "built", "built").unwrap();
        assert_eq!(
            out,
            text.replace("built: planned            #", "built: built              #")
        );
        let out = set_field(text, block.clone(), "agreed", "agreed").unwrap();
        assert_eq!(out, text.replace("agreed: proposed\n", "agreed: agreed\n"));
        assert_eq!(set_field(text, block, "terms", "[]"), None);

        // Layouts YAML allows: indented keys, quoted keys and values, values on later lines.
        let cases = [
            (
                "  agreed: proposed\n  built: planned\n",
                "  agreed: proposed\n  built: built\n",
            ),
            ("\"built\": planned\n", "\"built\": built\n"),
            ("'built' : 'planned'  # c\n", "'built' : built      # c\n"),
            (
                "built: >-\n  planned\nterms: []\n",
                "built: built\nterms: []\n",
            ),
            (
                "built:\n\n    planned\n\nterms: []\n",
                "built: built\n\nterms: []\n",
            ),
            (
                "# note\nprotects:\n  - built: planned\nbuilt: planned\n",
                "# note\nprotects:\n  - built: planned\nbuilt: built\n",
            ),
            ("builtin: x\nbuilt:planned\n", "builtin: x\nbuilt:planned\n"),
        ];
        for (yaml, expected) in cases {
            let text = format!("```yaml\n{yaml}```\n");
            let out = set_field(&text, 8..8 + yaml.len(), "built", "built");
            let expected = (expected != yaml).then(|| format!("```yaml\n{expected}```\n"));
            assert_eq!(out, expected, "{yaml:?}");
        }

        let tight = "```yaml\nbuilt: planned # c\n```\n";
        let block = 8..tight.len() - 4;
        assert_eq!(
            set_field(tight, block, "built", "something").unwrap(),
            "```yaml\nbuilt: something # c\n```\n"
        );
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
