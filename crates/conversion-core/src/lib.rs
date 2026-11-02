#![forbid(unsafe_code)]
//! Figma → Penpot conversion passes for the M2 vertical slice.
//!
//! M2 supports pages, frames, groups, rectangles, ellipses, solid fills and
//! strokes, opacity, and basic single-run text. Every decision is emitted to a
//! [`DiagnosticSink`]; unsupported nodes and paints are skipped with a report
//! event, never silently.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use conversion_report::{DiagnosticSink, ElementRef, Outcome, ReportEvent, Severity};
use fig_reader::{Color, Effect, FigDocument, FigNode, Guid, Paint, Size, StyleRef, TextData};
use penpot_writer::{
    BackgroundBlur, Blur, ColorStyle, ComponentEntry, Content, Fill, GradientFill,
    GradientStop as PenpotGradientStop, ImageFill, LayoutGap, LayoutPadding, Media, MediaObject,
    PathCommand, PathParams, PenpotDocument, PenpotPage, Shadow, ShadowColor, Shape, ShapeType,
    Stroke, SvgAttrs, TextContent, TextRun, TypographyStyle, VariantProperty,
};
use uuid::Uuid;

/// Fixed namespace for deterministic Penpot UUIDs derived from Figma GUIDs.
/// Stable IDs make golden reports reproducible; Penpot accepts any UUID.
const ID_NAMESPACE: Uuid = Uuid::from_u128(0x6d2f_9a51_4c7e_4b1d_9f3a_2c8e_7d6b_5a41);

/// Default bound on shapes created by component expansion and document traversal.
pub const DEFAULT_MAX_EXPANDED_SHAPES: usize = 250_000;
const MAX_CONVERSION_HIERARCHY_DEPTH: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("the document has no visible pages")]
    NoVisiblePages,
    #[error("conversion failed: {0}")]
    Invalid(String),
}

/// Conversion options exposed by the desktop settings.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Override the Penpot file name. Defaults to `meta.json`'s name or the input stem.
    pub file_name: Option<String>,
    /// Preserve token/style references. When false, resolved values are kept but links are
    /// removed ("Looks identical" in the desktop settings).
    pub keep_token_bindings: bool,
    /// Maximum expanded shapes, including restored component definitions.
    pub max_expanded_shapes: usize,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            file_name: None,
            keep_token_bindings: true,
            max_expanded_shapes: DEFAULT_MAX_EXPANDED_SHAPES,
        }
    }
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
    let assets = AssetRegistry::new(document);
    let file_name = options
        .file_name
        .clone()
        .or_else(|| document.meta_file_name().map(str::to_owned))
        .unwrap_or_else(|| "Untitled".to_owned());
    let file_id = id_for("file", &document.source_name);
    let styles = StyleLibrary::new(document, &file_id);
    let tokens = TokenLibrary::new(document, sink);
    let components = build_component_registry(
        document,
        &assets,
        &styles,
        &tokens,
        &tree,
        options.max_expanded_shapes,
        sink,
    )?;
    let mut penpot = PenpotDocument::new(file_id, file_name);
    let mut shape_count = components.library_shapes.len();

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
                &assets,
                &styles,
                &tokens,
                &components,
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
                shape_count = shape_count.saturating_add(child_shapes.len());
                if shape_count > options.max_expanded_shapes {
                    return Err(ConvertError::Invalid(format!(
                        "expanded shape count exceeds limit {}",
                        options.max_expanded_shapes
                    )));
                }
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
    if !components.library_shapes.is_empty() {
        let page_id = id_for("component-library-page", &document.source_name);
        let roots = components
            .library_shapes
            .iter()
            .filter(|shape| shape.parent_id.is_nil())
            .map(|shape| shape.id)
            .collect();
        penpot.pages.push(PenpotPage {
            id: page_id,
            name: "Imported components".to_owned(),
            index: penpot.pages.len(),
            background: None,
            root_frame: Shape::root_frame(page_id, roots),
            shapes: components.library_shapes,
        });
    }
    penpot.media = assets.media;
    penpot.objects = assets.objects;
    penpot.colors = styles.colors.values().cloned().collect();
    penpot.typographies = styles.typographies.values().cloned().collect();
    penpot.components = components.entries;
    penpot.tokens = tokens.document;
    detach_broken_component_refs(&mut penpot, sink);
    if !options.keep_token_bindings {
        inline_token_links(&mut penpot);
    }

    Ok(penpot)
}

#[derive(Default)]
struct TokenLibrary {
    document: Option<serde_json::Value>,
    names_by_key: HashMap<String, String>,
}

struct VariableSetInfo<'a> {
    guid: Guid,
    asset_key: Option<&'a str>,
    name: &'a str,
    modes: Vec<fig_reader::VariableMode>,
}

impl TokenLibrary {
    fn new(document: &FigDocument, sink: &mut dyn DiagnosticSink) -> Self {
        let mut sets = Vec::new();
        for node in &document.nodes {
            if node.kind != "VARIABLE_SET" {
                continue;
            }
            sets.push(VariableSetInfo {
                guid: node.guid,
                asset_key: node.variable_key.as_deref(),
                name: node.name.trim(),
                modes: node.variable_modes.clone(),
            });
        }
        if sets.is_empty() {
            return Self::default();
        }
        let mut set_names: HashMap<Guid, Vec<String>> = HashMap::new();
        let mut set_order = Vec::new();
        let mut themes: HashMap<String, Vec<String>> = HashMap::new();
        for set in &sets {
            let names = if set.modes.len() > 1 {
                set.modes
                    .iter()
                    .map(|mode| format!("{} / {}", set.name, mode.name))
                    .collect()
            } else {
                vec![set.name.to_owned()]
            };
            for (index, name) in names.iter().enumerate() {
                if !set_order.contains(name) {
                    set_order.push(name.clone());
                }
                let theme_name = set
                    .modes
                    .get(index)
                    .or_else(|| set.modes.first())
                    .map(|mode| mode.name.clone())
                    .unwrap_or_else(|| "Default".to_owned());
                themes.entry(theme_name).or_default().push(name.clone());
            }
            set_names.insert(set.guid, names);
        }

        let mut token_sets: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        let mut names_by_key = HashMap::new();
        let mut variables = Vec::new();
        let mut used_names: HashMap<(Option<Guid>, String), String> = HashMap::new();
        for node in &document.nodes {
            if node.kind != "VARIABLE" {
                continue;
            }
            let key = node
                .variable_key
                .clone()
                .unwrap_or_else(|| node.guid.to_string());
            let set_guid = node
                .variable_set_id
                .as_ref()
                .and_then(|reference| reference.guid)
                .or_else(|| {
                    node.variable_set_id
                        .as_ref()
                        .and_then(|reference| reference.asset_key.as_deref())
                        .and_then(|key| {
                            sets.iter()
                                .find(|set| set.asset_key == Some(key))
                                .map(|set| set.guid)
                        })
                });
            let base_name = normalize_token_name(&node.name);
            let mut name = base_name.clone();
            let mut suffix = 0usize;
            while used_names
                .get(&(set_guid, name.clone()))
                .is_some_and(|existing| existing != &key)
            {
                suffix += 1;
                name = format!(
                    "{base_name}_{}_{}_{suffix}",
                    node.guid.session_id, node.guid.local_id
                );
            }
            used_names.insert((set_guid, name.clone()), key.clone());
            let canonical = node
                .name
                .trim()
                .split('/')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(".");
            if name != canonical {
                let element = ElementRef {
                    figma_node_id: node.guid.to_string(),
                    name: node.name.clone(),
                    element_type: node.kind.clone(),
                    page: None,
                    path: vec![node.name.clone()],
                };
                sink.count_detected(&element);
                emit(sink, &element, "F2P-TOKEN-NAME-NORMALIZED", Severity::Warning,
                    Outcome::ConvertedWithModification, "token name was adapted for Penpot",
                    Some("Penpot token names have a restricted character set and must be unique within a collection".to_owned()),
                    format!("renamed the token to {name:?} and updated converted bindings"),
                    Some("Use the adjusted token name when connecting other designs.".to_owned()));
            }
            names_by_key.insert(key.clone(), name.clone());
            variables.push((node, key, name, set_guid));
        }

        for (node, key, name, set_guid) in variables {
            let names = set_guid
                .and_then(|guid| set_names.get(&guid).cloned())
                .unwrap_or_else(|| vec!["Figma Variables".to_owned()]);
            let mode_count = set_guid
                .and_then(|guid| sets.iter().find(|set| set.guid == guid))
                .map(|set| set.modes.len())
                .unwrap_or(0);
            for set_name in &names {
                let value = if mode_count > 1 {
                    let mode_name = set_name.rsplit(" / ").next().unwrap_or("");
                    let mode_id = set_guid
                        .and_then(|guid| sets.iter().find(|set| set.guid == guid))
                        .and_then(|set| {
                            set.modes
                                .iter()
                                .find(|mode| mode.name == mode_name)
                                .map(|mode| mode.id)
                        });
                    mode_id
                        .and_then(|mode_id| {
                            node.variable_values
                                .iter()
                                .find(|value| value.mode_id == mode_id)
                        })
                        .or_else(|| node.variable_values.first())
                } else {
                    node.variable_values.first()
                };
                let Some(value) = value else {
                    continue;
                };
                let Some((token_value, token_type)) = token_value(value, &names_by_key) else {
                    continue;
                };
                let token = serde_json::json!({
                    "$value": token_value,
                    "$type": token_type,
                    "$description": node.description.clone().unwrap_or_default(),
                });
                insert_token(&mut token_sets, set_name, &name, token);
                let _ = &key;
            }
        }

        let theme_values: Vec<serde_json::Value> = themes
            .iter()
            .map(|(name, selected)| {
                let selected: serde_json::Map<String, serde_json::Value> = selected
                    .iter()
                    .map(|set| (set.clone(), serde_json::json!("enabled")))
                    .collect();
                serde_json::json!({
                    "id": id_for("theme", name),
                    "name": name,
                    "group": "",
                    "description": "",
                    "isSource": true,
                    "selectedTokenSets": selected,
                })
            })
            .collect();

        let default_sets: Vec<String> = sets
            .iter()
            .filter_map(|set| {
                set_names
                    .get(&set.guid)
                    .and_then(|names| names.first().cloned())
            })
            .collect();
        let mut root = token_sets;
        root.insert("$themes".to_owned(), serde_json::Value::Array(theme_values));
        root.insert(
            "$metadata".to_owned(),
            serde_json::json!({
                "tokenSetOrder": set_order,
                "activeThemes": [],
                "activeSets": default_sets,
            }),
        );
        Self {
            document: Some(serde_json::Value::Object(root)),
            names_by_key,
        }
    }

    fn name_for(&self, key: &str) -> Option<&str> {
        self.names_by_key.get(key).map(String::as_str)
    }
}

fn token_value(
    value: &fig_reader::VariableValue,
    names_by_key: &HashMap<String, String>,
) -> Option<(serde_json::Value, &'static str)> {
    if let Some(alias) = &value.alias_key {
        let name = names_by_key.get(alias)?;
        return Some((serde_json::json!(format!("{{{name}}}")), "string"));
    }
    if let Some(color) = value.color {
        return Some((serde_json::json!(hex(color)), "color"));
    }
    if let Some(number) = value.number {
        return Some((serde_json::json!(format_number(number as f64)), "number"));
    }
    if let Some(text) = &value.text {
        return Some((serde_json::json!(text), "string"));
    }
    if let Some(boolean) = value.boolean {
        return Some((serde_json::json!(boolean.to_string()), "boolean"));
    }
    match value.kind.as_str() {
        "COLOR" => None,
        _ => None,
    }
}

fn map_blend_mode(
    node: &FigNode,
    element: &ElementRef,
    sink: &mut dyn DiagnosticSink,
) -> (Option<String>, bool) {
    let Some(blend) = node.blend_mode.as_deref() else {
        return (None, false);
    };
    if blend == "NORMAL" {
        return (None, false);
    }
    let mapped = blend.to_ascii_lowercase().replace('_', "-");
    if penpot_writer::is_valid_blend_mode(&mapped) {
        return (Some(mapped), false);
    }
    let substitute = match blend {
        "LINEAR_BURN" => "multiply",
        "LINEAR_DODGE" => "screen",
        _ => "normal",
    };
    emit(
        sink,
        element,
        "F2P-BLEND-MODE-LOST",
        Severity::Warning,
        Outcome::ConvertedWithModification,
        format!("{blend} blend mode has no supported Penpot equivalent"),
        Some("the target Penpot schema rejects this blend mode".to_owned()),
        format!("approximated using {substitute} blending"),
        Some("Review this layer's blending in Penpot.".to_owned()),
    );
    (
        if substitute == "normal" {
            None
        } else {
            Some(substitute.to_owned())
        },
        true,
    )
}

fn layout_number(value: Option<f32>, fallback: f64, field: &str, reasons: &mut Vec<String>) -> f64 {
    match value.map(f64::from) {
        Some(value)
            if value.is_finite() && (i32::MIN as f64..=i32::MAX as f64).contains(&value) =>
        {
            value
        }
        Some(_) => {
            reasons.push(format!("source {field} is unknown or outside Penpot's range; used {fallback} with the mapped distribution settings"));
            fallback
        }
        None => fallback,
    }
}

fn normalize_token_name(name: &str) -> String {
    let normalized = name
        .trim()
        .split(['/', '.'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .enumerate()
        .map(|(segment, part)| {
            part.chars()
                .enumerate()
                .map(|(index, character)| {
                    if character.is_ascii_alphanumeric()
                        || matches!(character, '_' | '-')
                        || (character == '$' && (segment > 0 || index > 0))
                    {
                        character.to_string()
                    } else {
                        format!("_u{:X}_", character as u32)
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(".");
    if normalized.is_empty() {
        "token".to_owned()
    } else {
        normalized
    }
}

fn insert_token(
    root: &mut serde_json::Map<String, serde_json::Value>,
    set: &str,
    name: &str,
    token: serde_json::Value,
) {
    let set_entry = root
        .entry(set.to_owned())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let serde_json::Value::Object(set_map) = set_entry else {
        return;
    };
    let mut current = set_map;
    let parts: Vec<&str> = name.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return;
    }
    for part in &parts[..parts.len() - 1] {
        current = current
            .entry((*part).to_owned())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("token group is always an object"));
    }
    current.insert(parts[parts.len() - 1].to_owned(), token);
}

struct StyleLibrary {
    file_id: Uuid,
    colors: HashMap<Guid, ColorStyle>,
    color_paints: HashMap<Guid, Vec<Paint>>,
    typographies: HashMap<Guid, TypographyStyle>,
    text_styles: HashMap<Guid, TextData>,
    effect_styles: HashMap<Guid, Vec<Effect>>,
    grid_styles: HashMap<Guid, String>,
}

impl StyleLibrary {
    fn new(document: &FigDocument, file_id: &Uuid) -> Self {
        let mut library = Self {
            file_id: *file_id,
            colors: HashMap::new(),
            color_paints: HashMap::new(),
            typographies: HashMap::new(),
            text_styles: HashMap::new(),
            effect_styles: HashMap::new(),
            grid_styles: HashMap::new(),
        };
        for node in &document.nodes {
            let Some(style_type) = node.style_type.as_deref() else {
                continue;
            };
            let (name, path) = split_style_name(&node.name);
            match style_type {
                "FILL" => {
                    let Some(paint) = node
                        .fills
                        .iter()
                        .find(|paint| paint.visible && paint.kind == "SOLID")
                    else {
                        continue;
                    };
                    let Some(color) = paint.color else {
                        continue;
                    };
                    let id = id_for_guid("color-style", node.guid);
                    library.colors.insert(
                        node.guid,
                        ColorStyle {
                            id,
                            name,
                            path: Some(path),
                            color: hex(color),
                            opacity: (color.a as f64 * paint.opacity as f64).clamp(0.0, 1.0),
                        },
                    );
                    library.color_paints.insert(node.guid, node.fills.clone());
                }
                "TEXT" => {
                    let Some(text) = node.text.as_ref() else {
                        continue;
                    };
                    let font_size = text.font_size.unwrap_or(16.0);
                    let (weight, font_style) =
                        font_properties(text.font_style.as_deref().unwrap_or("Regular"));
                    let id = id_for_guid("typography-style", node.guid);
                    library.typographies.insert(
                        node.guid,
                        TypographyStyle {
                            id,
                            name,
                            path: Some(path),
                            font_id: String::new(),
                            font_family: text
                                .font_family
                                .clone()
                                .unwrap_or_else(|| "Inter".to_owned()),
                            font_variant_id: String::new(),
                            font_size: format_number(font_size as f64),
                            font_weight: weight.to_owned(),
                            font_style: font_style.to_owned(),
                            line_height: line_height_string(text, font_size),
                            letter_spacing: letter_spacing_string(text, font_size),
                            text_transform: "none".to_owned(),
                        },
                    );
                    library.text_styles.insert(node.guid, text.clone());
                }
                "EFFECT" => {
                    library
                        .effect_styles
                        .insert(node.guid, node.effects.clone());
                }
                "GRID" => {
                    library.grid_styles.insert(node.guid, node.name.clone());
                }
                _ => {}
            }
        }
        library
    }

    fn color(&self, reference: Option<&StyleRef>) -> Option<&ColorStyle> {
        reference
            .and_then(|reference| reference.guid)
            .and_then(|guid| self.colors.get(&guid))
    }

    fn color_paints(&self, reference: Option<&StyleRef>) -> Option<&[Paint]> {
        reference
            .and_then(|reference| reference.guid)
            .and_then(|guid| self.color_paints.get(&guid))
            .map(Vec::as_slice)
    }

    fn typography(&self, reference: Option<&StyleRef>) -> Option<&TypographyStyle> {
        reference
            .and_then(|reference| reference.guid)
            .and_then(|guid| self.typographies.get(&guid))
    }

    fn text_style(&self, reference: Option<&StyleRef>) -> Option<&TextData> {
        reference
            .and_then(|reference| reference.guid)
            .and_then(|guid| self.text_styles.get(&guid))
    }

    fn effects(&self, reference: Option<&StyleRef>) -> Option<&[Effect]> {
        reference
            .and_then(|reference| reference.guid)
            .and_then(|guid| self.effect_styles.get(&guid))
            .map(Vec::as_slice)
    }
}

fn map_constraint(value: Option<&str>, horizontal: bool) -> Option<String> {
    match value? {
        "MIN" => Some(if horizontal { "left" } else { "top" }.to_owned()),
        "MAX" => Some(if horizontal { "right" } else { "bottom" }.to_owned()),
        "CENTER" => Some("center".to_owned()),
        "STRETCH" => Some(if horizontal { "leftright" } else { "topbottom" }.to_owned()),
        "SCALE" => Some("scale".to_owned()),
        "FIXED_MIN" => Some(if horizontal { "left" } else { "top" }.to_owned()),
        "FIXED_MAX" => Some(if horizontal { "right" } else { "bottom" }.to_owned()),
        _ => None,
    }
}

fn first_paint_token<'a>(paints: &[Paint], tokens: &'a TokenLibrary) -> Option<&'a str> {
    paints
        .iter()
        .filter(|paint| paint.visible)
        .find_map(|paint| {
            paint
                .color_variable_key
                .as_deref()
                .and_then(|key| tokens.name_for(key))
        })
}

fn apply_paint_tokens(shape: &mut Shape, paints: &[Paint], tokens: &TokenLibrary) {
    if let Some(name) = first_paint_token(paints, tokens) {
        shape
            .applied_tokens
            .get_or_insert_with(std::collections::BTreeMap::new)
            .insert("fill".to_owned(), name.to_owned());
    }
}

fn inline_token_links(penpot: &mut PenpotDocument) {
    for page in &mut penpot.pages {
        for shape in &mut page.shapes {
            shape.applied_tokens = None;
            for fill in &mut shape.fills {
                fill.fill_color_ref_id = None;
                fill.fill_color_ref_file = None;
            }
            if let Some(Content::Text(content)) = shape.content.as_mut() {
                for set in &mut content.children {
                    for paragraph in &mut set.children {
                        for run in &mut paragraph.children {
                            run.typography_ref_id = None;
                            run.typography_ref_file = None;
                        }
                    }
                }
            }
        }
    }
}

fn detach_broken_component_refs(penpot: &mut PenpotDocument, sink: &mut dyn DiagnosticSink) {
    let all_ids: HashSet<Uuid> = penpot
        .pages
        .iter()
        .flat_map(|page| page.shapes.iter().map(|shape| shape.id))
        .collect();
    for page in &mut penpot.pages {
        for shape in &mut page.shapes {
            let Some(reference) = shape.shape_ref else {
                continue;
            };
            if all_ids.contains(&reference) {
                continue;
            }
            shape.shape_ref = None;
            shape.component_id = None;
            shape.component_file = None;
            shape.component_root = None;
            let element = ElementRef {
                figma_node_id: shape.id.to_string(),
                name: shape.name.clone(),
                element_type: shape.kind.as_str().to_owned(),
                page: Some(page.name.clone()),
                path: vec![page.name.clone(), shape.name.clone()],
            };
            emit(
                sink,
                &element,
                "F2P-INSTANCE-DETACHED",
                Severity::Warning,
                Outcome::ConvertedWithModification,
                "component link was removed because its main shape is not in the output".to_owned(),
                Some(
                    "the main component lives on an internal-only canvas or a skipped subtree"
                        .to_owned(),
                ),
                "kept the materialized shapes without the component link".to_owned(),
                Some("Recreate this instance from a local component in Penpot.".to_owned()),
            );
        }
    }
    penpot
        .components
        .retain(|component| all_ids.contains(&component.main_instance_id));
}

fn split_style_name(full: &str) -> (String, String) {
    let mut parts: Vec<&str> = full
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let name = parts.pop().unwrap_or(full).to_owned();
    (name, parts.join(" / "))
}

struct ComponentRegistry {
    by_symbol: HashMap<Guid, ComponentSpec>,
    entries: Vec<ComponentEntry>,
    prebuilding: bool,
    library_shapes: Vec<Shape>,
    max_expanded_shapes: usize,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self {
            by_symbol: HashMap::new(),
            entries: Vec::new(),
            prebuilding: false,
            library_shapes: Vec::new(),
            max_expanded_shapes: DEFAULT_MAX_EXPANDED_SHAPES,
        }
    }
}

struct ComponentSpec {
    component_id: Uuid,
    root_shape_id: Uuid,
    shapes: Vec<Shape>,
    guid_to_shape: HashMap<Guid, Uuid>,
    key_to_shape: HashMap<Guid, Uuid>,
    override_targets: HashMap<Vec<Guid>, Uuid>,
    variant_id: Option<Uuid>,
    variant_name: Option<String>,
}

impl ComponentRegistry {
    fn shapes_for(&self, symbol: Guid) -> Option<Vec<Shape>> {
        self.by_symbol.get(&symbol).map(|spec| spec.shapes.clone())
    }

    fn spec(&self, symbol: Guid) -> Option<&ComponentSpec> {
        self.by_symbol.get(&symbol)
    }
}

fn build_component_registry(
    document: &FigDocument,
    assets: &AssetRegistry,
    styles: &StyleLibrary,
    tokens: &TokenLibrary,
    tree: &Tree<'_>,
    max_expanded_shapes: usize,
    sink: &mut dyn DiagnosticSink,
) -> Result<ComponentRegistry, ConvertError> {
    let mut registry = ComponentRegistry {
        prebuilding: true,
        max_expanded_shapes,
        ..ComponentRegistry::default()
    };
    let file_id = styles.file_id;
    let library_page_id = id_for("component-library-page", &document.source_name);
    let (mut library_x, mut library_y, mut row_height) = (0.0_f64, 0.0_f64, 0.0_f64);
    let mut library_column = 0usize;
    let mut cached_shape_count = 0usize;
    for (index, source_page) in component_build_order(document, tree) {
        let node = tree.node(index);
        let page = source_page.and_then(|index| document.pages.get(index));
        let internal = page.is_none_or(|page| page.internal_only);
        let page_name = page
            .map(|page| page.name.as_str())
            .unwrap_or("Imported components");
        let page_id = if internal {
            library_page_id
        } else {
            id_for_guid("page", page.expect("non-internal page exists").guid)
        };
        let parent_index = node
            .parent
            .and_then(|parent| tree.by_guid.get(&parent).copied());
        let parent_is_canvas = parent_index
            .map(|index| tree.node(index).kind == "CANVAS")
            .unwrap_or(true);
        let parent_id = if parent_is_canvas || internal {
            Uuid::nil()
        } else {
            node.parent
                .map(|parent| id_for_guid("shape", parent))
                .unwrap_or(Uuid::nil())
        };
        let parent_abs = parent_index
            .map(|index| tree.absolute_transform(index))
            .unwrap_or(Affine::IDENTITY);
        let frame_id = if parent_is_canvas || internal {
            Uuid::nil()
        } else {
            parent_index
                .and_then(|index| frame_of(tree, index))
                .map(|guid| id_for_guid("shape", guid))
                .unwrap_or(Uuid::nil())
        };
        let mut path = node_path(tree, index, page_name);
        path.pop();
        let Some(mut shapes) = convert_node(
            document, assets, styles, tokens, &registry, tree, index, parent_id, frame_id,
            parent_abs, page_id, page_name, &path, sink,
        ) else {
            continue;
        };
        if shapes.is_empty() {
            continue;
        }
        cached_shape_count = cached_shape_count.saturating_add(shapes.len());
        if cached_shape_count > max_expanded_shapes {
            return Err(ConvertError::Invalid(format!(
                "component cache exceeds limit {max_expanded_shapes}"
            )));
        }
        let component_id = id_for_guid("component", node.guid);
        let parent = node
            .parent
            .and_then(|parent| tree.by_guid.get(&parent).copied())
            .map(|index| tree.node(index));
        let variant_parent = parent.filter(|parent| parent.is_state_group == Some(true));
        let (name, path_name) = if let Some(parent) = variant_parent {
            split_style_name(&parent.name)
        } else {
            split_style_name(&node.name)
        };
        let variant_properties = if variant_parent.is_some() {
            parse_variant_properties(&node.name)
        } else {
            Vec::new()
        };
        let variant_name = variant_parent.map(|_| node.name.clone());
        let variant_id = variant_parent.map(|parent| id_for_guid("variant", parent.guid));
        {
            let root = &mut shapes[0];
            root.component_id = Some(component_id);
            root.component_file = Some(file_id);
            root.component_root = Some(true);
            root.main_instance = Some(true);
            root.name = name.clone();
            root.variant_id = variant_id;
            root.variant_name = variant_name.clone();
        }
        let root_shape_id = shapes[0].id;
        if internal {
            let dx = library_x - shapes[0].x;
            let dy = library_y - shapes[0].y;
            for shape in &mut shapes {
                shift_shape(shape, dx, dy);
            }
            library_x += shapes[0].width.max(1.0) + 80.0;
            row_height = row_height.max(shapes[0].height);
            library_column += 1;
            if library_column == 8 {
                library_x = 0.0;
                library_y += row_height + 80.0;
                row_height = 0.0;
                library_column = 0;
            }
            if registry.library_shapes.len().saturating_add(shapes.len()) > max_expanded_shapes {
                return Err(ConvertError::Invalid(format!(
                    "embedded component shapes exceed limit {max_expanded_shapes}"
                )));
            }
            registry.library_shapes.extend(shapes.iter().cloned());
        }
        let mut guid_to_shape = HashMap::new();
        let mut key_to_shape = HashMap::new();
        let mut override_targets = HashMap::new();
        let available_ids: HashSet<_> = shapes.iter().map(|shape| shape.id).collect();
        for descendant in std::iter::once(index).chain(descendants(tree, index)) {
            let descendant_node = tree.node(descendant);
            let shape_id = id_for_guid("shape", descendant_node.guid);
            if available_ids.contains(&shape_id) {
                guid_to_shape.insert(descendant_node.guid, shape_id);
                override_targets.insert(vec![descendant_node.guid], shape_id);
            }
            if let Some(key) = descendant_node.override_key {
                if available_ids.contains(&shape_id) {
                    key_to_shape.insert(key, shape_id);
                    override_targets.insert(vec![key], shape_id);
                }
            }
            if descendant_node.kind == "INSTANCE" {
                if let Some(nested) = descendant_node
                    .symbol_id
                    .and_then(|guid| registry.spec(guid))
                {
                    let prefix = descendant_node.override_key.unwrap_or(descendant_node.guid);
                    for (nested_path, old_id) in &nested.override_targets {
                        let target = if *old_id == nested.root_shape_id {
                            shape_id
                        } else {
                            id_for(
                                "instance-shape",
                                &format!("{}:{}", descendant_node.guid, old_id),
                            )
                        };
                        if available_ids.contains(&target) && nested_path.len() < 128 {
                            let mut path = Vec::with_capacity(nested_path.len() + 1);
                            path.push(prefix);
                            path.extend(nested_path);
                            override_targets.insert(path, target);
                        }
                    }
                }
            }
        }
        registry.entries.push(ComponentEntry {
            id: component_id,
            name: name.clone(),
            path: path_name.clone(),
            main_instance_id: root_shape_id,
            main_instance_page: page_id,
            variant_id,
            variant_properties: if variant_properties.is_empty() {
                None
            } else {
                Some(variant_properties.clone())
            },
        });
        registry.by_symbol.insert(
            node.guid,
            ComponentSpec {
                component_id,
                root_shape_id,
                shapes,
                guid_to_shape,
                key_to_shape,
                override_targets,
                variant_id,
                variant_name,
            },
        );
    }
    registry.prebuilding = false;
    Ok(registry)
}

/// Restore only referenced embedded definitions, with dependencies before their users.
/// Cycles are broken deterministically; an unresolved cyclic instance emits a diagnostic.
fn component_build_order(document: &FigDocument, tree: &Tree<'_>) -> Vec<(usize, Option<usize>)> {
    let mut pending = VecDeque::new();
    for page in document.pages.iter().filter(|page| !page.internal_only) {
        let Some(index) = tree.by_guid.get(&page.guid).copied() else {
            continue;
        };
        for child in descendants(tree, index) {
            let node = tree.node(child);
            if node.kind == "SYMBOL" && !has_symbol_ancestor(tree, child) {
                pending.push_back(node.guid);
            }
            if let Some(symbol) = node.symbol_id {
                pending.push_back(symbol);
            }
        }
    }
    let mut required = BTreeMap::new();
    let mut dependencies = BTreeMap::new();
    while let Some(symbol) = pending.pop_front() {
        if required.contains_key(&symbol) {
            continue;
        }
        let Some(index) = tree.by_guid.get(&symbol).copied() else {
            continue;
        };
        if tree.node(index).kind != "SYMBOL" {
            continue;
        }
        let mut current = Some(index);
        let mut source_page = None;
        for _ in 0..=tree.nodes.len().min(512) {
            let Some(ancestor) = current else {
                break;
            };
            let node = tree.node(ancestor);
            if node.kind == "CANVAS" {
                source_page = document
                    .pages
                    .iter()
                    .position(|page| page.guid == node.guid);
                break;
            }
            current = node
                .parent
                .and_then(|parent| tree.by_guid.get(&parent).copied());
        }
        required.insert(symbol, (index, source_page));
        let deps: BTreeSet<Guid> = descendants(tree, index)
            .into_iter()
            .filter_map(|index| tree.node(index).symbol_id)
            .collect();
        pending.extend(deps.iter().copied());
        dependencies.insert(symbol, deps);
    }
    let mut counts = HashMap::new();
    let mut dependents: HashMap<Guid, Vec<Guid>> = HashMap::new();
    let mut ready = BTreeSet::new();
    for (symbol, deps) in &dependencies {
        let mut count = 0usize;
        for dependency in deps
            .iter()
            .filter(|dependency| required.contains_key(dependency))
        {
            count += 1;
            dependents.entry(*dependency).or_default().push(*symbol);
        }
        counts.insert(*symbol, count);
        if count == 0 {
            ready.insert(*symbol);
        }
    }
    let mut ordered = Vec::new();
    while !required.is_empty() {
        let next = ready
            .pop_first()
            .or_else(|| required.keys().next().copied());
        let Some(symbol) = next else {
            break;
        };
        let Some(source) = required.remove(&symbol) else {
            continue;
        };
        ordered.push(source);
        for dependent in dependents.get(&symbol).into_iter().flatten() {
            if let Some(count) = counts.get_mut(dependent) {
                *count = count.saturating_sub(1);
                if *count == 0 && required.contains_key(dependent) {
                    ready.insert(*dependent);
                }
            }
        }
    }
    ordered
}

fn has_symbol_ancestor(tree: &Tree<'_>, index: usize) -> bool {
    let mut current = tree.node(index).parent;
    let mut guard = 0;
    while let Some(parent) = current {
        guard += 1;
        if guard > 512 {
            return false;
        }
        let Some(index) = tree.by_guid.get(&parent).copied() else {
            return false;
        };
        if tree.node(index).kind == "SYMBOL" {
            return true;
        }
        current = tree.node(index).parent;
    }
    false
}

fn descendants(tree: &Tree<'_>, index: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut seen = HashSet::from([tree.node(index).guid]);
    let mut stack = tree.children_of(tree.node(index).guid);
    while let Some(index) = stack.pop() {
        if !seen.insert(tree.node(index).guid) {
            continue;
        }
        out.push(index);
        stack.extend(tree.children_of(tree.node(index).guid));
    }
    out
}

fn frame_of(tree: &Tree<'_>, index: usize) -> Option<Guid> {
    let mut current = Some(index);
    let mut guard = 0;
    while let Some(index) = current {
        guard += 1;
        if guard > 512 {
            return None;
        }
        let node = tree.node(index);
        if matches!(
            node.kind.as_str(),
            "FRAME" | "SYMBOL" | "SECTION" | "CANVAS"
        ) {
            return Some(node.guid);
        }
        current = node
            .parent
            .and_then(|parent| tree.by_guid.get(&parent).copied());
    }
    None
}

fn node_path(tree: &Tree<'_>, index: usize, page_name: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = Some(index);
    let mut guard = 0;
    while let Some(index) = current {
        guard += 1;
        if guard > 512 {
            break;
        }
        let node = tree.node(index);
        if node.kind == "CANVAS" {
            break;
        }
        segments.push(node.name.clone());
        current = node
            .parent
            .and_then(|parent| tree.by_guid.get(&parent).copied());
    }
    segments.push(page_name.to_owned());
    segments.reverse();
    segments
}

#[allow(clippy::too_many_arguments)]
fn group_masked_children(
    immediate: Vec<Shape>,
    mask_roots: &HashSet<Uuid>,
    parent_id: Uuid,
    frame_id: Uuid,
    page_id: Uuid,
    element: &ElementRef,
    sink: &mut dyn DiagnosticSink,
    descendants: &mut Vec<Shape>,
) -> Vec<Shape> {
    if mask_roots.is_empty() {
        return immediate;
    }
    let mut result = Vec::new();
    let mut index = 0usize;
    while index < immediate.len() {
        let shape = &immediate[index];
        if !mask_roots.contains(&shape.id) {
            result.push(immediate[index].clone());
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < immediate.len() && !mask_roots.contains(&immediate[index].id) {
            index += 1;
        }
        let end = index;
        let segment = &immediate[start..end];
        if end - start == 1 {
            emit(
                sink,
                element,
                "F2P-MASK-NO-TARGET",
                Severity::Warning,
                Outcome::ConvertedWithModification,
                "mask has no sibling shapes to mask".to_owned(),
                Some("Figma masks apply to following siblings".to_owned()),
                "kept the mask shape without a masked group".to_owned(),
                Some("Review this mask in Penpot.".to_owned()),
            );
            result.push(immediate[start].clone());
            continue;
        }
        let group_id = id_for("mask-group", &segment[0].id.to_string());
        let (x, y, w, h) = bounds(segment).unwrap_or((
            segment[0].x,
            segment[0].y,
            segment[0].width,
            segment[0].height,
        ));
        let child_ids: Vec<Uuid> = segment.iter().map(|shape| shape.id).collect();
        for shape in segment {
            let mut child = shape.clone();
            child.parent_id = group_id;
            descendants.push(child);
        }
        let mut group = Shape::new(
            group_id,
            "Mask group".to_owned(),
            ShapeType::Group,
            x,
            y,
            w,
            h,
            parent_id,
            frame_id,
            page_id,
        );
        group.shapes = Some(child_ids);
        group.masked_group = Some(true);
        group.proportion = Some(1.0);
        group.proportion_lock = Some(false);
        result.push(group);
        emit(
            sink,
            element,
            "F2P-MASK-APPROXIMATED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "Figma mask was converted to a Penpot masked group".to_owned(),
            Some("mask mode (alpha/luminance/outline) is not preserved".to_owned()),
            "grouped the mask with its following siblings".to_owned(),
            Some("Check the mask mode and shape in Penpot.".to_owned()),
        );
    }
    result
}

fn parse_variant_properties(name: &str) -> Vec<VariantProperty> {
    name.split(',')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            Some(VariantProperty {
                name: key.trim().to_owned(),
                value: value.trim().to_owned(),
            })
        })
        .collect()
}

struct AssetRegistry {
    fills: HashMap<String, ImageFill>,
    media: Vec<Media>,
    objects: Vec<MediaObject>,
}

impl AssetRegistry {
    fn new(document: &FigDocument) -> Self {
        let mut registry = Self::empty();
        for image in &document.images {
            if registry.fills.contains_key(&image.name) {
                continue;
            }
            let Some((mtype, width, height)) = image_info(&image.bytes) else {
                continue;
            };
            let media_id = id_for("media", &image.name);
            let object_id = id_for("object", &image.name);
            registry.fills.insert(
                image.name.clone(),
                ImageFill {
                    width,
                    height,
                    mtype: mtype.clone(),
                    id: media_id,
                },
            );
            registry.media.push(Media {
                id: media_id,
                name: image.name.clone(),
                width,
                height,
                mtype: mtype.clone(),
                media_id: object_id,
                is_local: true,
            });
            registry.objects.push(MediaObject {
                id: object_id,
                content_type: mtype,
                bytes: image.bytes.clone(),
            });
        }
        registry
    }

    fn empty() -> Self {
        Self {
            fills: HashMap::new(),
            media: Vec::new(),
            objects: Vec::new(),
        }
    }

    fn image(&self, hash: &str) -> Option<&ImageFill> {
        self.fills.get(hash)
    }
}

fn image_info(bytes: &[u8]) -> Option<(String, u32, u32)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        if bytes.len() >= 24 {
            let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
            let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
            return Some(("image/png".to_owned(), width, height));
        }
        return None;
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        if bytes.len() >= 10 {
            let width = u16::from_le_bytes(bytes[6..8].try_into().ok()?) as u32;
            let height = u16::from_le_bytes(bytes[8..10].try_into().ok()?) as u32;
            return Some(("image/gif".to_owned(), width, height));
        }
        return None;
    }
    if bytes.starts_with(&[0xFF, 0xD8]) {
        let mut offset = 2usize;
        while offset + 4 <= bytes.len() {
            if bytes[offset] != 0xFF {
                offset += 1;
                continue;
            }
            let marker = bytes[offset + 1];
            offset += 2;
            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            let len = u16::from_be_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
            if len < 2 || offset + len > bytes.len() {
                break;
            }
            if matches!(marker, 0xC0..=0xC3) && len >= 7 {
                let height =
                    u16::from_be_bytes(bytes[offset + 3..offset + 5].try_into().ok()?) as u32;
                let width =
                    u16::from_be_bytes(bytes[offset + 5..offset + 7].try_into().ok()?) as u32;
                return Some(("image/jpeg".to_owned(), width, height));
            }
            offset += len;
        }
    }
    None
}

struct Tree<'a> {
    nodes: &'a [FigNode],
    children: HashMap<Guid, Vec<usize>>,
    by_guid: HashMap<Guid, usize>,
}

impl<'a> Tree<'a> {
    fn new(nodes: &'a [FigNode]) -> Self {
        let mut children: HashMap<Guid, Vec<usize>> = HashMap::new();
        let mut by_guid = HashMap::new();
        for (index, node) in nodes.iter().enumerate() {
            if node.removed {
                continue;
            }
            by_guid.insert(node.guid, index);
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
        Self {
            nodes,
            children,
            by_guid,
        }
    }

    fn children_of(&self, parent: Guid) -> Vec<usize> {
        self.children.get(&parent).cloned().unwrap_or_default()
    }

    fn node(&self, index: usize) -> &'a FigNode {
        &self.nodes[index]
    }

    fn absolute_transform(&self, index: usize) -> Affine {
        let mut chain = Vec::new();
        let mut current = Some(index);
        let mut guard = 0usize;
        while let Some(index) = current {
            if guard > 512 {
                break;
            }
            guard += 1;
            let node = self.node(index);
            chain.push(node.transform.map(Affine::from).unwrap_or(Affine::IDENTITY));
            current = node
                .parent
                .and_then(|parent| self.by_guid.get(&parent).copied());
        }
        chain
            .into_iter()
            .rev()
            .fold(Affine::IDENTITY, |acc, transform| acc.multiply(transform))
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_instance(
    assets: &AssetRegistry,
    styles: &StyleLibrary,
    components: &ComponentRegistry,
    node: &FigNode,
    parent_id: Uuid,
    frame_id: Uuid,
    parent_abs: Affine,
    page_id: Uuid,
    page_name: &str,
    path: &[String],
    sink: &mut dyn DiagnosticSink,
) -> Option<Vec<Shape>> {
    let mut element_path = path.to_vec();
    element_path.push(node.name.clone());
    let element = element_ref(node, page_name, &element_path);
    sink.count_detected(&element);
    let id = id_for_guid("shape", node.guid);
    let absolute =
        parent_abs.multiply(node.transform.map(Affine::from).unwrap_or(Affine::IDENTITY));
    let spec = node
        .symbol_id
        .and_then(|symbol| components.spec(symbol))
        .or_else(|| node.override_key.and_then(|key| components.spec(key)));
    let Some(spec) = spec else {
        emit(
            sink,
            &element,
            "F2P-INSTANCE-DETACHED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "instance has no convertible main component".to_owned(),
            Some("the referenced symbol is missing or depends on a cyclic component".to_owned()),
            "converted the instance bounds without children".to_owned(),
            Some("Recreate this instance from a local component in Penpot.".to_owned()),
        );
        let size = node.size.unwrap_or(Size {
            width: 0.0,
            height: 0.0,
        });
        let mut shape = Shape::new(
            id,
            node.name.clone(),
            ShapeType::Frame,
            absolute.m02,
            absolute.m12,
            size.width as f64,
            size.height as f64,
            parent_id,
            frame_id,
            page_id,
        );
        shape.hidden = node.visible.map(|visible| !visible);
        shape.component_root = Some(false);
        shape.shapes = Some(Vec::new());
        return Some(vec![shape]);
    };

    let mut shapes = spec.shapes.clone();
    let root_old = spec.root_shape_id;
    let root_old_index = shapes.iter().position(|shape| shape.id == root_old)?;
    let resized_constraints = node.size.is_some_and(|size| {
        let resized_h = (size.width as f64 - shapes[root_old_index].width).abs() > 1e-6;
        let resized_v = (size.height as f64 - shapes[root_old_index].height).abs() > 1e-6;
        shapes
            .iter()
            .filter(|shape| shape.parent_id == root_old)
            .any(|shape| {
                (resized_h
                    && shape
                        .constraints_h
                        .as_deref()
                        .is_some_and(|mode| mode != "left"))
                    || (resized_v
                        && shape
                            .constraints_v
                            .as_deref()
                            .is_some_and(|mode| mode != "top"))
            })
    });
    let transformed = !absolute.is_translation_scale()
        || (absolute.m00 - 1.0).abs() > 1e-6
        || (absolute.m11 - 1.0).abs() > 1e-6;
    if resized_constraints || transformed {
        emit(sink, &element, "F2P-NODE-APPROXIMATED", Severity::Warning, Outcome::ConvertedWithModification,
            "instance geometry requires a manual review",
            Some("instance transforms and resized child constraints are not fully reapplied to cached component geometry".to_owned()),
            "preserved the component subtree and instance bounds using cached child geometry",
            Some("Compare this instance's child positions and sizing with Figma.".to_owned()));
    }
    let delta_x = absolute.m02 - shapes[root_old_index].x;
    let delta_y = absolute.m12 - shapes[root_old_index].y;
    let mut id_map = HashMap::new();
    for shape in &shapes {
        let new_id = if shape.id == root_old {
            id
        } else {
            id_for("instance-shape", &format!("{}:{}", node.guid, shape.id))
        };
        id_map.insert(shape.id, new_id);
    }
    let old_ids: Vec<Uuid> = shapes.iter().map(|shape| shape.id).collect();
    for shape in &mut shapes {
        let old_id = shape.id;
        shape.id = id_map[&old_id];
        shape.parent_id = if old_id == root_old {
            parent_id
        } else {
            id_map.get(&shape.parent_id).copied().unwrap_or(id)
        };
        shape.frame_id = if old_id == root_old {
            frame_id
        } else if shape.frame_id == root_old {
            id
        } else {
            id_map.get(&shape.frame_id).copied().unwrap_or(id)
        };
        shape.page_id = page_id;
        shift_shape(shape, delta_x, delta_y);
        shape.component_id = None;
        shape.component_file = None;
        shape.component_root = None;
        shape.main_instance = None;
        shape.shape_ref = Some(old_id);
        if let Some(children) = shape.shapes.as_mut() {
            for child in children {
                if let Some(mapped) = id_map.get(child) {
                    *child = *mapped;
                }
            }
        }
    }
    let root = &mut shapes[root_old_index];
    root.component_id = Some(spec.component_id);
    root.component_file = Some(styles.file_id);
    root.component_root = Some(false);
    root.name = node.name.clone();
    root.variant_id = spec.variant_id;
    root.variant_name = spec.variant_name.clone();
    if let Some(size) = node.size {
        let width = size.width as f64;
        let height = size.height as f64;
        root.width = width;
        root.height = height;
        root.selrect.width = width;
        root.selrect.height = height;
        root.selrect.x2 = root.x + width;
        root.selrect.y2 = root.y + height;
        root.points = vec![
            penpot_writer::Point {
                x: root.x,
                y: root.y,
            },
            penpot_writer::Point {
                x: root.x + width,
                y: root.y,
            },
            penpot_writer::Point {
                x: root.x + width,
                y: root.y + height,
            },
            penpot_writer::Point {
                x: root.x,
                y: root.y + height,
            },
        ];
    }
    if node.visible == Some(false) {
        root.hidden = Some(true);
    }
    if let Some(disabled) = node.frame_mask_disabled {
        root.show_content = Some(disabled);
    }
    if node.blend_mode.is_some() {
        root.blend_mode = map_blend_mode(node, &element, sink).0;
    }

    let index_by_old: HashMap<Uuid, usize> = old_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect();
    for over in &node.symbol_overrides {
        if !over.unmapped_fields.is_empty() {
            emit(sink, &element, "F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED",
                Severity::Warning, Outcome::ConvertedWithModification,
                "some instance attributes could not be preserved",
                Some(format!("unmapped override fields: {}", over.unmapped_fields.join(", "))),
                "applied supported attributes and retained the component defaults for other attributes",
                Some("Review the listed component settings in Penpot.".to_owned()));
        }
        let target = spec.override_targets.get(&over.guid_path).or_else(|| {
            if over.guid_path.len() == 1 {
                over.guid_path.first().and_then(|guid| {
                    spec.guid_to_shape
                        .get(guid)
                        .or_else(|| spec.key_to_shape.get(guid))
                })
            } else {
                None
            }
        });
        let Some(old_id) = target else {
            emit(
                sink,
                &element,
                "F2P-INSTANCE-OVERRIDE-LOST",
                Severity::Warning,
                Outcome::ConvertedWithModification,
                "an instance override could not be resolved".to_owned(),
                Some("its guid path did not match a shape in the main component".to_owned()),
                "kept the main component value".to_owned(),
                Some("Reapply this override in Penpot.".to_owned()),
            );
            continue;
        };
        let Some(index) = index_by_old.get(old_id).copied() else {
            continue;
        };
        let shape = &mut shapes[index];
        if let Some(characters) = &over.characters {
            if let Some(Content::Text(content)) = shape.content.as_mut() {
                if let Some(run) = content
                    .children
                    .first_mut()
                    .and_then(|set| set.children.first_mut())
                    .and_then(|paragraph| paragraph.children.first_mut())
                {
                    run.text = characters.clone();
                }
            }
        }
        if !over.fills.is_empty() {
            let (fills, _) = map_fills(
                sink,
                &element,
                &over.fills,
                shape.width,
                shape.height,
                assets,
                None,
                styles.file_id,
            );
            shape.fills = fills;
        }
        if let Some(size) = over.size {
            shape.width = size.width as f64;
            shape.height = size.height as f64;
            shape.selrect.width = shape.width;
            shape.selrect.height = shape.height;
            shape.selrect.x2 = shape.x + shape.width;
            shape.selrect.y2 = shape.y + shape.height;
        }
        if let Some(visible) = over.visible {
            shape.hidden = Some(!visible);
        }
        if let Some(disabled) = over.frame_mask_disabled {
            shape.show_content = Some(disabled);
        }
    }

    emit(
        sink,
        &element,
        "F2P-NODE-CONVERTED",
        Severity::Info,
        Outcome::Converted,
        "instance was materialized from its main component".to_owned(),
        None,
        "copied the component subtree and applied supported overrides".to_owned(),
        Some("Review overrides in Penpot.".to_owned()),
    );
    Some(shapes)
}

fn shift_shape(shape: &mut Shape, dx: f64, dy: f64) {
    shape.x += dx;
    shape.y += dy;
    shape.selrect.x += dx;
    shape.selrect.y += dy;
    shape.selrect.x1 += dx;
    shape.selrect.y1 += dy;
    shape.selrect.x2 += dx;
    shape.selrect.y2 += dy;
    for point in &mut shape.points {
        point.x += dx;
        point.y += dy;
    }
    if let Some(Content::Path(commands)) = &mut shape.content {
        for command in commands {
            for value in [
                &mut command.params.x,
                &mut command.params.c1x,
                &mut command.params.c2x,
            ]
            .into_iter()
            .flatten()
            {
                *value += dx;
            }
            for value in [
                &mut command.params.y,
                &mut command.params.c1y,
                &mut command.params.c2y,
            ]
            .into_iter()
            .flatten()
            {
                *value += dy;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_node(
    document: &FigDocument,
    assets: &AssetRegistry,
    styles: &StyleLibrary,
    tokens: &TokenLibrary,
    components: &ComponentRegistry,
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
    if node.kind == "SYMBOL" && !components.prebuilding {
        return components.shapes_for(node.guid);
    }
    if node.kind == "INSTANCE" {
        return convert_instance(
            assets, styles, components, node, parent_id, frame_id, parent_abs, page_id, page_name,
            path, sink,
        );
    }
    if node.style_type.is_some()
        || matches!(
            node.kind.as_str(),
            "VARIABLE" | "VARIABLE_SET" | "SLIDE_GRID" | "SLIDE_ROW"
        )
    {
        return None;
    }
    let id = id_for_guid("shape", node.guid);
    let mut element_path = path.to_vec();
    element_path.push(node.name.clone());
    let element = element_ref(node, page_name, &element_path);
    sink.count_detected(&element);

    if element_path.len() > MAX_CONVERSION_HIERARCHY_DEPTH {
        emit(
            sink,
            &element,
            "F2P-NODE-FAILED",
            Severity::Error,
            Outcome::Failed,
            "node hierarchy exceeds the conversion depth limit",
            Some("the hierarchy may be cyclic or excessively nested".to_owned()),
            "stopped traversing this branch",
            Some("Repair the parent hierarchy in Figma.".to_owned()),
        );
        return None;
    }
    let kind = match node.kind.as_str() {
        "FRAME" | "SYMBOL" | "SECTION" => {
            if node.resize_to_fit == Some(true) {
                NodeClass::Group
            } else {
                NodeClass::Frame
            }
        }
        "GROUP" => NodeClass::Group,
        "BOOLEAN_OPERATION" => NodeClass::Group,
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
    let recover_container_size = absolute.is_finite()
        && (!width.is_finite() || !height.is_finite())
        && matches!(kind, NodeClass::Frame | NodeClass::Group);
    if recover_container_size {
        modified = true;
        reasons.push("source container size is unknown (non-finite); used bounds measured from converted children".to_owned());
    }

    if node.kind == "SECTION" {
        modified = true;
        reasons.push("Figma section was preserved as a Penpot board with its children".to_owned());
    }

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
    if !absolute.is_finite()
        || ((!width.is_finite() || !height.is_finite()) && !recover_container_size)
    {
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

    if node.kind == "BOOLEAN_OPERATION" && node.fill_geometry.is_empty() {
        emit(
            sink,
            &element,
            "F2P-BOOLEAN-FLATTENED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            format!(
                "{} boolean operation was flattened to its child shapes",
                node.boolean_operation.as_deref().unwrap_or("unknown")
            ),
            Some("the file has no rendered boolean geometry".to_owned()),
            "kept the operands as children without the boolean".to_owned(),
            Some("Rebuild the boolean in Penpot.".to_owned()),
        );
    }

    if node.kind == "BOOLEAN_OPERATION" && !node.fill_geometry.is_empty() {
        let mut shapes = convert_vector_node(
            document, assets, styles, tokens, node, &element, absolute, width, height, parent_id,
            frame_id, page_id, sink,
        )?;
        emit(
            sink,
            &element,
            "F2P-BOOLEAN-FLATTENED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            format!(
                "{} boolean operation was flattened to its rendered path",
                node.boolean_operation.as_deref().unwrap_or("unknown")
            ),
            Some("Penpot boolean parameters are not reconstructed from Figma geometry".to_owned()),
            "kept exact rendered geometry as editable paths".to_owned(),
            Some("Rebuild the boolean in Penpot if you need to edit its operands.".to_owned()),
        );
        if let Some(root) = shapes.first_mut() {
            root.name = node.name.clone();
        }
        return Some(shapes);
    }

    if kind == NodeClass::Vector {
        return convert_vector_node(
            document, assets, styles, tokens, node, &element, absolute, width, height, parent_id,
            frame_id, page_id, sink,
        );
    }

    let mut descendants = Vec::new();
    let shape = match kind {
        NodeClass::Frame | NodeClass::Group => {
            let mut immediate = Vec::new();
            let mut mask_roots = HashSet::new();
            let child_frame = if kind == NodeClass::Frame {
                id
            } else {
                frame_id
            };
            for child in tree.children_of(node.guid) {
                if let Some(child_shapes) = convert_node(
                    document,
                    assets,
                    styles,
                    tokens,
                    components,
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
                    if descendants
                        .len()
                        .saturating_add(immediate.len())
                        .saturating_add(child_shapes.len())
                        > components.max_expanded_shapes
                    {
                        emit(
                            sink,
                            &element,
                            "F2P-NODE-FAILED",
                            Severity::Error,
                            Outcome::Failed,
                            "container exceeds the expanded shape limit",
                            Some(format!(
                                "limit is {} shapes",
                                components.max_expanded_shapes
                            )),
                            "stopped expanding this container",
                            Some("Split the design into smaller files.".to_owned()),
                        );
                        return None;
                    }
                    if tree.node(child).mask == Some(true) {
                        if let Some(root) = child_shapes.first() {
                            mask_roots.insert(root.id);
                        }
                    }
                    for shape in child_shapes {
                        if shape.parent_id == id {
                            immediate.push(shape);
                        } else {
                            descendants.push(shape);
                        }
                    }
                }
            }
            let immediate = group_masked_children(
                immediate,
                &mask_roots,
                id,
                child_frame,
                page_id,
                &element,
                sink,
                &mut descendants,
            );
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
            let (x, y, w, h) =
                if recover_container_size {
                    let Some(measured) = bounds(&immediate) else {
                        emit(sink, &element, "F2P-NODE-FAILED", Severity::Error, Outcome::Failed,
                        "container size is unknown and no finite children can determine its bounds",
                        Some("source size is non-finite".to_owned()), "skipped the container",
                        Some("Repair its dimensions in Figma and retry.".to_owned()));
                        return None;
                    };
                    measured
                } else if kind == NodeClass::Group {
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
                if node.kind == "SECTION" || recover_container_size {
                    shape.show_content = Some(true);
                } else {
                    shape.show_content = Some(node.frame_mask_disabled.unwrap_or(false));
                }
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

    if node.is_state_group == Some(true) {
        shape.is_variant_container = Some(true);
    }
    let (blend, blend_modified) = map_blend_mode(node, &element, sink);
    shape.blend_mode = blend;
    if blend_modified {
        modified = true;
        reasons.push("blend mode required a reported approximation".to_owned());
    }
    shape.constraints_h = map_constraint(node.constraints_h.as_deref(), true);
    shape.constraints_v = map_constraint(node.constraints_v.as_deref(), false);
    if matches!(
        node.constraints_h.as_deref(),
        Some("FIXED_MIN") | Some("FIXED_MAX")
    ) || matches!(
        node.constraints_v.as_deref(),
        Some("FIXED_MIN") | Some("FIXED_MAX")
    ) {
        emit(
            sink,
            &element,
            "F2P-CONSTRAINT-APPROXIMATED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "fixed-size constraint was approximated".to_owned(),
            Some("Figma FIXED_MIN/FIXED_MAX has no exact Penpot constraint".to_owned()),
            "mapped to the closest min/max constraint".to_owned(),
            Some("Check resize behavior in Penpot.".to_owned()),
        );
        modified = true;
        reasons.push("a fixed-size constraint was approximated".to_owned());
    }
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

    let fill_style = styles.color(node.style_id_for_fill.as_ref());
    let style_paints = styles.color_paints(node.style_id_for_fill.as_ref());
    let paints = if node.fills.is_empty() {
        style_paints.unwrap_or(&[])
    } else {
        &node.fills
    };
    let (fills, fills_modified) = map_fills(
        sink,
        &element,
        paints,
        shape.width,
        shape.height,
        assets,
        fill_style,
        styles.file_id,
    );
    shape.fills = fills;
    apply_paint_tokens(&mut shape, paints, tokens);
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
    let style_effects = styles.effects(node.style_id_for_effect.as_ref());
    if style_effects.is_some() {
        emit(
            sink,
            &element,
            "F2P-STYLE-EFFECT-INLINED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "shared effect style was inlined".to_owned(),
            Some("Penpot 2.18.5 has no effect-style library entry".to_owned()),
            "copied the style's effects onto this shape".to_owned(),
            Some("Edit the effects directly in Penpot.".to_owned()),
        );
        modified = true;
        reasons.push("a shared effect style was inlined".to_owned());
    }
    if node.style_id_for_grid.is_some() {
        emit(
            sink,
            &element,
            "F2P-STYLE-GRID-LOST",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            "shared grid style was not converted".to_owned(),
            Some("grid styles are not mapped in M3.6".to_owned()),
            "kept the shape without its grid".to_owned(),
            Some("Recreate layout grids in Penpot.".to_owned()),
        );
        modified = true;
        reasons.push("a shared grid style was skipped".to_owned());
    }
    let effects = style_effects.unwrap_or(&node.effects);
    let (shadow, blur, background_blur, effects_modified) =
        map_effects(node, effects, &element, sink);
    shape.shadow = shadow;
    shape.blur = blur;
    shape.background_blur = background_blur;
    if effects_modified {
        modified = true;
        reasons.push("one or more effects were skipped".to_owned());
    }
    if matches!(kind, NodeClass::Frame | NodeClass::Group) {
        apply_layout(
            node,
            &mut shape,
            &element,
            sink,
            &mut modified,
            &mut reasons,
        );
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
        let text_style = styles.typography(node.style_id_for_text.as_ref());
        let text_override = styles.text_style(node.style_id_for_text.as_ref());
        match map_text(
            node,
            &shape.fills,
            text_style,
            text_override,
            styles.file_id,
        ) {
            Ok((content, grow_type, text_modified, text_reason)) => {
                shape.content = Some(Content::Text(content));
                shape.grow_type = Some(grow_type);
                if node.text.as_ref().is_some_and(|text| text.mixed_styles) {
                    emit(
                        sink,
                        &element,
                        "F2P-TEXT-MIXED-STYLES-LOST",
                        Severity::Warning,
                        Outcome::ConvertedWithModification,
                        "mixed character styles were flattened".to_owned(),
                        Some("only the base text style is converted in M3.4".to_owned()),
                        "kept the text and its base style".to_owned(),
                        Some("Restyle individual characters in Penpot.".to_owned()),
                    );
                    modified = true;
                    reasons.push("mixed character styles were flattened".to_owned());
                }
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
    assets: &AssetRegistry,
    styles: &StyleLibrary,
    tokens: &TokenLibrary,
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
    let (blend_mode, blend_modified) = map_blend_mode(node, element, sink);
    // Figma can cache an empty fill stream alongside a valid expanded stroke.
    // A missing blob remains an error; only known empty streams are filtered.
    let nonempty = |geometry: &&fig_reader::VectorGeometry| {
        document
            .blob(geometry.commands_blob as usize)
            .is_none_or(|blob| !blob.is_empty())
    };
    let fill_geometry: Vec<_> = node.fill_geometry.iter().filter(nonempty).collect();
    let stroke_geometry: Vec<_> = node.stroke_geometry.iter().filter(nonempty).collect();
    let using_fill = !fill_geometry.is_empty();
    let ignored_empty = fill_geometry.len() != node.fill_geometry.len()
        || stroke_geometry.len() != node.stroke_geometry.len();
    let geometries = if using_fill {
        &fill_geometry
    } else {
        &stroke_geometry
    };
    if geometries.is_empty() {
        emit(
            sink,
            element,
            "F2P-VECTOR-EMPTY",
            Severity::Warning,
            Outcome::Unsupported,
            "vector node has no geometry blobs".to_owned(),
            Some("no non-empty fill or stroke geometry is available".to_owned()),
            "skipped the vector".to_owned(),
            Some("Recreate the vector manually in Penpot.".to_owned()),
        );
        return None;
    }

    let fill_style = styles.color(node.style_id_for_fill.as_ref());
    let style_paints = styles.color_paints(node.style_id_for_fill.as_ref());
    let paints = if node.fills.is_empty() {
        style_paints.unwrap_or(&[])
    } else {
        &node.fills
    };
    let (mut fills, fills_modified) = map_fills(
        sink,
        element,
        paints,
        width,
        height,
        assets,
        fill_style,
        styles.file_id,
    );
    let mut applied_tokens = std::collections::BTreeMap::new();
    if let Some(name) = first_paint_token(paints, tokens) {
        applied_tokens.insert("fill".to_owned(), name.to_owned());
    }
    if let Some(name) = first_paint_token(&node.strokes, tokens) {
        applied_tokens.insert("stroke-color".to_owned(), name.to_owned());
    }
    let (mut strokes, strokes_modified) = map_strokes(sink, element, node);
    let style_effects = styles.effects(node.style_id_for_effect.as_ref());
    let effects = style_effects.unwrap_or(&node.effects);
    let (shadow, blur, background_blur, effects_modified) =
        map_effects(node, effects, element, sink);
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
        shape.blend_mode = blend_mode.clone();
        shape.svg_attrs = Some(SvgAttrs {
            fill_rule: if geometry.winding_rule == "ODD" {
                "evenodd".to_owned()
            } else {
                "nonzero".to_owned()
            },
        });
        shape.fills = fills.clone();
        shape.strokes = strokes.clone();
        if !applied_tokens.is_empty() {
            shape.applied_tokens = Some(applied_tokens.clone());
        }
        shape.shadow = shadow.clone();
        shape.blur = blur.clone();
        shape.background_blur = background_blur.clone();
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

    let modified = using_stroke_outline
        || blend_modified
        || ignored_empty
        || fills_modified
        || strokes_modified
        || effects_modified
        || node.has_vector_network;
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
        if ignored_empty {
            Some("empty cached geometry was ignored; valid path geometry was preserved".to_owned())
        } else if modified {
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
        .rev()
        .filter(|paint| paint.visible)
        .filter_map(|paint| {
            if paint.kind != "SOLID" {
                return None;
            }
            let color = paint.color?;
            if !color.is_finite() {
                return None;
            }
            Some(Fill::solid(
                hex(color),
                (color.a as f64 * paint.opacity as f64).clamp(0.0, 1.0),
            ))
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

fn apply_layout(
    node: &FigNode,
    shape: &mut Shape,
    element: &ElementRef,
    sink: &mut dyn DiagnosticSink,
    modified: &mut bool,
    reasons: &mut Vec<String>,
) {
    let Some(mode) = node.stack_mode.as_deref() else {
        return;
    };
    if mode != "HORIZONTAL" && mode != "VERTICAL" {
        emit(
            sink,
            element,
            "F2P-LAYOUT-GRID-UNSUPPORTED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            format!("{mode} layout was not converted"),
            Some("grid layout is not mapped in M3.5".to_owned()),
            "kept the container without layout".to_owned(),
            Some("Recreate the grid layout in Penpot.".to_owned()),
        );
        *modified = true;
        reasons.push("a grid layout was skipped".to_owned());
        return;
    }
    let horizontal = mode == "HORIZONTAL";
    let padding_h = layout_number(
        node.stack_horizontal_padding,
        0.0,
        "horizontal padding",
        reasons,
    );
    let padding_v = layout_number(
        node.stack_vertical_padding,
        0.0,
        "vertical padding",
        reasons,
    );
    let p1 = padding_v;
    let p2 = layout_number(
        node.stack_padding_right,
        padding_h,
        "right padding",
        reasons,
    );
    let p3 = layout_number(
        node.stack_padding_bottom,
        padding_v,
        "bottom padding",
        reasons,
    );
    let p4 = padding_h;
    let primary = layout_number(node.stack_spacing, 0.0, "main-axis gap", reasons);
    let counter = layout_number(node.stack_counter_spacing, 0.0, "cross-axis gap", reasons);
    shape.layout = Some("flex".to_owned());
    shape.layout_flex_dir = Some(if horizontal { "row" } else { "column" }.to_owned());
    shape.layout_gap = Some(if horizontal {
        LayoutGap {
            row_gap: counter,
            column_gap: primary,
        }
    } else {
        LayoutGap {
            row_gap: primary,
            column_gap: counter,
        }
    });
    shape.layout_gap_type = Some("multiple".to_owned());
    shape.layout_padding = Some(LayoutPadding { p1, p2, p3, p4 });
    shape.layout_padding_type = Some(
        if (p1 - p3).abs() < 1e-6 && (p2 - p4).abs() < 1e-6 {
            "simple"
        } else {
            "multiple"
        }
        .to_owned(),
    );
    shape.layout_justify_content = Some(
        match node.stack_primary_align_items.as_deref() {
            Some("CENTER") => "center",
            Some("MAX") => "end",
            Some("SPACE_BETWEEN") | Some("SPACE_EVENLY") => "space-between",
            _ => "start",
        }
        .to_owned(),
    );
    shape.layout_justify_items = Some("stretch".to_owned());
    shape.layout_align_items = Some(
        match node.stack_counter_align_items.as_deref() {
            Some("MIN") => "start",
            Some("CENTER") => "center",
            Some("MAX") => "end",
            _ => "stretch",
        }
        .to_owned(),
    );
    shape.layout_align_content = Some(
        match node.stack_counter_align_content.as_deref() {
            Some("SPACE_BETWEEN") => "space-between",
            Some("MIN") => "start",
            Some("CENTER") => "center",
            Some("MAX") => "end",
            _ => "stretch",
        }
        .to_owned(),
    );
    if horizontal {
        shape.layout_wrap_type = Some(
            if node.stack_wrap.as_deref() == Some("WRAP") {
                "wrap"
            } else {
                "nowrap"
            }
            .to_owned(),
        );
    }
    *modified = true;
    reasons.push(
        "Auto Layout container mapped to flex; child sizing is not fully preserved".to_owned(),
    );
    emit(
        sink,
        element,
        "F2P-LAYOUT-PARTIAL",
        Severity::Warning,
        Outcome::ConvertedWithModification,
        "Auto Layout mapped to a Penpot flex container".to_owned(),
        Some("child grow/fill sizing is not yet mapped".to_owned()),
        "preserved direction, gap, padding, and primary/counter alignment".to_owned(),
        Some("Review child sizing after import.".to_owned()),
    );
}

#[allow(clippy::too_many_arguments)]
fn map_fills(
    sink: &mut dyn DiagnosticSink,
    element: &ElementRef,
    paints: &[Paint],
    width: f64,
    height: f64,
    assets: &AssetRegistry,
    style: Option<&ColorStyle>,
    file_id: Uuid,
) -> (Vec<Fill>, bool) {
    let mut fills = Vec::new();
    let mut modified = false;
    let mut warned_blend = false;
    for paint in paints {
        if !paint.visible {
            continue;
        }
        if !warned_blend
            && paint
                .blend_mode
                .as_deref()
                .is_some_and(|mode| mode != "NORMAL")
        {
            warned_blend = true;
            modified = true;
            emit(
                sink,
                element,
                "F2P-BLEND-MODE-LOST",
                Severity::Warning,
                Outcome::ConvertedWithModification,
                "per-paint blend mode was not preserved".to_owned(),
                Some("Penpot fills do not carry individual blend modes".to_owned()),
                "kept the paint color".to_owned(),
                Some("Rebuild the paint blend in Penpot.".to_owned()),
            );
        }
        if let Some((color, opacity)) = solid_color(paint) {
            let mut fill = Fill::solid(color, opacity);
            if let Some(style) = style {
                fill.fill_color_ref_id = Some(style.id);
                fill.fill_color_ref_file = Some(file_id);
            }
            fills.push(fill);
            continue;
        }
        if let Some(gradient) = gradient_fill(paint, width, height) {
            fills.push(Fill::gradient(gradient, paint.opacity as f64));
            continue;
        }
        if let Some(hash) = paint.image_hash.as_deref() {
            if let Some(image) = assets.image(hash) {
                fills.push(Fill::image(image.clone(), paint.opacity as f64));
                continue;
            }
        }
        modified = true;
        emit(
            sink,
            element,
            "F2P-PAINT-UNSUPPORTED",
            Severity::Warning,
            Outcome::ConvertedWithModification,
            format!("{} fill was skipped", paint.kind),
            Some("only solid and linear/radial gradient paints are supported".to_owned()),
            "kept the remaining paints".to_owned(),
            Some("Recreate this fill manually in Penpot.".to_owned()),
        );
    }
    fills.reverse();
    (fills, modified)
}

fn gradient_fill(paint: &Paint, width: f64, height: f64) -> Option<GradientFill> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let kind = match paint.kind.as_str() {
        "GRADIENT_LINEAR" => "linear",
        "GRADIENT_RADIAL" => "radial",
        _ => return None,
    };
    let gradient = paint.gradient.as_ref()?;
    if gradient.stops.is_empty() {
        return None;
    }
    let transform = gradient
        .transform
        .map(Affine::from)
        .unwrap_or(Affine::IDENTITY);
    let det = transform.m00 * transform.m11 - transform.m10 * transform.m01;
    if det.abs() < 1e-12 {
        return None;
    }
    let ia = transform.m11 / det;
    let ic = -transform.m01 / det;
    let ie = (transform.m01 * transform.m12 - transform.m11 * transform.m02) / det;
    let ib = -transform.m10 / det;
    let id = transform.m00 / det;
    let if_ = (transform.m10 * transform.m02 - transform.m00 * transform.m12) / det;
    let point = |gx: f64, gy: f64| {
        let x = (ia * gx + ic * gy + ie) * width;
        let y = (ib * gx + id * gy + if_) * height;
        (x / width, y / height)
    };
    let (start_x, start_y, end_x, end_y) = if kind == "linear" {
        let (sx, sy) = point(0.0, 0.5);
        let (ex, ey) = point(1.0, 0.5);
        (sx, sy, ex, ey)
    } else {
        let (cx, cy) = point(0.5, 0.5);
        let (px, py) = point(1.0, 0.5);
        let radius_x = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
        let angle = (py - cy).atan2(px - cx);
        let ex = cx + angle.cos() * radius_x;
        let ey = cy + angle.sin() * radius_x;
        (cx, cy, ex, ey)
    };
    let mut stops: Vec<_> = gradient
        .stops
        .iter()
        .map(|stop| PenpotGradientStop {
            color: hex(stop.color),
            opacity: Some(stop.color.a as f64),
            offset: stop.position as f64,
        })
        .collect();
    stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    Some(GradientFill {
        kind: kind.to_owned(),
        start_x,
        start_y,
        end_x,
        end_y,
        width: 1.0,
        stops,
    })
}

fn map_effects(
    node: &FigNode,
    effects: &[Effect],
    element: &ElementRef,
    sink: &mut dyn DiagnosticSink,
) -> (
    Option<Vec<Shadow>>,
    Option<Blur>,
    Option<BackgroundBlur>,
    bool,
) {
    let mut shadows = Vec::new();
    let mut blur = None;
    let mut background_blur = None;
    let mut modified = false;
    for (order, effect) in effects.iter().enumerate() {
        match effect.kind.as_str() {
            "DROP_SHADOW" | "INNER_SHADOW" => {
                let color = effect.color.unwrap_or(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.25,
                });
                shadows.push(Shadow {
                    id: id_for("shadow", &format!("{}:{}", node.guid, order)),
                    style: if effect.kind == "INNER_SHADOW" {
                        "inner-shadow".to_owned()
                    } else {
                        "drop-shadow".to_owned()
                    },
                    offset_x: effect.offset_x as f64,
                    offset_y: effect.offset_y as f64,
                    blur: effect.radius as f64,
                    spread: effect.spread as f64,
                    hidden: !effect.visible,
                    color: ShadowColor {
                        color: hex(color),
                        opacity: color.a as f64,
                    },
                });
            }
            "LAYER_BLUR" | "FOREGROUND_BLUR" => {
                if blur.is_none() {
                    blur = Some(Blur {
                        id: id_for("blur", &node.guid.to_string()),
                        kind: "layer-blur".to_owned(),
                        value: effect.radius as f64,
                        hidden: !effect.visible,
                    });
                } else {
                    modified = true;
                    emit(
                        sink,
                        element,
                        "F2P-EFFECT-DUPLICATE",
                        Severity::Warning,
                        Outcome::ConvertedWithModification,
                        "shape has more than one layer blur".to_owned(),
                        Some("Penpot keeps a single layer blur per shape".to_owned()),
                        "kept the first layer blur".to_owned(),
                        Some("Review the blur in Penpot.".to_owned()),
                    );
                }
            }
            "BACKGROUND_BLUR" => {
                if background_blur.is_none() {
                    background_blur = Some(BackgroundBlur {
                        id: id_for("background-blur", &node.guid.to_string()),
                        kind: "background-blur".to_owned(),
                        value: effect.radius as f64,
                        hidden: !effect.visible,
                    });
                } else {
                    modified = true;
                    emit(
                        sink,
                        element,
                        "F2P-EFFECT-DUPLICATE",
                        Severity::Warning,
                        Outcome::ConvertedWithModification,
                        "shape has more than one background blur".to_owned(),
                        Some("Penpot keeps a single background blur per shape".to_owned()),
                        "kept the first background blur".to_owned(),
                        Some("Review the blur in Penpot.".to_owned()),
                    );
                }
            }
            other => {
                modified = true;
                emit(
                    sink,
                    element,
                    "F2P-EFFECT-UNSUPPORTED",
                    Severity::Warning,
                    Outcome::ConvertedWithModification,
                    format!("{other} effect was skipped"),
                    Some("no Penpot equivalent in M3.2".to_owned()),
                    "kept the remaining effects".to_owned(),
                    Some("Recreate this effect manually in Penpot.".to_owned()),
                );
            }
        }
    }
    // Penpot paints shadow[0] on top; Figma's effects array runs bottom to top.
    shadows.reverse();
    (
        if shadows.is_empty() {
            None
        } else {
            Some(shadows)
        },
        blur,
        background_blur,
        modified,
    )
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
    strokes.reverse();
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
    typography: Option<&TypographyStyle>,
    override_text: Option<&TextData>,
    file_id: Uuid,
) -> Result<(TextContent, String, bool, Option<String>), String> {
    let text = node
        .text
        .as_ref()
        .or(override_text)
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
    let text_align = match text.align_horizontal.as_deref() {
        Some("CENTER") => "center",
        Some("RIGHT") => "right",
        Some("JUSTIFIED") => "justify",
        _ => "left",
    };
    let line_height = line_height_string(text, font_size);
    let letter_spacing = letter_spacing_string(text, font_size);
    let vertical_align = match text.align_vertical.as_deref() {
        Some("CENTER") => Some("center".to_owned()),
        Some("BOTTOM") => Some("bottom".to_owned()),
        Some("TOP") | None => Some("top".to_owned()),
        Some(_) => None,
    };
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
        vec![Fill::solid("#000000", 1.0)]
    } else {
        run_fills
    };
    let mut content = TextContent::single_run(TextRun {
        text: characters.to_owned(),
        font_family,
        font_size: format_number(font_size as f64),
        font_weight: weight.to_owned(),
        font_style: font_style.to_owned(),
        text_align: Some(text_align.to_owned()),
        line_height: Some(line_height),
        letter_spacing: Some(letter_spacing),
        typography_ref_id: typography.map(|style| style.id),
        typography_ref_file: typography.map(|_| file_id),
        fills: run_fills,
    });
    content.vertical_align = vertical_align;
    Ok((content, grow_type.to_owned(), modified, reason))
}

fn line_height_string(text: &TextData, font_size: f32) -> String {
    let value = text.line_height_value.unwrap_or(1.2 * font_size) as f64;
    let size = font_size as f64;
    let ratio = match text.line_height_units.as_deref() {
        Some("PERCENT") => value / 100.0,
        Some("RAW") => value,
        Some("PIXELS") if size > 0.0 => value / size,
        _ if size > 0.0 => value / size,
        _ => 1.2,
    };
    format_number(ratio)
}

fn letter_spacing_string(text: &TextData, font_size: f32) -> String {
    let value = text.letter_spacing_value.unwrap_or(0.0) as f64;
    let size = font_size as f64;
    let px = match text.letter_spacing_units.as_deref() {
        Some("PERCENT") => (value / 100.0) * size,
        _ => value,
    };
    format_number(px)
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
            frame_mask_disabled: None,
            text: None,
            effects: Vec::new(),
            fill_geometry: Vec::new(),
            stroke_geometry: Vec::new(),
            has_vector_network: false,
            mask: None,
            boolean_operation: None,
            blend_mode: None,
            constraints_h: None,
            constraints_v: None,
            style_type: None,
            style_id_for_fill: None,
            style_id_for_stroke: None,
            style_id_for_text: None,
            style_id_for_effect: None,
            style_id_for_grid: None,
            symbol_id: None,
            symbol_overrides: Vec::new(),
            override_key: None,
            is_state_group: None,
            state_group_properties: Vec::new(),
            variant_properties: Vec::new(),
            variable_modes: Vec::new(),
            variable_set_id: None,
            variable_resolved_type: None,
            variable_values: Vec::new(),
            variable_key: None,
            description: None,
            stack_mode: None,
            stack_spacing: None,
            stack_counter_spacing: None,
            stack_horizontal_padding: None,
            stack_vertical_padding: None,
            stack_padding_right: None,
            stack_padding_bottom: None,
            stack_primary_align_items: None,
            stack_counter_align_items: None,
            stack_counter_align_content: None,
            stack_wrap: None,
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
            gradient: None,
            image_hash: None,
            color_variable_key: None,
            blend_mode: None,
        }
    }

    #[test]
    fn translating_instance_paths_moves_endpoints_and_control_points() {
        let mut shape = Shape::new(
            Uuid::from_u128(1),
            "Path",
            ShapeType::Path,
            10.0,
            20.0,
            30.0,
            40.0,
            Uuid::nil(),
            Uuid::nil(),
            Uuid::from_u128(2),
        );
        shape.content = Some(Content::Path(vec![PathCommand {
            command: "curve-to".to_owned(),
            params: PathParams {
                x: Some(40.0),
                y: Some(60.0),
                c1x: Some(15.0),
                c1y: Some(25.0),
                c2x: Some(30.0),
                c2y: Some(50.0),
            },
        }]));
        shift_shape(&mut shape, 100.0, -10.0);
        let Some(Content::Path(commands)) = &shape.content else {
            panic!("path content");
        };
        assert_eq!(
            (commands[0].params.x, commands[0].params.y),
            (Some(140.0), Some(50.0))
        );
        assert_eq!(
            (commands[0].params.c1x, commands[0].params.c1y),
            (Some(115.0), Some(15.0))
        );
        assert_eq!(
            (commands[0].params.c2x, commands[0].params.c2y),
            (Some(130.0), Some(40.0))
        );
        assert_eq!((shape.x, shape.y), (110.0, 10.0));
    }

    #[test]
    fn gradient_paint_is_reported_and_skipped() {
        let mut sink = ReportCollector::new();
        let paints = [Paint {
            kind: "GRADIENT_LINEAR".to_owned(),
            color: None,
            opacity: 1.0,
            visible: true,
            gradient: None,
            image_hash: None,
            color_variable_key: None,
            blend_mode: None,
        }];
        let (fills, modified) = map_fills(
            &mut sink,
            &element(),
            &paints,
            10.0,
            10.0,
            &AssetRegistry::empty(),
            None,
            Uuid::nil(),
        );
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
    fn text_metrics_are_mapped() {
        let mut node = blank_node();
        node.kind = "TEXT".to_owned();
        node.text = Some(TextData {
            characters: Some("Metrics".to_owned()),
            font_family: Some("Inter".to_owned()),
            font_size: Some(20.0),
            line_height_value: Some(150.0),
            line_height_units: Some("PERCENT".to_owned()),
            letter_spacing_value: Some(10.0),
            letter_spacing_units: Some("PERCENT".to_owned()),
            align_horizontal: Some("CENTER".to_owned()),
            align_vertical: Some("CENTER".to_owned()),
            ..TextData::default()
        });
        let (content, _grow, _modified, _reason) =
            map_text(&node, &[], None, None, Uuid::nil()).unwrap();
        assert_eq!(content.vertical_align.as_deref(), Some("center"));
        let run = &content.children[0].children[0].children[0];
        assert_eq!(run.text_align.as_deref(), Some("center"));
        assert_eq!(run.line_height.as_deref(), Some("1.5"));
        assert_eq!(run.letter_spacing.as_deref(), Some("2"));
    }

    #[test]
    fn empty_text_is_skipped() {
        let mut node = blank_node();
        node.kind = "TEXT".to_owned();
        node.text = Some(TextData {
            characters: Some(String::new()),
            ..TextData::default()
        });
        assert!(map_text(&node, &[], None, None, Uuid::nil()).is_err());
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
