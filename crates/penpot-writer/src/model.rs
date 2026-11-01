use serde::Serialize;
use uuid::Uuid;

pub const TARGET_PENPOT: &str = "2.18.5";
pub const PENPOT_FILE_VERSION: u32 = 67;
pub const PENPOT_MANIFEST_VERSION: u32 = 1;

pub const FEATURES: &[&str] = &[
    "plugins/runtime",
    "design-tokens/v1",
    "variants/v1",
    "layout/grid",
    "styles/v2",
    "components/v2",
    "fdata/shape-data-type",
];

/// Migrations applied by the official library for file data version 67.
/// This is format data from the target Penpot release, not executable code.
pub const MIGRATIONS: &[&str] = &[
    "legacy-2",
    "legacy-3",
    "legacy-5",
    "legacy-6",
    "legacy-7",
    "legacy-8",
    "legacy-9",
    "legacy-10",
    "legacy-11",
    "legacy-12",
    "legacy-13",
    "legacy-14",
    "legacy-16",
    "legacy-17",
    "legacy-18",
    "legacy-19",
    "legacy-25",
    "legacy-26",
    "legacy-27",
    "legacy-28",
    "legacy-29",
    "legacy-31",
    "legacy-32",
    "legacy-33",
    "legacy-34",
    "legacy-36",
    "legacy-37",
    "legacy-38",
    "legacy-39",
    "legacy-40",
    "legacy-41",
    "legacy-42",
    "legacy-43",
    "legacy-44",
    "legacy-45",
    "legacy-46",
    "legacy-47",
    "legacy-48",
    "legacy-49",
    "legacy-50",
    "legacy-51",
    "legacy-52",
    "legacy-53",
    "legacy-54",
    "legacy-55",
    "legacy-56",
    "legacy-57",
    "legacy-59",
    "legacy-62",
    "legacy-65",
    "legacy-66",
    "legacy-67",
    "0001-remove-tokens-from-groups",
    "0002-normalize-bool-content-v2",
    "0003-convert-path-content-v2",
    "0005-deprecate-image-type",
    "0006-fix-old-texts-fills",
    "0008-fix-library-colors-v4",
    "0009-clean-library-colors",
    "0009-add-partial-text-touched-flags",
    "0010-fix-swap-slots-pointing-non-existent-shapes",
    "0011-fix-invalid-text-touched-flags",
    "0012-fix-position-data",
    "0013-fix-component-path",
    "0013-clear-invalid-strokes-and-fills",
    "0014-fix-tokens-lib-duplicate-ids",
    "0014-clear-components-nil-objects",
    "0015-fix-text-attrs-blank-strings",
    "0015-clean-shadow-color",
    "0016-copy-fills-from-position-data-to-text-node",
    "0017-fix-layout-flex-dir",
    "0018-remove-unneeded-objects-from-components",
    "0019-fix-missing-swap-slots",
    "0020-sync-component-id-with-near-main",
    "0021-fix-shape-svg-attrs",
    "0022-normalize-component-root-and-resync",
    "0023-repair-token-themes-with-inexistent-sets",
    "0024b-fix-stroke-cap-placement",
    "0025-repair-empty-text-content",
    "0026-fix-svg-raw-shapes-uuids",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PenpotDocument {
    pub file: PenpotFile,
    pub pages: Vec<PenpotPage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PenpotFile {
    pub id: Uuid,
    pub name: String,
    pub is_shared: bool,
    pub features: Vec<String>,
    pub migrations: Vec<String>,
    pub version: u32,
    pub options: FileOptions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOptions {
    pub components_v2: bool,
    pub base_font_size: String,
}

impl PenpotDocument {
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            file: PenpotFile {
                id,
                name: name.into(),
                is_shared: false,
                features: FEATURES.iter().map(|value| (*value).to_owned()).collect(),
                migrations: MIGRATIONS.iter().map(|value| (*value).to_owned()).collect(),
                version: PENPOT_FILE_VERSION,
                options: FileOptions {
                    components_v2: true,
                    base_font_size: "16px".to_owned(),
                },
            },
            pages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PenpotPage {
    pub id: Uuid,
    pub name: String,
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    pub root_frame: Shape,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeType {
    Frame,
    Group,
    Rect,
    Circle,
    Text,
}

impl ShapeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::Group => "group",
            Self::Rect => "rect",
            Self::Circle => "circle",
            Self::Text => "text",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub selrect: Rect,
    pub points: Vec<Point>,
    pub transform: Transform,
    pub transform_inverse: Transform,
    pub parent_id: Uuid,
    pub frame_id: Uuid,
    pub page_id: Uuid,
    pub fills: Vec<Fill>,
    pub strokes: Vec<Stroke>,
    pub opacity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r4: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shapes: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<TextContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grow_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proportion: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proportion_lock: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_fill_on_export: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_content: Option<bool>,
}

impl Shape {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        name: impl Into<String>,
        kind: ShapeType,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        parent_id: Uuid,
        frame_id: Uuid,
        page_id: Uuid,
    ) -> Self {
        let x = clean_coordinate(x);
        let y = clean_coordinate(y);
        let width = clean_coordinate(width);
        let height = clean_coordinate(height);
        let selrect = Rect::from_xywh(x, y, width, height);
        Self {
            id,
            name: name.into(),
            kind,
            x,
            y,
            width,
            height,
            rotation: 0.0,
            points: selrect.points(),
            selrect,
            transform: Transform::IDENTITY,
            transform_inverse: Transform::IDENTITY,
            parent_id,
            frame_id,
            page_id,
            fills: Vec::new(),
            strokes: Vec::new(),
            opacity: 1.0,
            hidden: None,
            r1: None,
            r2: None,
            r3: None,
            r4: None,
            shapes: None,
            content: None,
            grow_type: None,
            proportion: None,
            proportion_lock: None,
            hide_fill_on_export: None,
            show_content: None,
        }
    }

    pub fn root_frame(page_id: Uuid, children: Vec<Uuid>) -> Self {
        let mut shape = Self::new(
            Uuid::nil(),
            "Root Frame",
            ShapeType::Frame,
            0.0,
            0.0,
            0.01,
            0.01,
            Uuid::nil(),
            Uuid::nil(),
            page_id,
        );
        shape.fills = vec![Fill {
            fill_color: "#FFFFFF".to_owned(),
            fill_opacity: 1.0,
        }];
        shape.shapes = Some(children);
        shape.proportion = Some(1.0);
        shape.proportion_lock = Some(false);
        shape.hide_fill_on_export = Some(false);
        shape.r1 = Some(0.0);
        shape.r2 = Some(0.0);
        shape.r3 = Some(0.0);
        shape.r4 = Some(0.0);
        shape
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

fn clean_coordinate(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let rounded = (value * 10_000.0).round() / 10_000.0;
    if rounded.abs() < 1e-9 {
        0.0
    } else {
        rounded
    }
}

impl Rect {
    pub fn from_xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    pub fn points(self) -> Vec<Point> {
        vec![
            Point {
                x: self.x1,
                y: self.y1,
            },
            Point {
                x: self.x2,
                y: self.y1,
            },
            Point {
                x: self.x2,
                y: self.y2,
            },
            Point {
                x: self.x1,
                y: self.y2,
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub fill_color: String,
    pub fill_opacity: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub stroke_color: String,
    pub stroke_opacity: f64,
    pub stroke_width: f64,
    pub stroke_alignment: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub children: Vec<TextParagraphSet>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextParagraphSet {
    #[serde(rename = "type")]
    pub kind: String,
    pub children: Vec<TextParagraph>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextParagraph {
    #[serde(rename = "type")]
    pub kind: String,
    pub children: Vec<TextRun>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub text: String,
    pub font_family: String,
    pub font_size: String,
    pub font_weight: String,
    pub font_style: String,
    pub fills: Vec<Fill>,
}

impl TextContent {
    pub fn single_run(run: TextRun) -> Self {
        Self {
            kind: "root".to_owned(),
            children: vec![TextParagraphSet {
                kind: "paragraph-set".to_owned(),
                children: vec![TextParagraph {
                    kind: "paragraph".to_owned(),
                    children: vec![run],
                }],
            }],
        }
    }
}
