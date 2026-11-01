#![forbid(unsafe_code)]
//! Event-based conversion reports.
//!
//! Converters emit a [`ReportEvent`] at every decision point through a
//! [`DiagnosticSink`]. Counts are derived from those events and a real traversal,
//! never estimated from the produced file.

use std::collections::HashMap;
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
