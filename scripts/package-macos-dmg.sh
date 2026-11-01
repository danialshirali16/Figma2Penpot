#!/usr/bin/env bash
# Build a compressed DMG from the packaged macOS .app.
# Usage: scripts/package-macos-dmg.sh [version] [dist-dir]
set -euo pipefail

VERSION="${1:-0.1.0}"
DIST="${2:-dist}"
APP="$DIST/Figma 2 Penpot.app"

if [[ ! -d "$APP" ]]; then
  echo "missing $APP (run scripts/package-macos.sh first)" >&2
  exit 1
fi

ARCH="$(uname -m)"
DMG="$DIST/figma2penpot-$VERSION-macos-$ARCH.dmg"
rm -f "$DMG"
hdiutil create \
  -volname "Figma 2 Penpot" \
  -srcfolder "$APP" \
  -ov -format UDZO \
  "$DMG" >/dev/null
(cd "$DIST" && shasum -a 256 "$(basename "$DMG")" > "$(basename "$DMG").sha256")
echo "wrote $DMG"
