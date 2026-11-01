# Architecture

Status: M0 design baseline; updated at M3.1. The reader, M2 converter, vector-path
expansion, `.penpot` writer, structural validator, JSON report, and CLI are implemented.
The GUI, batch queue, HTML/CSV export, and fidelity beyond M3.1 are not.

This document describes the intended Rust system, what was learned from the upstream
projects, and why the implementation is a reimplementation rather than a port.

Related: `docs/FIG_FORMAT.md`, `docs/PENPOT_FORMAT.md`, `docs/RISKS.md`,
`docs/LICENSE_ASSESSMENT.md`, `Roadmap.md`.

---

## 1. Constraints that shape the design

1. **Pure Rust.** No JavaScript engine, Node.js, webview UI, subprocess converter, or
   backend. Native GUI via `egui`/`eframe`.
2. **Offline.** Conversion performs no network I/O. Dependencies must not phone home.
3. **Event-sourced honesty.** Every decision is recorded when it happens, not inferred
   from output.
4. **Per-file isolation.** A panic or error in one job cannot take down the batch.
5. **Bounded resources.** Input is untrusted; no unbounded allocation, recursion, or
   extraction.
6. **Atomic output.** Temp file → validate → rename. No partial file is ever presented as
   a success.
7. **Testability without a display.** Conversion and report logic live in headless crates;
   the GUI is a client of the same API as the CLI.

## 2. What the upstream projects taught us

### 2.1 The reference converter

`girafic/penpot-converter` is a Node/TypeScript pipeline:

```text
.fig / .sketch
  -> openfig-core (MIT, JS) for .fig; hand-written reader for .sketch
  -> PenpotDocument (JSON IR)
  -> upstream exporter's ui-src/parser, taken 1:1 (MPL-2.0)
  -> .penpot
```

Its packages:

| Package | Role |
|---|---|
| `@ppc/core` | Upstream `ui-src/parser`, `lib/types`, fonts, common |
| `@ppc/fig` | `.fig` → `PenpotDocument` via `openfig-core` |
| `@ppc/sketch` | `.sketch` → `PenpotDocument` (out of scope for v1) |
| `@ppc/cli` | arguments, page selection, statistics, API upload |

Useful behaviors and documented traps:

- Absolute coordinates: Penpot stores page-absolute geometry; `.fig` is parent-relative.
- Sibling order is a lexicographic fractional index string, not array order.
- `undefined` becomes `nil` through ClojureScript and can fail Penpot's schema; optional
  keys must be omitted, not set to null.
- A plain group can be encoded as a frame with `resizeToFit`; classify above the raw node.
- Components need override resolution via `overrideKey` paths, and instances can nest.
- Tokens map Figma variables/modes onto Tokens Studio sets/themes with sanitized names.
- Large component-heavy files are slow because of *instance expansion*, not file size; page
  splitting is the practical workaround.

What it does **not** provide:

- No structured per-element diagnostic log. It prints aggregate counters and a few
  warnings; the report cannot answer "which node lost its binding?".
- No native desktop UI, no batch queue, no atomic-write story, no Rust.
- Mixed license provenance: its own repo declares no license, and its core is copied from
  MPL-2.0 code.

Its README also contradicts itself on effects: line 947 lists effects as implemented,
line 951 lists them as open. This is why the Rust implementation treats upstream claims as
research leads, not truth.

### 2.2 The official exporter

`penpot/penpot-exporter-figma-plugin` (MPL-2.0) is the behavioral reference for the
Figma→Penpot mapping. It builds a `PenpotDocument` and serializes it through
`@penpot/library`, which emits the v3 ZIP.

The seam is the useful part:

```text
Figma API reader  ->  PenpotDocument IR  ->  official serializer  ->  .penpot
```

We replace both the reader and the serializer with Rust. We do not reuse the code.

## 3. Crate layout

```text
figma-2-penpot/
├── Cargo.toml                 workspace
├── crates/
│   ├── fig-reader/            container, Kiwi decode, raw node tree
│   ├── conversion-core/       IR, passes, mapping rules, diagnostics
│   ├── penpot-writer/         Penpot document model + archive writer + validator
│   ├── conversion-report/     event model, counts, JSON/HTML/CSV
│   ├── conversion-queue/      jobs, scheduler, cancellation, progress
│   ├── f2p-cli/               headless inspect/convert/batch
│   └── desktop-app/           egui/eframe UI
├── tests/
│   ├── fixtures/              redistributable inputs + provenance
│   ├── integration/           end-to-end batch and failure tests
│   └── golden/                expected archives/reports
├── docs/
└── .github/workflows/
```

Dependency direction:

```text
desktop-app -> conversion-queue -> conversion-core -> fig-reader
                                                   -> penpot-writer
                                                   -> conversion-report
f2p-cli     -> conversion-queue -> conversion-core
```

`conversion-core` must not depend on `egui`, `rfd`, or any UI crate. The report crate is a
sink, not a global logger; core receives a `&mut dyn DiagnosticSink`.

## 4. Pipeline

```text
bytes
  │
  ├─ fig-reader
  │    container detect -> safe ZIP/raw read -> chunk framing
  │    -> Kiwi schema decode -> message decode -> RawDocument
  │
  ├─ conversion-core
  │    build tree + order siblings
  │    normalize transforms to absolute
  │    resolve styles
  │    resolve variables/tokens
  │    resolve components/instances/overrides
  │    map layout (auto-layout -> flex/grid)
  │    map geometry (rects, ellipses, paths, booleans, masks)
  │    map text runs
  │    -> PenpotDocument (typed, not JSON strings)
  │
  ├─ penpot-writer
  │    assign ids -> serialize file/pages/shapes/assets
  │    -> ZIP to temp -> structural validate -> atomic rename
  │
  └─ conversion-report
       events + traversal counts -> FileReport -> JSON/HTML/CSV
```

Each pass is a pure-ish function over the document plus a mutable sink. No pass may write
output directly; the writer is the only component that touches the destination.

### 4.1 Pass ordering rationale

- **Tree before transforms**: sibling order and parent links must be established first.
- **Transforms before geometry**: absolute coordinates are needed for `x/y`, `selrect`,
  `points`, and path content.
- **Styles before paints**: a paint style reference may replace a stale literal.
- **Variables before components**: an instance override may be a variable binding.
- **Components before layout**: an instance's materialized children affect flex bounds.
- **Layout before final geometry**: flex/grid may reposition children after sizing.
- **Text runs last**: they depend on fonts, styles, and final frame width.

Passes may be revisited if a dependency is discovered; the order is a design decision, not
a law. Record changes in an ADR if they affect fidelity.

## 5. Core data model

Names are indicative, not final.

```rust
pub struct RawDocument {
    pub prelude: Prelude,          // FigKiwi | FigDeck | FigJam | Unknown
    pub version: u32,
    pub schema_defs: usize,
    pub nodes: Vec<RawNode>,       // flat, preserves order and phase
    pub blobs: Vec<Vec<u8>>,       // bounds-checked index
    pub images: ImageStore,        // hash -> bytes, streamed
}

pub struct RawNode {
    pub guid: Guid,                // (session_id, local_id)
    pub parent: Option<Guid>,
    pub position: Option<String>,  // fractional index, lexicographic
    pub kind: RawKind,             // known type or Unknown(String)
    pub name: String,
    pub phase: Phase,
    pub visible: Option<bool>,
    pub size: Option<Size>,
    pub transform: Matrix,         // with per-field finite checks
    pub fields: FieldBag,          // unknown fields retained for diagnostics
}
```

Normalized IR:

```rust
pub struct FigmaDocument {
    pub pages: Vec<Page>,
    pub nodes: Arena<Node>,        // stable ids, no parent-index arithmetic
    pub styles: StyleTable,
    pub components: ComponentIndex,
    pub variables: VariableTable,
    pub images: ImageTable,
    pub origin: SourceInfo,        // file name, version, stats
}

pub struct Page {
    pub id: NodeId,
    pub name: String,
    pub background: Option<Color>,
    pub children: Vec<NodeId>,
    pub is_internal_only: bool,
}
```

The arena avoids recursive ownership and makes traversal, cancellation checks, and report
paths cheap. Each node stores its page and ancestor path for diagnostics, either directly
or via a resolved path table.

## 6. Diagnostics and report

### 6.1 Model

```rust
pub enum Severity { Info, Warning, Error }
pub enum Outcome { Converted, ConvertedWithModification, NotConverted, Failed }

pub struct ElementRef {
    pub figma_node_id: String,
    pub name: String,
    pub element_type: String,
    pub page: Option<String>,
    pub path: Vec<String>,
}

pub struct ReportEvent {
    pub code: IssueCode,          // stable, documented
    pub severity: Severity,
    pub outcome: Outcome,
    pub element: ElementRef,
    pub description: String,
    pub reason: Option<String>,
    pub action_taken: String,
    pub suggested_action: Option<String>,
}
```

### 6.2 Sink

```rust
pub trait DiagnosticSink {
    fn emit(&mut self, event: ReportEvent);
    fn count_detected(&mut self, kind: ElementKind);
}
```

Rules:

- Passes receive `&mut dyn DiagnosticSink`; there is no global logger path.
- A missing sink is a compile-time impossibility for a pass that needs one, not a runtime
  default that silently drops events.
- Every transformation branch must either emit an event or be provably identical.
  Code review enforces this; tests assert the expected code for each unsupported case.
- Grouping for display is a report-layer concern. The event list keeps element-level
  references.

### 6.3 Counts

Counts are derived from a real traversal plus events:

- `detected`: incremented once per element the converter decides to handle.
- `converted`: element has at least one event and no modification/not-converted/failed.
- `approximated`: element has an event with `ConvertedWithModification`.
- `skipped` / `unsupported`: terminal outcomes mapped from `NotConverted`.
- `failed`: element conversion panicked or returned an error.

An element can emit several events (for example, a frame converts but one fill is
approximated). Its **element outcome is the worst outcome among its events**, so
`converted + approximated + skipped + failed == detected` for elements, while event counts
may be larger. This invariant is tested on every golden fixture.

### 6.4 Validation state

`ValidationState` is `Passed | Failed | NotRun`. A file that converted but could not be
validated is reported as `Completed — not validated`, never as a clean success.

## 7. Batch architecture

```text
UI / CLI
  -> JobQueue (bounded, persistent in memory)
       -> Scheduler (semaphore, user concurrency limit)
            -> worker: catch_unwind(convert_one(job))
                 -> progress events -> channel -> UI
                 -> FileReport -> results store
```

Job state machine:

```text
Queued -> Analyzing -> Converting -> Validating -> Completed
                                               -> CompletedWithWarnings
                                               -> Failed
                                               -> Cancelled
```

- Each job has a cancellation token, checked between passes and inside long loops
  (node traversal, asset copy, zip entry write).
- Cancellation deletes the temp file; it never renames.
- Per-job panic containment uses `catch_unwind` around the whole job body, not around
  individual passes, so a poisoned sink cannot corrupt the queue.
- Bounded concurrency: default `min(4, max(1, physical_cores / 2))`, further reduced if
  available memory is low. The exact formula is documented and user-overridable.
- Memory scales with *active* jobs plus one file's working set, not with queue length.
- Assets are streamed: decode image dimensions on demand, copy bytes to the archive
  without holding all images for all jobs.
- Startup cleanup removes stale temp files in the destination volume, identified by a
  marker prefix and age, never by deleting arbitrary files.

## 8. Desktop application

`egui`/`eframe`, native window, no webview. App ID `app.figma2penpot.desktop`.

State machine:

```text
Home/Import -> Queue -> Analysis -> Settings -> Progress -> Results -> Report
                                                                  -> Preferences
```

- The UI owns view state only. Conversion state lives in `conversion-queue` and is
  delivered as an event stream; the UI does not poll unsynchronized shared state.
- Drag and drop, native file/folder pickers via `rfd`, multi-select, remove/reorder/clear.
- Long paths are middle-truncated with a full-path tooltip.
- Status uses icon + text + color, never color alone.
- Themes: one token source for light/dark; no hard-coded colors in widgets.
- All user-visible strings live in one module to allow localization later.
- Accessibility: keyboard navigation, focus order, labels for icon-only buttons,
  `accesskit` support where egui provides it. A screen-reader pass is a manual QA item;
  egui's accessibility story is good but not identical to native AppKit/WinUI.

Headless testability:

- `conversion-queue` and all core crates have no UI dependency.
- UI state reducers are pure functions tested with `egui_kittest` or plain unit tests.
- Screenshot tests for key screens, with a documented tolerance and platform caveat.

## 9. Security and resource limits

Inputs are untrusted. The parser must never panic, never allocate unboundedly, and never
write outside the destination.

- **ZIP**: reject `..`, absolute paths, backslashes, NULs; cap entry count, per-entry
  uncompressed size, total uncompressed size, and compression ratio; never extract by
  entry name.
- **Kiwi**: cap schema/message decompressed size, definition counts, array lengths,
  nesting depth, blob sizes, and node counts. M1 implements an in-house bounded decoder
  (ADR-0006); it is fuzzed rather than trusted.
- **Images**: identify by content hash; verify declared size/hash where available; do not
  decode untrusted image data unless needed; cap dimensions and total bytes.
- **Paths**: all output paths are constructed from our own UUIDs, never from input names.
  Names are metadata only.
- **Disk**: check free space before writing; fail with an actionable error, not a partial
  archive.
- **Panics**: no `unwrap`/`expect`/slice indexing on untrusted paths; `catch_unwind` at the
  job boundary is a safety net, not the primary strategy.
- **Unsafe**: none planned. Any `unsafe` requires an ADR, a safety comment, and a test.

Proposed limits are listed in `docs/FIG_FORMAT.md` §13. They are defaults, configurable,
and must be documented in the app's advanced settings.

## 10. Dependencies

All licenses below were checked against crates.io metadata on 2026-11-01. See
`docs/LICENSE_ASSESSMENT.md` for the full audit.

| Area | Crate | Version | License | Notes |
|---|---|---|---|---|
| GUI | `eframe`, `egui` | 0.37 | MIT OR Apache-2.0 | MSRV 1.95 |
| Dialogs | `rfd` | 0.17 | MIT | Native dialogs |
| Async orchestration | `tokio` | 1.54 | MIT | Limited to queue/IO |
| Parallelism | `rayon` | 1.12 | MIT OR Apache-2.0 | CPU-bound work |
| Channels | `crossbeam-channel` or `flume` | 0.5 / 0.12 | MIT OR Apache-2.0 / Apache-2.0 OR MIT | Pick one when the queue lands (M5) |
| Serialization | `serde`, `serde_json` | 1.0 | MIT OR Apache-2.0 | — |
| ZIP | `zip` | 9.2 | MIT | Disable unused features |
| Deflate | `flate2` + `miniz_oxide` | 1.1 / 0.9 | MIT OR Apache-2.0 | Pure Rust backend |
| Zstd read | `ruzstd` | 0.9 | MIT | Pure Rust; **not** the C `zstd` binding |
| Kiwi | in-house decoder | — | Project license | ADR-0006; bounded, fuzzed |
| Errors | `thiserror` (libs), `anyhow` (apps) | 2.0 / 1.0 | MIT OR Apache-2.0 | — |
| Logging | `tracing`, `tracing-subscriber` | 0.1 / 0.3 | MIT | — |
| IDs | `uuid` | 1.29 | Apache-2.0 OR MIT | — |
| Hashing | `sha1` | 0.11 | MIT OR Apache-2.0 | Image naming |
| Geometry | `kurbo` | 0.13 | Apache-2.0 OR MIT | Evaluate in M3; not needed for M2 |
| Images | `image` | 0.25 | MIT OR Apache-2.0 | Defer until thumbnails/dimensions |
| CSV | `csv` | 1.4 | Unlicense OR MIT | Report export |
| Open folder | `open` | 5.4 | MIT | Results screen |
| Config dirs | `directories` | 6.0 | MIT OR Apache-2.0 | Preferences |
| Temp files | `tempfile` | 3.27 | MIT OR Apache-2.0 | Atomic write |
| Snapshots | `insta` | 1.49 | Apache-2.0 | Golden tests |
| Property tests | `proptest` | 1.12 | MIT OR Apache-2.0 | Parser invariants |
| UI tests | `egui_kittest` | 0.37 | MIT OR Apache-2.0 | Headless UI |
| FS assertions | `assert_fs` | 1.1 | MIT OR Apache-2.0 | Integration tests |
| Fuzzing | `cargo-fuzz` / libFuzzer | tool | — | Linux CI preferred |
| Compliance | `cargo-deny`, `cargo-audit`, `cargo-about` | tools | — | M8 |

Deliberately avoided:

- The C `zstd` binding (BSD-3, native build complexity) — `ruzstd` is enough for reading.
- `image` in the core read path — it is not needed to parse node data, and decoding
  untrusted images is a separate risk surface.
- Any crate that embeds a JS engine, webview, or network client in the conversion path.

## 11. Testing strategy

| Level | What | Tooling |
|---|---|---|
| Unit | Matrix math, name sanitization, color conversion, path transform, hex parsing | `#[test]`, `proptest` |
| Reader | Fixtures, negative samples, truncation, limits | `insta`, `assert_fs` |
| Writer | Golden archive structure, validator, exact JSON invariants | `insta` |
| Report | Counts invariant, issue codes, export snapshots | `insta` |
| Integration | CLI convert, batch mixed outcomes, cancellation, panic isolation | `assert_fs` |
| Fuzz | Container, schema, message, path blobs, report parsers | `cargo-fuzz` |
| UI | State reducers, headless widget tests | `egui_kittest` |
| Manual | Real Penpot import, keyboard, HiDPI, dark mode, long paths | `docs/MANUAL_QA.md` |

Golden policy:

- Golden archives are compared structurally, not byte-for-byte, because ZIP metadata and
  JSON key order can vary. Normalize IDs and timestamps for comparison.
- Every golden report includes the issue codes it expects; a new warning fails the test
  unless the fixture's expected list is updated deliberately.
- A test that asserts "no events" is as important as one that asserts a specific event.

## 12. Implementation strategy

1. **M1 read path (done)**: `fig-reader` with container, schema decode, typed extraction,
   limits, fuzz target, `inspect` CLI. No conversion yet.
2. **M2 vertical slice (done locally)**: pages/frames/groups/rectangles/ellipses, solid
   paints, opacity, basic text → writer → validator → JSON report. Real import pending.
3. **M3 depth**: vectors, effects, images, text, layout, styles, components, tokens, in that
   order. M3.1 filled vector paths are done and import-verified; each remaining feature gets
   its own fixture, golden report, and support-matrix row.
4. **M4-M6**: UI and batch around the already-tested core; never the other way round.
5. **M7-M8**: fuzz long runs, large-file corpus, benchmarks, installers.

## 13. Open architecture questions

- Whether `conversion-core` should expose a streaming node API or materialize an arena.
  Arena is simpler and bounded; a 1M-node file at ~200 bytes/node is ~200 MB, within the
  proposed limit but tight on 8 GiB with concurrent jobs.
- Whether to run each job in a process instead of a thread for stronger isolation. Threads
  with `catch_unwind` are the default; process isolation is a fallback if a dependency is
  found to corrupt global state.
- Whether `egui` can meet the accessibility bar on Windows and macOS without custom
  platform work. This is a real risk, not a checkbox; see `docs/RISKS.md`.
- ~~Whether to vendor `kiwi-schema` for resource limits or write a decoder.~~ Resolved by
  ADR-0006: the decoder is in-house and bounded.
- How to represent Penpot rotation for non-axis-aligned shapes. Deferred to M3; do not
  invent a transform convention before a fixture proves it.
