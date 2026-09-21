#!/usr/bin/env bash
# Antigravity Unlocker - macOS App Bundle Builder
# Builds the native release binary and packages it into Antigravity Unlocker.app
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

# Определение полной версии релиза
if grep -q 'VERSION = "' "$DIR/build_rust.py" 2>/dev/null; then
    VERSION=$(grep 'VERSION = "' "$DIR/build_rust.py" | head -n1 | cut -d '"' -f2)
else
    VERSION=$(grep '^version =' "$DIR/Cargo.toml" | head -n1 | cut -d '"' -f2)
fi
export AG_FULL_VERSION="$VERSION"
echo "==> Версия приложения: $VERSION (AG_FULL_VERSION=$AG_FULL_VERSION)"

echo "==> Сборка релизного бинарника для macOS (Apple Silicon / Intel)..."
cargo build --release

APP_NAME="Antigravity Unlocker.app"
APP_DIR="$DIR/target/release/$APP_NAME"
CONTENTS="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

echo "==> Создание структуры бандла $APP_NAME..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

# Копирование исполняемого файла
cp "$DIR/target/release/ag_unlocker" "$MACOS_DIR/ag_unlocker"
chmod +x "$MACOS_DIR/ag_unlocker"

# Копирование иконки приложения
if [ -f "$DIR/macos/AppIcon.icns" ]; then
    cp "$DIR/macos/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
fi

# Создание Info.plist
cat > "$CONTENTS/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Antigravity Unlocker</string>
    <key>CFBundleDisplayName</key>
    <string>Antigravity Unlocker</string>
    <key>CFBundleIdentifier</key>
    <string>com.antigravity.unlocker</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleExecutable</key>
    <string>ag_unlocker</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
EOF

echo "==> Выполнение ad-hoc подписи (codesign)..."
codesign --force --deep -s - "$APP_DIR"

echo "==> Готово! Бандл создан в: $APP_DIR"
echo "Для установки в систему выполните: ./macos/install.sh"
