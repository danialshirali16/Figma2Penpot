# Manual QA

Status: **M2 and M3.1–M3.5 imports PASSED** on a local Penpot 2.18.5 stack (backend
reports `2.18.5-RC1`), 2026-11-01. The converter outputs imported without errors and the
features below were inspected in the Penpot UI. Reopen/edit persistence was not tested.
Text metrics (M3.4) are implemented and unit-tested but have **no import evidence** yet.

Evidence:

- Import IDs: `483d95b6-a489-8130-8008-e13ecfd8ed17` (circle) and
  `483d95b6-a489-8130-8008-e1400a1891fc` (Text Group Fixture); backend logged
  `error?=false`.
- Screenshots: [`docs/qa/circle-import.png`](qa/circle-import.png),
  [`docs/qa/text-group-layers-expanded.png`](qa/text-group-layers-expanded.png),
  [`docs/qa/red-rect-inspect.png`](qa/red-rect-inspect.png),
  [`docs/qa/text-inspect.png`](qa/text-inspect.png),
  [`docs/qa/group-inspect.png`](qa/group-inspect.png).

---

## 1. M2 — Penpot import

Target: **Penpot 2.18.5** (image tag `2.18.5`; backend version `2.18.5-RC1`), archive v3,
file data version 67. Docker Compose stack from the official `docker/images/docker-compose.yaml`
at tag 2.18.5, with telemetry disabled.

### Setup

1. Start self-hosted Penpot 2.18.5.
2. Create a local account and a project. No cloud account is required.
3. Build and generate outputs:

   ```bash
   cargo run -p f2p-cli -- convert tests/fixtures/circle.fig --out /tmp/f2p-manual --force
   cargo run -p f2p-cli -- convert tests/fixtures/text-group.fig --out /tmp/f2p-manual --force
   ```

4. Reports said `Validation: passed` and `status` `completed`.

### Import and inspect

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import accepts the file | No import error | **Pass** — both files; `error?=false` |
| 2 | File name | `circle` / `Text Group Fixture` | **Pass** |
| 3 | Pages | One visible page; no internal canvas | **Pass** — only `Page 1` |
| 4 | Frame position | `Card` at (20,30), 400×300 | **Pass** — inspector shows X 20, Y 30, W 400, H 300 |
| 5 | Child position | Ellipse at (25,25), 300×300; `Red Rect` at absolute (30,50) | **Pass** — ellipse visually centered; red rect inspector shows X 10, Y 20 *relative to Card* (absolute 30,50) |
| 6 | Fill | Red `#FF0000` | **Pass** — inspector `FF0000`, 100% |
| 7 | Stroke | 2 px black on `Red Rect`; 5 px on circle ellipse | **Pass** — red rect inspector shows `000000`, 2; circle stroke visible |
| 8 | Group | `Badge Group` 80×30, child `Group Rect` | **Pass** — inspector W 80, H 30, X 5, Y 205 (relative); layer tree shows child |
| 9 | Rounded corners | `Red Rect` radius 4 | **Pass** — inspector shows 4 |
| 10 | Text | `Title` = `Hello Penpot`, Inter 16, weight 400 | **Pass** — inspector shows Inter, 16, 400 |
| 11 | Text position | `Title` at absolute (30,130) | **Pass** — inspector X 10, Y 100 relative to Card |
| 12 | Z-order | Layers match Figma order | **Pass** — `Title`, `Green Ellipse`, `Red Rect`, `Badge Group` |
| 13 | Selection | Shapes can be selected | **Pass** — selected frame, group, text, and rect |
| 14 | Editing/move | Move a shape and undo | **Untested** |
| 15 | Reopen | Save/close/reopen keeps the document intact | **Untested** |

Observed difference: the text box height changed from 40 to 20 because Figma
`textAutoResize: HEIGHT` was mapped to Penpot `auto-height`. This is expected auto-sizing,
not data loss, and the text is fully visible.

### Failure handling

If a future import fails:

1. Capture the server/import error verbatim.
2. Re-run the structural validator and record its result.
3. Add the failing archive as a regression fixture only if its license allows.
4. Open an issue with the archive, report, Penpot version, and steps.
5. Mark the affected support-matrix rows **Partial** or **Planned**; never **Verified**.

---

## 2. M3.1 — Vector paths

Verified on the same Penpot 2.18.5 stack, 2026-11-01.

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Filled vector import | No error | **Pass** — `OpenFigs.penpot`, `error?=false`, import id `d047d42d-cd5b-80cd-8008-e1452fac5611` |
| 2 | Multi-path expansion | One path per fill geometry | **Pass** — 20 path layers under the frame |
| 3 | Visual result | White OpenFig icon rendered | **Pass** — see `qa/vector-paths-expanded.png` |
| 4 | Stroke-only vector import | No error; reported approximation | **Pass** — `straight-2-point-line.penpot`, import id `d047d42d-cd5b-80cd-8008-e145acec5587` |
| 5 | Stroke-only appearance | Line rendered | **Pass** — green line; converted as a filled outline, report says approximated |
| 6 | Editable vector network | Handles/vertices editable | **Untested** — M3.1 uses `commandsBlob`, not the network |
| 7 | Boolean operations | Preserved/tree | **Untested** — not implemented |
| 8 | Rotated/scaled vector | Transform preserved | **Untested** — matrix is applied to commands, no fixture yet |
| 9 | Vector network handles | Editable handles preserved | **Partial** — `commandsBlob` path is editable; `vectorData` network is not read. Current builds report `F2P-VECTOR-NETWORK-LOST`; the imported archive was generated before that diagnostic was added, with identical geometry |

The stroke-only case is intentionally **Partial**: Figma stores an expanded outline, so the
line is a filled path rather than an editable stroke. The report labels it
`F2P-VECTOR-STROKE-OUTLINE` and the file is marked completed with warnings.

---

## 3. M3.2 — Gradients, shadows, blur

Verified on the same Penpot 2.18.5 stack, 2026-11-01. Fixture:
`effects-gradient.fig` (linear red→blue gradient, drop shadow, layer blur, background blur).

- Import id: `17b9706f-02f0-8188-8008-e14cce8b07fa`; backend `error?=false`.
- Screenshots: [`docs/qa/effects-gradient-import.png`](qa/effects-gradient-import.png),
  [`docs/qa/effects-gradient-inspect.png`](qa/effects-gradient-inspect.png),
  [`docs/qa/effects-gradient-panel.png`](qa/effects-gradient-panel.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Gradient fill | Red→blue linear gradient | **Pass** — rendered and listed as a gradient fill |
| 3 | Drop shadow | Offset 2/4, blur 8, 25% black | **Pass** — shadow effect listed |
| 4 | Layer blur | Blur 4 | **Pass** — listed in the effects panel |
| 5 | Background blur | Blur 6 | **Pass** — listed in the effects panel |
| 6 | Radial gradient / inner shadow | Mapped | **Untested** — code path exists, no fixture/import |

## 4. M3.3 — Image fills

- Import ids: `17b9706f-02f0-8188-8008-e1519513b5a7` (white pixel) and
  `17b9706f-02f0-8188-8008-e153dadf0c7c` (red 8×8); backend `error?=false`.
- Screenshots: [`docs/qa/image-import.png`](qa/image-import.png),
  [`docs/qa/image-red-import.png`](qa/image-red-import.png),
  [`docs/qa/image-inspect.png`](qa/image-inspect.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — both archives; `error?=false` |
| 2 | Media packaging | `files/<file>/media/<id>.json` + `objects/<id>.json` + binary | **Pass** — structural validator and import |
| 3 | Image rendering | Red 8×8 image visible in the rect | **Pass** — rendered red |
| 4 | Image fill inspector | Fill references packaged media | **Pass** — fill listed on `Image Fill` |
| 5 | Deduplication | Identical hashes share one media entry | **Pass (unit)** — registry keyed by hash |
| 6 | JPEG/GIF dimensions | Sniffed and converted | **Untested** — code path exists, no fixture |

## 5. M3.5 — Auto Layout

- Import id: `17b9706f-02f0-8188-8008-e1588bbf28a8`; backend `error?=false`.
- Screenshots: [`docs/qa/auto-layout-import.png`](qa/auto-layout-import.png),
  [`docs/qa/auto-layout-inspect.png`](qa/auto-layout-inspect.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Flex container | Layout = flex, direction row | **Pass** — inspector shows flex row |
| 3 | Gap | 12 px column gap | **Pass** — inspector shows 12 |
| 4 | Padding | Top 8, right 20, bottom 12, left 10 | **Pass** — inspector shows 8/20/12/10 |
| 5 | Alignment | Primary center, counter center | **Pass** — inspector matches |
| 6 | Child sizing | Grow/fill preserved | **Partial** — not mapped; report emits `F2P-LAYOUT-PARTIAL` |
| 7 | Grid layout | Mapped | **Untested** — not implemented |

---

## 6. Future platform checklist (M4/M8)

Not started. To be filled when the desktop app exists:

- Windows 10/11 installer: install, launch, drag-drop, convert, uninstall.
- macOS `.app`/`.dmg`: Gatekeeper behavior, Apple Silicon, Intel where built.
- Light/dark themes, HiDPI, long paths, keyboard shortcuts, screen reader.
- Batch: pause, cancel, retry, crash isolation.
- Offline: conversion with network disabled.
