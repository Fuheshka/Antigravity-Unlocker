#!/bin/sh
# Antigravity Unlocker — терминальный режим одной командой (macOS & Linux):
#
#   curl -fsSL https://raw.githubusercontent.com/Fuheshka/Antigravity-Unlocker/main/tui.sh | sh
#
# Берёт последний релиз с GitHub, кладёт программу в
# ~/.local/share/agunlocker/ и запускает её в этом терминале. Повторный запуск
# той же командой скачивает заново, только если вышла новая версия. Без root.
set -eu

OS="$(uname -s)"
ARCH="$(uname -m)"
DIR="${XDG_DATA_HOME:-$HOME/.local/share}/agunlocker"
BIN="$DIR/ag_unlocker"

# На macOS: если приложение уже установлено в /Applications, запускаем напрямую
if [ "$OS" = "Darwin" ]; then
    if [ -x "/Applications/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker" ]; then
        exec "/Applications/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker" --tui </dev/tty
    elif [ -x "$HOME/Applications/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker" ]; then
        exec "$HOME/Applications/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker" --tui </dev/tty
    fi
    REPO="Fuheshka/Antigravity-Unlocker"
else
    case "$ARCH" in
        x86_64 | amd64) ;;
        *)
            echo "Antigravity Unlocker собран только для x86-64, а здесь $ARCH." >&2
            exit 1
            ;;
    esac
    REPO="confeden/Antigravity"
fi

# Получение последнего тега через редирект /releases/latest (без API лимитов)
TAG=""
if URL=$(curl -fsSLo /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest"); then
    TAG="${URL##*/}"
fi
VER="${TAG#v}"

# Теги форка для macOS: 2.26.0-macos.1 — суффикс в проверку не идёт.
case "${VER%-macos.*}" in
    "" | *[!0-9._]*)
        if [ -x "$BIN" ]; then
            echo "GitHub не ответил — запускаю уже скачанную версию." >&2
        else
            echo "Не удалось узнать последнюю версию на github.com/$REPO." >&2
            exit 1
        fi
        ;;
    *)
        if [ ! -x "$BIN" ] || [ "$(cat "$BIN.version" 2>/dev/null)" != "$VER" ]; then
            echo "Скачиваю Antigravity Unlocker $VER…" >&2
            mkdir -p "$DIR"
            TMP=$(mktemp -d)
            trap 'rm -rf "$TMP"' EXIT
            if [ "$OS" = "Darwin" ]; then
                curl -fsSL "https://github.com/$REPO/releases/download/$TAG/Antigravity-Unlocker-macOS.zip" -o "$TMP/app.zip"
                unzip -q -o "$TMP/app.zip" -d "$TMP"
                mv -f "$TMP/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker" "$BIN"
                codesign --force -s - "$BIN" 2>/dev/null || true
            else
                # The tag spells a fourth number with a dot (v2.24.0.1), the asset with
                # an underscore (AG_2.24.0_1_linux.tar.gz): try both.
                ALT=$(echo "$VER" | sed -E 's/\.([0-9]+)$/_\1/')
                NAME=""
                for cand in "$VER" "$ALT"; do
                    if curl -fsIL "https://github.com/$REPO/releases/download/$TAG/AG_${cand}_linux.tar.gz" >/dev/null 2>&1; then
                        NAME="$cand"
                        break
                    fi
                done
                if [ -z "$NAME" ]; then
                    echo "В релизе $TAG нет сборки для Linux." >&2
                    exit 1
                fi
                curl -fsSL "https://github.com/$REPO/releases/download/$TAG/AG_${NAME}_linux.tar.gz" |
                    tar xz -C "$TMP"
                mv -f "$TMP/AG_${NAME}_linux/ag_unlocker" "$BIN"
            fi
            chmod +x "$BIN"
            echo "$VER" >"$BIN.version"
            rm -rf "$TMP"
            trap - EXIT
        fi
        ;;
esac

# `curl … | sh` передает скрипт в sh на stdin; нажатия клавиш берутся из терминала
exec "$BIN" --tui </dev/tty
