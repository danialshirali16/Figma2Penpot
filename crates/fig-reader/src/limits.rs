//! Resource limits for reading untrusted `.fig` files.
//!
//! Defaults are deliberately generous for real design files but bounded. They are
//! configurable so tests can use tiny limits and future large-file work can raise them
//! deliberately.

/// Hard ceilings applied while reading a `.fig` file.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Maximum size of the input file on disk.
    pub max_input_bytes: u64,
    /// Maximum number of entries in a ZIP-wrapped `.fig`.
    pub max_zip_entries: usize,
    /// Maximum uncompressed size of `canvas.fig`.
    pub max_canvas_bytes: usize,
    /// Maximum decompressed size of the embedded Kiwi schema.
    pub max_schema_bytes: usize,
    /// Maximum decompressed size of the Kiwi message.
    pub max_message_bytes: usize,
    /// Maximum total uncompressed size declared by all ZIP entries.
    pub max_total_uncompressed: u64,
    /// Maximum uncompressed/compressed ratio for a ZIP entry.
    pub max_compression_ratio: u64,
    /// Maximum number of length-prefixed chunks in `canvas.fig`.
    pub max_chunks: usize,
    /// Maximum number of nodes accepted from a message.
    pub max_nodes: usize,
    /// Maximum number of blobs accepted from a message.
    pub max_blobs: usize,
    /// Maximum size of a single blob.
    pub max_blob_bytes: usize,
    /// Maximum element count for a non-byte Kiwi array.
    pub max_array_items: usize,
    /// Maximum Kiwi nesting depth (schema types and messages).
    pub max_depth: usize,
    /// Maximum bytes in a schema/definition/field name.
    pub max_schema_name_bytes: usize,
    /// Maximum bytes in a node name read by this crate.
    pub max_name_bytes: usize,
    /// Maximum bytes in a text run's characters.
    pub max_text_bytes: usize,
    /// Maximum number of schema definitions.
    pub max_schema_defs: usize,
    /// Maximum total number of schema fields.
    pub max_schema_fields: usize,
    /// Maximum zstd window the decoder may allocate.
    pub max_zstd_window: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_bytes: 1024 * 1024 * 1024,
            max_zip_entries: 10_000,
            max_canvas_bytes: 512 * 1024 * 1024,
            max_schema_bytes: 16 * 1024 * 1024,
            max_message_bytes: 512 * 1024 * 1024,
            max_total_uncompressed: 4 * 1024 * 1024 * 1024,
            max_compression_ratio: 1_000,
            max_chunks: 16,
            max_nodes: 1_000_000,
            max_blobs: 1_000_000,
            max_blob_bytes: 256 * 1024 * 1024,
            max_array_items: 4_000_000,
            max_depth: 64,
            max_schema_name_bytes: 4_096,
            max_name_bytes: 1024 * 1024,
            max_text_bytes: 16 * 1024 * 1024,
            max_schema_defs: 10_000,
            max_schema_fields: 200_000,
            max_zstd_window: 128 * 1024 * 1024,
        }
    }
}

impl Limits {
    /// Small limits for parser fuzzing and negative tests.
    pub fn for_fuzzing() -> Self {
        Self {
            max_input_bytes: 1024 * 1024,
            max_zip_entries: 128,
            max_canvas_bytes: 512 * 1024,
            max_schema_bytes: 256 * 1024,
            max_message_bytes: 1024 * 1024,
            max_total_uncompressed: 2 * 1024 * 1024,
            max_compression_ratio: 100,
            max_chunks: 4,
            max_nodes: 2_000,
            max_blobs: 2_000,
            max_blob_bytes: 256 * 1024,
            max_array_items: 4_000,
            max_depth: 32,
            max_schema_name_bytes: 256,
            max_name_bytes: 64 * 1024,
            max_text_bytes: 256 * 1024,
            max_schema_defs: 2_000,
            max_schema_fields: 20_000,
            max_zstd_window: 8 * 1024 * 1024,
        }
    }
}
