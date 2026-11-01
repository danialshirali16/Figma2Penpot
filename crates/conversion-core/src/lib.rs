#![forbid(unsafe_code)]
//! Figma → Penpot conversion passes for the M2 vertical slice.
//!
//! M2 supports pages, frames, groups, rectangles, ellipses, solid fills and
//! strokes, opacity, and basic single-run text. Every decision is emitted to a
//! [`DiagnosticSink`]; unsupported nodes and paints are skipped with a report
//! event, never silently.

use std::collections::HashMap;

use conversion_report::{DiagnosticSink, ElementRef, Outcome, ReportEvent, Severity};
use fig_reader::{Color, FigDocument, FigNode, Guid, Paint};
use penpot_writer::{
    Content, Fill, PathCommand, PathParams, PenpotDocument, PenpotPage, Shape, ShapeType, Stroke,
    SvgAttrs, TextContent, TextRun,
};
use uuid::Uuid;

/// Fixed namespace for deterministic Penpot UUIDs derived from Figma GUIDs.
/// Stable IDs make golden reports reproducible; Penpot accepts any UUID.
const ID_NAMESPACE: Uuid = Uuid::from_u128(0x6d2f_9a51_4c7e_4b1d_9f3a_2c8e_7d6b_5a41);

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("the document has no visible pages")]
    NoVisiblePages,
    #[error("conversion failed: {0}")]
    Invalid(String),
}

/// Options for the M2 slice. Kept small until the settings UI lands.
#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    /// Override the Penpot file name. Defaults to `meta.json`'s name or the input stem.
    pub file_name: Option<String>,
}

/// Convert a decoded Figma document into a Penpot document.
///
/// The caller owns the sink and can finish a [`conversion_report::FileReport`]
/// after validation. Errors here are document-level; node-level failures are
/// reported and contained.
pub fn convert(
    document: &FigDocument,
    options: &ConvertOptions,
    sink: &mut dyn DiagnosticSink,
) -> Result<PenpotDocument, ConvertError> {
    let tree = Tree::new(&document.nodes);
    let file_name = options
        .file_name
        .clone()
        .or_else(|| document.meta_file_name().map(str::to_owned))
        .unwrap_or_else(|| "Untitled".to_owned());
    let file_id = id_for("file", &document.source_name);
    let mut penpot = PenpotDocument::new(file_id, file_name);

    let mut visible_pages = document
        .pages
        .iter()
        .filter(|page| !page.internal_only)
        .peekable();
    if visible_pages.peek().is_none() {
        return Err(ConvertError::NoVisiblePages);
    }

    for (page_index, page) in visible_pages.enumerate() {
        let page_id = id_for_guid("page", page.guid);
        let root_id = Uuid::nil();
        let mut shapes = Vec::new();
        let path = vec![page.name.clone()];
        let children = tree.children_of(page.guid);
        for child in children {
            if let Some(mut child_shapes) = convert_node(
                document,
                &tree,
                child,
                root_id,
                root_id,
                Affine::IDENTITY,
                page_id,
                &page.name,
                &path,
                sink,
            ) {
                shapes.append(&mut child_shapes);
            }
        }
        let root_children = shapes
            .iter()
            .filter(|shape| shape.parent_id.is_nil())
            .map(|shape| shape.id)
            .collect();
        let root_frame = Shape::root_frame(page_id, root_children);
        penpot.pages.push(PenpotPage {
            id: page_id,
            name: page.name.clone(),
            index: page_index,
            background: None,
            root_frame,
            shapes,
        });
    }

    Ok(penpot)
}

struct Tree<'a> {
    nodes: &'a [FigNode],
    children: HashMap<Guid, Vec<usize>>,
}

impl<'a> Tree<'a> {
    fn new(nodes: &'a [FigNode]) -> Self {
        let mut children: HashMap<Guid, Vec<usize>> = HashMap::new();
        for (index, node) in nodes.iter().enumerate() {
            if node.removed {
                continue;
            }
            if let Some(parent) = node.parent {
                children.entry(parent).or_default().push(index);
            }
        }
        for indices in children.values_mut() {
            indices.sort_by(|left, right| {
                let a = nodes[*left].position.as_deref().unwrap_or("");
                let b = nodes[*right].position.as_deref().unwrap_or("");
                a.cmp(b)
                    .then_with(|| nodes[*left].guid.cmp(&nodes[*right].guid))
            });
        }
        Self { nodes, children }
    }

    fn children_of(&self, parent: Guid) -> Vec<usize> {
        self.children.get(&parent).cloned().unwrap_or_default()
    }

    fn node(&self, index: usize) -> &'a FigNode {
        &self.nodes[index]
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_node(
    document: &FigDocument,
    tree: &Tree<'_>,
    index: usize,
    parent_id: Uuid,
    frame_id: Uuid,
    parent_abs: Affine,
    page_id: Uuid,
    page_name: &str,
    path: &[String],
    sink: &mut dyn DiagnosticSink,
) -> Option<Vec<Shape>> {
    let node = tree.node(index);
    let id = id_for_guid("shape", node.guid);
    let mut element_path = path.to_vec();
    element_path.push(node.name.clone());
    let element = element_ref(node, page_name, &element_path);
    sink.count_detected(&element);

    let kind = match node.kind.as_str() {
        "FRAME" => {
            if node.resize_to_fit == Some(true) {
                NodeClass::Group
            } else {
                NodeClass::Frame
            }
        }
        "GROUP" => NodeClass::Group,
        "RECTANGLE" | "ROUNDED_RECTANGLE" => NodeClass::Rect,
        "ELLIPSE" => NodeClass::Circle,
        "TEXT" => NodeClass::Text,
        "VECTOR" | "STAR" | "LINE" | "REGULAR_POLYGON" => NodeClass::Vector,
        _ => {
            emit(
                sink,
                &element,
                "F2P-NODE-UNSUPPORTED",
                Severity::Warning,
                Outcome::Unsupported,
                format!("Figma node type {} is not converted", node.kind),
                Some("not implemented in the M2 conversion slice".to_owned()),
                "skipped the node and its subtree".to_owned(),
                Some("Convert this node manually in Penpot.".to_owned()),
            );
            return None;
        }
    };

    let local = node.transform.map(Affine::from).unwrap_or(Affine::IDENTITY);
    let absolute = parent_abs.multiply(local);
    let (mut width, mut height) = node
        .size
        .map(|size| (size.width as f64, size.height as f64))
        .unwrap_or((0.0, 0.0));
    let mut modified = false;
    let mut reasons: Vec<String> = Vec::new();

    if !absolute.is_translation_scale() {
        modified = true;
        reasons.push("rotation or skew was approximated".to_owned());
    }
    if (absolute.m00.abs() - 1.0).abs() > 1e-4 || (absolute.m11.abs() - 1.0).abs() > 1e-4 {
        modified = true;
        reasons.push("scale was applied to bounds".to_owned());
        width *= absolute.m00.abs();
        height *= absolute.m11.abs();
    }
    if !width.is_finite() || !height.is_finite() || !absolute.is_finite() {
        emit(
            sink,
            &element,
            "F2P-NODE-FAILED",
            Severity::Error,
            Outcome::Failed,
            "node has non-finite geometry".to_owned(),
            Some("Figma size or transform contained NaN or infinity".to_owned()),
            "skipped the node and its subtree".to_owned(),
            Some("Repair the node in Figma and retry.".to_owned()),
        );
        return None;
    }

    if kind == NodeClass::Vector {
        return convert_vector_node(
            document, node, &element, absolute, width, height, parent_id, frame_id, page_id, sink,
        );
    }

    let mut descendants = Vec::new();
    let shape = match kind {
        NodeClass::Frame | NodeClass::Group => {
            let mut immediate = Vec::new();
            let child_frame = if kind == NodeClass::Frame {
                id
            } else {
                frame_id
            };
            for child in tree.children_of(node.guid) {
                if let Some(child_shapes) = convert_node(
                    document,
                    tree,
                    child,
                    id,
                    child_frame,
                    absolute,
                    page_id,
                    page_name,
                    &element_path,
                    sink,
                ) {
                    for shape in child_shapes {
                        if shape.parent_id == id {
                            immediate.push(shape);
                        } else {
                            descendants.push(shape);
                        }
                    }
                }
            }
            if kind == NodeClass::Group && immediate.is_empty() {
                emit(
                    sink,
                    &element,
                    "F2P-GROUP-EMPTY",
                    Severity::Warning,
                    Outcome::Skipped,
                    "empty Figma group has no Penpot equivalent".to_owned(),
                    Some("Penpot does not keep empty groups".to_owned()),
                    "skipped the group".to_owned(),
                    None,
                );
                return None;
            }
            let (x, y, w, h) = if kind == NodeClass::Group {
                bounds(&immediate).unwrap_or((absolute.m02, absolute.m12, width, height))
            } else {
                (absolute.m02, absolute.m12, width, height)
            };
            let child_ids = immediate.iter().map(|child| child.id).collect();
            descendants.extend(immediate);
            let mut shape = Shape::new(
                id,
                node.name.clone(),
                if kind == NodeClass::Frame {
                    ShapeType::Frame
                } else {
                    ShapeType::Group
                },
                x,
                y,
                w,
                h,
                parent_id,
                frame_id,
                page_id,
            );
            shape.shapes = Some(child_ids);
            shape.proportion = Some(1.0);
            shape.proportion_lock = Some(false);
            if kind == NodeClass::Frame {
                shape.hide_fill_on_export = Some(false);
            }
            shape
        }
        NodeClass::Rect | NodeClass::Circle | NodeClass::Text => {
            let mut shape = Shape::new(
                id,
                node.name.clone(),
                match kind {
                    NodeClass::Rect => ShapeType::Rect,
                    NodeClass::Circle => ShapeType::Circle,
                    NodeClass::Text => ShapeType::Text,
                    _ => unreachable!(),
                },
                absolute.m02,
                absolute.m12,
                width,
                height,
                parent_id,
                frame_id,
                page_id,
            );
            shape.proportion = Some(1.0);
            shape.proportion_lock = Some(false);
            shape
        }
        NodeClass::Vector => unreachable!("vector nodes are handled before shape construction"),
    };
    let mut shape = shape;

    if node.visible == Some(false) {
        shape.hidden = Some(true);
    }
    let opacity = node.opacity.unwrap_or(1.0) as f64;
    if opacity.is_finite() {
        shape.opacity = opacity.clamp(0.0, 1.0);
    } else {
        modified = true;
        reasons.push("non-finite opacity was replaced with 1".to_owned());
    }

    let (fills, fills_modified) = map_fills(sink, &element, &node.fills);
    shape.fills = fills;
    if fills_modified {
        modified = true;
        reasons.push("one or more paints were skipped".to_owned());
    }
    let (strokes, strokes_modified) = map_strokes(sink, &element, node);
    shape.strokes = strokes;
    if strokes_modified {
        modified = true;
        reasons.push("one or more strokes were skipped".to_owned());
    }

    if kind == NodeClass::Rect {
        let radii = node
            .corner_radii
            .or_else(|| node.corner_radius.map(|radius| [radius; 4]));
        if let Some(radii) = radii {
            shape.r1 = Some(radii[0] as f64);
            shape.r2 = Some(radii[1] as f64);
            shape.r3 = Some(radii[2] as f64);
            shape.r4 = Some(radii[3] as f64);
        }
    }

    if kind == NodeClass::Text {
        match map_text(node, &shape.fills) {
            Ok((content, grow_type, text_modified, text_reason)) => {
                shape.content = Some(Content::Text(content));
                shape.grow_type = Some(grow_type);
                if text_modified {
                    modified = true;
                    if let Some(reason) = text_reason {
                        reasons.push(reason);
                    }
                }
            }
            Err(reason) => {
                emit(
                    sink,
                    &element,
                    "F2P-TEXT-EMPTY",
                    Severity::Warning,
                    Outcome::Skipped,
                    "text node has no usable characters".to_owned(),
                    Some(reason),
                    "skipped the text node".to_owned(),
                    Some("Add text content in Figma and retry.".to_owned()),
                );
                return None;
            }
        }
    }

    if modified {
        emit(
            sink,
            &element,
            "F2P-NODE-APPROXIMATED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            format!("{} was converted with modifications", node.kind),
            Some(reasons.join("; ")),
            format!("converted as {}", shape.kind.as_str()),
            Some("Review the shape in Penpot after import.".to_owned()),
        );
    } else {
        emit(
            sink,
            &element,
            "F2P-NODE-CONVERTED",
            Severity::Info,
            Outcome::Converted,
            format!("{} was converted", node.kind),
            None,
            format!("converted as {}", shape.kind.as_str()),
            None,
        );
    }

    let mut result = Vec::with_capacity(descendants.len() + 1);
    result.push(shape);
    result.append(&mut descendants);
    Some(result)
}

#[allow(clippy::too_many_arguments)]
fn convert_vector_node(
    document: &FigDocument,
    node: &FigNode,
    element: &ElementRef,
    absolute: Affine,
    width: f64,
    height: f64,
    parent_id: Uuid,
    frame_id: Uuid,
    page_id: Uuid,
    sink: &mut dyn DiagnosticSink,
) -> Option<Vec<Shape>> {
    let using_fill = !node.fill_geometry.is_empty();
    let geometries = if using_fill {
        &node.fill_geometry
    } else {
        &node.stroke_geometry
    };
    if geometries.is_empty() {
        emit(
            sink,
            element,
            "F2P-VECTOR-EMPTY",
            Severity::Warning,
            Outcome::Unsupported,
            "vector node has no geometry blobs".to_owned(),
            Some("fillGeometry and strokeGeometry are both empty".to_owned()),
            "skipped the vector".to_owned(),
            Some("Recreate the vector manually in Penpot.".to_owned()),
        );
        return None;
    }

    let (mut fills, fills_modified) = map_fills(sink, element, &node.fills);
    let (mut strokes, strokes_modified) = map_strokes(sink, element, node);
    let using_stroke_outline = !using_fill;
    if using_stroke_outline {
        fills = fills_from_strokes(node);
        strokes.clear();
        emit(
            sink,
            element,
            "F2P-VECTOR-STROKE-OUTLINE",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "vector has only stroke geometry".to_owned(),
            Some(
                "Figma stores an expanded stroke outline, not editable stroke geometry".to_owned(),
            ),
            "converted the outline as a filled path".to_owned(),
            Some("Convert to an editable stroke manually in Penpot.".to_owned()),
        );
    }
    if node.has_vector_network {
        emit(
            sink,
            element,
            "F2P-VECTOR-NETWORK-LOST",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "vector network handle metadata was not preserved".to_owned(),
            Some(
                "M3.1 converts commandsBlob geometry into path commands, not the editable network"
                    .to_owned(),
            ),
            "kept the path geometry; dropped network handles".to_owned(),
            Some(
                "Re-edit the path handles in Penpot if exact curvature control matters.".to_owned(),
            ),
        );
    }

    let base_id = id_for_guid("shape", node.guid);
    let mut shapes = Vec::with_capacity(geometries.len());
    for (order, geometry) in geometries.iter().enumerate() {
        let Some(blob) = document.blob(geometry.commands_blob as usize) else {
            emit(
                sink,
                element,
                "F2P-VECTOR-FAILED",
                Severity::Error,
                Outcome::Failed,
                "vector geometry blob is missing".to_owned(),
                Some(format!(
                    "blob index {} is out of range",
                    geometry.commands_blob
                )),
                "skipped the vector".to_owned(),
                Some("Repair the file in Figma and retry.".to_owned()),
            );
            return None;
        };
        let commands = match decode_path_commands(blob, absolute) {
            Ok(commands) => commands,
            Err(reason) => {
                emit(
                    sink,
                    element,
                    "F2P-VECTOR-FAILED",
                    Severity::Error,
                    Outcome::Failed,
                    "vector geometry could not be decoded".to_owned(),
                    Some(reason),
                    "skipped the vector".to_owned(),
                    Some("Repair the vector in Figma and retry.".to_owned()),
                );
                return None;
            }
        };
        let id = if geometries.len() > 1 {
            Uuid::new_v5(
                &ID_NAMESPACE,
                format!(
                    "shape:{}:{}:{}",
                    node.guid.session_id, node.guid.local_id, order
                )
                .as_bytes(),
            )
        } else {
            base_id
        };
        let mut shape = Shape::new(
            id,
            if geometries.len() > 1 {
                format!("{} {}", node.name, order + 1)
            } else {
                node.name.clone()
            },
            ShapeType::Path,
            absolute.m02,
            absolute.m12,
            width,
            height,
            parent_id,
            frame_id,
            page_id,
        );
        shape.content = Some(Content::Path(commands));
        shape.svg_attrs = Some(SvgAttrs {
            fill_rule: if geometry.winding_rule == "ODD" {
                "evenodd".to_owned()
            } else {
                "nonzero".to_owned()
            },
        });
        shape.fills = fills.clone();
        shape.strokes = strokes.clone();
        shape.proportion = Some(1.0);
        shape.proportion_lock = Some(false);
        if node.visible == Some(false) {
            shape.hidden = Some(true);
        }
        let opacity = node.opacity.unwrap_or(1.0) as f64;
        if opacity.is_finite() {
            shape.opacity = opacity.clamp(0.0, 1.0);
        }
        shapes.push(shape);
    }

    let modified =
        using_stroke_outline || fills_modified || strokes_modified || node.has_vector_network;
    emit(
        sink,
        element,
        if modified {
            "F2P-NODE-APPROXIMATED"
        } else {
            "F2P-NODE-CONVERTED"
        },
        if modified {
            Severity::Warning
        } else {
            Severity::Info
        },
        if modified {
            Outcome::ConvertedWithModification
        } else {
            Outcome::Converted
        },
        format!("{} was converted as {} path(s)", node.kind, shapes.len()),
        if modified {
            Some("vector geometry required an approximation".to_owned())
        } else {
            None
        },
        format!("converted as {} path shape(s)", shapes.len()),
        if modified {
            Some("Review the vector in Penpot after import.".to_owned())
        } else {
            None
        },
    );
    Some(shapes)
}

fn fills_from_strokes(node: &FigNode) -> Vec<Fill> {
    node.strokes
        .iter()
        .filter(|paint| paint.visible)
        .filter_map(|paint| {
            if paint.kind != "SOLID" {
                return None;
            }
            let color = paint.color?;
            if !color.is_finite() {
                return None;
            }
            Some(Fill {
                fill_color: hex(color),
                fill_opacity: (color.a as f64 * paint.opacity as f64).clamp(0.0, 1.0),
            })
        })
        .collect()
}

fn decode_path_commands(bytes: &[u8], matrix: Affine) -> Result<Vec<PathCommand>, String> {
    let mut index = 0usize;
    let mut commands = Vec::new();
    while index < bytes.len() {
        let opcode = bytes[index];
        index += 1;
        match opcode {
            0x00 => commands.push(PathCommand {
                command: "close-path".to_owned(),
                params: PathParams::default(),
            }),
            0x01 => {
                let (x, y) = read_point(bytes, &mut index, matrix)?;
                commands.push(PathCommand {
                    command: "move-to".to_owned(),
                    params: PathParams {
                        x: Some(x),
                        y: Some(y),
                        ..PathParams::default()
                    },
                });
            }
            0x02 => {
                let (x, y) = read_point(bytes, &mut index, matrix)?;
                commands.push(PathCommand {
                    command: "line-to".to_owned(),
                    params: PathParams {
                        x: Some(x),
                        y: Some(y),
                        ..PathParams::default()
                    },
                });
            }
            0x04 => {
                let (c1x, c1y) = read_point(bytes, &mut index, matrix)?;
                let (c2x, c2y) = read_point(bytes, &mut index, matrix)?;
                let (x, y) = read_point(bytes, &mut index, matrix)?;
                commands.push(PathCommand {
                    command: "curve-to".to_owned(),
                    params: PathParams {
                        x: Some(x),
                        y: Some(y),
                        c1x: Some(c1x),
                        c1y: Some(c1y),
                        c2x: Some(c2x),
                        c2y: Some(c2y),
                    },
                });
            }
            other => {
                return Err(format!(
                    "unknown command opcode 0x{other:02X} at byte {}",
                    index - 1
                ))
            }
        }
    }
    if commands.is_empty() {
        return Err("geometry blob is empty".to_owned());
    }
    Ok(commands)
}

fn read_point(bytes: &[u8], index: &mut usize, matrix: Affine) -> Result<(f64, f64), String> {
    let x = read_f32(bytes, index)? as f64;
    let y = read_f32(bytes, index)? as f64;
    let tx = matrix.m00 * x + matrix.m01 * y + matrix.m02;
    let ty = matrix.m10 * x + matrix.m11 * y + matrix.m12;
    Ok((round_coord(tx), round_coord(ty)))
}

fn read_f32(bytes: &[u8], index: &mut usize) -> Result<f32, String> {
    let end = index
        .checked_add(4)
        .ok_or_else(|| "geometry offset overflow".to_owned())?;
    let slice = bytes
        .get(*index..end)
        .ok_or_else(|| "geometry blob ended inside a coordinate".to_owned())?;
    *index = end;
    Ok(f32::from_le_bytes(slice.try_into().map_err(|_| {
        "geometry coordinate has wrong size".to_owned()
    })?))
}

fn round_coord(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeClass {
    Frame,
    Group,
    Rect,
    Circle,
    Text,
    Vector,
}

fn bounds(children: &[Shape]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for child in children {
        if child.width <= 0.0 || child.height <= 0.0 {
            continue;
        }
        min_x = min_x.min(child.x);
        min_y = min_y.min(child.y);
        max_x = max_x.max(child.x + child.width);
        max_y = max_y.max(child.y + child.height);
    }
    if !min_x.is_finite() || !min_y.is_finite() {
        None
    } else {
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

fn map_fills(
    sink: &mut dyn DiagnosticSink,
    element: &ElementRef,
    paints: &[Paint],
) -> (Vec<Fill>, bool) {
    let mut fills = Vec::new();
    let mut modified = false;
    for paint in paints {
        if !paint.visible {
            continue;
        }
        match solid_color(paint) {
            Some((color, opacity)) => fills.push(Fill {
                fill_color: color,
                fill_opacity: opacity,
            }),
            None => {
                modified = true;
                emit(
                    sink,
                    element,
                    "F2P-PAINT-UNSUPPORTED",
                    Severity::Warning,
                    Outcome::ConvertedWithModification,
                    format!("{} fill was skipped", paint.kind),
                    Some("only solid paints are supported in M2".to_owned()),
                    "kept the remaining paints".to_owned(),
                    Some("Recreate this fill manually in Penpot.".to_owned()),
                );
            }
        }
    }
    (fills, modified)
}

fn map_strokes(
    sink: &mut dyn DiagnosticSink,
    element: &ElementRef,
    node: &FigNode,
) -> (Vec<Stroke>, bool) {
    let weight = node.stroke_weight.unwrap_or(0.0) as f64;
    if !weight.is_finite() {
        emit(
            sink,
            element,
            "F2P-STROKE-WIDTH",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "stroke width is not finite".to_owned(),
            Some("Figma encoded NaN or infinity".to_owned()),
            "skipped the strokes".to_owned(),
            Some("Set a valid stroke width in Penpot.".to_owned()),
        );
        return (Vec::new(), true);
    }
    if weight <= 0.0 || node.strokes.is_empty() {
        return (Vec::new(), false);
    }
    let mut modified = false;
    let alignment = match node.stroke_align.as_deref() {
        Some("INSIDE") => "inner",
        Some("OUTSIDE") => "outer",
        Some("CENTER") | None => "center",
        Some(other) => {
            modified = true;
            emit(
                sink,
                element,
                "F2P-STROKE-ALIGNMENT",
                Severity::Warning,
                Outcome::ConvertedWithModification,
                format!("stroke alignment {other} is unknown"),
                Some("the Figma enum is not recognized".to_owned()),
                "used center alignment".to_owned(),
                Some("Check the stroke alignment in Penpot.".to_owned()),
            );
            "center"
        }
    };
    let mut strokes = Vec::new();
    for paint in &node.strokes {
        if !paint.visible {
            continue;
        }
        match solid_color(paint) {
            Some((color, opacity)) => strokes.push(Stroke {
                stroke_color: color,
                stroke_opacity: opacity,
                stroke_width: weight,
                stroke_alignment: alignment.to_owned(),
            }),
            None => {
                modified = true;
                emit(
                    sink,
                    element,
                    "F2P-STROKE-UNSUPPORTED",
                    Severity::Warning,
                    Outcome::ConvertedWithModification,
                    format!("{} stroke was skipped", paint.kind),
                    Some("only solid strokes are supported in M2".to_owned()),
                    "kept the remaining strokes".to_owned(),
                    Some("Recreate this stroke manually in Penpot.".to_owned()),
                );
            }
        }
    }
    (strokes, modified)
}

fn solid_color(paint: &Paint) -> Option<(String, f64)> {
    if paint.kind != "SOLID" {
        return None;
    }
    let color = paint.color?;
    if !color.is_finite() {
        return None;
    }
    let opacity = (color.a as f64 * paint.opacity as f64).clamp(0.0, 1.0);
    Some((hex(color), opacity))
}

fn hex(color: Color) -> String {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

fn map_text(
    node: &FigNode,
    fills: &[Fill],
) -> Result<(TextContent, String, bool, Option<String>), String> {
    let text = node
        .text
        .as_ref()
        .ok_or_else(|| "missing textData".to_owned())?;
    let characters = text
        .characters
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "empty characters".to_owned())?;
    let mut modified = false;
    let mut reason = None;
    let font_family = text.font_family.clone().unwrap_or_else(|| {
        modified = true;
        reason = Some("font family was missing; used Inter".to_owned());
        "Inter".to_owned()
    });
    let font_size = text.font_size.unwrap_or_else(|| {
        modified = true;
        reason = Some("font size was missing; used 16".to_owned());
        16.0
    });
    let font_size = if font_size.is_finite() && font_size > 0.0 {
        font_size
    } else {
        modified = true;
        reason = Some("font size was invalid; used 16".to_owned());
        16.0
    };
    let style = text.font_style.as_deref().unwrap_or("Regular");
    let (weight, font_style) = font_properties(style);
    let (grow_type, resized) = match text.auto_resize.as_deref() {
        Some("HEIGHT") => ("auto-height", false),
        Some("WIDTH_AND_HEIGHT") => ("auto-width", true),
        _ => ("fixed", false),
    };
    if resized {
        modified = true;
        reason =
            Some("Figma width-and-height auto-resize has no exact Penpot equivalent".to_owned());
    }
    let run_fills = fills.to_vec();
    let run_fills = if run_fills.is_empty() {
        modified = true;
        reason = Some("text had no solid fill; used black".to_owned());
        vec![Fill {
            fill_color: "#000000".to_owned(),
            fill_opacity: 1.0,
        }]
    } else {
        run_fills
    };
    let content = TextContent::single_run(TextRun {
        text: characters.to_owned(),
        font_family,
        font_size: format_number(font_size as f64),
        font_weight: weight.to_owned(),
        font_style: font_style.to_owned(),
        fills: run_fills,
    });
    Ok((content, grow_type.to_owned(), modified, reason))
}

fn font_properties(style: &str) -> (&'static str, &'static str) {
    let lower = style.to_ascii_lowercase();
    let weight = if lower.contains("thin") {
        "100"
    } else if lower.contains("extralight") || lower.contains("extra light") {
        "200"
    } else if lower.contains("light") {
        "300"
    } else if lower.contains("medium") {
        "500"
    } else if lower.contains("semibold") || lower.contains("semi bold") {
        "600"
    } else if lower.contains("bold") {
        "700"
    } else if lower.contains("black") {
        "900"
    } else {
        "400"
    };
    let style = if lower.contains("italic") || lower.contains("oblique") {
        "italic"
    } else {
        "normal"
    };
    (weight, style)
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value}")
    }
}

fn element_ref(node: &FigNode, page: &str, path: &[String]) -> ElementRef {
    ElementRef {
        figma_node_id: node.guid.to_string(),
        name: node.name.clone(),
        element_type: node.kind.clone(),
        page: Some(page.to_owned()),
        path: path.to_vec(),
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
    sink: &mut dyn DiagnosticSink,
    element: &ElementRef,
    code: &str,
    severity: Severity,
    outcome: Outcome,
    description: impl Into<String>,
    reason: Option<String>,
    action_taken: impl Into<String>,
    suggested_action: Option<String>,
) {
    sink.emit(ReportEvent {
        code: code.to_owned(),
        severity,
        outcome,
        element: element.clone(),
        description: description.into(),
        reason,
        action_taken: action_taken.into(),
        suggested_action,
    });
}

fn id_for(kind: &str, value: &str) -> Uuid {
    Uuid::new_v5(&ID_NAMESPACE, format!("{kind}:{value}").as_bytes())
}

fn id_for_guid(kind: &str, guid: Guid) -> Uuid {
    id_for(kind, &format!("{}:{}", guid.session_id, guid.local_id))
}

#[derive(Debug, Clone, Copy)]
struct Affine {
    m00: f64,
    m01: f64,
    m02: f64,
    m10: f64,
    m11: f64,
    m12: f64,
}

impl Affine {
    const IDENTITY: Self = Self {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
    };

    fn multiply(self, other: Self) -> Self {
        Self {
            m00: self.m00 * other.m00 + self.m01 * other.m10,
            m01: self.m00 * other.m01 + self.m01 * other.m11,
            m02: self.m00 * other.m02 + self.m01 * other.m12 + self.m02,
            m10: self.m10 * other.m00 + self.m11 * other.m10,
            m11: self.m10 * other.m01 + self.m11 * other.m11,
            m12: self.m10 * other.m02 + self.m11 * other.m12 + self.m12,
        }
    }

    fn is_translation_scale(self) -> bool {
        self.m01.abs() < 1e-6 && self.m10.abs() < 1e-6
    }

    fn is_finite(self) -> bool {
        [self.m00, self.m01, self.m02, self.m10, self.m11, self.m12]
            .iter()
            .all(|value| value.is_finite())
    }
}

impl From<fig_reader::Matrix> for Affine {
    fn from(value: fig_reader::Matrix) -> Self {
        Self {
            m00: value.m00 as f64,
            m01: value.m01 as f64,
            m02: value.m02 as f64,
            m10: value.m10 as f64,
            m11: value.m11 as f64,
            m12: value.m12 as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conversion_report::ReportCollector;
    use fig_reader::{Matrix, Size, TextData};

    fn element() -> ElementRef {
        ElementRef {
            figma_node_id: "1:1".to_owned(),
            name: "Node".to_owned(),
            element_type: "RECTANGLE".to_owned(),
            page: Some("Page 1".to_owned()),
            path: vec!["Page 1".to_owned(), "Node".to_owned()],
        }
    }

    fn blank_node() -> FigNode {
        FigNode {
            guid: Guid::new(1, 1),
            parent: None,
            position: None,
            kind: "RECTANGLE".to_owned(),
            kind_raw: None,
            name: "Node".to_owned(),
            phase: None,
            visible: Some(true),
            size: Some(Size {
                width: 10.0,
                height: 10.0,
            }),
            transform: Some(Matrix::IDENTITY),
            internal_only: false,
            removed: false,
            opacity: Some(1.0),
            fills: Vec::new(),
            strokes: Vec::new(),
            stroke_weight: None,
            stroke_align: None,
            corner_radius: None,
            corner_radii: None,
            resize_to_fit: None,
            text: None,
            fill_geometry: Vec::new(),
            stroke_geometry: Vec::new(),
            has_vector_network: false,
        }
    }

    fn solid() -> Paint {
        Paint {
            kind: "SOLID".to_owned(),
            color: Some(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            opacity: 1.0,
            visible: true,
        }
    }

    #[test]
    fn gradient_paint_is_reported_and_skipped() {
        let mut sink = ReportCollector::new();
        let paints = [Paint {
            kind: "GRADIENT_LINEAR".to_owned(),
            color: None,
            opacity: 1.0,
            visible: true,
        }];
        let (fills, modified) = map_fills(&mut sink, &element(), &paints);
        assert!(fills.is_empty());
        assert!(modified);
        assert!(sink
            .events()
            .iter()
            .any(|event| event.code == "F2P-PAINT-UNSUPPORTED"));
    }

    #[test]
    fn unknown_stroke_alignment_is_reported() {
        let mut sink = ReportCollector::new();
        let mut node = blank_node();
        node.stroke_weight = Some(2.0);
        node.stroke_align = Some("DIAGONAL".to_owned());
        node.strokes = vec![solid()];
        let (strokes, modified) = map_strokes(&mut sink, &element(), &node);
        assert!(modified);
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].stroke_alignment, "center");
        assert!(sink
            .events()
            .iter()
            .any(|event| event.code == "F2P-STROKE-ALIGNMENT"));
    }

    #[test]
    fn empty_text_is_skipped() {
        let mut node = blank_node();
        node.kind = "TEXT".to_owned();
        node.text = Some(TextData {
            characters: Some(String::new()),
            ..TextData::default()
        });
        assert!(map_text(&node, &[]).is_err());
    }

    #[test]
    fn font_style_maps_to_penpot_properties() {
        assert_eq!(font_properties("Bold"), ("700", "normal"));
        assert_eq!(font_properties("SemiBold Italic"), ("600", "italic"));
        assert_eq!(font_properties("Regular"), ("400", "normal"));
    }

    #[test]
    fn color_conversion_clamps_and_uppercases() {
        assert_eq!(
            hex(Color {
                r: 1.0,
                g: 0.5,
                b: 0.0,
                a: 1.0
            }),
            "#FF8000"
        );
    }
}
