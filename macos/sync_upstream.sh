#!/usr/bin/env bash
# Antigravity Unlocker - Upstream Sync & Release Helper
# Checks upstream changes, stages clean merges, and prepares release builds.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

UPSTREAM_REMOTE="upstream"
UPSTREAM_BRANCH="main"

echo "==> Проверка удалённого репозитория upstream..."
if ! git remote get-url "$UPSTREAM_REMOTE" &>/dev/null; then
    echo "    Добавление remote upstream (https://github.com/confeden/Antigravity)..."
    git remote add "$UPSTREAM_REMOTE" "https://github.com/confeden/Antigravity.git"
fi

echo "==> Получение свежих данных из upstream..."
git fetch "$UPSTREAM_REMOTE" --tags

LOCAL_REV=$(git rev-parse HEAD)
UPSTREAM_REV=$(git rev-parse "$UPSTREAM_REMOTE/$UPSTREAM_BRANCH")

if [ "$LOCAL_REV" = "$UPSTREAM_REV" ]; then
    echo "✓ Форк полностью синхронизирован с upstream/main (коммит $LOCAL_REV)."
    exit 0
fi

AHEAD_COUNT=$(git rev-list --count "$UPSTREAM_REV..$LOCAL_REV" || echo "0")
BEHIND_COUNT=$(git rev-list --count "$LOCAL_REV..$UPSTREAM_REV" || echo "0")

echo "============================================================"
echo " Статус синхронизации: $BEHIND_COUNT новых коммитов в upstream"
echo " Локальных коммитов впереди: $AHEAD_COUNT"
echo "============================================================"

if [ "$BEHIND_COUNT" -gt 0 ]; then
    echo "Список новых коммитов из upstream:"
    git log "$LOCAL_REV..$UPSTREAM_REV" --oneline --no-merges
    echo

    LATEST_TAG=$(git describe --tags --abbrev=0 "$UPSTREAM_REV" 2>/dev/null || echo "v-latest")
    echo "Последний апстрим-тег: $LATEST_TAG"

    if [ "${1:-}" = "--merge" ]; then
        SYNC_BRANCH="sync-upstream-$LATEST_TAG"
        echo "==> Создание рабочей ветки $SYNC_BRANCH..."
        git checkout -b "$SYNC_BRANCH"
        echo "==> Слияние без автокоммита (--no-commit)..."
        git merge "$UPSTREAM_REMOTE/$UPSTREAM_BRANCH" --no-commit
        echo
        echo "Слияние подготовленно в ветке $SYNC_BRANCH."
        echo "Запустите тесты: cargo test"
    else
        echo "Подсказка: Для запуска слияния выполните:"
        echo "  ./macos/sync_upstream.sh --merge"
    fi
else
    echo "Новых коммитов в upstream нет. Форк актуален."
fi
