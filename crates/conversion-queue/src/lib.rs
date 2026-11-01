#![forbid(unsafe_code)]
//! Bounded batch conversion jobs.
//!
//! The queue is deliberately UI-free. It owns job isolation, bounded worker
//! concurrency, real stage events, cancellation checkpoints, and per-job
//! results. A panic in one job is contained and reported as that job's failure.

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use conversion_core::ConvertOptions;
use conversion_report::{FileReport, FileStatus, ReportCollector, ValidationState};

/// Two workers is intentional for an 8 GiB machine: each conversion holds one
/// decoded document. Callers may raise it explicitly.
pub const DEFAULT_MAX_WORKERS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueConfig {
    pub max_workers: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_workers: DEFAULT_MAX_WORKERS,
        }
    }
}

impl QueueConfig {
    pub fn bounded(max_workers: usize) -> Self {
        Self {
            max_workers: max_workers.max(1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Reading,
    Converting,
    Writing,
    Validating,
    Saving,
}

#[derive(Debug, Clone)]
pub struct JobSpec {
    pub id: u64,
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct JobResult {
    pub id: u64,
    pub input: PathBuf,
    pub output: PathBuf,
    pub report_path: PathBuf,
    pub state: JobState,
    pub report: Option<FileReport>,
    pub validation_checks: usize,
    pub message: Option<String>,
}

impl JobResult {
    pub fn succeeded(&self) -> bool {
        self.state == JobState::Completed
    }
}

#[derive(Debug)]
pub enum QueueEvent {
    JobStarted { id: u64 },
    Stage { id: u64, stage: Stage },
    JobFinished { result: Box<JobResult> },
    BatchFinished,
}

#[derive(Clone)]
pub struct QueueControl {
    cancelled: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
}

impl QueueControl {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn pause(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}

pub struct BatchRunner {
    config: QueueConfig,
    jobs: Vec<JobSpec>,
    next_id: u64,
    control: QueueControl,
}

impl BatchRunner {
    pub fn new(config: QueueConfig) -> Self {
        Self {
            config,
            jobs: Vec::new(),
            next_id: 1,
            control: QueueControl {
                cancelled: Arc::new(AtomicBool::new(false)),
                paused: Arc::new(AtomicBool::new(false)),
            },
        }
    }

    pub fn control(&self) -> QueueControl {
        self.control.clone()
    }

    pub fn add_job(&mut self, input: PathBuf, output_dir: PathBuf, force: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(JobSpec {
            id,
            input,
            output_dir,
            force,
        });
        id
    }

    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }

    /// Spawn the batch and return an event stream plus a join handle carrying
    /// the final results.
    pub fn spawn(self) -> (Receiver<QueueEvent>, JoinHandle<Vec<JobResult>>) {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || self.run(sender));
        (receiver, handle)
    }

    /// Run the batch on the current thread and stream events.
    pub fn run(self, sender: Sender<QueueEvent>) -> Vec<JobResult> {
        if self.jobs.is_empty() {
            let _ = sender.send(QueueEvent::BatchFinished);
            return Vec::new();
        }

        let workers = self.config.max_workers.min(self.jobs.len()).max(1);
        let queue = Arc::new(Mutex::new(VecDeque::from(self.jobs)));
        let results = Arc::new(Mutex::new(Vec::new()));

        thread::scope(|scope| {
            for _ in 0..workers {
                let queue = Arc::clone(&queue);
                let results = Arc::clone(&results);
                let control = self.control.clone();
                let sender = sender.clone();
                scope.spawn(move || {
                    worker_loop(queue, results, control, sender);
                });
            }
        });

        // Jobs left queued because of cancellation still need a terminal result.
        let leftover: Vec<JobSpec> = {
            let mut queue = queue.lock().expect("queue mutex poisoned");
            queue.drain(..).collect()
        };
        for job in leftover {
            let result = cancelled_result(&job, None);
            let _ = sender.send(QueueEvent::JobFinished {
                result: Box::new(result.clone()),
            });
            results.lock().expect("results mutex poisoned").push(result);
        }

        let mut final_results = Arc::try_unwrap(results)
            .map(|mutex| mutex.into_inner().expect("results mutex poisoned"))
            .unwrap_or_else(|_| Vec::new());
        final_results.sort_by_key(|result| result.id);
        let _ = sender.send(QueueEvent::BatchFinished);
        final_results
    }
}

fn worker_loop(
    queue: Arc<Mutex<VecDeque<JobSpec>>>,
    results: Arc<Mutex<Vec<JobResult>>>,
    control: QueueControl,
    sender: Sender<QueueEvent>,
) {
    loop {
        if control.is_cancelled() {
            break;
        }
        while control.is_paused() {
            if control.is_cancelled() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let job = {
            let mut queue = queue.lock().expect("queue mutex poisoned");
            queue.pop_front()
        };
        let Some(job) = job else {
            break;
        };
        if control.is_cancelled() {
            let result = cancelled_result(&job, Some("cancelled before start"));
            let _ = sender.send(QueueEvent::JobFinished {
                result: Box::new(result.clone()),
            });
            results.lock().expect("results mutex poisoned").push(result);
            continue;
        }
        let _ = sender.send(QueueEvent::JobStarted { id: job.id });
        let job_for_fallback = job.clone();
        let result = match protect(|| run_job(job, &control, &sender)) {
            Ok(result) => result,
            Err(message) => failed_result(
                &job_for_fallback,
                None,
                None,
                format!("panic while processing this job: {message}"),
            ),
        };
        let _ = sender.send(QueueEvent::JobFinished {
            result: Box::new(result.clone()),
        });
        results.lock().expect("results mutex poisoned").push(result);
    }
}

fn protect<F>(run: F) -> std::result::Result<JobResult, String>
where
    F: FnOnce() -> JobResult,
{
    catch_unwind(AssertUnwindSafe(run)).map_err(panic_message)
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_owned()
    }
}

fn run_job(job: JobSpec, control: &QueueControl, sender: &Sender<QueueEvent>) -> JobResult {
    let stem = job
        .input
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "output".to_owned());
    let output = job.output_dir.join(format!("{stem}.penpot"));
    let report_path = job.output_dir.join(format!("{stem}.report.json"));

    let stage = |stage: Stage| {
        let _ = sender.send(QueueEvent::Stage { id: job.id, stage });
    };

    if control.is_cancelled() {
        return cancelled_result(&job, Some("cancelled before reading"));
    }
    if let Err(error) = std::fs::create_dir_all(&job.output_dir) {
        return failed_result(
            &job,
            Some(output),
            Some(report_path),
            format!(
                "Could not create output directory {}: {error}",
                job.output_dir.display()
            ),
        );
    }
    if !job.force {
        if output.exists() {
            return failed_result(
                &job,
                Some(output.clone()),
                Some(report_path.clone()),
                format!(
                    "Output already exists: {}. Enable overwrite or choose another folder.",
                    output.display()
                ),
            );
        }
        if report_path.exists() {
            return failed_result(
                &job,
                Some(output.clone()),
                Some(report_path.clone()),
                format!(
                    "Report already exists: {}. Enable overwrite or choose another folder.",
                    report_path.display()
                ),
            );
        }
    }

    stage(Stage::Reading);
    let document = match fig_reader::read_fig(&job.input) {
        Ok(document) => document,
        Err(error) => {
            return failed_result(
                &job,
                Some(output),
                Some(report_path),
                format!("Reading {} failed: {error:#}", job.input.display()),
            );
        }
    };
    if control.is_cancelled() {
        return cancelled_result(&job, Some("cancelled after reading"));
    }

    stage(Stage::Converting);
    let started = Instant::now();
    let mut collector = ReportCollector::new();
    let penpot =
        match conversion_core::convert(&document, &ConvertOptions::default(), &mut collector) {
            Ok(penpot) => penpot,
            Err(error) => {
                return failed_result(
                    &job,
                    Some(output),
                    Some(report_path),
                    format!("Converting {} failed: {error:#}", job.input.display()),
                );
            }
        };

    stage(Stage::Writing);
    let bytes = match penpot_writer::build_archive(&penpot) {
        Ok(bytes) => bytes,
        Err(error) => {
            let mut report = collector.finish(
                job.input.display().to_string(),
                Some(output.display().to_string()),
                started.elapsed(),
                ValidationState::NotRun,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            report.status = FileStatus::Failed;
            let _ = write_report(&report_path, &report);
            return failed_result(
                &job,
                Some(output),
                Some(report_path),
                format!("Building the Penpot archive failed: {error}"),
            );
        }
    };

    stage(Stage::Validating);
    let validation = match penpot_writer::validate_archive(&bytes) {
        Ok(validation) => validation,
        Err(error) => {
            let mut report = collector.finish(
                job.input.display().to_string(),
                Some(output.display().to_string()),
                started.elapsed(),
                ValidationState::Failed,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            report.status = FileStatus::Failed;
            let _ = write_report(&report_path, &report);
            return failed_result(
                &job,
                Some(output),
                Some(report_path),
                format!("Generated archive failed validation: {error}"),
            );
        }
    };
    if control.is_cancelled() {
        return cancelled_result(&job, Some("cancelled before writing"));
    }

    stage(Stage::Saving);
    if let Err(error) = penpot_writer::write_atomic(&output, &bytes) {
        let mut report = collector.finish(
            job.input.display().to_string(),
            Some(output.display().to_string()),
            started.elapsed(),
            ValidationState::Passed,
            env!("CARGO_PKG_VERSION"),
            penpot_writer::TARGET_PENPOT,
        );
        report.status = FileStatus::Failed;
        let _ = write_report(&report_path, &report);
        return failed_result(
            &job,
            Some(output.clone()),
            Some(report_path),
            format!("Writing {} failed: {error}", output.display()),
        );
    }

    let report = collector.finish(
        job.input.display().to_string(),
        Some(output.display().to_string()),
        started.elapsed(),
        ValidationState::Passed,
        env!("CARGO_PKG_VERSION"),
        penpot_writer::TARGET_PENPOT,
    );
    if let Err(error) = write_report(&report_path, &report) {
        return failed_result(
            &job,
            Some(output),
            Some(report_path),
            format!("Writing report failed: {error:#}"),
        );
    }

    JobResult {
        id: job.id,
        input: job.input,
        output,
        report_path,
        state: if report.status == FileStatus::Failed {
            JobState::Failed
        } else {
            JobState::Completed
        },
        report: Some(report),
        validation_checks: validation.checks,
        message: None,
    }
}

fn write_report(path: &Path, report: &FileReport) -> Result<()> {
    let json = report.to_json_pretty().context("serializing report")?;
    penpot_writer::write_atomic(path, json.as_bytes())
        .with_context(|| format!("writing report {}", path.display()))
}

fn failed_result(
    job: &JobSpec,
    output: Option<PathBuf>,
    report_path: Option<PathBuf>,
    message: String,
) -> JobResult {
    let stem = job
        .input
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "output".to_owned());
    JobResult {
        id: job.id,
        input: job.input.clone(),
        output: output.unwrap_or_else(|| job.output_dir.join(format!("{stem}.penpot"))),
        report_path: report_path
            .unwrap_or_else(|| job.output_dir.join(format!("{stem}.report.json"))),
        state: JobState::Failed,
        report: None,
        validation_checks: 0,
        message: Some(message),
    }
}

fn cancelled_result(job: &JobSpec, message: Option<&str>) -> JobResult {
    let mut result = failed_result(job, None, None, message.unwrap_or("cancelled").to_owned());
    result.state = JobState::Cancelled;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name)
    }

    fn collect(rx: Receiver<QueueEvent>) -> (Vec<JobResult>, bool) {
        let mut results = Vec::new();
        let mut finished = false;
        for event in rx {
            match event {
                QueueEvent::JobFinished { result } => results.push(*result),
                QueueEvent::BatchFinished => {
                    finished = true;
                    break;
                }
                QueueEvent::JobStarted { .. } | QueueEvent::Stage { .. } => {}
            }
        }
        (results, finished)
    }

    #[test]
    fn runs_a_mixed_batch_without_cross_contamination() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.fig");
        std::fs::write(&bad, b"not a fig").unwrap();
        let mut runner = BatchRunner::new(QueueConfig::bounded(2));
        runner.add_job(fixture("circle.fig"), dir.path().to_path_buf(), false);
        runner.add_job(bad.clone(), dir.path().to_path_buf(), false);
        runner.add_job(fixture("text-group.fig"), dir.path().to_path_buf(), false);
        let (rx, handle) = runner.spawn();
        let (mut events, finished) = collect(rx);
        let results = handle.join().unwrap();
        assert!(finished);
        assert_eq!(events.len(), 3);
        events.sort_by_key(|result| result.id);
        assert_eq!(results.len(), 3);
        assert!(results
            .iter()
            .any(|result| result.state == JobState::Failed));
        assert!(results.iter().any(|result| result.succeeded()));
        assert!(results.iter().any(|result| result
            .input
            .file_name()
            .is_some_and(|name| name == "text-group.fig")));
        assert!(dir.path().join("circle.penpot").exists());
        assert!(dir.path().join("text-group.penpot").exists());
    }

    #[test]
    fn cancellation_marks_queued_jobs_without_touching_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let mut runner = BatchRunner::new(QueueConfig::bounded(1));
        let control = runner.control();
        control.pause(true);
        for _ in 0..4 {
            runner.add_job(fixture("circle.fig"), dir.path().to_path_buf(), false);
        }
        let (rx, handle) = runner.spawn();
        control.cancel();
        control.pause(false);
        let (events, finished) = collect(rx);
        let results = handle.join().unwrap();
        assert!(finished);
        assert_eq!(events.len(), 4);
        assert_eq!(results.len(), 4);
        assert!(results
            .iter()
            .all(|result| result.state == JobState::Cancelled));
        assert!(!dir.path().join("circle.penpot").exists());
    }

    #[test]
    fn panic_is_contained_to_one_job() {
        let message = protect(|| panic!("deliberate panic"));
        assert_eq!(message.unwrap_err(), "deliberate panic");
    }

    #[test]
    fn overwrite_is_refused_without_force() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("circle.penpot"), b"keep me").unwrap();
        let mut runner = BatchRunner::new(QueueConfig::bounded(1));
        runner.add_job(fixture("circle.fig"), dir.path().to_path_buf(), false);
        let (rx, handle) = runner.spawn();
        let (_, _) = collect(rx);
        let results = handle.join().unwrap();
        assert_eq!(results[0].state, JobState::Failed);
        assert_eq!(
            std::fs::read(dir.path().join("circle.penpot")).unwrap(),
            b"keep me"
        );
    }
}
