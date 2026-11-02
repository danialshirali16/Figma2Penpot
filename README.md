# Figma 2 Penpot

**Move your designs from Figma to Penpot. Simply.**

[![CI](https://github.com/danialshirali16/Figma2Penpot/actions/workflows/ci.yml/badge.svg)](https://github.com/danialshirali16/Figma2Penpot/actions/workflows/ci.yml)

A native, offline, open-source desktop application for Windows and macOS that converts Figma
`.fig` files into Penpot `.penpot` files, in batches, with a conversion report you can trust.

---

> ## Status: pre-alpha — source compatibility fixes; published binaries are `v0.1.1`
>
> The Rust workspace reads real `.fig` files and converts the M2 slice plus vectors, masks,
> boolean fallbacks, gradients, shadows, blur, image fills, text metrics, flex containers,
> styles, components/variants, and design tokens to `.penpot`. It validates the archive and
> writes an event-based JSON report. M2–M3.8 outputs imported into a local **Penpot 2.18.5**
> stack and were inspected. The native desktop app now has a designer-first UI: eight
> screens, plain-language conversion report, light/dark themes, native file dialogs,
> background conversion, pause/cancel, and headless UI tests. Batch retry/remove controls,
> memory benchmarks, macOS DMG packaging, and self-contained HTML/CSV/JSON report exports
> are implemented. Native dialog, drag-drop, keyboard-only, screen-reader, and
> Windows/Linux visual checks remain unverified. Fuzz and release benchmark evidence
> is recorded in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md), with a checked-in fixture
> baseline under `tests/benchmarks/`. Stroke-only vectors, editable network handles, fixed constraints, Auto Layout child
> sizing, effect style linkage, instance swaps, mixed text runs, and non-paint variable
> bindings are documented Partial items with runtime diagnostics. See
> [`PROGRESS.md`](PROGRESS.md) and [`docs/MANUAL_QA.md`](docs/MANUAL_QA.md).

As of **2026-11-02**, five privately supplied version 106/110 Figma files convert and
import into local **Penpot 2.18.5-RC1**, with zero failed elements and explicit warnings.
Import compatibility is **Verified**; overall fidelity is **Partial**. These fixes restore
section contents and referenced embedded components, resolve nested override paths, and
handle target layout, token-name, paint-order, and blend constraints. Full visual comparison
and edit/reopen persistence remain pending. Private files and reports are not distributed.
See [ADR-0011](docs/DECISIONS/0011-private-file-compatibility.md).

The compatibility changes are in source and are **not included in the existing `v0.1.1`
downloads**. Build this revision to use them; no new release has been published for them.

---

## Why this exists

Migrating a design file between tools is not a file-format problem alone. It is a
*trust* problem. Existing routes to Penpot require either running Figma's plugin inside a
Figma account, or installing Node.js and a command-line converter. Neither tells you, in
detail, what survived the trip and what did not.

Figma 2 Penpot is built around three beliefs:

1. **No terminal, no account, no cloud.** Select files, press Convert, read the report.
2. **A conversion is only as good as its disclosure.** Silent approximations are bugs.
3. **Native software should be small, fast, and private.** One Rust binary, no embedded
   browser, no JavaScript runtime, no network calls during conversion.

## Current workflow

| Step | What you get |
|---|---|
| **Import** | Drag and drop, multi-file and folder pickers, recursive folder scan, and a file queue; native interactions still need platform QA. |
| **Analyze** | Pages, frames, layers, components, instances, variants, variables, styles, images, and likely compatibility problems — with `Unknown` shown honestly instead of guessed. |
| **Configure** | Output location, overwrite behavior, and bounded concurrency. The token preference works for single-file conversion; batch propagation remains a known gap. Page selection and splitting are Planned. |
| **Convert** | Real stages (`Reading → Converting → Writing → Validating → Saving`). No invented percentages. |
| **Review** | Per-file results, a combined batch summary, and a report that links every issue to a node, page, and layer path. |
| **Export** | Reports as JSON, standalone HTML, or CSV. Open the output folder. |

One `.fig` in, one `.penpot` plus one report out. A batch produces a combined summary as
well. One failing file never stops the rest.

## The Conversion Report is the product

The report is generated from events emitted **during** conversion, not reconstructed from
the output afterwards. Every transformation decision — converted, approximated,
substituted, skipped, failed — is recorded at the point where it happens.

Illustrative entry using a supported issue code (not a private-file report):

> **F2P-INSTANCE-ATTRIBUTES-UNSUPPORTED · Component settings need checking** — *Warning*, converted with modification
> **Element:** Primary Button (`12:340`) · **Page:** Components · **Path:** Components / Buttons / Primary
> **Problem:** Some instance attributes have no mapping yet.
> **Result:** Supported attributes were applied; other attributes retain component defaults.
> **Suggested action:** Review the listed settings against the original design.

Repeated issues may be grouped for readability, but element-level traceability is always
preserved. If output could not be validated, the report says so.

## Non-negotiables

| # | Rule |
|---|---|
| 1 | **Pure Rust.** No Node.js, no JS engine, no embedded browser or webview UI, no subprocess to another converter, no backend. |
| 2 | **Local-first and offline.** No telemetry, no uploads, no account. Conversion never touches the network. |
| 3 | **Honest reporting.** Nothing is dropped, flattened, approximated, or substituted without a recorded event. |
| 4 | **Never fabricate.** No invented counts, fidelity numbers, or test results. Undeterminable values are shown as *Unknown*. |
| 5 | **Source files are read-only.** Output is written atomically: temp file → validate → rename. |
| 6 | **Licensing first.** No code is copied or ported until its license is resolved. |
| 7 | **Batch resilience.** A panic or error in one job is contained to that job. |

## Support status vocabulary

Used identically in the app, the reports, and the documentation. There are no other
status words.

| Label | Meaning |
|---|---|
| **Verified** | Implemented, tested, and confirmed — ideally by importing into a real Penpot instance. |
| **Implemented (unverified)** | Implemented and unit-tested, not yet confirmed in Penpot. |
| **Partial** | Some cases handled; limits documented. |
| **Planned** | Not started. |
| **Not supported** | Out of reach or out of scope, and reported as such at runtime. |

The authoritative per-feature list is [`docs/SUPPORT_MATRIX.md`](docs/SUPPORT_MATRIX.md).
It covers all implemented reader, converter, queue, report, and desktop features, including
the remaining Partial behavior. Import acceptance alone never establishes lossless fidelity.

## Project status at a glance

| Milestone | Scope | Status |
|---|---|---|
| M0 | Research, format findings, license assessment, architecture | **Verified** — recorded baseline and decisions; release compliance review pending |
| M1 | Cargo workspace, `.fig` container/decode, `f2p-cli inspect` | **Verified** — fixture, hostile-input, fuzz, and historical CI evidence in `PROGRESS.md` |
| M2 | Minimal end-to-end conversion + validator + JSON report | **Verified** — imported and inspected in Penpot 2.18.5-RC1 |
| M3 | Fidelity expansion (vectors → effects → images → text → layout → styles → components → tokens) | **Partial** — all eight waves implemented; sub-feature losses remain reported |
| M4 | Native desktop application (eight screens) | **Partial** — designer UI; native dialog/drag-drop/keyboard/manual checks pending |
| M5 | Batch conversion (bounded concurrency, pause, cancel, retry) | **Partial** — queue core, CLI batch, desktop Convert all; mid-write cancel pending |
| M6 | Report UI and JSON/HTML/CSV exports | **Partial** — exports verified; full report interaction/platform QA pending |
| M7 | Reliability, fuzzing, performance benchmarks | **Partial** — fuzz, benchmarks, checked-in baseline; new component cases need broader fuzz/scale coverage |
| M8 | Release engineering and installers | **Partial** — v0.1.1 artifacts and macOS DMG; signing/MSI pending |

## Architecture

```text
.fig
  → fig-reader          container detection, safe extraction, schema + data decode   [implemented]
  → conversion-core     Figma IR, ordered mapping passes, diagnostics at every decision   [M2–M3]
  → penpot-writer       Penpot document model → .penpot archive   [implemented]
  → validation          structural checks against derived Penpot rules   [implemented]
  → atomic write        temp file in destination volume → validate → rename   [implemented]
  → report              per-file and batch reports (JSON / HTML / CSV)   [implemented]
```

A thin headless CLI (`f2p-cli`) wraps the same API as the desktop app so conversion stays
testable in CI. The core crates never depend on the UI.

## Building

Requires a stable Rust toolchain. Linux desktop builds need GTK3 and the usual X11/Wayland/
OpenGL development packages; macOS and Windows use native dialogs and need no extra system
packages.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Native desktop app (optionally stage a file)
cargo run -p desktop-app
cargo run -p desktop-app -- tests/fixtures/circle.fig

# Inspect and convert a real file (headless)
cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig
cargo run -p f2p-cli -- convert tests/fixtures/circle.fig --out /tmp/f2p-out

# Batch convert
cargo run -p f2p-cli -- batch tests/fixtures/circle.fig tests/fixtures/text-group.fig \
  --out /tmp/f2p-out --jobs 2

# Export an existing JSON report
cargo run -p f2p-cli -- report /tmp/f2p-out/circle.report.json \
  --html /tmp/f2p-out/circle.html --csv /tmp/f2p-out/circle.csv
```

Desktop shortcuts use **Cmd** on macOS and **Ctrl** elsewhere: `+O` open files,
`+Shift+O` add a folder, `+Shift+A` analyze, `+Enter` convert, `+,` preferences, and
`+1`…`+8` jump between screens.

On Linux, install the desktop build dependencies first:

```bash
sudo apt-get install -y libgtk-3-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev \
  libegl1-mesa-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libwayland-dev
```

Release [`v0.1.1`](https://github.com/danialshirali16/Figma2Penpot/releases/tag/v0.1.1)
contains macOS app/CLI, Windows zip, and Linux tarball artifacts with SHA-256 checksums.
Builds are currently unsigned; see [`docs/RELEASE.md`](docs/RELEASE.md) and
[`PRIVACY.md`](PRIVACY.md).

The current source passed **116 workspace tests**, formatting, and Clippy with warnings
denied in a Linux container on Windows. `cargo-deny` and `cargo-audit` were unavailable;
their scans remain unverified. Platform evidence and exact checks are in
[`docs/DEV_ENVIRONMENT.md`](docs/DEV_ENVIRONMENT.md).

Optional fuzzing (the harness is in `fuzz/`):

```bash
cargo fuzz run read_fig --sanitizer none -- -max_total_time=60
```

`--sanitizer none` is needed on a stable toolchain without rustup/nightly; the harness has
already been run with 829,577 executions and no crashes. See `PROGRESS.md` for the exact
result.

## Documentation

| Document | Purpose | Status |
|---|---|---|
| [`README.md`](README.md) | Product overview | This file |
| [`PROGRESS.md`](PROGRESS.md) | Living status, evidence, and next steps | Exists |
| [`Roadmap.md`](Roadmap.md) | Milestones, gates, and acceptance criteria | Exists |
| [`docs/DOCUMENTATION_INDEX.md`](docs/DOCUMENTATION_INDEX.md) | Complete document map and current/historical evidence scope | Updated 2026-11-02 |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Current crates, conversion bounds, reporting, queue | Updated 2026-11-02 |
| [`docs/PROJECT_BRIEF.md`](docs/PROJECT_BRIEF.md) | Working restatement of the requirements | Exists (M0) |
| [`docs/SUPPORT_MATRIX.md`](docs/SUPPORT_MATRIX.md) | Per-feature support status and remaining limits | Updated 2026-11-02 |
| [`docs/ISSUE_CODES.md`](docs/ISSUE_CODES.md) | Reader warnings and 30 converter issue codes | Updated 2026-11-02 |
| [`docs/MANUAL_QA.md`](docs/MANUAL_QA.md) | Recorded imports, platform checks, pending visual/persistence QA | Updated 2026-11-02 |
| [`docs/FIG_FORMAT.md`](docs/FIG_FORMAT.md) | `.fig` container, Kiwi, nodes, vectors, unknowns | Exists (M0) |
| [`docs/PENPOT_FORMAT.md`](docs/PENPOT_FORMAT.md) | `.penpot` v3 archive, schema, import limits | Exists (M0) |
| [`docs/LICENSE_ASSESSMENT.md`](docs/LICENSE_ASSESSMENT.md) | Upstream licenses and reuse decisions | Exists (M0) |
| [`docs/RISKS.md`](docs/RISKS.md) | Technical risk register | Exists (M0) |
| [`docs/DEV_ENVIRONMENT.md`](docs/DEV_ENVIRONMENT.md) | Verified toolchain and platform facts | Exists (M0) |
| [`docs/RESEARCH_SOURCES.md`](docs/RESEARCH_SOURCES.md) | Pinned upstream commits and provenance | Exists (M0) |
| [`docs/DECISIONS/`](docs/DECISIONS/) | Architecture decision records | 0001–0011 |
| [`docs/RELEASE.md`](docs/RELEASE.md) | Published artifacts, source-only fixes, release checklist | Updated 2026-11-02 |
| [`docs/UI_SPEC.md`](docs/UI_SPEC.md) | Desktop design contract, copy, and implementation gaps | Updated 2026-11-02 |

## Non-goals for v1

- Penpot → Figma conversion
- Fetching files through the Figma REST API or any cloud service
- Figma or Penpot plugins
- Real-time sync, accounts, or collaboration features
- Editing designs inside the app
- Automatic repair beyond what conversion requires

## Privacy

There is no telemetry, no crash reporting endpoint, no update check, and no network access
during conversion. A future network feature, if any, must be opt-in and documented. This
statement will be revisited when a network-capable dependency enters the tree.

## License and upstream research

The project is dual-licensed as **MIT OR Apache-2.0** (ADR-0001). The full assessment is in
`docs/LICENSE_ASSESSMENT.md`; see [`LICENSE-MIT`](LICENSE-MIT),
[`LICENSE-APACHE`](LICENSE-APACHE), and [`NOTICE`](NOTICE). It concludes that no upstream
code may be copied or ported: the reference converter is unlicensed, and the official
exporter and Penpot are MPL-2.0.

M0 facts were checked on 2026-11-01; the reference converter's license metadata was
rechecked on 2026-11-02 (still no declared license):

- The reference community project [`girafic/penpot-converter`](https://github.com/girafic/penpot-converter)
  declares **no license** in its GitHub metadata. No permission to copy, modify, or
  redistribute its code is granted. It may be studied for behavior and documented
  observations only.
- The official [`penpot/penpot-exporter-figma-plugin`](https://github.com/penpot/penpot-exporter-figma-plugin)
  is **MPL-2.0**, and depends on `@penpot/library` (**MPL-2.0**). MPL-2.0 is file-level
  copyleft: copying or translating those files into Rust would carry obligations.
- Penpot itself is **MPL-2.0**.
- `openfig-core`, an independent `.fig` parser used by the community converter, is **MIT**.
  It is JavaScript, so it is research input, not a dependency.

**No upstream code has been copied or ported into this repository.** The project is a
reimplementation without code reuse, from documented behavior, observed file structure, and
our own fixtures. It is not a formal clean-room: the same engineers read upstream source,
and `docs/DECISIONS/0002-no-code-reuse.md` says so plainly.

## Contributing

Use [`AGENTS.md`](AGENTS.md) for build, testing, reporting, and fixture rules. Changes need
appropriate tests and accurate support/issue documentation. Dedicated contributor templates
remain Planned. Keep private designs and reports out of commits. Do not paste code from the
reference converter or the Penpot exporter; see `docs/LICENSE_ASSESSMENT.md`.
