use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use uuid::Uuid;

use crate::model::PenpotDocument;
use crate::validate::{validate_archive, ValidationReport};

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("cannot write {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error("JSON serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("generated Penpot file failed validation: {0}")]
    Validation(String),
}

impl From<zip::result::ZipError> for WriteError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value.to_string())
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    version: u32,
    #[serde(rename = "type")]
    kind: String,
    generated_by: String,
    referer: String,
    files: Vec<ManifestFile>,
    relations: Vec<(Uuid, Uuid)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
    id: Uuid,
    name: String,
    features: Vec<String>,
}

/// Serialize a document into a `.penpot` archive and validate the bytes.
pub fn build_archive(document: &PenpotDocument) -> Result<Vec<u8>, WriteError> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let file_id = document.file.id;
    write_json(
        &mut writer,
        options,
        &format!("files/{file_id}.json"),
        &document.file,
    )?;

    for page in &document.pages {
        write_json(
            &mut writer,
            options,
            &format!("files/{file_id}/pages/{}.json", page.id),
            &PageMeta {
                id: page.id,
                name: page.name.clone(),
                index: page.index,
                background: page.background.clone(),
            },
        )?;

        write_json(
            &mut writer,
            options,
            &shape_path(file_id, page.id, page.root_frame.id),
            &page.root_frame,
        )?;
        for shape in &page.shapes {
            write_json(
                &mut writer,
                options,
                &shape_path(file_id, page.id, shape.id),
                shape,
            )?;
        }
    }

    for media in &document.media {
        write_json(
            &mut writer,
            options,
            &format!("files/{file_id}/media/{}.json", media.id),
            media,
        )?;
    }

    for object in &document.objects {
        write_json(
            &mut writer,
            options,
            &format!("objects/{}.json", object.id),
            &ObjectMeta {
                id: object.id,
                bucket: "file-media-object".to_owned(),
                content_type: object.content_type.clone(),
                size: object.bytes.len() as u64,
            },
        )?;
        let extension = extension_for(&object.content_type);
        writer.start_file(format!("objects/{}.{extension}", object.id), options)?;
        writer
            .write_all(&object.bytes)
            .map_err(|source| WriteError::Io {
                path: format!("objects/{}.{extension}", object.id),
                source,
            })?;
    }

    let manifest = Manifest {
        version: crate::model::PENPOT_MANIFEST_VERSION,
        kind: "penpot/export-files".to_owned(),
        generated_by: format!("figma-2-penpot/{}", env!("CARGO_PKG_VERSION")),
        referer: "figma-2-penpot".to_owned(),
        files: vec![ManifestFile {
            id: file_id,
            name: document.file.name.clone(),
            features: document.file.features.clone(),
        }],
        relations: Vec::new(),
    };
    write_json(&mut writer, options, "manifest.json", &manifest)?;

    let bytes = writer.finish()?.into_inner();
    validate_archive(&bytes).map_err(|error| WriteError::Validation(error.to_string()))?;
    Ok(bytes)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageMeta {
    id: Uuid,
    name: String,
    index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObjectMeta {
    id: Uuid,
    bucket: String,
    content_type: String,
    size: u64,
}

fn extension_for(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "png",
    }
}

fn shape_path(file_id: Uuid, page_id: Uuid, shape_id: Uuid) -> String {
    format!("files/{file_id}/pages/{page_id}/{shape_id}.json")
}

fn write_json<T: Serialize>(
    writer: &mut zip::ZipWriter<Cursor<Vec<u8>>>,
    options: zip::write::SimpleFileOptions,
    path: &str,
    value: &T,
) -> Result<(), WriteError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    writer.start_file(path.to_owned(), options)?;
    writer.write_all(&bytes).map_err(|source| WriteError::Io {
        path: path.to_owned(),
        source,
    })?;
    Ok(())
}

/// Write a validated archive to `path` atomically: temp file then rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), WriteError> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = parent.unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output.penpot");
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp: PathBuf = dir.join(format!(".{file_name}.tmp-{}-{counter}", std::process::id()));

    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|source| WriteError::Io {
                path: temp.display().to_string(),
                source,
            })?;
        file.write_all(bytes).map_err(|source| WriteError::Io {
            path: temp.display().to_string(),
            source,
        })?;
        file.sync_all().map_err(|source| WriteError::Io {
            path: temp.display().to_string(),
            source,
        })?;
        drop(file);
        fs::rename(&temp, path).map_err(|source| WriteError::Io {
            path: path.display().to_string(),
            source,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Build, validate, and atomically write a document.
pub fn write_penpot_atomic(
    path: &Path,
    document: &PenpotDocument,
) -> Result<ValidationReport, WriteError> {
    let bytes = build_archive(document)?;
    let report =
        validate_archive(&bytes).map_err(|error| WriteError::Validation(error.to_string()))?;
    write_atomic(path, &bytes)?;
    Ok(report)
}
