# Penpot `.penpot` Format — Target Model

Status: M0 research baseline; updated at M2. The M2 writer exists in
`crates/penpot-writer` and targets the subset in `SUPPORT_MATRIX.md`. It emits a v3 archive
and runs a structural validator before writing. **Real Penpot import is still pending.**

Confidence labels are the same as in `docs/FIG_FORMAT.md`: **Confirmed**, **Reported**,
**Hypothesis**, **Unknown**.

Primary sources: Penpot `2.18.5` source (`f445669`), its technical file-format document,
and a real archive generated in M0 with `@penpot/library 1.2.0`. See
`docs/RESEARCH_SOURCES.md`.

---

## 1. Target pin

| Item | Value | Why |
|---|---|---|
| Penpot release | **2.18.5** (2026-10-26) | Current stable at research time |
| Archive format | **v3** (ZIP + JSON) | Current; v1 is a deprecated binary format |
| Manifest version | **1** | Written by Penpot 2.18.5 and the library |
| File data version | **67** | Observed in generated output; migrations end at `legacy-67` |
| Reference generator | `@penpot/library` **1.2.0** | Official builder; produced the probe archive |
| Import acceptance | internal binfile versions **1 and 3** | `import_binfile.clj` schema enum |

Do not silently target "latest". A later Penpot release may change the data version or
import rules; that requires a new ADR and new golden fixtures.

## 2. Archive layout (v3)

**Confirmed** by generating `probe.penpot` with the official library:

```text
manifest.json
files/<file-id>.json
files/<file-id>/pages/<page-id>.json
files/<file-id>/pages/<page-id>/<shape-id>.json
files/<file-id>/media/<media-id>.json
files/<file-id>/components/<component-id>.json
files/<file-id>/colors/<color-id>.json
files/<file-id>/typographies/<typography-id>.json
files/<file-id>/tokens.json
files/<file-id>/tokens-status.json
files/<file-id>/thumbnails/<tag>/<page-id>/<frame-id>.json
objects/<object-id>.json
objects/<object-id>.<ext>
```

Notes:

- All JSON is UTF-8 and uses **camelCase** keys on disk; the importer converts to kebab-case
  keywords internally.
- The root frame is itself a shape file named
  `00000000-0000-0000-0000-000000000000.json` on every page. Penpot will not open a page
  without it.
- A minimal file only needs `manifest.json`, the file JSON, page JSONs, the root frame, and
  shape files. Optional collections can be omitted.
- `objects/` holds binary media and its metadata; `files/.../media/` holds the file-level
  reference. A media reference without a matching object is a broken file.
- ZIP entry order is not significant. The importer indexes entries by path shape
  (`backend/src/app/binfile/v3.clj:660-730`).

## 3. `manifest.json`

Schema in Penpot 2.18.5 (`backend/src/app/binfile/v3.clj:55-71`):

| Field | Type | Required | Notes |
|---|---|---|---|
| `version` | integer | Yes | `1` in v3 archives |
| `type` | string | Yes | `"penpot/export-files"` |
| `referer` | string | No | Source system; library writes its referer |
| `generatedBy` | string | No | Tool name/version |
| `files` | array | Yes | At least one file |
| `files[].id` | UUID | Yes | Must match `files/<id>.json` |
| `files[].name` | string | Yes | Display name |
| `files[].features` | string[] | Yes | Feature flags |
| `relations` | array | No | Pairs of file UUIDs |
| `externalLibraries` | array | No | Added in a later release; not in 2.18.5 |

Observed archive (`probe.penpot`, IDs shortened):

```json
{
  "type": "penpot/export-files",
  "version": 1,
  "generatedBy": "penpot-library/1.2.0",
  "referer": "f2p-research",
  "files": [
    {
      "id": "ffddbef8-…",
      "name": "F2P Probe",
      "features": [
        "plugins/runtime", "design-tokens/v1", "variants/v1",
        "layout/grid", "styles/v2", "components/v2",
        "fdata/shape-data-type"
      ]
    }
  ],
  "relations": []
}
```

**Discrepancy:** the official technical document's example uses `refer`; the 2.18.5
schema uses `referer`, and the importer accepts either. Emit `referer`. Also note the
document's `generatedBy` example uses `penpot/2.12.0`; our writer must not pretend to be
Penpot. Use `figma-2-penpot/<version>`.

`externalLibraries` appeared after 2.18.5. Do not emit it for the pinned target; an
older importer may reject unknown manifest fields, depending on schema strictness.

## 4. File metadata — `files/<file-id>.json`

**Observed** minimal output from the official library:

```json
{
  "features": [ … same list as manifest … ],
  "name": "F2P Probe",
  "id": "ffddbef8-…",
  "isShared": false,
  "migrations": [ "legacy-2", … "0026-fix-svg-raw-shapes-uuids" ],
  "version": 67,
  "options": { "componentsV2": true, "baseFontSize": "16px" }
}
```

The importer fills in defaults for `revn`, `createdAt`, and `modifiedAt`
(`v3.clj`, `read-file`), so they are not required in an archive. The official format
document lists them as required; that describes a Penpot-exported file, not the minimum
the importer accepts. **Reality wins:** match the library.

Rules:

- `version` must be the data version the target understands. For 2.18.5 it is **67**.
- `migrations` should list the migrations already applied. The library emits the full
  current list; copy that list as data, not as code. If the list is wrong, Penpot may
  re-run migrations that assume an older shape and damage the file.
- File `features` must be a superset of the features whose data is present. Emit the same
  set as the library for the supported feature subset. Enabling a feature you do not
  actually write can make Penpot expect data that is absent.
- `options.componentsV2: true` and `baseFontSize: "16px"` are observed defaults.

## 5. Page metadata

`files/<file-id>/pages/<page-id>.json`:

```json
{ "id": "a72328c3-…", "name": "Page One", "index": 0 }
```

`index` is the display order. `background` is an optional hex color. Figma canvas
background maps here; the reference converter notes that dropping it can make light text
invisible on a dark Figma canvas (`toDocument.ts:281-296`).

### 5.1 Root frame

Every page has a root frame with the all-zero UUID:

```json
{
  "id": "00000000-0000-0000-0000-000000000000",
  "name": "Root Frame",
  "type": "frame",
  "x": 0, "y": 0, "width": 0.01, "height": 0.01,
  "rotation": 0,
  "selrect": { "x": 0, "y": 0, "width": 0.01, "height": 0.01,
               "x1": 0, "y1": 0, "x2": 0.01, "y2": 0.01 },
  "points": [ … four corners … ],
  "transform": { "a": 1, "b": 0, "c": 0, "d": 1, "e": 0, "f": 0 },
  "transformInverse": { "a": 1, "b": 0, "c": 0, "d": 1, "e": 0, "f": 0 },
  "parentId": "00000000-0000-0000-0000-000000000000",
  "frameId": "00000000-0000-0000-0000-000000000000",
  "fills": [ { "fillColor": "#FFFFFF", "fillOpacity": 1 } ],
  "strokes": [],
  "shapes": [ "<top-level-shape-id>" ],
  "pageId": "<page-id>"
}
```

The 0.01 size is intentional in the library output; do not "fix" it to 1 without evidence.
The root frame's `shapes` array defines top-level order.

## 6. Shape JSON

### 6.1 Required base fields

From `common/src/app/common/types/shape.cljc:172-189`:

| Field | Type | Notes |
|---|---|---|
| `id` | UUID | Unique within the file |
| `name` | string | Must be present; may be empty? The schema requires a string |
| `type` | enum | `frame`, `group`, `bool`, `rect`, `path`, `text`, `circle`, `image`, `svg-raw` |
| `selrect` | rect | `{x, y, width, height, x1, y1, x2, y2}` in observed output |
| `points` | point[] | Four corners in observed output, even for circles |
| `transform` | matrix | Observed as object `{a,b,c,d,e,f}` |
| `transformInverse` | matrix | Inverse of `transform` |
| `parentId` | UUID | Parent shape or root frame |
| `frameId` | UUID | Nearest containing frame |
| `x`, `y`, `width`, `height` | numbers | Required for all types except `path` and `bool` |

`pageId` is optional in the schema but the exporter writes it on every shape. Write it.

**Discrepancy:** the technical document shows `transform` as an array
`[a,b,c,d,e,f]`. The 2.18.5 library emits an object. Follow the object; the array example
is stale. **Unknown:** whether the importer normalizes arrays for backward compatibility.
Do not rely on it.

### 6.2 Absolute coordinates (critical)

Penpot stores shape `x/y`, `selrect`, `points`, and path content in **absolute page**
coordinates. Parent containers do not establish a local coordinate system. Evidence:

- The frame renderer draws children without a parent translation
  (`frontend/src/app/main/ui/shapes/frame.cljs`).
- The reference converter accumulates all ancestor transforms
  (`penpot-converter/.../toDocument.ts:614-615`; README lines 134-138).
- The official builder stores whatever `x/y` it is given; a nested probe with a frame at
  `(100,200)` and a child at `(5,7)` wrote the child at `(5,7)`, not `(105,207)`. The
  writer, not the library, must add the parent offset.

Consequences:

- Accumulate the Figma matrix from the page downward for every node.
- Update `selrect` and `points` consistently; they must describe the same absolute box.
- Path content must be transformed by the same accumulated matrix, not left in local space.
- Clipping and grouping are expressed by `parentId`/`frameId` and the frame's `shapes`
  list, not by a local transform.

ADR-0005 records this decision. Real import verification is still required before the
feature is called Verified.

### 6.3 Fills and strokes

Observed fill:

```json
{ "fillColor": "#FF0000", "fillOpacity": 1 }
```

Observed stroke:

```json
{
  "strokeColor": "#000000",
  "strokeOpacity": 1,
  "strokeWidth": 2,
  "strokeAlignment": "inner"
}
```

Other fill keys in the model: `fillColorGradient`, `fillImage`, `fillColorRefId`,
`fillColorRefFile`. The official document's `{color, opacity}` example is stale; the
2.18.5 schema uses `fillColor`/`fillOpacity` (`common/types/shape.cljc:483-484`).

Rules:

- Colors are `#RRGGBB` or `#RRGGBBAA` hex strings, not Figma's 0-1 floats. Convert and
  round-trip alpha explicitly.
- `strokeAlignment` is `inner`, `center`, or `outer`.
- Multiple fills and strokes are arrays; order is significant.
- Invisible paints should be omitted or preserved with a visibility flag only if the
  Penpot schema supports it for that paint type. Do not emit a paint with a missing color.

### 6.4 Text

Observed minimal text shape includes `content`:

```json
"content": {
  "type": "root",
  "children": [
    { "type": "paragraph-set",
      "children": [
        { "type": "paragraph",
          "children": [
            { "text": "Hello Penpot",
              "fontFamily": "Inter", "fontSize": "16",
              "fontWeight": "400", "fontStyle": "normal",
              "fills": [ { "fillColor": "#111111", "fillOpacity": 1 } ] }
          ] } ] } ] }
```

Notes:

- Font sizes, weights, line heights, and letter spacing are **strings** in the text tree,
  not numbers. This is a common cause of import rejection.
- `growType` controls auto-sizing (`fixed`, `auto-width`, `auto-height`). It is separate
  from Figma's `textAutoResize` and must be mapped deliberately.
- Mixed styling is represented by multiple text nodes in a paragraph, each with its own
  style. Figma's per-character overrides must be converted into runs.
- Missing fonts are recorded, not silently replaced. A fallback family may be written only
  with a report event.
- Empty strings have historically broken Penpot text handling (migration
  `0015-fix-text-attrs-blank-strings`). Do not emit empty style strings; omit the key or
  use a space for genuinely empty content.

### 6.5 Paths

A Penpot path stores geometry as a **command array**, not an SVG string:

```json
{
  "type": "path",
  "x": -229, "y": 5.9557, "width": 190.5, "height": 51.0443,
  "content": [
    { "command": "move-to",  "params": { "x": -229, "y": 57 } },
    { "command": "line-to",  "params": { "x": -227.7059, "y": 61.8296 } },
    { "command": "curve-to", "params": {
        "x": 50, "y": 60, "c1x": 20, "c1y": 30, "c2x": 30, "c2y": 40 } },
    { "command": "close-path", "params": {} }
  ],
  "svgAttrs": { "fillRule": "nonzero" }
}
```

Observed commands: `move-to`, `line-to`, `curve-to`, `close-path`. Curve params use
`c1x/c1y/c2x/c2y/x/y`. Coordinates are **absolute page coordinates**, consistent with the
other shapes.

M3.1 implementation:

- Figma `fillGeometry[].commandsBlob` is decoded from the blob (opcodes `0x01` moveTo,
  `0x02` lineTo, `0x04` cubicTo, `0x00` closePath).
- Each geometry becomes one Penpot path; one Figma vector can produce several shapes.
- All produced paths are linked as children of the same parent (`parentId`/`frameId` and the
  parent's `shapes` array).
- `windingRule` maps to `svgAttrs.fillRule` (`NONZERO → nonzero`, `ODD → evenodd`).
- Filled vectors use `fillGeometry`; stroke-only vectors use the expanded `strokeGeometry`
  as a filled path, because Figma stores no editable stroke for that case. This is reported
  as `F2P-VECTOR-STROKE-OUTLINE` and marked Partial.

Verified by importing a 20-path OpenFigs icon and a stroke-only line into Penpot
2.18.5-RC1; see `docs/MANUAL_QA.md`.

## 7. Components and library assets

Penpot's component model (from the official format document and exporter types):

- `files/<file-id>/components/<component-id>.json`:
  `{ id, name, path, mainInstanceId, mainInstancePage, modifiedAt?, objects? }`.
- A component's main instance is a frame/board shape with `componentRoot: true` and
  `componentId` set.
- An instance is a shape with `componentId`, `componentFile`, and `shapeRef` pointing at
  the corresponding shape inside the main instance.
- `touched` records which fields the instance overrides; without it, Penpot may resync the
  instance and discard the override.
- Variants use `variantId`, `variantName`, `variantProperties`, and
  `isVariantContainer` on the frame. Component property definitions/references are
  separate maps (`componentProperties`, `componentPropertyReferences`).
- `mainInstancePage` must name a page that exists; `mainInstanceId` must name a shape in
  that page.

**Status: Partial knowledge.** We know the field names from the exporter types and the
official document, but we have not generated or imported a component-bearing archive. Do
not claim component fidelity until M3 has a golden fixture and a real import.

## 8. Media

From the 2.18.5 exporter (`backend/src/app/binfile/v3.clj:300-330, 385-394`):

- Binary: `objects/<object-id>.<ext>` where extension comes from the content type.
- Metadata: `objects/<object-id>.json` with `{id, size, contentType, bucket, hash?}`.
- File reference: `files/<file-id>/media/<media-id>.json`, with the object id, dimensions,
  name, and media type.
- Thumbnails are `files/<file-id>/thumbnails/<tag>/<page-id>/<frame-id>.json`, pointing at
  a media/object id.

**Unknown:** the exact minimal `media` JSON accepted by 2.18.5 for an image fill. We will
derive it by generating an image-bearing archive and importing it; until then this is a
format item to confirm, not a finished design.

## 9. Design tokens

`files/<file-id>/tokens.json` holds a Tokens Studio-shaped library: sets, themes, and
token values. Penpot applies it file-wide. Figma modes do not map one-to-one.

We deliberately defer the full token mapping to M3.8. The structural writer must still
reserve the path and must not emit a partial tokens file that claims more than it contains.
Issues such as "mode dropped" and "binding lost" are report events, not silent omissions.

## 10. Import behavior and limits

### 10.1 What the importer accepts

- Archive versions 1 and 3; v3 is the ZIP format.
- Manifest version 1.
- File data version 67 for the pinned release.
- Paths are constructed from manifest UUIDs and known suffixes; the importer does not
  extract by arbitrary entry name. Our writer emits only canonical paths.
- Missing mandatory entries produce `:inconsistent-penpot-file`; malformed JSON or schema
  violations produce validation errors.

### 10.2 Limits in 2.18.5

| Limit | Value | Source |
|---|---:|---|
| Single storage object | 100 MiB | `backend/src/app/binfile/common.clj:56-58` |

That is the only explicit binfile cap in 2.18.5. Later development builds add per-entry
text caps (20 MiB), a total text cap (500 MiB), and a ZIP entry cap (500,000)
(`develop` `common.clj:62-87`). **Do not assume those exist in 2.18.5.**

Our writer must be stricter than the importer, because the importer's tolerance is not a
security boundary:

- Cap shape/page JSON size we generate (a single shape should be KB, not MB).
- Cap total text entries and total archive size.
- Refuse to emit a media object over 100 MiB; split or report, never silently truncate.
- Validate every path and UUID before writing.

### 10.3 Validation to implement (M2)

A structural validator must check, at minimum:

1. ZIP has a valid manifest; `version == 1`, `type == "penpot/export-files"`.
2. Every `files[].id` has a matching `files/<id>.json`, and vice versa.
3. Every page id referenced by the file exists; every page has the zero root frame.
4. Every shape id referenced by `shapes`, `parentId`, `frameId`, `componentId`,
   `mainInstanceId`, and `shapeRef` resolves.
5. No duplicate UUIDs anywhere in the archive.
6. Every shape has the required base fields; `type` is known; geometry numbers are finite.
7. `selrect` equals the bounding box of `points` (within tolerance) and matches
   `x/y/width/height`.
8. `transformInverse × transform ≈ identity`.
9. Fills/strokes use valid hex colors and known enums.
10. Text style numbers are strings, `content.type == "root"`, and paragraphs are non-empty.
11. Media references resolve to an object metadata entry and a binary entry.
12. No entry name escapes `files/`, `objects/`, or the manifest; no absolute paths, `..`,
    backslashes, or NUL bytes.
13. Total uncompressed size and entry count are within our own limits.

The validator is part of the product, not a test helper: "output validation" is a setting,
and a failed validation must prevent the temp file from being renamed to the final name.

## 11. Writer strategy

Implemented in `crates/penpot-writer` for the M2 subset:

- deterministic UUIDs derived from Figma GUIDs (ADR-0007);
- file, pages, root frames, frames/groups/rectangles/circles/basic text;
- solid fills/strokes, opacity, corner radii;
- JSON entries with canonical paths, manifest written last;
- structural validation of the generated bytes before atomic rename.

```text
PenpotDocument (typed)
  -> assign UUIDs (stable per run; deterministic in tests)
  -> emit file JSON, pages, root frames
  -> emit shapes top-down (absolute coordinates already resolved)
  -> emit components, media, tokens (when supported)
  -> write JSON entries with canonical paths
  -> write objects
  -> write manifest last (so a truncated archive is obviously incomplete)
  -> close ZIP to temp file
  -> run structural validator on the temp file
  -> atomic rename
```

Design rules:

- Use a ZIP writer that supports deterministic entry order and store/deflate per entry.
- JSON key order is not semantically significant, but stable order makes golden tests and
  diffs readable. Use `serde_json` with a stable map ordering or explicit structs.
- Write the manifest last. A missing manifest is an unambiguously incomplete archive.
- Never emit `null` for optional fields. Penpot's Clojure schema rejects `nil` where a
  string is expected; the reference converter hit this exact failure
  (`penpot-converter/README.md:140-145`). Omit the key instead.
- Use absolute coordinates; see §6.2.
- The target is 2.18.5. Do not use fields or features added in 2.19 without a new ADR.

## 12. Explicit unknowns

- Whether every field the official document marks "required" is actually enforced by the
  importer, and which fields are defaulted. We observed that file metadata is defaulted.
- Exact minimal component, media, and token JSON accepted by 2.18.5.
- Rotation representation on shapes: `rotation` plus `transform` plus axis-aligned
  `x/y/width/height`; the correct combination for rotated, non-axis-aligned shapes.
- How Penpot resolves `touched` and `shapeRef` during import; the rules are central to
  component fidelity.
- Whether the importer normalizes array-form transforms or `{color, opacity}` fills for
  older files.
- The precise behavior of the importer on unknown extra keys: ignored or rejected. Until
  tested, emit only known keys.
- Penpot's maximum accepted page/shape counts and archive size in practice.

Each unknown becomes a fixture and a test in M2/M3. None of them may be filled in by
guesswork in the writer.
