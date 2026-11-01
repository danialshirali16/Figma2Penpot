#![forbid(unsafe_code)]
//! Penpot document model, `.penpot` v3 archive writer, and structural validator.
//!
//! The writer targets Penpot 2.18.5 / file data version 67. It builds the archive
//! in memory, validates the bytes structurally, then callers may write it
//! atomically with [`write_atomic`] or [`write_penpot_atomic`].

mod archive;
mod model;
mod validate;

pub use archive::{build_archive, write_atomic, write_penpot_atomic, WriteError};
pub use model::{
    FileOptions, Fill, PenpotDocument, PenpotFile, PenpotPage, Point, Rect, Shape, ShapeType,
    Stroke, TextContent, TextParagraph, TextParagraphSet, TextRun, Transform, FEATURES, MIGRATIONS,
    PENPOT_FILE_VERSION, PENPOT_MANIFEST_VERSION, TARGET_PENPOT,
};
pub use validate::{validate_archive, ValidationError, ValidationReport};
