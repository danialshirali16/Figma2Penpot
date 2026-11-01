# Support Matrix

Last updated: **2026-11-01** (M3.1). M2 features and filled vector paths imported
successfully into a local Penpot 2.18.5 stack (`2.18.5-RC1`) and were inspected in the UI.
Stroke-only vector outlines are Partial; booleans and editable networks remain Planned.

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
| Images embedded in the ZIP | **Partial** | Entries and sizes enumerated; bytes not loaded/mapped yet |
| Unknown fields/node types | **Partial** | Unknown enum types reported (`BRUSH` fixture); unknown message fields error |
| Safe archive/parser limits | **Implemented (unverified)** | M1; entry, size, ratio, node, depth, string caps |
| `f2p-cli inspect` | **Implemented (unverified)** | M1; integration-tested on real fixtures |
| Raw node metadata (type/name/GUID/transform) | **Partial** | Core geometry, solid paints, basic text, and vector blob indexes; components/variables not mapped |
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
| Editable vector networks | Planned | M3.1 follow-up; handle metadata not yet used |
| Boolean operations | Planned | M3.1; preservation vs fallback documented |
| Masks and clipping | Planned | M3.2 |
| Rotation and flip | **Partial** | M2 reports rotation/skew as approximated and keeps translation/scale bounds |
| Constraints and resizing | Planned | M3.5 |

## 4. Appearance

| Feature | Status | Notes |
|---|---|---|
| Solid fills | **Verified** | M2; red fill and green 50% fill inspected |
| Solid strokes, width, alignment | **Verified** | M2; red rect black 2 px; circle stroke visible |
| Gradient fills | Planned | M3.2; gradient transform is the hazard |
| Opacity and blend modes | **Partial** | Fill opacity verified; node opacity and blend modes not fixture-tested |
| Shadows (drop/inner) | Planned | M3.2 |
| Blur (foreground/background) | Planned | M3.2 |
| Procedural effects (grain, noise, glass, etc.) | Not supported | Reported as not converted |
| Image fills | Planned | M3.3; dedup by hash |

## 5. Text

| Feature | Status | Notes |
|---|---|---|
| Single-style text | **Verified** | M2; Inter 16/400 text rendered and inspected |
| Paragraphs and basic alignment | **Partial** | One paragraph/run; horizontal alignment not yet written |
| Mixed/per-character styles | Planned | M3.4 |
| Line height and letter spacing | Planned | M3.4 |
| Text styles | Planned | M3.6 |
| Lists (bulleted/numbered) | Not supported | Penpot has no list model; marker may be written into text and reported |
| Missing fonts | **Partial** | Font name/style/size preserved; availability not checked; fallback is reported |

## 6. Layout and design system

| Feature | Status | Notes |
|---|---|---|
| Auto Layout → flex | Planned | M3.5 |
| Grid layout | Planned | M3.5 |
| Paint/text/effect styles | Planned | M3.6 |
| Components | Planned | M3.7 |
| Instances and overrides | Planned | M3.7 |
| Variants | Planned | M3.7 |
| Variables / modes | Planned | M3.8; lossy mapping must be reported |
| Design tokens | Planned | M3.8; tokens file deferred until mapped |

## 7. Delivery and product

| Feature | Status | Notes |
|---|---|---|
| Structural validator | **Verified** | M2; generated archives pass; real imports accepted |
| JSON report | **Verified** | M2; event-based counts, schema version 1; tested |
| HTML/CSV report export | Planned | M6 |
| Headless CLI | **Verified** | `inspect` + `convert` used for the import |
| Native desktop UI | Planned | M4 |
| Batch queue, cancel, retry | Planned | M5 |
| Real Penpot import verification | **Verified (M2 subset)** | Penpot 2.18.5-RC1; see `MANUAL_QA.md` |

## 8. Non-goals

Penpot → Figma, Figma cloud/REST fetching, plugins, real-time sync, accounts, editing
designs, automatic repair beyond conversion. These are **Not supported** and must be
reported as such if a user attempts them.
