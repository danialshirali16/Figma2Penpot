# Manual QA

Status: **M2 import PASSED** on a local Penpot 2.18.5 stack (backend reports
`2.18.5-RC1`), 2026-11-01. The converter outputs imported without errors and the M2
features were inspected in the Penpot UI. Reopen/edit persistence was not tested.

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

## 2. Future platform checklist (M4/M8)

Not started. To be filled when the desktop app exists:

- Windows 10/11 installer: install, launch, drag-drop, convert, uninstall.
- macOS `.app`/`.dmg`: Gatekeeper behavior, Apple Silicon, Intel where built.
- Light/dark themes, HiDPI, long paths, keyboard shortcuts, screen reader.
- Batch: pause, cancel, retry, crash isolation.
- Offline: conversion with network disabled.
