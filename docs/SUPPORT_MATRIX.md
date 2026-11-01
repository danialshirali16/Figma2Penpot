# Support Matrix

Last updated: **2026-11-01** (M4.1). M2 features, vectors, masks, boolean fallbacks,
gradients, shadows, blur, images, text metrics, flex containers, styles, components,
variants, and tokens imported successfully into a local Penpot 2.18.5 stack
(`2.18.5-RC1`) and were inspected in the UI. Stroke-only vectors, Auto Layout child sizing,
effect-style linkage, instance swaps/properties, and variable bindings are Partial;
mixed text runs and grid layout are reported losses.

Status vocabulary (no other labels are used):

- **Verified** — implemented, tested, confirmed (ideally by importing into Penpot).
- **Implemented (unverified)** — implemented and unit-tested, not confirmed in Penpot.
- **Partial** — some cases handled; limits documented.
- **Planned** — not started.
- **Not supported** — out of reach or scope; reported at runtime.

---

## 1. Input and container

| Feature | Status | Notes |
|---|---|---|
| ZIP-wrapped `.fig` | **Implemented (unverified)** | M1; tested on 4 fixtures, ZIP-slip/bomb tests |
| Raw `fig-kiwi` binary | **Implemented (unverified)** | M1; raw canvas fixture test |
| `.deck` / `.jam` | Not supported | Out of v1 scope; explicit `UnsupportedFormat` error |
| Embedded schema decode | **Implemented (unverified)** | M1; in-house bounded decoder, 550/627 defs |
| zstd and deflate message decoding | **Implemented (unverified)** | M1; zstd window capped at 128 MiB |
| Images embedded in the ZIP | **Verified** | M3.3; bytes loaded with size caps, 8×8 PNG fill imported and rendered |
| Unknown fields/node types | **Partial** | Unknown enum types reported (`BRUSH` fixture); unknown message fields error |
| Safe archive/parser limits | **Implemented (unverified)** | M1; entry, size, ratio, node, depth, string caps |
| `f2p-cli inspect` | **Implemented (unverified)** | M1; integration-tested on real fixtures |
| Raw node metadata (type/name/GUID/transform) | **Partial** | Geometry, paints, text, vector geometry, styles, components, and variables are mapped; constraints/blend modes are partially mapped |
| `f2p-cli convert` | **Verified** | M2; circle + synthetic text/group imported into Penpot 2.18.5-RC1 |
| M2 import verification | **Verified** | Both fixtures; backend `error?=false`; see `MANUAL_QA.md` |

## 2. Document structure

| Feature | Status | Notes |
|---|---|---|
| Pages (canvases) | **Verified** | M2; only `Page 1` after import |
| Internal-only canvas exclusion | **Verified** | M2; internal canvas absent |
| Frames / boards | **Verified** | M2; Card 400×300 at (20,30) |
| Groups (including frame-encoded groups) | **Verified** | M2; Badge Group with Group Rect child, 80×30 |
| Sections | Planned | M3 |
| Nested frames and absolute coordinates | **Verified** | M2; ADR-0005; inspector showed relative display for absolute data |
| Layer order (`parentIndex.position`) | **Partial** | Simple sibling order verified; complex fractional indices not fixture-tested |
| Hidden/locked layers | **Partial** | Visibility maps to `hidden`; locked/blocked not yet |
| Page background | Planned | M2 does not read Figma canvas background |

## 3. Shapes and geometry

| Feature | Status | Notes |
|---|---|---|
| Rectangles and corner radii | **Verified** | M2; radius 4 inspected |
| Ellipses / circles | **Verified** | M2; red ellipse rendered at 300×300 |
| Polygons and stars | **Partial** | Mapped through vector geometry when `commandsBlob` exists; no dedicated fixture |
| Vector paths (filled `commandsBlob`) | **Verified** | M3.1; OpenFigs 20-path icon imported and rendered |
| Stroke-only vector outlines | **Partial** | M3.1; outline converted to filled path, reported `F2P-VECTOR-STROKE-OUTLINE` |
| Editable vector networks | **Partial** | M3.1; rendered commands are editable; `vectorData` handle topology is not read and is reported (`F2P-VECTOR-NETWORK-LOST`) |
| Boolean operations | **Partial** | M3.1; rendered boolean geometry becomes editable paths, operands are not reconstructed (`F2P-BOOLEAN-FLATTENED`) |
| Rotation and flip | **Partial** | M2 reports rotation/skew as approximated and keeps translation/scale bounds |
| Constraints and resizing | **Partial** | M3.5; MIN/MAX/STRETCH/CENTER/SCALE map to Penpot constraints; FIXED_MIN/FIXED_MAX are approximated |

## 4. Appearance

| Feature | Status | Notes |
|---|---|---|
| Solid fills | **Verified** | M2; red fill and green 50% fill inspected |
| Solid strokes, width, alignment | **Verified** | M2; red rect black 2 px; circle stroke visible |
| Gradient fills | **Verified** | M3.2; linear gradient with stops imported and rendered; radial mapped, not fixture-tested |
| Opacity and blend modes | **Partial** | Fill/node opacity verified; node blend mode maps to Penpot, per-paint blend mode is reported as lost |
| Shadows (drop/inner) | **Verified** | M3.2; drop shadow imported and listed; inner shadow mapped, not fixture-tested |
| Blur (foreground/background) | **Verified** | M3.2; layer blur and background blur imported and listed |
| Procedural effects (grain, noise, glass, etc.) | Not supported | Reported as `F2P-EFFECT-UNSUPPORTED`; not converted |
| Image fills | **Verified** | M3.3; PNG media/object packaged; red 8×8 image rendered in Penpot |
| Masks and clipping | **Partial** | M3.2; alpha masks become Penpot masked groups and render correctly; mask mode is reported as approximated |

## 5. Text

| Feature | Status | Notes |
|---|---|---|
| Single-style text | **Verified** | M2; Inter 16/400 text rendered and inspected |
| Paragraphs and basic alignment | **Verified** | M3.4; text style and text imported; horizontal/vertical alignment written |
| Mixed/per-character styles | **Partial** | M3.4; flattened to the base run and reported (`F2P-TEXT-MIXED-STYLES-LOST`) |
| Line height and letter spacing | **Implemented (unverified)** | M3.4; percent/pixels/raw converted to Penpot units, unit-tested; values carried through the verified text style but not independently inspected |
| Text styles | **Verified** | M3.6; typography library entry imported and linked from a text run |
| Lists (bulleted/numbered) | Not supported | Penpot has no list model; marker may be written into text and reported |
| Missing fonts | **Partial** | Font name/style/size preserved; availability not checked; fallback is reported |

## 6. Layout and design system

| Feature | Status | Notes |
|---|---|---|
| Auto Layout → flex | **Partial** | M3.5; direction, gap, padding, and alignment verified in Penpot; child grow/fill sizing and constraints are partial (`F2P-LAYOUT-PARTIAL`) |
| Grid layout | Not supported | M3.5; reported as `F2P-LAYOUT-GRID-UNSUPPORTED`; the container is kept without layout |
| Paint styles | **Verified** | M3.6; color library entry imported and linked from a fill |
| Text styles | **Verified** | M3.6; typography library entry imported and linked from text |
| Effect styles | **Partial** | M3.6; effects are inlined onto the shape and reported (`F2P-STYLE-EFFECT-INLINED`); no library linkage |
| Grid styles | Not supported | M3.6; reported as `F2P-STYLE-GRID-LOST` |
| Components | **Verified** | M3.7; main component registered and shown in the Assets panel |
| Instances and overrides | **Partial** | M3.7; instances materialize and basic text/fill/size/visibility overrides apply; swaps and component properties are not mapped |
| Variants | **Verified** | M3.7; two variants registered under one variant set and collapsed in the Assets panel |
| Variables / modes | **Partial** | M3.8; color/number/string/boolean variables become token sets; modes become themes; layout/typography bindings are not applied |
| Design tokens | **Verified** | M3.8; `tokens.json` imported with Light/Dark themes; a fill shows `brand.primary` applied |

## 7. Delivery and product

| Feature | Status | Notes |
|---|---|---|
| Structural validator | **Verified** | M2; generated archives pass; real imports accepted |
| JSON report | **Verified** | M2; event-based counts, schema version 1; tested |
| HTML/CSV report export | Planned | M6 |
| Headless CLI | **Verified** | `inspect` + `convert` used for the import |
| Native desktop UI | **Partial** | M4.1; eight-screen eframe shell, native dialogs, background conversion, settings persistence; packaging/manual checklist pending |
| Desktop drag-and-drop / file picker | **Implemented (unverified)** | M4.1; code path exists; OS interaction not automated |
| Desktop folder scan | **Verified** | Recursive `.fig` scan, hidden folders skipped, bounded to 512 files; unit-tested |
| Desktop keyboard shortcuts | **Implemented (unverified)** | Cmd/Ctrl+O, Shift+O, Shift+A, Enter, comma, 1–8; headless key-event test only |
| Desktop headless UI tests | **Verified** | All eight screens render through `egui::Context`; 20 desktop tests |
| Batch queue | **Partial** | M5 core + CLI `batch` + desktop “Convert all”; bounded workers, stage events, pause/cancel, per-job failure isolation |
| CLI batch command | **Verified** | Ran 2 valid + 1 corrupt fixture: 2 completed, 1 failed, outputs/reports written |
| Batch retry/remove controls | **Implemented (unverified)** | Retry failed, retry selected, remove completed; UI manual check pending |
| Batch memory | **Verified** | 1/4/16 files at 2 workers: 5.2/6.8/7.0 MB peak RSS |
| Batch cancel mid-write | Planned | Atomic writer is used; the race is not exercised yet |
| Real Penpot import verification | **Verified (M2–M3.8 subset)** | Penpot 2.18.5-RC1; see `MANUAL_QA.md` |

## 8. Non-goals

Penpot → Figma, Figma cloud/REST fetching, plugins, real-time sync, accounts, editing
designs, automatic repair beyond conversion. These are **Not supported** and must be
reported as such if a user attempts them.
