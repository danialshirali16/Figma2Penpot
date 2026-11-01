use std::io::{Cursor, Write};

use penpot_writer::{
    build_archive, validate_archive, write_atomic, PenpotDocument, PenpotPage, Shape, ShapeType,
    WriteError,
};
use uuid::Uuid;

fn sample() -> PenpotDocument {
    let file_id = Uuid::from_u128(1);
    let page_id = Uuid::from_u128(2);
    let rect_id = Uuid::from_u128(3);
    let mut root = Shape::root_frame(page_id, vec![rect_id]);
    root.page_id = page_id;
    let mut rect = Shape::new(
        rect_id,
        "Rectangle",
        ShapeType::Rect,
        10.0,
        20.0,
        100.0,
        50.0,
        Uuid::nil(),
        Uuid::nil(),
        page_id,
    );
    rect.fills = vec![penpot_writer::Fill::solid("#FF0000", 1.0)];
    let mut document = PenpotDocument::new(file_id, "Test");
    document.pages.push(PenpotPage {
        id: page_id,
        name: "Page 1".to_owned(),
        index: 0,
        background: None,
        root_frame: root,
        shapes: vec![rect],
    });
    document
}

#[test]
fn builds_and_validates_minimal_document() {
    let bytes = build_archive(&sample()).expect("archive builds");
    let report = validate_archive(&bytes).expect("archive validates");
    assert!(report.checks >= 10);
}

#[test]
fn rejects_a_missing_shape_reference() {
    let mut document = sample();
    document.pages[0].root_frame.shapes = Some(vec![Uuid::from_u128(4)]);
    let error = build_archive(&document).unwrap_err();
    assert!(matches!(error, WriteError::Validation(_)), "{error}");
}

#[test]
fn rejects_non_archive_bytes() {
    assert!(validate_archive(b"not a zip").is_err());
    assert!(validate_archive(b"PK\x03\x04").is_err());
}

#[test]
fn rejects_zip_without_manifest() {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "files/x.json",
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
    writer.write_all(b"{}").unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let error = validate_archive(&bytes).unwrap_err();
    assert!(error.to_string().contains("manifest"), "{error}");
}

#[test]
fn atomic_write_failure_removes_the_temporary_file() {
    let dir = std::env::temp_dir().join(format!(
        "f2p-writer-failure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let destination = dir.join("out.penpot");
    std::fs::create_dir(&destination).unwrap();
    let error = write_atomic(&destination, b"data").unwrap_err();
    assert!(matches!(error, WriteError::Io { .. }));
    assert!(destination.is_dir());
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files were not cleaned up");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn atomic_write_replaces_the_destination() {
    let dir = std::env::temp_dir().join(format!(
        "f2p-writer-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("out.penpot");
    write_atomic(&path, b"first").unwrap();
    write_atomic(&path, b"second").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files were not cleaned up");
    let _ = std::fs::remove_dir_all(dir);
}
