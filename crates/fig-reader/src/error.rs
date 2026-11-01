/// Errors produced while reading a `.fig` file.
///
/// Every variant is data-bearing and actionable. The reader never panics on input.
#[derive(Debug, thiserror::Error)]
pub enum FigError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("not a Figma file: {0}")]
    NotFig(String),

    #[error("unsupported Figma format {0:?} (only fig-kiwi design files are supported)")]
    UnsupportedFormat(String),

    #[error("invalid ZIP container: {0}")]
    Zip(String),

    #[error("unsafe archive entry {entry:?}: {reason}")]
    UnsafeEntry { entry: String, reason: String },

    #[error("{context} exceeds the configured limit ({actual} > {limit})")]
    Limit {
        context: String,
        actual: u64,
        limit: u64,
    },

    #[error("truncated .fig payload: {0}")]
    Truncated(String),

    #[error("{context} could not be decompressed: {detail}")]
    Decompress { context: String, detail: String },

    #[error("invalid Kiwi schema: {0}")]
    Schema(String),

    #[error("invalid Kiwi message: {0}")]
    Decode(String),

    #[error("corrupt document: {0}")]
    Corrupt(String),

    #[error("internal reader error: {0}")]
    Internal(String),
}

impl FigError {
    pub(crate) fn limit(context: impl Into<String>, actual: usize, limit: usize) -> Self {
        Self::Limit {
            context: context.into(),
            actual: actual as u64,
            limit: limit as u64,
        }
    }

    pub(crate) fn limit_u64(context: impl Into<String>, actual: u64, limit: u64) -> Self {
        Self::Limit {
            context: context.into(),
            actual,
            limit,
        }
    }
}

impl From<zip::result::ZipError> for FigError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value.to_string())
    }
}

/// Result alias for reader operations.
pub type Result<T, E = FigError> = std::result::Result<T, E>;
