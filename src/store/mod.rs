//! Parses each kind of Statute file from text and renders edits back to text. No I/O.

mod lists;
pub mod lock;
mod markdown;
pub mod rules;
pub mod terms;

pub use lists::{
    CONFIG_TEMPLATE, EMPTY_LIST, LOCK_TEMPLATE, parse_config, parse_decisions, parse_gaps,
};

use crate::model::Finding;

pub use markdown::set_field;

/// A YAML error as a finding, with the line in the file. `first_line` is the file line of the YAML's first line.
fn yaml_finding(
    code: &'static str,
    what: &str,
    e: &yaml_serde::Error,
    path: &str,
    first_line: usize,
) -> Finding {
    let line = e.location().map(|l| first_line + l.line() - 1);
    // The location is reported separately; drop it from the message.
    let message = e.to_string();
    let message = message.split(" at line ").next().unwrap_or(&message);
    Finding::error(code, format!("{what} does not parse: {message}"))
        .at(path, line.or(Some(first_line)))
}

const SNIPPET_BEGIN: &str = "<!-- statute:begin -->";
const SNIPPET_END: &str = "<!-- statute:end -->";

/// AGENTS.md with the Statute snippet appended, or `None` when it already has one.
pub fn with_agents_snippet(existing: Option<&str>, snippet: &str) -> Option<String> {
    let existing = existing.unwrap_or("");
    if existing.contains(SNIPPET_BEGIN) {
        return None;
    }
    let block = format!("{SNIPPET_BEGIN}\n{}\n{SNIPPET_END}\n", snippet.trim_end());
    Some(rules::append_section(existing, &block))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_is_appended_once() {
        let first = with_agents_snippet(Some("# Project\n"), "## Statute\n\n- Rule.\n").unwrap();
        assert_eq!(
            first,
            "# Project\n\n<!-- statute:begin -->\n## Statute\n\n- Rule.\n<!-- statute:end -->\n"
        );
        assert_eq!(with_agents_snippet(Some(&first), "## Statute\n"), None);
        assert!(
            with_agents_snippet(None, "x")
                .unwrap()
                .starts_with(SNIPPET_BEGIN)
        );
    }
}
