use crate::model::ids::{is_prefix, parse_rule_id};
use crate::model::{Agreed, Finding, Model, Rule};
use crate::state;
use std::collections::{BTreeMap, BTreeSet};

fn at(f: Finding, rule: &Rule) -> Finding {
    f.rule(&rule.id).at(&rule.file, Some(rule.line))
}

/// Every rule file declares a valid prefix, and no two files share one.
pub fn prefixes(model: &Model, out: &mut Vec<Finding>) {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for file in &model.rule_files {
        let Some(prefix) = file.prefix.as_deref() else {
            continue;
        };
        if !is_prefix(prefix) {
            out.push(
                Finding::error(
                    "rule.prefix_invalid",
                    format!("prefix `{prefix}` must be uppercase letters and digits, starting with a letter"),
                )
                .at(&file.path, None),
            );
        } else if let Some(other) = seen.insert(prefix, &file.path) {
            out.push(
                Finding::error(
                    "rule.prefix_duplicate",
                    format!("prefix `{prefix}` is also declared by {other}"),
                )
                .at(&file.path, None),
            );
        }
    }
}

/// IDs have the form `PREFIX-NNN`, use the file's prefix, and are unique across all rule files.
pub fn ids(model: &Model, out: &mut Vec<Finding>) {
    let prefixes: BTreeMap<&str, Option<&str>> = model
        .rule_files
        .iter()
        .map(|f| (f.path.as_str(), f.prefix.as_deref()))
        .collect();
    let mut seen: BTreeMap<&str, &Rule> = BTreeMap::new();
    for rule in &model.rules {
        if parse_rule_id(&rule.id).is_none() {
            out.push(at(
                Finding::error(
                    "rule.id_invalid",
                    format!("`{}` is not a rule ID; a rule heading starts with an ID such as `ATTR-003`", rule.id),
                ),
                rule,
            ));
            continue;
        }
        if let Some(Some(prefix)) = prefixes.get(rule.file.as_str())
            && !rule.id.starts_with(&format!("{prefix}-"))
        {
            out.push(at(
                Finding::error(
                    "rule.id_invalid",
                    format!("{} does not use the file's prefix `{prefix}`", rule.id),
                ),
                rule,
            ));
        }
        if let Some(first) = seen.insert(&rule.id, rule) {
            out.push(at(
                Finding::error(
                    "rule.duplicate",
                    format!(
                        "{} is also defined at {}:{}",
                        rule.id, first.file, first.line
                    ),
                ),
                rule,
            ));
        }
    }
}

/// Every rule has a statement: the first paragraph after its heading.
pub fn statements(model: &Model, out: &mut Vec<Finding>) {
    for rule in model.rules.iter().filter(|r| r.statement.is_none()) {
        out.push(at(
            Finding::error(
                "rule.statement_missing",
                "rule has no statement: the first paragraph after the heading, before the `yaml` block",
            ),
            rule,
        ));
    }
}

/// Every entry in a rule's `terms` names a term.
pub fn terms_resolve(model: &Model, out: &mut Vec<Finding>) {
    let names: BTreeSet<&str> = model.terms.iter().map(|t| t.name.as_str()).collect();
    for rule in &model.rules {
        for term in rule.meta.iter().flat_map(|m| &m.terms) {
            if !names.contains(term.as_str()) {
                out.push(at(
                    Finding::error(
                        "rule.term_unknown",
                        format!("term `{term}` is not defined in terms.md"),
                    )
                    .term(term),
                    rule,
                ));
            }
        }
    }
}

/// An agreed rule has at least one `protects` and one `boundary` case.
pub fn cases(model: &Model, out: &mut Vec<Finding>) {
    for rule in &model.rules {
        let Some(meta) = &rule.meta else { continue };
        if meta.agreed == Agreed::Agreed && (meta.protects.is_empty() || meta.boundary.is_empty()) {
            out.push(at(
                Finding::error(
                    "rule.cases_missing",
                    "an agreed rule needs at least one `protects` case and one `boundary` case",
                ),
                rule,
            ));
        }
    }
}

/// An agreed or retired rule is established by exactly one decision; no rule by more than one.
pub fn establishment(model: &Model, out: &mut Vec<Finding>) {
    if model.decisions.is_none() {
        return;
    }
    let establishers = state::establishers(model);
    for rule in &model.rules {
        let by = establishers
            .get(rule.id.as_str())
            .map_or(&[][..], Vec::as_slice);
        let settled = rule
            .meta
            .as_ref()
            .is_some_and(|m| m.agreed != Agreed::Proposed);
        if by.len() > 1 {
            out.push(at(
                Finding::error(
                    "rule.established_twice",
                    format!(
                        "{} is established by more than one decision: {}",
                        rule.id,
                        by.join(", ")
                    ),
                ),
                rule,
            ));
        } else if by.is_empty() && settled {
            out.push(at(
                Finding::error(
                    "rule.unestablished",
                    format!(
                        "{} is agreed or retired, but no decision establishes it",
                        rule.id
                    ),
                ),
                rule,
            ));
        }
    }
}

/// A rule statement that uses a term's `avoid` word gets a warning.
pub fn avoid_words(model: &Model, out: &mut Vec<Finding>) {
    for term in &model.terms {
        for word in term.meta.iter().flat_map(|m| &m.avoid) {
            for rule in &model.rules {
                if rule
                    .statement
                    .as_deref()
                    .is_some_and(|s| contains_word(s, word))
                {
                    out.push(at(
                        Finding::warning(
                            "rule.avoid_word",
                            format!(
                                "statement uses `{word}`, which the term {} avoids",
                                term.name
                            ),
                        )
                        .term(&term.name),
                        rule,
                    ));
                }
            }
        }
    }
}

/// Case-insensitive match of a word or phrase that is not part of a longer word.
fn contains_word(text: &str, word: &str) -> bool {
    let (text, word) = (text.to_lowercase(), word.to_lowercase());
    if word.is_empty() {
        return false;
    }
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(&word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
}

#[cfg(test)]
mod tests {
    use super::contains_word;

    #[test]
    fn whole_words_only() {
        assert!(contains_word("Open a Chat window.", "chat"));
        assert!(contains_word("an agent session ends", "agent session"));
        assert!(!contains_word("chatter", "chat"));
        assert!(!contains_word("webchat", "chat"));
    }
}
