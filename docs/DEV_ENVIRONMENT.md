# Development Environment

Recorded: **2026-11-01** (session 1). Re-verify before trusting any number here.

Current Windows environment: **2026-11-02**, recorded in §§9–10. Section 10 supersedes
the initial Docker/toolchain availability checks in §9. Sections 1–8 retain earlier macOS
environment and verification history; they do not describe the current machine.

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
| Desktop stack | `eframe`/`egui` 0.36.2, `rfd` 0.16.1, `dirs` 7.0.0 | M4.1; rfd uses GTK3 on Linux |
| Batch queue | `conversion-queue`, default 2 workers | M5 core; memory benchmark pending |
| Screen capture | `screencapture -l <id>` works; full-screen capture blocked | Terminal lacks Screen Recording permission; window capture succeeds |

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
| Penpot instance | Local Docker stack used for M2–M3.8, image `2.18.5` (backend `2.18.5-RC1`); stopped after verification |
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
- CI: GitHub Actions added (`.github/workflows/ci.yml`); M3.1 run **41329353648** passed
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
telemetry disabled, and a local account. M2 archives and M3.1 vector archives imported with
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

## 9. Windows orientation — 2026-11-02 (session 15)

| Item | Observed value | Evidence |
|---|---|---|
| OS | Microsoft Windows 11 Pro, 10.0.26300, 64-bit | `Get-CimInstance Win32_OperatingSystem` |
| Shell | PowerShell 7.6.5 | `$PSVersionTable.PSVersion` |
| Checkout | `C:\Users\Danny\Downloads\Figma2Penpot`, branch `main`, starting commit `824b098` | `Get-Location`, `git branch --show-current`, `git log -1` |
| Git | 2.56.0.windows.3 | `git --version` |
| Rust tools | `rustc`, `cargo`, and `rustup` not found on `PATH`; `%USERPROFILE%\.cargo\bin\cargo.exe` and `%USERPROFILE%\.rustup` absent | `Get-Command`, `Test-Path` |
| Compliance tools | `cargo-deny` and `cargo-audit` not found on `PATH` | `Get-Command` |
| Docker CLI | 29.8.1, build `4a63305` | `docker --version`; daemon and Penpot availability unverified |
| Code index | Unavailable: `list_projects`, `index_repository`, and `check_index_coverage` returned `Transport closed` | Codebase-memory MCP; direct source reads and `rg` used as fallback |

This session reviewed the repository and its main source entry points. No application
behavior changed. Build, format, clippy, tests, license/advisory checks, desktop interaction,
and real Penpot import remain **unverified in this environment**; prior results above are
historical evidence only. To verify here, make a stable Rust toolchain available, then run
the workspace gates in `AGENTS.md` and the Windows checks in `docs/MANUAL_QA.md`.

## 10. Local container verification — 2026-11-02 (session 16)

The Windows host still has no Rust toolchain on `PATH`. Docker Desktop was started for
isolated development and local Penpot import checks; no design data was sent to a remote
conversion service.

| Item | Observed value |
|---|---|
| Docker daemon | 29.8.1 |
| Rust / Cargo in containers | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` / `cargo 1.99.1 (5f94df478 2026-08-27)` |
| Workspace gate container | Debian 13/trixie, native GTK3/X11/Wayland/OpenGL development packages |
| Initial development image | `penpotapp/devenv:latest`; its hardened-package mirror prevented installing matching GTK headers, so full gates used a separate Debian container |
| Public package access | HTTPS with the existing trusted CA bundle explicitly configured for APT; signature/TLS validation retained |
| Full workspace tests | 116 passed, including headless desktop tests |
| Formatting / Clippy | Passed, workspace/all-targets, warnings denied |
| `cargo-deny` / `cargo-audit` | Not installed; commands report `no such command`; scans unverified |
| Local Penpot | Image tag 2.18.5; browser runtime reports `2.18.5-RC1`; isolated database/assets, loopback port 19001, telemetry disabled |
| Private-file imports | Five accepted; SSE end events recorded; source/output hashes and reports remain local |
| Visual checks | Navigation file viewed; Swiss page list opened; full screenshots/comparison and edit persistence remain partial/unverified |

The codebase-memory service remained unavailable (`Transport closed`), including indexing
and coverage requests. No graph completeness claim is made.

The seven task-owned build/verification containers were stopped after verification; their
volumes and private import evidence were retained. The isolated browser tab was closed.
Other Docker containers were left running.
