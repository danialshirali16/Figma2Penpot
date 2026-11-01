# Figma 2 Penpot

**Move your designs from Figma to Penpot. Simply.**

[![CI](https://github.com/danialshirali16/Figma2Penpot/actions/workflows/ci.yml/badge.svg)](https://github.com/danialshirali16/Figma2Penpot/actions/workflows/ci.yml)

A native, offline, open-source desktop application for Windows and macOS that converts Figma
`.fig` files into Penpot `.penpot` files, in batches, with a conversion report you can trust.

---

> ## Status: pre-alpha — M4 desktop shell and M5 batch core in progress
>
> The Rust workspace reads real `.fig` files and converts the M2 slice plus vectors, masks,
> boolean fallbacks, gradients, shadows, blur, image fills, text metrics, flex containers,
> styles, components/variants, and design tokens to `.penpot`. It validates the archive and
> writes an event-based JSON report. M2–M3.8 outputs imported into a local **Penpot 2.18.5**
> stack and were inspected. The native desktop shell now exists with eight screens, native
> file dialogs, background conversion, headless UI tests, and a batch queue core exposed
> through CLI `batch` and the desktop “Convert all” button. Batch retry/remove controls,
> memory benchmarks, and release packaging are not built yet. Stroke-only vectors, editable network handles, fixed constraints, Auto Layout child
> sizing, effect style linkage, instance swaps, mixed text runs, and non-paint variable
> bindings are documented Partial items with runtime diagnostics. See
> [`PROGRESS.md`](PROGRESS.md) and [`docs/MANUAL_QA.md`](docs/MANUAL_QA.md).

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

## What it will do

| Step | What you get |
|---|---|
| **Import** | Drag and drop, multi-file and folder pickers (optionally recursive), a reorderable queue, and per-file validation before anything is converted. |
| **Analyze** | Pages, frames, layers, components, instances, variants, variables, styles, images, and likely compatibility problems — with `Unknown` shown honestly instead of guessed. |
| **Configure** | Sane defaults, advanced options behind progressive disclosure: page selection, fidelity-vs-token-binding preference, page splitting, concurrency, overwrite policy, validation. |
| **Convert** | Real stage-based progress (`Reading → Decoding → Mapping → Writing → Validating`). No invented percentages. |
| **Review** | Per-file results, a combined batch summary, and a report that links every issue to a node, page, and layer path. |
| **Export** | Reports as JSON, standalone HTML, or CSV. Open the output folder. |

One `.fig` in, one `.penpot` plus one report out. A batch produces a combined summary as
well. One failing file never stops the rest.

## The Conversion Report is the product

The report is generated from events emitted **during** conversion, not reconstructed from
the output afterwards. Every transformation decision — converted, approximated,
substituted, skipped, failed — is recorded at the point where it happens.

Illustrative entry (format only; this is not output from a working converter):

> **F2P-VAR-001 · Unsupported variable binding** — *Warning*, converted with modification
> **Element:** Primary Button (`12:340`) · **Page:** Components · **Path:** Components / Buttons / Primary
> **Problem:** This variable binding has no Penpot equivalent.
> **Result:** The resolved value was preserved as a static value; the dynamic binding was lost.
> **Suggested action:** Reconnect the matching design token in Penpot.

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
Reader, M2 conversion, and M3.1 vector-path features are **Verified**, **Implemented
(unverified)**, or **Partial**; the rest remain **Planned** until they exist.

## Project status at a glance

| Milestone | Scope | Status |
|---|---|---|
| M0 | Research, format findings, license assessment, architecture | **Done** — evidence in `PROGRESS.md` |
| M1 | Cargo workspace, `.fig` container/decode, `f2p-cli inspect` | **Done** — evidence in `PROGRESS.md` |
| M2 | Minimal end-to-end conversion + validator + JSON report | **Done** — imported and inspected in Penpot 2.18.5-RC1 |
| M3 | Fidelity expansion (vectors → effects → images → text → layout → styles → components → tokens) | **Done with Partial items** — import-verified in Penpot 2.18.5-RC1 |
| M4 | Native desktop application (eight screens) | **In progress** — M4.1 shell and worker |
| M5 | Batch conversion (bounded concurrency, pause, cancel, retry) | **In progress** — queue core, CLI batch, desktop Convert all |
| M6 | Report UI and JSON/HTML/CSV exports | **Planned** |
| M7 | Reliability, fuzzing, performance benchmarks | **Planned** |
| M8 | Release engineering and installers | **Planned** |

## Intended architecture

```text
.fig
  → fig-reader          container detection, safe extraction, schema + data decode   [implemented]
  → conversion-core     Figma IR, ordered mapping passes, diagnostics at every decision   [M2–M3]
  → penpot-writer       Penpot document model → .penpot archive   [implemented]
  → validation          structural checks against derived Penpot rules   [implemented]
  → atomic write        temp file in destination volume → validate → rename   [implemented]
  → report              per-file and batch reports (JSON / HTML / CSV)   [per-file JSON implemented]
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
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System design, crates, passes, reporting | Exists (M0) |
| [`docs/PROJECT_BRIEF.md`](docs/PROJECT_BRIEF.md) | Working restatement of the requirements | Exists (M0) |
| [`docs/SUPPORT_MATRIX.md`](docs/SUPPORT_MATRIX.md) | Per-feature support status | Exists (M2 reader + conversion) |
| [`docs/ISSUE_CODES.md`](docs/ISSUE_CODES.md) | Stable diagnostic codes | Exists (reader + M2 conversion) |
| [`docs/MANUAL_QA.md`](docs/MANUAL_QA.md) | Platform test checklist | Exists (import checklist pending) |
| [`docs/FIG_FORMAT.md`](docs/FIG_FORMAT.md) | `.fig` container, Kiwi, nodes, vectors, unknowns | Exists (M0) |
| [`docs/PENPOT_FORMAT.md`](docs/PENPOT_FORMAT.md) | `.penpot` v3 archive, schema, import limits | Exists (M0) |
| [`docs/LICENSE_ASSESSMENT.md`](docs/LICENSE_ASSESSMENT.md) | Upstream licenses and reuse decisions | Exists (M0) |
| [`docs/RISKS.md`](docs/RISKS.md) | Technical risk register | Exists (M0) |
| [`docs/DEV_ENVIRONMENT.md`](docs/DEV_ENVIRONMENT.md) | Verified toolchain and platform facts | Exists (M0) |
| [`docs/RESEARCH_SOURCES.md`](docs/RESEARCH_SOURCES.md) | Pinned upstream commits and provenance | Exists (M0) |
| [`docs/DECISIONS/`](docs/DECISIONS/) | Architecture decision records | 8 accepted/superseding |
| `docs/ISSUE_CODES.md` | Stable diagnostic codes | Exists (M1 reader warnings) |
| `docs/MANUAL_QA.md` | Platform test checklist | Planned (M4) |

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

Facts established during M0 (checked 2026-11-01):

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

Contribution guidelines, a code of conduct, and issue templates will arrive with M1.
Until then, please open an issue rather than a pull request. Do not paste code from the
reference converter or the Penpot exporter; see `docs/LICENSE_ASSESSMENT.md`.
