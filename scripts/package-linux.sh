#!/usr/bin/env bash
# Package the Linux desktop binary and CLI into a tar.gz.
# The desktop binary needs GTK3, X11/Wayland, and OpenGL runtime libraries.
set -euo pipefail

VERSION="${1:-0.1.0}"
DIST="${2:-dist}"
STAGING="$DIST/staging-linux"

if [[ ! -x target/release/figma2penpot ]]; then
  echo "missing target/release/figma2penpot (run cargo build --release -p desktop-app)" >&2
  exit 1
fi
if [[ ! -x target/release/f2p-cli ]]; then
  echo "missing target/release/f2p-cli (run cargo build --release -p f2p-cli)" >&2
  exit 1
fi

rm -rf "$STAGING"
mkdir -p "$STAGING" "$DIST"
cp target/release/figma2penpot "$STAGING/figma2penpot"
cp target/release/f2p-cli "$STAGING/f2p-cli"
cp README.md "$STAGING/README.md"

ARCH="$(uname -m)"
ARCHIVE="$DIST/figma2penpot-$VERSION-linux-$ARCH.tar.gz"
tar -czf "$ARCHIVE" -C "$STAGING" figma2penpot f2p-cli README.md
rm -rf "$STAGING"
(cd "$DIST" && sha256sum "$(basename "$ARCHIVE")" > "$(basename "$ARCHIVE").sha256")
echo "wrote $ARCHIVE"
