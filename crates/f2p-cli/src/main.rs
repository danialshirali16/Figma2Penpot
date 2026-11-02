use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use conversion_core::ConvertOptions;
use conversion_report::{FileReport, ReportCollector, ValidationState};

/// Headless tools for Figma 2 Penpot.
#[derive(Debug, Parser)]
#[command(
    name = "f2p-cli",
    version,
    about = "Inspect and convert Figma .fig files for Penpot",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Read a .fig file and print verified metadata. No conversion is performed.
    Inspect {
        /// Path to a .fig file.
        file: PathBuf,
    },

    /// Convert several .fig files with the bounded batch queue.
    Batch {
        /// One or more .fig files.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output directory. File names are derived from the inputs.
        #[arg(short, long)]
        out: PathBuf,
        /// Overwrite existing .penpot and report files.
        #[arg(long)]
        force: bool,
        /// Maximum concurrent conversions.
        #[arg(long, default_value_t = conversion_queue::DEFAULT_MAX_WORKERS)]
        jobs: usize,
        /// Keep resolved token/style values but remove token links.
        #[arg(long)]
        inline_tokens: bool,
        /// Write a self-contained HTML batch report here.
        #[arg(long)]
        html: Option<PathBuf>,
        /// Write a CSV batch report here.
        #[arg(long)]
        csv: Option<PathBuf>,
        /// Write a batch JSON aggregate here.
        #[arg(long)]
        json: Option<PathBuf>,
    },

    /// Export an existing JSON report to HTML and/or CSV.
    Report {
        /// JSON report produced by convert or batch.
        report: PathBuf,
        /// Write self-contained HTML here.
        #[arg(long)]
        html: Option<PathBuf>,
        /// Write CSV here.
        #[arg(long)]
        csv: Option<PathBuf>,
    },

    /// Convert a .fig file to a .penpot file and a JSON report.
    Convert {
        /// Path to a .fig file.
        file: PathBuf,
        /// Output directory. The file name is derived from the input.
        #[arg(short, long)]
        out: PathBuf,
        /// Overwrite an existing .penpot or report file.
        #[arg(long)]
        force: bool,
        /// Override the report path.
        #[arg(long)]
        report: Option<PathBuf>,
        /// Keep resolved token/style values but remove token links.
        #[arg(long)]
        inline_tokens: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Inspect { file } => inspect(&file),
        Command::Convert {
            file,
            out,
            force,
            report,
            inline_tokens,
        } => convert(&file, &out, force, report.as_deref(), !inline_tokens),
        Command::Batch {
            files,
            out,
            force,
            jobs,
            inline_tokens,
            html,
            csv,
            json,
        } => batch(
            files,
            &out,
            force,
            jobs,
            !inline_tokens,
            html.as_deref(),
            csv.as_deref(),
            json.as_deref(),
        ),
        Command::Report { report, html, csv } => {
            export_report(&report, html.as_deref(), csv.as_deref())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn batch(
    files: Vec<PathBuf>,
    out_dir: &Path,
    force: bool,
    jobs: usize,
    keep_token_bindings: bool,
    html: Option<&Path>,
    csv: Option<&Path>,
    json: Option<&Path>,
) -> Result<()> {
    use conversion_queue::{BatchRunner, JobState, QueueConfig, QueueEvent};

    let mut runner = BatchRunner::new(QueueConfig::bounded(jobs));
    for file in files {
        runner.add_job(file, out_dir.to_path_buf(), force, keep_token_bindings);
    }
    let (receiver, handle) = runner.spawn();
    for event in receiver {
        match event {
            QueueEvent::JobStarted { id } => println!("[{id}] started"),
            QueueEvent::Stage { id, stage } => println!("[{id}] {stage:?}"),
            QueueEvent::JobFinished { result } => {
                let state = match result.state {
                    JobState::Completed => "completed",
                    JobState::Failed => "failed",
                    JobState::Cancelled => "cancelled",
                };
                println!("[{}] {state}: {}", result.id, result.input.display());
                if let Some(message) = &result.message {
                    println!("[{}] {message}", result.id);
                }
            }
            QueueEvent::BatchFinished => break,
        }
    }
    let results = handle
        .join()
        .map_err(|_| anyhow::anyhow!("batch worker panicked"))?;
    let failed = results
        .iter()
        .filter(|result| result.state == JobState::Failed)
        .count();
    let cancelled = results
        .iter()
        .filter(|result| result.state == JobState::Cancelled)
        .count();
    let completed = results.len().saturating_sub(failed + cancelled);
    println!("Batch: {completed} completed, {failed} failed, {cancelled} cancelled");
    let reports: Vec<conversion_report::FileReport> = results
        .iter()
        .filter_map(|result| result.report.clone())
        .collect();
    if let Some(path) = html {
        let document = conversion_report::batch_html(&reports);
        penpot_writer::write_atomic(path, document.as_bytes())
            .with_context(|| format!("writing HTML report {}", path.display()))?;
        println!("HTML report: {}", path.display());
    }
    if let Some(path) = csv {
        let document = conversion_report::batch_csv(&reports);
        penpot_writer::write_atomic(path, document.as_bytes())
            .with_context(|| format!("writing CSV report {}", path.display()))?;
        println!("CSV report: {}", path.display());
    }
    if let Some(path) = json {
        let document =
            conversion_report::batch_json(&reports).context("serializing batch JSON report")?;
        penpot_writer::write_atomic(path, document.as_bytes())
            .with_context(|| format!("writing JSON batch report {}", path.display()))?;
        println!("JSON batch report: {}", path.display());
    }
    if failed > 0 {
        anyhow::bail!("{failed} batch job(s) failed");
    }
    Ok(())
}

fn export_report(report_path: &Path, html: Option<&Path>, csv: Option<&Path>) -> Result<()> {
    if html.is_none() && csv.is_none() {
        anyhow::bail!("choose --html and/or --csv");
    }
    let text = std::fs::read_to_string(report_path)
        .with_context(|| format!("reading {}", report_path.display()))?;
    let report: FileReport = serde_json::from_str(&text)
        .with_context(|| format!("parsing {}", report_path.display()))?;
    if let Some(path) = html {
        penpot_writer::write_atomic(path, report.to_html().as_bytes())
            .with_context(|| format!("writing HTML report {}", path.display()))?;
        println!("HTML report: {}", path.display());
    }
    if let Some(path) = csv {
        penpot_writer::write_atomic(path, report.to_csv().as_bytes())
            .with_context(|| format!("writing CSV report {}", path.display()))?;
        println!("CSV report: {}", path.display());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn convert(
    path: &Path,
    out_dir: &Path,
    force: bool,
    report_override: Option<&Path>,
    keep_token_bindings: bool,
) -> Result<()> {
    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "output".to_owned());
    let out_path = out_dir.join(format!("{stem}.penpot"));
    let report_path = report_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| out_dir.join(format!("{stem}.report.json")));

    if !force {
        if out_path.exists() {
            anyhow::bail!(
                "output already exists: {} (use --force to overwrite)",
                out_path.display()
            );
        }
        if report_path.exists() {
            anyhow::bail!(
                "report already exists: {} (use --force to overwrite)",
                report_path.display()
            );
        }
    }
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output directory {}", out_dir.display()))?;

    let document =
        fig_reader::read_fig(path).with_context(|| format!("reading {}", path.display()))?;
    let started = Instant::now();
    let mut collector = ReportCollector::new();
    let options = ConvertOptions {
        keep_token_bindings,
        ..ConvertOptions::default()
    };
    let penpot = conversion_core::convert(&document, &options, &mut collector)
        .with_context(|| format!("converting {}", path.display()))?;

    let bytes = match penpot_writer::build_archive(&penpot) {
        Ok(bytes) => bytes,
        Err(error) => {
            let report = collector.finish(
                path.display().to_string(),
                None,
                started.elapsed(),
                ValidationState::NotRun,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            let mut report = report;
            report.status = conversion_report::FileStatus::Failed;
            write_report(&report_path, &report)?;
            return Err(error).context("building Penpot archive");
        }
    };

    let validation = match penpot_writer::validate_archive(&bytes) {
        Ok(validation) => validation,
        Err(error) => {
            let mut report = collector.finish(
                path.display().to_string(),
                Some(out_path.display().to_string()),
                started.elapsed(),
                ValidationState::Failed,
                env!("CARGO_PKG_VERSION"),
                penpot_writer::TARGET_PENPOT,
            );
            report.status = conversion_report::FileStatus::Failed;
            write_report(&report_path, &report)?;
            return Err(error).context("validating generated Penpot archive");
        }
    };

    let write_result = penpot_writer::write_atomic(&out_path, &bytes);
    let mut report = collector.finish(
        path.display().to_string(),
        Some(out_path.display().to_string()),
        started.elapsed(),
        ValidationState::Passed,
        env!("CARGO_PKG_VERSION"),
        penpot_writer::TARGET_PENPOT,
    );
    if let Err(error) = write_result {
        report.status = conversion_report::FileStatus::Failed;
        report.validation = ValidationState::Failed;
        write_report(&report_path, &report)?;
        return Err(error).context("writing output");
    }

    write_report(&report_path, &report)?;
    println!("Converted {} -> {}", path.display(), out_path.display());
    println!(
        "Status: {:?}; detected {}, converted {}, approximated {}, skipped {}, unsupported {}, failed {}",
        report.status,
        report.counts.detected,
        report.counts.converted,
        report.counts.approximated,
        report.counts.skipped,
        report.counts.unsupported,
        report.counts.failed
    );
    println!("Validation: passed ({} checks)", validation.checks);
    println!("Report: {}", report_path.display());
    Ok(())
}

fn write_report(path: &Path, report: &FileReport) -> Result<()> {
    let json = report.to_json_pretty().context("serializing report")?;
    penpot_writer::write_atomic(path, json.as_bytes())
        .with_context(|| format!("writing report {}", path.display()))
}

fn inspect(path: &Path) -> Result<()> {
    let document =
        fig_reader::read_fig(path).with_context(|| format!("reading {}", path.display()))?;

    let display_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    println!("File:            {display_name}");
    println!("Path:            {}", path.display());
    println!(
        "Document name:   {}",
        document
            .meta_file_name()
            .unwrap_or("Unknown (no meta.json)")
    );
    println!("Container:       {}", document.container);
    println!(
        "Format:          {} (version {})",
        document.prelude, document.version
    );
    println!(
        "Schema:          {} definitions; {} chunk(s)",
        document.schema_defs, document.chunk_count
    );
    println!(
        "Nodes:           {} total, {} active, {} removed",
        document.stats.node_count, document.stats.active_nodes, document.stats.removed_nodes
    );
    println!(
        "Blobs:           {} ({} bytes)",
        document.stats.blob_count, document.stats.blob_bytes
    );
    println!(
        "Images:          {} ({} bytes); thumbnail: {}",
        document.stats.image_count,
        document.stats.image_bytes,
        if document.has_thumbnail {
            "present"
        } else {
            "not present"
        }
    );

    let visible_pages = document
        .pages
        .iter()
        .filter(|page| !page.internal_only)
        .count();
    let internal_pages = document.pages.len().saturating_sub(visible_pages);
    println!(
        "Pages:           {} visible, {} internal",
        visible_pages, internal_pages
    );
    for page in &document.pages {
        let marker = if page.internal_only {
            " [internal]"
        } else {
            ""
        };
        println!(
            "  {}. {} ({}) — {} direct, {} total nodes{}",
            page.index + 1,
            if page.name.is_empty() {
                "Unnamed page"
            } else {
                &page.name
            },
            page.guid,
            page.direct_children,
            page.descendant_nodes,
            marker
        );
    }
    if document.pages.is_empty() {
        println!("  Unknown (no CANVAS nodes found)");
    }

    if !document.stats.node_types.is_empty() {
        let types = document
            .stats
            .node_types
            .iter()
            .map(|(kind, count)| format!("{kind} {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("Node types:      {types}");
    }
    if !document.stats.unknown_node_types.is_empty() {
        println!(
            "Unknown types:   {}",
            document
                .stats
                .unknown_node_types
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if document.stats.orphan_nodes > 0 {
        println!(
            "Orphan nodes:    {} (parent GUID not found)",
            document.stats.orphan_nodes
        );
    }
    println!("Max depth:       {}", document.stats.max_depth);

    if document.warnings.is_empty() {
        println!("Warnings:        none");
    } else {
        println!("Warnings:        {}", document.warnings.len());
        for warning in &document.warnings {
            println!("  {} — {}", warning.code, warning.message);
        }
    }

    Ok(())
}
