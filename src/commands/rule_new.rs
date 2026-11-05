use super::SCHEMA;
use crate::model::{ids, paths};
use crate::repo::Repo;
use crate::store;
use crate::txn::Txn;
use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Serialize)]
pub struct RuleNewReport {
    pub schema: u32,
    pub id: String,
    pub file: String,
}

/// Appends a rule skeleton to `statute/rules/<area>.md`. With `requested_prefix`, creates the file.
pub fn run(area: &str, title: &str, requested_prefix: Option<&str>) -> Result<RuleNewReport> {
    let valid_area = !area.is_empty()
        && area
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid_area {
        bail!("area `{area}` must be a rule file name: letters, digits, `-` and `_`");
    }
    let title = title.trim();
    if title.is_empty() || title.contains('\n') {
        bail!("the title must be one non-empty line");
    }
    if let Some(prefix) = requested_prefix
        && !ids::is_prefix(prefix)
    {
        bail!("prefix `{prefix}` must be {}", ids::PREFIX_FORMAT);
    }

    let repo = Repo::discover()?;
    let file = paths::rule_file(area);
    let text = match (repo.read(&file)?, requested_prefix) {
        (Some(_), Some(_)) => bail!(
            "{file} already exists; `--prefix` only creates a new rule file, so run the command without it"
        ),
        (None, None) => bail!(
            "{file} does not exist; create it with `prefix: <PREFIX>` in its frontmatter first, or use `statute rule new {area} \"{title}\" --prefix <PREFIX>`"
        ),
        (Some(text), None) => text,
        (None, Some(prefix)) => store::rules::new_file(prefix),
    };
    let model = repo.load()?;
    let prefix = if let Some(prefix) = requested_prefix {
        if let Some(other) = model
            .rule_files
            .iter()
            .find(|other| other.prefix.as_deref() == Some(prefix))
        {
            bail!(
                "prefix `{prefix}` is already declared by {}; choose a unique prefix",
                other.path
            );
        }
        prefix
    } else {
        let declared = model
            .rule_files
            .iter()
            .find(|f| f.area == area)
            .and_then(|f| f.prefix.as_deref());
        match declared {
            Some(prefix) if ids::is_prefix(prefix) => prefix,
            Some(invalid) => bail!("{file}: prefix `{invalid}` must be {}", ids::PREFIX_FORMAT),
            None => {
                bail!("{file} needs a valid `prefix:` in its frontmatter, such as `prefix: BILL`")
            }
        }
    };

    let last = model
        .rules
        .iter()
        .filter_map(|r| ids::parse_rule_id(&r.id))
        .filter(|(p, _)| *p == prefix)
        .map(|(_, n)| n)
        .max()
        .unwrap_or(0);
    if last >= 999 {
        bail!("prefix {prefix} has no free rule IDs left");
    }
    let id = format!("{prefix}-{:03}", last + 1);

    let new_text = store::rules::append_section(&text, &store::rules::skeleton(&id, title));
    // The end of the file can be inside an unclosed code block, which would swallow the new section.
    let parsed = store::rules::parse(&file, area, &new_text);
    if !parsed.rules.iter().any(|r| r.id == id && r.meta.is_some()) {
        bail!(
            "cannot append {id} to {file}: the file ends inside an unclosed block, such as a code fence; close it first"
        );
    }

    let mut txn = Txn::new(&repo);
    txn.write(&file, new_text);
    txn.commit()?;
    Ok(RuleNewReport {
        schema: SCHEMA,
        id,
        file,
    })
}
