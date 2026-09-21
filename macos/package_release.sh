#!/usr/bin/env bash
# Antigravity Unlocker - macOS Release Packager
# Packages Antigravity Unlocker.app into DMG and ZIP distributions in dist/
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

# 1. Ensure fresh app build
"$DIR/macos/build_app.sh"

DIST_DIR="$DIR/dist"
mkdir -p "$DIST_DIR"
rm -f "$DIST_DIR"/*

APP_BUNDLE="$DIR/target/release/Antigravity Unlocker.app"
DMG_PATH="$DIST_DIR/Antigravity-Unlocker-macOS.dmg"
ZIP_PATH="$DIST_DIR/Antigravity-Unlocker-macOS.zip"

echo "==> Создание ZIP-архива..."
(cd "$DIR/target/release" && zip -r -y -q "$ZIP_PATH" "Antigravity Unlocker.app")

echo "==> Создание DMG-образа с помощью create-dmg..."
create-dmg \
  --volname "Antigravity Unlocker" \
  --volicon "$DIR/macos/AppIcon.icns" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 128 \
  --text-size 12 \
  --icon "Antigravity Unlocker.app" 150 190 \
  --app-drop-link 450 190 \
  --hide-extension "Antigravity Unlocker.app" \
  --no-internet-enable \
  "$DMG_PATH" \
  "$APP_BUNDLE"

echo
echo "============================================================"
echo " Релизные пакеты macOS успешно собраны в: $DIST_DIR"
echo "============================================================"
ls -lh "$DIST_DIR"
echo
shasum -a 256 "$DIST_DIR"/*
