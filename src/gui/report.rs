//! «Скопировать отчёт»: everything needed to say why Antigravity is or is not
//! answering, in one paste.
//!
//! «Поставил анлокер, но не помогает что-то» plus a screenshot of a green card
//! was undiagnosable (G50): the one fact that mattered - the client was not
//! going through us at all - was nowhere on screen. This puts every fact the
//! window and the relay have into plain text a user can send to the group.
//!
//! Nothing secret goes in: paths are masked the way the window shows them, the
//! built-in exits are named only as «встроенный выход» (I46), and the relay's
//! log - which is included - never names an address of ours in the first place.

use std::fmt::Write as _;

use crate::gate::{Report, View};
use crate::ops::{State, Status};
use crate::utils::mask_path;

/// How much of the relay's own log goes in: enough to cover the last few
/// minutes of a busy session, short enough to paste into a chat.
const LOG_LINES: usize = 60;

pub fn build(status: Option<&Status>, view: &View) -> String {
    let mut out = String::new();
    let now = crate::utils::local_clock().map_or_else(String::new, |c| c.hms());
    let _ = writeln!(
        out,
        "Antigravity Unlocker v{} — отчёт {}",
        crate::update::current_version(),
        now
    );

    match status {
        Some(s) => status_part(&mut out, s),
        None => {
            let _ = writeln!(out, "Состояние системы ещё не прочитано.");
        }
    }

    let _ = writeln!(out, "\n— Antigravity (его собственные логи)");
    match view.answered {
        Some(a) => {
            let _ = writeln!(
                out,
                "Последний ответ модели: {} (за 12 ч: {})",
                super::status::ago_text(a.ago),
                a.count
            );
        }
        None => {
            let _ = writeln!(out, "Ответов модели за 12 ч в логах нет.");
        }
    }
    match view.refused_long {
        Some(r) => {
            let _ = writeln!(
                out,
                "Последняя ошибка 400: {} (строк за 10 мин: {})",
                super::status::ago_text(r.ago),
                view.seen.map_or(0, |s| s.count)
            );
        }
        None => {
            let _ = writeln!(out, "Ошибок 400 за 12 ч в логах нет.");
        }
    }

    // Fresh from the file, not the window's copy: the route table's ages move
    // every pass and the watcher does not wake the window for them.
    let _ = writeln!(out, "\n— Служба обхода (её запись)");
    match crate::gate::read() {
        Some(r) => relay_part(&mut out, &r),
        None => {
            let _ = writeln!(out, "Записи нет — служба не запущена или старая.");
        }
    }

    let _ = writeln!(out, "\n— Журнал службы (последние строки)");
    let log = std::fs::read_to_string(crate::dns_forwarder::log_path()).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    let from = lines.len().saturating_sub(LOG_LINES);
    // The relay logs the user's own proxy as `user:***@host:port`: their login
    // and their server, on its way into a public chat. Masked here, once.
    let own = crate::upstream::configured().map(|u| u.display());
    for line in &lines[from..] {
        let line = match &own {
            Some(own) if !own.is_empty() => line.replace(own.as_str(), "<ваш прокси>"),
            _ => line.to_string(),
        };
        let _ = writeln!(out, "{line}");
    }
    if lines.is_empty() {
        let _ = writeln!(out, "(пусто)");
    }
    out
}

fn onoff(s: &State) -> String {
    match s {
        State::On => "вкл".to_string(),
        State::Off => "выкл".to_string(),
        State::Partial(n) => format!("частично ({n})"),
        State::OffNote(n) => format!("выкл ({n})"),
        State::Blocked(n) => format!("недоступно ({n})"),
    }
}

fn status_part(out: &mut String, s: &Status) {
    let _ = writeln!(
        out,
        "Права администратора: {}",
        if s.admin { "да" } else { "нет" }
    );
    let _ = writeln!(out, "\n— Установки");
    for row in &s.installs {
        let path = row
            .path
            .as_ref()
            .map(|p| mask_path(&p.display().to_string()))
            .unwrap_or_else(|| "не найдена".to_string());
        let patched = match row.patched {
            Some(true) => "пропатчена",
            Some(false) => "НЕ пропатчена",
            None => "не проверена",
        };
        let _ = writeln!(out, "{}: {} — {}", row.label, path, patched);
    }
    let _ = writeln!(out, "\n— Переключатели");
    let _ = writeln!(out, "Разблокировка входа: {}", onoff(&s.client_patch));
    let _ = writeln!(out, "Автопатч: {}", onoff(&s.watchdog));
    let _ = writeln!(out, "Обход через DNS: {}", onoff(&s.dns));
    let _ = writeln!(out, "Локальный прокси: {}", onoff(&s.local_proxy));
    let _ = writeln!(out, "Встроенные выходы: {}", onoff(&s.builtin_exits));
    let _ = writeln!(out, "Свой прокси: {}", onoff(&s.own_proxy));
    let _ = writeln!(out, "Сверять TLS: {}", onoff(&s.verify_tls));
    let _ = writeln!(
        out,
        "Служба: {}{}; правила DNS: {}",
        if s.relay_running {
            "запущена"
        } else {
            "НЕ запущена"
        },
        if s.relay_outdated {
            ", устарела"
        } else {
            ""
        },
        if s.rules { "есть" } else { "НЕТ" }
    );
    if let Some(vpn) = s.vpn {
        let _ = writeln!(out, "VPN (по сокетам Antigravity): {:?}", vpn);
    }
    if let Some(eg) = s.relay_egress {
        let _ = writeln!(out, "Сокеты службы: {:?}", eg);
    }
}

fn relay_part(out: &mut String, r: &Report) {
    let _ = writeln!(
        out,
        "Версия службы: {} (эта программа ждёт {}), запись {} с назад",
        r.version,
        crate::dns_forwarder::RELAY_VERSION,
        r.age().as_secs()
    );
    let _ = writeln!(
        out,
        "Гейт-хосты через локальные адреса: {}",
        if r.loopback { "да" } else { "нет" }
    );
    let _ = writeln!(
        out,
        "VPN держит маршрут по умолчанию: {}{}",
        if r.tunnel { "да" } else { "нет" },
        if r.vpn_exit.is_empty() {
            String::new()
        } else {
            format!(", выход: {}", r.vpn_exit)
        }
    );
    let _ = writeln!(out, "Первый маршрут сейчас: {}", non_empty(&r.route));
    if let Some(ok) = &r.last_ok {
        let _ = writeln!(
            out,
            "Последний ответ модели, который видела служба: {} с назад, маршрут: {}",
            crate::gate::now_unix().saturating_sub(ok.at),
            non_empty(&ok.route)
        );
    }
    if let Some(e) = &r.last_400 {
        let _ = writeln!(
            out,
            "Последняя ошибка 400, которую видела служба: {} с назад, строк: {}, маршрут: {}{}",
            crate::gate::now_unix().saturating_sub(e.at),
            e.count,
            non_empty(&e.route),
            if e.bypassed {
                " (мимо обхода)"
            } else {
                ""
            }
        );
        if !e.acted.is_empty() {
            let _ = writeln!(out, "Что сделано: {}", e.acted);
        }
    }
    let _ = writeln!(out, "Маршруты (в порядке выбора):");
    for row in &r.routes {
        let mut parts: Vec<String> = Vec::new();
        parts.push(if row.usable {
            "доступен".to_string()
        } else {
            "недоступен".to_string()
        });
        if let Some(ms) = row.latency_ms {
            parts.push(format!("{ms} мс"));
        }
        if row.proven {
            parts.push("проверен ответом модели".to_string());
        }
        if let Some(a) = row.ok_ago {
            parts.push(format!("ответ {a} с назад"));
        }
        if let Some(a) = row.refused_ago {
            parts.push(format!("ошибка 400 {a} с назад"));
        }
        if let Some(b) = row.bench_left {
            parts.push(format!("отложен ещё на {} мин", b / 60 + 1));
        }
        if row.open > 0 {
            parts.push(format!("открытых соединений: {}", row.open));
        }
        let _ = writeln!(out, "  {} — {}", row.label, parts.join(", "));
    }
}

fn non_empty(s: &str) -> &str {
    if s.is_empty() {
        "—"
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    /// What «Скопировать отчёт» puts on the clipboard on this machine, minus the
    /// system scan the window adds. Reads, never asserts content.
    ///
    ///     cargo test prints_the_report -- --ignored --nocapture
    #[test]
    #[ignore = "reads the live relay record and log; run with --ignored"]
    fn prints_the_report() {
        let text = super::build(None, &crate::gate::View::default());
        println!("{text}");
        assert!(text.contains("отчёт"));
    }
}
