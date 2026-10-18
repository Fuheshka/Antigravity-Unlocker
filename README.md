# Antigravity Unlocker для macOS

[![Platform](https://img.shields.io/badge/Platform-macOS-blue?logo=apple&logoColor=white)](README.md)
[![macOS Architecture](https://img.shields.io/badge/Architecture-Apple%20Silicon%20(arm64)%20%7C%20Intel%20(x86__64)-success)](macos/README.md)
[![Rust](https://img.shields.io/badge/Rust-2021%20edition-orange?logo=rust)](Cargo.toml)
[![UI](https://img.shields.io/badge/UI-egui%20%2F%20eframe-purple)](src/gui/mod.rs)
[![Downloads](https://img.shields.io/github/downloads/Fuheshka/Antigravity-Unlocker/total?label=Downloads&logo=github)](https://github.com/Fuheshka/Antigravity-Unlocker/releases)

Нативный порт [Antigravity Unlocker](https://github.com/confeden/Antigravity) для macOS (Apple Silicon и Intel). С ним аккаунт Google из любой страны, в том числе из России, может пользоваться Antigravity 2.0, Antigravity IDE и Antigravity CLI без VPN и без смены региона. Настройка делается один раз для компьютера. После обновления Antigravity анлокер сам патчит его заново.

## Установка

```bash
brew install --cask fuheshka/tap/antigravity-unlocker
```

Либо скачайте `.dmg` в [Releases](https://github.com/Fuheshka/Antigravity-Unlocker/releases/latest) и перетащите приложение в «Программы».

1. Установите Antigravity с [официального сайта](https://antigravity.google/download).
2. Запустите Antigravity Unlocker. Права администратора не нужны.
3. Введите бесплатный ключ из закреплённых сообщений группы [t.me/nova_txt](https://t.me/nova_txt/69864). Для каждой версии анлокера свой ключ, после обновления нужен новый.
4. Нажмите **«Включить всё»**. Antigravity закроется на пару секунд.
5. Запустите Antigravity, войдите в аккаунт и напишите что-нибудь в чат. Карточка вверху окна станет зелёной: **«Работает. Модель ответила …»**.

Любой переключатель можно выключить: выключение и есть откат этой части.

Если macOS ругается на непроверенного разработчика:

```bash
xattr -cr "/Applications/Antigravity Unlocker.app"
```

## Терминальная версия (TUI)

```bash
curl -fsSL https://raw.githubusercontent.com/Fuheshka/Antigravity-Unlocker/main/tui.sh | sh
```

Стрелки выбирают пункт, Пробел включает или выключает, `q` выходит.

## Сборка из исходников

```bash
git clone https://github.com/Fuheshka/Antigravity-Unlocker.git
cd Antigravity-Unlocker
./macos/install.sh
```

## С VPN и без

Ничего настраивать не нужно. Antigravity соединяется с серверами Google через локальную службу, и VPN эти соединения не перехватывает. Путь выбирается по тому, на каком модель реально ответила.

## Если не работает

Посмотрите на карточку вверху окна: там написано, что мешает и что делать. Если не помогло, нажмите «Скопировать отчёт» и пришлите его в группу.

## Что меняется в системе

1. **Language Server и CLI** (`language_server*`, `agy`): меняются две строки одинаковой длины, `ineligible` → `inexigible` и `https_proxy` → `AG_LS_PROXY`. Потом бинарник переподписывается ad-hoc (`codesign -s -`), иначе macOS не даст его запустить.
2. **Локальный прокси** `127.0.0.1:53129`, работает как LaunchAgent `~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist`. На него указывает только `AG_LS_PROXY`, а системный `HTTPS_PROXY` не меняется. TLS не расшифровывается, сертификаты не ставятся.
3. DNS и сетевые настройки macOS не меняются.

---

* Оригинал: Brent ([@nova_txt](https://t.me/nova_txt)), [confeden/Antigravity](https://github.com/confeden/Antigravity). ☕ [Отблагодарить](https://nova-app.eu/donate)
* macOS-порт: [@Fuheshka](https://github.com/Fuheshka) · [Telegram](https://t.me/fuheshka) · [me@kuviko.ru](mailto:me@kuviko.ru)
