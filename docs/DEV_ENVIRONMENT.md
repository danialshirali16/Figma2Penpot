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
| `cargo-fuzz` | absent | M1/M7 dependency |
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
| Penpot instance | **Not available** |
| Figma account / plugin runtime | Not required; Figma.app is installed but was not used |

**Conversion must not require any of this.** Network access is a development-time capability
only. The product's conversion path stays offline.

## 4. Repository state at this record

```text
Figma 2 Penpot/
├── README.md
├── Roadmap.md
├── State.md            pointer to PROGRESS.md
├── PROGRESS.md         canonical status
├── LICENSE-MIT
├── LICENSE-APACHE
├── NOTICE
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

- Git repository: **not initialized** at `~/Downloads/Figma 2 Penpot`.
- Cargo workspace: **does not exist** (M1).
- CI: **does not exist** (M1).
- Research checkouts live **outside** the repo at `~/Downloads/f2p-research-upstream/`
  so unlicensed upstream code cannot accidentally be committed.

## 5. What can be verified here vs. what cannot

| Area | Locally verifiable? | Consequence |
|---|---|---|
| Rust build, unit tests, clippy, fmt | Yes, after M1 workspace exists | Full local gate |
| `.fig` parsing and `.penpot` generation | Yes | M1/M2/local golden tests |
| Structural validation of `.penpot` | Yes | Build in Rust (M2) |
| Real Penpot import | **No** (no instance, Docker daemon down) | Mark **pending**; provide exact manual steps |
| Windows build and installer | **No** | CI only; label **unverified on this platform** |
| macOS Intel build | **No** (Apple Silicon host) | CI cross-build or omit, label honestly |
| macOS GUI behavior | Yes, but only on this machine | Manual QA checklist; still not a second machine |
| GPU/rendering differences | Partially | Screenshot tests + manual QA |
| Large-file performance | Yes, once fixtures exist | Record machine specs with every benchmark |

## 6. Penpot import — current blocker and manual path

Docker is installed but its daemon is not running, and no Penpot instance is reachable.
Until that changes, no output can be called **Verified** in the support vocabulary.

When an instance becomes available, the manual check is:

1. Start a supported Penpot version. Target: **2.18.5** (see
   `docs/DECISIONS/0003-target-penpot-version.md`).
2. Sign in to a local account (no cloud account needed for self-hosted).
3. Create a project, choose **Import file**, select the generated `.penpot`.
4. Confirm the file opens without an import error.
5. Inspect, in this order: page count and names, frame positions, child positions inside a
   frame that is not at the origin, fills/strokes, text content, image fills, component
   links, tokens.
6. Record screenshots and the exact Penpot version in `PROGRESS.md`.

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
```

## 8. Setup still needed (M1)

- Initialize git and the Cargo workspace.
- Pin the toolchain in CI (stable, at least 1.95 for `eframe 0.37`).
- Add `cargo-fuzz`, `cargo-deny`, and `cargo-audit` to CI images or install locally.
- Add a Linux CI job as well as macOS and Windows; containerized Linux is the cheapest
  place to run parser fuzzing.
- Decide whether to run Penpot via Docker for import checks; if yes, document the image
  tag and startup command, and keep it optional so ordinary builds stay offline.
