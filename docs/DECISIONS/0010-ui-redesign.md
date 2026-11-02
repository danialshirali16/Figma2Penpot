# 0010 — Designer-first UI redesign

Date: 2026-11-02
Status: Accepted

## Context

The M4 shell worked but spoke like the engine: issue codes, node IDs, validation
states, and a report organized for auditors. The primary users are product designers
migrating real work. They need to know **“did my design survive, and what do I need to
check?”** without learning the converter's vocabulary. The conversion engine and its
event-based report are already honest; the redesign is a presentation layer over the same
data.

The redesign must stay pure Rust, offline, and native. It must not invent thumbnails,
counts, progress, or success states.

## Decisions

1. **Design contract first.** `docs/UI_SPEC.md` and `docs/ui/WIREFRAMES.md` were written
   and committed before UI code (`914daee`). Screens use only `theme.rs` tokens.
2. **Bundled Inter variable font.** Inter is licensed under SIL OFL 1.1; the file and
   license are committed under `crates/desktop-app/assets/fonts/`. No system-font lookup.
3. **Project-owned line icons.** Icons are painted in `components.rs`; no third-party icon
   font or set is bundled.
4. **No fake thumbnails.** The reader does not expose a verified thumbnail path, so file
   rows use a neutral file icon. A placeholder that looks like a preview would be invented
   data.
5. **No dead controls.** Page selection, page splitting, report-format selection inside
   Settings, and a language switcher are omitted until they exist. “Keep both” is shown
   disabled with an explicit note rather than silently behaving like Replace.
6. **“What matters more?” is a real engine option.** `ConvertOptions.keep_token_bindings`
   drives token/style link preservation. “Looks identical” keeps resolved values and
   removes links; “Keeps tokens connected” preserves links where Penpot supports them. The
   option is propagated through the queue and the CLI (`--inline-tokens`).
7. **Report defaults to what to do.** Groups are Needs your attention / Not converted /
   Slightly different / All good. Filters are search, severity, result, and grouping. Raw
   codes, node IDs, and converter messages stay under “Show details”.
8. **Cooperative cancellation and pause.** Single-file and batch workers check a control
   token between stages. Cancelling before saving leaves no output. Cancellation during an
   atomic write is still not interruptible (see `SUPPORT_MATRIX.md`).
9. **Preview hooks are developer-only.** Environment variables in `app.rs`
   (`F2P_PREVIEW_*`, `F2P_GALLERY`) stage screens for screenshots and visual QA. They are
   not exposed in the UI and do not fabricate conversion data; batch previews run the real
   engine on real fixtures.
10. **Friendly errors, raw details retained.** `copy::failure_message` maps known worker
    failures to designer language. The raw message remains available behind “Show details”
    and in reports/exports.

## Consequences

- Every internal status, outcome, severity, validation state, stage, queue state, and known
  issue code has a tested presentation mapping in `copy.rs`.
- The report remains event-based; grouping is presentation only and keeps element-level
  traceability.
- The UI can be reviewed headlessly, and screenshots can be regenerated with documented
  environment variables.
- Native dialogs, drag-and-drop, screen-reader output, and Windows/Linux visuals still need
  manual verification; they are labeled unverified in `docs/MANUAL_QA.md`.
- Thumbnails require a future reader change and must be verified before they appear.
