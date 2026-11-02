# Release and Packaging

Status: **M8 partial (2026-11-02).** `v0.1.1` is published as a GitHub pre-release with
macOS app/DMG/CLI, Windows zip, and Linux tarball artifacts plus SHA-256 checksums. Builds
are **unsigned** and **not notarized**; the Windows MSI installer is not built yet.

Release: <https://github.com/danialshirali16/Figma2Penpot/releases/tag/v0.1.1>

The ADR-0011 compatibility changes are source-only relative to that release. Merging them
does not publish replacement binaries or retag `v0.1.1`. The workspace package version is
still `0.1.0`; the published tag and Cargo package version currently differ. Choose and
align a new version before the next release, and generate artifacts from that exact commit.

Source validation on 2026-11-02: 116 workspace tests, formatting, and Clippy passed in a
Linux container. Five private archives were accepted by local Penpot 2.18.5-RC1; full
fidelity remains Partial. `cargo-deny`/`cargo-audit`, exhaustive visual/persistence checks,
and native Windows/macOS checks of these new mappings remain pending.

## Artifacts

| Platform | Artifact | Contents |
|---|---|---|
| macOS | `figma2penpot-<version>-macos-<arch>.zip` | `Figma 2 Penpot.app` bundle |
| macOS DMG | `figma2penpot-<version>-macos-<arch>.dmg` | Compressed disk image with the `.app` |
| macOS CLI | `figma2penpot-<version>-cli-macos-<arch>.zip` | `f2p-cli` binary |
| Windows | `figma2penpot-<version>-windows-x86_64.zip` | `figma2penpot.exe`, `f2p-cli.exe`, README |
| Linux | `figma2penpot-<version>-linux-<arch>.tar.gz` | `figma2penpot`, `f2p-cli`, README |

Every archive has a sibling `.sha256` file.

## Build locally

```bash
cargo build --release -p desktop-app -p f2p-cli

# macOS
bash scripts/package-macos.sh 0.1.1 target/release/figma2penpot dist
bash scripts/package-macos-dmg.sh 0.1.1

# Windows (PowerShell)
./scripts/package-windows.ps1 -Version 0.1.1

# Linux
bash scripts/package-linux.sh 0.1.1
```

## Release workflow

`.github/workflows/release.yml` runs on tags matching `v*` and on manual dispatch:

1. Build release binaries on macOS, Windows, and Linux.
2. Package the macOS `.app`/DMG/CLI, Windows zip, and Linux tarball.
3. Generate SHA-256 checksums.
4. Upload artifacts and, for tags, create a GitHub Release with generated notes.

## Before the next release

1. Align the Cargo package version, new tag, and package labels; keep prior tags immutable.
2. Run workspace gates on the release commit, including cross-platform CI.
3. Run dependency license/advisory scans and regenerate full cross-platform license texts.
4. Review ADR-0011 reports/remaining limitations; verify edits and reopening in Penpot and
   native platform interactions. Import acceptance alone is not a fidelity certification.
5. Exclude private designs, reports, thumbnails, and local import credentials from artifacts.
6. Build/package from that commit, verify checksums, and label unsigned artifacts honestly.

## Signing and notarization plan

- **Current:** ad-hoc/linker-signed macOS binaries; Windows unsigned. Gatekeeper and
  SmartScreen will warn. Do not claim a frictionless install.
- **macOS:** obtain a Developer ID Application certificate, sign the `.app` with hardened
  runtime, submit with `notarytool`, and staple the ticket.
- **Windows:** obtain a code-signing certificate, sign `figma2penpot.exe` and
  `f2p-cli.exe`, timestamp the signatures, and prefer an MSI/EXE installer over a zip.
- **Linux:** document GTK3/X11/Wayland/OpenGL runtime dependencies; consider AppImage or a
  distro package later.

## Verification status

| Check | Status |
|---|---|
| macOS `.app` builds | **Pass** — local build, `open` launched the app, window captured |
| macOS zip/checksum | **Pass** — 3.8 MB zip, SHA-256 generated |
| macOS DMG | **Pass locally** — 4.3 MB compressed DMG + SHA-256 |
| macOS signing/notarization | **Not done** — ad-hoc signature only |
| Windows zip build | **Pass in CI** — 5.0 MB artifact; local install **untested** |
| Linux tarball build | **Pass in CI** — 6.8 MB artifact; local install **untested** |
| Install-and-launch on all platforms | **Partial** — macOS pass; Windows/Linux untested |
| GitHub Release artifacts | **Pass** — `v0.1.1` prerelease, 10 assets including DMG + checksums |

## Licensing and privacy

- Project license: MIT OR Apache-2.0.
- Conversion is local-only. See `PRIVACY.md`.
- Third-party dependency licenses are listed in `THIRD_PARTY.md`; run `cargo deny check`
  before a stable release.
