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
    BackgroundBlur, Blur, ColorStyle, ComponentEntry, Content, FileOptions, Fill, GradientFill,
    GradientStop, ImageFill, LayoutGap, LayoutPadding, Media, MediaObject, PathCommand, PathParams,
    PenpotDocument, PenpotFile, PenpotPage, Point, Rect, Shadow, ShadowColor, Shape, ShapeType,
    Stroke, SvgAttrs, TextContent, TextParagraph, TextParagraphSet, TextRun, Transform,
    TypographyStyle, VariantProperty, FEATURES, MIGRATIONS, PENPOT_FILE_VERSION,
    PENPOT_MANIFEST_VERSION, TARGET_PENPOT,
};
pub use validate::{
    is_valid_blend_mode, is_valid_token_name, validate_archive, validate_archive_with_limits,
    ValidationError, ValidationLimits, ValidationReport,
};
