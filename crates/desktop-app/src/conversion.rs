//! Background conversion and analysis workers. The UI only receives events; it
//! never performs file parsing or archive work on the render thread.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use conversion_core::ConvertOptions;
use conversion_report::{FileStatus, ReportCollector, ValidationState};
use fig_reader::FigDocument;

use crate::model::{AnalysisSummary, ConversionFailure, ConversionOutcome, Stage};

#[derive(Debug, Clone)]
pub struct ConversionRequest {
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub force: bool,
}

#[derive(Debug)]
pub enum WorkerEvent {
    Stage {
        input: PathBuf,
        stage: Stage,
    },
    Finished(ConversionOutcome),
    Failed(ConversionFailure),
    AnalysisFinished {
        path: PathBuf,
        summary: AnalysisSummary,
    },
    AnalysisFailed {
        path: PathBuf,
        message: String,
    },
}

pub fn spawn_conversion(request: ConversionRequest, sender: Sender<WorkerEvent>) {
    thread::spawn(move || run_conversion(request, &sender));
}

pub fn spawn_analysis(path: PathBuf, sender: Sender<WorkerEvent>) {
    thread::spawn(move || {
        let event = match fig_reader::read_fig(&path) {
            Ok(document) => WorkerEvent::AnalysisFinished {
                path,
                summary: AnalysisSummary::from_document(&document),
            },
            Err(error) => WorkerEvent::AnalysisFailed {
                path,
                message: format!("{error:#}"),
            },
        };
        let _ = sender.send(event);
    });
}

/// Run one conversion on the current thread. Callers on the UI thread should
/// use [`spawn_conversion`] instead.
pub fn run_conversion(request: ConversionRequest, sender: &Sender<WorkerEvent>) {
    let input = request.input.clone();
    let stem = input
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "output".to_owned());
    let output = request.output_dir.join(format!("{stem}.penpot"));
    let report_path = request.output_dir.join(format!("{stem}.report.json"));

    let fail = |message: String, report: Option<conversion_report::FileReport>| {
        let _ = sender.send(WorkerEvent::Failed(ConversionFailure {
            input: input.clone(),
            output: Some(output.clone()),
            report_path: Some(report_path.clone()),
            report,
            message,
        }));
    };

    if let Err(error) = std::fs::create_dir_all(&request.output_dir) {
        fail(
            format!(
                "Could not create output directory {}: {error}",
                request.output_dir.display()
            ),
            None,
        );
        return;
    }
    if !request.force {
        if output.exists() {
            fail(
                format!(
                    "Output already exists: {}. Enable overwrite or choose another folder.",
                    output.display()
                ),
                None,
            );
            return;
        }
        if report_path.exists() {
            fail(
                format!(
                    "Report already exists: {}. Enable overwrite or choose another folder.",
                    report_path.display()
                ),
                None,
            );
            return;
        }
    }

    let stage = |stage: Stage| {
        let _ = sender.send(WorkerEvent::Stage {
            input: input.clone(),
            stage,
        });
    };

    stage(Stage::Reading);
    let document = match fig_reader::read_fig(&input) {
        Ok(document) => document,
        Err(error) => {
            fail(
                format!("Reading {} failed: {error:#}", input.display()),
                None,
            );
            return;
        }
    };

    stage(Stage::Converting);
    let started = Instant::now();
    let mut collector = ReportCollector::new();
    let penpot =
        match conversion_core::convert(&document, &ConvertOptions::default(), &mut collector) {
            Ok(penpot) => penpot,
            Err(error) => {
                fail(
                    format!("Converting {} failed: {error:#}", input.display()),
                    None,
                );
                return;
            }
        };

    stage(Stage::Writing);
    let bytes = match penpot_writer::build_archive(&penpot) {
        Ok(bytes) => bytes,
        Err(error) => {
            let mut report = collector.finish(
                input.display().to_string(),
                Some(output.display().to_string()),
                started.elapsed(),
                ValidationState::NotRun,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            report.status = FileStatus::Failed;
            let _ = write_report(&report_path, &report);
            fail(
                format!("Building the Penpot archive failed: {error}"),
                Some(report),
            );
            return;
        }
    };

    stage(Stage::Validating);
    let validation = match penpot_writer::validate_archive(&bytes) {
        Ok(validation) => validation,
        Err(error) => {
            let mut report = collector.finish(
                input.display().to_string(),
                Some(output.display().to_string()),
                started.elapsed(),
                ValidationState::Failed,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            report.status = FileStatus::Failed;
            let _ = write_report(&report_path, &report);
            fail(
                format!("Generated archive failed validation: {error}"),
                Some(report),
            );
            return;
        }
    };

    stage(Stage::Saving);
    let write_result = penpot_writer::write_atomic(&output, &bytes);
    let mut report = collector.finish(
        input.display().to_string(),
        Some(output.display().to_string()),
        started.elapsed(),
        ValidationState::Passed,
        env!("CARGO_PKG_VERSION"),
        penpot_writer::TARGET_PENPOT,
    );
    if let Err(error) = write_result {
        report.status = FileStatus::Failed;
        let _ = write_report(&report_path, &report);
        fail(
            format!("Writing {} failed: {error}", output.display()),
            Some(report),
        );
        return;
    }
    if let Err(error) = write_report(&report_path, &report) {
        fail(
            format!("Writing report {} failed: {error}", report_path.display()),
            Some(report),
        );
        return;
    }

    let _ = sender.send(WorkerEvent::Finished(ConversionOutcome {
        input,
        output,
        report_path,
        report,
        validation_checks: validation.checks,
    }));
}

fn write_report(path: &Path, report: &conversion_report::FileReport) -> Result<()> {
    let json = report.to_json_pretty().context("serializing report")?;
    penpot_writer::write_atomic(path, json.as_bytes())
        .with_context(|| format!("writing report {}", path.display()))
}

/// Parse a document on the current thread and return its verified metadata.
/// This is exposed for tests and for callers that already have a worker.
pub fn analyze_document(document: &FigDocument) -> AnalysisSummary {
    AnalysisSummary::from_document(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name)
    }

    fn drain(rx: mpsc::Receiver<WorkerEvent>) -> Vec<WorkerEvent> {
        rx.try_iter().collect()
    }

    #[test]
    fn converts_a_fixture_and_writes_both_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = mpsc::channel();
        run_conversion(
            ConversionRequest {
                input: fixture("circle.fig"),
                output_dir: dir.path().to_path_buf(),
                force: false,
            },
            &tx,
        );
        let events = drain(rx);
        assert!(events.iter().any(|event| matches!(
            event,
            WorkerEvent::Stage {
                stage: Stage::Reading,
                ..
            }
        )));
        let finished = events
            .iter()
            .find_map(|event| match event {
                WorkerEvent::Finished(outcome) => Some(outcome),
                _ => None,
            })
            .expect("finished event");
        assert_eq!(finished.output, dir.path().join("circle.penpot"));
        assert!(finished.output.exists());
        assert!(finished.report_path.exists());
        assert!(finished.validation_checks > 0);
        assert_ne!(finished.report.status, FileStatus::Failed);
        let report = std::fs::read_to_string(&finished.report_path).unwrap();
        assert!(report.contains("\"schemaVersion\": 1"));
    }

    #[test]
    fn refuses_to_overwrite_without_permission() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("circle.penpot");
        std::fs::write(&output, b"existing").unwrap();
        let (tx, rx) = mpsc::channel();
        run_conversion(
            ConversionRequest {
                input: fixture("circle.fig"),
                output_dir: dir.path().to_path_buf(),
                force: false,
            },
            &tx,
        );
        let events = drain(rx);
        assert!(matches!(events.last(), Some(WorkerEvent::Failed(_))));
        assert_eq!(std::fs::read(&output).unwrap(), b"existing");
    }

    #[test]
    fn analysis_reports_document_metadata() {
        let document = fig_reader::read_fig(fixture("circle.fig")).unwrap();
        let summary = analyze_document(&document);
        assert!(summary.pages >= 1);
        assert!(summary.nodes > 0);
        assert_eq!(summary.version, 101);
    }
}
