# PROGRESS

Living status file for Figma 2 Penpot. Canonical place to resume from.

Last updated: **2026-11-02** (session 17, documentation synchronization and GitHub integration).

---

## 1. Where we are

**Milestone: all M3 waves are implemented and the M4/M6 desktop UI has been redesigned for
product designers. M3.1–M3.3 and M3.5–M3.8 are import-verified; M3.4 metrics are
unit-tested and carried through a verified typography style. Several sub-features remain
Partial and are reported at runtime.**

The workspace reads real `.fig` files, converts the M2 subset plus vectors, masks, boolean
fallbacks, gradients, shadows, blur, images, text metrics, flex containers, styles,
components/variants, and tokens to `.penpot`, validates archives structurally, and writes
event-based JSON reports. Every M3 wave imported into a local **Penpot 2.18.5** stack
(`2.18.5-RC1`) with `error?=false`. Remaining Partial items: stroke-only vectors, editable
network handles, fixed constraints, Auto Layout child sizing, per-paint blend modes, effect
style linkage, instance swaps/properties, mixed text runs, and variable bindings outside
painting.

Session 16 compatibility result: all five privately supplied files converted with zero
failed elements and were accepted by local Penpot **2.18.5-RC1**. Import compatibility is
**Verified**; overall fidelity remains **Partial**, with warnings and unsupported outcomes
preserved in the reports. Source hashes were recorded and checked for delivery. Private
inputs/outputs are not fixtures and are not committed. The Rust workspace passes **116 tests**,
formatting, and Clippy. `cargo-deny`/`cargo-audit` remain unavailable. See ADR-0011 and the
latest session log; older acceptance tables retain historical evidence.

Current documentation map: `docs/DOCUMENTATION_INDEX.md`. Source compatibility fixes
postdate published `v0.1.1` artifacts; no replacement release has been published. Batch
token-preference propagation remains a known source gap (`run_job` uses default options).

M3.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| Decode `commandsBlob` geometry | Done | `crates/fig-reader` + `decode_path_commands` |
| Emit Penpot path command arrays | Done | `content` is `move-to`/`line-to`/`curve-to`/`close-path` |
| Absolute path coordinates | Done | commands transformed by accumulated matrix |
| Multiple subpaths per Figma vector | Done | OpenFigs icon: 20 paths, all linked to the frame |
| Filled vector import | Verified | Penpot import `d047d42d-…-e1452fac5611`, icon rendered |
| Stroke-only vector | Partial | expanded outline converted to filled path; report says approximated |
| Vector network handles | Partial / reported | `F2P-VECTOR-NETWORK-LOST` |
| Booleans | Partial | rendered geometry imported; operands not reconstructed |
| Golden test | Done | `filled_vector_converts_to_multiple_paths`; 20 paths; `openfigs.report.json` |

M3.2–M3.5 acceptance:

| Requirement | Status | Evidence |
|---|---|---|
| Gradients (linear/radial) | Done | normalized start/end + stops; linear import verified |
| Shadows (drop/inner) | Done | Penpot `shadow` array; drop import verified |
| Layer and background blur | Done | Penpot `blur`/`backgroundBlur`; both import verified |
| Masks/clipping | Partial | masked group imported; mode reported as approximated |
| Images + hash dedup | Done | media/object archive; red 8×8 PNG rendered |
| Text alignment | Done (unverified) | horizontal/vertical strings; unit test only |
| Line height / letter spacing | Done (unverified) | percent/pixels/raw → ratio/px; unit test only |
| Auto Layout → flex | Partial | direction/gap/padding/alignment verified; child sizing reported |
| Grid layout | Not supported / reported | `F2P-LAYOUT-GRID-UNSUPPORTED` |

M3.6–M3.8 acceptance:

| Requirement | Status | Evidence |
|---|---|---|
| Paint styles | Done | color library entry imported and linked from a fill |
| Text styles | Done | typography entry imported and linked from text |
| Effect styles | Partial | effects inlined with `F2P-STYLE-EFFECT-INLINED` |
| Components | Done | main `Button` registered; Assets shows `Components 3` |
| Instances + overrides | Partial | subtree materialized; text override rendered; swaps/properties not mapped |
| Variants | Done | two variants registered in one set and collapsed in Assets |
| Variables + modes | Partial | COLOR/FLOAT/etc. → token sets; modes → themes; no layout bindings |
| Design tokens | Done | `tokens.json` imported; `brand.primary` applied to a fill |
| Masks | Partial | masked group imported; mode reported approximated |
| Booleans | Partial | rendered path imported; operands not reconstructed |

M4.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| eframe app and app id | Done | `crates/desktop-app`, `app.figma2penpot.desktop` |
| Eight screens | Done | Home, Queue, Analysis, Settings, Progress, Results, Report, Preferences |
| Non-blocking conversion | Done | worker thread + `mpsc`; UI drains events only |
| Native file/folder dialogs | Implemented (unverified) | `rfd`; dialog interaction not automated |
| Drag and drop | Implemented (unverified) | egui dropped/hovered files; manual check pending |
| Folder scan | Done (unit) | recursive `.fig` scan, hidden folders skipped, 512-file cap |
| Keyboard shortcuts | Done (headless) | Cmd/Ctrl+O, Shift+O, Shift+A, Enter, comma, 1–8; shortcut key event tested |
| Light/dark/system theme | Done | Settings + `egui::ThemePreference`; dark and light captured |
| Settings persistence | Done (unit) | JSON round-trip and malformed-file fallback tests |
| Headless UI tests | Done | all eight screens drawn through `egui::Context`; 22 desktop tests |
| End-to-end desktop worker test | Done | fixture converts, result lands in Results/Report screens |
| Manual desktop QA | Partial | All eight screens captured in light/dark at two sizes; native dialogs, drag-drop, keyboard-only, and screen reader still manual |
| Installers | Planned | M8 |

M5.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| Bounded worker pool | Done | `QueueConfig::bounded`, default 2 workers |
| Per-job isolation | Done | corrupt fixture fails without affecting valid jobs |
| Panic containment | Done (unit) | `protect` catches a deliberate panic |
| Real stage events | Done | `JobStarted`, `Stage`, `JobFinished`, `BatchFinished` |
| Cancellation checkpoints | Partial | pause/cancel before and between stages; mid-write cancellation pending |
| CLI batch | Done | 2 valid + 1 corrupt: 2 completed, 1 failed; outputs/reports written |
| Desktop Convert all | Done (headless) | 2-file batch test reaches Results with both converted |
| Retry failed / retry selected | Done (state) | Desktop actions; no automated retry test yet |
| Remove completed | Done (unit) | completed items removed; failed items retained |
| Memory benchmarks | Done | 1/4/16 files, 2 workers: 5.2/6.8/7.0 MB peak RSS |

M6.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| Self-contained HTML exporter | Done | Inline CSS, no external assets; rendered in a browser |
| CSV exporter | Done | RFC 4180 escaping unit-tested |
| Batch aggregate HTML/CSV/JSON | Done | `batch --html/--csv/--json`; totals + per-file detail |
| CLI `report` | Done | JSON report → HTML/CSV smoke-tested |
| Desktop export buttons | Done (unverified UI) | Results/Report actions; native save dialog not automated |
| Report screen grouping/copy | Done (headless) | Search + severity/outcome + code/page grouping; copy issue/node ID |
| Exporter snapshot tests | Done | 4 exporter tests with escaping, batch HTML/CSV, and batch JSON |

M4.2 / M6.2 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| Design contract before code | Done | `docs/UI_SPEC.md` + `docs/ui/WIREFRAMES.md`, committed as `914daee` |
| Tokenized theme, bundled font | Done | `theme.rs`, Inter OFL 1.1 under `desktop-app/assets/fonts/`, recorded in `THIRD_PARTY_LICENSES.md` |
| Reusable components + gallery | Done | Buttons, chips, banners, file rows, search, segmented control, empty states; `F2P_GALLERY` |
| Designer copy + tested mappings | Done | `copy.rs`; 28 issue codes and every status/outcome/stage/queue state/validation state |
| Honest report grouping | Done | Default Needs your attention / Not converted / Slightly different / All good; raw details collapsed |
| Real fidelity choice | Done | `keep_token_bindings` through core, queue, desktop, CLI `--inline-tokens`; integration test |
| Single-file + batch pause/cancel | Partial | Cooperative checkpoints between stages; cancellation before saving leaves no output; mid-write not interruptible |
| Error copy | Done | Friendly first message; raw worker text under Show details; tested mapping |
| Light/dark + large/compact evidence | Done | 32 core screenshots + gallery/empty/error/license/clear extras in `docs/ui/screenshots/` |
| Heuristic / 5-second review | Done | `docs/ui/REVIEW.md` (internal review, no external participants) |
| Native dialogs / drag-drop / keyboard-only | Unverified | Manual OS checks listed in `docs/MANUAL_QA.md` |
| Windows/Linux visuals, screen reader | Unverified | Not run; AccessKit is not enabled in the current eframe features |
| `cargo deny` / `cargo audit` | Pending | Tools absent in this environment |

M7.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| Fuzz campaign | Done | 818,204 executions / 61 s; 49 MB peak RSS; no artifacts |
| Fixture benchmarks | Done | 10 fixtures, 0.01–0.03 s, 5.3–6.2 MB peak RSS |
| Large-file benchmark | Done | 3,303-node synthetic: 0.23 s / 66.7 MB; local 4,629-node: 1.09 s / 226 MB |
| Batch memory | Done | 1/4/16 files at 2 workers: 5.2/6.8/7.0 MB peak RSS |
| Regression baseline | Done | 11 fixtures in `tests/benchmarks/baseline.json`; counts/status/validation checked |
| Error recovery drills | Partial | atomic write failure + temp cleanup, overwrite refusal, batch isolation; low-disk/cancel-mid-write pending |

M8.1 acceptance so far:

| Requirement | Status | Evidence |
|---|---|---|
| macOS `.app` bundle | Done | `scripts/package-macos.sh`; launched via `open`; window captured |
| macOS zip + checksum | Done | 3.8 MB `figma2penpot-0.1.1-macos-arm64.zip` + SHA-256 |
| macOS DMG | Done locally | 4.3 MB compressed DMG + SHA-256 |
| Windows zip script | Built in CI | `figma2penpot-0.1.1-windows-x86_64.zip` (5.0 MB); local install untested |
| Linux tarball script | Built in CI | `figma2penpot-0.1.1-linux-x86_64.tar.gz` (6.8 MB); local install untested |
| Tag-driven release workflow | Done | tags `v0.1.0`/`v0.1.1`; run 41352464597; `v0.1.1` has 10 assets including DMG |
| Signing/notarization | Planned | ad-hoc/linker-signed macOS build only |
| MSI/DMG installers | Partial | DMG published; MSI not built |
| `cargo deny` / `cargo audit` | Pending | tools absent in development environment |

M2 acceptance remains as recorded below.

### 1.1 M2 acceptance (retained)

| Requirement | Status | Evidence |
|---|---|---|
| Pages, frames, groups, rectangles, ellipses | Done | Imported and inspected; layer tree and inspector |
| Solid fills/strokes, opacity, radii | Done (opacity partially) | Red/green fills, 2 px stroke, radius 4; fill alpha 0.5 |
| Basic text | Done | Inter 16/400, `Hello Penpot`, correct position |
| Diagnostic sink + first report events | Done | `conversion-report`; tests; JSON reports |
| JSON report | Done | `circle.report.json`, golden comparison |
| Structural validator | Done | All generated archives pass; real imports accepted |
| `f2p-cli convert` | Done | Used for both imports |
| Golden snapshot | Done | `crates/conversion-core/tests/golden/circle.report.json` |
| Report counts match manual traversal | Done | circle: 2 detected/2 converted; text-group: 6/6 |
| Real Penpot import attempted | Done | Penpot 2.18.5-RC1; screenshots + logs |
| Reopen/move/edit persistence | Untested | Not claimed |

## 2. What exists

```text
Figma 2 Penpot/
├── Cargo.toml / Cargo.lock
├── crates/
│   ├── fig-reader/          container, Kiwi decode, geometry/paints/text fields
│   ├── conversion-core/     tree → Penpot shapes, diagnostics, absolute coordinates
│   ├── conversion-report/   ReportEvent, DiagnosticSink, counts, JSON
│   ├── penpot-writer/       document model, v3 archive, structural validator, atomic write
│   ├── desktop-app/         eframe shell, theme, components, copy, screens
│   ├── conversion-queue/    bounded batch workers, pause/cancel, stage events
│   └── f2p-cli/             inspect, convert, batch, report
├── fuzz/                    libFuzzer harness `read_fig`
├── tests/fixtures/          5 fixtures (4 upstream MIT + 1 synthetic) + provenance
├── docs/qa/                 real-import screenshots (Penpot 2.18.5-RC1)
├── .github/workflows/       CI (Linux/macOS/Windows)
└── docs/                    platform docs, UI spec/wireframes/review, ADRs 0001–0010
```

## 3. Environment

See `docs/DEV_ENVIRONMENT.md`.

The current session is on Windows 11 Pro with PowerShell 7.6.5. Rust/Cargo are not
available on the host `PATH`. Docker-hosted stable Rust 1.99.1 provided full workspace
verification: 116 tests, formatting and Clippy passed; five real Penpot imports passed.
See `docs/DEV_ENVIRONMENT.md` §§9–10. The following facts describe earlier macOS sessions.

- macOS 27.2, arm64, 8 GiB RAM.
- Rust 1.99.1, no rustup; `cargo-fuzz 0.13.2`.
- Docker Desktop daemon 29.9.2; Penpot 2.18.5 stack was started, used, and stopped.
- Telemetry was disabled for the local Penpot stack.
- crates.io access became intermittent; M2 built with `--offline` after dependencies were
  cached. Do not assume network in CI/local builds.

## 4. Evidence log (session 4)

| Command / check | Result |
|---|---|
| `cargo test --offline --workspace` | **98 tests pass** (71 core/CLI + 4 queue + 22 desktop + 1 bench) |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | Success |
| `cargo fmt --all -- --check` | Success |
| `cargo run -p f2p-cli -- convert tests/fixtures/OpenFigs.fig` | 2 detected, 1 converted, 1 approximated (vector network loss), validation passed |
| `cargo run -p f2p-cli -- convert tests/fixtures/vector-network/straight-2-point-line.fig` | 1 detected, 0 converted, 1 approximated (stroke outline), validation passed |
| Penpot import OpenFigs | `d047d42d-cd5b-80cd-8008-e1452fac5611`, `error?=false`, 0.158 s |
| Penpot import straight line | `d047d42d-cd5b-80cd-8008-e145acec5587`, `error?=false`, 0.052 s |
| UI inspection | 20 path layers under the OpenFigs frame; icon rendered; green stroke-only line rendered |
| `convert effects-gradient.fig` | 2 detected, 2 converted; validation passed (14 checks) |
| `convert image.fig` | 2 detected, 2 converted; validation passed (20 checks) |
| `convert auto-layout.fig` | 3 detected, 2 converted, 1 approximated; validation passed (15 checks) |
| Penpot import effects | `17b9706f-02f0-8188-8008-e14cce8b07fa`, `error?=false` |
| Penpot import image (white) | `17b9706f-02f0-8188-8008-e1519513b5a7`, `error?=false` |
| Penpot import image (red) | `17b9706f-02f0-8188-8008-e153dadf0c7c`, `error?=false` |
| Penpot import auto layout | `17b9706f-02f0-8188-8008-e1588bbf28a8`, `error?=false` |
| UI inspection (M3.2–M3.5) | gradient/shadow/blur rendered; red image rendered; flex row gap 12, padding 8/20/12/10 |
| `convert styles.fig` | 3 detected/3 converted; validation passed (19 checks) |
| `convert components.fig` | 9 detected/9 converted; validation passed (23 checks) |
| `convert tokens.fig` | 2 detected/2 converted; validation passed (16 checks) |
| `convert masks-boolean.fig` | 5 detected/3 converted/2 approximated; validation passed (18 checks) |
| Penpot import styles | `17b9706f-02f0-8188-8008-e15ca6d94f79`, `error?=false` |
| Penpot import components | `17b9706f-02f0-8188-8008-e15e6267215d`, `error?=false` |
| Penpot import tokens | `17b9706f-02f0-8188-8008-e16014ec506a`, `error?=false` |
| Penpot import masks/boolean | `17b9706f-02f0-8188-8008-e16179c815cf`, `error?=false` |
| UI inspection (M3.6–M3.8) | color/typography libraries and links; 3 components + 2 variants; instance override; Light/Dark tokens and applied `brand.primary`; masked lens + boolean donut |
| Real-world stress check | 4,629-node file with 539 symbols, 791 instances, and 22 variables: conversion completed, validator passed (6,698 checks); status Failed only because 1 node failed and 33 were unsupported, all reported |
| Desktop shell build | `cargo build -p desktop-app` succeeds; window 1180×792 opens |
| Desktop headless tests | 22 state/worker/UI tests, including all screens, shortcuts, cancellation, and a real fixture conversion |
| Workspace gates | `cargo fmt --check` + `cargo clippy -D warnings` pass; 98 tests pass |
| CI for M4.1–M5 | run 41347094495: Ubuntu/macOS/Windows success, including queue and desktop |
| CI for M6.1 | run 41349015729: Ubuntu/macOS/Windows success |
| CI for M7.1 | run 41349735081: Ubuntu/macOS/Windows success |
| CI for M7.2 baseline | run 41350365611: Ubuntu/macOS/Windows success |
| CI for scale/DMG | run 41352461334: Ubuntu/macOS/Windows success |
| CLI batch smoke test | 2 valid + 1 corrupt fixture: 2 completed, 1 failed; outputs/reports written |
| Batch memory | 1 file 5.2 MB, 4 files 6.8 MB, 16 files 7.0 MB peak RSS (`--jobs 2`, release) |
| CI (M2–M3.8) | run 41339781578: Ubuntu/macOS/Windows success |

Screenshots and exact checks: `docs/MANUAL_QA.md`, `docs/qa/`.

The Penpot backend reports `2.18.5-RC1` even though the image tag is `2.18.5`; record the
backend string, not just the tag, in future checks.

## 5. Implemented behavior

- Reader: ZIP/raw containers, bounded Kiwi schema/message decoding, nodes, transforms,
  paints (solid/gradient/image), effects, opacity, corner radii, `resizeToFit`, text
  metrics, stack/Auto Layout fields, vector geometry blob references, and `vectorData`
  presence; image bytes are loaded under per-entry and total caps.
- Conversion:
  - visible pages; internal-only canvas excluded;
  - frames, frame-encoded groups (resizeToFit), rectangles, ellipses, basic text;
  - filled vector paths from `commandsBlob`, multiple subpaths per node, absolute commands;
  - absolute world coordinates (ADR-0005);
  - solid fills and strokes, alpha → fill/stroke opacity, node opacity;
  - linear/radial gradients from normalized geometry and stops;
  - drop/inner shadows, layer blur, background blur;
  - image fills backed by deterministic media/object entries;
  - corner radii r1–r4;
  - text: single paragraph/run, family/style/size, weight, fill, horizontal/vertical
    alignment, line height, letter spacing; auto-height mapping;
  - Auto Layout → flex direction, gap, padding, wrap, alignment; constraints mapped;
  - masks → Penpot masked groups; booleans → rendered editable paths with diagnostics;
  - paint and text styles → library entries; effect styles inlined with a diagnostic;
  - components, instances with basic overrides, and variant sets;
  - Figma variables → DTCG token sets/themes; paint variable bindings → `appliedTokens`;
  - unsupported node types, non-solid paints, unknown stroke alignment, empty text/groups,
    stroke-only outlines, and lost vector networks are reported with stable codes.
- Writer: Penpot 2.18.5/v67 archive, deterministic UUIDs (ADR-0007), manifest last,
  atomic write.
- Validator: manifest/file/page/shape structure, IDs, refs, finite geometry, fills,
  strokes, text/path content; rejects missing refs.
- Report: event-based counts, worst-outcome per element (ADR-0008), schema-versioned JSON,
  self-contained HTML, RFC 4180 CSV, batch aggregate HTML/CSV/JSON, grouping, and copy
  actions.
- Queue: bounded workers, stage events, pause/cancel, per-job failure isolation, CLI batch,
  and desktop “Convert all”.

## 6. Explicitly not implemented

- Boolean operands/parameters, editable vector networks (handle-level), grid layout, Auto
  Layout child sizing, per-paint blend modes, effect-style library linkage, grid styles.
- Conditionally partial: instance swaps/component properties, variables bound to layout or
  typography, mixed/per-character text styles, multiple paragraphs, lists.
- Rotation/flip is approximated/reported; not preserved.
- Locked/blocked, page background.
- Full instance resize/reflow, component property assignments, and instance swaps.
- Mid-write cancellation; Windows MSI and signing/notarization.
- Reopen/move/edit persistence verification for documents.

## 7. Decisions

| ADR | Decision |
|---|---|
| 0001 | Project license: MIT OR Apache-2.0 |
| 0002 | Reimplementation without code reuse |
| 0003 | Target Penpot 2.18.5 / v67 |
| 0004 | Superseded by 0006 for decoding |
| 0005 | Penpot geometry is absolute |
| 0006 | Bounded in-house Kiwi decoder |
| 0007 | Deterministic UUIDs from Figma GUIDs |
| 0008 | Explicit Skipped/Unsupported report outcomes |
| 0009 | egui/eframe desktop shell, std-thread worker, native dialogs (GTK3 on Linux) |
| 0010 | Designer-first UI and honest presentation mappings |
| 0011 | Bounded private-file compatibility fixes and explicit remaining fidelity limits |

## 8. Verified vs. unverified

- **Verified (Penpot 2.18.5-RC1):** M2 pages, frames, groups, rectangles/radii, ellipses,
  solid fills/strokes, basic text, absolute positions, layer hierarchy, CLI convert;
  M3.1 filled vector paths, masks, boolean fallback; M3.2 gradients/shadows/layer+background
  blur; M3.3 image fills; M3.5 flex direction/gap/padding/alignment; M3.6 paint/text styles;
  M3.7 components/instances/variants; M3.8 tokens and applied fill tokens.
- **Implemented (unverified):** JSON report/validator beyond the inspected subset; opacity
  node-level; other node types handled by unit tests.
- **Implemented (unverified):** M3.4 line-height/letter-spacing numeric values; radial
  gradients; inner shadows; JPEG/GIF image sniffing; effect-style inlining.
- **Partial:** stroke-only vectors, vector network handles, booleans (flattened), mask mode,
  layer order, hidden/locked, missing fonts, Auto Layout child sizing, fixed constraints,
  per-paint blend modes, instance swaps/properties, variable layout bindings, mixed text
  runs, rotation.
- **Not supported / reported:** grid layout, grid styles, procedural effects, lists.
- **Partial:** M4/M6 designer UI: all screens, worker, plain-language reports, grouping,
  filters and exports exist; native dialogs, drag/drop, accessibility and full platform
  interactions need QA. Unsigned packaging exists; the new source fixes are unreleased.
- **Partial:** M5 batch core, retry/remove, and historical memory benchmarks are implemented;
  mid-write cancellation and batch token-preference propagation remain pending.
- **Planned:** jump-to-element (canvas navigation), broader redistributable component-heavy
  corpus, signing/notarization, and Windows MSI. macOS DMG is already implemented.
- **Not supported:** `.deck`/`.jam`, lists, procedural effects.

## 9. Known gaps and open questions

1. **Reopen/move/edit persistence untested.** Do not claim it.
2. **Group inspector showed a blue "GROUP FILL"** while the generated group has no fills;
   visual output is correct, but understand Penpot's display before claiming group styling.
3. **Text auto-height changed 40→20**, expected from `HEIGHT`, but should be documented.
4. **crates.io intermittent.** M2/M3.1 built with `--offline` after dependencies were
   cached; CI still needs network for new crates.
5. **Historical CI was green at `5a06cc4`** (run 41352461334) on Linux, macOS, and Windows; release
   run 41351622397 published `v0.1.0`; release run 41352464597 published `v0.1.1`
   with a DMG. The Node 20 action annotation remains cosmetic; harden action pinning in M8.
6. **Docker stack was stopped** after M3.8 imports. Re-run via compose for future imports;
   telemetry off.
7. **No redistributable component-heavy real-world corpus yet.** Five private compatibility
   files were tested locally; committed component/token fixtures remain project-owned.
8. **`openfig-core` synthetic fixture generated with Node** (research-only); the product
   remains pure Rust and the fixture is project-owned.
9. **Vector handle metadata is not read.** Paths are command-array geometry; current reports
   emit `F2P-VECTOR-NETWORK-LOST` when `vectorData` is present.
10. **Auto Layout child sizing is not mapped.** The container is flex and verified, but child
    grow/fill behavior is reported via `F2P-LAYOUT-PARTIAL`.
11. **Text metrics are only partly import-verified.** The typography style and linkage
    imported, but line-height/letter-spacing values were not read back from form fields.
12. **Effect styles and instance swaps are not fully modeled.** Effects are inlined; swaps
    and component properties are reported as Partial/unmapped.
13. **Token modes use duplicate token names across mode sets.** Only the first mode's set is
    active by default; activating the other theme selects the other set. This is honest but
    differs from Figma's per-mode override model.

## 10. Next steps (in order)

1. Fix batch token-preference propagation with an end-to-end regression test.
2. Implement and diagnose remaining component properties/swaps, resized child layout and
   transform behavior; continue the documented vector/text/grid/token follow-ups.
3. Compare every migrated page against Figma; verify component editing and reopening, and
   read back typography metrics in Penpot before upgrading fidelity claims.
4. Measure peak RSS and run targeted fuzz cases for expanded components; broaden the
   redistributable corpus without committing private inputs.
5. Complete native platform/accessibility QA, license/advisory scans and full notices;
   align versions and package a new release from its verified commit.

## 11. How to resume

From this checkout, use a stable Rust environment with the required platform dependencies:

```bash
git status
cargo test --offline --locked --workspace
cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig
cargo run -p f2p-cli -- convert tests/fixtures/circle.fig --out /tmp/f2p-check --force
```

For a real import, start the Penpot compose stack described in `docs/MANUAL_QA.md`, then
follow its checklist. Do not trust a status claim without recorded evidence.

## 12. Session log

### Session 17 — 2026-11-02

- Reviewed the documentation set and synchronized overview, roadmap, architecture, format,
  support/reporting, environment/QA, privacy, provenance/license, release, UI, and fixture/
  benchmark guidance with the tested source. Added a complete documentation index.
- Replaced the stale proposed architecture with the implemented standard-thread/mpsc,
  materialized conversion pipeline. Corrected obsolete release status, pending-import
  claims, planned-feature descriptions, and the batch token-preference propagation claim.
- Regenerated the locked Linux metadata inventory: **221 third-party packages**, all with
  declared license expressions. This is target-scoped metadata, not a clean compliance
  scan or full cross-platform license-text bundle. Dependency/lockfile contents are unchanged.
- Retained historical ADRs, screenshots, goldens, and timing results as dated evidence;
  no private designs/reports or new fidelity percentages were added to public documentation.
- Source gates are the recorded **116 passing workspace tests**, formatting and Clippy;
  documentation link checks cover **40 Markdown files**, **149 local links**, and all
  **30 converter codes**, with no missing targets or mappings. Diff checks passed. The private delivery
  checksum/ZIP integrity checks remain valid. `cargo-deny`/`cargo-audit` scans are unverified.
- Publishing scope: the compatibility implementation/tests and synchronized documentation;
  private outputs, development helpers, and credentials remain local/ignored. No release
  version or immutable tag is changed. The user requested push and merge into `main`.

### Session 16 — 2026-11-02

- Parsed all five private files (Figma versions 106/110), recorded source hashes, and ran
  baseline conversions. Found pruned sections, missing embedded component content, a
  10,000-entry output ceiling, empty fill caches hiding stroke outlines, and invalid numbers.
- Added dependency-ordered embedded master restoration, nested override key paths, path
  translation, section boards, clipping, target paint order, bounded expansion/cache checks,
  and configurable archive validation limits. Added explicit diagnostics for unmapped
  instance fields, resized constraints/transforms, and token name normalization.
- Fixed issues found by real imports: non-finite layout gaps, token identifiers, and
  unsupported linear blend identifiers. Recovered one unknown container size from actual
  converted child bounds, retaining the uncertainty in the event.
- Studied the reference README at `c29242f60ebe3579265260894b40e9bcfe54ffb3`; no upstream
  implementation was copied or ported. See ADR-0011 and the existing license policy.
- Every delivered archive was structurally validated and accepted by an isolated local
  Penpot **2.18.5-RC1** instance (SSE `end`, no error event). All five reports say
  `completed_with_warnings`, with **0 failed elements**. Unsupported counts by supplied file
  order: **26, 0, 120, 41, 20**. These are not lossless conversions.
- Native Penpot UI opened the navigation file and Swiss Army page list; a navigation
  screenshot was inspected inline. Full visual comparison, edit/reopen persistence, and
  exhaustive component behavior remain unverified. A Swiss screenshot capture timed out.
- `cargo test --offline --locked --workspace`: **116 passed**; full-workspace Clippy with
  `-D warnings` and formatting pass. No dependency/lockfile changes. `cargo deny check` and
  `cargo audit` return `no such command`; their scans remain unverified.
- Windows has no host Rust toolchain. Used isolated local Linux containers with stable Rust
  1.99.1 and native GTK/X11/Wayland build dependencies. Graph tools still return
  `Transport closed`; source reads and tests supply verification.
- Private artifacts/reports/import logs live under
  `C:\Users\Danny\Downloads\Figma2Penpot-converted-2026-11-02`, outside tracked fixtures.
- Packaged five source-named archives and fifteen JSON/HTML/CSV reports; the ZIP integrity
  check passed. Source/output hashes matched. Stopped only task-owned verification/build
  containers after testing and closed the isolated browser tab; volumes remain available.
- Next: review the remaining report limitations, implement component properties/swaps and
  resize/reflow behavior, then broaden visual/fuzz and native platform QA. Do not upgrade
  overall fidelity to Verified from import acceptance alone.

### Session 15 — 2026-11-02

- Reviewed repository instructions, status, product brief, architecture, license assessment,
  support matrix, issue codes, UI contract, recent ADRs, crate manifests, CI/release workflows,
  and the main reader/converter/writer/report/queue/desktop/CLI entry points.
- Recorded the current Windows environment in `docs/DEV_ENVIRONMENT.md` §9. No application
  code or conversion behavior changed; no new architecture decision was needed.
- Codebase-memory indexing and coverage checks failed with `Transport closed`; structural
  understanding is based on direct source reads and `rg`, without graph verification.
- Source-only follow-up: `conversion-queue::run_job` currently passes
  `ConvertOptions::default()` instead of the job's `keep_token_bindings` value. The latest
  UI notes describe that preference as propagated through batch conversion; runtime
  verification and any fix are pending a task and a working Rust toolchain.
- Older sections of this file retain stale pending/export/packaging claims. Use the latest
  session notes, support matrix, and source together when choosing the next change.
- Build, fmt, clippy, tests, deny, audit, GUI, and Penpot import were not run in this session;
  previous passing results are historical evidence, not verification on this machine.
- Next step: receive the implementation task and verify the affected paths and requirements.

### Session 1 — 2026-11-01

- Completed M0 research and pushed the public repository.

### Session 2 — 2026-11-01

- Completed M1 reader/CLI, fixtures, fuzz, CI.

### Session 3 — 2026-11-01

- Extended the reader for paints, strokes, radii, text, and group flags.
- Implemented `conversion-report`, `conversion-core`, `penpot-writer` (writer + validator),
  and `f2p-cli convert`.
- Added a project-owned synthetic text/group fixture and a golden report.
- Ran 54 tests, clippy, fmt; all pass.
- Pushed M2; GitHub Actions run 41327038266 passed on Linux, macOS, and Windows.
- Started Penpot 2.18.5 via Docker, imported both outputs, inspected UI, captured
  screenshots; backend logged `error?=false`. Stopped the stack.
- Recorded ADR-0007/0008 and updated docs.

### Session 13 — 2026-11-02

- Added committed `large-synthetic.fig`: 3,300 nodes, 2.4 MB archive, 0.23 s / 66.7 MB peak
  RSS; added it to the regression baseline (11 fixtures).
- Added a macOS DMG packager (4.3 MB compressed image + SHA-256) and wired it into the
  release workflow.
- Tagged `v0.1.1`; release run 41352464597 published 10 assets including the macOS DMG.
- 95 tests pass.

### Session 12 — 2026-11-02

- Added macOS `.app` packaging, Windows zip and Linux tarball scripts, and a tag-driven
  GitHub Actions release workflow with checksums.
- Built and launched the macOS bundle locally; captured `docs/qa/macos-app-bundle.png`.
- Added release, privacy, and third-party notices documentation.
- Tagged `v0.1.0`; release workflow built macOS, Windows, and Linux artifacts, generated
  checksums, and published a GitHub pre-release.
- Windows/Linux install-and-launch and signing/notarization remain unverified.

### Session 11 — 2026-11-02

- Added a checked-in conversion baseline (`tests/benchmarks/baseline.json`) and a CLI
  integration test that re-converts all 10 fixtures and asserts counts, status, validation.
- Added error-recovery tests: atomic write failure cleans temporary files; CLI write failure
  reports failed status without a partial archive.
- 95 tests pass; M7 still needs a large corpus and low-disk/cancel-mid-write drills.

### Session 10 — 2026-11-02

- Added a repeatable benchmark record in `docs/PERFORMANCE.md`.
- Ran a 60-second fuzz campaign: 818,204 executions, 49 MB peak RSS, no artifacts.
- Measured all synthetic fixtures and a local 4,629-node file in release mode.
- Confirmed bounded batch memory at 1/4/16 files with 2 workers.
- M7 still pending: large-file corpus, checked-in regression baseline, low-disk/short-write
  and cancel-mid-write drills.

### Session 14 — 2026-11-02

- Redesigned the desktop UI for product designers: bundled Inter (OFL 1.1), tokenized
  light/dark theme, reusable components, project-owned line icons, and a dev-only gallery.
- Added `copy.rs`: designer-facing copy plus tested mappings for every file status, queue
  state, stage, outcome, severity, validation state, and all 28 known issue codes. Raw
  codes, node IDs, and worker messages stay under **Show details** and in exports.
- Added a designer-first conversion report: default groups by what to do, search, severity
  and result filters, page/what-to-do grouping, advice text, copy actions, and JSON/HTML/CSV
  export. The report remains event-based; grouping is presentation only.
- Wired **What matters more?** to the real engine option `keep_token_bindings` through
  `conversion-core`, `conversion-queue`, the desktop workers, and CLI `--inline-tokens`.
- Added cooperative single-file pause/cancel and corrected the single-file worker to honor
  the token preference. Cancelling before saving leaves no output.
- Added empty, loading, partial, error, and confirmation states; friendly error mapping;
  clear-all and reset-preference confirmations; offline license modal.
- Captured all eight screens in light and dark at large/compact sizes plus gallery, empty
  Files, error Results, licenses, and clear-all evidence under `docs/ui/screenshots/`.
- Wrote `docs/UI_SPEC.md`, `docs/ui/WIREFRAMES.md`, `docs/ui/REVIEW.md`, and ADR 0010.
- `cargo fmt --all -- --check` and
  `cargo clippy --offline --workspace --all-targets -- -D warnings` pass;
  `cargo test --offline --workspace` reports **98 tests pass**.
- Not verified: native dialogs, drag-and-drop, keyboard-only traversal, screen-reader
  output, Windows/Linux visuals, and cancellation during an in-flight atomic write.
  `cargo deny`/`cargo audit` are not installed.
- Pushed as `4421884`; CI run 41415820909 passed on Ubuntu, macOS, and Windows.

### Session 9 — 2026-11-02

- Added self-contained HTML and CSV exporters to `conversion-report`, with escaping and
  batch aggregate coverage (3 tests).
- Added `f2p-cli report` and `batch --html/--csv`.
- Added desktop HTML/CSV export actions using native save dialogs.
- Added batch JSON aggregate, report grouping by issue/page, and copy-issue/node actions.
- Rendered the generated HTML report in a browser and captured
  `docs/qa/report-html.png`.
- Pushed as `571fa52`; CI run 41349015729 passed on Ubuntu, macOS, and Windows.
- Workspace now has 92 tests.

### Session 8 — 2026-11-02

- Added `conversion-queue` with bounded workers, per-job isolation, stage events,
  pause/cancel checkpoints, panic containment, and a mixed-batch test.
- Added `f2p-cli batch`; smoke-tested with two valid fixtures and one corrupt file
  (2 completed, 1 failed, no cross-contamination).
- Added desktop “Convert all” with per-file batch states, pause/cancel controls, batch
  summary, and a 2-file headless integration test.
- Workspace now has 88 tests; 4 queue tests and 20 desktop tests.
- Added retry-failed/retry-selected/remove-completed controls.
- Measured release batch peak RSS: 5.2 MB (1 file), 6.8 MB (4), 7.0 MB (16) at 2 workers.
- Pushed as `c7b8453`; CI run 41347094495 passed on Ubuntu, macOS, and Windows.
- M5 still pending: cancellation during a write and a batch aggregate report.

### Session 7 — 2026-11-01

- Added `crates/desktop-app`: eframe/egui native shell with app ID
  `app.figma2penpot.desktop`.
- Implemented all eight screens with a headless `AppState`; conversion/analysis run on
  worker threads and report stages over `mpsc`.
- Added native file/folder dialogs (`rfd`), drag-and-drop staging, settings persistence,
  middle-truncated paths with tooltips, and light/dark/system theme selection.
- Added command-line `.fig` path staging for manual QA and future open-with support.
- Added 20 desktop tests: queue/settings/state, real fixture conversion, all-screen headless
  rendering, and end-to-end worker → Results/Report.
- Updated CI with Linux GTK3/X11/Wayland build dependencies.
- Captured Home, Queue, and Light theme screenshots; native dialog, drag-drop, and
  remaining screen interactions still need manual QA.
- Pushed as `df5b13f`; CI run 41344261599 passed on Ubuntu, macOS, and Windows.

### Session 6 — 2026-11-01

- Decoded Figma styles, style references, symbols/instances, variants, variables, masks,
  booleans, blend modes, and constraints.
- Implemented Penpot color/typography libraries and links; effect/grid style diagnostics.
- Implemented component registration, instance materialization with basic overrides, and
  variant sets.
- Implemented variable → DTCG token sets/themes and paint `appliedTokens`.
- Implemented mask groups and boolean rendered-path fallback.
- Added `styles.fig`, `components.fig`, `tokens.fig`, `masks-boolean.fig` fixtures with
  provenance and hashes; 64 tests pass.
- Imported all four into Penpot 2.18.5; `error?=false`; captured UI evidence.
- Updated support matrix, issue codes, manual QA, fixtures README, README, and roadmap.
- Pushed as `7abe99a`; CI run 41339781578 passed on Ubuntu, macOS, and Windows.

### Session 5 — 2026-11-01

- Added paint gradients/images, effects, text metrics, and stack fields to the reader.
- Implemented gradients, shadows, layer/background blur, image media/object packaging,
  text alignment/line height/letter spacing, and Auto Layout → flex.
- Added synthetic fixtures `effects-gradient.fig`, `image.fig`, `auto-layout.fig` with
  provenance and hashes; added integration tests (60 tests total).
- Imported all four outputs into Penpot 2.18.5; `error?=false`; UI inspected and
  screenshots added under `docs/qa/`.
- Updated support matrix, issue codes, manual QA, and fixture README.

### Session 4 — 2026-11-01

- Added vector geometry references and `vectorData` presence to the reader.
- Implemented `commandsBlob` decoding, absolute command transforms, and Penpot path
  command arrays in `penpot-writer`.
- Fixed multi-path parent linkage: every path from one Figma vector is now a child of the
  same parent.
- Added vector tests: filled OpenFigs icon (20 paths), stroke-only line (approximated).
- Imported the OpenFigs icon (20 paths) and stroke-only line into Penpot 2.18.5; both
  `error?=false`; rendered correctly.
- Added `F2P-VECTOR-*` diagnostics and updated support matrix, manual QA, and docs.
- 56 tests, clippy, fmt pass locally; pushed M3.1, CI run 41329353648 green on all three OS.
