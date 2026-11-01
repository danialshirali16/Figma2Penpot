#!/usr/bin/env bash
# Build a macOS .app bundle from an already-built release binary and zip it.
# Usage: scripts/package-macos.sh [version] [binary] [dist-dir]
set -euo pipefail

VERSION="${1:-0.1.0}"
BINARY="${2:-target/release/figma2penpot}"
DIST="${3:-dist}"

if [[ ! -x "$BINARY" ]]; then
  echo "missing executable: $BINARY (run cargo build --release -p desktop-app)" >&2
  exit 1
fi

APP_NAME="Figma 2 Penpot.app"
APP="$DIST/$APP_NAME"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$DIST"
cp "$BINARY" "$APP/Contents/MacOS/figma2penpot"
chmod +x "$APP/Contents/MacOS/figma2penpot"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Figma 2 Penpot</string>
  <key>CFBundleDisplayName</key><string>Figma 2 Penpot</string>
  <key>CFBundleIdentifier</key><string>app.figma2penpot.desktop</string>
  <key>CFBundleExecutable</key><string>figma2penpot</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

ARCH="$(uname -m)"
ZIP="$DIST/figma2penpot-$VERSION-macos-$ARCH.zip"
rm -f "$ZIP"
(cd "$DIST" && ditto -c -k --sequesterRsrc --keepParent "$APP_NAME" "$(basename "$ZIP")")
(cd "$DIST" && shasum -a 256 "$(basename "$ZIP")" > "$(basename "$ZIP").sha256")
echo "wrote $ZIP"
