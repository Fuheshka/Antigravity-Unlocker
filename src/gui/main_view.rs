//! The one screen the tool has once the key is in.
//!
//! Top to bottom, in the order a person who is not a programmer reads it:
//!
//! 1. **One status card** — is Antigravity going to answer, and if not, the one
//!    thing to press (`status`). Everything else on the screen is mechanism.
//! 2. **Antigravity** — the three switches anyone needs (unlock sign-in, get
//!    past the 400, keep the patch after updates) and the installs found.
//! 3. **Для опытных**, folded — the parts of the bypass one by one, the DNS
//!    pool, the user's own proxy, what the network looks like right now.
//! 4. **Журнал**, folded.
//!
//! Everything is still a switch that undoes itself - the rule the window was
//! built on - but the switches are no longer the first thing a user has to
//! understand. The bypass picks its own path on every network (D25), so there
//! is nothing about VPNs, DNS or proxies anyone *has* to decide.

use eframe::egui;

use std::time::Duration;

use super::status::{self, Action, Tone};
use super::{theme, widgets, App, DONATE_URL, TELEGRAM_GROUP_URL};
use crate::ops::{Cap, Cmd, Level, State};
use crate::utils::mask_path;

pub fn view(app: &mut App, ui: &mut egui::Ui) {
    header(app, ui);

    // A bottom panel, reserved *before* the scrolling body. Laid out the other
    // way round — scroll area first, footer after — the scroll area claims every
    // remaining pixel and the footer is positioned past the bottom of the
    // window: measured, painted, and never on screen. A panel is the one
    // construct that takes its strip out of the parent first.
    egui::Panel::bottom("footer")
        .frame(egui::Frame::new().inner_margin(egui::Margin {
            top: 2,
            bottom: 2,
            ..Default::default()
        }))
        .show_separator_line(true)
        .show(ui, footer);

    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Put back what the outer frame gave up to the scroll bar,
                    // on the inside of it, so the cards keep their margin and the
                    // bar still lands against the window edge.
                    ui.set_max_width(ui.available_width() - 10.0);
                    status_card(app, ui);
                    ui.add_space(12.0);
                    antigravity_card(app, ui);
                    ui.add_space(12.0);
                    advanced_card(app, ui);
                    ui.add_space(6.0);
                    log_card(app, ui);
                    ui.add_space(12.0);
                });
        });
}

// ---------------------------------------------------------------------------

fn header(app: &mut App, ui: &mut egui::Ui) {
    app.update_banner(ui);

    // No product name and no version here: both are in the title bar already,
    // and repeating them costs a line of a window this narrow.
    if let Some(what) = &app.busy {
        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().size(14.0));
            ui.label(
                egui::RichText::new(what.clone())
                    .size(12.0)
                    .color(theme::MUTED),
            );
        });
        ui.add_space(8.0);
    }
}

// ---------------------------------------------------------------------------
// The status card
// ---------------------------------------------------------------------------

/// How far *before* a refusal the relay's note may be stamped and still be an
/// answer to it: the whole-second quantisation both stamps carry, and nothing
/// more. A note stamped earlier is an answer to something else (I58).
const EPISODE_SLACK: u64 = 3;

/// Whether the relay's note answers a refusal stamped at `refusal_at`.
fn answers(episode_at: u64, refusal_at: u64) -> bool {
    episode_at.saturating_add(EPISODE_SLACK) >= refusal_at
}

/// Everything the verdict is made of, read off the window's own state.
fn facts(app: &App) -> Option<status::Facts> {
    let s = app.status.as_ref()?;
    let aged = |ago: Duration| ago + app.gate_at.elapsed();
    // Newest over twelve hours, so an old refusal still outranks an older
    // answer; counted over the last ten minutes, which is what "it keeps
    // happening" means.
    let refusal = app
        .gate
        .refused_long
        .map(|x| (aged(x.ago), app.gate.seen.map_or(0, |s| s.count)));
    let answer = app.gate.answered.map(|x| aged(x.ago));
    // The relay's record is only worth anything while the relay runs: a record
    // outlives its writer by up to `STALE_AFTER`, and "обход перехватил" about a
    // dead service is the worst sentence this card could say (I58).
    let relay = app.gate.relay.as_ref().filter(|_| s.relay_running);
    let answered = refusal.and_then(|(ago, _)| {
        if ago > crate::gate::RECENT {
            return None;
        }
        let refusal_at = crate::gate::now_unix().saturating_sub(ago.as_secs());
        relay
            .and_then(|r| r.last_400.as_ref())
            .filter(|e| answers(e.at, refusal_at))
            .map(|e| status::Answered {
                acted: e.acted.clone(),
                bypassed: e.bypassed,
            })
    });
    // The path of the last answer when the relay saw one recently - and then
    // exactly that, empty included: an answer no tunnel of ours carried went
    // around us, and naming the route we would have used instead would be a
    // claim about traffic that never touched it. Otherwise the route in force.
    let route = relay.and_then(|r| {
        let recent_ok = r.last_ok.as_ref().filter(|ok| {
            crate::gate::now_unix().saturating_sub(ok.at) <= crate::gate::ANSWER_RECENT.as_secs()
        });
        match recent_ok {
            Some(ok) => (!ok.route.is_empty()).then(|| ok.route.clone()),
            None => (!r.route.is_empty()).then(|| r.route.clone()),
        }
    });
    Some(status::Facts {
        admin: s.admin || !cfg!(target_os = "windows"),
        installs_found: s.installs.iter().any(|r| r.path.is_some()),
        patch_on: s.client_patch.is_on(),
        bypass_on: s.dns.is_on(),
        relay_running: s.relay_running,
        relay_outdated: s.relay_outdated,
        rules: s.rules || !cfg!(target_os = "windows"),
        relay_reporting: relay.is_some(),
        refusal,
        answer,
        answered,
        route,
    })
}

fn status_card(app: &mut App, ui: &mut egui::Ui) {
    let Some(f) = facts(app) else {
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(16.0));
                ui.label(egui::RichText::new("Проверяю систему…").size(15.0));
            });
        });
        return;
    };
    let h = status::headline(&f);
    let accent = match h.tone {
        Tone::Ok => theme::OK,
        Tone::Wait => theme::ACCENT,
        Tone::Fixing => theme::WARN,
        Tone::Action => theme::BAD,
        Tone::Off => theme::MUTED,
    };
    // Ages on the card count up on their own; without a repaint «минуту назад»
    // would stay «минуту назад» until the mouse moved.
    if f.refusal.is_some() || f.answer.is_some() {
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }

    let busy = app.is_busy();
    let mut pressed: Option<Action> = None;
    let mut copy = false;
    egui::Frame::new()
        .fill(theme::CARD)
        .corner_radius(egui::CornerRadius::same(theme::RADIUS))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.5, accent))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 6.0, accent);
                ui.label(
                    egui::RichText::new(&h.title)
                        .size(19.0)
                        .strong()
                        .color(theme::TEXT),
                );
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&h.detail).size(13.5).color(theme::TEXT));
            if let Some(action) = h.action {
                ui.add_space(10.0);
                if widgets::primary(ui, action.label(), !busy).clicked() {
                    pressed = Some(action);
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!busy, egui::Button::new(egui::RichText::new("Скопировать отчёт").size(12.5)))
                    .on_hover_text("Всё, что нужно, чтобы понять, почему Antigravity отвечает или нет — одним текстом для группы")
                    .clicked()
                {
                    copy = true;
                }
                if app
                    .report_copied_at
                    .is_some_and(|at| at.elapsed() < Duration::from_secs(4))
                {
                    ui.label(
                        egui::RichText::new("Отчёт скопирован — вставьте его в сообщение.")
                            .size(12.5)
                            .color(theme::OK),
                    );
                    ui.ctx().request_repaint_after(Duration::from_millis(500));
                }
            });
        });

    match pressed {
        Some(Action::EnableAll) => app.worker.send(Cmd::EnableAll),
        Some(Action::Repair) => app.worker.send(Cmd::Repair),
        Some(Action::Elevate) => app.request_elevation(),
        None => {}
    }
    if copy {
        let text = super::report::build(app.status.as_ref(), &app.gate);
        ui.ctx().copy_text(text);
        app.report_copied_at = Some(std::time::Instant::now());
    }
}

// ---------------------------------------------------------------------------
// Antigravity: the three switches anyone needs, and the installs
// ---------------------------------------------------------------------------

fn antigravity_card(app: &mut App, ui: &mut egui::Ui) {
    widgets::card(ui, |ui| {
        cap_row(
            app,
            ui,
            Cap::ClientPatch,
            "Вход в аккаунт из-под санкций",
            "Снимает блокировку входа в Google-аккаунт из санкционного региона.",
        );

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        bypass_master(app, ui);

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        cap_row(
            app,
            ui,
            Cap::Watchdog,
            "Автопатч",
            "Сам накладывает патч на найденный Antigravity — сразу после установки и после каждого обновления, которое его стирает.",
        );

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Найденные установки Antigravity")
                .size(12.5)
                .color(theme::MUTED),
        );
        ui.add_space(6.0);
        install_rows(app, ui);
    });
}

/// The master switch of the bypass. Derived, never stored: it is on when any
/// part of the bypass is, which keeps one truth instead of two.
fn bypass_master(app: &mut App, ui: &mut egui::Ui) {
    let any_on = app
        .status
        .as_ref()
        .map(|s| s.dns.is_on() || s.local_proxy.is_on() || s.builtin_exits.is_on())
        .unwrap_or(false);
    let mut master = any_on;
    let busy = app.is_busy();
    let flipped = widgets::switch_row(ui, &mut master, !busy, |ui| {
        ui.label(egui::RichText::new("Снять ошибку 400 в чате с ИИ").size(14.0));
        widgets::hint(
            ui,
            "«User location is not supported». Сам находит рабочий путь до серверов Google \
             и переключается, если путь перестал работать — с VPN и без.",
        );
    });
    if flipped {
        // Order matters and it is not the same in both directions.
        // ON: the relay has to be answering before the proxy variable may
        // name it (I53) — the worker runs these in order, so DNS finishes
        // first. OFF: the variable comes off *before* the listener it names
        // goes away, or a sign-in that lands in between dials a dead port
        // (G31).
        let order = if master {
            [Cap::Dns, Cap::LocalProxy, Cap::BuiltinExits]
        } else {
            [Cap::LocalProxy, Cap::BuiltinExits, Cap::Dns]
        };
        for cap in order {
            app.worker.send(Cmd::Set(cap, master));
        }
    }
}

fn install_rows(app: &mut App, ui: &mut egui::Ui) {
    let Some(rows) = app.status.as_ref().map(|s| s.installs.clone()) else {
        widgets::hint(ui, "Идёт поиск…");
        return;
    };

    let mut forget: Option<std::path::PathBuf> = None;
    let mut edit: Option<String> = None;

    for row in &rows {
        ui.horizontal(|ui| {
            let color = match (&row.path, row.patched) {
                (None, _) => theme::LINE,
                (Some(_), Some(true)) => theme::OK,
                (Some(_), Some(false)) => theme::MUTED,
                (Some(_), None) => theme::LINE,
            };
            widgets::dot(ui, color);
            ui.label(egui::RichText::new(row.label).size(13.0));

            match &row.path {
                Some(path) => {
                    let shown = mask_path(&path.display().to_string());
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(shown)
                                .size(12.0)
                                .color(theme::MUTED)
                                .monospace(),
                        )
                        // One line, cut with an ellipsis rather than wrapped: a
                        // long install path would push the pencil off the row.
                        .truncate(),
                    )
                    .on_hover_text(path.display().to_string());
                }
                None => {
                    ui.label(
                        egui::RichText::new("не найдено — укажите путь")
                            .size(12.0)
                            .color(theme::MUTED),
                    );
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if row.manual {
                    if let Some(p) = &row.path {
                        if ui
                            .small_button("✖")
                            .on_hover_text("Убрать указанный путь")
                            .clicked()
                        {
                            forget = Some(p.clone());
                        }
                    }
                }
                if ui
                    .small_button("✏")
                    .on_hover_text("Указать путь вручную")
                    .clicked()
                {
                    edit = Some(
                        row.path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                    );
                }
            });
        });
    }

    if let Some(p) = forget {
        app.worker.send(Cmd::ForgetPath(p));
    }
    if let Some(text) = edit {
        app.path_dialog = Some(text);
    }
}

// ---------------------------------------------------------------------------
// Для опытных
// ---------------------------------------------------------------------------

/// Folded sections start open in a *debug* build run with `AG_UNLOCKER_DEV_OPEN`
/// set, so the whole screen can be looked at without clicking. Always folded in
/// a release build.
fn dev_open() -> bool {
    cfg!(debug_assertions) && std::env::var_os("AG_UNLOCKER_DEV_OPEN").is_some()
}

fn advanced_card(app: &mut App, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(
        egui::RichText::new("Настройки для опытных")
            .size(13.0)
            .color(theme::MUTED),
    )
    .id_salt("advanced")
    .default_open(dev_open())
    .show(ui, |ui| {
        widgets::card(ui, |ui| {
            network_facts(app, ui);

            cap_row(
                app,
                ui,
                Cap::Dns,
                "Обход через DNS",
                "Служба на этом компьютере отвечает на имена двух серверов Google и держит их \
                 разрешающимися через сервисы разблокировки. Без неё обход не работает.",
            );
            providers_list(app, ui);

            ui.add_space(10.0);
            cap_row(
                app,
                ui,
                Cap::LocalProxy,
                "Локальный прокси",
                "Соединения Antigravity с этими двумя серверами идут через посредника внутри \
                 вашего компьютера: он выбирает путь, который реально работает, и переключается \
                 сам. Содержимое не расшифровывается — посредник только передаёт байты.",
            );

            ui.add_space(10.0);
            cap_row(
                app,
                ui,
                Cap::BuiltinExits,
                "Встроенные выходы",
                // Deliberately says what they are and never which they are: a
                // free service that gets named publicly stops being free (I46).
                "Запасной путь до серверов Google — через страну без ограничений.",
            );

            ui.add_space(10.0);
            cap_row(
                app,
                ui,
                Cap::VerifyTls,
                "Сверять TLS",
                "Адрес от сервиса разблокировки принимается, только если предъявил настоящий \
                 сертификат Google. Выключать без причины не стоит.",
            );

            ui.add_space(10.0);
            cap_row(
                app,
                ui,
                Cap::OwnProxy,
                "Свой HTTP-прокси",
                "Ваш собственный прокси в разрешённой стране — он всегда пробуется первым. \
                 Google может не принять прокси из дата-центра даже там.",
            );
            own_proxy_field(app, ui);
        });
    });
}

/// What the relay says about the network right now: the VPN, where it comes
/// out, and the path in use. Facts, not advice - there is nothing here the
/// user has to act on, which is why it sits in the folded section.
fn network_facts(app: &App, ui: &mut egui::Ui) {
    let Some(r) = app.gate.relay.as_ref() else {
        return;
    };
    let vpn = if !r.tunnel {
        "VPN не обнаружен.".to_string()
    } else if r.vpn_exit.is_empty() {
        "VPN включён — соединения службы с сервисами разблокировки идут мимо него.".to_string()
    } else if crate::upstream::region_is_blocked(&r.vpn_exit) {
        format!(
            "VPN включён, выход: {} — там ошибка 400, поэтому соединения идут мимо него.",
            r.vpn_exit
        )
    } else {
        format!(
            "VPN включён, выход: {} — используется как один из путей.",
            r.vpn_exit
        )
    };
    widgets::hint(ui, &vpn);
    if !r.route.is_empty() {
        widgets::hint(ui, &format!("Первый путь сейчас: {}.", r.route));
    }
    if !r.loopback {
        widgets::hint(
            ui,
            "Служба ещё не перехватывает имена серверов Google — Antigravity идёт через них, \
             только если перезапущен после включения обхода.",
        );
    }
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}

/// How a provider's name is written in the list.
///
/// The pool stores them the way they are typed as hostnames — all lower case —
/// which reads as sloppy in a list of proper names. An acronym stays an acronym
/// (`dns-ai.ru` → `DNS-AI.RU`); everything else just gets its first letter.
/// Presentation only: the stored name is what every switch, the deny-list and
/// the saved order are keyed by, and it never changes.
fn display_name(name: &str) -> String {
    let lead: String = name.chars().take_while(|c| c.is_alphabetic()).collect();
    if lead.eq_ignore_ascii_case("dns") {
        return name.to_uppercase();
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn providers_list(app: &mut App, ui: &mut egui::Ui) {
    // Copied out before anything is drawn: the rows below need `&mut app` for
    // the rotation switch, and holding a borrow of `app.status` across that is
    // what the borrow checker (rightly) refuses.
    let Some((dns_on, rotating)) = app
        .status
        .as_ref()
        .map(|s| (s.dns.is_on(), s.dns_rotation.is_on()))
    else {
        return;
    };
    // Drawn from the window's own copy, which a drag rearranges immediately; the
    // worker's snapshot is adopted back into it whenever no drag is in flight.
    let providers = app.providers_local.clone();
    if providers.is_empty() {
        return;
    }
    let busy = app.is_busy();

    let mut flip: Option<(String, bool)> = None;
    // (from, to) — set the moment the pointer passes over another row, not on
    // release: the row has to follow the cursor while the button is still down.
    let mut moved: Option<(usize, usize)> = None;
    let mut dropped = false;

    egui::CollapsingHeader::new(
        egui::RichText::new("Какие DNS использовать")
            .size(12.5)
            .color(theme::MUTED),
    )
    .id_salt("providers")
    .default_open(false)
    .show(ui, |ui| {
        let first_on = providers.iter().position(|p| p.enabled);
        widgets::hint(
            ui,
            "Порядок можно менять: зажмите полоски слева и перетащите. \
             Первый в списке спрашивается первым.",
        );
        ui.add_space(4.0);

        for (i, p) in providers.iter().enumerate() {
            let row = ui
                .horizontal(|ui| {
                    ui.add_space(6.0);
                    // Only the grip drags. Making the whole row a drag source
                    // put a drag sense over the switch too, so aiming at the
                    // switch and moving a pixel dragged the row instead of
                    // toggling it.
                    //
                    // The id is keyed by name, not by index: an id tied to the
                    // position would follow the slot rather than the row.
                    let id = egui::Id::new(("dns-provider", p.name.as_str()));
                    ui.dnd_drag_source(id, i, |ui| {
                        ui.label(egui::RichText::new("≡").size(15.0).color(theme::MUTED));
                    })
                    .response
                    .on_hover_cursor(egui::CursorIcon::Grab);

                    let mut on = p.enabled;
                    if widgets::switch(ui, &mut on, dns_on && !busy).changed() {
                        flip = Some((p.name.clone(), on));
                    }
                    // Without rotation only the first enabled one is ever asked,
                    // so the rest are drawn as what they are: on, but not in use.
                    let idle = !rotating && p.enabled && first_on != Some(i);
                    let text = egui::RichText::new(display_name(&p.name)).size(13.0);
                    ui.label(if idle { text.color(theme::MUTED) } else { text });
                    if idle {
                        widgets::hint(ui, "— не используется");
                    }
                })
                .response;

            // The whole row is the drop target, not just the grip: aiming at a
            // 15 px glyph to finish a drag is not something anyone should have
            // to do.
            // Hover, not release: the list rearranges under the pointer while
            // the button is still down, which is what makes a drag feel like
            // moving a thing rather than aiming at a slot.
            if let Some(from) = row.dnd_hover_payload::<usize>() {
                if *from != i {
                    moved = Some((*from, i));
                }
            }
            if row.dnd_release_payload::<usize>().is_some() {
                dropped = true;
            }
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        cap_row(
            app,
            ui,
            Cap::DnsRotation,
            "Ротация между серверами",
            "Включено: запрос идёт ко всем включённым серверам, ответ сверяется \
             с эталонным резолвером. Выключено: используется только первый \
             включённый в списке, запасных не будет.",
        );
    });

    if let Some((name, on)) = flip {
        app.worker.send(Cmd::SetProvider(name, on));
    }
    if let Some((from, to)) = moved {
        if from < app.providers_local.len() && to < app.providers_local.len() {
            let row = app.providers_local.remove(from);
            app.providers_local.insert(to, row);
            // The payload has to follow the row to its new index, or the next
            // frame would think it is still being dragged from the old slot and
            // move it straight back.
            egui::DragAndDrop::set_payload(ui.ctx(), to);
            app.providers_reordering = true;
        }
    }

    // Saved once, when the button comes up. Writing on every hover would be a
    // file write per frame of a drag.
    if dropped && app.providers_reordering {
        app.providers_reordering = false;
        let order: Vec<String> = app.providers_local.iter().map(|p| p.name.clone()).collect();
        app.worker.send(Cmd::ReorderProviders(order));
    }
    // A drag abandoned outside the list (or one that changed nothing) must not
    // leave the window refusing the worker's snapshots for ever.
    if app.providers_reordering && ui.input(|i| i.pointer.any_released()) {
        app.providers_reordering = false;
        let order: Vec<String> = app.providers_local.iter().map(|p| p.name.clone()).collect();
        app.worker.send(Cmd::ReorderProviders(order));
    }
}

fn own_proxy_field(app: &mut App, ui: &mut egui::Ui) {
    let busy = app.is_busy();
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let field = egui::TextEdit::singleline(&mut app.own_proxy_input)
            .hint_text("host:port или user:pass@host:port")
            .desired_width(ui.available_width() - 110.0);
        let resp = ui.add_enabled(!busy, field);
        let entered = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui
            .add_enabled(!busy, egui::Button::new("Применить"))
            .clicked()
            || entered
        {
            let text = app.own_proxy_input.clone();
            app.worker.send(Cmd::SetOwnProxy(text));
        }
    });
}

// ---------------------------------------------------------------------------

/// One switch with its title, description and — when the system disagrees with
/// the switch — the reason.
fn cap_row(app: &mut App, ui: &mut egui::Ui, cap: Cap, title: &str, hint: &str) {
    let state = app
        .status
        .as_ref()
        .map(|s| s.get(cap).clone())
        .unwrap_or(State::Off);
    let mut on = state.is_on();
    let blocked = matches!(state, State::Blocked(_));
    let enabled = !app.is_busy() && !blocked;

    let note = state.note().map(|s| s.to_string());
    let flipped = widgets::switch_row(ui, &mut on, enabled, |ui| {
        ui.label(egui::RichText::new(title).size(14.0));
        widgets::hint(ui, hint);
        if let Some(note) = note {
            ui.label(egui::RichText::new(note).size(12.0).color(if blocked {
                theme::WARN
            } else {
                theme::MUTED
            }));
        }
    });
    if flipped {
        app.worker.send(Cmd::Set(cap, on));
    }
}

// ---------------------------------------------------------------------------

/// One log line as it is drawn — and as it is copied, so what lands on the
/// clipboard is what was on screen.
fn log_line(level: Level, line: &str) -> String {
    if level == Level::Step {
        format!("— {line}")
    } else {
        line.to_string()
    }
}

fn log_card(app: &mut App, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(egui::RichText::new("Журнал").size(13.0).color(theme::MUTED))
        .id_salt("log")
        .default_open(dev_open())
        .show(ui, |ui| {
            // **Before** the lines are drawn, and that ordering is the whole
            // trick. egui's `LabelSelectionState` accumulates its copy per label,
            // as each one is drawn, and flushes it to the clipboard in
            // `end_pass` — i.e. after everything here. Consuming the Copy event
            // afterwards was too late: the labels had already accumulated (just
            // the one holding the cursor, hence "copies a single line"), and
            // their flush overwrote ours. Taking the event first means no label
            // ever sees it and our copy is the only one.
            log_keys(app, ui);

            // Selecting with the mouse and copying it is egui's own label
            // selection; the only thing missing was a way to take the lot, which
            // is what Ctrl+A does.
            let all_selected = app.log_all_selected;
            let fill = ui.visuals().selection.bg_fill;

            egui::ScrollArea::vertical()
                .max_height(160.0)
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if app.log.is_empty() {
                        widgets::hint(ui, "Пока ничего не делалось.");
                    }
                    for (level, line) in &app.log {
                        let color = match level {
                            Level::Ok => theme::OK,
                            Level::Warn => theme::WARN,
                            Level::Err => theme::BAD,
                            Level::Step => theme::TEXT,
                            Level::Info => theme::MUTED,
                        };
                        let mut text = egui::RichText::new(log_line(*level, line))
                            .color(color)
                            .size(12.5);
                        if *level == Level::Step {
                            text = text.strong();
                        }
                        if all_selected {
                            text = text.background_color(fill);
                        }
                        ui.label(text);
                    }
                });
        });
}

/// Ctrl+A over the journal, then Ctrl+C.
///
/// Both work on a Russian layout, and that is not an accident of this code:
/// egui-winit resolves a key as `logical.or(physical)`, so with a Cyrillic
/// layout the logical key («ф», «с») maps to nothing and the *physical* A and C
/// are used instead. What this adds is the select-all, which egui has no notion
/// of across a pile of separate labels — so it is our own flag, drawn as a
/// selection behind every line and copied as one block.
fn log_keys(app: &mut App, ui: &mut egui::Ui) {
    // While a text field has the keyboard, Ctrl+A belongs to that field.
    let typing = ui.memory(|m| m.focused()).is_some();

    if !typing && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A)) {
        app.log_all_selected = !app.log.is_empty();
    }

    if app.log_all_selected {
        // Only while our own selection is up, so a selection made with the mouse
        // is still copied by egui's label machinery rather than overwritten with
        // the whole journal.
        let copy = ui.input_mut(|i| {
            let asked = i
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::Copy | egui::Event::Cut));
            if asked {
                i.events
                    .retain(|e| !matches!(e, egui::Event::Copy | egui::Event::Cut));
            }
            asked
        });
        if copy {
            let text: String = app
                .log
                .iter()
                .map(|(level, line)| log_line(*level, line))
                .collect::<Vec<_>>()
                .join("\n");
            ui.ctx().copy_text(text);
        }
        // Any click, or Escape, gives the selection up — otherwise the next
        // Ctrl+C anywhere in the window would still copy the journal.
        let dismissed = ui.input(|i| i.pointer.any_pressed() || i.key_pressed(egui::Key::Escape));
        if dismissed {
            app.log_all_selected = false;
        }
    }
}

/// Text size in the footer. One point up from the rest of the small print — it
/// is the line people are meant to read, not a caption under something else.
const FOOTER_TEXT: f32 = 13.0;

fn footer(ui: &mut egui::Ui) {
    // The strip was about twice as tall as its text. Two things made it so, and
    // the margin was the smaller one: a link is an *interactive* widget, so it
    // claims `interact_size.y` (26 px, sized for buttons) however short its text
    // is. Shrinking that here — and only here — is what actually halves the bar.
    ui.spacing_mut().interact_size.y = 18.0;
    ui.spacing_mut().item_spacing.y = 0.0;

    // Built right-to-left, so it reads left-to-right on screen while staying
    // pinned to the right edge.
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // The outer frame keeps only 4 px on the right, for the scroll bar. The
        // footer is not inside the scrolling area, so it pays that back itself
        // or its last link is cut off by the window edge.
        ui.add_space(12.0);
        if ui
            .link(egui::RichText::new("t.me/nova_txt").size(FOOTER_TEXT))
            .clicked()
        {
            crate::utils::open_url(TELEGRAM_GROUP_URL);
        }
        ui.label(
            egui::RichText::new("Группа в Telegram:")
                .size(FOOTER_TEXT)
                .color(theme::MUTED),
        );
        ui.label(
            egui::RichText::new("|")
                .size(FOOTER_TEXT)
                .color(theme::LINE),
        );
        if ui
            .link(egui::RichText::new("nova-app.eu/donate").size(FOOTER_TEXT))
            .clicked()
        {
            crate::utils::open_url(DONATE_URL);
        }
        ui.label(
            egui::RichText::new("Отблагодарить копеечкой:")
                .size(FOOTER_TEXT)
                .color(theme::MUTED),
        );
    });
}

/// The pencil dialog: type or paste a folder, we resolve it to an install root.
pub fn path_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(mut text) = app.path_dialog.take() else {
        return;
    };
    let mut keep_open = true;
    let mut submit = false;

    egui::Window::new("Путь к Antigravity")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            widgets::hint(
                ui,
                "Папка установки Antigravity, IDE или CLI. Можно указать вложенную — \
                 корень будет найден сам.",
            );
            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut text)
                    .desired_width(f32::INFINITY)
                    .hint_text("C:\\Users\\...\\Programs\\Antigravity"),
            );
            if let Some(err) = &app.path_dialog_error {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(err).color(theme::BAD).size(12.5));
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if widgets::primary(ui, "Добавить", !text.trim().is_empty()).clicked() {
                    submit = true;
                }
                if widgets::ghost(ui, "Отмена").clicked() {
                    keep_open = false;
                }
            });
        });

    if submit {
        let cleaned = crate::clean_input_path(&text);
        match crate::ops::resolve_manual_path(std::path::Path::new(&cleaned)) {
            Some(root) => {
                app.worker.send(Cmd::AddPath(root));
                app.path_dialog_error = None;
                keep_open = false;
            }
            None => {
                app.path_dialog_error =
                    Some("По этому пути установка Antigravity не найдена.".into());
            }
        }
    }

    if keep_open {
        app.path_dialog = Some(text);
    } else {
        app.path_dialog_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::display_name;

    #[test]
    fn an_acronym_stays_an_acronym_and_everything_else_gets_one_capital() {
        assert_eq!(display_name("dns-ai.ru"), "DNS-AI.RU");
        assert_eq!(display_name("comss.one"), "Comss.one");
        assert_eq!(display_name("geohide.ru"), "Geohide.ru");
        // Must not panic on a name the pool could grow later.
        assert_eq!(display_name(""), "");
        assert_eq!(display_name("1.1.1.1"), "1.1.1.1");
    }

    #[test]
    fn only_a_note_written_after_the_refusal_answers_it() {
        use super::answers;
        assert!(answers(1_000, 1_000));
        assert!(answers(1_015, 1_000));
        assert!(answers(998, 1_000));
        assert!(!answers(940, 1_000));
        assert!(!answers(0, 1_000));
        assert!(answers(u64::MAX, 1_000));
        assert!(!answers(0, u64::MAX));
    }
}
