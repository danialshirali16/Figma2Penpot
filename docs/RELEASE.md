# Release and Packaging

Status: **M8 partial (2026-11-02).** `v0.1.0` is published as a GitHub pre-release with
macOS, Windows, and Linux artifacts plus SHA-256 checksums. Builds are **unsigned** and
**not notarized**; installers (MSI/DMG) are not built yet.

Release: <https://github.com/danialshirali16/Figma2Penpot/releases/tag/v0.1.0>

## Artifacts

| Platform | Artifact | Contents |
|---|---|---|
| macOS | `figma2penpot-<version>-macos-<arch>.zip` | `Figma 2 Penpot.app` bundle |
| macOS CLI | `figma2penpot-<version>-cli-macos-<arch>.zip` | `f2p-cli` binary |
| Windows | `figma2penpot-<version>-windows-x86_64.zip` | `figma2penpot.exe`, `f2p-cli.exe`, README |
| Linux | `figma2penpot-<version>-linux-<arch>.tar.gz` | `figma2penpot`, `f2p-cli`, README |

Every archive has a sibling `.sha256` file.

## Build locally

```bash
cargo build --release -p desktop-app -p f2p-cli

# macOS
bash scripts/package-macos.sh 0.1.0 target/release/figma2penpot dist

# Windows (PowerShell)
./scripts/package-windows.ps1 -Version 0.1.0

# Linux
bash scripts/package-linux.sh 0.1.0
```

## Release workflow

`.github/workflows/release.yml` runs on tags matching `v*` and on manual dispatch:

1. Build release binaries on macOS, Windows, and Linux.
2. Package the `.app`, `.exe` zip, and Linux tarball.
3. Generate SHA-256 checksums.
4. Upload artifacts and, for tags, create a GitHub Release with generated notes.

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
| macOS signing/notarization | **Not done** — ad-hoc signature only |
| Windows zip build | **Pass in CI** — 5.0 MB artifact; local install **untested** |
| Linux tarball build | **Pass in CI** — 6.8 MB artifact; local install **untested** |
| Install-and-launch on all platforms | **Partial** — macOS pass; Windows/Linux untested |
| GitHub Release artifacts | **Pass** — `v0.1.0` prerelease, 8 assets + checksums |

## Licensing and privacy

- Project license: MIT OR Apache-2.0.
- Conversion is local-only. See `PRIVACY.md`.
- Third-party dependency licenses are listed in `THIRD_PARTY.md`; run `cargo deny check`
  before a stable release.
