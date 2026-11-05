use super::markdown::{self, Block};
use super::yaml_finding;
use crate::model::{Agreed, Built, Finding, Rule, RuleFile, RuleMeta};
use pulldown_cmark::HeadingLevel;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    prefix: String,
}

pub struct ParsedRuleFile {
    pub file: RuleFile,
    pub rules: Vec<Rule>,
    pub problems: Vec<Finding>,
}

/// Parses `statute/rules/<area>.md`.
pub fn parse(path: &str, area: &str, text: &str) -> ParsedRuleFile {
    let doc = markdown::scan(text, HeadingLevel::H1);
    let mut problems = Vec::new();

    let prefix = match &doc.frontmatter {
        None => {
            problems.push(
                Finding::error(
                    "rule.frontmatter",
                    "rule file has no frontmatter; start it with `---`, `prefix: <PREFIX>`, `---`",
                )
                .at(path, Some(1)),
            );
            None
        }
        Some(fm) => match yaml_serde::from_str::<Frontmatter>(&fm.text) {
            Ok(f) => Some(f.prefix),
            Err(e) => {
                problems.push(yaml_finding(
                    "rule.frontmatter",
                    "frontmatter",
                    &e,
                    path,
                    fm.line + 1,
                ));
                None
            }
        },
    };

    for line in &doc.subheadings {
        problems.push(
            Finding::error(
                "rule.subheading",
                "rule files have no subheadings: every `#` heading is a rule, and other headings are not allowed",
            )
            .at(path, Some(*line)),
        );
    }

    let mut rules = Vec::new();
    for section in doc.sections {
        let (id, title) = match section.heading.split_once(char::is_whitespace) {
            Some((id, title)) => (id.to_string(), title.trim().to_string()),
            None => (section.heading.clone(), String::new()),
        };
        let meta = match &section.block {
            None => {
                problems.push(
                    Finding::error("rule.block_missing", "rule has no `yaml` metadata block")
                        .rule(&id)
                        .at(path, Some(section.line)),
                );
                None
            }
            Some(Block { text, line }) => match yaml_serde::from_str::<RuleMeta>(text) {
                Ok(meta) => Some(meta),
                Err(e) => {
                    problems
                        .push(yaml_finding("rule.schema", "rule block", &e, path, *line).rule(&id));
                    None
                }
            },
        };
        rules.push(Rule {
            id,
            title,
            file: path.to_string(),
            area: area.to_string(),
            line: section.line,
            statement: section.statement,
            meta,
            span: section.span,
        });
    }

    ParsedRuleFile {
        file: RuleFile {
            path: path.to_string(),
            area: area.to_string(),
            prefix,
        },
        rules,
        problems,
    }
}

/// The section `rule new` appends.
pub fn skeleton(id: &str, title: &str) -> String {
    format!(
        "# {id} {title}

State the obligation in one or two sentences: what must or must not happen, to what, and under which conditions.

```yaml
agreed: proposed          # proposed | agreed | retired
built: planned            # planned | built
scope: \"\"
terms: []
exceptions: []
protects: []
boundary: []
```
"
    )
}

/// The rule file with `built:` set on the rule with `rule`'s ID, or `None` when the block cannot be
/// patched without changing anything else the block says. The rule is found in `text` by its ID, so
/// earlier patches to the same file do not matter.
pub fn set_built(text: &str, rule: &Rule, built: Built) -> Option<String> {
    let value = match built {
        Built::Planned => "planned",
        Built::Built => "built",
    };
    patch_meta(text, rule, "built", value, |meta| RuleMeta {
        built,
        ..meta
    })
}

/// The rule file with `agreed:` set on the rule with `rule`'s ID, or `None` as for `set_built`.
pub fn set_agreed(text: &str, rule: &Rule, agreed: Agreed) -> Option<String> {
    let value = match agreed {
        Agreed::Proposed => "proposed",
        Agreed::Agreed => "agreed",
        Agreed::Retired => "retired",
    };
    patch_meta(text, rule, "agreed", value, |meta| RuleMeta {
        agreed,
        ..meta
    })
}

fn patch_meta(
    text: &str,
    rule: &Rule,
    key: &str,
    value: &str,
    expected: impl FnOnce(RuleMeta) -> RuleMeta,
) -> Option<String> {
    let parsed = parse(&rule.file, &rule.area, text);
    let current = parsed.rules.iter().find(|r| r.id == rule.id)?;
    let expected = expected(current.meta.clone()?);
    let patched = super::set_field(text, current.span.block.clone()?, key, value)?;
    let reparsed = parse(&rule.file, &rule.area, &patched);
    let same = reparsed
        .rules
        .iter()
        .find(|r| r.span.heading == current.span.heading)
        .is_some_and(|r| r.meta.as_ref() == Some(&expected) && r.statement == current.statement);
    same.then_some(patched)
}

/// Appends a section to the end of a Markdown file, separated by one blank line. Existing bytes are kept.
pub fn append_section(text: &str, section: &str) -> String {
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(section);
    out
}

/// Adds the rule-file prefix to frontmatter, or creates frontmatter when the file has none.
/// Existing content and frontmatter keys are preserved byte for byte.
pub fn add_prefix(text: &str, prefix: &str) -> Option<String> {
    let doc = markdown::scan(text, HeadingLevel::H1);
    if doc.frontmatter.is_none() {
        // A leading delimiter without a matching close is malformed frontmatter, not a file
        // without frontmatter. Do not prepend a second block that could reinterpret its contents.
        if text
            .lines()
            .next()
            .is_some_and(|line| line.trim_end() == "---")
        {
            return None;
        }
        return Some(format!("---\nprefix: {prefix}\n---\n\n{text}"));
    }

    let mut offset = 0;
    let mut delimiter_count = 0;
    let mut close = None;
    for line in text.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if content == "---" {
            delimiter_count += 1;
            if delimiter_count == 2 {
                close = Some(offset);
                break;
            }
        }
        offset += line.len();
    }
    let close = close?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = String::with_capacity(text.len() + prefix.len() + 10);
    out.push_str(&text[..close]);
    out.push_str("prefix: ");
    out.push_str(prefix);
    out.push_str(newline);
    out.push_str(&text[close..]);
    Some(out)
}

/// Whether the parsed frontmatter already contains a `prefix` key, including an invalid value.
pub fn has_prefix_key(text: &str) -> bool {
    let doc = markdown::scan(text, HeadingLevel::H1);
    let Some(frontmatter) = doc.frontmatter else {
        return false;
    };
    frontmatter.text.lines().any(|line| {
        line.split_once(':')
            .is_some_and(|(key, _)| key.trim().trim_matches(['\'', '"']) == "prefix")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Agreed, Built};

    const FILE: &str = "statute/rules/billing.md";

    #[test]
    fn parses_prefix_rules_and_metadata() {
        let text = format!(
            "---\nprefix: BILL\n---\n\n{}",
            skeleton("BILL-001", "Large refunds")
        );
        let parsed = parse(FILE, "billing", &text);
        assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
        assert_eq!(parsed.file.prefix.as_deref(), Some("BILL"));
        let rule = &parsed.rules[0];
        assert_eq!(
            (rule.id.as_str(), rule.title.as_str()),
            ("BILL-001", "Large refunds")
        );
        let meta = rule.meta.as_ref().unwrap();
        assert_eq!(
            (meta.agreed, meta.built),
            (Agreed::Proposed, Built::Planned)
        );
        assert!(
            rule.statement
                .as_ref()
                .unwrap()
                .starts_with("State the obligation")
        );
    }

    #[test]
    fn unknown_keys_are_schema_errors_with_the_line() {
        let text = "---\nprefix: BILL\n---\n\n# BILL-001 T\n\nS.\n\n```yaml\nagreed: proposed\nbuilt: planned\nprotect: []\n```\n";
        let parsed = parse(FILE, "billing", text);
        let [problem] = &parsed.problems[..] else {
            panic!("{:?}", parsed.problems);
        };
        assert_eq!(problem.code, "rule.schema");
        assert_eq!(problem.rule.as_deref(), Some("BILL-001"));
        assert_eq!(problem.line, Some(12));
        assert!(problem.message.contains("protect"), "{}", problem.message);
        assert!(parsed.rules[0].meta.is_none());
    }

    #[test]
    fn missing_frontmatter_and_block_are_reported() {
        let parsed = parse(FILE, "billing", "# BILL-001 T\n\nS.\n");
        let codes: Vec<_> = parsed.problems.iter().map(|p| p.code).collect();
        assert_eq!(codes, ["rule.frontmatter", "rule.block_missing"]);
        assert_eq!(parsed.file.prefix, None);
    }

    #[test]
    fn set_built_patches_only_the_built_field() {
        let text = "---\nprefix: BILL\n---\n\n# BILL-001 T\n\nS.\n\n```yaml\n  agreed: proposed\n  built: >-\n    planned\n  terms: []\n```\n";
        let rule = &parse(FILE, "billing", text).rules[0];
        let out = set_built(text, rule, Built::Built).unwrap();
        assert_eq!(out, text.replace("built: >-\n    planned", "built: built"));
        // A layout the patch cannot read is refused, not mangled.
        let flow = "---\nprefix: BILL\n---\n\n# BILL-001 T\n\nS.\n\n```yaml\n{agreed: proposed, built: planned}\n```\n";
        let rule = &parse(FILE, "billing", flow).rules[0];
        assert!(rule.meta.is_some());
        assert_eq!(set_built(flow, rule, Built::Built), None);
    }

    #[test]
    fn set_agreed_patches_only_the_agreed_field() {
        let text = format!("---\nprefix: BILL\n---\n\n{}", skeleton("BILL-001", "T"));
        let rule = &parse(FILE, "billing", &text).rules[0];
        let out = set_agreed(&text, rule, Agreed::Agreed).unwrap();
        assert_eq!(
            out,
            text.replace("agreed: proposed          #", "agreed: agreed            #")
        );
    }

    #[test]
    fn set_agreed_finds_the_rule_after_earlier_patches_moved_it() {
        let text = format!(
            "---\nprefix: BILL\n---\n\n{}\n{}",
            skeleton("BILL-001", "T"),
            skeleton("BILL-002", "U")
        );
        let parsed = parse(FILE, "billing", &text);
        let [first, second] = &parsed.rules[..] else {
            panic!();
        };
        // Shortening the first rule's block moves the second rule's offsets.
        let once = set_agreed(&text, first, Agreed::Retired).unwrap();
        let twice = set_built(&once, second, Built::Built).unwrap();
        let rules = parse(FILE, "billing", &twice).rules;
        assert_eq!(rules[0].agreed(), Some(Agreed::Retired));
        assert_eq!(rules[1].meta.as_ref().map(|m| m.built), Some(Built::Built));
        assert_eq!(rules[1].agreed(), Some(Agreed::Proposed));
    }

    #[test]
    fn append_keeps_existing_bytes_and_one_blank_line() {
        assert_eq!(append_section("a\n", "b\n"), "a\n\nb\n");
        assert_eq!(append_section("a", "b\n"), "a\n\nb\n");
        assert_eq!(append_section("a\n\n", "b\n"), "a\n\nb\n");
        assert_eq!(append_section("", "b\n"), "b\n");
    }
}
