use serde::Serialize;

/// A problem `check` reports. Skills match on `code`, never on `message`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub code: &'static str,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    /// A link's target, for `link.*` findings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl Finding {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Finding {
            code,
            severity: Severity::Error,
            rule: None,
            term: None,
            decision: None,
            gap: None,
            target: None,
            file: None,
            line: None,
            message: message.into(),
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Finding {
            severity: Severity::Warning,
            ..Finding::error(code, message)
        }
    }

    pub fn rule(mut self, id: &str) -> Self {
        self.rule = Some(id.to_string());
        self
    }

    pub fn term(mut self, name: &str) -> Self {
        self.term = Some(name.to_string());
        self
    }

    pub fn decision(mut self, id: &str) -> Self {
        self.decision = Some(id.to_string());
        self
    }

    pub fn gap(mut self, id: &str) -> Self {
        self.gap = Some(id.to_string());
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn at(mut self, file: &str, line: Option<usize>) -> Self {
        self.file = Some(file.to_string());
        self.line = line;
        self
    }
}
