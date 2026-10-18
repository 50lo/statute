use super::SCHEMA;
use crate::model::{
    Agreed, Built, Decision, DecisionStatus, Gap, Link, LinkRole, Model, Rule, Span, Term,
    TermAgreed, paths, split_target,
};
use crate::repo::Repo;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct ShowReport {
    pub schema: u32,
    /// One item per distinct rule, term or decision, in the order the arguments name them.
    pub items: Vec<Item>,
    /// File and symbol arguments that no rule links to.
    pub unlinked: Vec<String>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Item {
    Rule(RuleShow),
    Term(TermShow),
    Decision(Decision),
}

#[derive(Serialize)]
pub struct RuleShow {
    pub id: String,
    pub title: String,
    pub file: String,
    pub line: usize,
    /// `None` when the rule's block does not parse; `check` reports it.
    pub agreed: Option<Agreed>,
    pub built: Option<Built>,
    /// The rule's section as written: heading, statement, block and notes.
    pub text: String,
    pub terms: Vec<TermRef>,
    /// `None` when `statute.lock` does not parse.
    pub links: Option<Vec<LinkRef>>,
    /// `None` when `decisions.db` does not parse.
    pub decisions: Option<Vec<DecisionRef>>,
    /// `None` when `gaps.yml` does not parse.
    pub gaps: Option<Vec<Gap>>,
}

#[derive(Serialize)]
pub struct TermRef {
    pub name: String,
    /// `None` when no term has this name, or its `agreed` field is absent or does not parse.
    pub agreed: Option<TermAgreed>,
    /// `None` when no term has this name, or the term has no definition.
    pub definition: Option<String>,
}

#[derive(Serialize)]
pub struct LinkRef {
    pub target: String,
    pub role: LinkRole,
}

#[derive(Serialize)]
pub struct DecisionRef {
    pub id: String,
    /// How the decision names the rule or term.
    pub role: Role,
    pub status: DecisionStatus,
    pub summary: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Establishes,
    Changes,
    Retires,
    /// A term the decision introduces or redefines: it is in `terms`.
    Defines,
}

#[derive(Serialize)]
pub struct TermShow {
    pub name: String,
    pub file: String,
    pub line: usize,
    /// `None` when the `agreed` field is absent or the block does not parse.
    pub agreed: Option<TermAgreed>,
    /// The term's section as written: heading, definition and block.
    pub text: String,
    pub rules: Vec<RuleRef>,
    /// The decisions that define or retire the term. `None` when `decisions.db` does not parse.
    pub decisions: Option<Vec<DecisionRef>>,
}

#[derive(Serialize)]
pub struct RuleRef {
    pub id: String,
    pub title: String,
}

/// Prints each named rule, term or decision once. An argument is matched as a rule ID, then a decision
/// ID, then a term name, then a file or symbol, which shows the rules linked to it.
pub fn run(names: &[String]) -> Result<ShowReport> {
    let repo = Repo::discover()?;
    let model = repo.load()?;
    let mut texts: BTreeMap<&str, String> = BTreeMap::new();
    let mut items = Vec::new();
    let mut unlinked = Vec::new();
    let mut seen = Vec::new();
    for name in names {
        let found = if let Some(rule) = model.rules.iter().find(|r| r.id == *name) {
            vec![rule_item(&repo, &mut texts, &model, rule)?]
        } else if let Some(d) = model.decisions.iter().flatten().find(|d| d.id == *name) {
            vec![Item::Decision(d.clone())]
        } else if let Some(term) = model.terms.iter().find(|t| t.name == *name) {
            vec![Item::Term(TermShow {
                name: term.name.clone(),
                file: paths::TERMS.to_string(),
                line: term.line,
                agreed: term.agreed(),
                text: section(&repo, &mut texts, paths::TERMS, &term.span)?,
                rules: rules_using(&model, &term.name),
                decisions: model
                    .decisions
                    .as_ref()
                    .map(|ds| decisions_naming_term(ds, &term.name)),
            })]
        } else if let Some(rules) = linked_to(&repo, &model, name) {
            if rules.is_empty() {
                unlinked.push(name.clone());
            }
            rules
                .into_iter()
                .map(|rule| rule_item(&repo, &mut texts, &model, rule))
                .collect::<Result<_>>()?
        } else if model.decisions.is_none() {
            bail!(
                "no rule or term named `{name}`, and {} does not parse; run `statute check`",
                paths::DECISIONS
            );
        } else {
            bail!("no rule, term, decision or file named `{name}`");
        };
        for item in found {
            let key = match &item {
                Item::Rule(r) => ("rule", r.id.clone()),
                Item::Term(t) => ("term", t.name.clone()),
                Item::Decision(d) => ("decision", d.id.clone()),
            };
            if !seen.contains(&key) {
                seen.push(key);
                items.push(item);
            }
        }
    }
    Ok(ShowReport {
        schema: SCHEMA,
        items,
        unlinked,
    })
}

/// The rules linked to a file (whole-file and symbol links in it) or to a symbol (its own links and
/// whole-file links to its file), in lock order. `None` when the argument is neither an existing file
/// nor a linked target.
fn linked_to<'m>(repo: &Repo, model: &'m Model, name: &str) -> Option<Vec<&'m Rule>> {
    let (file, symbol) = split_target(name);
    let links = model.links.as_deref().unwrap_or_default();
    let matching: Vec<&Link> = links
        .iter()
        .filter(|l| l.file() == file && (symbol.is_none() || l.target == name || l.target == file))
        .collect();
    if matching.is_empty() && !repo.is_file(file) {
        return None;
    }
    let mut rules: Vec<&Rule> = Vec::new();
    for link in matching {
        if let Some(rule) = model.rules.iter().find(|r| r.id == link.rule())
            && !rules.iter().any(|r| r.id == rule.id)
        {
            rules.push(rule);
        }
    }
    Some(rules)
}

fn rule_item<'a>(
    repo: &Repo,
    texts: &mut BTreeMap<&'a str, String>,
    model: &Model,
    rule: &'a Rule,
) -> Result<Item> {
    Ok(Item::Rule(RuleShow {
        id: rule.id.clone(),
        title: rule.title.clone(),
        file: rule.file.clone(),
        line: rule.line,
        agreed: rule.meta.as_ref().map(|m| m.agreed),
        built: rule.meta.as_ref().map(|m| m.built),
        text: section(repo, texts, &rule.file, &rule.span)?,
        terms: rule
            .meta
            .iter()
            .flat_map(|m| &m.terms)
            .map(|t| {
                let term = model.terms.iter().find(|term| term.name == *t);
                TermRef {
                    name: t.clone(),
                    agreed: term.and_then(Term::agreed),
                    definition: term.and_then(|term| term.definition.clone()),
                }
            })
            .collect(),
        links: model.links.as_ref().map(|links| {
            links
                .iter()
                .filter(|l| l.rule() == rule.id)
                .map(|l| LinkRef {
                    target: l.target.clone(),
                    role: l.role,
                })
                .collect()
        }),
        decisions: model
            .decisions
            .as_ref()
            .map(|ds| decisions_naming(ds, &rule.id)),
        gaps: model
            .gaps
            .as_ref()
            .map(|gaps| gaps.iter().filter(|g| g.rule == rule.id).cloned().collect()),
    }))
}

/// A section's source text, without trailing blank lines. Each file is read once.
fn section<'a>(
    repo: &Repo,
    texts: &mut BTreeMap<&'a str, String>,
    file: &'a str,
    span: &Span,
) -> Result<String> {
    if !texts.contains_key(file) {
        texts.insert(file, repo.read(file)?.unwrap_or_default());
    }
    let text = &texts[file];
    let section = text.get(span.heading.start..span.end).unwrap_or_default();
    Ok(section.trim_end().to_string())
}

fn decisions_naming(decisions: &[Decision], rule: &str) -> Vec<DecisionRef> {
    let mut out = Vec::new();
    for d in decisions {
        let roles = [
            (Role::Establishes, &d.establishes),
            (Role::Changes, &d.changes),
            (Role::Retires, &d.retires),
        ];
        for (role, ids) in roles {
            if ids.iter().any(|id| id == rule) {
                out.push(DecisionRef {
                    id: d.id.clone(),
                    role,
                    status: d.status,
                    summary: d.summary.clone(),
                });
            }
        }
    }
    out
}

fn decisions_naming_term(decisions: &[Decision], term: &str) -> Vec<DecisionRef> {
    let mut out = Vec::new();
    for d in decisions {
        for (role, names) in [(Role::Defines, &d.terms), (Role::Retires, &d.retires_terms)] {
            if names.iter().any(|n| n == term) {
                out.push(DecisionRef {
                    id: d.id.clone(),
                    role,
                    status: d.status,
                    summary: d.summary.clone(),
                });
            }
        }
    }
    out
}

fn rules_using(model: &Model, term: &str) -> Vec<RuleRef> {
    model
        .rules
        .iter()
        .filter(|r| {
            r.meta
                .as_ref()
                .is_some_and(|m| m.terms.iter().any(|t| t == term))
        })
        .map(|r| RuleRef {
            id: r.id.clone(),
            title: r.title.clone(),
        })
        .collect()
}
