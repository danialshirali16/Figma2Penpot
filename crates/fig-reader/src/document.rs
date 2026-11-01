use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;

/// The container a `.fig` file arrived in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    /// Standard ZIP archive with `canvas.fig`, metadata, thumbnail, and images.
    Zip,
    /// Raw `fig-kiwi` binary without a ZIP wrapper.
    Raw,
}

impl fmt::Display for ContainerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => f.write_str("ZIP"),
            Self::Raw => f.write_str("raw fig-kiwi"),
        }
    }
}

/// A Figma node GUID, formatted as `sessionID:localID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Guid {
    pub session_id: u32,
    pub local_id: u32,
}

impl Guid {
    pub const ZERO: Self = Self {
        session_id: 0,
        local_id: 0,
    };

    pub const fn new(session_id: u32, local_id: u32) -> Self {
        Self {
            session_id,
            local_id,
        }
    }

    pub const fn is_zero(self) -> bool {
        self.session_id == 0 && self.local_id == 0
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.session_id, self.local_id)
    }
}

/// A 2x3 affine transform, matching Figma's `m00..m12` layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
}

impl Matrix {
    pub const IDENTITY: Self = Self {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
    };

    /// True when every component is finite. Non-finite values are preserved for
    /// diagnostics rather than silently replaced.
    pub fn is_finite(self) -> bool {
        [self.m00, self.m01, self.m02, self.m10, self.m11, self.m12]
            .iter()
            .all(|v| v.is_finite())
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Node size in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn is_finite(self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }
}

/// RGBA color in Figma's 0-1 float space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn is_finite(self) -> bool {
        [self.r, self.g, self.b, self.a]
            .iter()
            .all(|value| value.is_finite())
    }
}

/// A gradient stop in Figma's 0-1 color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,
}

/// A gradient paint's stops and geometry transform.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: String,
    pub stops: Vec<GradientStop>,
    pub transform: Option<Matrix>,
}

/// A fill or stroke paint. M2 maps solid paints; M3.2 maps linear/radial gradients.
#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    pub kind: String,
    pub color: Option<Color>,
    pub opacity: f32,
    pub visible: bool,
    pub gradient: Option<Gradient>,
    pub image_hash: Option<String>,
    /// Asset key of the Figma variable bound to this paint's color.
    pub color_variable_key: Option<String>,
    pub blend_mode: Option<String>,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            kind: "UNKNOWN".to_owned(),
            color: None,
            opacity: 1.0,
            visible: true,
            gradient: None,
            image_hash: None,
            color_variable_key: None,
            blend_mode: None,
        }
    }
}

/// A shadow or blur effect from Figma's `effects` array.
#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub kind: String,
    pub color: Option<Color>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub radius: f32,
    pub spread: f32,
    pub visible: bool,
}

/// A reference to a shared style: either a local GUID or a library asset key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleRef {
    pub guid: Option<Guid>,
    pub asset_key: Option<String>,
}

/// One mode declared by a Figma variable collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableMode {
    pub id: Guid,
    pub name: String,
}

/// One mode-specific value of a Figma variable.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableValue {
    pub mode_id: Guid,
    pub kind: String,
    pub color: Option<Color>,
    pub alias_key: Option<String>,
    pub number: Option<f32>,
    pub text: Option<String>,
    pub boolean: Option<bool>,
}

/// One instance override addressed by a path of GUIDs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SymbolOverride {
    pub guid_path: Vec<Guid>,
    pub characters: Option<String>,
    pub fills: Vec<Paint>,
    pub size: Option<Size>,
    pub visible: Option<bool>,
}

/// Basic text attributes extracted for M2.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextData {
    pub characters: Option<String>,
    pub font_family: Option<String>,
    pub font_style: Option<String>,
    pub font_size: Option<f32>,
    pub align_horizontal: Option<String>,
    pub align_vertical: Option<String>,
    pub auto_resize: Option<String>,
    pub line_height_value: Option<f32>,
    pub line_height_units: Option<String>,
    pub letter_spacing_value: Option<f32>,
    pub letter_spacing_units: Option<String>,
    pub mixed_styles: bool,
}

/// A path geometry entry referencing a blob index in the decoded message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorGeometry {
    pub commands_blob: u32,
    pub winding_rule: String,
}

/// A node decoded from Figma's flat `nodeChanges` array.
#[derive(Debug, Clone)]
pub struct FigNode {
    pub guid: Guid,
    pub parent: Option<Guid>,
    /// Fractional sibling index. Compare lexicographically, never numerically.
    pub position: Option<String>,
    /// Raw node type name, or `UNKNOWN(<value>)` when the enum value is unknown.
    pub kind: String,
    /// Raw enum value behind `kind`, when it was an enum.
    pub kind_raw: Option<u32>,
    pub name: String,
    pub phase: Option<String>,
    pub visible: Option<bool>,
    pub size: Option<Size>,
    pub transform: Option<Matrix>,
    pub internal_only: bool,
    pub removed: bool,
    pub opacity: Option<f32>,
    pub fills: Vec<Paint>,
    pub strokes: Vec<Paint>,
    pub stroke_weight: Option<f32>,
    pub stroke_align: Option<String>,
    pub corner_radius: Option<f32>,
    pub corner_radii: Option<[f32; 4]>,
    pub resize_to_fit: Option<bool>,
    pub text: Option<TextData>,
    pub effects: Vec<Effect>,
    pub fill_geometry: Vec<VectorGeometry>,
    pub stroke_geometry: Vec<VectorGeometry>,
    pub has_vector_network: bool,
    pub mask: Option<bool>,
    pub boolean_operation: Option<String>,
    pub blend_mode: Option<String>,
    pub constraints_h: Option<String>,
    pub constraints_v: Option<String>,
    /// Style master type when this node is a Figma style definition.
    pub style_type: Option<String>,
    pub style_id_for_fill: Option<StyleRef>,
    pub style_id_for_stroke: Option<StyleRef>,
    pub style_id_for_text: Option<StyleRef>,
    pub style_id_for_effect: Option<StyleRef>,
    pub style_id_for_grid: Option<StyleRef>,
    /// Instance link: the SYMBOL this INSTANCE points at.
    pub symbol_id: Option<Guid>,
    pub symbol_overrides: Vec<SymbolOverride>,
    /// Stable key bridging imported library copies to their originals.
    pub override_key: Option<Guid>,
    /// Component set (Figma "state group") container flag.
    pub is_state_group: Option<bool>,
    pub state_group_properties: Vec<String>,
    pub variant_properties: Vec<(String, String)>,
    /// Variable collection metadata for VARIABLE_SET nodes.
    pub variable_modes: Vec<VariableMode>,
    /// Variable fields for VARIABLE nodes.
    pub variable_set_id: Option<StyleRef>,
    pub variable_resolved_type: Option<String>,
    pub variable_values: Vec<VariableValue>,
    pub variable_key: Option<String>,
    pub description: Option<String>,
    pub stack_mode: Option<String>,
    pub stack_spacing: Option<f32>,
    pub stack_counter_spacing: Option<f32>,
    pub stack_horizontal_padding: Option<f32>,
    pub stack_vertical_padding: Option<f32>,
    pub stack_padding_right: Option<f32>,
    pub stack_padding_bottom: Option<f32>,
    pub stack_primary_align_items: Option<String>,
    pub stack_counter_align_items: Option<String>,
    pub stack_counter_align_content: Option<String>,
    pub stack_wrap: Option<String>,
}

impl FigNode {
    pub fn is_canvas(&self) -> bool {
        self.kind == "CANVAS" && !self.removed
    }

    pub fn is_known_kind(&self) -> bool {
        is_known_node_type(&self.kind)
    }
}

/// A blob's location inside the decoded message buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blob {
    pub offset: usize,
    pub len: usize,
}

/// A page (Figma `CANVAS`) with counts derived from a real traversal.
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub guid: Guid,
    pub name: String,
    pub internal_only: bool,
    /// Position among canvases after lexicographic ordering.
    pub index: usize,
    pub direct_children: usize,
    pub descendant_nodes: usize,
}

/// An image entry loaded from the ZIP container.
#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub name: String,
    pub size: u64,
    pub bytes: Vec<u8>,
}

/// Metadata from `meta.json`, when present and readable.
#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub file_name: Option<String>,
    pub version: Option<String>,
}

/// A non-fatal finding recorded while reading.
#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub code: &'static str,
    pub message: String,
}

impl ParseWarning {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Counts derived from decoded nodes, not estimated from output.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub node_count: usize,
    pub active_nodes: usize,
    pub removed_nodes: usize,
    pub blob_count: usize,
    pub blob_bytes: usize,
    pub page_count: usize,
    pub internal_page_count: usize,
    pub image_count: usize,
    pub image_bytes: u64,
    pub node_types: BTreeMap<String, usize>,
    pub unknown_node_types: BTreeSet<String>,
    pub orphan_nodes: usize,
    pub max_depth: usize,
}

/// A decoded `.fig` document and its verified metadata.
pub struct FigDocument {
    pub source_name: String,
    pub container: ContainerKind,
    pub has_thumbnail: bool,
    pub prelude: String,
    pub version: u32,
    pub schema_defs: usize,
    pub chunk_count: usize,
    pub nodes: Vec<FigNode>,
    pub blobs: Vec<Blob>,
    pub pages: Vec<PageInfo>,
    pub images: Vec<ImageEntry>,
    pub meta: Option<Meta>,
    pub warnings: Vec<ParseWarning>,
    pub stats: Stats,
    message: Arc<[u8]>,
}

impl fmt::Debug for FigDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FigDocument")
            .field("source_name", &self.source_name)
            .field("container", &self.container)
            .field("prelude", &self.prelude)
            .field("version", &self.version)
            .field("schema_defs", &self.schema_defs)
            .field("nodes", &self.stats.node_count)
            .field("blobs", &self.blobs.len())
            .field("pages", &self.pages.len())
            .field("images", &self.images.len())
            .finish_non_exhaustive()
    }
}

pub(crate) struct DocumentParts {
    pub source_name: String,
    pub container: ContainerKind,
    pub has_thumbnail: bool,
    pub prelude: String,
    pub version: u32,
    pub schema_defs: usize,
    pub chunk_count: usize,
    pub nodes: Vec<FigNode>,
    pub blobs: Vec<Blob>,
    pub images: Vec<ImageEntry>,
    pub meta: Option<Meta>,
    pub warnings: Vec<ParseWarning>,
    pub message: Vec<u8>,
}

impl FigDocument {
    pub(crate) fn new(parts: DocumentParts) -> Self {
        let DocumentParts {
            source_name,
            container,
            has_thumbnail,
            prelude,
            version,
            schema_defs,
            chunk_count,
            nodes,
            blobs,
            images,
            meta,
            mut warnings,
            message,
        } = parts;
        let (pages, stats) = build_pages_and_stats(&nodes, &blobs, &images, &mut warnings);
        Self {
            source_name,
            container,
            has_thumbnail,
            prelude,
            version,
            schema_defs,
            chunk_count,
            nodes,
            blobs,
            pages,
            images,
            meta,
            warnings,
            stats,
            message: Arc::from(message),
        }
    }

    /// Bytes of a blob referenced by index, or `None` when the index is invalid.
    pub fn blob(&self, index: usize) -> Option<&[u8]> {
        let blob = self.blobs.get(index)?;
        self.message
            .get(blob.offset..blob.offset.checked_add(blob.len)?)
    }

    /// The decompressed Kiwi message. Kept for later conversion passes.
    pub fn message(&self) -> &[u8] {
        &self.message
    }

    /// The human-readable file name from `meta.json`, when present.
    pub fn meta_file_name(&self) -> Option<&str> {
        self.meta.as_ref()?.file_name.as_deref()
    }
}

fn build_pages_and_stats(
    nodes: &[FigNode],
    blobs: &[Blob],
    images: &[ImageEntry],
    warnings: &mut Vec<ParseWarning>,
) -> (Vec<PageInfo>, Stats) {
    let mut stats = Stats {
        node_count: nodes.len(),
        blob_count: blobs.len(),
        blob_bytes: blobs.iter().map(|blob| blob.len).sum(),
        image_count: images.len(),
        image_bytes: images.iter().map(|image| image.size).sum(),
        ..Stats::default()
    };

    let mut by_guid: HashMap<Guid, usize> = HashMap::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        by_guid.insert(node.guid, index);
        if !node.removed {
            stats.active_nodes += 1;
            *stats.node_types.entry(node.kind.clone()).or_default() += 1;
            if !node.is_known_kind() {
                stats.unknown_node_types.insert(node.kind.clone());
            }
        } else {
            stats.removed_nodes += 1;
        }
    }

    let mut children: HashMap<Guid, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if node.removed {
            continue;
        }
        match node.parent {
            Some(parent) if parent != node.guid && by_guid.contains_key(&parent) => {
                children.entry(parent).or_default().push(index);
            }
            Some(_) => {
                stats.orphan_nodes += 1;
                roots.push(index);
            }
            None => roots.push(index),
        }
    }

    // Sibling order is the fractional index string, never array order.
    for indices in children.values_mut() {
        indices.sort_by(|a, b| {
            let left = nodes[*a].position.as_deref().unwrap_or("");
            let right = nodes[*b].position.as_deref().unwrap_or("");
            left.cmp(right)
                .then_with(|| nodes[*a].guid.cmp(&nodes[*b].guid))
        });
    }

    // Depth from the roots; any unvisited active node indicates a cycle.
    let mut depth = vec![0usize; nodes.len()];
    let mut visited = vec![false; nodes.len()];
    let mut queue: VecDeque<usize> = roots.into_iter().collect();
    while let Some(index) = queue.pop_front() {
        if visited[index] {
            continue;
        }
        visited[index] = true;
        stats.max_depth = stats.max_depth.max(depth[index]);
        if let Some(kids) = children.get(&nodes[index].guid) {
            for kid in kids {
                if !visited[*kid] {
                    depth[*kid] = depth[index].saturating_add(1);
                    queue.push_back(*kid);
                }
            }
        }
    }
    let unvisited = nodes
        .iter()
        .enumerate()
        .filter(|(index, node)| !node.removed && !visited[*index])
        .count();
    if unvisited > 0 {
        warnings.push(ParseWarning::new(
            "FIG-WARN-CYCLE",
            format!("{unvisited} node(s) are not reachable from a root (possible parent cycle)"),
        ));
    }

    let mut canvases: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.is_canvas())
        .map(|(index, _)| index)
        .collect();
    canvases.sort_by(|a, b| {
        let left = nodes[*a].position.as_deref().unwrap_or("");
        let right = nodes[*b].position.as_deref().unwrap_or("");
        left.cmp(right)
            .then_with(|| nodes[*a].guid.cmp(&nodes[*b].guid))
    });

    let mut epoch = vec![0u32; nodes.len()];
    let mut current_epoch: u32 = 0;
    let mut pages = Vec::with_capacity(canvases.len());
    for (index, canvas_index) in canvases.iter().enumerate() {
        let canvas = &nodes[*canvas_index];
        if canvas.internal_only {
            stats.internal_page_count += 1;
        }
        let direct_children = children.get(&canvas.guid).map_or(0, Vec::len);

        current_epoch = current_epoch.wrapping_add(1);
        if current_epoch == 0 {
            epoch.fill(0);
            current_epoch = 1;
        }
        let mut descendants = 0usize;
        let mut stack: Vec<usize> = children.get(&canvas.guid).cloned().unwrap_or_default();
        while let Some(node_index) = stack.pop() {
            if epoch[node_index] == current_epoch {
                continue;
            }
            epoch[node_index] = current_epoch;
            descendants += 1;
            if let Some(kids) = children.get(&nodes[node_index].guid) {
                stack.extend(kids.iter().copied());
            }
        }

        pages.push(PageInfo {
            guid: canvas.guid,
            name: canvas.name.clone(),
            internal_only: canvas.internal_only,
            index,
            direct_children,
            descendant_nodes: descendants,
        });
    }

    stats.page_count = pages.len();
    (pages, stats)
}

/// Node types documented in `docs/FIG_FORMAT.md`. Unknown values are reported,
/// never treated as an error.
pub fn is_known_node_type(kind: &str) -> bool {
    matches!(
        kind,
        "DOCUMENT"
            | "CANVAS"
            | "SECTION"
            | "FRAME"
            | "GROUP"
            | "RECTANGLE"
            | "ROUNDED_RECTANGLE"
            | "ELLIPSE"
            | "POLYGON"
            | "STAR"
            | "LINE"
            | "VECTOR"
            | "BOOLEAN_GROUP"
            | "SHAPE_WITH_TEXT"
            | "TABLE"
            | "TEXT"
            | "SYMBOL"
            | "INSTANCE"
            | "COMPONENT_SET"
            | "VARIABLE_SET"
            | "VARIABLE"
            | "SLIDE_GRID"
            | "SLIDE_ROW"
            | "SLIDE"
            | "MODULE"
    )
}
