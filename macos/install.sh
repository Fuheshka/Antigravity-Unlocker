#!/usr/bin/env bash
# Antigravity Unlocker - macOS Installer
# Installs Antigravity Unlocker.app to /Applications (or ~/Applications)
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Antigravity Unlocker.app"
BUILT_APP="$DIR/target/release/$APP_NAME"

if [ ! -d "$BUILT_APP" ]; then
    echo "==> Бандл $APP_NAME не найден. Запуск сборки..."
    "$DIR/macos/build_app.sh"
fi

DEST_DIR="/Applications"
if [ ! -w "$DEST_DIR" ]; then
    DEST_DIR="$HOME/Applications"
    mkdir -p "$DEST_DIR"
fi

DEST_APP="$DEST_DIR/$APP_NAME"

echo "==> Установка в $DEST_APP..."
rm -rf "$DEST_APP"
cp -R "$BUILT_APP" "$DEST_APP"

# Снятие атрибута карантина Gatekeeper
xattr -dr com.apple.quarantine "$DEST_APP" 2>/dev/null || true

# Повторная ad-hoc подпись после копирования
codesign --force --deep -s - "$DEST_APP" 2>/dev/null || true

echo
echo "============================================================"
echo " Установка Antigravity Unlocker на macOS успешно завершена!"
echo " Расположение: $DEST_APP"
echo "============================================================"
echo "Теперь вы можете запускать программу:"
echo " • Через Spotlight / Raycast: просто введите «Antigravity Unlocker»"
echo " • Из Launchpad или папки «Программы» (Applications)"
echo " • Либо из консоли: open -a \"$DEST_APP\""
echo
