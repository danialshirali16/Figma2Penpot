# UI Review — M4.2 / M6.2

Date: 2026-11-02
Scope: designer-facing redesign of the eight desktop screens, the conversion report, and
the component gallery.
Method: build and launch the native app on macOS, capture light/dark at large and compact
window sizes, read every screen cold, and inspect the screenshots against
`docs/UI_SPEC.md`. This is an **internal heuristic review**, not a usability study with
external participants.

Status vocabulary follows `AGENTS.md`: **Verified**, **Implemented (unverified)**,
**Partial**, **Planned**, **Not supported**.

## 1. Evidence

All captures are in [`screenshots/`](screenshots/). They were taken at Retina resolution
and scaled to 1440 px wide.

| Screen | Light large | Dark large | Light compact | Dark compact |
|---|---|---|---|---|
| Home / Import | [link](screenshots/home-light-large.png) | [link](screenshots/home-dark-large.png) | [link](screenshots/home-light-compact.png) | [link](screenshots/home-dark-compact.png) |
| Files | [link](screenshots/files-light-large.png) | [link](screenshots/files-dark-large.png) | [link](screenshots/files-light-compact.png) | [link](screenshots/files-dark-compact.png) |
| File details | [link](screenshots/details-light-large.png) | [link](screenshots/details-dark-large.png) | [link](screenshots/details-light-compact.png) | [link](screenshots/details-dark-compact.png) |
| Settings | [link](screenshots/settings-light-large.png) | [link](screenshots/settings-dark-large.png) | [link](screenshots/settings-light-compact.png) | [link](screenshots/settings-dark-compact.png) |
| Converting | [link](screenshots/converting-light-large.png) | [link](screenshots/converting-dark-large.png) | [link](screenshots/converting-light-compact.png) | [link](screenshots/converting-dark-compact.png) |
| Results | [link](screenshots/results-light-large.png) | [link](screenshots/results-dark-large.png) | [link](screenshots/results-light-compact.png) | [link](screenshots/results-dark-compact.png) |
| Report | [link](screenshots/report-light-large.png) | [link](screenshots/report-dark-large.png) | [link](screenshots/report-light-compact.png) | [link](screenshots/report-dark-compact.png) |
| Preferences | [link](screenshots/preferences-light-large.png) | [link](screenshots/preferences-dark-large.png) | [link](screenshots/preferences-light-compact.png) | [link](screenshots/preferences-dark-compact.png) |

Additional states:

- [Component gallery, light](screenshots/gallery-light-large.png) ·
  [dark](screenshots/gallery-dark-large.png)
- [Empty Files](screenshots/files-empty-light-large.png)
- [Error state: one converted, one failed](screenshots/results-error-light-large.png)
- [License modal](screenshots/licenses-light-large.png)
- [Clear-all confirmation](screenshots/clear-light-large.png)

The batch Results captures are real conversions of four fixtures, not staged rows. The
report capture is the real report generated from `Masks.fig`.

## 2. Five-second test

Read each screen cold and answer: *what is this, and what do I do?*

| Screen | First read | Result |
|---|---|---|
| Home | “Drop files or choose them; nothing leaves my computer.” | **Pass.** One dominant drop target; privacy line inside it. |
| Files | “These files are ready; Convert saves them there.” | **Pass.** Count, destination, and one primary button are visible without scrolling. |
| File details | “This file has 3 components; nothing unusual; convert it.” | **Pass.** Counts are the first thing after the title; warnings come before the action. |
| Settings | “Choose output; choose looks vs tokens; Done.” | **Pass.** Two choices with plain explanations; Advanced collapsed. |
| Converting | “It is converting layers; I can pause or cancel.” | **Pass after fix.** Single-file mode previously had no controls; both are now present. |
| Results | “4 converted, 1 needs review; open the folder or the report.” | **Pass after fix.** The batch line previously omitted review count. |
| Report | “2 things are slightly different; here is what to do.” | **Pass after fix.** The default group answers “what to do”, not “which code”. |
| Preferences | “Theme, output folder, privacy, licenses.” | **Pass after fix.** Licenses was a dead label; it now opens the OFL text. |

The first useful action is one click from launch (**Choose files…** or **Choose a
folder…**). Converting is one click from Files (**Convert**). No setup wizard exists.

## 3. Nielsen heuristics

| # | Heuristic | Finding | Action |
|---|---|---|---|
| 1 | Visibility of system status | Batch Results said “3 converted” without saying one needed review; a single conversion had no cancel control. | Batch summary now includes review and cancelled counts; single-file Converting has Pause and Cancel; every stage is named. |
| 2 | Match between system and real world | Validation used “Checked and ready”, which read like a clean result even when warnings existed. | Changed to “File check passed”; raw validation state stays in the report data. |
| 3 | User control and freedom | Clear all removed the queue instantly; reset preferences was one click. | Clear-all and reset-preference confirmations; Esc/backdrop dismiss; cancelled conversions leave no output. |
| 4 | Consistency and standards | File rows, result rows, settings cards, and report cards each clipped right-aligned content differently because frames re-added margins. | Added explicit content widths and a 32 px page gutter; verified at both sizes. |
| 5 | Error prevention | Overwrite policy was a bare checkbox; “Keep both” did not exist. | Advanced now says **Ask me / Replace** and shows **Keep both** disabled with an explanation. |
| 6 | Recognition rather than recall | Report filters showed two unlabelled “All” dropdowns. | Added **Severity** and **Result** labels; grouping is explicit. |
| 7 | Flexibility and efficiency | Export only offered HTML/CSV; batch could not retry failed files from Results. | Added JSON export, Retry failed, per-file View report, search, result filter, and copy actions. |
| 8 | Aesthetic and minimalist design | Count cards and the report card overflowed their columns; empty Files had two competing add actions. | Equal-width metric cards; full-width report cards; empty state has one centered choice pair. |
| 9 | Help users recognize and recover from errors | Batch failures exposed raw worker text and absolute paths as the first message. | `copy::failure_message` gives what happened + what to do; raw text stays under Show details. |
| 10 | Help and documentation | The import guide names the exact Penpot path; no in-app manual is needed for the main flow. | Kept the verified import guide; licenses and privacy are available offline. |

## 4. Fixes made during review

1. Removed the duplicate **Open folder** action from single-result rows; the batch action
   remains primary.
2. Fixed right-edge clipping by adding a 32 px page gutter and explicit frame content
   widths. This affected Files rows, Settings, details metrics, Results rows, report cards,
   and the compact report toolbar.
3. Added **Pause** and **Cancel** to single-file conversion; cancellation is cooperative and
   checked between stages, and leaves no output when it happens before saving.
4. Added `ConvertOptions.keep_token_bindings`, `JobSpec`, and CLI `--inline-tokens`.
   **Correction, 2026-11-02:** the queue stores the option but `run_job` still passes
   defaults into the converter. Single-file propagation works; batch propagation is
   **Partial** and requires a regression test and fix.
5. Fixed the single-file path to honor `keep_token_bindings` (it previously used defaults).
6. Added a confirmation modal for **Clear all**; removed the dead `Clear all` behavior on
   empty Files.
7. Added **Licenses** as an offline modal and a reset-preference confirmation.
8. Mapped known worker errors to designer copy; raw messages are retained under
   **Show details**.
9. Changed the report action layout so **Copy advice** is not clipped; it sits with the
   advice block instead of the title row.
10. Made compact Home, Settings, and Report responsive: shorter hero/drop zone, two-row
    report filters, stacked empty-state actions.

## 5. Remaining limitations (honest)

- **Native dialogs and drag-and-drop** are implemented but the OS interaction was not
  automated. Label: **Implemented (unverified)**. Manual steps are in `docs/MANUAL_QA.md`.
- **Keyboard-only traversal** has headless shortcut tests, but the full tab/focus pass is
  **Implemented (unverified)**.
- **Screen reader / AccessKit** is **Not verified**. The current eframe feature set does not
  enable AccessKit.
- **Windows and Linux** visuals are **Not verified**; screenshots are macOS only.
- **No thumbnails.** File rows use a neutral icon until the reader exposes a verified
  thumbnail path.
- **Cancellation during an atomic write** is not interruptible; it is checked before the
  write. The queue and writer remain safe, but the mid-write race is untested.
- **No component grouping** in the report: events do not carry a reliable main-component
  identity. Grouping by what to do, page, or none is real.
- **No jump-to-element** without a canvas viewer; the report shows the layer path and offers
  copy actions instead.
- **No page selection or page splitting**; the engine does not support it yet, so no dead
  controls are shown.
- **40% longer strings / RTL** are not pseudo-localized; wrapping and truncation exist but
  the layout was not tested with an RTL locale.
- **No external usability participants.** The five-second test is an internal read-through.

## 6. State coverage

| State | Where it exists |
|---|---|
| Empty | Home (no files), Files (no files), Report (no filter matches), Results (no run) |
| Loading | Analyzing a file, Converting stages, batch progress |
| Success | Results all converted, report All good, Settings saved |
| Partial | Report “Slightly different” and “Not converted” groups, validation not run |
| Error | Invalid file row, unreadable analysis, failed conversion row, failed export notice |
| Cancelled | Queue status, batch summary, cancellation notice |

Screenshots cover the empty, success, partial, loading, and error families. The cancelled
result is covered by state tests and the status mapping, not by a staged screenshot. The
error capture shows a friendly first message with the raw worker text behind **Show
details**; the title changes to **Your conversion finished — some files need attention**.
