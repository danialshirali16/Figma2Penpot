# Support Matrix

Last updated: **2026-11-01** (M1). The `.fig` reader and `inspect` CLI are implemented and
tested; every conversion feature remains **Planned**. Nothing here is **Verified** by a real
Penpot import.

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
| Raw node metadata (type/name/GUID/transform) | **Partial** | Core fields only; paints/text/vector blobs not mapped yet |

## 2. Document structure

| Feature | Status | Notes |
|---|---|---|
| Pages (canvases) | Planned | M2 |
| Internal-only canvas exclusion | Planned | M2 |
| Frames / boards | Planned | M2 |
| Groups (including frame-encoded groups) | Planned | M2 |
| Sections | Planned | M3 |
| Nested frames and absolute coordinates | Planned | M2; ADR-0005 |
| Layer order (`parentIndex.position`) | Planned | M2 |
| Hidden/locked layers | Planned | M2 |
| Page background | Planned | M2 |

## 3. Shapes and geometry

| Feature | Status | Notes |
|---|---|---|
| Rectangles and corner radii | Planned | M2 |
| Ellipses / circles | Planned | M2 |
| Polygons and stars | Planned | M3 |
| Vector paths (`commandsBlob`) | Planned | M3.1 |
| Editable vector networks | Planned | M3.1; handle metadata may be Partial |
| Boolean operations | Planned | M3.1; preservation vs fallback documented |
| Masks and clipping | Planned | M3.2 |
| Rotation and flip | Planned | M2/M3; representation needs a fixture |
| Constraints and resizing | Planned | M3.5 |

## 4. Appearance

| Feature | Status | Notes |
|---|---|---|
| Solid fills | Planned | M2 |
| Solid strokes, width, alignment | Planned | M2 |
| Gradient fills | Planned | M3.2; gradient transform is the hazard |
| Opacity and blend modes | Planned | M2/M3.2 |
| Shadows (drop/inner) | Planned | M3.2 |
| Blur (foreground/background) | Planned | M3.2 |
| Procedural effects (grain, noise, glass, etc.) | Not supported | Report as not converted |
| Image fills | Planned | M3.3; dedup by hash |

## 5. Text

| Feature | Status | Notes |
|---|---|---|
| Single-style text | Planned | M2 |
| Paragraphs and basic alignment | Planned | M2 |
| Mixed/per-character styles | Planned | M3.4 |
| Line height and letter spacing | Planned | M3.4 |
| Text styles | Planned | M3.6 |
| Lists (bulleted/numbered) | Not supported | Penpot has no list model; marker may be written into text and reported |
| Missing fonts | Planned | Record and report; never silently substitute |

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
| Structural validator | Planned | M2 |
| JSON report | Planned | M2 |
| HTML/CSV report export | Planned | M6 |
| Headless CLI | Planned | M1/M2 |
| Native desktop UI | Planned | M4 |
| Batch queue, cancel, retry | Planned | M5 |
| Real Penpot import verification | Planned | Pending: no instance available |

## 8. Non-goals

Penpot → Figma, Figma cloud/REST fetching, plugins, real-time sync, accounts, editing
designs, automatic repair beyond conversion. These are **Not supported** and must be
reported as such if a user attempts them.
