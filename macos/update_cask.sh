#!/usr/bin/env bash
# Antigravity Unlocker - Homebrew Cask Generator
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ZIP_PATH="$DIR/dist/Antigravity-Unlocker-macOS.zip"

if [ ! -f "$ZIP_PATH" ]; then
    echo "Ошибка: $ZIP_PATH не найден. Сначала выполните ./macos/package_release.sh" >&2
    exit 1
fi

SHA=$(shasum -a 256 "$ZIP_PATH" | awk '{print $1}')
TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v2.17.0.3-macos.1")
VER="${TAG#v}"

CASK_FILE="$DIR/macos/antigravity-unlocker.rb"

cat > "$CASK_FILE" << EOF
cask "antigravity-unlocker" do
  version "$VER"
  sha256 "$SHA"

  url "https://github.com/Fuheshka/Antigravity-Unlocker/releases/download/v#{version}/Antigravity-Unlocker-macOS.zip"
  name "Antigravity Unlocker"
  desc "Native macOS unlocker tool for Google Antigravity IDE and CLI"
  homepage "https://github.com/Fuheshka/Antigravity-Unlocker"

  depends_on macos: ">= :big_sur"

  app "Antigravity Unlocker.app"
  binary "#{appdir}/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker", target: "ag_unlocker"

  zap trash: [
    "~/Library/Application Support/AGUnlocker",
    "~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist",
  ]
end
EOF

echo "✓ Cask обновлен: $CASK_FILE (версия $VER, sha256 $SHA)"
