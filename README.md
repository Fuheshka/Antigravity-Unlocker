# Antigravity Unlocker (macOS, Windows, Linux)

[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?logo=apple&logoColor=white)](README.md)
[![macOS Architecture](https://img.shields.io/badge/Architecture-Apple%20Silicon%20(arm64)%20%7C%20Intel%20(x86__64)-success)](macos/README.md)
[![Rust](https://img.shields.io/badge/Rust-2021%20edition-orange?logo=rust)](Cargo.toml)
[![Tests](https://img.shields.io/badge/Tests-215%20passed-brightgreen)](Cargo.toml)
[![License](https://img.shields.io/badge/License-Open%20Source-blue)](Cargo.toml)
[![UI](https://img.shields.io/badge/UI-egui%20%2F%20eframe-purple)](src/gui/mod.rs)

Разблокировка Antigravity IDE и CLI в России и регионах с ограничениями без VPN, без смены региона аккаунта Google и без внешних платных сервисов.

Кроссплатформенная утилита для **macOS** (Apple Silicon M1/M2/M3/M4 и Intel x86_64), **Windows** (x86_64) и **Linux** (x86_64, aarch64), позволяющая любому аккаунту Google использовать все возможности экосистемы Antigravity.

Анлокер работает как нативное графическое окно или в терминале через TUI. Вверху одна карточка отвечает на главный вопрос: **работает ли сейчас Antigravity**, а если нет - что нажать. Процедура разблокировки проводится один раз для компьютера. После этого можно авторизовываться с любыми аккаунтами Google из любых стран. При обновлении Antigravity анлокер пропатчит его автоматически. Любой переключатель можно выключить: выключение и есть откат этой части, отдельного пункта «откатить всё» нет.

> [!NOTE]
> Начиная с версии **2.12.2** анлокер работает как нативное графическое приложение (GUI на базе egui/eframe) с понятными переключателями, а также поддерживает быстрый терминальный режим (TUI). Все операции выполняются один раз. При обновлении Antigravity утилита автоматически повторно накладывает патчи. Все изменения полностью обратимы в один клик.

---

## Поддерживаемые платформы

| Платформа | Архитектура | Механизм обхода | Фоновый сервис | Права администратора |
| :--- | :--- | :--- | :--- | :--- |
| **macOS** | Apple Silicon (arm64), Intel (x86_64) | Local CONNECT Proxy + бинарный патч + auto-codesign | LaunchAgent (`launchd`) | **Не требуются** (Unprivileged) |
| **Windows** | x86_64 | NRPT DNS + Local CONNECT Proxy | Служба `%ProgramData%\AGUnlocker` / Logon Task | Нужны для правил NRPT |
| **Linux** | x86_64, aarch64 | Local CONNECT Proxy + бинарный патч | systemd user unit | **Не требуются** (Unprivileged) |

---

## Как разблокировать Antigravity?
1) Скачиваете Antigravity 2.0 / IDE / CLI с официального сайта [https://antigravity.google/download](https://antigravity.google/download) и устанавливаете.
2) Запускаете анлокер (на Windows: от имени администратора, на macOS и Linux: обычным запуском без sudo).
3) Вводите бесплатный лицензионный ключ (кнопка под полем открывает комнату в Telegram, где он лежит в закрепленном сообщении).
4) Нажимаете **«Включить всё»** в карточке вверху (Antigravity при этом закроется на пару секунд).
5) Запускаете Antigravity, входите в аккаунт Google и пишете что-нибудь в чат: карточка в анлокере станет зеленой: **«Работает. Модель ответила …»**.

---

## Архитектура работы

```mermaid
flowchart LR
    subgraph Antigravity["Antigravity IDE & CLI"]
        Client["Language Server / agy"]
        PatchedBin["Бинарный патч:<br/>ineligible -> inexigible<br/>https_proxy -> AG_LS_PROXY"]
    end

    subgraph System["Локальная система"]
        Env["Переменная окружения:<br/>AG_LS_PROXY=http://127.0.0.1:53129"]
        ProxyDaemon["Фоновый демон CONNECT-прокси<br/>(macOS: launchd LaunchAgent<br/>Linux: systemd --user<br/>Windows: Background Task / Service)"]
    end

    subgraph Google["Google Cloud"]
        Gate["cloudcode-pa.googleapis.com"]
        Models["Gemini / Cloud Code Backend"]
    end

    Client --> PatchedBin
    PatchedBin --> Env
    Env --> ProxyDaemon
    ProxyDaemon --> Gate
    Gate --> Models

    classDef default fill:#1e1e2e,stroke:#89b4fa,stroke-width:2px,color:#ffffff;
    classDef highlight fill:#313244,stroke:#a6e3a1,stroke-width:2px,color:#ffffff;
    class PatchedBin,ProxyDaemon highlight;
```

### Состояния карточки и возможности
* **Карточка вверху** - одно из состояний:
  * **Работает** - модель в Antigravity ответила (время и путь, которым шел запрос). Зеленой карточка становится только после настоящего ответа модели, а не просто от того, что ошибок давно не было;
  * **Всё включено** - обход готов, осталось написать что-нибудь в Antigravity;
  * **Чиним** - только что была ошибка 400; обход сам переключил путь, отправьте сообщение еще раз;
  * **Нужна проверка** - ошибка была давно, и после нее ответов не было;
  * **Нужно действие** - с кнопкой: «Включить всё», «Перезапустить от имени администратора» или «Починить».
  Кнопка **«Скопировать отчёт»** копирует в буфер обмена диагностику для разбора проблем.
* **Разблокировать вход в аккаунт** - бинарный патч клиента.
* **Снять ошибку 400 в чате с ИИ** - автоматический подбор рабочего пути до серверов Google с переключением при сбоях.
* **Автопатч** - автоматически накладывает патч на найденный Antigravity сразу после установки и обновлений.
* **Настройки для опытных** (свернуты) - управление DNS, локальным прокси, встроенными выходами и внешними HTTP-прокси.

## С VPN и без

Ничего настраивать не нужно - ни с VPN, ни без него, ни с DNS-AI или любым другим DNS в системе:
* Соединения Antigravity с серверами Google всегда идут через службу анлокера на этом компьютере, поэтому VPN их не перехватывает.
* Свои соединения с сервисами разблокировки служба при включенном VPN отправляет мимо туннеля, напрямую через провайдера.
* Путь выбирается по тому, **на каком реально ответила модель**, а не просто по пингу.

## Терминальная версия (TUI) - одной командой

Анлокер прямо в терминале: для сервера по SSH, для систем без графической оболочки или для быстрого управления.

**Linux / macOS** - в терминале:
```bash
curl -fsSL https://raw.githubusercontent.com/confeden/Antigravity/main/tui.sh | sh
```

**Windows** - в PowerShell (Win+X → «Терминал (администратор)»):
```powershell
irm https://raw.githubusercontent.com/confeden/Antigravity/main/tui.ps1 | iex
```

Вставьте ключ и нажмите Enter. Стрелки: выбор, Пробел: включить или выключить, `q`: выход.

## Если не работает

Смотрите на карточку вверху окна: если обходу мешает антивирус, файрвол или другая программа, там написано, что именно и что сделать. Не помогло: нажмите «Скопировать отчёт» и пришлите его в группу поддержки.

Окно открывается на любой Windows 10/11 без дополнительных библиотек. Если графический драйвер не справляется, анлокер автоматически переключается на программную отрисовку, а если окно не открылось совсем - запускается терминальная версия.

# Работает ли анлокер для 2+ версии?
Да, он работает для всех агентских программ для кодинга от Google:
- Antigravity 2.0
- Antigravity IDE
- Antigravity CLI

---

## Что именно анлокер меняет в системе?

Все внесенные изменения строго обратимы:

1. **Бинарник Language Server / CLI** (`language_server*`, `agy`):
   - Переименовываются две строки фиксированной длины: поле protobuf-дескриптора (`ineligible` → `inexigible`) и имя переменной прокси (`https_proxy` → `AG_LS_PROXY`).
   - Размер и структура файла не меняются, откат побайтово точный.
2. **Изолированная переменная среды `AG_LS_PROXY`**:
   - Указывает на локальный прокси `127.0.0.1:53129`. В отличие от глобальной `HTTPS_PROXY`, не затрагивает сторонние утилиты, Git, браузеры и пакетные менеджеры.
   - На macOS: устанавливается через `launchctl setenv AG_LS_PROXY` и синхронизируется с шелл-профилями (`~/.zprofile`, `~/.zshrc`).
3. **Фоновый сервис прокси**:
   - macOS: LaunchAgent `~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist`.
   - Linux: пользовательский systemd unit `~/.config/systemd/user/ag_unlocker_proxy.service`.
   - Windows: служба `%ProgramData%\AGUnlocker\ag_dns.exe` / планировщик задач.
4. **Сетевые правила DNS (Windows)**:
   - Только 2 конкретных хоста (`cloudcode-pa.googleapis.com` и `daily-cloudcode-pa.googleapis.com`) направляются на локальную службу.
   - На macOS и Linux системные настройки DNS не затрагиваются.
5. **Автоматический ad-hoc codesign на macOS**:
   - На Apple Silicon изменение любого байта в Mach-O файле нарушает цифровую подпись, из-за чего подсистема AMFI моментально убивает процесс (`SIGKILL`). Анлокер автоматически выполняет ad-hoc переподпись (`codesign --force -s -`), сохраняя запуск бинарников.

---

## Быстрый старт на macOS

### Вариант 1: Загрузка готового установщика .dmg (Рекомендуется)

Скачайте готовый `.dmg` или `.zip` из раздела [Releases](https://github.com/Fuheshka/Antigravity-Unlocker/releases/latest):
1. Откройте `Antigravity-Unlocker-macOS.dmg`.
2. Перетащите `Antigravity Unlocker.app` в папку **«Программы»** (`/Applications`).
3. Запустите приложение из Spotlight или Launchpad.

### Вариант 2: Сборка и установка из исходников в 1 команду

```bash
# Клонируйте репозиторий:
git clone https://github.com/Fuheshka/Antigravity-Unlocker.git
cd Antigravity-Unlocker

# Запустите скрипт сборки и установки в /Applications:
./macos/install.sh
```

Скрипт автоматически:
1. Соберет оптимизированный бинарник под текущую архитектуру (arm64 или x86_64).
2. Сформирует нативный macOS бандл `Antigravity Unlocker.app` с иконкой и `Info.plist`.
3. Применит ad-hoc цифровую подпись.
4. Скопирует приложение в `/Applications` (или в `~/Applications`).
5. Снимет флаги карантина macOS Gatekeeper.

### Вариант 3: Запуск без установки в `/Applications`

```bash
./macos/launch.sh
```

### Вариант 4: Сборка напрямую через Cargo

```bash
cargo build --release
./target/release/ag_unlocker
```

> [!TIP]
> Если macOS при первом запуске предупреждает о непроверенном разработчике, выполните:
> ```bash
> xattr -cr "/Applications/Antigravity Unlocker.app"
> ```

---

## Быстрый старт на Windows и Linux

### Windows:
1. Скачайте `ag_unlocker.exe` из раздела [Releases](https://github.com/confeden/Antigravity/releases).
2. Нажмите правой кнопкой мыши → **«Запуск от имени администратора»** (права требуются для добавления правил NRPT DNS).
3. Введите лицензионный ключ и включите оба тумблера.

### Linux:
1. Соберите проект: `cargo build --release`.
2. Запустите `./target/release/ag_unlocker` (или запустите `tui.sh` / `launch.sh`).
3. Права root не требуются: фоновый сервис регистрируется как пользовательский systemd unit (`systemctl --user`).

---

## Где взять ключ доступа?

Для активации функций разблокировки используется бесплатный ключ доступа из официального сообщества:
- Telegram-канал (в закрепленных сообщениях): [t.me/nova_txt](https://t.me/nova_txt/69864)
- В интерфейсе программы предусмотрена удобная кнопка, открывающая нужный пост в один клик.

---

## Проверка и диагностика

### Проверка работы локального прокси
В окне терминала выполните команду:
```bash
curl -I -x http://127.0.0.1:53129 https://www.google.com
```
Ожидаемый ответ: `HTTP/1.1 200 Connection Established` и затем `HTTP/2 200`.

### Проверка статуса LaunchAgent на macOS
```bash
launchctl list | grep antigravity
```

### Ручной перезапуск или отключение LaunchAgent
```bash
# Остановить и выгрузить сервис:
launchctl unload ~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist

# Запустить снова:
launchctl load ~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist
```

---

## Разработка и тестирование

Проект написан на чистом Rust (2021 edition) без тяжелых зависимостей.

```bash
# Запуск полного набора unit и интеграционных тестов:
cargo test

# Сборка debug-версии:
cargo build

# Сборка релизной версии под текущую ОС:
cargo build --release
```

---

## Безопасность и открытость

- **100% Open Source:** весь исходный код доступен для независимого аудита прямо в этом репозитории.
- **Никакой телеметрии:** программа не собирает статистику, не передает токены авторизации Google и не производит скрытых сетевых соединений.
- **Откат в один клик:** отключение любого тумблера в GUI мгновенно возвращает оригинальные файлы и удаляет системные службы.

Официальный Telegram проекта: [t.me/nova_txt](https://t.me/nova_txt)
