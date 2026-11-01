# Figma `.fig` Format — What We Know

Status: M0 research baseline; updated at M2. Container detection, bounded Kiwi decoding,
core geometry, solid paints/strokes, and basic text attributes are implemented in
`crates/fig-reader`; M2 converts a solid/basic subset. Vectors, components, variables, and
full text fidelity are not implemented yet.

Confidence labels:

- **Confirmed** — reproduced by byte inspection, source reading, or a runnable check.
- **Reported** — claimed by an upstream project or document, not independently reproduced.
- **Hypothesis** — plausible, untested.
- **Unknown** — not established. Do not build on it without a test.

Primary sources: `openfig-core` at commit `0d67354`, the reference converter at `c29242f`,
and our own byte-level checks. See `docs/RESEARCH_SOURCES.md`.

---

## 1. Container

### 1.1 Two flavours

`.fig` files appear in two container forms:

| Form | Detection | Notes |
|---|---|---|
| ZIP archive | first bytes `50 4B` (`PK`) | Modern "Save local copy" files |
| Raw `fig-kiwi` binary | first bytes `66 69 67 2D` (`fig-`) | Older/alternate exports; no embedded images |

The reference converter dispatches on the ZIP magic
(`penpot-converter/packages/fig/src/readFig.ts:11-18`). **Confirmed** by inspection of the
fixtures we opened. A reader must not trust the `.fig` extension.

### 1.2 ZIP contents

Observed in `openfig-core/test-fixtures`:

```text
canvas.fig            required, the document payload
meta.json             required by Figma, file name/version metadata
thumbnail.png         required by Figma, preview image
images/<sha1-hex>     zero or more bitmaps, no extension
```

Example (`circle.fig`, 32,231 bytes):

| Entry | Uncompressed | Method |
|---|---:|---|
| `canvas.fig` | 25,795 | store (0) |
| `thumbnail.png` | 5,395 | store (0) |
| `meta.json` | 351 | deflate (8) |
| `images/` | 0 | directory |

`coloradjusted.fig` additionally contains two images named by 40-character lowercase hex
SHA-1, with no extension. This matches `openfig-core/docs/images.md`.

The ZIP is a normal archive, so the usual archive hazards apply: zip-slip names, entry
count, total uncompressed size, compression ratio, and nested archives. We never extract
to disk by entry name; assets are streamed into a controlled store keyed by content hash.

## 2. `canvas.fig` binary layout

**Confirmed** by byte inspection and by `openfig-core/src/parser.ts:28-80`.

```text
offset  size  content
0       8     prelude, ASCII, space-padded: "fig-kiwi", "fig-deck", "fig-jam."
8       4     uint32 little-endian format version
12      ...   length-prefixed chunks, until end of file

chunk:
  4 bytes  uint32 little-endian length N
  N bytes  chunk data
```

- Preludes seen: `fig-kiwi` for `.fig`, `fig-deck` for Slides, `fig-jam.` for FigJam.
  This project reads `fig-kiwi` only; other preludes are reported as unsupported.
- There is **no checksum and no integrity field** (`openfig-core/docs/archive.md`).
- Typically exactly two chunks; additional chunks are opaque and must be preserved if
  round-tripping. We only read.

### 2.1 Chunk 0 — Kiwi schema

- Compression: **raw deflate** (RFC 1951, no zlib/gzip wrapper).
- Content: the binary Kiwi schema, which is itself a Kiwi-encoded structure.
- Decompressed size observed: 62,366 bytes for a 25,238-byte compressed schema
  (`circle.fig`, version 101), and 550 definitions.
- The schema is **embedded in every file**. It is the authoritative description of the
  data chunk; the format is self-describing.

Decoding order: deflate → `Schema::decode` → compile/lookup definitions → decode message.

### 2.2 Chunk 1 — message

- Compression: **zstd**, detected by magic `28 B5 2F FD`. Older files may use raw deflate;
  the reference parser falls back when the magic is absent (`parser.ts:72-80`).
- Content: a Kiwi `Message` value containing the whole document.
- Decompressed sizes observed: 1,106 bytes (`circle.fig`), 12,801 bytes (`OpenFigs.fig`).
  Real design files are much larger; openfig's notes cite ~29 MB decompressed for a
  typical cloud export. Treat that as **Reported**, not a limit.

### 2.3 Version field

Observed values: **101** and **106**. The version is stamped at creation and preserved
across edits; it is not a guarantee about which fields exist. The embedded schema is the
real contract.

openfig's notes additionally report that Figma accepted a file written with version 105
and rejected 109 (`docs/archive.md`). That is about producing `.fig` files, which we do
not do. It is recorded for context, not relied on.

## 3. Kiwi wire format

Kiwi is a schema-based binary format by Evan Wallace, similar to Protocol Buffers but with
detectable optional fields. The wire format is documented by Evan Wallace's Kiwi project and the Rust crate of the same
name. M1 implements a bounded decoder in-house (ADR-0006) rather than depending on a parser
that allocates from untrusted counts. `flate2`/`miniz_oxide` and `ruzstd` handle
decompression.

- Native types: `bool`, `byte`, `int` (varint zigzag), `uint`, `float`, `string`
  (UTF-8, NUL-terminated), `int64`, `uint64`, and arrays.
- User types: `enum` (uint with names), `struct` (fixed required fields, in order),
  `message` (optional fields, field IDs).
- Unknown fields in a message can be skipped because the schema supplies their type.
- Messages are single-scan, so decoding is linear in input size.

For `.fig` we always decode with the file's own embedded schema. That removes most
"schema drift" problems at the wire level; drift appears instead as **new definitions or
fields our typed layer does not know**, which must produce diagnostics, not crashes.

## 4. Message root

**Confirmed** by decoding two fixtures with the Rust crate:

```text
Message (definition index 404 of 550)
  type        : enum MessageType
  ackID       : uint
  nodeChanges : NodeChange[]      <- the entire document, flat
  blobs       : Blob[]            <- ByteArray payloads
  sessionID   : uint
```

`Message` has 44 fields in the tested schema. Additional fields must be preserved as
unknown, not rejected.

### 4.1 `nodeChanges` is flat, not a tree

Every node, from `DOCUMENT` down to a text run, is an entry in one array. The tree is
encoded by `parentIndex`:

```text
{
  guid:        { sessionID: uint, localID: uint },
  type:        "FRAME" | "RECTANGLE" | "TEXT" | ...,
  name:        string,
  phase:       "CREATED" | "REMOVED" | undefined,
  parentIndex: { guid: {...}, position: string },
  ...
}
```

Rules that matter for conversion:

- **Do not filter the array when writing `.fig`.** We only read, but the same mental model
  applies: a removed node is marked `phase: "REMOVED"`, and an editor must not delete the
  entry. Our reader must ignore `REMOVED` for output while still resolving references.
- `parentIndex.position` is a **string**, compared lexicographically, not numerically. It
  is a fractional index designed to allow insertion between siblings. Treating it as a
  single character caps a parent at ~94 children and then reorders or breaks
  (`openfig-core/docs/nodes.md`; `penpot-converter/.../readFig.ts:31-42`).
- `guid` is `"sessionID:localID"` for reporting. Keep the numeric pair internally; string
  formatting is lossy only if you choose the wrong separator or trim zeros.

### 4.2 Phases and visibility

| Field | Meaning |
|---|---|
| `phase: undefined` | Existing, unmodified node |
| `phase: "CREATED"` | Newly created node |
| `phase: "REMOVED"` | Deleted; must remain in the array |
| `visible: false` | Hidden layer; still present |
| `internalOnly: true` on a `CANVAS` | Figma's hidden component page |

The hidden "Internal Only Canvas" holds main components and scratch content. Figma does not
show it as a page. It must be excluded from page output but still indexed for component
resolution. The reference converter does exactly this
(`penpot-converter/packages/fig/src/toDocument.ts:141-145`).

## 5. Node types

Known types from `openfig-core/docs/nodes.md`, with the caveat that the list is empirical:

| Family | Types |
|---|---|
| Structure | `DOCUMENT`, `CANVAS`, `SECTION`, `FRAME`, `GROUP` |
| Basic shapes | `RECTANGLE`, `ROUNDED_RECTANGLE`, `ELLIPSE`, `POLYGON`, `STAR`, `LINE` |
| Vector | `VECTOR`, `BOOLEAN_GROUP`, `SHAPE_WITH_TEXT`, `TABLE` |
| Text | `TEXT` |
| Components | `SYMBOL`, `INSTANCE`, `COMPONENT_SET` |
| Variables | `VARIABLE_SET`, `VARIABLE` |
| Slides | `SLIDE_GRID`, `SLIDE_ROW`, `SLIDE`, `MODULE` |

### 5.1 Groups may be frames

A visible Figma "Group" is not guaranteed to decode as `GROUP`. In several fixtures it is
a `FRAME` with:

- `resizeToFit: true`
- empty or absent fills
- no `stackMode`

This is documented in `openfig-core/docs/containers.md` and confirmed there against
`frame and group.fig`. We must classify it above the raw parser; otherwise a group becomes
a Penpot board, which changes clipping, layout, and semantics.

**Unknown:** whether modern Figma still emits raw `GROUP` in other cases, and whether a
discriminator beyond `resizeToFit` + fills + stack fields exists.

### 5.2 Sections and slides

`SECTION` is a dedicated type and must not be inferred. Slides use a different hierarchy
(`SLIDE_GRID` → `SLIDE_ROW` → `SLIDE`). v1 of this project targets `.fig` design files,
not `.deck` or `.jam`; those preludes are rejected with a clear diagnostic.

## 6. Geometry and transforms

Every visual node carries:

```text
size:      { x: width, y: height }
transform: { m00, m01, m02, m10, m11, m12 }   // 2x3 affine
```

- `m02` is translation X, `m12` is translation Y. **Confirmed** in
  `openfig-core/docs/shapes.md` and in the reference converter's `matrix.ts`.
- Transforms are **parent-relative**. Computing a Penpot position requires accumulating the
  matrix down the tree (`penpot-converter/packages/fig/src/matrix.ts:64-76`).
- Rotation and flip are encoded in the matrix, not as separate fields. Penpot can represent
  rotation and flip, but its primary fields are axis-aligned `x/y/width/height` plus a
  transform; do not lose the matrix.
- Figma omits fields equal to defaults, and a value bound to a deleted variable can surface
  as `NaN`. Per-field fallback to identity is required; `transform ?? identity` is not
  enough. The reference converter documents this trap
  (`penpot-converter/packages/fig/src/matrix.ts:23-45`).

### 6.1 Absolute vs. parent-relative

`.fig` is parent-relative. Penpot stores **absolute canvas coordinates**. Evidence:

- Penpot's frame renderer draws children inside a `<g>` with no parent translation
  (`penpot/2.18.5 frontend/src/app/main/ui/shapes/frame.cljs`, container section).
- The reference converter accumulates to absolute before writing
  (`toDocument.ts:614-615`) and states the rule in its README (lines 134-138).
- The official builder library stores `x/y` verbatim; passing relative coordinates yields
  a file whose shapes sit at the wrong place. The writer must do the accumulation.

This is recorded as ADR-0005. It is **Confirmed from source + generated output**, but a
real Penpot import of a nested fixture is still the final proof (M2).

## 7. Vector geometry

VECTOR nodes do not carry path commands inline. They reference blobs by index:

```text
fillGeometry[]   { windingRule, commandsBlob, styleID }
strokeGeometry[] { windingRule, commandsBlob, styleID }
vectorData       { vectorNetworkBlob, normalizedSize, styleOverrideTable }
blobs[]          { bytes }
```

### 7.1 `commandsBlob`

A command stream of:

| Byte | Command | Parameters |
|---|---|---|
| `0x01` | moveTo | x, y (f32 LE) |
| `0x02` | lineTo | x, y (f32 LE) |
| `0x04` | cubicTo | c1x, c1y, c2x, c2y, x, y (6 × f32 LE) |
| `0x00` | closePath | none |

Coordinates are in **node size space**, not page space. This is the low-risk path for
rendering and for building a Penpot `path`; the editable vector network is a separate
payload (`openfig-core/docs/vector.md`).

### 7.2 `vectorNetworkBlob`

Verified layout, byte-exact on the openfig fixture corpus (43 blobs, 26k vertices):

```text
header:   [vertexCount u32][segmentCount u32][regionCount u32]   (12 bytes)
vertex:   [styleID u32][x f32][y f32]                            (12 bytes)
segment:  [styleID u32][startVertex u32][tsx f32][tsy f32]
          [endVertex u32][tex f32][tey f32]                      (28 bytes)
region:   [styleID<<1 | windingRule u32][numLoops u32]
          per loop: [segCount u32][segIndex u32 × segCount]
```

Caveats from the fixture README:

- A segment is a straight line iff all four tangent components are zero; there is no
  segment type field.
- The vertex leading word is a `styleID` into `styleOverrideTable`, not a handle-mirroring
  enum. It must be preserved.
- The segment leading word is presumed to be a style ID but is zero in every observed file;
  treat it as unidentified and write `0` if ever writing.
- `commandsBlob` and the vector network may differ by one closing command; do not assume
  equal command counts.

For v1 we can convert `commandsBlob` to Penpot path content and keep the vector network
only for diagnostics. Editability in Penpot is preserved because Penpot stores path content
as editable path data; the original network's handle metadata is lost, which must be
reported if it mattered.

**M3.1 implementation:** `commandsBlob` is decoded and transformed into Penpot command
arrays. Filled vectors are imported and rendered (OpenFigs icon, 20 paths). Stroke-only
vectors use the expanded stroke outline as a filled path and are reported as approximated.
`vectorData` networks are not read; current builds emit `F2P-VECTOR-NETWORK-LOST` when the
field is present. Booleans and editable handle-level networks remain Planned.

## 8. Paints, effects, text

### 8.1 Paints

`fillPaints[]` and `strokePaints[]` entries:

| Field | Type | Notes |
|---|---|---|
| `type` | enum | `SOLID`, `GRADIENT_LINEAR`, `GRADIENT_RADIAL`, `GRADIENT_ANGULAR`, `GRADIENT_DIAMOND`, `IMAGE` |
| `color` | `{r,g,b,a}` | 0-1 floats |
| `opacity` | number | 0-1 |
| `visible` | bool | Inactive paints stay in the array |
| `blendMode` | enum | 19 values reported |
| `stops` | `{color, position}[]` | Gradients |
| `transform` | 2x3 matrix | Gradient geometry |
| `image` | `{hash, name}` | SHA-1, 20 bytes + hex name |

Gradient geometry requires a Penpot gradient transform; a naive copying of stops produces
the wrong angle/scale. Treat as a fidelity item, not a structural one.

### 8.2 Effects

`effects[]` with types `DROP_SHADOW`, `INNER_SHADOW`, `FOREGROUND_BLUR`,
`BACKGROUND_BLUR`, `REPEAT`, `SYMMETRY`, `GRAIN`, `NOISE`, `GLASS`. Fields: `offset`,
`radius`, `spread`, `color`, `visible`, `blendMode`, `showShadowBehindNode`, `opacity`.
Penpot has shadow and blur models; the later procedural effects have no direct equivalent
and must be reported as not converted.

### 8.3 Text

A TEXT node carries:

- `textData.characters` — the string.
- `fontName { family, style, postscript }`, `fontSize`.
- `lineHeight { value, units }`, `letterSpacing { value, units }`.
- `textAutoResize`, `textAlignHorizontal`, `textAlignVertical`.
- `styleIdForText` — a named text style reference, or the sentinel
  `0xFFFFFFFF:0xFFFFFFFF` when detached.
- `fillPaints` for the base fill.
- `derivedTextData` — cached glyph layout. Figma recomputes it; do not treat it as content.
- Mixed styling via `characterStyleIDs` + `styleOverrideTable`, one entry per character
  run (Reported; the reference converter handles it, we have not decoded a mixed-style
  fixture yet).

Font availability is a conversion hazard: a font missing from the machine/Penpot must be
recorded, not silently substituted. The reference converter maintains lists of Google,
custom, and local fonts; we will start with name preservation plus a diagnostic, and only
add substitution as an explicit, reported action.

## 9. Components, instances, variants

The public Figma model is familiar; the binary encoding is not identical:

- `SYMBOL` is a component definition (the public API calls it `COMPONENT`).
- `INSTANCE` references a symbol by `componentId` (local) or `componentKey` (library).
  Orphan instances exist when the symbol does not resolve.
- `COMPONENT_SET` groups variants; variant values live in component property definitions.
- Overrides live in `symbolData.symbolOverrides`, each with a `guidPath` that references
  **`overrideKey` values, not node `guid`s** (`openfig-core/docs/overrides.md`).
- Overrides are deltas: text characters, fills, visibility, nested instance swaps, and
  more. Resolution is recursive and can nest; the reference converter caps instance depth
  at 12 to avoid cycles (`toDocument.ts:81,836`).
- An instance's overridden `symbolData` describes deviations; it is not a full copy of the
  component. To materialize it in Penpot you must walk the main component and apply
  overrides in order, outermost instance wins.

**Unknown:** the complete override field set across Figma versions, and the exact semantics
of variant property matching. Both need fixture-driven tests before claiming fidelity.

## 10. Variables and design tokens

Figma stores variables as nodes:

- `VARIABLE_SET` — a collection with modes.
- `VARIABLE` — a typed value (color, number, string, boolean), possibly an alias to another
  variable by GUID or published `assetRef`.
- Values can be per-mode.
- Paints may carry `colorVar` directly; additionally there is a `variableConsumptionMap`
  for other bindings.

The reference converter's findings, **Reported** and worth testing:

- Most color bindings are on paints as `colorVar`, not only in the consumption map.
- Penpot tokens are file-wide sets/themes; Figma modes do not map one-to-one. Their
  `--token-faithful` mode picks one mode per board and one theme per file.
- Boolean variables are dropped deliberately.
- Consumption fields without a Penpot counterpart include `VISIBLE`, `TEXT_DATA`,
  `VARIANT_PROPERTIES`, `PARAGRAPH_SPACING`, `PARAGRAPH_INDENT`.
- Token names must match Penpot's path grammar; sanitize every segment **and** every
  reference identically or references dangle.

We will implement tokens later than structure and appearance, and only claim **Partial**
until a real Penpot import proves bindings survive.

## 11. Images

- Stored under `images/<40-char lowercase hex SHA-1>` with no extension.
- Every image normally has a companion thumbnail at ~320 px wide, also SHA-1 named.
- Paint references use `{ hash: 20 bytes, name: hex string }`.
- Multiple paints may reference the same image; deduplicate by hash.
- Missing image bytes are possible in malformed files. Report and skip the paint; never
  emit a dangling reference.

Penpot stores media as `objects/<uuid>.json` + `objects/<uuid>.<ext>` plus a
`files/<file-id>/media/<media-id>.json` reference. The mapping is content-hash → new UUID,
with a manifest of dimensions and content type.

## 12. Versioning and drift

The schema is self-describing, which is the saving grace of this format:

- A file can add node types and fields without breaking an old reader **if** the reader
  skips unknown fields and does not assume a closed enum.
- A file can also change the meaning of an existing field. That cannot be detected from
  the schema; only regression fixtures can catch it.
- The version integer is not a compatibility contract. Do not gate on `version == 101`.

Our policy:

1. Decode using the embedded schema.
2. Map known fields into typed IR.
3. For unknown node types: keep a placeholder node with its name and bounds if possible,
   emit `FIG-UNKNOWN-NODE`, and continue.
4. For unknown fields on known nodes: ignore but count once per field name per file, so a
   report can say what was not understood.
5. Never fail the whole file because one node is unfamiliar.

## 13. Security limits

Kiwi's decoder is linear, but counts inside the schema and message can still cause large
allocations. M1 implements the following defaults in
`crates/fig-reader/src/limits.rs`; they are configurable per reader call:

| Limit | Implemented default | Reason |
|---|---:|---|
| Input file size | 1 GiB | Refuse absurd files before reading |
| ZIP entries | 10,000 | `.fig` has few entries; images are the bulk |
| `canvas.fig` decompressed | 512 MiB | Hard ceiling for the payload |
| Schema decompressed | 16 MiB | Observed 62 KiB; 16 MiB is generous |
| Message decompressed | 512 MiB | Large design files need room; cap memory |
| Chunks | 16 | Observed 2 |
| Nodes | 1,000,000 | Bound traversal and report memory |
| Blobs | 1,000,000 | Bound index allocation |
| Single blob | 256 MiB | Images/vector payloads |
| Generic array items | 4,000,000 | Bound non-byte arrays before allocation |
| Nesting depth | 64 | Bound recursion |
| ZIP compression ratio | 1,000:1 | Detect obvious bombs; absolute caps still apply |
| Total ZIP uncompressed | 4 GiB | Bound declared extraction size |
| zstd window | 128 MiB | Bound decoder allocation independently of output |
| Schema definitions | 10,000 | Observed 550/627 |
| Schema fields | 200,000 | Bound schema metadata |

These are enforced by the in-house decoder, not by a third-party parser. Tests cover
truncation, corrupt chunk lengths, zip-slip names, entry-count limits, compression-ratio
limits, and node limits. Fuzzing is the ongoing check; see ADR-0006.

## 14. Rust decoding implementation (M1)

Implemented in `crates/fig-reader`:

```text
read bytes
  -> detect ZIP vs raw (crates/fig-reader/src/container.rs)
  -> read header, chunk lengths with checked arithmetic (src/kiwi/payload.rs)
  -> inflate schema (flate2 / miniz_oxide, raw deflate)
  -> parse schema with bounded counts (src/kiwi/schema.rs)
  -> zstd decode message with a window cap (ruzstd)
  -> decode message with bounded arrays/depth (src/kiwi/cursor.rs, decode.rs)
  -> FigNode list + blob ranges in FigDocument
```

M1 results on four committed fixtures:

- `circle.fig`: version 101, 550 definitions, 5 nodes, 3 blobs (540 bytes).
- `OpenFigs.fig`: version 106, 550 definitions, 5 nodes, 21 blobs (12,098 bytes).
- `straight-2-point-line.fig`: version 101, **627** definitions, 4 nodes.
- `corner-radius-middle-dot-20.fig`: version 101, 627 definitions, 4 nodes.

The 627-definition files prove the schema is not fixed at 550 and that version integers are
not a compatibility contract. A fifth research file (`coloradjusted.fig`) exposed an unknown
`BRUSH` node type and was reported as unknown rather than rejected.

`cargo fuzz run read_fig --sanitizer none -- -max_total_time=60` executed **829,577**
inputs with no crashes and 61 MB peak RSS.

Design rules actually followed:

- Every length, count, string, and chunk offset is checked before allocation.
- Byte arrays become `(offset, len)` ranges into the message buffer, not per-byte values.
- GUIDs are `(u32, u32)`, not strings.
- Matrices are typed with finite-number checks; non-finite geometry is reported.
- Blob indexes are bounds-checked at access time.
- Unknown enum values are preserved as `UNKNOWN(n)`; unknown fields in a known message are a
  decode error because the embedded schema should describe them.
- Recursion is capped at 64.

## 15. Fixtures and conformance

Initial corpus (MIT, see `RESEARCH_SOURCES.md`):

1. `circle.fig` — minimal, version 101, 5 nodes, 3 blobs.
2. `OpenFigs.fig` — richer, version 106, 21 blobs.
3. `vector-network/straight-2-point-line.fig` — smallest vector network.
4. `vector-network/corner-radius-middle-dot-20.fig` — proves vertex style IDs.
5. `vector-network/coloradjusted.fig` — 2.2 MB, images, 25k vertices.

Negative fixtures to construct ourselves: truncated header, chunk length overflow, zip-slip
entry name, decompression bomb, schema with absurd counts, deep parent chain, dangling blob
index, duplicate GUIDs.

## 16. Explicit unknowns

- Full node-type coverage across Figma versions; the list is empirical.
- Whether raw `GROUP` still occurs, and whether `resizeToFit` is the only group
  discriminator.
- Complete override semantics and variant matching rules.
- All variable consumption fields and how modes should map to Penpot themes.
- Exact behavior for boolean operations whose child geometry is only cached.
- Text list formatting, paragraph spacing, and per-character style details.
- Whether Figma's embedded schema ever includes fields our chosen Kiwi crate cannot decode
  (e.g. newer native types).
- Maximum real-world file sizes and schema definition counts; the 550-definition sample is
  not a ceiling.
