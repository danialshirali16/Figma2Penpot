# PROGRESS

Living status file for Figma 2 Penpot. Canonical place to resume from.

Last updated: **2026-11-01** (session 1). Update this file at the end of every session.

---

## 1. Where we are

**Milestone: M0 complete. M1 (workspace and `.fig` reading) is next.**

There is **no Cargo workspace and no application code yet**. The repository contains the
M0 research baseline, decision records, and status documents. Every product feature remains
**Planned**. No conversion has been run because no converter exists.

M0 is complete because every acceptance item has evidence:

| M0 acceptance item | Status | Evidence |
|---|---|---|
| Clone and study the reference converter | Done | `docs/RESEARCH_SOURCES.md` §1; pinned commit `c29242f` |
| Review community thread, official exporter, migration docs | Done | `docs/RESEARCH_SOURCES.md` §2 |
| `docs/ARCHITECTURE.md` | Done | Upstream pipeline, mapping strategy, crates, reporting |
| `docs/FIG_FORMAT.md` | Done | Container, chunks, Kiwi, nodes, vectors, unknowns |
| `docs/PENPOT_FORMAT.md` | Done | Archive v3, manifest, shapes, import limits, unknowns |
| `docs/LICENSE_ASSESSMENT.md` | Done | Per-source licenses, obligations, clean-room-style decision |
| Technical risk assessment | Done | `docs/RISKS.md` |
| Rust implementation strategy | Done | `docs/ARCHITECTURE.md` §10-12; ADR-0004 |
| Claims cite files/lines/URLs | Done | Citations throughout the M0 documents |
| Unknowns explicitly listed | Done | Each format document has an "Explicit unknowns" section |
| Fork-vs-clean-room decision recorded | Done | `docs/DECISIONS/0002-no-code-reuse.md` |

## 2. What exists

```text
Figma 2 Penpot/
├── AGENTS.md
├── README.md
├── Roadmap.md
├── State.md                    pointer to this file (kept for the original brief naming)
├── PROGRESS.md                 this file
├── LICENSE-MIT
├── LICENSE-APACHE
├── NOTICE
└── docs/
    ├── ARCHITECTURE.md
    ├── DEV_ENVIRONMENT.md
    ├── FIG_FORMAT.md
    ├── LICENSE_ASSESSMENT.md
    ├── PENPOT_FORMAT.md
    ├── PROJECT_BRIEF.md
    ├── RESEARCH_SOURCES.md
    ├── RISKS.md
    ├── SUPPORT_MATRIX.md
    └── DECISIONS/
        ├── 0001-project-license.md
        ├── 0002-no-code-reuse.md
        ├── 0003-target-penpot-version.md
        ├── 0004-fig-decoding-strategy.md
        └── 0005-absolute-coordinates.md
```

- Git repository: **not initialized**.
- Cargo workspace: **does not exist**.
- Tests, fixtures, CI: **do not exist**.
- Research checkouts: `~/Downloads/f2p-research-upstream/` (outside the repo).

## 3. Environment

Full record: `docs/DEV_ENVIRONMENT.md`. Summary:

- macOS 27.2, arm64 (Apple Silicon), 8 cores, 8 GiB RAM, ~28 GiB free.
- Rust 1.99.1 (Homebrew), **no rustup**, Xcode CLT present.
- Docker CLI present, **daemon not running**; no Penpot instance.
- GUI available; Node.js present but forbidden in the product.
- Windows and Intel macOS are **unverified on this platform**.
- Network was available for research only; conversion must stay offline.

## 4. Evidence log (session 1)

| Command / check | Result |
|---|---|
| `git -C penpot-converter rev-parse --short HEAD` | `c29242f` |
| `git -C penpot-exporter-figma-plugin rev-parse --short HEAD` | `eae25d8` |
| `git -C openfig-core rev-parse --short HEAD` | `0d67354` |
| GitHub API license: reference converter | `license: null` |
| GitHub API license: exporter / Penpot | `MPL-2.0` / `MPL-2.0` |
| npm `openfig-core@0.4.1` | MIT; deps `fzstd`, `fflate`, `kiwi-schema` |
| Python ZIP inspection of `circle.fig` | `canvas.fig`, `thumbnail.png`, `meta.json`, `images/`; version 101; zstd magic |
| Rust probe: `circle.canvas.fig` | 550 schema defs, root `Message` index 404, 5 nodes, 3 blobs |
| Rust probe: `OpenFigs.canvas.fig` | 550 schema defs, 21 blobs |
| Rust probe: truncated 4,000-byte input | Typed error `chunk past end`; no panic |
| `@penpot/library 1.2.0` probe | `manifest.json`, file/page/shape JSON, root frame, file version 67 |
| Coordinate probe (frame at 100,200; child at 5,7) | Library stored child `(5,7)`; writer must add the offset |
| Penpot 2.18.5 vs develop importer diff | 2.18.5 lacks the newer text/entry limits and `externalLibraries` |

The scratch Rust probe lives outside the repo and is not product code. It proves the
decoding path is feasible, not that a parser exists.

## 5. Decisions

| ADR | Decision |
|---|---|
| 0001 | Project license: **MIT OR Apache-2.0** |
| 0002 | Reimplement without code reuse; no formal clean-room claim |
| 0003 | Target **Penpot 2.18.5**, archive v3, file data version **67** |
| 0004 | Decode with `kiwi-schema` 0.2.1 + `flate2`/`miniz_oxide` + `ruzstd`, wrapped in limits |
| 0005 | Penpot geometry is **absolute**; accumulate transforms |

## 6. Verified vs. unverified

- **Verified:** the research facts in the M0 documents, in the sense that they were checked
  against source, bytes, or generated output. This is research verification, not product
  verification.
- **Implemented (unverified):** none. There is no implementation.
- **Partial:** none.
- **Planned:** every product feature.
- **Pending:** real Penpot import of any output; Windows and Intel macOS behavior;
  component/media/token JSON details.

## 7. Known gaps and open questions

1. **No code.** M1 must create the workspace, CI, and reader.
2. **No real import.** Docker daemon is down and no Penpot instance is reachable. The
   manual steps are in `docs/DEV_ENVIRONMENT.md` §6.
3. **Component, media, and token JSON are only partly mapped.** The field names are known;
   no archive with those objects has been generated and imported. Do not claim fidelity.
4. **Rotation/flip representation** in Penpot is unresolved; defer to M3 with a fixture.
5. **`openfig-core` has no LICENSE file**, only MIT package metadata. Fixtures may be reused
   with attribution, but record the gap.
6. **`kiwi-schema` has no resource limits.** M1 must prove the wrapper can enforce them or
   trigger the vendoring fallback in ADR-0004.
7. **`PROJECT_BRIEF.md` was missing.** A working restatement now exists at
   `docs/PROJECT_BRIEF.md`; README, Roadmap, and Architecture remain the detailed sources.
8. **Git is not initialized.** The first M1 commit should include the M0 documents.

## 8. Next steps (in order)

1. **Initialize git**, add `.gitignore`, commit the M0 baseline with a clear message.
2. **Create the Cargo workspace** with `fig-reader`, `conversion-core`,
   `penpot-writer`, `conversion-report`, `conversion-queue`, `f2p-cli`, `desktop-app`
   stubs; add `rust-toolchain.toml` and `#![forbid(unsafe_code)]`.
3. **Add CI**: fmt, clippy `-D warnings`, test on Linux/macOS/Windows; `cargo-deny` and
   `cargo-audit`.
4. **Implement `fig-reader` M1**: ZIP/raw detection, limits, schema decode, typed raw tree,
   `f2p-cli inspect`, fixtures, negative samples, fuzz target.
5. **Add the first fixture manifest** (`tests/fixtures/PROVENANCE.md`) before copying any
   upstream fixture, and wire `cargo-deny` into CI.

## 9. How to resume

```bash
cd "/Users/danial/Downloads/Figma 2 Penpot"
cat PROGRESS.md
cat docs/ARCHITECTURE.md
cat docs/FIG_FORMAT.md
cat docs/PENPOT_FORMAT.md
cat docs/RISKS.md
```

Then check whether `Cargo.toml` exists. If not, start at §8 step 1. Do not trust a status
claim that is not backed by a command result recorded here or in a later entry.

## 10. Session log

### Session 1 — 2026-11-01

- Bootstrapped README, Roadmap, State.
- Completed M0 research: upstream study, format documents, license assessment, risk
  assessment, architecture, five ADRs.
- Proved the pure-Rust Kiwi read path with a scratch probe.
- Generated `.penpot` probes with the official library to confirm archive layout, file
  version, shape fields, and absolute-coordinate behavior.
- Renamed the canonical status file to `PROGRESS.md` per `AGENTS.md`; `State.md` remains a
  pointer.
- Removed an accidental copy of upstream MPL-2.0 source files (`penpot-files/`) from the
  repository before any commit. Research checkouts stay outside the repo.
- Reviewed all six posts of the community migration thread; recorded the import-failure
  context in `docs/RESEARCH_SOURCES.md`.
- Added `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE`.
- No product code was written. No conversion claim is made.
