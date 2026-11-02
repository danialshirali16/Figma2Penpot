# Manual QA

Status: **M2 and M3.1–M3.8 imports PASSED** on a local Penpot 2.18.5 stack (backend
reports `2.18.5-RC1`), 2026-11-01. **M4.2/M6.2 designer UI reviewed on macOS 27.2**
(light/dark, large/compact) on 2026-11-02; native dialog, drag-drop, keyboard-only, and
screen-reader checks remain manual/unverified. The converter outputs imported without
errors and the features below were inspected in the Penpot UI. Reopen/edit persistence was
not tested.
Text line-height/letter-spacing values are unit-tested and carried through the imported
typography style, but were not independently read from the form fields.

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

## 5. M3.4 — Text metrics

No separate import; the `styles.fig` text uses the `Heading` typography with line height
1.5 and letter spacing 2. Screenshot:
[`docs/qa/text-metrics-inspect.png`](qa/text-metrics-inspect.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Typography link | Text uses `Heading (Inter)` | **Pass** — shown in the text section |
| 2 | Font family/size | Inter 24 | **Pass (style asset)** — typography asset shows Inter |
| 3 | Line height | 1.5 | **Untested** — value carried in the style but not read back from the form |
| 4 | Letter spacing | 2 | **Untested** — value carried in the style but not read back from the form |
| 5 | Mixed runs | Flattened with a diagnostic | **Pass (unit)** — no mixed-run fixture/import |

## 6. M3.5 — Auto Layout

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

## 7. M3.6 — Styles

- Import id: `17b9706f-02f0-8188-8008-e15ca6d94f79`; backend `error?=false`.
- Screenshots: [`docs/qa/styles-import.png`](qa/styles-import.png),
  [`docs/qa/styles-assets.png`](qa/styles-assets.png),
  [`docs/qa/styles-colors-expanded.png`](qa/styles-colors-expanded.png),
  [`docs/qa/styles-color-primary.png`](qa/styles-color-primary.png),
  [`docs/qa/styles-rect-inspect.png`](qa/styles-rect-inspect.png),
  [`docs/qa/styles-text-inspect.png`](qa/styles-text-inspect.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Color library | One `Brand/Primary` color | **Pass** — Assets shows `Colors 1`, `Brand > Primary #FF0040` |
| 3 | Fill linkage | Rect fill shows the style name and a Detach action | **Pass** — `Primary` shown in the fill section |
| 4 | Typography library | One `Type/Heading` typography | **Pass** — `Typographies 1`, `Heading (Inter)` |
| 5 | Text linkage | Text run uses the typography | **Pass** — text panel shows `Heading (Inter)` |
| 6 | Effect styles | Inline effects with a diagnostic | **Pass (unit)** — no effect-style fixture/import |
| 7 | Grid styles | Reported as lost | **Pass (unit)** — no fixture |

## 8. M3.7 — Components and variants

- Import id: `17b9706f-02f0-8188-8008-e15e6267215d`; backend `error?=false`.
- Screenshots: [`docs/qa/components-import.png`](qa/components-import.png),
  [`docs/qa/components-assets.png`](qa/components-assets.png),
  [`docs/qa/components-assets-expanded.png`](qa/components-assets-expanded.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Component library | Three components | **Pass** — Assets shows `Components 3` |
| 3 | Main component | Registered from the symbol board | **Pass** — `Button` under `Controls` |
| 4 | Instance | Materialized with the component subtree | **Pass** — blue button rendered inside `Usage` |
| 5 | Text override | Instance label differs from the main | **Pass** — `Overridden` rendered |
| 6 | Variant set | Two variants under one set | **Pass** — two thumbnails; `Button Size` variant set |
| 7 | Instance swaps / component properties | Mapped | **Untested** — not implemented; reported as Partial |

## 9. M3.8 — Variables and tokens

- Import id (final): `17b9706f-02f0-8188-8008-e16014ec506a`; backend `error?=false`.
- Screenshots: [`docs/qa/tokens-panel.png`](qa/tokens-panel.png),
  [`docs/qa/tokens-color-expanded.png`](qa/tokens-color-expanded.png),
  [`docs/qa/tokens2-panel.png`](qa/tokens2-panel.png),
  [`docs/qa/tokens-rect-inspect.png`](qa/tokens-rect-inspect.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Token sets | Light and Dark sets | **Pass** — `Primitives` with `Light`/`Dark` |
| 3 | Themes | One theme per mode | **Pass** — Light and Dark themes listed |
| 4 | Mode values | Light `#FF0040`, Dark `#80001A` | **Pass** — Light active shows `#ff0040`; Dark defined in its set |
| 5 | Applied token | Fill references `brand.primary` | **Pass** — fill shows `Name: brand.primary`, resolved `#ff0040` |
| 6 | Number token | `space.sm` = 8/12 | **Pass** — `Number 1` in the token panel |
| 7 | Layout/typography bindings | Applied to shapes | **Untested** — not implemented; reported as Partial |

## 10. M3.1/M3.2 — Masks and booleans

- Import id: `17b9706f-02f0-8188-8008-e16179c815cf`; backend `error?=false`.
- Screenshot: [`docs/qa/masks-boolean2-import.png`](qa/masks-boolean2-import.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Import | No error | **Pass** — `error?=false` |
| 2 | Mask group | Rect mask clips the ellipse | **Pass** — red lens shape clipped to the circle |
| 3 | Mask diagnostic | Reported approximation | **Pass** — `F2P-MASK-APPROXIMATED` |
| 4 | Boolean path | Subtract donut renders | **Pass** — green square with a hole |
| 5 | Boolean diagnostic | Reported flattening | **Pass** — `F2P-BOOLEAN-FLATTENED` |
| 6 | Boolean editability | Operands editable | **Untested** — rendered path is editable, operands are not reconstructed |

---

## 11. M4.1 — Desktop shell

Captured on macOS 27.2 (arm64), 2026-11-01. Desktop tests pass headlessly; native dialog
and drag-drop interaction were not automated. Screenshots:
[`docs/qa/desktop-home.png`](qa/desktop-home.png),
[`docs/qa/desktop-queue.png`](qa/desktop-queue.png),
[`docs/qa/desktop-light.png`](qa/desktop-light.png).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Launch | Native window opens | **Pass** — 1180×792 titled `Figma 2 Penpot` |
| 2 | Home | Brand, drop zone, Choose files, nav | **Pass** — captured |
| 3 | Queue | File staged from CLI argument; size/state/Remove visible | **Pass** — `circle.fig`, 31.5 KiB, `— Ready` |
| 4 | Eight screens | All render without panic | **Pass** — headless `egui` smoke test |
| 5 | Background conversion | Worker converts fixture and writes archive/report | **Pass** — desktop integration test |
| 6 | Results/Report | Real report data renders | **Pass** — headless end-to-end test |
| 7 | Settings persistence | JSON round-trip and malformed fallback | **Pass (unit)** |
| 8 | Native file dialog | Choose `.fig` | **Untested** — requires OS interaction |
| 9 | Drag and drop | Dropped file enters queue | **Untested** — requires OS interaction |
| 10 | Folder scan | Finds `.fig`, skips hidden folders, caps at 512 | **Pass (unit)** |
| 11 | Keyboard shortcuts | Cmd/Ctrl+2 jumps to Queue | **Pass (headless)**; remaining shortcuts untested manually |
| 12 | Light theme | Readable light appearance | **Pass** — settings file selected Light; captured |
| 13 | Preferences reload | Saved settings load on launch | **Pass** — Light theme loaded from disk |

Observed harness limitation: this terminal lacks Screen Recording permission, so the window
was captured with `screencapture -x -l <window-id>` after locating the window via
CoreGraphics. That does not affect the app.

## 12. M4.2 / M6.2 — Designer UI review

Reviewed on **macOS 27.2 (arm64, Apple M1)**, 2026-11-02. The app was built with
`cargo build --offline -p desktop-app` and launched against real `.fig` fixtures. Window
captures used `screencapture -x -l <window-id>` (the terminal lacks Screen Recording
permission for full-screen capture). This is an internal visual review; no external
participants were involved.

Evidence: [`docs/ui/screenshots/`](ui/screenshots/), with all eight screens in light and
dark at large and compact sizes, plus the gallery, empty Files, error Results, license
modal, and clear-all confirmation. Findings are in [`docs/ui/REVIEW.md`](ui/REVIEW.md).

| # | Check | Expected | Result |
|---|---|---|---|
| 1 | Launch and Home | One obvious import action; privacy visible | **Pass** — light/dark, both sizes |
| 2 | Files queue | Names, sizes, statuses, destination, Convert | **Pass** — four real files; rows fit at 900 px |
| 3 | Empty Files | One centered add choice, no duplicate toolbar | **Pass** — `files-empty-light-large.png` |
| 4 | File details | Real counts, warnings, collapsed technical details | **Pass** — `components.fig` parsed; counts shown |
| 5 | Settings | Output folder; one two-way fidelity choice | **Pass** — choice wired to `keep_token_bindings`; unit test |
| 6 | Single-file Converting | Named stages, Pause and Cancel, no invented % | **Pass** — `converting-*-*.png` |
| 7 | Batch Results | Real conversions, review count, retry, import guide | **Pass** — four fixtures; title changes on failure |
| 8 | Error Results | Friendly first message, raw details available | **Pass** — `results-error-light-large.png`; unit mapping |
| 9 | Report | What-to-do groups, advice text, filters, export menu | **Pass** — real report; compact toolbar wraps |
| 10 | Preferences | Theme, output, privacy, licenses, reset | **Pass** — license modal captured |
| 11 | Component gallery | Every component in both themes and long text | **Pass** — gallery light/dark |
| 12 | Clear all | Confirmation before removing the queue | **Pass** — modal captured; Esc path in code |
| 13 | Light/dark consistency | No unstyled default widgets | **Pass** — both themes at both sizes |
| 14 | Conversion report honesty | Event-based counts; no hidden Not converted/Approximated | **Pass (unit + report sample)** — unchanged engine/report model |
| 15 | Internal status mappings | Every value has designer copy | **Pass (unit)** — `copy::tests` |
| 16 | Keyboard-only flow | Tab/Enter through import → convert → report | **Untested** — headless Cmd+2 test only |
| 17 | Screen reader | VoiceOver labels and reading order | **Untested** — eframe build has no AccessKit feature |
| 18 | Native file picker | Choose files/folder on macOS | **Untested** — requires OS interaction |
| 19 | Drag and drop | Dropped files/folder enter queue | **Untested** — requires OS interaction |
| 20 | Windows/Linux visuals | Same screens and themes | **Untested** — no Windows/Linux machine run |
| 21 | Penpot import of UI-produced files | New outputs import | **Not re-run** — UI does not change the writer; converter imports are recorded in §1–10 |

Quality gates at review time: `cargo fmt --all -- --check` **pass**;
`cargo clippy --offline --workspace --all-targets -- -D warnings` **pass**;
`cargo test --offline --workspace` **98 tests pass**. `cargo deny` and `cargo audit` are
not installed in this environment, so license/advisory checks remain **pending**.

### Manual steps still needed

1. **Native dialogs:** launch the app, press **Choose files…**, pick a `.fig`, confirm it
   appears in Files; repeat with **Choose a folder…** on a nested folder.
2. **Drag and drop:** drag one `.fig` and then a folder onto Home; confirm the hover state
   says “Release to add these files” and the queue receives the files.
3. **Keyboard only:** unplug the pointer, use Tab/Shift+Tab, Enter/Space, Esc, and
   Cmd/Ctrl+1–8 to reach Home → Files → Convert → Results → Report; record the focus ring.
4. **Screen reader:** build with an AccessKit-enabled eframe feature set, then run VoiceOver
   (macOS) or NVDA (Windows) through the same flow. Record labels, roles, and reading order.
5. **Windows/Linux:** repeat the screenshot matrix and native dialog/drag-drop checks on
   each OS; Windows High Contrast and Linux dark themes are not covered by this review.
6. **Penpot import:** import one UI-produced `.penpot` into Penpot 2.18.5 and confirm the
   import guide path (`Drafts → Options → Import Penpot files`) is still current.

---

## 13. Future platform checklist (M4/M8)

Not started. To be filled when the desktop app exists:

- Windows 10/11 installer: install, launch, drag-drop, convert, uninstall.
- macOS `.app`/`.dmg`: Gatekeeper behavior, Apple Silicon, Intel where built.
- Light/dark themes, HiDPI, long paths, keyboard shortcuts, screen reader.
- Batch: pause, cancel, retry, crash isolation.
- Offline: conversion with network disabled.

## 14. Private-file compatibility — 2026-11-02

Five user-supplied files were tested locally on Windows using the Rust CLI in an isolated
Linux container and a separate local Penpot stack. Target image: 2.18.5; browser runtime:
**2.18.5-RC1**. Telemetry was disabled; imports used loopback HTTP. Private designs, reports,
and screenshots are not committed as fixtures.

| Check | Result |
|---|---|
| Read all five source archives | **Verified** — versions 106/110 |
| Generated archive structural validation | **Verified** — all passed |
| Real Penpot imports | **Verified** — all five SSE streams ended successfully with no import error event |
| Per-file conversion reports | **Verified** — event-based; all `completed_with_warnings`, zero failed elements |
| Full lossless fidelity | **Partial** — unsupported concepts and approximations remain explicitly reported |
| Navigation-file UI | **Verified** — opened, page list/layers and design visible; screenshot inspected inline |
| Swiss Army UI | **Partial** — title and source/restored page list opened; full screenshot capture timed out |
| Exhaustive visual comparison / every page | **Partial** — limited UI checks only; no pixel-fidelity percentage claimed |
| Editing/reopen persistence | **Planned** — still pending |

Private evidence is under
`C:\Users\Danny\Downloads\Figma2Penpot-converted-2026-11-02\verification`:
`source-manifest.json`, `delivery-summary.json`, per-file import SSE/JSON records, and the
workspace test log. The delivery includes source-named `.penpot` files and JSON/HTML/CSV
reports. Hash comparisons verify source immutability and that delivered archive copies
match the imported bytes.

Real imports first exposed null layout gaps, invalid token names, and unsupported blend
identifiers that the old validator missed. Tests now assert these failure cases and the
corresponding recorded conversions/fallbacks. See ADR-0011 for choices and remaining limits.
