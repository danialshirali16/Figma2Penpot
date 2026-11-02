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
    let (bytes, report, penpot) = run_full(relative);
    let shapes: Vec<Shape> = penpot
        .pages
        .iter()
        .flat_map(|page| page.shapes.iter().cloned())
        .collect();
    (bytes, report, shapes)
}

fn run_full(
    relative: &str,
) -> (
    Vec<u8>,
    conversion_report::FileReport,
    penpot_writer::PenpotDocument,
) {
    let document = read_fig(fixture(relative)).expect("fixture parses");
    let mut collector = ReportCollector::new();
    let penpot =
        convert(&document, &ConvertOptions::default(), &mut collector).expect("conversion works");
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
    (bytes, report, penpot)
}

#[test]
fn sections_preserve_nested_designs_in_unclipped_boards() {
    let mut document = read_fig(fixture("text-group.fig")).unwrap();
    let section = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "FRAME")
        .unwrap();
    section.kind = "SECTION".to_owned();
    let name = section.name.clone();
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let board = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == name)
        .unwrap();
    assert_eq!(board.kind, penpot_writer::ShapeType::Frame);
    assert_eq!(board.show_content, Some(true));
    assert!(!board.shapes.as_ref().unwrap().is_empty());
    assert!(penpot.pages[0]
        .shapes
        .iter()
        .any(|shape| shape.name == "Title"));
    assert!(collector.events().iter().any(|event| {
        event.element.element_type == "SECTION"
            && event.outcome == conversion_report::Outcome::ConvertedWithModification
            && event
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("section"))
    }));
    build_archive(&penpot).expect("section board and descendants validate");
}

fn embed_button(document: &mut fig_reader::FigDocument) {
    let source_page = document.pages[0].clone();
    let mut canvas = document
        .nodes
        .iter()
        .find(|node| node.guid == source_page.guid)
        .unwrap()
        .clone();
    canvas.guid = fig_reader::Guid::new(900, 1);
    canvas.name = "Embedded library".to_owned();
    canvas.internal_only = true;
    document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "SYMBOL" && node.name.ends_with("Button"))
        .unwrap()
        .parent = Some(canvas.guid);
    let mut page = source_page;
    page.guid = canvas.guid;
    page.name = canvas.name.clone();
    page.internal_only = true;
    page.index = document.pages.len();
    document.pages.push(page);
    document.nodes.push(canvas);
}

#[test]
fn embedded_components_are_restored_with_editable_instance_children() {
    let mut document = read_fig(fixture("components.fig")).unwrap();
    embed_button(&mut document);
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let library = penpot
        .pages
        .iter()
        .find(|page| page.name == "Imported components")
        .unwrap();
    let component = penpot
        .components
        .iter()
        .find(|component| component.name == "Button")
        .unwrap();
    assert_eq!(component.main_instance_page, library.id);
    assert!(library
        .shapes
        .iter()
        .any(|shape| shape.id == component.main_instance_id));
    let instance = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Button Instance")
        .unwrap();
    assert_eq!(instance.component_id, Some(component.id));
    assert!(!instance.shapes.as_ref().unwrap().is_empty());
    assert!(!collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-INSTANCE-DETACHED"));
    build_archive(&penpot).unwrap();
}

#[test]
fn nested_override_paths_target_the_correct_embedded_instance() {
    use fig_reader::{Guid, SymbolOverride};
    let mut document = read_fig(fixture("components.fig")).unwrap();
    embed_button(&mut document);
    let main = document
        .nodes
        .iter()
        .find(|node| node.name.ends_with("Button") && node.kind == "SYMBOL")
        .unwrap()
        .clone();
    let label = document
        .nodes
        .iter()
        .find(|node| node.parent == Some(main.guid) && node.kind == "TEXT")
        .unwrap()
        .clone();
    let mut inner = main.clone();
    inner.guid = Guid::new(900, 2);
    inner.name = "Inner".to_owned();
    let mut inner_label = label.clone();
    inner_label.guid = Guid::new(900, 3);
    inner_label.parent = Some(inner.guid);
    inner_label.override_key = Some(Guid::new(901, 3));
    let mut first = label.clone();
    first.kind = "INSTANCE".to_owned();
    first.text = None;
    first.symbol_id = Some(inner.guid);
    first.override_key = Some(Guid::new(901, 4));
    let mut second = first.clone();
    second.guid = Guid::new(900, 5);
    second.override_key = Some(Guid::new(901, 5));
    *document
        .nodes
        .iter_mut()
        .find(|node| node.guid == label.guid)
        .unwrap() = first;
    document.nodes.extend([inner, inner_label, second]);
    document
        .nodes
        .iter_mut()
        .find(|node| node.name == "Button Instance")
        .unwrap()
        .symbol_overrides = vec![SymbolOverride {
        guid_path: vec![Guid::new(901, 4), Guid::new(901, 3)],
        characters: Some("Nested override".to_owned()),
        ..SymbolOverride::default()
    }];
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let texts: Vec<_> = penpot.pages[0]
        .shapes
        .iter()
        .filter_map(|shape| {
            if let Some(Content::Text(text)) = &shape.content {
                Some(text.children[0].children[0].children[0].text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        texts
            .iter()
            .filter(|text| **text == "Nested override")
            .count(),
        1
    );
    assert!(texts.iter().any(|text| *text != "Nested override"));
    assert!(!collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-INSTANCE-OVERRIDE-LOST"));
    build_archive(&penpot).unwrap();
}

#[test]
fn unsupported_instance_attributes_emit_a_diagnostic() {
    let mut document = read_fig(fixture("components.fig")).unwrap();
    let instance = document
        .nodes
        .iter_mut()
        .find(|node| node.name == "Button Instance")
        .unwrap();
    instance.symbol_overrides[0]
        .unmapped_fields
        .push("layoutSizing".to_owned());
    let mut collector = ReportCollector::new();
    convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    assert!(collector.events().iter().any(|event| event.code
        == "F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED"
        && event.outcome == conversion_report::Outcome::ConvertedWithModification
        && event
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("layoutSizing"))));
}

#[test]
fn cyclic_component_dependencies_are_contained_and_reported() {
    let mut document = read_fig(fixture("components.fig")).unwrap();
    embed_button(&mut document);
    let main = document
        .nodes
        .iter()
        .find(|node| node.kind == "SYMBOL" && node.name.ends_with("Button"))
        .unwrap()
        .guid;
    let label = document
        .nodes
        .iter_mut()
        .find(|node| node.parent == Some(main) && node.kind == "TEXT")
        .unwrap();
    label.kind = "INSTANCE".to_owned();
    label.text = None;
    label.symbol_id = Some(main);
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    assert!(collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-INSTANCE-DETACHED"));
    build_archive(&penpot).unwrap();
}

#[test]
fn expanded_shape_limit_rejects_excessive_output() {
    let document = read_fig(fixture("circle.fig")).unwrap();
    let mut collector = ReportCollector::new();
    let result = convert(
        &document,
        &ConvertOptions {
            max_expanded_shapes: 1,
            ..ConvertOptions::default()
        },
        &mut collector,
    );
    assert!(result.is_err());
}

#[test]
fn unknown_container_size_is_recovered_from_real_children_and_reported() {
    let mut document = read_fig(fixture("text-group.fig")).unwrap();
    let frame = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "FRAME")
        .unwrap();
    let name = frame.name.clone();
    frame.size = Some(fig_reader::Size {
        width: f32::NAN,
        height: f32::INFINITY,
    });
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let frame = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == name)
        .unwrap();
    assert!(frame.width.is_finite() && frame.width > 0.0);
    assert!(frame.height.is_finite() && frame.height > 0.0);
    assert_eq!(frame.show_content, Some(true));
    assert!(penpot.pages[0]
        .shapes
        .iter()
        .any(|shape| shape.name == "Title"));
    assert!(collector
        .events()
        .iter()
        .any(|event| event.element.name == name
            && event.outcome == conversion_report::Outcome::ConvertedWithModification
            && event
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("unknown"))));
    assert!(!collector
        .events()
        .iter()
        .any(|event| event.outcome == conversion_report::Outcome::Failed));
    build_archive(&penpot).unwrap();
}

#[test]
fn nonfinite_layout_metrics_are_reported_and_never_serialize_as_null() {
    let mut document = read_fig(fixture("auto-layout.fig")).unwrap();
    let node = document
        .nodes
        .iter_mut()
        .find(|node| node.stack_mode.is_some())
        .unwrap();
    let name = node.name.clone();
    node.stack_spacing = Some(f32::NAN);
    node.stack_counter_spacing = Some(f32::INFINITY);
    node.stack_horizontal_padding = Some(f32::NAN);
    node.stack_primary_align_items = Some("SPACE_BETWEEN".to_owned());
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let frame = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == name)
        .unwrap();
    let gaps = frame.layout_gap.as_ref().unwrap();
    assert_eq!((gaps.row_gap, gaps.column_gap), (0.0, 0.0));
    assert_eq!(
        frame.layout_justify_content.as_deref(),
        Some("space-between")
    );
    assert!(collector.events().iter().any(|event| event
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("source main-axis gap is unknown"))));
    build_archive(&penpot).unwrap();
}

#[test]
fn token_names_are_escaped_and_converted_bindings_follow_the_new_name() {
    let mut document = read_fig(fixture("tokens.fig")).unwrap();
    let variable = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "VARIABLE" && node.name.contains("primary"))
        .unwrap();
    variable.name = "BG Context".to_owned();
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let expected = "BG_u20_Context";
    assert!(penpot
        .pages
        .iter()
        .flat_map(|page| &page.shapes)
        .any(|shape| shape
            .applied_tokens
            .as_ref()
            .is_some_and(|tokens| tokens.values().any(|name| name == expected))));
    assert!(collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-TOKEN-NAME-NORMALIZED"));
    build_archive(&penpot).unwrap();
}

#[test]
fn clipping_paint_order_and_unsupported_blends_map_to_valid_penpot_shapes() {
    let mut document = read_fig(fixture("text-group.fig")).unwrap();
    let frame = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "FRAME")
        .unwrap();
    let frame_name = frame.name.clone();
    frame.frame_mask_disabled = Some(true);
    let rectangle = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "ROUNDED_RECTANGLE" || node.kind == "RECTANGLE")
        .unwrap();
    let rectangle_name = rectangle.name.clone();
    let mut bottom = rectangle.fills[0].clone();
    bottom.color = Some(fig_reader::Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    let mut top = bottom.clone();
    top.color = Some(fig_reader::Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    });
    rectangle.fills = vec![bottom, top];
    rectangle.blend_mode = Some("LINEAR_BURN".to_owned());
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let frame = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == frame_name)
        .unwrap();
    assert_eq!(frame.show_content, Some(true));
    let rectangle = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == rectangle_name)
        .unwrap();
    assert_eq!(rectangle.fills[0].fill_color.as_deref(), Some("#0000FF"));
    assert_eq!(rectangle.fills[1].fill_color.as_deref(), Some("#FF0000"));
    assert_eq!(rectangle.blend_mode.as_deref(), Some("multiply"));
    assert!(collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-BLEND-MODE-LOST"));
    build_archive(&penpot).unwrap();
}

#[test]
fn resized_instance_constraints_are_reported_until_fully_supported() {
    let mut document = read_fig(fixture("components.fig")).unwrap();
    let main = document
        .nodes
        .iter()
        .find(|node| node.kind == "SYMBOL" && node.name.ends_with("Button"))
        .unwrap()
        .clone();
    document
        .nodes
        .iter_mut()
        .find(|node| node.parent == Some(main.guid) && node.kind == "TEXT")
        .unwrap()
        .constraints_h = Some("MAX".to_owned());
    let instance = document
        .nodes
        .iter_mut()
        .find(|node| node.name == "Button Instance")
        .unwrap();
    instance.size.as_mut().unwrap().width = main.size.unwrap().width + 100.0;
    let mut collector = ReportCollector::new();
    convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    assert!(collector
        .events()
        .iter()
        .any(|event| event.element.name == "Button Instance"
            && event.code == "F2P-NODE-APPROXIMATED"
            && event
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("resized child constraints"))));
}

#[test]
fn empty_fill_cache_uses_the_existing_stroke_outline() {
    let mut document = read_fig(fixture("vector-network/straight-2-point-line.fig")).unwrap();
    let empty_blob = document.blobs.iter().position(|blob| blob.len == 0);
    let empty_blob = empty_blob.unwrap_or_else(|| {
        document.blobs.push(fig_reader::Blob { offset: 0, len: 0 });
        document.blobs.len() - 1
    });
    let vector = document
        .nodes
        .iter_mut()
        .find(|node| node.kind == "VECTOR")
        .unwrap();
    assert!(!vector.stroke_geometry.is_empty());
    vector.fill_geometry = vec![fig_reader::VectorGeometry {
        commands_blob: empty_blob as u32,
        winding_rule: "NONZERO".to_owned(),
    }];
    let mut collector = ReportCollector::new();
    let penpot = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    assert!(!penpot.pages[0].shapes.is_empty());
    assert!(collector
        .events()
        .iter()
        .any(|event| event.code == "F2P-VECTOR-STROKE-OUTLINE"));
    assert!(!collector
        .events()
        .iter()
        .any(|event| event.outcome == conversion_report::Outcome::Failed));
    build_archive(&penpot).unwrap();
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
    assert_eq!(ellipse.fills[0].fill_color.as_deref(), Some("#FF0000"));
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
fn styles_are_registered_and_linked() {
    let (_bytes, report, penpot) = run_full("styles.fig");
    assert_eq!(report.counts.detected, 3);
    assert_eq!(penpot.colors.len(), 1);
    assert_eq!(penpot.colors[0].name, "Primary");
    assert_eq!(penpot.colors[0].path.as_deref(), Some("Brand"));
    assert_eq!(penpot.typographies.len(), 1);
    assert_eq!(penpot.typographies[0].name, "Heading");
    let rect = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Rect")
        .expect("styled rect");
    assert_eq!(rect.fills[0].fill_color_ref_id, Some(penpot.colors[0].id));
    let text = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Text")
        .expect("styled text");
    let Some(penpot_writer::Content::Text(content)) = text.content.as_ref() else {
        panic!("text content");
    };
    assert_eq!(
        content.children[0].children[0].children[0].typography_ref_id,
        Some(penpot.typographies[0].id)
    );
}

#[test]
fn components_instances_and_variants_are_registered() {
    let (_bytes, report, penpot) = run_full("components.fig");
    assert_eq!(report.counts.detected, 9);
    assert_eq!(penpot.components.len(), 3);
    let button = penpot
        .components
        .iter()
        .find(|component| component.name == "Button")
        .expect("button component");
    let instance = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Button Instance")
        .expect("instance");
    assert_eq!(instance.component_id, Some(button.id));
    assert_eq!(instance.component_root, Some(false));
    assert_eq!(instance.shape_ref, Some(button.main_instance_id));
    let label = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Label" && shape.shape_ref.is_some())
        .expect("instance label");
    let Some(penpot_writer::Content::Text(content)) = label.content.as_ref() else {
        panic!("label content");
    };
    assert_eq!(
        content.children[0].children[0].children[0].text,
        "Overridden"
    );
    let variants: Vec<_> = penpot
        .components
        .iter()
        .filter(|component| component.variant_id.is_some())
        .collect();
    assert_eq!(variants.len(), 2);
    assert!(variants
        .iter()
        .all(|component| component.variant_properties.as_ref().map(Vec::len) == Some(1)));
}

#[test]
fn variables_become_tokens_and_applied_fills() {
    let (_bytes, report, penpot) = run_full("tokens.fig");
    assert_eq!(report.counts.detected, 2);
    let tokens = penpot.tokens.as_ref().expect("tokens document");
    let tokens = tokens.as_object().expect("token object");
    assert!(tokens.contains_key("Primitives / Light"));
    assert!(tokens.contains_key("Primitives / Dark"));
    assert_eq!(
        tokens["Primitives / Light"]["brand"]["primary"]["$value"],
        "#FF0040"
    );
    assert_eq!(
        tokens["Primitives / Dark"]["brand"]["primary"]["$value"],
        "#80001A"
    );
    let rect = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Brand Rect")
        .expect("token rect");
    assert_eq!(
        rect.applied_tokens
            .as_ref()
            .and_then(|tokens| tokens.get("fill"))
            .map(String::as_str),
        Some("brand.primary")
    );
}

#[test]
fn masks_and_booleans_are_reported_and_rendered() {
    let (_bytes, report, penpot) = run_full("masks-boolean.fig");
    assert_eq!(report.counts.detected, 5);
    assert_eq!(report.counts.approximated, 2);
    let group = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Mask group")
        .expect("mask group");
    assert_eq!(group.masked_group, Some(true));
    assert_eq!(group.shapes.as_ref().map(Vec::len), Some(3));
    let boolean = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Boolean Donut")
        .expect("boolean path");
    assert!(matches!(
        boolean.content.as_ref(),
        Some(penpot_writer::Content::Path(_))
    ));
    assert!(report
        .events
        .iter()
        .any(|event| event.code == "F2P-MASK-APPROXIMATED"));
    assert!(report
        .events
        .iter()
        .any(|event| event.code == "F2P-BOOLEAN-FLATTENED"));
}

#[test]
fn inline_token_option_keeps_values_but_removes_links() {
    let document = read_fig(fixture("styles.fig")).expect("styles fixture");
    let mut collector = ReportCollector::new();
    let linked = convert(&document, &ConvertOptions::default(), &mut collector).unwrap();
    let linked_rect = linked.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Rect")
        .unwrap();
    assert!(linked_rect.fills[0].fill_color_ref_id.is_some());
    let linked_text = linked.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Text")
        .unwrap();
    let Some(Content::Text(content)) = &linked_text.content else {
        panic!("text content");
    };
    assert!(content.children[0].children[0].children[0]
        .typography_ref_id
        .is_some());

    let mut collector = ReportCollector::new();
    let inlined = convert(
        &document,
        &ConvertOptions {
            keep_token_bindings: false,
            ..ConvertOptions::default()
        },
        &mut collector,
    )
    .unwrap();
    let rect = inlined.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Rect")
        .unwrap();
    assert_eq!(rect.fills[0].fill_color.as_deref(), Some("#FF0040"));
    assert!(rect.fills[0].fill_color_ref_id.is_none());
    let text = inlined.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Styled Text")
        .unwrap();
    let Some(Content::Text(content)) = &text.content else {
        panic!("text content");
    };
    assert!(content.children[0].children[0].children[0]
        .typography_ref_id
        .is_none());
    assert_eq!(content.children[0].children[0].children[0].text, "Heading");
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
fn gradients_shadows_and_blur_convert() {
    let (_bytes, report, shapes) = run("effects-gradient.fig");
    assert_eq!(report.counts.detected, 2);
    assert_eq!(report.counts.converted, 2);
    assert_eq!(report.counts.failed, 0);
    let shape = shapes
        .iter()
        .find(|shape| shape.name == "Gradient Shadow")
        .expect("gradient rect");
    let gradient = shape.fills[0]
        .fill_color_gradient
        .as_ref()
        .expect("gradient fill");
    assert_eq!(gradient.kind, "linear");
    assert_eq!(gradient.stops.len(), 2);
    assert_eq!(shape.shadow.as_ref().map(Vec::len), Some(1));
    assert!(shape.blur.is_some());
    assert!(shape.background_blur.is_some());
}

#[test]
fn image_media_is_registered() {
    let (_bytes, report, penpot) = run_full("image.fig");
    assert_eq!(report.counts.detected, 2);
    assert_eq!(report.counts.converted, 2);
    assert_eq!(penpot.media.len(), 1);
    assert_eq!(penpot.objects.len(), 1);
    assert_eq!(penpot.media[0].mtype, "image/png");
    assert_eq!(penpot.media[0].width, 8);
    assert_eq!(penpot.media[0].height, 8);
    assert_eq!(penpot.media[0].media_id, penpot.objects[0].id);
    let shape = penpot.pages[0]
        .shapes
        .iter()
        .find(|shape| shape.name == "Image Fill")
        .expect("image rect");
    let image = shape.fills[0].fill_image.as_ref().expect("image fill");
    assert_eq!(image.id, penpot.media[0].id);
}

#[test]
fn auto_layout_maps_to_flex_container() {
    let (_bytes, report, shapes) = run("auto-layout.fig");
    assert_eq!(report.counts.detected, 3);
    assert_eq!(report.counts.converted, 2);
    assert_eq!(report.counts.approximated, 1);
    let frame = shapes
        .iter()
        .find(|shape| shape.name == "Auto Layout")
        .expect("frame");
    assert_eq!(frame.layout.as_deref(), Some("flex"));
    assert_eq!(frame.layout_flex_dir.as_deref(), Some("row"));
    assert_eq!(
        frame
            .layout_gap
            .as_ref()
            .map(|gap| (gap.row_gap, gap.column_gap)),
        Some((0.0, 12.0))
    );
    assert_eq!(
        frame
            .layout_padding
            .as_ref()
            .map(|padding| (padding.p1, padding.p2, padding.p3, padding.p4)),
        Some((8.0, 20.0, 12.0, 10.0))
    );
    assert_eq!(frame.layout_justify_content.as_deref(), Some("center"));
    assert_eq!(frame.layout_align_items.as_deref(), Some("center"));
}

#[test]
fn circle_report_snapshot() {
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
