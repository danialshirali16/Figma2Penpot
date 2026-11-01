# PROGRESS

Living status file for Figma 2 Penpot. Canonical place to resume from.

Last updated: **2026-11-01** (session 3, M2).

---

## 1. Where we are

**Milestone: M2 complete and verified by real Penpot import. M3 is next.**

The workspace reads real `.fig` files, converts an M2 subset to `.penpot`, validates the
archive structurally, and writes an event-based JSON report. Both M2 fixtures imported into
a local **Penpot 2.18.5** stack (`2.18.5-RC1`) with `error?=false`; geometry, fills, strokes,
corner radius, text, and group hierarchy were inspected in the UI.

M2 acceptance:

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
└── docs/                    M0/M1/M2 docs + ADRs 0001–0008
```

## 3. Environment

See `docs/DEV_ENVIRONMENT.md`.

- macOS 27.2, arm64, 8 GiB RAM.
- Rust 1.99.1, no rustup; `cargo-fuzz 0.13.2`.
- Docker Desktop daemon 29.9.2; Penpot 2.18.5 stack was started, used, and stopped.
- Telemetry was disabled for the local Penpot stack.
- crates.io access became intermittent; M2 built with `--offline` after dependencies were
  cached. Do not assume network in CI/local builds.

## 4. Evidence log (session 3)

| Command / check | Result |
|---|---|
| `cargo test --offline --workspace` | **54 tests pass** (unit/integration + 1 doctest) |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | Success |
| `cargo fmt --all -- --check` | Success |
| `cargo run -p f2p-cli -- convert tests/fixtures/circle.fig --out …` | 2 detected, 2 converted, validation passed |
| `cargo run -p f2p-cli -- convert tests/fixtures/text-group.fig --out …` | 6 detected, 6 converted, validation passed |
| `cargo run -p f2p-cli -- convert tests/fixtures/OpenFigs.fig` | 2 detected, 1 converted, 1 unsupported (`VECTOR`) |
| Penpot import circle | `483d95b6-a489-8130-8008-e13ecfd8ed17`, `error?=false`, 0.157 s |
| Penpot import text-group | `483d95b6-a489-8130-8008-e1400a1891fc`, `error?=false`, 0.044 s |
| UI inspection | Card X20 Y30 W400 H300; Red Rect relative X10 Y20 (absolute 30,50), fill FF0000, stroke 2, radius 4; Title Inter 16/400; group 80×30 with child |

Screenshots and exact checks: `docs/MANUAL_QA.md`, `docs/qa/`.

The Penpot backend reports `2.18.5-RC1` even though the image tag is `2.18.5`; record the
backend string, not just the tag, in future checks.

## 5. Implemented behavior

- Reader: ZIP/raw containers, bounded Kiwi schema/message decoding, nodes, transforms,
  solid paints/strokes, opacity, corner radii, `resizeToFit`, basic text attributes.
- Conversion:
  - visible pages; internal-only canvas excluded;
  - frames, frame-encoded groups (resizeToFit), rectangles, ellipses, basic text;
  - absolute world coordinates (ADR-0005);
  - solid fills and strokes, alpha → fill/stroke opacity, node opacity;
  - corner radii r1–r4;
  - text: single paragraph/run, family/style/size, weight, fill; auto-height mapping;
  - unsupported node types, non-solid paints, unknown stroke alignment, empty text/groups
    are reported with stable codes.
- Writer: Penpot 2.18.5/v67 archive, deterministic UUIDs (ADR-0007), manifest last,
  atomic write.
- Validator: manifest/file/page/shape structure, IDs, refs, finite geometry, fills,
  strokes, text content; rejects missing refs.
- Report: event-based counts, worst-outcome per element (ADR-0008), schema-versioned JSON.

## 6. Explicitly not implemented

- Vectors/paths/booleans, gradients, effects, masks, images, layout constraints, styles,
  components/instances/variants, tokens.
- Text: mixed styles, paragraph alignment, line height, letter spacing, lists, text styles.
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
  solid fills/strokes, basic text, absolute positions, layer hierarchy, CLI convert.
- **Implemented (unverified):** JSON report/validator beyond the inspected subset; opacity
  node-level; other node types handled by unit tests.
- **Partial:** layer order complex cases, hidden/locked, text paragraphs/alignment, missing
  fonts, images enumeration, rotation.
- **Planned:** all M3+ fidelity, reports UI/exports, GUI, batch.
- **Not supported:** `.deck`/`.jam`, lists, procedural effects.

## 9. Known gaps and open questions

1. **Reopen/move/edit persistence untested.** Do not claim it.
2. **Group inspector showed a blue "GROUP FILL"** while the generated group has no fills;
   visual output is correct, but understand Penpot's display before claiming group styling.
3. **Text auto-height changed 40→20**, expected from `HEIGHT`, but should be documented.
4. **crates.io intermittent.** CI should be retried; local M2 used `--offline`.
5. **CI not yet run for M2.** Push and verify fmt/clippy/test on three OS.
6. **Docker stack was stopped.** Re-run via compose for future imports; telemetry off.
7. **No real component/media/token data**, so those remain unverified.
8. **`openfig-core` synthetic fixture generated with Node** (research-only); the product
   remains pure Rust and the fixture is project-owned.

## 10. Next steps (in order)

1. Push M2; confirm CI green on Linux/macOS/Windows.
2. Add M3.1 vectors/paths (`commandsBlob`) with golden tests and Penpot import.
3. Add gradients/effects/masks (M3.2) with explicit fallbacks.
4. Add images (M3.3) with hash dedup and media writer.
5. Then text fidelity (M3.4), layout/constraints (M3.5), styles (M3.6), components (M3.7),
   tokens (M3.8), each with support-matrix rows and import evidence.

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
- Started Penpot 2.18.5 via Docker, imported both outputs, inspected UI, captured
  screenshots; backend logged `error?=false`. Stopped the stack.
- Recorded ADR-0007/0008 and updated docs.
