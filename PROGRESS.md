# PROGRESS

Living status file for Figma 2 Penpot. Canonical place to resume from.

Last updated: **2026-11-01** (session 2, M1).

---

## 1. Where we are

**Milestone: M1 complete. M2 (minimal end-to-end conversion) is next.**

The workspace builds and the `.fig` reader works on four real fixtures. `f2p-cli inspect`
prints verified metadata. There is **no `.penpot` writer, no conversion, and no report yet**.
Every conversion feature remains **Planned**.

M1 acceptance:

| Requirement | Status | Evidence |
|---|---|---|
| Cargo workspace compiles on stable | Done | `cargo build --workspace`, `cargo test --workspace` |
| CI runs fmt, clippy, test | Done | Run 41322990919: ubuntu-latest, macos-latest, windows-latest all success |
| Container detection + safe extraction | Done | ZIP and raw tests; zip-slip, entry-count, ratio, truncation tests |
| Schema + data decoding | Done | In-house bounded Kiwi decoder; 550 and 627 definition schemas |
| Raw node tree | Done (core fields) | GUID, parent, position, type, name, phase, visible, size, transform |
| `f2p-cli inspect` prints verified metadata | Done | Real output on `circle.fig`; integration test |
| Unit tests + ≥3 fixtures | Done | 4 fixtures; 29 unit/integration tests + 1 doctest |
| Malformed/truncated/bomb samples error without panic | Done | 6 truncation lengths, corrupt chunk, zip-slip, entry bomb, ratio bomb, node limit |
| Fuzz target exists and ran | Done | 829,577 executions in 61 s, no crashes, 61 MB peak RSS |

## 2. What exists

```text
Figma 2 Penpot/
├── Cargo.toml / Cargo.lock
├── rust-toolchain.toml
├── crates/
│   ├── fig-reader/
│   │   ├── src/container.rs       ZIP/raw detection, entry limits, meta.json, images
│   │   ├── src/document.rs        FigDocument, FigNode, Guid, Matrix, pages, stats
│   │   ├── src/kiwi/cursor.rs     checked primitive reader
│   │   ├── src/kiwi/schema.rs     bounded binary schema parser
│   │   ├── src/kiwi/decode.rs     bounded message decoder
│   │   ├── src/kiwi/payload.rs    header/chunks, deflate, zstd window cap
│   │   └── tests/fixtures.rs      14 integration tests
│   └── f2p-cli/
│       ├── src/main.rs            `inspect` command
│       └── tests/inspect.rs       2 CLI tests
├── fuzz/                          libFuzzer harness `read_fig`
├── tests/fixtures/                4 MIT fixtures + provenance + MIT notice
├── .github/workflows/ci.yml       Linux/macOS/Windows fmt, clippy, test
├── docs/                          M0 baseline + ADRs 0001–0006
└── PROGRESS.md
```

## 3. Environment

Full record: `docs/DEV_ENVIRONMENT.md`.

- macOS 27.2, arm64 (Apple Silicon), 8 cores, 8 GiB RAM.
- Rust 1.99.1 (Homebrew), no rustup, Xcode CLT.
- `cargo-fuzz 0.13.2` installed via Homebrew.
- Docker daemon not running; no Penpot instance.
- Windows and Intel macOS builds/tests are green in CI; **GUI and installer behavior remain
  unverified** on those platforms.
- Network to crates.io became intermittent during M1; dependencies were already cached and
  the build/fuzz completed. Do not assume network at build time.

## 4. Evidence log (session 2)

| Command | Result |
|---|---|
| `cargo build --workspace` | Success |
| `cargo test --workspace` | 29 unit/integration tests + 1 doctest pass; 0 failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | Success |
| `cargo fmt --all -- --check` | Success |
| `cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig` | 5 nodes, 3 blobs (540 B), 1 visible + 1 internal page, version 101 |
| `cargo run ... OpenFigs.fig` | Version 106, 5 nodes, 21 blobs (12,098 B) |
| `cargo run ... straight-2-point-line.fig` | Version 101, 627 schema defs, 4 nodes |
| `cargo run ... corner-radius-middle-dot-20.fig` | Version 101, 627 schema defs, 4 nodes |
| Research file `coloradjusted.fig` | 29 nodes, 51 blobs, 2 images; reports `BRUSH` as unknown type |
| `cargo fuzz run read_fig --sanitizer none -- -max_total_time=60` | 829,577 runs, 61 s, no crashes, 61 MB peak RSS |
| GitHub Actions run 41322990919 | Ubuntu, macOS, Windows: all success (fmt, clippy, test) |

The `coloradjusted.fig` research file is **not committed** (2.1 MB); it was inspected from the
upstream checkout outside the repo. Its `BRUSH` nodes prove unknown node types are reported
rather than rejected.

## 5. Implemented behavior

- Container detection by magic bytes: ZIP (`PK`) and raw `fig-kiwi` (`fig-`).
- ZIP entry validation: absolute paths, `..`, backslashes, NULs, control characters,
  Windows drive prefixes, empty components, names over 1024 bytes.
- ZIP limits: entry count, total declared uncompressed size, per-entry compression ratio,
  `canvas.fig` size, meta size.
- Raw payload: `fig-kiwi` header, 8-byte prelude, version, length-prefixed chunks with
  checked arithmetic, maximum chunk count.
- Schema: binary Kiwi schema parsed with definition/field/name/type-reference validation.
- Message: bounded arrays, byte-array ranges, depth limit, blob size limit, node limit.
- Metadata: pages, internal-only canvas, direct/descendant counts, node type histogram,
  unknown node types, orphan count, max depth, warnings.
- Blob access is bounds-checked; message bytes are retained for later conversion passes.
- `f2p-cli inspect` prints document name, container, format/version, schema/chunk counts,
  nodes, blobs, images, thumbnail, pages, node types, unknown types, orphans, depth, warnings.

## 6. Explicitly not implemented

- Any `.penpot` writing or conversion.
- Any report event model or JSON/HTML/CSV export.
- Paints, strokes, effects, text content, vector geometry, components, variables, tokens.
- Image bytes; only names/sizes are enumerated in M1.
- A materialized tree/arena API; M1 keeps the flat node list plus parent GUIDs.
- UI, batch queue, cancellation.
- Real Penpot import verification.

## 7. Decisions

| ADR | Decision |
|---|---|
| 0001 | Project license: MIT OR Apache-2.0 |
| 0002 | Reimplementation without code reuse; not a formal clean-room claim |
| 0003 | Target Penpot 2.18.5, archive v3, file data version 67 |
| 0004 | Superseded by 0006 for the decoder choice |
| 0005 | Penpot geometry is absolute; accumulate transforms |
| 0006 | Bounded in-house Kiwi decoder; no `kiwi-schema` runtime dependency |

## 8. Verified vs. unverified

- **Verified:** reader facts above, against four fixtures and hostile synthetic inputs.
- **Implemented (unverified):** container detection, schema/message decoding, `inspect`,
  resource limits. Not confirmed by a Penpot import (not applicable to reading) and not yet
  exercised by the full CI matrix.
- **Partial:** image enumeration (no bytes), raw node metadata (core fields only), unknown
  node type reporting (reports, does not map).
- **Planned:** every conversion/report/UI/batch feature.
- **Not supported:** `.deck`/`.jam`; rejected with a typed error.

## 9. Known gaps and open questions

1. **No conversion.** M2 must build the minimal writer, validator, and report.
2. **CI is green at `eef72fc`** on Linux, macOS, and Windows. Actions are pinned by tag, not
   SHA, and the Node 20 deprecation annotation is cosmetic for now; harden in M8.
3. **No real Penpot instance.** Import verification stays pending.
4. **`BRUSH` is an unknown Figma node type** in the research corpus. Decide in M3 whether it
   is in scope; until then it is reported.
5. **Images are only enumerated.** Conversion will need a lazy asset reader with byte limits.
6. **The raw tree is flat.** M2 needs an arena/tree for mapping passes.
7. **Sanitizer fuzzing needs nightly.** `--sanitizer none` is the current evidence; M7
   should run ASAN/UBSAN on CI or a nightly toolchain.
8. **`cargo-deny`/`cargo-audit` are absent.** Add them next; `THIRD_PARTY_LICENSES.md` is
   current as of M1.
9. **CI actions are pinned by tag, not SHA.** Harden in M8.

## 10. Next steps (in order)

1. Push M1; confirm the GitHub Actions matrix (Ubuntu, macOS, Windows) is green.
2. Add `docs/ISSUE_CODES.md` and the `conversion-report` crate with `ReportEvent`,
   `DiagnosticSink`, severity/outcome, and counts.
3. Build the minimal `conversion-core` and `penpot-writer` for pages, frames, groups,
   rectangles, ellipses, solid fills/strokes, opacity, and basic text.
4. Implement the structural validator derived from `docs/PENPOT_FORMAT.md`.
5. Add `f2p-cli convert` and a golden fixture test; attempt a real Penpot import or record
   exact pending steps.

## 11. How to resume

```bash
cd "/Users/danial/Downloads/Figma 2 Penpot"
git status
cat PROGRESS.md
cargo test --workspace
cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig
```

Then check whether `crates/conversion-core` and `crates/penpot-writer` exist. If not, start
at §10 step 2. Do not trust a status claim that is not backed by a command result recorded
here or in a later entry.

## 12. Session log

### Session 1 — 2026-11-01

- Completed M0: upstream study, format documents, license assessment, risk register,
  architecture, ADRs 0001–0005.
- Proved the pure-Rust read path with a scratch probe.
- Generated `.penpot` probes with the official library.
- Created the public repository and pushed the M0 baseline.

### Session 2 — 2026-11-01

- Created the Cargo workspace (`fig-reader`, `f2p-cli`), toolchain file, CI workflow.
- Implemented the bounded `.fig`/Kiwi reader and `inspect` CLI.
- Added 4 MIT fixtures with provenance and attribution.
- Added 29 unit/integration tests + 1 doctest, hostile-input tests, and a fuzz harness.
- Ran 829,577 fuzz executions in 61 s with no crashes.
- Pushed M1; GitHub Actions matrix (Ubuntu, macOS, Windows) passed.
- Recorded ADR-0006 (in-house decoder) and updated the format/architecture/license docs.
