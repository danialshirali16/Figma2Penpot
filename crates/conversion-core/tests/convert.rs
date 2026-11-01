use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use conversion_core::{convert, ConvertOptions};
use conversion_report::{FileStatus, ReportCollector, ValidationState};
use fig_reader::read_fig;
use penpot_writer::{build_archive, validate_archive, Content, Shape};

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

fn run(relative: &str) -> (Vec<u8>, conversion_report::FileReport, Vec<Shape>) {
    let document = read_fig(fixture(relative)).expect("fixture parses");
    let mut collector = ReportCollector::new();
    let penpot =
        convert(&document, &ConvertOptions::default(), &mut collector).expect("conversion works");
    let shapes: Vec<Shape> = penpot
        .pages
        .iter()
        .flat_map(|page| page.shapes.iter().cloned())
        .collect();
    let bytes = build_archive(&penpot).expect("archive builds");
    validate_archive(&bytes).expect("archive validates");
    let report = collector.finish(
        relative,
        Some("out.penpot".to_owned()),
        Duration::ZERO,
        ValidationState::Passed,
        "test",
        "2.18.5",
    );
    (bytes, report, shapes)
}

#[test]
fn circle_converts_with_correct_counts() {
    let (_bytes, report, shapes) = run("circle.fig");
    assert_eq!(report.counts.detected, 2);
    assert_eq!(report.counts.converted, 2);
    assert_eq!(report.counts.approximated, 0);
    assert_eq!(report.counts.skipped, 0);
    assert_eq!(report.counts.unsupported, 0);
    assert_eq!(report.counts.failed, 0);
    assert_eq!(report.status, FileStatus::Completed);
    assert_eq!(shapes.len(), 2); // Frame 1 + Ellipse 1; root frame is separate

    let frame = shapes.iter().find(|shape| shape.name == "Frame 1").unwrap();
    let ellipse = shapes
        .iter()
        .find(|shape| shape.name == "Ellipse 1")
        .unwrap();
    assert_eq!(frame.kind.as_str(), "frame");
    assert_eq!(
        (frame.x, frame.y, frame.width, frame.height),
        (0.0, 0.0, 350.0, 350.0)
    );
    assert_eq!(ellipse.kind.as_str(), "circle");
    assert_eq!(
        (ellipse.x, ellipse.y, ellipse.width, ellipse.height),
        (25.0, 25.0, 300.0, 300.0)
    );
    assert_eq!(ellipse.frame_id, frame.id);
    assert_eq!(ellipse.fills[0].fill_color, "#FF0000");
    assert_eq!(ellipse.strokes[0].stroke_width, 5.0);
}

#[test]
fn text_group_converts_absolute_geometry_and_text() {
    let (_bytes, report, shapes) = run("text-group.fig");
    assert_eq!(report.counts.detected, 6);
    assert_eq!(report.counts.converted, 6);
    assert_eq!(report.counts.failed, 0);

    let card = shapes.iter().find(|shape| shape.name == "Card").unwrap();
    assert_eq!((card.x, card.y), (20.0, 30.0));
    let text = shapes.iter().find(|shape| shape.name == "Title").unwrap();
    assert_eq!(text.kind.as_str(), "text");
    assert_eq!((text.x, text.y), (30.0, 130.0));
    let Content::Text(content) = text.content.as_ref().expect("text content") else {
        panic!("expected text content");
    };
    let run = &content.children[0].children[0].children[0];
    assert_eq!(run.text, "Hello Penpot");
    assert_eq!(run.font_family, "Inter");
    assert_eq!(run.font_size, "16");
    assert_eq!(run.font_weight, "400");

    let group = shapes
        .iter()
        .find(|shape| shape.name == "Badge Group")
        .unwrap();
    assert_eq!(group.kind.as_str(), "group");
    assert_eq!(
        (group.x, group.y, group.width, group.height),
        (25.0, 235.0, 80.0, 30.0)
    );
    let group_rect = shapes
        .iter()
        .find(|shape| shape.name == "Group Rect")
        .unwrap();
    assert_eq!((group_rect.x, group_rect.y), (25.0, 235.0));
    assert_eq!(group_rect.frame_id, card.id);
    assert_eq!(group_rect.parent_id, group.id);

    let red = shapes
        .iter()
        .find(|shape| shape.name == "Red Rect")
        .unwrap();
    assert_eq!(red.r1, Some(4.0));
    assert_eq!(red.r4, Some(4.0));
}

#[test]
fn filled_vector_converts_to_multiple_paths() {
    let (_bytes, report, shapes) = run("OpenFigs.fig");
    assert_eq!(report.counts.detected, 2);
    assert_eq!(report.counts.converted, 1);
    assert_eq!(report.counts.approximated, 1);
    assert_eq!(report.counts.unsupported, 0);
    assert_eq!(report.counts.failed, 0);
    assert_eq!(report.status, FileStatus::CompletedWithWarnings);
    assert!(report.events.iter().any(|event| {
        event.code == "F2P-VECTOR-NETWORK-LOST" && event.element.element_type == "VECTOR"
    }));
    assert!(shapes.iter().any(|shape| shape.kind.as_str() == "frame"));
    let frame = shapes
        .iter()
        .find(|shape| shape.kind.as_str() == "frame")
        .expect("frame");
    assert_eq!(frame.shapes.as_ref().map(Vec::len), Some(20));
    let paths: Vec<_> = shapes
        .iter()
        .filter(|shape| shape.kind.as_str() == "path")
        .collect();
    assert_eq!(paths.len(), 20);
    for path in &paths {
        let Content::Path(commands) = path.content.as_ref().expect("path content") else {
            panic!("expected path content");
        };
        assert!(!commands.is_empty());
    }
}

#[test]
fn stroke_only_vector_is_reported_as_approximated() {
    let (_bytes, report, shapes) = run("vector-network/straight-2-point-line.fig");
    assert_eq!(report.counts.detected, 1);
    assert_eq!(report.counts.converted, 0);
    assert_eq!(report.counts.approximated, 1);
    assert!(report.events.iter().any(|event| {
        event.code == "F2P-VECTOR-STROKE-OUTLINE" && event.element.element_type == "VECTOR"
    }));
    let path = shapes
        .iter()
        .find(|shape| shape.kind.as_str() == "path")
        .expect("path");
    assert!(path.strokes.is_empty());
    assert!(!path.fills.is_empty());
}

#[test]
fn archive_is_deterministic() {
    let document = read_fig(fixture("circle.fig")).unwrap();
    let mut first = ReportCollector::new();
    let mut second = ReportCollector::new();
    let a = convert(&document, &ConvertOptions::default(), &mut first).unwrap();
    let b = convert(&document, &ConvertOptions::default(), &mut second).unwrap();
    assert_eq!(build_archive(&a).unwrap(), build_archive(&b).unwrap());
}

#[test]
fn validator_rejects_non_archive_input() {
    assert!(validate_archive(b"not a zip").is_err());
}

#[test]
fn archive_has_expected_entries_and_refs() {
    let (bytes, _report, _shapes) = run("circle.fig");
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(archive.len(), 6); // manifest, file, page, root, frame, ellipse

    let mut manifest = String::new();
    archive
        .by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut manifest)
        .unwrap();
    assert!(manifest.contains("penpot/export-files"));
    assert!(manifest.contains("figma-2-penpot"));

    let mut root = String::new();
    let root_name = archive
        .file_names()
        .filter_map(|name| name.ok())
        .map(|name| name.into_owned())
        .find(|name| name.ends_with("00000000-0000-0000-0000-000000000000.json"))
        .unwrap();
    archive
        .by_name(&root_name)
        .unwrap()
        .read_to_string(&mut root)
        .unwrap();
    assert!(root.contains("\"name\": \"Root Frame\""));
}

#[test]
fn vector_report_snapshot() {
    let (_bytes, report, _shapes) = run("OpenFigs.fig");
    let mut value = serde_json::to_value(report).unwrap();
    value["source"] = serde_json::json!("OpenFigs.fig");
    value["output"] = serde_json::json!("out.penpot");
    value["durationMs"] = serde_json::json!(0);
    value["toolVersion"] = serde_json::json!("test");

    let golden_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/openfigs.report.json");
    let golden: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&golden_path).unwrap()).unwrap();
    assert_eq!(
        value,
        golden,
        "report changed; review {}",
        golden_path.display()
    );
}

#[test]
fn report_snapshot() {
    let (_bytes, report, _shapes) = run("circle.fig");
    let mut value = serde_json::to_value(report).unwrap();
    value["source"] = serde_json::json!("circle.fig");
    value["output"] = serde_json::json!("out.penpot");
    value["durationMs"] = serde_json::json!(0);
    value["toolVersion"] = serde_json::json!("test");

    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/circle.report.json");
    let golden: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&golden_path).unwrap()).unwrap();
    assert_eq!(
        value,
        golden,
        "report changed; review {}",
        golden_path.display()
    );
}
