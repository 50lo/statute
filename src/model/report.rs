use serde::Serialize;

/// Version of the `--json` output format.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Makes the command exit with 1.
    Error,
    /// Reported; the exit code stays 0.
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable code, grouped by concept, such as `link.stale`. Skills match on it.
    pub code: String,
    pub severity: Severity,
    pub message: String,
}

/// What a command produces. Text, `--json` and `--md` are renderings of it.
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema: u32,
    pub command: &'static str,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn new(command: &'static str) -> Self {
        Self {
            schema: SCHEMA,
            command,
            findings: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }
}
