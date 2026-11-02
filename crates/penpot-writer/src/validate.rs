use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use serde_json::Value;
use uuid::Uuid;

use crate::model::{PENPOT_FILE_VERSION, PENPOT_MANIFEST_VERSION};

/// Resource ceilings for structural validation of a Penpot archive.
#[derive(Debug, Clone, Copy)]
pub struct ValidationLimits {
    /// Maximum ZIP entries, including per-shape JSON and library assets.
    pub max_entries: usize,
    /// Maximum uncompressed bytes in one entry.
    pub max_entry_bytes: u64,
    /// Maximum uncompressed bytes across all entries.
    pub max_total_bytes: u64,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_entries: 300_000,
            max_entry_bytes: 64 * 1024 * 1024,
            max_total_bytes: 512 * 1024 * 1024,
        }
    }
}

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
    validate_archive_with_limits(bytes, &ValidationLimits::default())
}

/// Validate an archive with explicit resource ceilings, enforcing size limits before reading.
pub fn validate_archive_with_limits(
    bytes: &[u8],
    limits: &ValidationLimits,
) -> Result<ValidationReport, ValidationError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    if archive.len() > limits.max_entries {
        return Err(ValidationError::Invalid(format!(
            "archive has {} entries, limit is {}",
            archive.len(),
            limits.max_entries
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
        if file.size() > limits.max_entry_bytes {
            return Err(ValidationError::Invalid(format!(
                "entry {name:?} is larger than {} bytes",
                limits.max_entry_bytes
            )));
        }
        let remaining = limits.max_total_bytes.saturating_sub(total);
        if file.size() > remaining {
            return Err(ValidationError::Invalid(
                "archive exceeds the total size limit".into(),
            ));
        }
        let read_limit = limits.max_entry_bytes.min(remaining);
        let mut data = Vec::new();
        file.by_ref()
            .take(read_limit.saturating_add(1))
            .read_to_end(&mut data)
            .map_err(|error| ValidationError::Zip(format!("{name}: {error}")))?;
        if data.len() as u64 > read_limit {
            return Err(ValidationError::Invalid(format!(
                "entry {name:?} exceeds the size limit"
            )));
        }
        total = total
            .checked_add(data.len() as u64)
            .ok_or_else(|| ValidationError::Invalid("archive size overflow".into()))?;
        if total > limits.max_total_bytes {
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
    let mut object_meta: HashMap<Uuid, String> = HashMap::new();
    for (path, data) in &entries {
        let Some(rest) = path.strip_prefix("objects/") else {
            continue;
        };
        let Some(id_text) = rest.strip_suffix(".json") else {
            continue;
        };
        let Ok(object_id) = Uuid::parse_str(id_text) else {
            continue;
        };
        let object = parse_json(data, path)?;
        let object = require_object(Some(&object), path)?;
        if require_uuid(object.get("id"), "object.id")? != object_id {
            return Err(ValidationError::Invalid(format!(
                "{path}: object id does not match path"
            )));
        }
        if require_str(object.get("bucket"), "object.bucket")? != "file-media-object" {
            return Err(ValidationError::Invalid(format!(
                "{path}: object bucket is not file-media-object"
            )));
        }
        let content_type = require_str(object.get("contentType"), "object.contentType")?;
        let size = require_u64(object.get("size"), "object.size")?;
        let extension = object_extension(content_type);
        let binary_path = format!("objects/{object_id}.{extension}");
        let binary = entries.get(&binary_path).ok_or_else(|| {
            ValidationError::Invalid(format!("missing media object binary {binary_path}"))
        })?;
        if binary.len() as u64 != size {
            return Err(ValidationError::Invalid(format!(
                "{binary_path}: size {} does not match metadata {size}",
                binary.len()
            )));
        }
        object_meta.insert(object_id, content_type.to_owned());
        checks += 3;
    }

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

        let media_prefix = format!("files/{file_id}/media/");
        let mut media_ids = HashSet::new();
        for (path, data) in &entries {
            if !path.starts_with(&media_prefix) || !path.ends_with(".json") {
                continue;
            }
            let rest = &path[media_prefix.len()..];
            if rest.contains('/') {
                continue;
            }
            let media = parse_json(data, path)?;
            let media = require_object(Some(&media), path)?;
            let media_id = require_uuid(media.get("id"), "media.id")?;
            if !path.ends_with(&format!("/{media_id}.json")) {
                return Err(ValidationError::Invalid(format!(
                    "{path}: media id does not match path"
                )));
            }
            require_str(media.get("name"), "media.name")?;
            let width = require_u64(media.get("width"), "media.width")?;
            let height = require_u64(media.get("height"), "media.height")?;
            if width == 0 || height == 0 {
                return Err(ValidationError::Invalid(format!(
                    "{path}: media dimensions must be positive"
                )));
            }
            let mtype = require_str(media.get("mtype"), "media.mtype")?;
            let object_id = require_uuid(media.get("mediaId"), "media.mediaId")?;
            let object_type = object_meta.get(&object_id).ok_or_else(|| {
                ValidationError::Invalid(format!(
                    "{path}: mediaId {object_id} has no object metadata"
                ))
            })?;
            if object_type != mtype {
                return Err(ValidationError::Invalid(format!(
                    "{path}: mtype {mtype:?} does not match object type {object_type:?}"
                )));
            }
            media_ids.insert(media_id);
            checks += 3;
        }

        let component_prefix = format!("files/{file_id}/components/");
        let mut component_ids = HashSet::new();
        let mut component_mains: HashMap<Uuid, Uuid> = HashMap::new();
        for (path, data) in &entries {
            if !path.starts_with(&component_prefix) || !path.ends_with(".json") {
                continue;
            }
            let rest = &path[component_prefix.len()..];
            if rest.contains('/') {
                continue;
            }
            let component = parse_json(data, path)?;
            let component = require_object(Some(&component), path)?;
            let id = require_uuid(component.get("id"), "component.id")?;
            if !path.ends_with(&format!("/{id}.json")) {
                return Err(ValidationError::Invalid(format!(
                    "{path}: component id does not match path"
                )));
            }
            require_str(component.get("name"), "component.name")?;
            require_str(component.get("path"), "component.path")?;
            let main = require_uuid(component.get("mainInstanceId"), "component.mainInstanceId")?;
            let page = require_uuid(
                component.get("mainInstancePage"),
                "component.mainInstancePage",
            )?;
            component_ids.insert(id);
            component_mains.insert(id, main);
            let _ = page;
            checks += 2;
        }

        let color_prefix = format!("files/{file_id}/colors/");
        let mut color_ids = HashSet::new();
        for (path, data) in &entries {
            if !path.starts_with(&color_prefix) || !path.ends_with(".json") {
                continue;
            }
            let rest = &path[color_prefix.len()..];
            if rest.contains('/') {
                continue;
            }
            let color = parse_json(data, path)?;
            let color = require_object(Some(&color), path)?;
            let id = require_uuid(color.get("id"), "color.id")?;
            if !path.ends_with(&format!("/{id}.json")) {
                return Err(ValidationError::Invalid(format!(
                    "{path}: color id does not match path"
                )));
            }
            require_str(color.get("name"), "color.name")?;
            validate_hex(color.get("color"), "color.color")?;
            validate_unit(color.get("opacity"), "color.opacity")?;
            color_ids.insert(id);
            checks += 2;
        }

        let typography_prefix = format!("files/{file_id}/typographies/");
        let mut typography_ids = HashSet::new();
        for (path, data) in &entries {
            if !path.starts_with(&typography_prefix) || !path.ends_with(".json") {
                continue;
            }
            let rest = &path[typography_prefix.len()..];
            if rest.contains('/') {
                continue;
            }
            let typography = parse_json(data, path)?;
            let typography = require_object(Some(&typography), path)?;
            let id = require_uuid(typography.get("id"), "typography.id")?;
            if !path.ends_with(&format!("/{id}.json")) {
                return Err(ValidationError::Invalid(format!(
                    "{path}: typography id does not match path"
                )));
            }
            for field in [
                "name",
                "fontId",
                "fontFamily",
                "fontVariantId",
                "fontSize",
                "fontWeight",
                "fontStyle",
                "lineHeight",
                "letterSpacing",
                "textTransform",
            ] {
                require_str(typography.get(field), &format!("typography.{field}"))?;
            }
            typography_ids.insert(id);
            checks += 2;
        }

        let tokens_path = format!("files/{file_id}/tokens.json");
        if let Some(tokens) = entries.get(&tokens_path) {
            let tokens = parse_json(tokens, &tokens_path)?;
            let tokens = require_object(Some(&tokens), &tokens_path)?;
            require_array(tokens.get("$themes"), "tokens.$themes")?;
            require_object(tokens.get("$metadata"), "tokens.$metadata")?;
            for (name, group) in tokens.iter().filter(|(name, _)| !name.starts_with('$')) {
                validate_token_group(group, "", 0).map_err(|error| {
                    ValidationError::Invalid(format!("token set {name:?}: {error}"))
                })?;
            }
            checks += 2;
        }

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
        let mut pending_shape_refs: Vec<(Uuid, Uuid)> = Vec::new();
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
                if !shape_id.is_nil() && !file_shape_ids.insert(shape_id) {
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
                validate_shape(
                    shape,
                    *shape_id,
                    &media_ids,
                    &color_ids,
                    &typography_ids,
                    file_id,
                )?;
                for field in ["parentId", "frameId"] {
                    let reference = require_uuid(shape.get(field), field)?;
                    if !reference.is_nil() && !shapes.contains_key(&reference) {
                        return Err(ValidationError::Invalid(format!(
                            "shape {shape_id} references missing {field} {reference}"
                        )));
                    }
                }
                if let Some(component_id) = shape.get("componentId") {
                    let component_id = require_uuid(Some(component_id), "shape.componentId")?;
                    if !component_ids.contains(&component_id) {
                        return Err(ValidationError::Invalid(format!(
                            "shape {shape_id} references missing component {component_id}"
                        )));
                    }
                    let component_file =
                        require_uuid(shape.get("componentFile"), "shape.componentFile")?;
                    if component_file != file_id {
                        return Err(ValidationError::Invalid(format!(
                            "shape {shape_id} componentFile does not match the file"
                        )));
                    }
                    if shape.get("componentRoot").and_then(Value::as_bool) == Some(true)
                        && component_mains.get(&component_id) != Some(shape_id)
                    {
                        return Err(ValidationError::Invalid(format!(
                            "shape {shape_id} is a component root but is not the main instance"
                        )));
                    }
                } else if shape.contains_key("componentFile") {
                    return Err(ValidationError::Invalid(format!(
                        "shape {shape_id} has componentFile without componentId"
                    )));
                }
                if let Some(reference) = shape.get("shapeRef") {
                    let reference = require_uuid(Some(reference), "shape.shapeRef")?;
                    pending_shape_refs.push((*shape_id, reference));
                }
                if let Some(blend) = shape.get("blendMode") {
                    let blend = require_str(Some(blend), "shape.blendMode")?;
                    if !is_valid_blend_mode(blend) {
                        return Err(ValidationError::Invalid(format!(
                            "unsupported Penpot blend mode {blend:?}"
                        )));
                    }
                }
                if let Some(tokens) = shape.get("appliedTokens") {
                    let tokens = require_object(Some(tokens), "shape.appliedTokens")?;
                    for value in tokens.values() {
                        let name = require_str(Some(value), "appliedTokens value")?;
                        if !is_valid_token_name(name) {
                            return Err(ValidationError::Invalid(format!(
                                "invalid applied token name {name:?}"
                            )));
                        }
                    }
                }
                for (field, keys) in [
                    ("layoutGap", &["rowGap", "columnGap"][..]),
                    ("layoutPadding", &["p1", "p2", "p3", "p4"][..]),
                ] {
                    if let Some(value) = shape.get(field) {
                        let values = require_object(Some(value), field)?;
                        for key in keys {
                            let number =
                                require_finite(values.get(*key), &format!("{field}.{key}"))?;
                            if !(i32::MIN as f64..=i32::MAX as f64).contains(&number) {
                                return Err(ValidationError::Invalid(format!(
                                    "{field}.{key} is outside Penpot's numeric range"
                                )));
                            }
                        }
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
        for (shape_id, reference) in pending_shape_refs {
            if !file_shape_ids.contains(&reference) {
                return Err(ValidationError::Invalid(format!(
                    "shape {shape_id} references missing shapeRef {reference}"
                )));
            }
        }
    }

    Ok(ValidationReport { checks })
}

fn validate_shape(
    shape: &serde_json::Map<String, Value>,
    expected_id: Uuid,
    media_ids: &HashSet<Uuid>,
    color_ids: &HashSet<Uuid>,
    typography_ids: &HashSet<Uuid>,
    file_id: Uuid,
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
        validate_fill(fill, media_ids, color_ids, file_id)?;
    }
    if let Some(content) = shape.get("content") {
        validate_typography_refs(content, typography_ids, file_id)?;
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
    if let Some(shadows) = shape.get("shadow") {
        for shadow in require_array(Some(shadows), "shape.shadow")? {
            validate_shadow(shadow)?;
        }
    }
    if let Some(blur) = shape.get("blur") {
        validate_blur(blur, "layer-blur", "shape.blur")?;
    }
    if let Some(blur) = shape.get("backgroundBlur") {
        validate_blur(blur, "background-blur", "shape.backgroundBlur")?;
    }
    Ok(())
}

fn object_extension(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "png",
    }
}

fn validate_fill(
    fill: &serde_json::Map<String, Value>,
    media_ids: &HashSet<Uuid>,
    color_ids: &HashSet<Uuid>,
    file_id: Uuid,
) -> Result<(), ValidationError> {
    let has_color = fill.contains_key("fillColor");
    let has_gradient = fill.contains_key("fillColorGradient");
    let has_image = fill.contains_key("fillImage");
    let kinds = [has_color, has_gradient, has_image]
        .into_iter()
        .filter(|value| *value)
        .count();
    if kinds != 1 {
        return Err(ValidationError::Invalid(
            "fill must have exactly one of fillColor, fillColorGradient, or fillImage".into(),
        ));
    }
    if has_image {
        let image = require_object(fill.get("fillImage"), "fillImage")?;
        let width = require_u64(image.get("width"), "fillImage.width")?;
        let height = require_u64(image.get("height"), "fillImage.height")?;
        let mtype = require_str(image.get("mtype"), "fillImage.mtype")?;
        let id = require_uuid(image.get("id"), "fillImage.id")?;
        if width == 0 || height == 0 || mtype.is_empty() {
            return Err(ValidationError::Invalid(
                "fillImage has invalid dimensions or type".into(),
            ));
        }
        if !media_ids.contains(&id) {
            return Err(ValidationError::Invalid(format!(
                "fillImage references missing media {id}"
            )));
        }
    }
    if has_color {
        validate_hex(fill.get("fillColor"), "fillColor")?;
    }
    if has_gradient {
        let gradient = require_object(fill.get("fillColorGradient"), "fillColorGradient")?;
        let kind = require_str(gradient.get("type"), "gradient.type")?;
        if !matches!(kind, "linear" | "radial") {
            return Err(ValidationError::Invalid(format!(
                "unsupported gradient type {kind:?}"
            )));
        }
        for field in ["startX", "startY", "endX", "endY", "width"] {
            require_finite(gradient.get(field), &format!("gradient.{field}"))?;
        }
        let stops = require_array(gradient.get("stops"), "gradient.stops")?;
        if stops.is_empty() {
            return Err(ValidationError::Invalid(
                "gradient must have at least one stop".into(),
            ));
        }
        for stop in stops {
            let stop = require_object(Some(stop), "gradient stop")?;
            validate_hex(stop.get("color"), "gradient stop color")?;
            if let Some(opacity) = stop.get("opacity") {
                validate_unit(Some(opacity), "gradient stop opacity")?;
            }
            let offset = require_finite(stop.get("offset"), "gradient stop offset")?;
            if !(0.0..=1.0).contains(&offset) {
                return Err(ValidationError::Invalid(
                    "gradient stop offset must be between 0 and 1".into(),
                ));
            }
        }
    }
    if let Some(opacity) = fill.get("fillOpacity") {
        validate_unit(Some(opacity), "fillOpacity")?;
    }
    if let Some(reference) = fill.get("fillColorRefId") {
        let id = require_uuid(Some(reference), "fillColorRefId")?;
        if !color_ids.contains(&id) {
            return Err(ValidationError::Invalid(format!(
                "fill references missing color style {id}"
            )));
        }
        let reference_file = require_uuid(fill.get("fillColorRefFile"), "fillColorRefFile")?;
        if reference_file != file_id {
            return Err(ValidationError::Invalid(
                "fillColorRefFile does not match the file".into(),
            ));
        }
    } else if fill.contains_key("fillColorRefFile") {
        return Err(ValidationError::Invalid(
            "fillColorRefFile without fillColorRefId".into(),
        ));
    }
    Ok(())
}

fn validate_typography_refs(
    value: &Value,
    typography_ids: &HashSet<Uuid>,
    file_id: Uuid,
) -> Result<(), ValidationError> {
    match value {
        Value::Object(map) => {
            if let Some(reference) = map.get("typographyRefId") {
                let id = require_uuid(Some(reference), "typographyRefId")?;
                if !typography_ids.contains(&id) {
                    return Err(ValidationError::Invalid(format!(
                        "text references missing typography style {id}"
                    )));
                }
                let reference_file =
                    require_uuid(map.get("typographyRefFile"), "typographyRefFile")?;
                if reference_file != file_id {
                    return Err(ValidationError::Invalid(
                        "typographyRefFile does not match the file".into(),
                    ));
                }
            } else if map.contains_key("typographyRefFile") {
                return Err(ValidationError::Invalid(
                    "typographyRefFile without typographyRefId".into(),
                ));
            }
            for child in map.values() {
                validate_typography_refs(child, typography_ids, file_id)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_typography_refs(item, typography_ids, file_id)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_shadow(value: &Value) -> Result<(), ValidationError> {
    let shadow = require_object(Some(value), "shadow")?;
    require_uuid(shadow.get("id"), "shadow.id")?;
    let style = require_str(shadow.get("style"), "shadow.style")?;
    if !matches!(style, "drop-shadow" | "inner-shadow") {
        return Err(ValidationError::Invalid(format!(
            "unknown shadow style {style:?}"
        )));
    }
    for field in ["offsetX", "offsetY", "blur", "spread"] {
        require_finite(shadow.get(field), &format!("shadow.{field}"))?;
    }
    let hidden = shadow.get("hidden").and_then(Value::as_bool);
    if hidden.is_none() {
        return Err(ValidationError::Invalid(
            "shadow.hidden must be boolean".into(),
        ));
    }
    let color = require_object(shadow.get("color"), "shadow.color")?;
    validate_hex(color.get("color"), "shadow.color.color")?;
    validate_unit(color.get("opacity"), "shadow.color.opacity")?;
    Ok(())
}

fn validate_blur(value: &Value, expected: &str, context: &str) -> Result<(), ValidationError> {
    let blur = require_object(Some(value), context)?;
    require_uuid(blur.get("id"), "blur.id")?;
    let kind = require_str(blur.get("type"), "blur.type")?;
    if kind != expected {
        return Err(ValidationError::Invalid(format!(
            "{context} type {kind:?}, expected {expected:?}"
        )));
    }
    require_finite(blur.get("value"), "blur.value")?;
    if blur.get("hidden").and_then(Value::as_bool).is_none() {
        return Err(ValidationError::Invalid(
            "blur.hidden must be boolean".into(),
        ));
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

/// Whether a shape blend mode is accepted by the target Penpot schema.
pub fn is_valid_blend_mode(mode: &str) -> bool {
    matches!(
        mode,
        "normal"
            | "darken"
            | "multiply"
            | "color-burn"
            | "lighten"
            | "screen"
            | "color-dodge"
            | "overlay"
            | "soft-light"
            | "hard-light"
            | "difference"
            | "exclusion"
            | "hue"
            | "saturation"
            | "color"
            | "luminosity"
    )
}

/// Whether a name satisfies the target Penpot token identifier grammar.
pub fn is_valid_token_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let Some(first) = bytes.first() else {
        return false;
    };
    if !(first.is_ascii_alphanumeric() || matches!(first, b'_' | b'-')) {
        return false;
    }
    if bytes.last() == Some(&b'.') {
        return false;
    }
    let mut dot = false;
    for byte in bytes.iter().skip(1) {
        if *byte == b'.' {
            if dot {
                return false;
            }
            dot = true;
        } else {
            if !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'$')) {
                return false;
            }
            dot = false;
        }
    }
    true
}

fn validate_token_group(value: &Value, prefix: &str, depth: usize) -> Result<(), ValidationError> {
    if depth > 128 {
        return Err(ValidationError::Invalid(
            "token nesting exceeds 128".to_owned(),
        ));
    }
    let group = require_object(Some(value), "token group")?;
    for (key, value) in group.iter().filter(|(key, _)| !key.starts_with('$')) {
        let name = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        let object = require_object(Some(value), "token or group")?;
        if object.contains_key("$value") || object.contains_key("value") {
            if !is_valid_token_name(&name) {
                return Err(ValidationError::Invalid(format!(
                    "invalid token name {name:?}"
                )));
            }
        } else {
            validate_token_group(value, &name, depth + 1)?;
        }
    }
    Ok(())
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
