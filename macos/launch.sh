#!/usr/bin/env bash
# Antigravity Unlocker - macOS CLI / Direct Launcher
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$DIR/target/release/ag_unlocker"

if [ ! -x "$BIN" ]; then
    echo "==> Бинарник не найден. Сборка через cargo..."
    (cd "$DIR" && cargo build --release)
fi

exec "$BIN" "$@"
