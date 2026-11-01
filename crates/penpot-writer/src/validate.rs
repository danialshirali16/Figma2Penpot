use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use serde_json::Value;
use uuid::Uuid;

use crate::model::{PENPOT_FILE_VERSION, PENPOT_MANIFEST_VERSION};

const MAX_ENTRIES: usize = 10_000;
const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("{0}")]
    Invalid(String),
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error("JSON error: {0}")]
    Json(String),
}

impl From<zip::result::ZipError> for ValidationError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationReport {
    pub checks: usize,
}

/// Validate a generated `.penpot` archive structurally.
///
/// This checks the rules we committed to in `docs/PENPOT_FORMAT.md`; it is not a
/// substitute for importing into a real Penpot instance.
pub fn validate_archive(bytes: &[u8]) -> Result<ValidationReport, ValidationError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    if archive.len() > MAX_ENTRIES {
        return Err(ValidationError::Invalid(format!(
            "archive has {} entries, limit is {MAX_ENTRIES}",
            archive.len()
        )));
    }

    let mut entries: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let name = file
            .name()
            .map_err(|error| ValidationError::Zip(error.to_string()))?
            .into_owned();
        validate_entry_name(&name)?;
        if file.is_dir() {
            continue;
        }
        if file.size() > MAX_ENTRY_BYTES {
            return Err(ValidationError::Invalid(format!(
                "entry {name:?} is larger than {MAX_ENTRY_BYTES} bytes"
            )));
        }
        let mut data = Vec::new();
        file.by_ref()
            .take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(|error| ValidationError::Zip(format!("{name}: {error}")))?;
        if data.len() as u64 > MAX_ENTRY_BYTES {
            return Err(ValidationError::Invalid(format!(
                "entry {name:?} exceeds the size limit"
            )));
        }
        total = total
            .checked_add(data.len() as u64)
            .ok_or_else(|| ValidationError::Invalid("archive size overflow".into()))?;
        if total > MAX_TOTAL_BYTES {
            return Err(ValidationError::Invalid(
                "archive exceeds the total size limit".into(),
            ));
        }
        if entries.insert(name.to_string(), data).is_some() {
            return Err(ValidationError::Invalid(format!(
                "duplicate ZIP entry {name:?}"
            )));
        }
    }

    let manifest_bytes = entries
        .get("manifest.json")
        .ok_or_else(|| ValidationError::Invalid("missing manifest.json".into()))?;
    let manifest = parse_json(manifest_bytes, "manifest.json")?;
    let manifest = require_object(Some(&manifest), "manifest")?;
    let version = require_u64(manifest.get("version"), "manifest.version")?;
    if version != PENPOT_MANIFEST_VERSION as u64 {
        return Err(ValidationError::Invalid(format!(
            "manifest.version is {version}, expected {PENPOT_MANIFEST_VERSION}"
        )));
    }
    if require_str(manifest.get("type"), "manifest.type")? != "penpot/export-files" {
        return Err(ValidationError::Invalid(
            "manifest.type must be penpot/export-files".into(),
        ));
    }
    let files = require_array(manifest.get("files"), "manifest.files")?;
    if files.is_empty() {
        return Err(ValidationError::Invalid(
            "manifest.files must not be empty".into(),
        ));
    }

    let mut checks = 0usize;
    for (index, file_value) in files.iter().enumerate() {
        let file = require_object(Some(file_value), &format!("manifest.files[{index}]"))?;
        let file_id = require_uuid(file.get("id"), "manifest file id")?;
        let file_name = require_str(file.get("name"), "manifest file name")?;
        let manifest_features = require_array(file.get("features"), "manifest file features")?;
        checks += 3;

        let file_path = format!("files/{file_id}.json");
        let file_bytes = entries
            .get(&file_path)
            .ok_or_else(|| ValidationError::Invalid(format!("missing {file_path}")))?;
        let file_json = parse_json(file_bytes, &file_path)?;
        let file_obj = require_object(Some(&file_json), &file_path)?;
        if require_uuid(file_obj.get("id"), "file.id")? != file_id {
            return Err(ValidationError::Invalid(format!(
                "{file_path}: id does not match its path"
            )));
        }
        if require_str(file_obj.get("name"), "file.name")? != file_name {
            return Err(ValidationError::Invalid(format!(
                "{file_path}: name does not match manifest"
            )));
        }
        let file_version = require_u64(file_obj.get("version"), "file.version")?;
        if file_version != PENPOT_FILE_VERSION as u64 {
            return Err(ValidationError::Invalid(format!(
                "{file_path}: version {file_version}, expected {PENPOT_FILE_VERSION}"
            )));
        }
        let file_features = require_array(file_obj.get("features"), "file.features")?;
        if file_features != manifest_features {
            return Err(ValidationError::Invalid(format!(
                "{file_path}: features do not match manifest"
            )));
        }
        checks += 4;

        let page_prefix = format!("files/{file_id}/pages/");
        let page_paths: Vec<&str> = entries
            .keys()
            .filter(|path| path.starts_with(&page_prefix) && path.ends_with(".json"))
            .map(String::as_str)
            .filter(|path| {
                let rest = &path[page_prefix.len()..];
                !rest.contains('/')
            })
            .collect();
        if page_paths.is_empty() {
            return Err(ValidationError::Invalid(format!(
                "file {file_name:?} has no pages"
            )));
        }

        let mut file_shape_ids = HashSet::new();
        for page_path in page_paths {
            let page_bytes = entries
                .get(page_path)
                .ok_or_else(|| ValidationError::Invalid(format!("missing {page_path}")))?;
            let page_json = parse_json(page_bytes, page_path)?;
            let page = require_object(Some(&page_json), page_path)?;
            let page_id = require_uuid(page.get("id"), "page.id")?;
            if !page_path.ends_with(&format!("/{page_id}.json")) {
                return Err(ValidationError::Invalid(format!(
                    "{page_path}: page id does not match path"
                )));
            }
            require_str(page.get("name"), "page.name")?;
            checks += 2;

            let shape_prefix = format!("files/{file_id}/pages/{page_id}/");
            let shape_paths: Vec<&str> = entries
                .keys()
                .filter(|path| path.starts_with(&shape_prefix) && path.ends_with(".json"))
                .map(String::as_str)
                .collect();
            if shape_paths.is_empty() {
                return Err(ValidationError::Invalid(format!(
                    "page {page_id} has no shapes"
                )));
            }
            let root_path = format!("{shape_prefix}{}.json", Uuid::nil());
            if !entries.contains_key(&root_path) {
                return Err(ValidationError::Invalid(format!(
                    "page {page_id} is missing the root frame"
                )));
            }

            let mut shapes: HashMap<Uuid, Value> = HashMap::new();
            for shape_path in shape_paths {
                let shape_bytes = entries
                    .get(shape_path)
                    .ok_or_else(|| ValidationError::Invalid(format!("missing {shape_path}")))?;
                let shape_json = parse_json(shape_bytes, shape_path)?;
                let shape = require_object(Some(&shape_json), shape_path)?;
                let shape_id = require_uuid(shape.get("id"), "shape.id")?;
                if !shape_path.ends_with(&format!("/{shape_id}.json")) {
                    return Err(ValidationError::Invalid(format!(
                        "{shape_path}: shape id does not match path"
                    )));
                }
                if !file_shape_ids.insert(shape_id) {
                    return Err(ValidationError::Invalid(format!(
                        "duplicate shape id {shape_id} in file"
                    )));
                }
                if require_uuid(shape.get("pageId"), "shape.pageId")? != page_id {
                    return Err(ValidationError::Invalid(format!(
                        "{shape_path}: pageId does not match page"
                    )));
                }
                shapes.insert(shape_id, shape_json);
            }

            let mut checks_for_page = 0usize;
            for (shape_id, shape_json) in &shapes {
                let shape = require_object(Some(shape_json), &format!("shape {shape_id}"))?;
                validate_shape(shape, *shape_id)?;
                for field in ["parentId", "frameId"] {
                    let reference = require_uuid(shape.get(field), field)?;
                    if !reference.is_nil() && !shapes.contains_key(&reference) {
                        return Err(ValidationError::Invalid(format!(
                            "shape {shape_id} references missing {field} {reference}"
                        )));
                    }
                }
                if let Some(children) = shape.get("shapes") {
                    for child in require_array(Some(children), "shape.shapes")? {
                        let child = require_uuid(Some(child), "shape child")?;
                        if !shapes.contains_key(&child) {
                            return Err(ValidationError::Invalid(format!(
                                "shape {shape_id} references missing child {child}"
                            )));
                        }
                    }
                }
                checks_for_page += 1;
            }
            if checks_for_page == 0 {
                return Err(ValidationError::Invalid(format!(
                    "page {page_id} has no valid shapes"
                )));
            }
            checks += checks_for_page;

            let root = shapes
                .get(&Uuid::nil())
                .ok_or_else(|| ValidationError::Invalid("root frame missing".into()))?;
            let root = require_object(Some(root), "root frame")?;
            if require_str(root.get("type"), "root.type")? != "frame" {
                return Err(ValidationError::Invalid(
                    "root frame type must be frame".into(),
                ));
            }
            let root_parent = require_uuid(root.get("parentId"), "root.parentId")?;
            let root_frame = require_uuid(root.get("frameId"), "root.frameId")?;
            if !root_parent.is_nil() || !root_frame.is_nil() {
                return Err(ValidationError::Invalid(
                    "root frame parentId/frameId must be nil".into(),
                ));
            }
            let root_children = require_array(root.get("shapes"), "root.shapes")?;
            for child in root_children {
                let child_id = require_uuid(Some(child), "root child id")?;
                if !shapes.contains_key(&child_id) {
                    return Err(ValidationError::Invalid(format!(
                        "root references missing shape {child_id}"
                    )));
                }
            }
            checks += 2;
        }
    }

    Ok(ValidationReport { checks })
}

fn validate_shape(
    shape: &serde_json::Map<String, Value>,
    expected_id: Uuid,
) -> Result<(), ValidationError> {
    if require_uuid(shape.get("id"), "shape.id")? != expected_id {
        return Err(ValidationError::Invalid("shape id mismatch".into()));
    }
    require_str(shape.get("name"), "shape.name")?;
    let kind = require_str(shape.get("type"), "shape.type")?;
    if !matches!(
        kind,
        "frame" | "group" | "rect" | "circle" | "text" | "path"
    ) {
        return Err(ValidationError::Invalid(format!(
            "shape {expected_id}: unsupported type {kind:?}"
        )));
    }
    for field in ["x", "y", "width", "height", "rotation", "opacity"] {
        require_finite(shape.get(field), field)?;
    }
    for field in ["parentId", "frameId", "pageId"] {
        require_uuid(shape.get(field), field)?;
    }
    let selrect = require_object(shape.get("selrect"), "shape.selrect")?;
    for field in ["x", "y", "width", "height", "x1", "y1", "x2", "y2"] {
        require_finite(selrect.get(field), &format!("selrect.{field}"))?;
    }
    let points = require_array(shape.get("points"), "shape.points")?;
    if points.len() != 4 {
        return Err(ValidationError::Invalid(format!(
            "shape {expected_id}: points must have four entries"
        )));
    }
    let transform = require_object(shape.get("transform"), "shape.transform")?;
    let inverse = require_object(shape.get("transformInverse"), "shape.transformInverse")?;
    let mut t = [0.0; 6];
    let mut i = [0.0; 6];
    for (index, field) in ["a", "b", "c", "d", "e", "f"].iter().enumerate() {
        t[index] = require_finite(transform.get(*field), &format!("transform.{field}"))?;
        i[index] = require_finite(inverse.get(*field), &format!("inverse.{field}"))?;
    }
    validate_inverse(t, i)?;

    let fills = require_array(shape.get("fills"), "shape.fills")?;
    for fill in fills {
        let fill = require_object(Some(fill), "fill")?;
        validate_hex(fill.get("fillColor"), "fillColor")?;
        validate_unit(fill.get("fillOpacity"), "fillOpacity")?;
    }
    let strokes = require_array(shape.get("strokes"), "shape.strokes")?;
    for stroke in strokes {
        let stroke = require_object(Some(stroke), "stroke")?;
        validate_hex(stroke.get("strokeColor"), "strokeColor")?;
        validate_unit(stroke.get("strokeOpacity"), "strokeOpacity")?;
        let width = require_finite(stroke.get("strokeWidth"), "strokeWidth")?;
        if width < 0.0 {
            return Err(ValidationError::Invalid("strokeWidth is negative".into()));
        }
        let alignment = require_str(stroke.get("strokeAlignment"), "strokeAlignment")?;
        if !matches!(alignment, "inner" | "center" | "outer") {
            return Err(ValidationError::Invalid(format!(
                "unknown strokeAlignment {alignment:?}"
            )));
        }
    }

    if matches!(kind, "frame" | "group") {
        let children = require_array(shape.get("shapes"), "shape.shapes")?;
        for child in children {
            require_uuid(Some(child), "child id")?;
        }
    }
    if kind == "text" {
        let content = require_object(shape.get("content"), "text.content")?;
        if require_str(content.get("type"), "text.content.type")? != "root" {
            return Err(ValidationError::Invalid(
                "text content type must be root".into(),
            ));
        }
        require_array(content.get("children"), "text.content.children")?;
    }
    if kind == "path" {
        let content = require_array(shape.get("content"), "path.content")?;
        if content.is_empty() {
            return Err(ValidationError::Invalid(
                "path content must not be empty".into(),
            ));
        }
        for command in content {
            let command = require_object(Some(command), "path command")?;
            let name = require_str(command.get("command"), "path command name")?;
            if !matches!(name, "move-to" | "line-to" | "curve-to" | "close-path") {
                return Err(ValidationError::Invalid(format!(
                    "unknown path command {name:?}"
                )));
            }
            let params = require_object(command.get("params"), "path command params")?;
            for field in ["x", "y", "c1x", "c1y", "c2x", "c2y"] {
                if let Some(value) = params.get(field) {
                    require_finite(Some(value), &format!("path {name}.{field}"))?;
                }
            }
        }
        if let Some(attrs) = shape.get("svgAttrs") {
            let attrs = require_object(Some(attrs), "svgAttrs")?;
            let rule = require_str(attrs.get("fillRule"), "svgAttrs.fillRule")?;
            if !matches!(rule, "nonzero" | "evenodd") {
                return Err(ValidationError::Invalid(format!(
                    "unknown fillRule {rule:?}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_inverse(t: [f64; 6], i: [f64; 6]) -> Result<(), ValidationError> {
    let checks = [
        (t[0] * i[0] + t[2] * i[1], 1.0),
        (t[0] * i[2] + t[2] * i[3], 0.0),
        (t[0] * i[4] + t[2] * i[5] + t[4], 0.0),
        (t[1] * i[0] + t[3] * i[1], 0.0),
        (t[1] * i[2] + t[3] * i[3], 1.0),
        (t[1] * i[4] + t[3] * i[5] + t[5], 0.0),
    ];
    for (value, expected) in checks {
        if (value - expected).abs() > 1e-6 {
            return Err(ValidationError::Invalid(
                "transformInverse is not the inverse of transform".into(),
            ));
        }
    }
    Ok(())
}

fn validate_hex(value: Option<&Value>, field: &str) -> Result<(), ValidationError> {
    let value = require_str(value, field)?;
    if !(value.len() == 7 || value.len() == 9) || !value.starts_with('#') {
        return Err(ValidationError::Invalid(format!(
            "{field} is not a hex color: {value:?}"
        )));
    }
    if !value[1..].chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ValidationError::Invalid(format!(
            "{field} contains non-hex characters"
        )));
    }
    Ok(())
}

fn validate_unit(value: Option<&Value>, field: &str) -> Result<(), ValidationError> {
    let value = require_finite(value, field)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(ValidationError::Invalid(format!(
            "{field} must be between 0 and 1"
        )));
    }
    Ok(())
}

fn validate_entry_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty()
        || name.len() > 1024
        || name.starts_with('/')
        || name.contains('\\')
        || name.contains('\0')
        || name.split('/').any(|part| part == ".." || part == ".")
    {
        return Err(ValidationError::Invalid(format!(
            "unsafe archive entry {name:?}"
        )));
    }
    Ok(())
}

fn parse_json(bytes: &[u8], context: &str) -> Result<Value, ValidationError> {
    serde_json::from_slice(bytes)
        .map_err(|error| ValidationError::Json(format!("{context}: {error}")))
}

fn require_object<'a>(
    value: Option<&'a Value>,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, ValidationError> {
    value
        .and_then(Value::as_object)
        .ok_or_else(|| ValidationError::Invalid(format!("{context} must be an object")))
}

fn require_array<'a>(
    value: Option<&'a Value>,
    context: &str,
) -> Result<&'a Vec<Value>, ValidationError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| ValidationError::Invalid(format!("{context} must be an array")))
}

fn require_str<'a>(value: Option<&'a Value>, context: &str) -> Result<&'a str, ValidationError> {
    value
        .and_then(Value::as_str)
        .ok_or_else(|| ValidationError::Invalid(format!("{context} must be a string")))
}

fn require_u64(value: Option<&Value>, context: &str) -> Result<u64, ValidationError> {
    value
        .and_then(Value::as_u64)
        .ok_or_else(|| ValidationError::Invalid(format!("{context} must be an unsigned integer")))
}

fn require_uuid(value: Option<&Value>, context: &str) -> Result<Uuid, ValidationError> {
    let value = require_str(value, context)?;
    Uuid::parse_str(value)
        .map_err(|_| ValidationError::Invalid(format!("{context} is not a UUID: {value:?}")))
}

fn require_finite(value: Option<&Value>, context: &str) -> Result<f64, ValidationError> {
    let value = value
        .and_then(Value::as_f64)
        .ok_or_else(|| ValidationError::Invalid(format!("{context} must be a number")))?;
    if !value.is_finite() {
        return Err(ValidationError::Invalid(format!("{context} is not finite")));
    }
    Ok(value)
}
