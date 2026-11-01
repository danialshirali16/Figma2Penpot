#![forbid(unsafe_code)]
//! Event-based conversion reports.
//!
//! Converters emit a [`ReportEvent`] at every decision point through a
//! [`DiagnosticSink`]. Counts are derived from those events and a real traversal,
//! never estimated from the produced file.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Stable report schema version for JSON consumers.
pub const REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Converted,
    ConvertedWithModification,
    Skipped,
    Unsupported,
    Failed,
}

impl Outcome {
    fn rank(self) -> u8 {
        match self {
            Self::Converted => 0,
            Self::ConvertedWithModification => 1,
            Self::Skipped => 2,
            Self::Unsupported => 3,
            Self::Failed => 4,
        }
    }

    fn worst(self, other: Self) -> Self {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementRef {
    pub figma_node_id: String,
    pub name: String,
    pub element_type: String,
    pub page: Option<String>,
    pub path: Vec<String>,
}

impl ElementRef {
    fn key(&self) -> &str {
        &self.figma_node_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportEvent {
    pub code: String,
    pub severity: Severity,
    pub outcome: Outcome,
    pub element: ElementRef,
    pub description: String,
    pub reason: Option<String>,
    pub action_taken: String,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub detected: u64,
    pub converted: u64,
    pub approximated: u64,
    pub skipped: u64,
    pub unsupported: u64,
    pub failed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    Passed,
    Failed,
    NotRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Completed,
    CompletedWithWarnings,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReport {
    pub schema_version: u32,
    pub source: String,
    pub output: Option<String>,
    pub status: FileStatus,
    pub duration_ms: u64,
    pub counts: Counts,
    pub events: Vec<ReportEvent>,
    pub validation: ValidationState,
    pub tool_version: String,
    pub target_penpot: String,
}

impl FileReport {
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// RFC 4180 CSV with one row per report event and a `source` column.
    pub fn to_csv(&self) -> String {
        batch_csv(std::slice::from_ref(self))
    }

    /// Self-contained HTML. No external scripts, fonts, or network assets.
    pub fn to_html(&self) -> String {
        batch_html(std::slice::from_ref(self))
    }
}

/// Receives diagnostics while conversion runs. No conversion pass may skip it.
pub trait DiagnosticSink {
    /// Record one Figma element as detected. Called once per element.
    fn count_detected(&mut self, element: &ElementRef);
    /// Record a decision or finding for an element.
    fn emit(&mut self, event: ReportEvent);
}

#[derive(Debug, Default)]
pub struct ReportCollector {
    detected: u64,
    seen: HashMap<String, ()>,
    outcomes: HashMap<String, Outcome>,
    events: Vec<ReportEvent>,
}

impl ReportCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[ReportEvent] {
        &self.events
    }

    pub fn finish(
        self,
        source: impl Into<String>,
        output: Option<String>,
        duration: Duration,
        validation: ValidationState,
        tool_version: impl Into<String>,
        target_penpot: impl Into<String>,
    ) -> FileReport {
        let mut counts = Counts {
            detected: self.detected,
            ..Counts::default()
        };
        for outcome in self.outcomes.values() {
            match outcome {
                Outcome::Converted => counts.converted += 1,
                Outcome::ConvertedWithModification => counts.approximated += 1,
                Outcome::Skipped => counts.skipped += 1,
                Outcome::Unsupported => counts.unsupported += 1,
                Outcome::Failed => counts.failed += 1,
            }
        }
        // A detected element without a terminal event is a converter bug. Do not
        // silently lose it: count it as failed so the report is conservative.
        let unaccounted = counts.detected.saturating_sub(self.outcomes.len() as u64);
        counts.failed += unaccounted;

        let has_errors = counts.failed > 0
            || self
                .events
                .iter()
                .any(|event| event.severity == Severity::Error);
        let has_warnings = self.events.iter().any(|event| {
            event.severity == Severity::Warning || event.outcome != Outcome::Converted
        });
        let status = if has_errors {
            FileStatus::Failed
        } else if has_warnings {
            FileStatus::CompletedWithWarnings
        } else {
            FileStatus::Completed
        };

        FileReport {
            schema_version: REPORT_SCHEMA_VERSION,
            source: source.into(),
            output,
            status,
            duration_ms: duration.as_millis().min(u64::MAX as u128) as u64,
            counts,
            events: self.events,
            validation,
            tool_version: tool_version.into(),
            target_penpot: target_penpot.into(),
        }
    }
}

impl DiagnosticSink for ReportCollector {
    fn count_detected(&mut self, element: &ElementRef) {
        if self.seen.insert(element.key().to_owned(), ()).is_none() {
            self.detected += 1;
        }
    }

    fn emit(&mut self, event: ReportEvent) {
        let key = event.element.key().to_owned();
        self.outcomes
            .entry(key)
            .and_modify(|outcome| *outcome = outcome.worst(event.outcome))
            .or_insert(event.outcome);
        self.events.push(event);
    }
}

/// A sink that discards everything. Useful in tests that only need the document.
#[derive(Debug, Default)]
pub struct NoopSink;

impl DiagnosticSink for NoopSink {
    fn count_detected(&mut self, _element: &ElementRef) {}

    fn emit(&mut self, _event: ReportEvent) {}
}

const CSV_HEADER: &str = "source,code,severity,outcome,figmaNodeId,elementName,elementType,page,path,description,reason,actionTaken,suggestedAction\n";

/// One CSV with a source column per report, preserving element-level rows.
pub fn batch_csv(reports: &[FileReport]) -> String {
    let mut out = String::from(CSV_HEADER);
    for report in reports {
        for event in &report.events {
            let row = [
                report.source.as_str(),
                event.code.as_str(),
                severity_name(event.severity),
                outcome_name(event.outcome),
                event.element.figma_node_id.as_str(),
                event.element.name.as_str(),
                event.element.element_type.as_str(),
                event.element.page.as_deref().unwrap_or(""),
                &event.element.path.join(" / "),
                event.description.as_str(),
                event.reason.as_deref().unwrap_or(""),
                event.action_taken.as_str(),
                event.suggested_action.as_deref().unwrap_or(""),
            ];
            for (index, value) in row.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&csv_escape(value));
            }
            out.push('\n');
        }
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

/// Self-contained batch HTML with a per-file summary and grouped events.
pub fn batch_html(reports: &[FileReport]) -> String {
    let title = if reports.len() == 1 {
        format!("Figma 2 Penpot report — {}", reports[0].source)
    } else {
        format!("Figma 2 Penpot batch report ({} files)", reports.len())
    };
    let mut out = String::new();
    let _ = write!(
        out,
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta name=\"color-scheme\" content=\"light dark\"><title>{}</title><style>{}</style></head><body>",
        escape_html(&title),
        HTML_CSS
    );
    let _ = write!(
        out,
        "<header><p class=\"eyebrow\">Figma 2 Penpot</p><h1>{}</h1></header>",
        escape_html(&title)
    );
    out.push_str("<section aria-labelledby=\"summary\"><h2 id=\"summary\">Summary</h2><table><thead><tr><th>File</th><th>Status</th><th>Detected</th><th>Converted</th><th>Approximated</th><th>Skipped</th><th>Unsupported</th><th>Failed</th><th>Validation</th></tr></thead><tbody>");
    for report in reports {
        let _ = write!(
            out,
            "<tr><td>{}</td><td class=\"status {}\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&report.source),
            status_class(report.status),
            escape_html(file_status_name(report.status)),
            report.counts.detected,
            report.counts.converted,
            report.counts.approximated,
            report.counts.skipped,
            report.counts.unsupported,
            report.counts.failed,
            escape_html(validation_name(report.validation)),
        );
    }
    out.push_str("</tbody></table></section>");
    for report in reports {
        out.push_str("<section class=\"file\"><h2>");
        out.push_str(&escape_html(&report.source));
        out.push_str("</h2><dl class=\"meta\">");
        meta(&mut out, "Status", file_status_name(report.status));
        meta(&mut out, "Validation", validation_name(report.validation));
        meta(&mut out, "Duration", &format!("{} ms", report.duration_ms));
        meta(&mut out, "Output", report.output.as_deref().unwrap_or("—"));
        meta(&mut out, "Tool", &report.tool_version);
        meta(&mut out, "Target", &report.target_penpot);
        out.push_str("</dl>");
        if report.events.is_empty() {
            out.push_str("<p class=\"empty\">No conversion events were recorded.</p>");
        } else {
            let mut groups: BTreeMap<&str, Vec<&ReportEvent>> = BTreeMap::new();
            for event in &report.events {
                groups.entry(event.code.as_str()).or_default().push(event);
            }
            for (code, events) in groups {
                let _ = write!(
                    out,
                    "<details><summary><code>{}</code> <span class=\"count\">{} event(s)</span></summary><table><thead><tr><th>Severity</th><th>Outcome</th><th>Element</th><th>Page</th><th>Description</th><th>Reason</th><th>Action taken</th><th>Suggested action</th></tr></thead><tbody>",
                    escape_html(code),
                    events.len()
                );
                for event in events {
                    let _ = write!(
                        out,
                        "<tr><td class=\"sev {}\">{}</td><td>{}</td><td>{} <span class=\"muted\">({})</span><br><span class=\"muted\">{}</span></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        severity_class(event.severity),
                        escape_html(severity_name(event.severity)),
                        escape_html(outcome_name(event.outcome)),
                        escape_html(&event.element.name),
                        escape_html(&event.element.element_type),
                        escape_html(&event.element.path.join(" / ")),
                        escape_html(event.element.page.as_deref().unwrap_or("—")),
                        escape_html(&event.description),
                        escape_html(event.reason.as_deref().unwrap_or("—")),
                        escape_html(&event.action_taken),
                        escape_html(event.suggested_action.as_deref().unwrap_or("—")),
                    );
                }
                out.push_str("</tbody></table></details>");
            }
        }
        out.push_str("</section>");
    }
    out.push_str("<footer><p>Generated locally by Figma 2 Penpot. No network access is required to view this report.</p></footer></body></html>");
    out
}

fn meta(out: &mut String, label: &str, value: &str) {
    let _ = write!(
        out,
        "<div><dt>{}</dt><dd>{}</dd></div>",
        escape_html(label),
        escape_html(value)
    );
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "Info",
        Severity::Warning => "Warning",
        Severity::Error => "Error",
    }
}

fn severity_class(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Converted => "Converted",
        Outcome::ConvertedWithModification => "Converted with modification",
        Outcome::Skipped => "Skipped",
        Outcome::Unsupported => "Unsupported",
        Outcome::Failed => "Failed",
    }
}

fn file_status_name(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Completed => "Completed",
        FileStatus::CompletedWithWarnings => "Completed with warnings",
        FileStatus::Failed => "Failed",
        FileStatus::Cancelled => "Cancelled",
    }
}

fn status_class(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Completed => "ok",
        FileStatus::CompletedWithWarnings => "warn",
        FileStatus::Failed => "bad",
        FileStatus::Cancelled => "muted",
    }
}

fn validation_name(state: ValidationState) -> &'static str {
    match state {
        ValidationState::Passed => "Passed",
        ValidationState::Failed => "Failed",
        ValidationState::NotRun => "Not run",
    }
}

fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(character),
        }
    }
    out
}

const HTML_CSS: &str = r#"
:root { color-scheme: light dark; --bg:#f7f7f8; --fg:#17181a; --muted:#5e6269; --line:#d9dce1; --panel:#fff; --accent:#0a6cff; }
@media (prefers-color-scheme: dark) { :root { --bg:#141517; --fg:#f0f1f3; --muted:#a5a9b0; --line:#303238; --panel:#1c1d20; --accent:#74a7ff; } }
* { box-sizing: border-box; }
body { margin:0; padding:28px; background:var(--bg); color:var(--fg); font:15px/1.5 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif; }
header, section { max-width:1200px; margin:0 auto 24px; }
h1 { font-size:28px; line-height:1.2; margin:4px 0 0; }
h2 { font-size:19px; margin:0 0 12px; }
.eyebrow { margin:0; color:var(--muted); text-transform:uppercase; letter-spacing:.08em; font-size:12px; }
table { width:100%; border-collapse:collapse; background:var(--panel); border:1px solid var(--line); }
th, td { text-align:left; vertical-align:top; padding:8px 10px; border-bottom:1px solid var(--line); }
th { font-size:12px; text-transform:uppercase; letter-spacing:.04em; color:var(--muted); }
tr:last-child td { border-bottom:0; }
.meta { display:grid; grid-template-columns:repeat(auto-fit,minmax(190px,1fr)); gap:8px 18px; margin:0 0 16px; }
.meta div { background:var(--panel); border:1px solid var(--line); padding:8px 10px; }
dt { color:var(--muted); font-size:12px; text-transform:uppercase; letter-spacing:.04em; }
dd { margin:2px 0 0; overflow-wrap:anywhere; }
details { background:var(--panel); border:1px solid var(--line); margin:8px 0; }
summary { cursor:pointer; padding:9px 11px; }
.count { color:var(--muted); font-size:13px; margin-left:8px; }
code { font:13px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace; }
.muted { color:var(--muted); }
.sev.info { color:var(--accent); }
.sev.warning { color:#9a6100; }
.sev.error { color:#b3261e; }
.status.ok { color:#1b7a35; }
.status.warn { color:#9a6100; }
.status.bad { color:#b3261e; }
.empty, footer { color:var(--muted); }
footer { max-width:1200px; margin:28px auto 0; font-size:13px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn element(id: &str) -> ElementRef {
        ElementRef {
            figma_node_id: id.to_owned(),
            name: "Node".to_owned(),
            element_type: "RECTANGLE".to_owned(),
            page: Some("Page 1".to_owned()),
            path: vec!["Page 1".to_owned()],
        }
    }

    fn event(id: &str, outcome: Outcome) -> ReportEvent {
        ReportEvent {
            code: "F2P-TEST".to_owned(),
            severity: Severity::Info,
            outcome,
            element: element(id),
            description: "test".to_owned(),
            reason: None,
            action_taken: "nothing".to_owned(),
            suggested_action: None,
        }
    }

    #[test]
    fn counts_use_worst_outcome_per_element() {
        let mut collector = ReportCollector::new();
        collector.count_detected(&element("1:1"));
        collector.emit(event("1:1", Outcome::Converted));
        collector.emit(event("1:1", Outcome::ConvertedWithModification));
        collector.count_detected(&element("1:2"));
        collector.emit(event("1:2", Outcome::Unsupported));

        let report = collector.finish(
            "in.fig",
            Some("out.penpot".into()),
            Duration::from_millis(12),
            ValidationState::Passed,
            "0.1.0",
            "2.18.5",
        );
        assert_eq!(report.counts.detected, 2);
        assert_eq!(report.counts.converted, 0);
        assert_eq!(report.counts.approximated, 1);
        assert_eq!(report.counts.unsupported, 1);
        assert_eq!(report.counts.failed, 0);
        assert_eq!(report.status, FileStatus::CompletedWithWarnings);
    }

    #[test]
    fn unaccounted_elements_are_counted_as_failed() {
        let mut collector = ReportCollector::new();
        collector.count_detected(&element("1:1"));
        let report = collector.finish(
            "in.fig",
            None,
            Duration::ZERO,
            ValidationState::NotRun,
            "0.1.0",
            "2.18.5",
        );
        assert_eq!(report.counts.detected, 1);
        assert_eq!(report.counts.failed, 1);
        assert_eq!(report.status, FileStatus::Failed);
    }

    #[test]
    fn csv_escapes_commas_quotes_and_newlines() {
        let mut collector = ReportCollector::new();
        let mut element = element("1:9");
        element.name = "Label, \"quoted\"".to_owned();
        collector.count_detected(&element);
        collector.emit(ReportEvent {
            code: "F2P-CSV".to_owned(),
            severity: Severity::Warning,
            outcome: Outcome::ConvertedWithModification,
            element,
            description: "line one\nline two".to_owned(),
            reason: Some("reason, with comma".to_owned()),
            action_taken: "kept".to_owned(),
            suggested_action: None,
        });
        let report = collector.finish(
            "in.fig",
            None,
            Duration::ZERO,
            ValidationState::NotRun,
            "0.1.0",
            "2.18.5",
        );
        let csv = report.to_csv();
        assert!(csv.starts_with(CSV_HEADER));
        assert!(csv.contains("\"Label, \"\"quoted\"\"\""));
        assert!(csv.contains("\"line one\nline two\""));
        assert!(csv.contains("reason, with comma"));
    }

    #[test]
    fn html_is_self_contained_and_escapes_user_text() {
        let mut collector = ReportCollector::new();
        let mut element = element("1:9");
        element.name = "<script>alert(1)</script>".to_owned();
        element.path = vec!["Page 1".to_owned(), "<b>node</b>".to_owned()];
        collector.count_detected(&element);
        collector.emit(ReportEvent {
            code: "F2P-HTML".to_owned(),
            severity: Severity::Error,
            outcome: Outcome::Failed,
            element,
            description: "Bad <tag> & value".to_owned(),
            reason: None,
            action_taken: "skipped".to_owned(),
            suggested_action: Some("Fix & retry".to_owned()),
        });
        let report = collector.finish(
            "in <1>.fig",
            None,
            Duration::from_millis(7),
            ValidationState::Failed,
            "0.1.0",
            "2.18.5",
        );
        let html = report.to_html();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("Figma 2 Penpot"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("Bad &lt;tag&gt; &amp; value"));
        assert!(html.contains("Fix &amp; retry"));
        assert!(html.contains("F2P-HTML"));
        assert!(html.contains("in &lt;1&gt;.fig"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
    }

    #[test]
    fn batch_export_keeps_every_file_and_summary() {
        let collector = ReportCollector::new();
        let one = collector.finish(
            "one.fig",
            Some("one.penpot".to_owned()),
            Duration::ZERO,
            ValidationState::Passed,
            "0.1.0",
            "2.18.5",
        );
        let collector = ReportCollector::new();
        let two = collector.finish(
            "two.fig",
            None,
            Duration::ZERO,
            ValidationState::NotRun,
            "0.1.0",
            "2.18.5",
        );
        let html = batch_html(&[one.clone(), two.clone()]);
        assert!(html.contains("one.fig"));
        assert!(html.contains("two.fig"));
        let csv = batch_csv(&[one, two]);
        assert!(csv.lines().count() == 1);
    }

    #[test]
    fn json_is_versioned() {
        let collector = ReportCollector::new();
        let report = collector.finish(
            "in.fig",
            None,
            Duration::ZERO,
            ValidationState::Passed,
            "0.1.0",
            "2.18.5",
        );
        let json = report.to_json_pretty().unwrap();
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"validation\": \"passed\""));
    }
}
