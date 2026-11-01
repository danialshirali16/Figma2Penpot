# PROGRESS

Living status file for Figma 2 Penpot. Canonical place to resume from.

Last updated: **2026-11-01** (session 5, M3.2–M3.5).

---

## 1. Where we are

**Milestone: M3 in progress. M3.1–M3.5 are implemented; M3.1–M3.3 and M3.5 container
properties are import-verified. M3.4 text metrics are unit-tested but not imported.**

The workspace reads real `.fig` files, converts the M2 subset plus vector geometry,
gradients, shadows, blur, image fills, text metrics, and flex containers to `.penpot`,
validates archives structurally, and writes event-based JSON reports. M2–M3.3 and M3.5
outputs imported into a local **Penpot 2.18.5** stack (`2.18.5-RC1`) with `error?=false`.
Booleans, editable vector networks, masks, Auto Layout child sizing, styles, components, and
tokens remain planned.

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
| Booleans | Planned | not implemented |
| Golden test | Done | `filled_vector_converts_to_multiple_paths`; 20 paths; `openfigs.report.json` |

M3.2–M3.5 acceptance:

| Requirement | Status | Evidence |
|---|---|---|
| Gradients (linear/radial) | Done | normalized start/end + stops; linear import verified |
| Shadows (drop/inner) | Done | Penpot `shadow` array; drop import verified |
| Layer and background blur | Done | Penpot `blur`/`backgroundBlur`; both import verified |
| Masks/clipping | Planned | not implemented |
| Images + hash dedup | Done | media/object archive; red 8×8 PNG rendered |
| Text alignment | Done (unverified) | horizontal/vertical strings; unit test only |
| Line height / letter spacing | Done (unverified) | percent/pixels/raw → ratio/px; unit test only |
| Auto Layout → flex | Partial | direction/gap/padding/alignment verified; child sizing reported |
| Grid layout | Planned | not implemented |

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
│   └── f2p-cli/             inspect, convert
├── fuzz/                    libFuzzer harness `read_fig`
├── tests/fixtures/          5 fixtures (4 upstream MIT + 1 synthetic) + provenance
├── docs/qa/                 real-import screenshots (Penpot 2.18.5-RC1)
├── .github/workflows/       CI (Linux/macOS/Windows)
└── docs/                    M0/M1/M2/M3.1 docs + ADRs 0001–0008
```

## 3. Environment

See `docs/DEV_ENVIRONMENT.md`.

- macOS 27.2, arm64, 8 GiB RAM.
- Rust 1.99.1, no rustup; `cargo-fuzz 0.13.2`.
- Docker Desktop daemon 29.9.2; Penpot 2.18.5 stack was started, used, and stopped.
- Telemetry was disabled for the local Penpot stack.
- crates.io access became intermittent; M2 built with `--offline` after dependencies were
  cached. Do not assume network in CI/local builds.

## 4. Evidence log (session 4)

| Command / check | Result |
|---|---|
| `cargo test --offline --workspace` | **60 tests pass** (unit/integration + 1 doctest) |
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
| CI (M2/M3.1) | run 41329353648: Ubuntu/macOS/Windows success |

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
  - Auto Layout → flex direction, gap, padding, wrap, alignment;
  - unsupported node types, non-solid paints, unknown stroke alignment, empty text/groups,
    stroke-only outlines, and lost vector networks are reported with stable codes.
- Writer: Penpot 2.18.5/v67 archive, deterministic UUIDs (ADR-0007), manifest last,
  atomic write.
- Validator: manifest/file/page/shape structure, IDs, refs, finite geometry, fills,
  strokes, text/path content; rejects missing refs.
- Report: event-based counts, worst-outcome per element (ADR-0008), schema-versioned JSON.

## 6. Explicitly not implemented

- Booleans, editable vector networks (handle-level), masks, grid layout, Auto Layout child
  sizing/constraints, styles, components/instances/variants, tokens.
- Text: mixed/per-character styles, multiple paragraphs, lists, text styles.
- Rotation/flip is approximated/reported; not preserved.
- Locked/blocked, page background.
- HTML/CSV report export.
- GUI, batch queue, cancellation.
- Reopen/move/edit persistence verification.

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

## 8. Verified vs. unverified

- **Verified (Penpot 2.18.5-RC1):** M2 pages, frames, groups, rectangles/radii, ellipses,
  solid fills/strokes, basic text, absolute positions, layer hierarchy, CLI convert;
  M3.1 filled vector paths; M3.2 gradients/shadows/layer+background blur; M3.3 image fills;
  M3.5 flex direction/gap/padding/alignment.
- **Implemented (unverified):** JSON report/validator beyond the inspected subset; opacity
  node-level; other node types handled by unit tests.
- **Implemented (unverified):** M3.4 text alignment/line height/letter spacing; radial
  gradients; inner shadows; JPEG/GIF image sniffing.
- **Partial:** stroke-only vectors (filled outline), vector network handles, layer order
  complex cases, hidden/locked, missing fonts, Auto Layout child sizing, rotation.
- **Planned:** booleans, masks, grid layout, styles, components, tokens, reports UI/exports,
  GUI, batch.
- **Not supported:** `.deck`/`.jam`, lists, procedural effects.

## 9. Known gaps and open questions

1. **Reopen/move/edit persistence untested.** Do not claim it.
2. **Group inspector showed a blue "GROUP FILL"** while the generated group has no fills;
   visual output is correct, but understand Penpot's display before claiming group styling.
3. **Text auto-height changed 40→20**, expected from `HEIGHT`, but should be documented.
4. **crates.io intermittent.** M2/M3.1 built with `--offline` after dependencies were
   cached; CI still needs network for new crates.
5. **CI is green at `01a4f52` for M3.1** (run 41329353648) on Linux, macOS, and Windows.
   The Node 20 action annotation remains cosmetic; harden action pinning in M8.
6. **Docker stack was stopped** after M3.1 import. Re-run via compose for future imports;
   telemetry off.
7. **No real component/media/token data**, so those remain unverified.
8. **`openfig-core` synthetic fixture generated with Node** (research-only); the product
   remains pure Rust and the fixture is project-owned.
9. **Vector handle metadata is not read.** Paths are command-array geometry; current reports
   emit `F2P-VECTOR-NETWORK-LOST` when `vectorData` is present.
10. **Auto Layout child sizing is not mapped.** The container is flex and verified, but child
    grow/fill behavior is reported via `F2P-LAYOUT-PARTIAL`.
11. **Text metrics have no import evidence yet.** Alignment, line height, and letter spacing
    are unit-tested only; add a text fixture and import before marking Verified.

## 10. Next steps (in order)

1. Commit and push M3.2–M3.5; confirm CI green on Linux/macOS/Windows.
2. M3.6 styles (paint/text/effect) with a fixture and import.
3. M3.7 components, instances, and variants with a fixture and import.
4. M3.8 variables/tokens with an explicit lossy-mapping report.
5. M3.1 remainder: booleans and editable vector networks (handle-level).
6. M3.4 follow-up: text-metrics fixture and import; masks; child sizing/constraints.

## 11. How to resume

```bash
cd "/Users/danial/Downloads/Figma 2 Penpot"
git status
cat PROGRESS.md
cargo test --offline --workspace
cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig
cargo run -p f2p-cli -- convert tests/fixtures/circle.fig --out /tmp/f2p-check --force
```

For a real import, start the Penpot compose stack described in `docs/MANUAL_QA.md`, then
follow its checklist. Do not trust a status claim without recorded evidence.

## 12. Session log

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
