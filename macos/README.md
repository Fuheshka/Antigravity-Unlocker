# Antigravity Unlocker для macOS

Нативная версия Antigravity Unlocker для macOS с полной поддержкой процессоров Apple Silicon (M1 / M2 / M3 / M4) и Intel (x86_64).

---

## Возможности на macOS

* **Нативный Mach-O бинарник:** компилируется без прослоек эмуляции, графический интерфейс работает через Metal / OpenGL (eframe/egui).
* **Автоматический патч Language Server & CLI:** находит и патчит `language_server_macos_arm`, `language_server_macos_x64` и `agy` в `/Applications`, `~/Applications` и `~/.gemini/bin`.
* **Автоматический ad-hoc codesign:** после модификации бинарника сразу выполняется `codesign --force -s -`, предотвращая сбой AMFI (`SIGKILL`) на Apple Silicon.
* **Фоновый сервис LaunchAgent (без root):** локальный CONNECT-прокси автоматически запускается при входе в систему через `~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist`. Права администратора (`sudo`) не требуются.
* **Бесшовная передача окружения:** переменная `AG_LS_PROXY` регистрируется в сессии `launchd` через `launchctl setenv` (наследуется всеми GUI-приложениями, запущенными из Finder / Dock / Spotlight) и прописывается в `~/.zprofile` для терминальных сессий.

---

## Быстрый запуск и установка

### Способ 1. Установка приложения в `/Applications` (Рекомендуется)

Запустите скрипт установки из корня репозитория:

```bash
./macos/install.sh
```

Скрипт автоматически соберет проект, сформирует бандл `Antigravity Unlocker.app`, подпишет его и установит в папку «Программы» (`/Applications`). После этого приложение доступно в Spotlight, Launchpad и Finder.

### Способ 2. Запуск через скрипт

```bash
./macos/launch.sh
```

### Способ 3. Ручная сборка

```bash
cargo build --release
./target/release/ag_unlocker
```

---

## Архитектура работы на macOS

```
+-------------------------------------------------------------+
|                     macOS User Session                      |
|                                                             |
|  +---------------------------+   launchctl setenv           |
|  |   Antigravity Unlocker    |--------------------------+   |
|  |       (GUI / App)         |   Sync ~/.zprofile       |   |
|  +---------------------------+                          |   |
|               |                                         v   |
|        Патч бинарников             +--------------------+   |
|        + codesign                  |    AG_LS_PROXY     |   |
|               v                    |  127.0.0.1:53129   |   |
|  +---------------------------+     +--------------------+   |
|  |  language_server_macos_*  |               |              |
|  |     (Antigravity IDE)     |<--------------+              |
|  +---------------------------+  (через env)                 |
|               |                                             |
|               v                                             |
|  +---------------------------+                              |
|  |   LaunchAgent (launchd)   |                              |
|  |     ag_proxy --proxy      |                              |
|  +---------------------------+                              |
+-------------------------------------------------------------+
```

1. **Патч сигнатур:** заменяет `ineligible` на `inexigible` и `https_proxy` на `AG_LS_PROXY` без изменения длины строк.
2. **Codesign:** восстанавливает валидность Mach-O подписи.
3. **Локальный прокси:** маршрутизирует обращения к гейт-хостам через разрешенные точки выхода.

---

## Решение проблем (Troubleshooting)

### Предупреждение Gatekeeper («Не удается открыть программу...»)

Если macOS блокирует первый запуск бандла:
1. Зайдите в **Системные настройки** -> **Конфиденциальность и безопасность** (Privacy & Security).
2. Внизу в блоке «Безопасность» нажмите **«Подтвердить вход»** (Open Anyway).
3. Либо выполните в терминале:
   ```bash
   xattr -dr com.apple.quarantine "/Applications/Antigravity Unlocker.app"
   ```

### Сброс и удаление прокси

При выключении переключателей в интерфейсе приложения или удалении LaunchAgent:
```bash
launchctl unsetenv AG_LS_PROXY
launchctl unload -w ~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist 2>/dev/null || true
rm -f ~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist
```
