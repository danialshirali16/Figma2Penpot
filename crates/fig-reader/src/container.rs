use std::io::{Cursor, Read};

use crate::document::{ContainerKind, ImageEntry, Meta, ParseWarning};
use crate::error::{FigError, Result};
use crate::limits::Limits;

const META_MAX_BYTES: u64 = 1024 * 1024;

pub(crate) struct ContainerData {
    pub kind: ContainerKind,
    pub canvas: Vec<u8>,
    pub images: Vec<ImageEntry>,
    pub meta: Option<Meta>,
    pub has_thumbnail: bool,
    pub warnings: Vec<ParseWarning>,
}

/// Detect and read the outer container. No entry is ever extracted by name to disk.
pub(crate) fn read_container(bytes: &[u8], limits: &Limits) -> Result<ContainerData> {
    if bytes.starts_with(b"PK") {
        read_zip(bytes, limits)
    } else if bytes.starts_with(b"fig-") {
        read_raw(bytes, limits)
    } else {
        Err(FigError::NotFig(format!(
            "expected a ZIP archive or a fig-kiwi payload, found {}",
            hex_prefix(bytes)
        )))
    }
}

fn read_raw(bytes: &[u8], limits: &Limits) -> Result<ContainerData> {
    if bytes.len() > limits.max_canvas_bytes {
        return Err(FigError::limit(
            "raw canvas payload",
            bytes.len(),
            limits.max_canvas_bytes,
        ));
    }
    Ok(ContainerData {
        kind: ContainerKind::Raw,
        canvas: bytes.to_vec(),
        images: Vec::new(),
        meta: None,
        has_thumbnail: false,
        warnings: Vec::new(),
    })
}

fn read_zip(bytes: &[u8], limits: &Limits) -> Result<ContainerData> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let entry_count = archive.len();
    if entry_count > limits.max_zip_entries {
        return Err(FigError::limit(
            "ZIP entry count",
            entry_count,
            limits.max_zip_entries,
        ));
    }

    let mut warnings = Vec::new();
    let mut images = Vec::new();
    let mut image_total: u64 = 0;
    let mut total_uncompressed: u64 = 0;
    let mut canvas_index: Option<usize> = None;
    let mut meta_index: Option<usize> = None;
    let mut thumbnail_size: Option<u64> = None;
    let mut unknown_entries = 0usize;
    let mut image_name_warnings = 0usize;

    for index in 0..entry_count {
        let mut file = archive.by_index(index)?;
        let name = file
            .name()
            .map_err(|error| FigError::Zip(error.to_string()))?
            .into_owned();
        validate_entry_name(&name)?;

        if file.is_dir() {
            continue;
        }

        let size = file.size();
        let compressed = file.compressed_size();
        total_uncompressed = total_uncompressed
            .checked_add(size)
            .ok_or_else(|| FigError::Corrupt("ZIP uncompressed size overflow".into()))?;
        if total_uncompressed > limits.max_total_uncompressed {
            return Err(FigError::limit_u64(
                "total ZIP uncompressed size",
                total_uncompressed,
                limits.max_total_uncompressed,
            ));
        }
        check_ratio(&name, size, compressed, limits)?;

        match name.as_str() {
            "canvas.fig" => {
                if canvas_index.replace(index).is_some() {
                    return Err(FigError::Corrupt(
                        "ZIP contains more than one canvas.fig entry".into(),
                    ));
                }
            }
            "meta.json" => meta_index = Some(index),
            "thumbnail.png" => thumbnail_size = Some(size),
            _ if name.starts_with("images/") => {
                let hash = &name["images/".len()..];
                if hash.is_empty() {
                    unknown_entries += 1;
                    continue;
                }
                if !is_sha1_hex(hash) {
                    image_name_warnings += 1;
                }
                if size > limits.max_image_bytes as u64 {
                    return Err(FigError::limit_u64(
                        "image entry size",
                        size,
                        limits.max_image_bytes as u64,
                    ));
                }
                let bytes = read_limited(&mut file, limits.max_image_bytes, &name)?;
                image_total = image_total
                    .checked_add(bytes.len() as u64)
                    .ok_or_else(|| FigError::Corrupt("image size overflow".into()))?;
                if image_total > limits.max_total_image_bytes {
                    return Err(FigError::limit_u64(
                        "total image bytes",
                        image_total,
                        limits.max_total_image_bytes,
                    ));
                }
                images.push(ImageEntry {
                    name: hash.to_owned(),
                    size: bytes.len() as u64,
                    bytes,
                });
            }
            _ => unknown_entries += 1,
        }
    }

    let Some(canvas_index) = canvas_index else {
        return Err(FigError::Corrupt(
            "ZIP does not contain canvas.fig".to_owned(),
        ));
    };

    let canvas = {
        let mut file = archive.by_index(canvas_index)?;
        if file.size() > limits.max_canvas_bytes as u64 {
            return Err(FigError::limit_u64(
                "canvas.fig uncompressed size",
                file.size(),
                limits.max_canvas_bytes as u64,
            ));
        }
        read_limited(&mut file, limits.max_canvas_bytes, "canvas.fig")?
    };

    let meta = match meta_index {
        Some(index) => {
            let mut file = archive.by_index(index)?;
            let bytes = read_limited_u64(&mut file, META_MAX_BYTES, "meta.json")?;
            match parse_meta(&bytes) {
                Ok(meta) => Some(meta),
                Err(message) => {
                    warnings.push(ParseWarning::new("FIG-WARN-META", message));
                    None
                }
            }
        }
        None => None,
    };

    if unknown_entries > 0 {
        warnings.push(ParseWarning::new(
            "FIG-WARN-UNKNOWN-ENTRY",
            format!(
                "{unknown_entries} unrecognized ZIP entr{} skipped",
                if unknown_entries == 1 { "y" } else { "ies" }
            ),
        ));
    }
    if image_name_warnings > 0 {
        warnings.push(ParseWarning::new(
            "FIG-WARN-IMAGE-NAME",
            format!(
                "{image_name_warnings} image entr{} do not use a 40-character SHA-1 name",
                if image_name_warnings == 1 { "y" } else { "ies" }
            ),
        ));
    }

    Ok(ContainerData {
        kind: ContainerKind::Zip,
        canvas,
        images,
        meta,
        has_thumbnail: thumbnail_size.is_some(),
        warnings,
    })
}

fn check_ratio(name: &str, size: u64, compressed: u64, limits: &Limits) -> Result<()> {
    if size == 0 {
        return Ok(());
    }
    if compressed == 0 {
        return Err(FigError::Zip(format!(
            "entry {name:?} declares {size} uncompressed bytes but no compressed data"
        )));
    }
    let limit = limits.max_compression_ratio;
    if (size as u128) > (compressed as u128) * (limit as u128) {
        return Err(FigError::Limit {
            context: format!("compression ratio of {name:?}"),
            actual: size / compressed.max(1),
            limit,
        });
    }
    Ok(())
}

fn read_limited(reader: &mut impl Read, max: usize, context: &str) -> Result<Vec<u8>> {
    read_limited_u64(reader, max as u64, context)
}

fn read_limited_u64(reader: &mut impl Read, max: u64, context: &str) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let read = reader
        .take(max.saturating_add(1))
        .read_to_end(&mut out)
        .map_err(|error| FigError::Zip(format!("cannot read {context}: {error}")))?;
    if read as u64 > max {
        return Err(FigError::limit_u64(
            format!("{context} size"),
            read as u64,
            max,
        ));
    }
    Ok(out)
}

fn parse_meta(bytes: &[u8]) -> std::result::Result<Meta, String> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("meta.json is not valid JSON: {error}"))?;
    let file_name = value
        .get("file_name")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let version = value.get("version").and_then(|value| {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|number| number.to_string()))
    });
    Ok(Meta { file_name, version })
}

/// Validate an archive entry name without extracting it.
///
/// Rejects absolute paths, parent traversal, Windows separators/drives, NUL bytes,
/// control characters, and overly long names. This is defense in depth: the reader
/// never writes entries by name.
pub(crate) fn validate_entry_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(unsafe_entry(name, "empty name"));
    }
    if name.len() > 1024 {
        return Err(unsafe_entry(name, "name is longer than 1024 bytes"));
    }
    if name.contains('\0') {
        return Err(unsafe_entry(name, "contains a NUL byte"));
    }
    if name.contains('\\') {
        return Err(unsafe_entry(name, "contains a backslash"));
    }
    if name.starts_with('/') {
        return Err(unsafe_entry(name, "is absolute"));
    }
    if name.chars().any(|ch| ch.is_control() && ch != '\t') {
        return Err(unsafe_entry(name, "contains a control character"));
    }
    if is_windows_drive(name) {
        return Err(unsafe_entry(name, "starts with a Windows drive prefix"));
    }
    let mut components = name.split('/').peekable();
    while let Some(component) = components.next() {
        if component == ".." {
            return Err(unsafe_entry(name, "contains a parent traversal component"));
        }
        if component == "." {
            return Err(unsafe_entry(name, "contains a current-directory component"));
        }
        if component.is_empty() && components.peek().is_some() {
            return Err(unsafe_entry(name, "contains an empty path component"));
        }
    }
    Ok(())
}

fn is_windows_drive(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn unsafe_entry(entry: &str, reason: &str) -> FigError {
    FigError::UnsafeEntry {
        entry: entry.to_owned(),
        reason: reason.to_owned(),
    }
}

fn is_sha1_hex(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn hex_prefix(bytes: &[u8]) -> String {
    let prefix: Vec<String> = bytes
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if bytes.is_empty() {
        "no bytes".to_owned()
    } else {
        format!("0x{}", prefix.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_names() {
        assert!(validate_entry_name("images/../secret").is_err());
        assert!(validate_entry_name("../canvas.fig").is_err());
        assert!(validate_entry_name("/etc/passwd").is_err());
        assert!(validate_entry_name("C:/temp/x").is_err());
        assert!(validate_entry_name("a\\b").is_err());
        assert!(validate_entry_name("a\0b").is_err());
        assert!(validate_entry_name("a//b").is_err());
    }

    #[test]
    fn accepts_normal_names() {
        for name in ["canvas.fig", "meta.json", "images/abc", "images/"] {
            assert!(validate_entry_name(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn sha1_names_are_lowercase_and_exact() {
        assert!(is_sha1_hex("780960f6236bd1305ceeb2590ca395e36e705816"));
        assert!(!is_sha1_hex("780960f6236bd1305ceeb2590ca395e36e70581"));
        assert!(!is_sha1_hex("780960F6236BD1305CEEB2590CA395E36E705816"));
        assert!(!is_sha1_hex("not-a-hash"));
    }
}
