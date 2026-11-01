use std::io::Read;

use crate::error::{FigError, Result};
use crate::limits::Limits;

#[derive(Debug)]
pub(crate) struct PayloadParts {
    pub prelude: String,
    pub version: u32,
    pub chunk_count: usize,
    pub schema: Vec<u8>,
    pub message: Vec<u8>,
}

/// Parse the `fig-kiwi` header, length-prefixed chunks, and decompress the
/// schema and message with bounded buffers.
pub(crate) fn decode_payload(canvas: &[u8], limits: &Limits) -> Result<PayloadParts> {
    if canvas.len() < 12 {
        return Err(FigError::Truncated(format!(
            "header needs 12 bytes, found {}",
            canvas.len()
        )));
    }

    let prelude_bytes = &canvas[0..8];
    let prelude = std::str::from_utf8(prelude_bytes)
        .map_err(|_| FigError::NotFig("prelude is not valid ASCII".into()))?
        .trim_end_matches(' ')
        .to_owned();
    if !prelude.starts_with("fig-") {
        return Err(FigError::NotFig(format!("unknown prelude {prelude:?}")));
    }
    if prelude != "fig-kiwi" {
        return Err(FigError::UnsupportedFormat(prelude));
    }

    let version = u32::from_le_bytes(
        canvas[8..12]
            .try_into()
            .map_err(|_| FigError::Internal("version slice has wrong length".into()))?,
    );

    let mut offset = 12usize;
    let mut chunks: Vec<&[u8]> = Vec::new();
    while offset < canvas.len() {
        if offset + 4 > canvas.len() {
            return Err(FigError::Truncated(format!(
                "{} byte(s) left at offset {offset}, expected a 4-byte chunk length",
                canvas.len() - offset
            )));
        }
        let len = u32::from_le_bytes(
            canvas[offset..offset + 4]
                .try_into()
                .map_err(|_| FigError::Internal("chunk length slice has wrong length".into()))?,
        ) as usize;
        offset += 4;
        let end = offset.checked_add(len).ok_or_else(|| {
            FigError::Truncated(format!("chunk length overflows at offset {offset}"))
        })?;
        if end > canvas.len() {
            return Err(FigError::Truncated(format!(
                "chunk {} declares {len} bytes at offset {offset} but only {} remain",
                chunks.len(),
                canvas.len() - offset
            )));
        }
        chunks.push(&canvas[offset..end]);
        offset = end;
        if chunks.len() > limits.max_chunks {
            return Err(FigError::limit(
                "chunk count",
                chunks.len(),
                limits.max_chunks,
            ));
        }
    }

    if chunks.len() < 2 {
        return Err(FigError::Truncated(format!(
            "expected at least 2 chunks, found {}",
            chunks.len()
        )));
    }

    let schema = inflate_limited(chunks[0], limits.max_schema_bytes, "Kiwi schema")?;
    let message = if chunks[1].starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        zstd_limited(chunks[1], limits.max_message_bytes, limits.max_zstd_window)?
    } else {
        inflate_limited(chunks[1], limits.max_message_bytes, "Kiwi message")?
    };

    Ok(PayloadParts {
        prelude,
        version,
        chunk_count: chunks.len(),
        schema,
        message,
    })
}

fn inflate_limited(input: &[u8], max: usize, context: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    flate2::read::DeflateDecoder::new(input)
        .take(max.saturating_add(1) as u64)
        .read_to_end(&mut output)
        .map_err(|error| FigError::Decompress {
            context: context.to_owned(),
            detail: error.to_string(),
        })?;
    if output.len() > max {
        return Err(FigError::limit(context, output.len(), max));
    }
    Ok(output)
}

fn zstd_limited(input: &[u8], max: usize, max_window: u64) -> Result<Vec<u8>> {
    let decoder = ruzstd::decoding::StreamingDecoder::new_with_max_window_size(input, max_window)
        .map_err(|error| FigError::Decompress {
        context: "Kiwi message (zstd)".to_owned(),
        detail: error.to_string(),
    })?;
    let mut output = Vec::new();
    decoder
        .take(max.saturating_add(1) as u64)
        .read_to_end(&mut output)
        .map_err(|error| FigError::Decompress {
            context: "Kiwi message (zstd)".to_owned(),
            detail: error.to_string(),
        })?;
    if output.len() > max {
        return Err(FigError::limit("Kiwi message (zstd)", output.len(), max));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_header() {
        let error = decode_payload(&[0; 4], &Limits::default()).unwrap_err();
        assert!(matches!(error, FigError::Truncated(_)));
    }

    #[test]
    fn rejects_unknown_prelude() {
        let mut bytes = vec![0u8; 12];
        bytes[..8].copy_from_slice(b"fig-jam.");
        let error = decode_payload(&bytes, &Limits::default()).unwrap_err();
        assert!(matches!(error, FigError::UnsupportedFormat(_)));
    }

    #[test]
    fn rejects_chunk_past_end() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&101u32.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        let error = decode_payload(&bytes, &Limits::default()).unwrap_err();
        assert!(matches!(error, FigError::Truncated(_)));
    }
}
