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

/// An image entry found in the ZIP container. Bytes are not loaded in M1.
#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub name: String,
    pub size: u64,
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
