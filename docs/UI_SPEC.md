# UI Specification — Figma 2 Penpot Desktop

Status: **M4/M6 redesign, 2026-11-02.** This document is the design contract for the
desktop app. `Reality wins` over this spec: discrepancies and engine limits are recorded in
`docs/DECISIONS/`.

## 1. Persona and jobs

**Primary persona — Product Designer.**
They use Figma and Penpot daily. They know frames, components, variants, auto layout,
styles, variables, tokens, and pages. They do not know — and should never need — terminal,
Rust, Kiwi, schema, archive, GUID, IR, or `F2P-*` codes. They are migrating real work and
are anxious about loss.

**Top three tasks**

1. Bring a folder of `.fig` files into the app without setup.
2. Convert them and know, in plain language, whether the designs survived.
3. Find exactly what needs checking in Penpot and open the output/report.

**Core question:** “Did my design survive, and what do I need to check?”

## 2. End-to-end flow

```text
Launch
  → Home / Import        drop files or choose a folder
  → Files                review names, sizes, states; choose output
  → File details         friendly contents + things to know
  → Settings             output folder + token preference (optional)
  → Converting           real stages, per-file + overall; pause/cancel
  → Results              status per file; open folder / report / retry failed
  → Report               grouped by what to do; expandable advice; export
  → Open in Penpot       exact verified import steps
```

First useful action (add files) is one click from launch. Converting is one click from the
Files screen. No setup wizard.

## 3. Screen inventory and primary action

| Screen | Purpose | One primary action |
|---|---|---|
| Home / Import | Start a migration | Drop files / **Choose files…** |
| Files | Review the queue | **Convert** |
| File details | Understand a file before conversion | **Convert this file** (or Back) |
| Settings | Choose where output goes and token preference | **Done** |
| Converting | See real stages | **Cancel** (always available) |
| Results | Know what survived | **Open folder** |
| Report | Know what to check | **Open in Penpot guide** / **Export** |
| Preferences | Theme, storage, privacy, licenses | **Done** |

A dev-only **Component gallery** exists behind `--gallery`; it is not part of navigation.

## 4. Layout

- Native utility: 1180×760 default, minimum 900×620.
- **Sidebar** 220 px: brand, workflow steps (Import, Files, Details, Convert, Results,
  Report), then Settings and Preferences. Collapses to icons below 980 px where practical.
- **Content** max width 1040 px, 32 px page padding, generous whitespace.
- **Persistent footer** on Files: file count, output folder, **Convert**.
- **Top bar** on content screens: title, short subtitle, privacy chip, optional secondary.
- Empty/loading/error states are designed for every screen.
- File lists are virtualized once counts exceed 100 (egui `ScrollArea::show_rows`).

## 5. Visual design system

Implement in `desktop-app/src/theme.rs`; screens use tokens only.

### Colour tokens

| Token | Light | Dark | Use |
|---|---|---|---|
| `bg` | `#F5F5F2` | `#141514` | App background |
| `surface` | `#FFFFFF` | `#1D1F1D` | Cards, panels |
| `surface_muted` | `#ECEDE8` | `#242624` | Inputs, secondary fills |
| `border` | `#DADBD4` | `#343732` | 1 px borders |
| `text` | `#1B1D1A` | `#F0F2EE` | Primary text |
| `text_muted` | `#656861` | `#A6AAA2` | Secondary text |
| `accent` | `#0F7A63` | `#6FD8B8` | Primary action, links |
| `accent_soft` | `#E2F2EC` | `#173B32` | Selected/soft accent |
| `success` | `#1E7A3A` | `#7ED99A` | Converted |
| `warning` | `#946100` | `#F2C46B` | Needs review |
| `danger` | `#B3261E` | `#FF8A80` | Couldn’t convert |
| `info` | `#245F9E` | `#8DBEF5` | Neutral information |

Status always uses **icon + text + colour**. Never colour alone.

### Typography

Bundled **Inter** variable font (SIL OFL 1.1; license in `THIRD_PARTY_LICENSES.md`).
Monospace: egui’s bundled monospace, details only.

| Token | Size / line height | Use |
|---|---|---|
| `caption` | 12 / 16 | Metadata, chips |
| `body` | 14 / 20 | Default |
| `body_large` | 16 / 24 | File names, form values |
| `section` | 20 / 28 | Section headings |
| `title` | 28 / 34 | Screen titles |
| `hero` | 40 / 46 | Home only |

Avoid all-caps except tiny overline labels. Support 40% longer translations; no fixed text
widths.

### Spacing, radii, borders

- Spacing: 4, 8, 12, 16, 24, 32, 48.
- Radii: 6 (inputs), 10 (cards), 14 (drop zone/modals), 999 (pills).
- Borders: 1 px `border`; focus 2 px `accent`.
- Shadows: only floating menus/tooltips; cards use borders.

### Icons

Project-owned line icons drawn with egui painter (`components::icon`): file, folder, check,
alert-triangle, x-circle, clock, pause, play, trash, arrow-right, settings, info, external,
chevron, search, upload, download, sparkle. Icons accompany labels; never replace them.

### Motion

120–180 ms ease-out for expansion/hover; no looping motion. No decorative animation. Respect
`ctx.options().reduce_motion` where available; if unavailable, keep motion optional.

## 6. Copy deck (canonical strings live in `copy.rs`)

### Global

- App: **Figma 2 Penpot**
- Tagline: **Move your designs from Figma to Penpot. Simply.**
- Privacy: **Everything stays on your computer.**
- Buttons: **Choose files…**, **Choose a folder…**, **Convert**, **Cancel**, **Pause**,
  **Resume**, **Open folder**, **View report**, **Export**, **Done**, **Back**,
  **Try again**, **Remove**, **Clear all**.

### Home

- Title: **Bring your Figma work to Penpot.**
- Body: **Drop your Figma files here. We’ll convert them and tell you exactly what to check.**
- Drop zone: **Drop your Figma files here**
- Secondary: **Choose files…** · **Choose a folder…**
- Reassurance: **Everything stays on your computer. No account, no upload.**
- How it works: **1 Add files · 2 Convert · 3 Review in Penpot**
- Recent: **Last saved to** + folder.

### Files

- Title: **Your files**
- Subtitle: **Nothing is converted until you press Convert.**
- Empty: **No files yet.** / **Add a .fig file or a folder to get started.**
- Row states: **Ready**, **Can’t be opened — try saving it again from Figma**,
  **Analyzing…**, **Queued**, **Converting…**, **Converted**, **Couldn’t be converted**,
  **Cancelled**.
- Footer: **{n} files** · **Save to {folder}** · **Change** · **Convert**
- Invalid type: **This isn’t a Figma file. Choose a .fig file.**

### File details

- Title: **What’s in this file?**
- Friendly counts: **Pages**, **Components**, **Styles**, **Variables**, **Images**,
  **Text layers**.
- Warning section: **Things to be aware of** / **Nothing unusual found.**
- Advanced (collapsed): **Technical details**.
- Unknown: **Not available** — never a guessed number.

### Settings

- Title: **Conversion settings**
- **Where should we save the converted files?** + folder picker.
- **What matters more?**
  - **Looks identical** — “Keep the visual result, even if tokens aren’t linked.”
  - **Keeps tokens connected** — “Prefer token and style links where Penpot supports them.”
- Advanced (collapsed): overwrite policy (**Ask me / Replace / Keep both**), validation,
  report formats, concurrency. Only controls that actually exist are shown; unsupported
  ones are omitted, not faked.
- Primary: **Done**.

### Converting

- Title: **Converting your files**
- Stages: **Reading your file**, **Converting layers**, **Packaging for Penpot**,
  **Checking the result**, **Saving**.
- Footer: **You can keep working. Pause stops new files; Cancel stops the batch.**
- Partial output: **Cancelled files leave no output behind.**
- No percentages unless the true denominator is known.

### Results

- Title: **Your conversion is ready**
- Summary: **{n} converted · {n} need a quick review · {n} couldn’t be converted**
- Per file: **Converted** / **Converted — needs a quick review** /
  **Couldn’t be converted** / **Cancelled**.
- Attention line: **{n} things to check in Penpot.**
- Buttons: **Open folder**, **View report**, **Retry failed**.
- Import guide: **In Penpot, open Drafts → Options → Import Penpot files, then choose the
  .penpot file.** (Verified against Penpot 2.18.5 in `MANUAL_QA.md`.)

### Report

- Title: **Conversion report**
- Summary: **{n} layers converted · {n} need a quick review · {n} weren’t converted**
- Default groups: **Needs your attention**, **Slightly different**, **Not converted**,
  **All good**.
- Issue pattern: element name; page/layer path; **What happened**, **What we did instead**,
  **What to do in Penpot**.
- **Show details** reveals issue code, node ID, raw converter message.
- Empty: **Nothing needs your attention.**
- Validation not run: **We couldn’t double-check this file. Open it in Penpot and look it
  over.**

### Preferences

- Title: **Preferences**
- **Appearance**: Follow system / Light / Dark.
- **Default output folder**.
- **Language: English** — **More languages are planned.**
- Privacy: **No telemetry, no uploads, no network calls during conversion.**
- **About** version, licenses link.

## 7. Status and issue mapping (presentation layer)

Internal values remain in data and exports. UI maps every value; tests assert total coverage.

### File status

| Internal | User-facing | Category |
|---|---|---|
| `Completed` | **Converted** | all_good |
| `CompletedWithWarnings` | **Converted — needs a quick review** | review |
| `Failed` | **Couldn’t be converted** | attention |
| `Cancelled` | **Cancelled** | neutral |

### Validation

| Internal | User-facing |
|---|---|
| `Passed` | **Checked and ready** |
| `Failed` | **Please check this file in Penpot** |
| `NotRun` | **We couldn’t double-check this file** |

### Severity / outcome

| Internal | User-facing |
|---|---|
| `Info` | **Good to know** |
| `Warning` | **Needs a quick look** |
| `Error` | **Needs your attention** |
| `Converted` | **Converted** |
| `ConvertedWithModification` | **Slightly different** |
| `Skipped` | **Skipped** |
| `Unsupported` | **Not converted** |
| `Failed` | **Couldn’t convert** |

### Issue advice

Implement `copy::issue(code) -> IssueCopy { title, happened, action, category }`. Known
codes:

| Code | User title | Category |
|---|---|---|
| `F2P-NODE-CONVERTED` | Layer converted | all_good |
| `F2P-NODE-APPROXIMATED` | Layer looks slightly different | review |
| `F2P-NODE-UNSUPPORTED` | Layer type isn’t supported | not_converted |
| `F2P-NODE-FAILED` | Layer couldn’t be converted | attention |
| `F2P-GROUP-EMPTY` | Empty group skipped | not_converted |
| `F2P-PAINT-UNSUPPORTED` | A fill was left out | review |
| `F2P-STROKE-UNSUPPORTED` | A stroke was left out | review |
| `F2P-STROKE-ALIGNMENT` | Stroke position changed | review |
| `F2P-STROKE-WIDTH` | Stroke width was missing | review |
| `F2P-VECTOR-STROKE-OUTLINE` | Line became a filled shape | review |
| `F2P-VECTOR-NETWORK-LOST` | Vector handles were simplified | review |
| `F2P-EFFECT-UNSUPPORTED` | An effect was left out | review |
| `F2P-EFFECT-DUPLICATE` | Extra blur was combined | review |
| `F2P-LAYOUT-PARTIAL` | Auto layout sizing changed | review |
| `F2P-LAYOUT-GRID-UNSUPPORTED` | Grid layout wasn’t kept | not_converted |
| `F2P-MASK-APPROXIMATED` | Mask may look different | review |
| `F2P-MASK-NO-TARGET` | Mask had nothing to clip | review |
| `F2P-BOOLEAN-FLATTENED` | Boolean became editable shapes | review |
| `F2P-BLEND-MODE-LOST` | Blend mode changed | review |
| `F2P-CONSTRAINT-APPROXIMATED` | Resize behavior changed | review |
| `F2P-STYLE-EFFECT-INLINED` | Effect style was copied onto the layer | review |
| `F2P-STYLE-GRID-LOST` | Grid style wasn’t kept | not_converted |
| `F2P-INSTANCE-DETACHED` | Component copy was detached | attention |
| `F2P-INSTANCE-OVERRIDE-LOST` | A component change was reset | review |
| `F2P-TEXT-MIXED-STYLES-LOST` | Mixed text styles were flattened | review |
| `F2P-VECTOR-EMPTY` | Empty vector skipped | not_converted |
| `F2P-VECTOR-FAILED` | Vector couldn’t be converted | attention |
| `F2P-TEXT-EMPTY` | Empty text skipped | not_converted |
| unknown | Something changed | review |

The advice text for each is written for designers, not auditors. Raw converter descriptions
remain under **Show details**.

## 8. Accessibility checklist

- [x] Text + icon + colour for every status; no colour-only meaning.
- [x] Contrast target WCAG AA (body 4.5:1, large text 3:1) in both themes.
- [x] Keyboard navigation for nav, buttons, lists, dialogs, and report expand/collapse.
- [x] Visible focus ring; no focus traps.
- [x] Labels are programmatic where egui/AccessKit exposes them; icon-only buttons have
      tooltips and accessible labels.
- [x] Minimum 32 px control height for touch/pointer comfort.
- [x] Layouts survive 40% longer strings; middle truncation + full-path tooltip.
- [x] Reduce-motion respected (no essential motion; short optional transitions).
- [x] Dialogs dismissible with Esc; destructive actions ask for confirmation.
- [x] Empty/loading/error states use text, not colour or animation alone.
- [x] Screen-reader support: build with AccessKit feature where packaging allows; document
      what remains unverified.

## 9. Discrepancies / decisions

- **No thumbnails in file rows** until the reader exposes a verified thumbnail path; use a
  neutral file icon, never a fake preview. Record in `DECISIONS/0010-ui-redesign.md`.
- **No page-selection / page-splitting / report-format UI** until the engine supports them;
  do not present dead controls.
- **Fidelity choice** maps to a real engine option: *Keeps tokens connected* preserves
  token/style links; *Looks identical* keeps resolved values but removes those links. If
  the option cannot be wired end-to-end, it is omitted rather than faked.
- **Concurrency** stays advanced and bounded; default 2.
- **Jump-to-element** is a report breadcrumb plus *Copy layer name/ID* until a canvas viewer
  exists; no dead “jump” button.
- **Languages**: UI is English and centralized for i18n; no fake language switcher.
