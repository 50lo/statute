use crate::model::{Report, SCHEMA, Severity};

#[derive(Clone, Copy)]
pub enum Format {
    Text,
    Json,
    Markdown,
}

pub fn report(report: &Report, format: Format) {
    match format {
        Format::Json => println!("{}", to_json(report)),
        Format::Text | Format::Markdown => {
            if report.findings.is_empty() {
                println!("No findings.");
            }
            for finding in &report.findings {
                let severity = match finding.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };
                println!("{severity} {}: {}", finding.code, finding.message);
            }
        }
    }
}

pub fn error(err: &anyhow::Error, format: Format) {
    match format {
        Format::Json => {
            let value = serde_json::json!({"schema": SCHEMA, "error": format!("{err:#}")});
            println!("{value}");
        }
        Format::Text | Format::Markdown => eprintln!("error: {err:#}"),
    }
}

fn to_json(report: &Report) -> String {
    serde_json::to_string_pretty(report).expect("a report always serializes")
}
