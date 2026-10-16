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

/// Appends a rule skeleton to `statute/rules/<area>.md` with the next free ID for the file's prefix.
pub fn run(area: &str, title: &str) -> Result<RuleNewReport> {
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

    let repo = Repo::discover()?;
    let file = paths::rule_file(area);
    let Some(text) = repo.read(&file)? else {
        bail!("{file} does not exist; create it with `prefix: <PREFIX>` in its frontmatter first");
    };
    let model = repo.load()?;
    let prefix = model
        .rule_files
        .iter()
        .find(|f| f.area == area)
        .and_then(|f| f.prefix.as_deref())
        .filter(|p| ids::is_prefix(p));
    let Some(prefix) = prefix else {
        bail!("{file} needs a valid `prefix:` in its frontmatter, such as `prefix: BILL`");
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

    let mut txn = Txn::new(&repo);
    txn.write(
        &file,
        store::rules::append_section(&text, &store::rules::skeleton(&id, title)),
    );
    txn.commit()?;
    Ok(RuleNewReport {
        schema: SCHEMA,
        id,
        file,
    })
}
