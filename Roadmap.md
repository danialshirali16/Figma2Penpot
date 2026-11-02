# Roadmap

Reviewed **2026-11-02**. The source compatibility work is recorded in ADR-0011. All five
private files import into Penpot 2.18.5-RC1, but overall fidelity remains **Partial**. The
116-test Linux-container result does not replace native platform or exhaustive visual QA.
Compatibility fixes are not included in the published `v0.1.1` artifacts.

Sequencing for Figma 2 Penpot, from research to a signed-ish, installable v1.0.

**Milestones are gates, not dates.** A milestone is done only when every acceptance item
is demonstrably true and the evidence (command + real output) is recorded in
[`PROGRESS.md`](PROGRESS.md). Dates are intentionally absent; inventing them would violate the
project's honesty rules. Progress is tracked by evidence, not by percentage complete.

Status vocabulary is fixed and defined in [`README.md`](README.md#support-status-vocabulary):
**Verified**, **Implemented (unverified)**, **Partial**, **Planned**, **Not supported**.

---

## 1. Sequencing principles

1. **Vertical slices first.** A trivial `.fig` → valid `.penpot` → one correct report entry
   is more valuable than three half-built subsystems.
2. **No silent gaps.** Any mapping pass that cannot represent something must emit a
   diagnostic. A pass without diagnostics is incomplete, not finished.
3. **Later features wait.** Do not start M4 (GUI) while M2's validator or report counts are
   wrong. Bounded research spikes are allowed and must be labeled as spikes.
4. **Cross-platform claims need CI.** Anything that cannot be run on the current machine is
   marked **unverified on this platform** until a CI job proves it.
5. **Licensing precedes code.** No porting from MPL-2.0 or unlicensed sources until
   `docs/LICENSE_ASSESSMENT.md` is signed off.

## 2. Milestone overview

| ID | Milestone | Ends when | Status |
|---|---|---|---|
| M0 | Research and legal baseline | Format, architecture, and license documents exist with citations and explicit unknowns | **Verified** — see `PROGRESS.md` |
| M1 | Workspace and `.fig` reading | `f2p-cli inspect` prints verified metadata for ≥3 fixtures; malformed inputs error without panicking | **Verified** — see `PROGRESS.md` |
| M2 | Minimal end-to-end conversion | Simple fixture converts to a validator-passing `.penpot` with correct report counts | **Verified** — verified in Penpot |
| M3 | Fidelity expansion | Each feature ships with tests, golden output, diagnostics, and a support-matrix row | **Partial** — all eight waves implemented; fidelity gaps remain |
| M4 | Desktop application | Eight screens working headlessly-testable, non-blocking queue, light/dark | **Partial** — eight-screen designer UI, worker and headless tests; native interaction QA pending |
| M5 | Batch conversion | Mixed batch test shows isolation, bounded memory, clean cancellation | **Partial** — bounded queue, CLI batch, desktop retry/remove; mid-write cancellation and token preference propagation pending |
| M6 | Reports UI and exports | Filterable report UI; JSON/HTML/CSV exporters snapshot-tested | **Partial** — JSON/HTML/CSV exports, grouping/filters and designer copy; full interaction QA pending |
| M7 | Reliability and performance | Fuzz campaign, large-file corpus, recorded time and peak-memory benchmarks | **Partial** — historical fuzz/benchmarks, unchanged fixture baseline; new component fuzz/scale coverage pending |
| M8 | Release engineering | Windows and macOS artifacts built in CI, docs and notices complete | **Partial** — v0.1.1 app/DMG/zip/tarball artifacts; signing/MSI and native install QA pending |

---

## M0 — Research and legal baseline

**Status: Verified (2026-11-01 evidence).** Evidence is recorded in `PROGRESS.md`. The deliverables are
`docs/ARCHITECTURE.md`, `docs/FIG_FORMAT.md`, `docs/PENPOT_FORMAT.md`,
`docs/LICENSE_ASSESSMENT.md`, `docs/RISKS.md`, `docs/DEV_ENVIRONMENT.md`,
`docs/RESEARCH_SOURCES.md`, and ADRs 0001–0008. Real Penpot import was pending at M0;
it is now verified for M2–M3.8 in `PROGRESS.md`.

**Goal.** Replace assumptions with cited facts, and settle what may legally be built upon.

**Work**

- Study `girafic/penpot-converter`: pipeline, supported structures, limits, reporting gaps.
- Study the official `penpot-exporter-figma-plugin` (MPL-2.0) and `@penpot/library`.
- Review the Penpot community migration thread and Penpot's migration documentation.
- Write `docs/ARCHITECTURE.md`, `docs/FIG_FORMAT.md`, `docs/PENPOT_FORMAT.md`,
  `docs/LICENSE_ASSESSMENT.md`, `docs/DEV_ENVIRONMENT.md`.
- Produce a technical risk register and a Rust implementation strategy.
- Decide: fork/port (with obligations) vs clean-room reimplementation. Record in
  `docs/DECISIONS/`.

**Acceptance**

- Every document distinguishes **confirmed** (with URL, file, or observed bytes),
  **inferred**, and **unknown**.
- License assessment names the license of every upstream project and dependency
  considered, and states what may be copied, ported, or only referenced.
- Fork-vs-clean-room decision is recorded with consequences.

**Risk if skipped.** A format assumption bakes into the IR, or a copied file drags the
whole project under an incompatible license.

---

## M1 — Workspace and `.fig` reading

**Status: Verified (2026-11-01 evidence).** The workspace builds; the reader parses four fixtures
(550/627-definition schemas, ZIP and raw forms); 29 unit/integration tests and a doctest
pass; hostile inputs return typed errors; the fuzz target ran 829,577 executions without a
crash; CI is green on Linux, macOS, and Windows. Evidence is in `PROGRESS.md`. Conversion
was out of scope for M1 and is now implemented through M3.

**Goal.** Open real files safely and print only what is actually known.

**Work**

- Cargo workspace with the crate boundaries from `docs/ARCHITECTURE.md`; stable toolchain,
  pinned dependencies.
- CI: `fmt`, `clippy -D warnings`, `test` on Linux, macOS, and Windows.
- `fig-reader`: magic-byte container detection, safe archive extraction (zip-slip,
  entry count, total uncompressed size, compression ratio, nesting depth), KIWI schema +
  data decoding, raw node tree.
- Resource limits as documented constants with defaults.
- `f2p-cli inspect file.fig` printing pages, node counts, and `Unknown` where the parser
  cannot determine a value.
- Fixtures: at least three genuinely distinct files (version/compression/structure
  variants), plus malformed, truncated, and decompression-bomb samples.
- `cargo-fuzz` target for the decode path.

**Acceptance**

- Workspace builds and tests clean on stable.
- `inspect` works on all three good fixtures; outputs are checked by golden tests.
- Every hostile sample returns a typed error; no panic, no unbounded allocation.
- A fuzz run of documented duration completes with zero panics (crash artifacts kept as
  regression fixtures).
- Fixture provenance and redistribution rights recorded.

**Explicitly out of scope:** writing `.penpot`, any UI.

---

## M2 — Minimal end-to-end conversion

**Status: Verified (2026-11-01); real import verified.**
`circle.fig` and the synthetic `text-group.fig` convert through the CLI, pass the structural
validator, and produce event-based JSON reports. Both archives imported into a local Penpot
2.18.5 stack (`2.18.5-RC1`); frames, groups, rect/ellipse geometry, fills, strokes, corner
radius, and text were inspected. A golden report and determinism test are checked in. Reopen
and move/edit persistence were not tested and are marked untested in `docs/MANUAL_QA.md`.

**Goal.** The first honest vertical slice.

**Work**

- Internal Figma IR and ordered mapping passes: normalize transforms → resolve styles →
  map geometry → serialize.
- Convert: pages, frames, groups, rectangles, ellipses, solid fills and strokes, opacity,
  basic single-style text.
- `DiagnosticSink` wired through every pass; no silent branches.
- `penpot-writer` producing a minimal but structurally valid archive.
- Structural validator derived from observed Penpot files.
- JSON report (versioned schema) with counts derived from a real traversal.
- `f2p-cli convert input.fig -o out.penpot --report report.json`.
- Golden snapshot test of both the archive structure and the report.

**Acceptance**

- Simple fixture converts; output passes the validator.
- Report counts equal an independently computed manual traversal of the fixture.
- Golden test fails loudly on any structural drift.
- Real Penpot import attempted. If unavailable, recorded as **pending** with exact manual
  steps and expected observations.

---

## M3 — Fidelity expansion

**Status: Partial (reviewed 2026-11-02).** All eight waves
are implemented. M3.1–M3.3 and M3.5–M3.8 are import-verified in Penpot 2.18.5; M3.4 metrics
are unit-tested and carried through a verified typography style. Partial: stroke-only
vectors, editable network handles, boolean operands, mask mode, Auto Layout child sizing,
fixed constraints, per-paint blend modes, effect-style linkage, instance swaps/properties,
mixed text runs, and variables bound outside painting. Grid layout and grid styles are
reported as unsupported rather than silently dropped.

Ordered by user-visible value, not by implementation convenience. Each feature is its own
slice: unit tests → golden test → expected diagnostics for unsupported variants → golden
report → `docs/SUPPORT_MATRIX.md` row → evidence in `PROGRESS.md`.

| Wave | Feature area | Target | Typical fallback / diagnostic |
|---|---|---|---|
| 3.1 | Vectors, paths, boolean ops | Penpot paths and booleans where representable | Keep path geometry; report lost boolean parameters |
| 3.2 | Gradients, effects, masks | Penpot gradients, shadows, blur, mask groups | Approximate stop positions; report unsupported blend modes |
| 3.3 | Images | Embedded media with dedup by content hash | Skip missing media with an error event, never a broken reference |
| 3.4 | Text fidelity | Mixed styles, line height, letter spacing, missing fonts | Preserve text and style runs; report substituted/missing fonts |
| 3.5 | Auto Layout and constraints | Penpot flex/grid + constraints | Fixed positioning with reported loss of reflow |
| 3.6 | Styles | Paint, text, effect styles mapped to Penpot library styles | Inline values with reported loss of style linkage |
| 3.7 | Components, instances, variants | Penpot components, copies, variant sets | Detach with report when overrides cannot resolve |
| 3.8 | Variables, modes, tokens | Penpot token sets/themes | Resolve to static values, report lost dynamic binding |

**Acceptance per wave:** all tests pass, golden output reviewed, every unsupported case has
a test asserting its issue code, and the support matrix reflects reality. A wave may ship
as **Partial**; it may not ship as **Verified** without a Penpot import.

**Dependency note:** the Penpot component and token models were implemented in 3.7 and 3.8.
Remaining gaps are listed as Partial in `docs/SUPPORT_MATRIX.md`; they are not silently
treated as complete.

---

## M4 — Desktop application

**Status: Partial (reviewed 2026-11-02; M4.2 designer UI implemented).** The native shell, eight screens, headless state
tests, native dialogs, settings persistence, and background conversion worker exist.
The designer UI has light/dark screenshots at two sizes and plain-language reports.
Drag/drop, dialogs, keyboard-only, and accessibility still need complete platform QA;
packaging exists as a separate Partial release-engineering milestone.

**Goal.** A finished-feeling native utility, not a web dashboard in a window.

**Work**

- `egui`/`eframe` app, app ID `app.figma2penpot.desktop`.
- Screens: Home/Import · File Queue · File Analysis · Conversion Settings · Progress ·
  Results · Report · Preferences.
- Drag and drop, native pickers, folder scan, middle-truncated paths with full-path
  tooltips, HiDPI, responsive layout.
- Light and dark themes with one token source.
- Non-blocking conversion through the queue; UI never polls unsynchronized state.
- Accessible status: icon + text + color, never color alone.
- All user strings in one module.
- Headless UI-state tests; `docs/MANUAL_QA.md` checklist per platform.

**Acceptance**

- Builds on all three targets in CI (behavior unverified where not runnable).
- UI logic tests pass headlessly.
- Manual checklist exists with each item marked pass/fail/untested per platform.
- Keyboard shortcuts per platform documented and tested manually.

---

## M5 — Batch conversion

**Status: Partial (reviewed 2026-11-02).** The headless queue, CLI `batch`, and desktop
“Convert all” path exist with bounded workers, stage events, pause/cancel, and per-job
isolation. Retry/remove controls and historical memory benchmarks are implemented.
Mid-write cancellation, expanded-component peak RSS, and batch token-preference propagation
remain pending. Default concurrency is 2; automatic CPU/memory tuning is Planned.

**Goal.** Many files, one bad apple.

**Work**

- Job state machine: `Queued → Analyzing → Converting → Validating → Completed |
  Completed with Warnings | Failed | Cancelled`.
- Bounded worker pool; concurrency derived from CPU and memory, user-overridable.
- Pause scheduling, cancel pending and active, retry one, retry failed only, remove
  completed, batch summary.
- Per-job cancellation tokens; panic containment via `catch_unwind` or isolation.
- Temp files on the destination volume; startup cleanup of stale temp files.
- Streaming assets; memory must not scale linearly with batch size.

**Acceptance**

- Mixed batch (valid, corrupt, unsupported, huge) produces correct per-file statuses with
  no cross-file contamination.
- Cancellation mid-write leaves no file that looks valid; temp files are removed.
- Peak memory measured for 1, 4, and 16 files and recorded; curve is bounded.
- A deliberate panic in one job marks only that job failed.

---

## M6 — Report UI and exports

**Status: Partial (2026-11-02).** Self-contained HTML and CSV exporters, CLI `report`,
batch aggregate exports, and desktop export buttons exist. The report screen has search,
severity/outcome filters, grouping by what to do/page/none, and 30 designer issue-code
mappings. Component grouping, jump-to-element, and full platform interaction QA remain.

**Goal.** The differentiator, finished.

**Work**

- Report screen: summary metrics, severity/category filters, search, grouping by page and
  by component/layer, expandable details, copy issue, jump-to-element where possible.
- JSON (versioned), standalone HTML (no external assets), and CSV exporters.
- Per-file reports and a batch aggregate.
- `docs/ISSUE_CODES.md` complete; every documented unsupported feature has a test proving
  its diagnostic.

**Acceptance**

- Exporter snapshot tests for all three formats.
- HTML report renders correctly with no network access (checked with the browser opened
  offline).
- Grouped issues still expose per-element references.
- Invalid or missing validation state appears explicitly as *not validated*.

---

## M7 — Reliability and performance

**Status: Partial (2026-11-02).** A 60-second fuzz campaign (818,204 executions, no
artifacts), per-fixture release benchmarks, batch memory measurements, one 4,629-node real
file, and a checked-in conversion baseline are recorded. A large corpus and remaining
error-recovery drills are pending.

**Work**

- Large-file corpus (structure, not just byte size); profiling; allocation limits.
- Extended fuzzing of container, schema, and message decoding.
- Error-recovery review: low disk, write failure, cancellation race, memory pressure.
- Benchmark harness with pinned fixtures and recorded machine specs.

**Acceptance**

- Documented fuzz duration with zero panics; crashes, if any, fixed and re-run.
- Benchmarks record wall time and peak RSS per fixture on a named machine.
- Regressions tracked against a checked-in baseline, with a documented tolerance.
- No known reachable panic on untrusted input.

---

## M8 — Release engineering

**Status: Partial (2026-11-02).** `v0.1.1` was published with macOS `.app`/DMG/CLI, Windows zip,
and Linux tarball artifacts plus SHA-256 files. The macOS bundle launched locally and a
DMG was built. Windows/Linux install-and-launch, the Windows MSI installer, and
signing/notarization remain pending. This is release evidence, not proof that the published
binaries include subsequent source fixes.

**Work**

- Windows installer (MSI or EXE) and macOS `.app` + `.dmg`, built in GitHub Actions.
- Versioned artifacts, checksums, reproducible build instructions.
- Signing and notarization plan (unsigned dev builds acceptable until certificates exist).
- README, user guide, privacy statement, contributor guide, third-party notices.
- `cargo-deny` / `cargo-audit` clean or exceptions documented.

**Acceptance**

- CI artifacts exist for both platforms and are downloadable from a tagged release.
- Install-and-launch verified manually on at least one platform per OS, or explicitly
  marked **untested**.
- License compliance document complete, including all bundled dependencies.

---

## 3. Release framing

No dates. Releases are cut when their gate is green.

| Version | Gate | Promise |
|---|---|---|
| `0.1.0` / `0.1.1` | Pre-alpha checkpoints, published | Desktop and CLI artifacts exist; full v1.0 acceptance remains open. |
| `0.3.0` | M3 (partial) | Useful fidelity on common files; support matrix documents the rest. |
| `0.5.0` | M4 + M5 | The desktop app people can actually use, including batches. |
| `0.8.0` | M6 + M7 | Reports complete, reliability measured. |
| `1.0.0` | Release DoD | Installable on Windows and macOS, verified imports, complete docs. |

The v1.0 definition of done is the one in the project brief: installs without dev tools,
fully native Rust UI, inspectable files, valid conversions, resilient batches, accurate
event-based reports, working exports, no JavaScript anywhere, representative files imported
and checked in a real Penpot instance, and complete tests, docs, and license compliance.

## 4. What could move this roadmap

| Risk | Impact | Mitigation |
|---|---|---|
| `.fig` schema drift across Figma versions | Reader breaks on new files | Version-tolerant decoding, unknown-field diagnostics, fixture corpus per version |
| Penpot import strictness changes between releases | Valid-looking output rejected | Pin target versions; structural validator; optional real-import checks in CI |
| Penpot model gaps (tokens, variants, layout) | Fidelity cap on M3 waves | Documented fallbacks; report events; no pretending parity |
| No license on the reference project | Cannot reuse its fixes | Reimplement from behavior and format facts without code reuse (ADR-0002); not a formal clean-room |
| GPU/egui rendering differences across platforms | UI defects only visible on real hardware | CI smoke builds; manual QA matrix; screenshot tests where possible |
| Very large files | Memory or time blowups | Streaming, page splitting, bounded concurrency, measured benchmarks |

## 5. Maintaining this roadmap

- Change a milestone's scope only by editing this file and recording why in
  `docs/DECISIONS/`.
- Never mark a milestone done here; mark evidence in `PROGRESS.md` and update the status
  column here to point at that evidence.
- If reality and this roadmap disagree, reality wins. Record the discrepancy, then fix the
  roadmap.
