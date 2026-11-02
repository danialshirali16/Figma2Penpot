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

**Screenshot evidence:** `docs/ui/screenshots/` contains every screen in light and dark at
two window sizes (large and compact), plus the component gallery, empty Files, error
Results, license modal, and clear-all confirmation. Captures were taken on macOS at Retina
resolution and scaled to 1440 px wide for the repository. Findings and fixes are in
`docs/ui/REVIEW.md`.

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
  **Try again**, **Remove**, **Clear all**, **Check again**, **Dismiss**.
- Output fallback: **Next to each .fig file**; **Use input folder**.
- Offline footer: **Offline · no account**.
- Clear-all confirmation: **Clear all files?** / “This removes every file from the list.
  Files on your computer aren’t deleted.”
- Paused state: **Paused. Resume when you’re ready.**

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
- Batch footer: **You can keep working. Pause stops new files; Cancel stops the batch.**
- Single-file footer: **You can keep working. Pause holds this file between steps; Cancel
  stops it.**
- Partial output: **Cancelled files leave no output behind.** (Cooperative checkpoints
  between stages; cancellation during an atomic write is not interruptible.)
- No percentages unless the true denominator is known.

### Results

- Title: **Your conversion is ready** when files converted cleanly; **Your conversion
  finished — some files need attention** when any failed; **Conversion cancelled** when
  only cancellations remain.
- Summary: **{n} converted · {n} need a quick review · {n} couldn’t be converted ·
  {n} cancelled** (singular: **1 needs a quick review**).
- Per file: **Converted** / **Converted — needs a quick review** /
  **Couldn’t be converted** / **Cancelled**.
- Attention line: **{n} things to check in Penpot.**
- Buttons: **Open folder**, **View report**, **Retry failed**.
- Import guide: **Bring it into Penpot** — **In Penpot, open Drafts → Options → Import
  Penpot files, then choose the .penpot file.** (Verified against Penpot 2.18.5 in
  `MANUAL_QA.md`.)
- Batch report selector: **Report for**.

### Report

- Title: **Conversion report**
- Summary: **{n} layers converted · {n} need a quick review · {n} weren’t converted**
- Default groups: **Needs your attention**, **Not converted**, **Slightly different**,
  **All good**.
- Filters: search, **Severity**, **Result**, group by **What to do / Page / None**.
- Issue pattern: element name; page/layer path; **What happened**, **What we did instead**,
  **What to do in Penpot**.
- **Show details** reveals issue code, node ID, raw converter message.
- Export: **HTML report**, **CSV report**, **JSON report**.
- Empty: **Nothing needs your attention.**
- Validation not run: **File check didn’t run — look it over in Penpot.**

### Preferences

- Title: **Preferences**
- **Appearance**: Follow system / Light / Dark.
- **Default output folder**.
- **Language: English** — **More languages are planned.**
- Privacy: **No telemetry, no uploads, no network calls during conversion.**
- **About** version; **Licenses** opens an offline modal with the Inter OFL text
  (“Scroll to read the full license.”).
- **Reset preferences** asks for confirmation before restoring defaults: **Reset
  preferences?** / “Theme, output folder, and conversion choices return to their defaults.
  Your files and converted outputs aren’t touched.”

## 7. Status and issue mapping (presentation layer)

Internal values remain in data and exports. UI maps every value; tests assert total coverage.

### File status

| Internal | User-facing | Category |
|---|---|---|
| `Completed` | **Converted** | all_good |
| `CompletedWithWarnings` | **Converted — needs a quick review** | review |
| `Failed` | **Couldn’t be converted** | attention |
| `Cancelled` | **Cancelled** | neutral |

### Queue state

| Internal | User-facing |
|---|---|
| `Ready` | **Ready** |
| `Analyzing` | **Analyzing…** |
| `Analyzed` | **Ready** |
| `Queued` | **Queued** |
| `Converting` | **Converting…** |
| `Converted` | **Converted** |
| `Failed(message)` | **Couldn’t be converted** + `copy::failure_message(message)` |
| `Cancelled` | **Cancelled** |

### Stage

| Internal | User-facing |
|---|---|
| `Reading` | **Reading your file** |
| `Converting` | **Converting layers** |
| `Writing` | **Packaging for Penpot** |
| `Validating` | **Checking the result** |
| `Saving` | **Saving** |

### Report category

| Internal | User-facing |
|---|---|
| `Attention` | **Needs your attention** |
| `NotConverted` | **Not converted** |
| `Review` | **Slightly different** |
| `AllGood` | **All good** |

### Validation

| Internal | User-facing |
|---|---|
| `Passed` | **File check passed** |
| `Failed` | **File check failed — open it in Penpot and look it over** |
| `NotRun` | **File check didn’t run — look it over in Penpot** |

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
| `F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED` | Component settings need checking | review |
| `F2P-TOKEN-NAME-NORMALIZED` | A token name was adjusted | review |
| `F2P-TEXT-MIXED-STYLES-LOST` | Mixed text styles were flattened | review |
| `F2P-VECTOR-EMPTY` | Empty vector skipped | not_converted |
| `F2P-VECTOR-FAILED` | Vector couldn’t be converted | attention |
| `F2P-TEXT-EMPTY` | Empty text skipped | not_converted |
| unknown | Something changed | review |

The advice text for each is written for designers, not auditors. Raw converter descriptions
remain under **Show details**. Unknown codes fall back to **Something changed** and stay in
the **Slightly different** group; they are never dropped.

Tests: `copy::tests::every_issue_code_has_designer_copy`,
`every_status_outcome_severity_and_validation_maps`, and
`raw_worker_errors_get_designer_copy`.

## 8. Accessibility checklist

Status vocabulary follows `AGENTS.md`. Nothing below is marked **Verified** without a
recorded check.

| Check | Status | Evidence / limit |
|---|---|---|
| Status uses text + icon + colour, never colour alone | **Verified** | Screenshots in `docs/ui/screenshots/`; chips and banners pair icon and label. |
| WCAG AA contrast (body 4.5:1, large 3:1) | **Verified (calculated)** | Light pairs 5.27–16.97:1; dark pairs 7.02–16.25:1. Border-only elements are decorative. |
| Keyboard navigation for nav, buttons, lists, dialogs, report controls | **Implemented (unverified)** | egui focus/tab support and Cmd/Ctrl shortcuts; a full keyboard-only pass is still manual. |
| Visible focus ring; no focus traps | **Implemented (unverified)** | egui default focus visuals; visual check pending. |
| Programmatic labels / screen-reader support | **Not verified** | The current eframe build does not enable AccessKit; icon-only controls do have tooltips. |
| Minimum 32 px control height | **Verified** | Theme interact size 34 px; primary/secondary buttons 36 px. |
| Layouts survive 40% longer strings; middle truncation + full-path tooltip | **Partial** | Truncation and wrapping are used; no pseudo-localization or RTL pass. |
| Reduce motion | **Not applicable / unverified** | No custom looping or essential motion; egui defaults are not audited against the OS setting. |
| Dialogs dismissible with Esc; destructive actions ask for confirmation | **Implemented (unverified)** | Clear-all and reset-preference modals handle Esc; key interaction not manually tested. |
| Empty/loading/partial/error states use text, not colour or animation alone | **Verified** | Empty Files, batch error, partial report, and in-progress screenshots. |
| Screen-reader verification | **Not verified** | Requires an AccessKit-enabled build and a manual VoiceOver/NVDA pass. |

## 9. Discrepancies / decisions

- **No thumbnails in file rows** until the reader exposes a verified thumbnail path; use a
  neutral file icon, never a fake preview. Record in `DECISIONS/0010-ui-redesign.md`.
- **No page-selection / page-splitting / report-format UI** until the engine supports them;
  do not present dead controls.
- **Fidelity choice** maps to a real engine option: *Keeps tokens connected* preserves
  token/style links; *Looks identical* keeps resolved values but removes those links.
  **Partial:** single-file conversion honors the option, but `conversion-queue::run_job`
  still uses `ConvertOptions::default()`, ignoring `JobSpec.keep_token_bindings`. Batch
  preference propagation needs a regression test and fix; it is not verified end to end.
- **Concurrency** stays advanced and bounded; default 2.
- **Jump-to-element** is a report breadcrumb plus *Copy layer name/ID* until a canvas viewer
  exists; no dead “jump” button.
- **Component grouping** is omitted: report events do not carry a reliable main-component
  identity. Grouping by **What to do**, **Page**, or **None** is real; a component selector
  would be invented.
- **Languages**: UI is English and centralized for i18n; no fake language switcher.
