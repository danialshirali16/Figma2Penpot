# AGENTS.md — Figma 2 Penpot

Operating manual for AI coding agents working in this repository. Humans are welcome to read it too.

The full product brief lives in `docs/PROJECT_BRIEF.md`. This file is the short, always-applicable version: what the project is, how to build and test it, and the rules you must not break. If this file and reality disagree, reality wins; fix this file in the same commit.

---

## 1. What this project is

**Figma 2 Penpot** is an open-source native desktop app (Windows + macOS), written entirely in Rust, that converts Figma `.fig` files into Penpot `.penpot` files. It supports batch conversion and produces a detailed, event-based Conversion Report per file plus a batch summary.

Tagline: *"Move your designs from Figma to Penpot. Simply."*

## 2. Hard rules (never violate)

1. **Pure Rust.** No Node.js, no JS/TS engine or runtime, no subprocess calls to the original converter, no Electron/webview/browser UI, no backend server, no cloud conversion.
2. **Local-first.** No telemetry, uploads, accounts, or network access during conversion. Network features must be opt-in and are out of scope for v1.
3. **Honest reporting.** The report is built from events recorded *while converting*, never inferred from the output afterwards. Never silently drop, flatten, approximate, or substitute anything. Every such decision emits a diagnostic.
4. **Never fabricate.** Do not invent test results, counts, fidelity numbers, or feature status. If you did not run it, say it is unverified. If the parser cannot determine a value, show it as *unknown*.
5. **Source files are read-only.** Write outputs to a temp file on the destination volume, validate, then atomically rename. Never present incomplete output as a success.
6. **Licenses first.** Do not copy or port code from `girafic/penpot-converter`, the Penpot exporter, or any dependency until `docs/LICENSE_ASSESSMENT.md` says it is allowed. If unclear, reimplement from documented behavior and keep attribution.
7. **One failing file never stops a batch.** Errors and panics are contained per job.
8. **Untrusted input.** `.fig` files are untrusted. Parsers must never panic, never allocate unbounded memory from file-supplied lengths, and must enforce archive limits (see §7).

## 3. Repository layout

```text
crates/
  fig-reader/          Container detection, safe extraction, schema + data decoding → raw node tree
  conversion-core/     Internal IR, mapping passes, conversion logic (no UI dependency)
  penpot-writer/       Penpot document model and .penpot serialization + structural validator
  conversion-report/   ReportEvent model, DiagnosticSink, JSON/HTML/CSV exporters
  conversion-queue/    Batch jobs, bounded concurrency, progress events, cancellation, retry
  f2p-cli/             Thin headless CLI over the same APIs (inspect, convert, batch)
  desktop-app/         egui/eframe application
tests/                 fixtures/, integration/, golden/
docs/                  ARCHITECTURE, FIG_FORMAT, PENPOT_FORMAT, LICENSE_ASSESSMENT,
                       SUPPORT_MATRIX, ISSUE_CODES, DEV_ENVIRONMENT, MANUAL_QA, DECISIONS/
PROGRESS.md            Living status file (update every session)
```

Dependency direction: `desktop-app` → `conversion-queue` → `conversion-core` → (`fig-reader`, `penpot-writer`, `conversion-report`). The core must never depend on the UI.

> Not every crate may exist yet. Check `PROGRESS.md` for the current state before assuming a path exists.

## 4. Commands

Run these from the repo root once the workspace exists. If a command does not exist or fails for a reason unrelated to your change, report that rather than hiding it.

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check            # licenses and advisories
cargo audit                 # known vulnerabilities

cargo run -p f2p-cli -- inspect  path/to/file.fig
cargo run -p f2p-cli -- convert  path/to/file.fig --out out/
cargo run -p desktop-app    # GUI (needs a display)
```

Snapshot tests use `insta`: review changes with `cargo insta review`; never blindly accept snapshots.

Use the stable toolchain. Pin it in `rust-toolchain.toml`.

## 5. Environment awareness

You may be running on Linux with no display, no Windows/macOS, possibly no internet or Docker. At the start of a task:

- Check what is available and record it in `docs/DEV_ENVIRONMENT.md` if it is not already there.
- Keep conversion logic and UI *logic* testable headlessly.
- Anything you cannot run here (GUI on Windows/macOS, installers, real Penpot import) must be labeled **unverified** or **pending**, with exact manual steps for a human, and covered by CI where possible.

## 6. Workflow

1. **Read first.** Read `PROGRESS.md`, then the docs relevant to your task, before changing code.
2. **Work in small vertical slices.** Prefer getting an end-to-end path working over building large isolated subsystems.
3. **Test with the change.** New behavior needs unit tests; conversion behavior needs a golden/integration test; every unsupported or approximated feature needs a test asserting its diagnostic.
4. **Run the gates** (§8) before declaring anything done.
5. **Decide and record.** When you make a non-obvious technical choice, add `docs/DECISIONS/NNNN-title.md` (context → options → decision → consequences).
6. **Update docs.** Keep `docs/SUPPORT_MATRIX.md`, `docs/ISSUE_CODES.md`, and `PROGRESS.md` accurate.
7. **Commit small and clearly.** One logical change per commit; imperative, descriptive messages.

Ask the user only when blocked on something you cannot resolve through research, a sensible default, or a recorded decision.

## 7. Coding conventions

**General**
- Edition 2021+ (or newer stable), `#![forbid(unsafe_code)]` in all crates unless an exception is documented with justification and tests.
- Errors: `thiserror` in libraries, `anyhow` only in binaries. Return errors; do not `unwrap`/`expect`/index-panic on paths that touch untrusted data. `expect` with a clear invariant message is acceptable for true internal invariants.
- Logging: `tracing` with structured fields. Never log design contents at `info` or above; no telemetry.
- Public APIs get doc comments. Keep modules small and focused.
- Prefer pure-Rust dependencies. Any C-binding dependency needs a written justification (cross-compilation, licensing, packaging impact).
- Add dependencies deliberately: check maintenance, license (must pass `cargo deny`), and size.

**Parsing and security** (defaults live as named constants in one place, configurable, and documented)
- Enforce limits on total uncompressed size, entry count, compression ratio, nesting depth, and node count.
- Reject absolute paths, `..` components, and symlinks in archives (zip-slip).
- Treat all length/count fields from files as hostile; cap before allocating.
- Unknown fields, node types, or schema versions produce a diagnostic, not a crash.
- Parsers get property tests (`proptest`) and a `cargo-fuzz` target.

**Conversion and reporting**
- Converters receive a `DiagnosticSink`; every conversion decision (converted, approximated, substituted, skipped, failed) emits a `ReportEvent` with: stable issue code, severity, outcome, element reference (Figma node ID, name, type, page, path), description, reason, action taken, suggested manual action.
- Report counts come from recorded events and a real traversal. Grouping repeated issues for display must not lose element-level traceability.
- If validity or fidelity cannot be confirmed, record that uncertainty (e.g. validation state `NotRun`), not success.
- Documented fallback for every Figma concept with no exact Penpot equivalent. Do not rasterize or outline complex objects unless it is the only option, and report it when you do.
- Fidelity priority: structure → visual appearance → components/instances → layout behavior → token relationships → editability.

**Concurrency**
- CPU-bound work on a bounded worker pool; `tokio` limited to orchestration/IO.
- UI communicates with the queue via channels; no UI-thread blocking.
- Each job has a cancellation token and runs under `catch_unwind` (or equivalent) so a failure marks only that job.
- Output goes to temp files on the destination volume, then atomic rename. Clean stale temp files on startup.

**UI**
- Never show fabricated progress percentages. Show real stages; determinate progress only when the true denominator is known.
- Status is never conveyed by color alone.
- Keep user-visible strings centralized so localization is possible later.
- Handle long names/paths (middle truncation + tooltip). Use native dialogs and platform-appropriate shortcuts.

## 8. Quality gates (a change is not done until these hold)

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes (report actual results)
- [ ] `cargo deny check` is clean, or exceptions are documented
- [ ] New or changed behavior has tests; new issue codes are in `docs/ISSUE_CODES.md`
- [ ] `docs/SUPPORT_MATRIX.md` reflects reality using the labels below
- [ ] `PROGRESS.md` updated

## 9. Support status labels

Use exactly these labels in docs, README, code comments, and reports:

| Label | Meaning |
|---|---|
| **Verified** | Implemented, tested, and confirmed (ideally by importing into Penpot) |
| **Implemented (unverified)** | Implemented and unit-tested, not confirmed in Penpot |
| **Partial** | Some cases handled; limits documented |
| **Planned** | Not started |
| **Not supported** | Out of reach or scope; reported at runtime |

Never mark a feature Verified without recorded evidence.

## 10. Fixtures and test data

- Only commit fixtures you have the right to redistribute. Record source and license next to each fixture (`tests/fixtures/README.md`).
- Prefer small, synthetic `.fig` files built to isolate one feature.
- Never commit private, client, or proprietary designs.
- Golden outputs are reviewed by hand when they change; explain why in the commit message.

## 11. Definition of done for a task

State, concisely and factually:

1. **Done:** what changed, with the commands you ran and their real output.
2. **Verified vs. unverified:** using the labels in §9.
3. **Decisions:** links to any `docs/DECISIONS/` entries.
4. **Blockers or open questions:** only things that truly need a human.
5. **Next steps:** the next few concrete actions.

## 12. Do not

- Add telemetry, analytics, auto-update pings, or any network call in the conversion path.
- Wrap, shell out to, or embed the original JS/TS converter.
- Silently catch and ignore errors, or swallow diagnostics.
- Mark partial work as complete or unsupported features as supported.
- Rewrite or reformat unrelated code in a feature commit.
- Commit build artifacts, secrets, or fixtures with unclear licenses.
- Edit `Cargo.lock` by hand.

## 13. Reference links

- Original repository: https://github.com/girafic/penpot-converter
- Original community discussion: https://community.penpot.app/t/migrate-figma-components-to-penpot/10870/5
- Official Penpot exporter: https://github.com/penpot/penpot-exporter-figma-plugin
- Penpot migration guide: https://help.penpot.app/user-guide/first-steps/migration-guide/

Treat these as research input. The original is JS/TS-based; reimplement in Rust, and honor license and attribution requirements.
