use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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
    }
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
