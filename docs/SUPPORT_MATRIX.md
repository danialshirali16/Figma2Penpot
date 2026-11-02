# Support Matrix

Last updated: **2026-11-02** (session 16 private-file compatibility). M2 features, vectors, masks, boolean fallbacks,
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
| Sections | **Partial** | Converted to unclipped Penpot boards with children retained; section semantics differ and are reported. Five private-file imports accepted in Penpot 2.18.5-RC1. |
| Frame clipping | **Implemented (unverified)** | `frameMaskDisabled` maps directly to `showContent`, including supported instance overrides; tested and imported, detailed overlap comparisons pending. |
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
| Constraints and resizing | **Partial** | Constraint identifiers map; FIXED_MIN/FIXED_MAX are approximated. Instance transforms and resized child constraints are not fully recomputed and are explicitly reported. |
| Unknown container dimensions | **Partial** | When the transform and converted children are finite, derive target bounds from those children and report source dimensions as unknown; otherwise fail the node. |

## 4. Appearance

| Feature | Status | Notes |
|---|---|---|
| Solid fills | **Verified** | M2; red fill and green 50% fill inspected |
| Solid strokes, width, alignment | **Verified** | M2; red rect black 2 px; circle stroke visible |
| Gradient fills | **Verified** | M3.2; linear gradient with stops imported and rendered; radial mapped, not fixture-tested |
| Opacity and blend modes | **Partial** | Supported node modes map; unsupported linear modes use reported multiply/screen approximations, other unsupported modes use normal; per-paint modes remain partial. Validator rejects invalid target mode names. |
| Multiple fill/stroke order | **Implemented (unverified)** | Reverse source stacks for Penpot's rendering order; regression-tested and accepted in private imports, comprehensive visual comparisons pending. |
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
| Token preference propagation | **Partial** | Single-file CLI/desktop honors `keep_token_bindings`; the queue stores the preference but still uses default conversion options. Batch propagation needs a regression test and fix. |
| Grid layout | Not supported | M3.5; reported as `F2P-LAYOUT-GRID-UNSUPPORTED`; the container is kept without layout |
| Paint styles | **Verified** | M3.6; color library entry imported and linked from a fill |
| Text styles | **Verified** | M3.6; typography library entry imported and linked from text |
| Effect styles | **Partial** | M3.6; effects are inlined onto the shape and reported (`F2P-STYLE-EFFECT-INLINED`); no library linkage |
| Grid styles | Not supported | M3.6; reported as `F2P-STYLE-GRID-LOST` |
| Components | **Verified** | M3.7; main component registered and shown in the Assets panel |
| Instances and overrides | **Partial** | Restore required embedded masters on an Imported components page; build dependencies first, resolve nested instance override paths/keys, translate path controls with copied geometry. Properties, swaps, and some attributes remain unmapped and are reported. |
| Token-name compatibility | **Implemented (unverified)** | Escape unsupported characters, disambiguate collection-local collisions, update converted bindings, and emit `F2P-TOKEN-NAME-NORMALIZED`; tested and accepted in real imports, editing/link behavior pending. |
| Variants | **Verified** | M3.7; two variants registered under one variant set and collapsed in the Assets panel |
| Variables / modes | **Partial** | M3.8; color/number/string/boolean variables become token sets; modes become themes; layout/typography bindings are not applied |
| Design tokens | **Verified** | M3.8; `tokens.json` imported with Light/Dark themes; a fill shows `brand.primary` applied |

## 7. Delivery and product

| Feature | Status | Notes |
|---|---|---|
| Structural validator | **Verified** | M2; generated archives pass; real imports accepted |
| JSON report | **Verified** | M2; event-based counts, schema version 1; tested |
| HTML report export | **Verified** | Self-contained offline HTML rendered in a browser; no external assets |
| CSV report export | **Verified** | RFC 4180 escaping tested; per-file and batch aggregate output |
| CLI `report`/batch export | **Verified** | JSON report → HTML/CSV; batch HTML/CSV/JSON smoke-tested |
| Report screen filters | **Implemented (unverified)** | Search, severity/result, grouping by what-to-do/page/none, copy advice/layer, JSON/HTML/CSV export; canvas jump-to-element pending |
| Headless CLI | **Verified** | `inspect` + `convert` used for the import |
| Native desktop UI | **Partial** | Designer-first eight-screen eframe shell; light/dark screenshots at large/compact on macOS 27.2; native dialogs, drag-drop, keyboard-only, and screen-reader checks unverified; Windows/Linux visuals untested |
| Designer status/issue copy | **Verified** | Every file status, queue state, stage, outcome, severity, validation state, and known issue code maps to designer text; 30 codes tested |
| Single-file pause/cancel | **Implemented (unverified)** | Cooperative checkpoints between stages; cancellation before saving leaves no output; UI screenshots checked |
| Desktop report export | **Implemented (unverified)** | HTML/CSV/JSON through native save dialogs; exporters tested, dialog interaction not automated |
| Accessibility | **Partial** | WCAG AA contrast calculated for both palettes; text+icon+colour statuses; keyboard-only and AccessKit/VoiceOver verification pending |
| Desktop drag-and-drop / file picker | **Implemented (unverified)** | M4.1; code path exists; OS interaction not automated |
| Desktop folder scan | **Verified** | Recursive `.fig` scan, hidden folders skipped, bounded to 512 files; unit-tested |
| Desktop keyboard shortcuts | **Implemented (unverified)** | Cmd/Ctrl+O, Shift+O, Shift+A, Enter, comma, 1–8; headless key-event test only |
| Release artifacts | **Verified (v0.1.1)** | macOS app/CLI + DMG packaging, Windows zip, Linux tarball, all with SHA-256 |
| Installers / signing | **Partial** | macOS DMG published; Windows MSI and signing/notarization pending |
| Desktop headless UI tests | **Verified** | All eight screens render through `egui::Context`; 22 desktop tests |
| Batch queue | **Partial** | Bounded queue, CLI batch, desktop Convert all, retry/remove, cooperative pause/cancel; mid-write cancellation pending |
| CLI batch command | **Verified** | Ran 2 valid + 1 corrupt fixture: 2 completed, 1 failed, outputs/reports written |
| Batch retry/remove controls | **Implemented (unverified)** | Retry failed, retry selected, remove completed; UI manual check pending |
| Batch memory | **Verified** | 1/4/16 files at 2 workers: 5.2/6.8/7.0 MB peak RSS |
| Batch cancel mid-write | Planned | Cancellation is checked between stages and before saving; an in-flight atomic write is not interrupted, and that race is not exercised |
| Real Penpot import verification | **Verified (M2–M3.8 subset)** | Penpot 2.18.5-RC1; see `MANUAL_QA.md` |

## 8. Reliability and performance

| Feature | Status | Notes |
|---|---|---|
| Fuzz target | **Verified (one campaign)** | 818,204 runs / 61 s, 49 MB peak RSS, no artifacts; not a proof of absence |
| Conversion benchmarks | **Partial** | Release wall time and peak RSS recorded for 10 fixtures + one 4.6k-node file; single machine/run |
| Batch memory | **Verified** | 1/4/16 files at 2 workers: 5.2/6.8/7.0 MB peak RSS |
| Large-file corpus | **Partial** | Committed 3,303-node synthetic fixture + one local 4,629-node file; broader corpus pending |
| Regression baseline | **Verified (counts/status)** | 11 fixtures in `tests/benchmarks/baseline.json` + CLI integration test; timing reference with 30% manual tolerance |
| Error recovery drills | **Partial** | Atomic write failure + temp cleanup, batch isolation, overwrite refusal tested; low-disk/cancel-mid-write pending |
| Large output archives | **Implemented (unverified)** | Configurable validator defaults: 300,000 entries, 64 MiB/entry, 512 MiB total. Core expansion/cache limit 250,000 shapes; bounds/cycles are tested. Five private archives imported; exhaustive resource/fuzz coverage pending. |
| Private-file import compatibility | **Verified** | All five supplied files accepted by Penpot 2.18.5-RC1. Overall fidelity remains **Partial**; reports contain warnings and unsupported outcomes. See `MANUAL_QA.md`. |

## 9. Non-goals

Penpot → Figma, Figma cloud/REST fetching, plugins, real-time sync, accounts, editing
designs, automatic repair beyond conversion. These are **Not supported** and must be
reported as such if a user attempts them.
