# Architecture

Reviewed **2026-11-02** against the current source. This describes the implemented Rust
pipeline and its limits. Product goals remain in `PROJECT_BRIEF.md`; support and evidence
are in `SUPPORT_MATRIX.md`, `MANUAL_QA.md`, and `../PROGRESS.md`.

## 1. Boundaries and dependencies

The application is native Rust (`egui`/`eframe`), with no webview, JS engine, converter
subprocess, backend service, or network access in the conversion path. Research tools and
an isolated local Penpot verification stack are development-only, not shipped dependencies.
The reference converter and official exporter are behavior/format references under the
no-code-reuse decision (ADR-0002); ADR-0011 records the latest compatibility choices.

```text
desktop-app      -> conversion-queue, fig-reader, conversion-core,
                   penpot-writer, conversion-report
f2p-cli          -> conversion-queue, fig-reader, conversion-core,
                   penpot-writer, conversion-report
conversion-queue -> fig-reader, conversion-core, penpot-writer, conversion-report
conversion-core  -> fig-reader, penpot-writer, conversion-report
```

All seven crates exist. The core and queue have no UI dependency. Integration and golden
tests live under individual crate `tests/` directories; redistributable `.fig` fixtures
and the shared benchmark baseline live in the root `tests/` directory.

## 2. Read and convert

```text
.fig bytes
  -> fig-reader: safe ZIP/raw framing, embedded Kiwi schema, bounded message decode
  -> FigDocument: typed nodes, GUID/parent links, blob ranges, image bytes, warnings
  -> conversion-core: tree/order, styles/tokens, component dependency closure
  -> geometry, layout metadata, text, paints/effects, instance overrides + events
  -> PenpotDocument
  -> penpot-writer: serialize an in-memory ZIP, validate bytes
  -> write_atomic: temporary file on the destination volume, then persist/rename
  -> event-based FileReport and optional HTML/CSV exports
```

This is a materialized pipeline, not a streaming or independent pass framework. Image
bytes, component caches, output shapes, and serialized archives consume memory. Tree
transforms accumulate into page coordinates; rotation/skew and instance child resizing
remain Partial. Layout containers map to flex metadata without complete child reflow.
Grid layout, mixed text runs, and several component attributes remain explicit losses.

### Components and compatibility

`ComponentRegistry` computes the closure of visible/referenced symbols, including required
masters on internal-only canvases. Dependencies are built before users; cyclic references
are broken deterministically with diagnostics. Required internal masters are placed on an
additional **Imported components** page while the internal source canvas remains excluded.
Instances clone cached shapes, remap IDs/references, and apply supported overrides through
full GUID paths and imported-library `overrideKey` aliases. Path endpoints and control
points move with copied geometry. Unmapped instance attributes are reported.

Sections become unclipped boards with their children retained. Frame clipping maps from
`frameMaskDisabled` to `showContent`. Known empty fill caches can fall back to usable
stroke outlines. Non-finite container dimensions can be recovered only from finite
converted children, with source dimensions recorded as unknown. Non-finite layout values
use reported finite fallbacks. Paint stacks are reversed for the target ordering; invalid
target blend identifiers use diagnosed approximations. Token names and bindings use the
same escaping/disambiguation map. These behaviors do not establish lossless fidelity.

## 3. Diagnostics and reports

Each mapping receives `&mut dyn DiagnosticSink`. `count_detected(&ElementRef)` deduplicates
by source Figma GUID; `emit(ReportEvent)` records decisions during conversion. Events keep
the source ID/name/type/page/path, code, severity, outcome, reason, action, and suggested
manual action. Restored/expanded shapes can outnumber detected source elements; event and
shape counts must not be described as the same quantity.

`Outcome` is `Converted | ConvertedWithModification | Skipped | Unsupported | Failed`.
The worst outcome per source element determines its count:

```text
converted + approximated + skipped + unsupported + failed == detected
```

`ValidationState` is `Passed | Failed | NotRun`; file status is
`Completed | CompletedWithWarnings | Failed | Cancelled`. They are separate claims.
The report schema remains version 1; adding issue codes does not change its structure.
Thirty converter codes have designer-facing copy, with a retained fallback for unknown
codes. JSON, self-contained offline HTML, and CSV exports are implemented for files/batches.
Single-file CLI conversion returns failure when the report contains failed elements,
even if a structurally valid partial archive was saved; saved output is labeled as saved.

## 4. Queue and desktop

`conversion-queue` uses a bounded pool of scoped standard-library threads, a mutex-protected
job deque, and `std::sync::mpsc` events. Default concurrency is **2**, explicitly adjustable
with a minimum of 1. There is no Tokio, Rayon, semaphore scheduler, or memory-based worker
formula in the current implementation. Each job is protected by `catch_unwind`; one job's
failure does not stop the rest. Results retain job identity and are returned in ID order.

Progress stages are `Reading`, `Converting`, `Writing`, `Validating`, and `Saving`.
Pause holds new batch jobs; cancel is checked before/after reading and before saving.
Active conversion/serialization is not interrupted inside every node or ZIP-entry loop.
The single-file desktop worker has separate cooperative stage controls. Neither path
promises cancellation during an atomic write. Retry/remove actions are desktop state
operations around the queue; full native interaction QA remains pending.

Known gap: `JobSpec.keep_token_bindings` is stored but the queue still constructs
`ConvertOptions::default()`. The single-file CLI/desktop path honors the preference;
batch propagation is Partial and needs a regression test/fix.

The desktop has eight screens, centralized copy, theme/component modules, persisted
settings, native `rfd` dialogs, and background conversion/analysis. Headless tests use
`egui::Context` and ordinary Rust tests. The current eframe features do not enable
AccessKit; screen-reader behavior is unverified. Screenshots record historical macOS
evidence; new mapping behavior has not received exhaustive native-platform visual QA.

## 5. Resource limits and output safety

| Boundary | Default / implementation |
|---|---|
| Input container | `fig-reader::Limits`: input 1 GiB, 10,000 ZIP entries, 4 GiB total expanded archive bytes, ratio 1,000 |
| Kiwi message | 512 MiB decompressed message, 1,000,000 nodes, depth 64, bounded strings/arrays/blobs/schema |
| Zstd | Maximum window 128 MiB |
| Conversion | `ConvertOptions.max_expanded_shapes`: 250,000; hierarchy depth 128 |
| Target archive validation | `ValidationLimits`: 300,000 entries, 64 MiB per entry, 512 MiB total uncompressed bytes |

Input and target archive ceilings are different. Custom Rust API limits are available;
the desktop does not expose every parser/validator ceiling as a setting. Archive validation
checks canonical paths, references, IDs, finite geometry, target blends, token identifiers,
layout numbers, and supported schema structure. Validation is not a substitute for a real
Penpot import or visual comparison. Files remain read-only at source; failures are surfaced
through reports/results, and structural validation failures prevent final archive writes.
Temporary files use the destination volume. Disk-space forecasting, streaming, stale-temp
startup sweeps, and every cancellation race are not verified product capabilities.

## 6. Dependencies and verification

The authoritative declarations are the crate manifests and `Cargo.lock`; the resolved
Linux license metadata inventory is in `../THIRD_PARTY_LICENSES.md`. Current GUI declarations
are `eframe 0.36.2`, `rfd 0.16.1` (GTK3 on Linux), and `dirs 7.0.0`. Compression uses the
Rust flate2 backend and `ruzstd`; Kiwi decoding and CSV/HTML rendering are implemented in
the project. There is no runtime dependency on openfig-core or the reference converter.
The workspace declares edition 2021 and MSRV 1.95; verification used stable Rust 1.99.1.

On 2026-11-02, **116 workspace tests**, formatting, and Clippy passed in a Linux container
on Windows. All five private outputs imported into local Penpot 2.18.5-RC1; overall fidelity
remains Partial. Baseline counts/goldens and dependencies were not changed. Historical
parser fuzz results do not verify the new component mappings. License/advisory tools were
unavailable; native dialogs, accessibility, complete visual comparison, component editing,
and reopening remain pending. Exact evidence and reproduction steps are in `MANUAL_QA.md`
and `DEV_ENVIRONMENT.md`; private inputs/outputs are not committed fixtures.
