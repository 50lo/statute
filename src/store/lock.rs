//! `statute.lock`: JSON Lines, a version line, then one link per line sorted by source and target.

#![expect(
    clippy::result_large_err,
    reason = "a parse error is rare and becomes a finding"
)]

use crate::model::{Finding, Link, LinkRole, LinkSource, Mode, paths};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionLine {
    version: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkLine {
    source: String,
    target: String,
    role: LinkRole,
    mode: Mode,
    sig: String,
}

fn problem(line: usize, message: String) -> Finding {
    Finding::error(
        "link.parse",
        format!("statute.lock does not parse: {message}"),
    )
    .at(paths::LOCK, Some(line))
}

/// Parses the lock. Blank lines are ignored, and an empty file has no links.
pub fn parse(text: &str) -> Result<Vec<Link>, Finding> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l))
        .filter(|(_, l)| !l.trim().is_empty());
    let Some((n, first)) = lines.next() else {
        return Ok(Vec::new());
    };
    let version: VersionLine = serde_json::from_str(first)
        .map_err(|e| problem(n, format!("the first line must be {{\"version\": 1}}: {e}")))?;
    if version.version != 1 {
        return Err(problem(
            n,
            format!(
                "version {}; this statute supports version 1",
                version.version
            ),
        ));
    }
    lines
        .map(|(n, line)| {
            let l: LinkLine = serde_json::from_str(line).map_err(|e| problem(n, e.to_string()))?;
            Ok(Link {
                source: LinkSource::Rule(l.source),
                target: l.target,
                role: l.role,
                mode: l.mode,
                sig: l.sig,
            })
        })
        .collect()
}

/// The whole lock in its fixed format: the version line, then links sorted by source and target.
pub fn render(links: &[Link]) -> String {
    let mut sorted: Vec<&Link> = links.iter().collect();
    sorted.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));
    let mut out = String::from(super::LOCK_TEMPLATE);
    for l in sorted {
        out.push_str(&format!(
            "{{\"source\": {}, \"target\": {}, \"role\": {}, \"mode\": {}, \"sig\": {}}}\n",
            json(l.rule()),
            json(&l.target),
            json(&l.role),
            json(&l.mode),
            json(&l.sig),
        ));
    }
    out
}

fn json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("strings and unit variants serialize")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = "{\"version\": 1}\n{\"source\": \"ATTR-001\", \"target\": \"src/a.go\", \"role\": \"code\", \"mode\": \"bytes-sha256-v1\", \"sig\": \"ab\"}\n";

    #[test]
    fn parses_and_renders_the_same_bytes() {
        let links = parse(LOCK).unwrap();
        assert_eq!(links[0].rule(), "ATTR-001");
        assert_eq!(links[0].role, LinkRole::Code);
        assert_eq!(render(&links), LOCK);
        assert!(parse("").unwrap().is_empty());
        assert!(parse(super::super::LOCK_TEMPLATE).unwrap().is_empty());
    }

    #[test]
    fn renders_links_sorted_by_source_and_target() {
        let link = |source: &str, target: &str| Link {
            source: LinkSource::Rule(source.into()),
            target: target.into(),
            role: LinkRole::Test,
            mode: Mode::BytesSha256V1,
            sig: "s".into(),
        };
        let out = render(&[link("B-001", "a"), link("A-001", "z"), link("A-001", "b")]);
        let targets: Vec<_> = parse(&out)
            .unwrap()
            .iter()
            .map(|l| format!("{} {}", l.rule(), l.target))
            .collect();
        assert_eq!(targets, ["A-001 b", "A-001 z", "B-001 a"]);
    }

    #[test]
    fn errors_carry_the_line() {
        let bad_mode = LOCK.replace("bytes-sha256-v1", "md5");
        assert_eq!(parse(&bad_mode).unwrap_err().line, Some(2));
        let unknown_key = LOCK.replace("\"sig\"", "\"sgi\"");
        assert_eq!(parse(&unknown_key).unwrap_err().code, "link.parse");
        let no_version = LOCK.lines().nth(1).unwrap();
        assert_eq!(parse(no_version).unwrap_err().line, Some(1));
        let v2 = parse("{\"version\": 2}\n").unwrap_err();
        assert!(v2.message.contains("version 2"), "{}", v2.message);
    }
}
