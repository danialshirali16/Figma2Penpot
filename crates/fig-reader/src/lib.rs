#![forbid(unsafe_code)]
//! Safe reader for Figma `.fig` files.
//!
//! The crate detects the ZIP and raw container forms, enforces resource limits,
//! decodes the embedded Kiwi schema, and extracts the flat node list without
//! trusting any length field. It intentionally does **not** convert to Penpot;
//! that is `conversion-core`'s job.
//!
//! ```no_run
//! let document = fig_reader::read_fig("design.fig")?;
//! println!("{} nodes, {} pages", document.stats.node_count, document.pages.len());
//! # Ok::<(), fig_reader::FigError>(())
//! ```

mod container;
mod document;
mod error;
mod kiwi;
pub mod limits;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub use document::{
    is_known_node_type, Blob, Color, ContainerKind, FigDocument, FigNode, Guid, ImageEntry, Matrix,
    Meta, PageInfo, Paint, ParseWarning, Size, Stats, TextData, VectorGeometry,
};
pub use error::{FigError, Result};
pub use limits::Limits;

/// Version of this reader, used in reports and diagnostics.
pub const READER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Read a `.fig` file with default limits.
pub fn read_fig(path: impl AsRef<Path>) -> Result<FigDocument> {
    read_fig_with_limits(path, &Limits::default())
}

/// Read a `.fig` file with explicit limits.
pub fn read_fig_with_limits(path: impl AsRef<Path>, limits: &Limits) -> Result<FigDocument> {
    let path = path.as_ref();
    let source_name = path.display().to_string();
    let metadata = fs::metadata(path).map_err(|source| FigError::Io {
        path: source_name.clone(),
        source,
    })?;
    if metadata.len() > limits.max_input_bytes {
        return Err(FigError::limit_u64(
            "input file size",
            metadata.len(),
            limits.max_input_bytes,
        ));
    }
    let bytes = fs::read(path).map_err(|source| FigError::Io {
        path: source_name.clone(),
        source,
    })?;
    read_fig_bytes(&source_name, &bytes, limits)
}

/// Read `.fig` data already in memory. Useful for tests, fuzzing, and callers
/// that stream input themselves.
pub fn read_fig_bytes(source_name: &str, bytes: &[u8], limits: &Limits) -> Result<FigDocument> {
    if bytes.len() as u64 > limits.max_input_bytes {
        return Err(FigError::limit_u64(
            "input size",
            bytes.len() as u64,
            limits.max_input_bytes,
        ));
    }

    let container = container::read_container(bytes, limits)?;
    let payload = kiwi::decode_payload(&container.canvas, limits)?;
    let schema = kiwi::Schema::parse(&payload.schema, limits)?;
    let schema_defs = schema.defs.len();
    let (nodes, blobs) = kiwi::decode_message(&schema, &payload.message, limits)?;

    let mut seen = HashSet::with_capacity(nodes.len());
    for node in &nodes {
        if !seen.insert(node.guid) {
            return Err(FigError::Corrupt(format!(
                "duplicate node GUID {}",
                node.guid
            )));
        }
    }

    let mut warnings = container.warnings;
    let non_finite = nodes
        .iter()
        .filter(|node| {
            node.size.is_some_and(|size| !size.is_finite())
                || node.transform.is_some_and(|matrix| !matrix.is_finite())
        })
        .count();
    if non_finite > 0 {
        warnings.push(ParseWarning::new(
            "FIG-WARN-NONFINITE",
            format!("{non_finite} node(s) contain non-finite geometry"),
        ));
    }

    Ok(FigDocument::new(document::DocumentParts {
        source_name: source_name.to_owned(),
        container: container.kind,
        has_thumbnail: container.has_thumbnail,
        prelude: payload.prelude,
        version: payload.version,
        schema_defs,
        chunk_count: payload.chunk_count,
        nodes,
        blobs,
        images: container.images,
        meta: container.meta,
        warnings,
        message: payload.message,
    }))
}
