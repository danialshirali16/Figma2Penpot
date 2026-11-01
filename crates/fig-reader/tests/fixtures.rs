use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use fig_reader::{read_fig, read_fig_bytes, ContainerKind, FigError, Limits};

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

fn fixture_bytes(relative: &str) -> Vec<u8> {
    fs::read(fixture(relative)).expect("fixture should be readable")
}

fn canvas_of(relative: &str) -> Vec<u8> {
    let bytes = fixture_bytes(relative);
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("fixture is a ZIP");
    let mut entry = archive.by_name("canvas.fig").expect("canvas.fig");
    let mut canvas = Vec::new();
    entry.read_to_end(&mut canvas).expect("canvas bytes");
    canvas
}

#[test]
fn reads_circle_fixture() {
    let document = read_fig(fixture("circle.fig")).expect("circle.fig should parse");

    assert_eq!(document.container, ContainerKind::Zip);
    assert_eq!(document.prelude, "fig-kiwi");
    assert_eq!(document.version, 101);
    assert_eq!(document.schema_defs, 550);
    assert_eq!(document.chunk_count, 2);
    assert!(document.has_thumbnail);
    assert_eq!(document.stats.node_count, 5);
    assert_eq!(document.stats.active_nodes, 5);
    assert_eq!(document.stats.removed_nodes, 0);
    assert_eq!(document.stats.blob_count, 3);
    assert_eq!(document.stats.blob_bytes, 540);
    assert_eq!(document.stats.image_count, 0);
    assert_eq!(document.stats.orphan_nodes, 0);
    assert_eq!(document.pages.len(), 2);
    assert_eq!(document.pages[0].name, "Page 1");
    assert!(!document.pages[0].internal_only);
    assert_eq!(document.pages[0].direct_children, 1);
    assert_eq!(document.pages[0].descendant_nodes, 2);
    assert!(document.pages[1].internal_only);
    assert_eq!(document.meta_file_name(), Some("circle"));
    assert!(document.warnings.is_empty());
}

#[test]
fn reads_openfigs_fixture() {
    let document = read_fig(fixture("OpenFigs.fig")).expect("OpenFigs.fig should parse");
    assert_eq!(document.version, 106);
    assert_eq!(document.schema_defs, 550);
    assert_eq!(document.stats.node_count, 5);
    assert_eq!(document.stats.blob_count, 21);
    assert_eq!(document.stats.blob_bytes, 12_098);
    assert_eq!(document.meta_file_name(), Some("OpenFigs"));
    assert!(document.pages.iter().any(|page| page.internal_only));
}

#[test]
fn reads_vector_network_fixtures() {
    for name in [
        "vector-network/straight-2-point-line.fig",
        "vector-network/corner-radius-middle-dot-20.fig",
    ] {
        let document = read_fig(fixture(name)).expect(name);
        assert_eq!(document.version, 101, "{name}");
        assert_eq!(document.schema_defs, 627, "{name}");
        assert_eq!(document.stats.node_count, 4, "{name}");
        assert_eq!(document.stats.blob_count, 2, "{name}");
        assert_eq!(document.stats.node_types.get("VECTOR").copied(), Some(1));
    }
}

#[test]
fn reads_raw_canvas_without_zip() {
    let canvas = canvas_of("circle.fig");
    let document = read_fig_bytes("circle.canvas.fig", &canvas, &Limits::default())
        .expect("raw canvas should parse");
    assert_eq!(document.container, ContainerKind::Raw);
    assert_eq!(document.stats.node_count, 5);
    assert_eq!(document.stats.image_count, 0);
    assert!(!document.has_thumbnail);
    assert_eq!(document.meta_file_name(), None);
}

#[test]
fn reads_text_and_group_fixture() {
    let document = read_fig(fixture("text-group.fig")).expect("text-group.fig should parse");
    assert_eq!(document.stats.node_count, 9);
    assert_eq!(document.stats.node_types.get("TEXT").copied(), Some(1));
    assert_eq!(document.stats.node_types.get("FRAME").copied(), Some(2));

    let text = document
        .nodes
        .iter()
        .find(|node| node.name == "Title")
        .expect("text node");
    let text_data = text.text.as_ref().expect("text data");
    assert_eq!(text_data.characters.as_deref(), Some("Hello Penpot"));
    assert_eq!(text_data.font_family.as_deref(), Some("Inter"));
    assert_eq!(text_data.font_size, Some(16.0));

    let red = document
        .nodes
        .iter()
        .find(|node| node.name == "Red Rect")
        .expect("red rect");
    assert_eq!(red.fills[0].kind, "SOLID");
    assert_eq!(red.strokes[0].kind, "SOLID");
    assert_eq!(red.stroke_weight, Some(2.0));
    assert_eq!(red.corner_radii, Some([4.0, 4.0, 4.0, 4.0]));

    let group = document
        .nodes
        .iter()
        .find(|node| node.name == "Badge Group")
        .expect("group frame");
    assert_eq!(group.resize_to_fit, Some(true));
}

#[test]
fn blob_access_is_bounds_checked() {
    let document = read_fig(fixture("circle.fig")).expect("circle.fig should parse");
    assert_eq!(document.blob(0).map(<[u8]>::len), Some(110));
    assert!(document.blob(999).is_none());
    assert!(!document.message().is_empty());
}

#[test]
fn truncation_returns_an_error_without_panicking() {
    let bytes = fixture_bytes("circle.fig");
    for length in [0, 4, 12, 40, 1_000, bytes.len() - 1] {
        let result = read_fig_bytes("truncated.fig", &bytes[..length], &Limits::default());
        assert!(result.is_err(), "length {length} should not parse");
    }
}

#[test]
fn corrupt_chunk_length_is_rejected() {
    let mut canvas = canvas_of("circle.fig");
    canvas[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    let error = read_fig_bytes("bad-chunk.fig", &canvas, &Limits::default()).unwrap_err();
    assert!(matches!(error, FigError::Truncated(_)), "{error}");
}

#[test]
fn non_fig_input_is_rejected() {
    let error = read_fig_bytes("not-a-fig.bin", b"hello world", &Limits::default()).unwrap_err();
    assert!(matches!(error, FigError::NotFig(_)), "{error}");
}

#[test]
fn unsupported_prelude_is_rejected() {
    let mut canvas = canvas_of("circle.fig");
    canvas[..8].copy_from_slice(b"fig-deck");
    let error = read_fig_bytes("deck.fig", &canvas, &Limits::default()).unwrap_err();
    assert!(matches!(error, FigError::UnsupportedFormat(_)), "{error}");
}

#[test]
fn node_limit_is_enforced() {
    let limits = Limits {
        max_nodes: 1,
        ..Limits::default()
    };
    let error = read_fig_with_limits_for_test("circle.fig", &limits).unwrap_err();
    assert!(matches!(error, FigError::Limit { .. }), "{error}");
}

fn read_fig_with_limits_for_test(
    relative: &str,
    limits: &Limits,
) -> fig_reader::Result<fig_reader::FigDocument> {
    let bytes = fixture_bytes(relative);
    read_fig_bytes(relative, &bytes, limits)
}

#[test]
fn zip_entry_limit_is_enforced() {
    let bytes = make_zip(&[("a.txt", b"a"), ("b.txt", b"b"), ("c.txt", b"c")]);
    let limits = Limits {
        max_zip_entries: 2,
        ..Limits::default()
    };
    let error = read_fig_bytes("entries.fig", &bytes, &limits).unwrap_err();
    assert!(matches!(error, FigError::Limit { .. }), "{error}");
}

#[test]
fn zip_slip_entry_is_rejected() {
    let bytes = make_zip(&[("../evil.fig", b"x"), ("canvas.fig", b"x")]);
    let error = read_fig_bytes("slip.fig", &bytes, &Limits::default()).unwrap_err();
    assert!(matches!(error, FigError::UnsafeEntry { .. }), "{error}");
}

#[test]
fn compression_ratio_limit_is_enforced() {
    let zeros = vec![0u8; 256 * 1024];
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut cursor);
        writer
            .start_file(
                "canvas.fig",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .expect("start entry");
        writer.write_all(&zeros).expect("write zeros");
        writer.finish().expect("finish zip");
    }
    let limits = Limits {
        max_compression_ratio: 10,
        ..Limits::default()
    };
    let error = read_fig_bytes("bomb.fig", cursor.get_ref(), &limits).unwrap_err();
    assert!(matches!(error, FigError::Limit { .. }), "{error}");
}

#[test]
fn unknown_entries_produce_a_warning_and_valid_canvas_still_parses() {
    let canvas = canvas_of("circle.fig");
    let bytes = make_zip_with_stored(&[("canvas.fig", &canvas), ("notes.txt", b"ignore me")]);
    let document = read_fig_bytes("with-notes.fig", &bytes, &Limits::default())
        .expect("valid canvas should parse");
    assert_eq!(document.stats.node_count, 5);
    assert!(document
        .warnings
        .iter()
        .any(|warning| warning.code == "FIG-WARN-UNKNOWN-ENTRY"));
}

fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    make_zip_with_stored(entries)
}

fn make_zip_with_stored(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut cursor);
        for (name, bytes) in entries {
            writer
                .start_file(
                    *name,
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .expect("start entry");
            writer.write_all(bytes).expect("write entry");
        }
        writer.finish().expect("finish zip");
    }
    cursor.into_inner()
}
