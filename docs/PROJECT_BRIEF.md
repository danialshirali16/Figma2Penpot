# Project Brief (working restatement)

This file captures the requirements the repository is built against. It is a structured
restatement, not a verbatim copy of the original brief. The detailed, maintained sources
are:

- Product and principles: [`../README.md`](../README.md)
- Sequence and gates: [`../Roadmap.md`](../Roadmap.md)
- System design: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- Formats: [`FIG_FORMAT.md`](FIG_FORMAT.md), [`PENPOT_FORMAT.md`](PENPOT_FORMAT.md)
- Legal: [`LICENSE_ASSESSMENT.md`](LICENSE_ASSESSMENT.md)
- Current status: [`../PROGRESS.md`](../PROGRESS.md)

If this file and the maintained sources disagree, the maintained sources win; fix this file.

Reviewed **2026-11-02**. This is the product requirements document, not a claim that every
listed feature is implemented. Current source imports all five private compatibility files
into Penpot 2.18.5-RC1; overall fidelity remains **Partial**. Page selection/splitting,
complete component behavior, exhaustive visual/persistence checks, and signed installers
remain outside the verified implementation. See `SUPPORT_MATRIX.md` and ADR-0011.

---

## 1. Mission

Make migrating from Figma to Penpot as simple as: **select files → Convert → review the
report.** No terminal, no Node.js, no scripts, no cloud, no account.

Tagline: *"Move your designs from Figma to Penpot. Simply."*

## 2. Non-negotiables

1. **Pure Rust.** No Node.js, JS engine, embedded browser/webview UI, subprocess converter,
   backend, or cloud conversion.
2. **Local-first and offline.** No telemetry, uploads, accounts, or network access during
   conversion. Future network features must be opt-in.
3. **Honest reporting.** Report events are recorded during conversion, never inferred from
   output. Nothing is silently dropped, flattened, approximated, or substituted.
4. **Never fabricate.** No invented results, counts, fidelity numbers, or feature status.
   Undeterminable values are shown as *Unknown*.
5. **Source files are read-only.** Output goes to a temp file, is validated, then atomically
   renamed. Incomplete output is never presented as success.
6. **Licenses first.** No copying or porting until the license assessment allows it.
7. **One failing file never stops a batch.** Errors and panics are contained per job.
8. **Untrusted input.** Parsers never panic, never allocate unboundedly from file-supplied
   lengths, and enforce archive limits.

## 3. Product scope

**Input:** `.fig` (Figma design files). **Output:** `.penpot` plus a Conversion Report.

**Platforms:** Windows 10/11 x64; macOS Apple Silicon required, Intel where practical.

**User journey:** Import → Analyze → Configure → Convert → Review → Export.

- Import: drag and drop, multi-file/folder pickers, queue management, per-file validation.
- Analyze: pages, frames, layers, components, instances, variants, variables, styles,
  images, and likely compatibility issues. Unknown values shown as *Unknown*.
- Configure: output directory, naming, overwrite policy, page selection,
  fidelity-vs-token-binding preference, page splitting, concurrency, validation, report
  format. Sensible defaults; advanced options behind progressive disclosure.
- Convert: real stages (`Reading → Decoding → Mapping → Writing → Validating`); no
  fabricated percentages.
- Review: completed / completed with warnings / failed, with links to outputs and reports.
- Export: JSON, standalone HTML, CSV; open output folder.

**Batch:** one `.penpot` and one report per input, plus a batch summary. Bounded
concurrency, pause scheduling, cancel pending/active, retry one, retry failed only, remove
completed, per-file results. Statuses: `Queued → Analyzing → Converting → Validating →
Completed | Completed with Warnings | Failed | Cancelled`. Cancellation leaves no partial
output that looks valid. Memory does not scale linearly with batch size.

**Report categories:** Successfully Converted · Converted with Modifications · Not Converted ·
Warnings · Errors. Each issue includes severity, stable issue code, element type/name,
Figma node ID, page, layer/component path, description, reason, action taken, and suggested
manual action. Grouping is allowed; element-level traceability is not lost.

## 4. Non-goals for v1

Penpot → Figma; Figma REST/API fetching; Figma or Penpot plugins; online features;
real-time sync; editing designs in the app; automatic repair beyond conversion needs.

## 5. Architecture

```text
.fig → fig-reader → raw node tree → conversion-core (IR + passes)
     → penpot-writer → .penpot archive → validator → atomic write
     → conversion-report (per-file + batch)
```

Crates: `fig-reader`, `conversion-core`, `penpot-writer`, `conversion-report`,
`conversion-queue`, `f2p-cli`, `desktop-app`. The core never depends on the UI. A headless
CLI is required for integration tests and CI; the GUI is the primary product.

Fidelity priority: document structure → visual appearance → components/instances → layout
behavior → design-token relationships → editability in Penpot.

## 6. Milestones

| ID | Scope | Gate |
|---|---|---|
| M0 | Research and legal baseline | Docs with citations, explicit unknowns, license decision |
| M1 | Workspace and `.fig` reading | `inspect` works on real fixtures; hostile inputs error without panic; fuzz target |
| M2 | Minimal end-to-end conversion | Valid `.penpot`, golden report, validator, real import attempted |
| M3 | Fidelity expansion | Per-feature tests, goldens, diagnostics, support-matrix rows |
| M4 | Desktop application | Eight screens, headless-tested UI logic, light/dark, non-blocking |
| M5 | Batch conversion | Mixed-batch isolation, clean cancellation, bounded memory |
| M6 | Report UI and exports | Filterable report, JSON/HTML/CSV snapshot tests, issue-code docs |
| M7 | Reliability and performance | Fuzzing, large-file corpus, recorded benchmarks |
| M8 | Release engineering | CI artifacts for Windows/macOS, docs, notices, signing plan |

Milestones are gates, not dates. A milestone is done only with recorded evidence.

## 7. Quality gates

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo deny check` / `cargo audit` clean or documented exceptions
- New behavior has tests; every unsupported/approximated feature asserts its diagnostic
- `docs/SUPPORT_MATRIX.md` reflects reality
- `PROGRESS.md` updated

## 8. Support vocabulary

**Verified** · **Implemented (unverified)** · **Partial** · **Planned** ·
**Not supported**. No other status words.

## 9. Definition of done for v1

Installs without dev tools; fully native Rust UI; drag and drop; inspect/analyze; valid
conversion; resilient batches; accurate event-based reports; JSON/HTML/CSV export; no
JavaScript anywhere; offline; representative files imported and checked in a real Penpot
instance with evidence; complete tests, docs, and license compliance.

Partial work is labeled partial. Unsupported features are never marked done.
