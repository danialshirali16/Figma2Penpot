# Development Environment

Recorded: **2026-11-01** (session 1). Re-verify before trusting any number here.

This file records what was actually detected on the development machine, not what the
project wishes were available. Missing capability is listed as missing.

---

## 1. Machine

| Item | Value | Command / source |
|---|---|---|
| OS | macOS 27.2 (Darwin 27.2.0) | `sw_vers`, `uname -a` |
| Architecture | `arm64` (Apple Silicon, `T8103`) | `uname -m`, `uname -a` |
| CPU cores | 8 logical | `sysctl -n hw.ncpu` |
| RAM | 8 GiB | `sysctl -n hw.memsize` |
| Free disk at start | ~28 GiB | `df -h .` |
| Display | WindowServer running | `pgrep -x WindowServer` |
| Sandbox note | macOS may quarantine downloaded apps; no admin prompt was used | — |

8 GiB is the real constraint to design against. It is the main reason the batch queue must
be bounded and memory-aware, and why streaming is preferred over holding decoded assets.

## 2. Toolchain

| Tool | Version | Notes |
|---|---|---|
| `rustc` | 1.99.1 (2026-10-13), Homebrew | Works for the planned crates |
| `cargo` | 1.99.1 (2026-08-27), Homebrew | — |
| `rustup` | **not installed** | No cross targets, no `rustup target add`, no toolchain pinning via rustup |
| Xcode Command Line Tools | present (`/Library/Developer/CommandLineTools`) | Supplies the native linker |
| `git` | 2.56.0 | — |
| Node.js | v22.23.1 | **Research only.** Never used by the product |
| Python 3 | present (pyenv shim) | Used for research inspection scripts only |
| Docker CLI | present (`/usr/local/bin/docker`) | Daemon **not running** |
| `cargo-fuzz` | 0.13.2 (Homebrew) | Installed in M1; `--sanitizer none` used on stable |
| `cargo-deny` | absent | M8 / compliance gate |
| `cargo-audit` | absent | M8 / compliance gate |
| `insta` CLI | absent (crate can still be used) | Not needed yet |

Because `rustup` is absent, the workspace must be buildable with the Homebrew toolchain
and CI must pin its own stable version. Minimum supported Rust version (MSRV) is an M1
decision driven by the slowest-moving dependency (`eframe 0.37` currently declares
`rust 1.95`).

## 3. Network and external services

| Capability | Status |
|---|---|
| Internet reachable during research | Yes (GitHub HTTP 200) |
| GitHub API unauthenticated | Yes, rate-limited |
| npm registry | Yes, used only to inspect upstream packages |
| crates.io | Yes, used to check licenses and versions |
| Penpot instance | **Available for M2**: local Docker stack, image `2.18.5` (backend `2.18.5-RC1`), telemetry disabled |
| Figma account / plugin runtime | Not required; Figma.app is installed but was not used |

**Conversion must not require any of this.** Network access is a development-time capability
only. The product's conversion path stays offline.

## 4. Repository state at this record

```text
Figma 2 Penpot/
├── Cargo.toml / Cargo.lock
├── rust-toolchain.toml
├── crates/
│   ├── fig-reader/     container, Kiwi schema/message decode, raw nodes
│   └── f2p-cli/        `inspect` command
├── fuzz/               libFuzzer harness (`read_fig`)
├── tests/fixtures/     4 MIT fixtures + provenance
├── .github/workflows/  CI (fmt, clippy, test)
├── README.md
├── Roadmap.md
├── PROGRESS.md
├── State.md            pointer to PROGRESS.md
├── LICENSE-MIT / LICENSE-APACHE / NOTICE
└── docs/
    ├── DEV_ENVIRONMENT.md
    ├── RESEARCH_SOURCES.md
    ├── PROJECT_BRIEF.md
    ├── FIG_FORMAT.md
    ├── PENPOT_FORMAT.md
    ├── ARCHITECTURE.md
    ├── RISKS.md
    ├── SUPPORT_MATRIX.md
    ├── LICENSE_ASSESSMENT.md
    └── DECISIONS/
```

- Git repository: initialized on `main`; public remote
  `https://github.com/danialshirali16/Figma2Penpot`.
- Cargo workspace: exists; `fig-reader` and `f2p-cli`; 4 fixtures under `tests/fixtures`.
- CI: GitHub Actions added (`.github/workflows/ci.yml`); run **41322990919** passed
  `fmt`, `clippy`, and `test` on Ubuntu, macOS, and Windows.
- Research checkouts live **outside** the repo at `~/Downloads/f2p-research-upstream/`
  so unlicensed upstream code cannot accidentally be committed.

## 5. What can be verified here vs. what cannot

| Area | Locally verifiable? | Consequence |
|---|---|---|
| Rust build, unit tests, clippy, fmt | Yes | Workspace exists; gates run locally |
| `.fig` parsing and `.penpot` generation | Yes | M1/M2/local golden tests |
| Structural validation of `.penpot` | Yes | Build in Rust (M2) |
| Real Penpot import | **Yes for M2** (2026-11-01) | See `MANUAL_QA.md`; reopen/edit persistence untested |
| Windows build and installer | **Build/test: yes via CI; GUI/installer: no** | CI is green; label GUI/installer **unverified on this platform** |
| macOS Intel build | **No** (Apple Silicon host) | Not built; label honestly |
| macOS GUI behavior | Yes, but only on this machine | Manual QA checklist; still not a second machine |
| GPU/rendering differences | Partially | Screenshot tests + manual QA |
| Large-file performance | Yes, once fixtures exist | Record machine specs with every benchmark |

## 6. Penpot import — verified for M2

A local Penpot stack was started via Docker Desktop (daemon 29.9.2) using the official
`docker/images/docker-compose.yaml` from Penpot tag 2.18.5, with image tag `2.18.5`,
telemetry disabled, and a local account. Both M2 archives imported with
`app.binfile.v3 ... error?=false` and were inspected in the UI.

Backend reports `2.18.5-RC1`. Evidence, screenshots, and the exact checks are in
`docs/MANUAL_QA.md`. The stack was stopped after verification; it is not left running.

For future manual checks, the steps are:

1. Start a supported Penpot version. Target: **2.18.5**
   (see `docs/DECISIONS/0003-target-penpot-version.md`).
2. Sign in to a local account (no cloud account needed for self-hosted).
3. Create a project, choose **Import file**, select the generated `.penpot`.
4. Confirm the file opens without an import error.
5. Inspect page count/names, frame positions, child positions, fills/strokes, text, groups,
   component links, and tokens as applicable.
6. Record screenshots and the exact Penpot version in `PROGRESS.md` / `MANUAL_QA.md`.

Do not mark a feature Verified from a structural validator alone.

## 7. Reproduction commands

```bash
uname -a
sw_vers
rustc --version
cargo --version
rustup target list --installed        # fails: no rustup
xcode-select -p
docker info                           # fails: daemon not running
sysctl -n hw.ncpu hw.memsize
df -h .
git --version
node --version                        # research only

# Project gates
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p f2p-cli -- inspect tests/fixtures/circle.fig
cargo fuzz run read_fig --sanitizer none -- -max_total_time=60
```

## 8. Setup still needed (M2+)

- Keep the CI matrix green; harden action pinning and address the Node 20 deprecation
  annotation in M8.
- Add `cargo-deny` and `cargo-audit` gates; the tools are not installed yet.
- Decide whether to run Penpot via Docker for import checks; if yes, document the image
  tag and startup command, and keep it optional so ordinary builds stay offline.
- Optional nightly toolchain for sanitizer-enabled fuzzing; the stable
  `--sanitizer none` run is the current evidence.
