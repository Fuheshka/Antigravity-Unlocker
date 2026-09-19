//! The one line at the top of the window: is it working, and if not, what now.
//!
//! Everything below it is mechanism - which DNS answers, which route, which
//! switch - and a user who is not a programmer should not have to read any of
//! it to know whether Antigravity will answer. So the window's whole verdict is
//! derived here, as a pure function of facts the window already has, and it is
//! tested cell by cell: this is the sentence a user acts on.
//!
//! The rules that shaped it, each one a bug that shipped before:
//!
//! * **"Работает" needs proof.** Only a model answer in the client's own log
//!   turns the card green. Silence after a refusal used to read as "fixed" -
//!   and silence is also exactly what a user who gave up produces (G50).
//! * **One action at most, and a real one.** A card that says what is wrong
//!   carries the button that fixes it, or says plainly that there is nothing
//!   to press.
//! * **What was done is said by the side that did it** (the relay's record),
//!   never inferred from a switch being on (I58).

use std::time::Duration;

/// How the card is coloured, and how loud it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Proven working.
    Ok,
    /// Set up, nothing proven yet: waiting for the user's next message.
    Wait,
    /// A refusal was seen and is being dealt with.
    Fixing,
    /// Something only the user can do.
    Action,
    /// Switched off, or nothing to work on.
    Off,
}

/// The one button a card may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Turn everything on.
    EnableAll,
    /// Restart this program elevated.
    Elevate,
    /// Reinstall and restart the service.
    Repair,
}

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Action::EnableAll => "Включить всё",
            Action::Elevate => "Перезапустить от имени администратора",
            Action::Repair => "Починить",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Headline {
    pub tone: Tone,
    pub title: String,
    pub detail: String,
    pub action: Option<Action>,
}

/// What the relay said it did about the last refusal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answered {
    pub acted: String,
    /// No tunnel of ours carried it: the client went to Google by itself.
    pub bypassed: bool,
}

/// Everything the verdict is made of. Plain values, so a test can build any
/// state the window can be in.
#[derive(Debug, Clone, Default)]
pub struct Facts {
    /// Windows and elevated (or not Windows, where nothing needs it).
    pub admin: bool,
    /// At least one Antigravity install was found.
    pub installs_found: bool,
    /// The account patch is on everywhere it can be.
    pub patch_on: bool,
    /// The 400 bypass is on: its DNS half, which everything else rides on - the
    /// local proxy lives in the same service, and the loopback door only opens
    /// once the relay answers the gate hosts.
    pub bypass_on: bool,
    pub relay_running: bool,
    pub relay_outdated: bool,
    /// Our DNS rules are in. Without them nothing on the machine asks the
    /// service anything, however healthy it is.
    pub rules: bool,
    /// The relay's record is fresh, i.e. it is alive and saying what it does.
    pub relay_reporting: bool,
    /// The newest refusal in a client log (over the same twelve hours as
    /// `answer`): how long ago, and how many lines in the last ten minutes.
    pub refusal: Option<(Duration, usize)>,
    /// The newest model answer in a client log, how long ago.
    pub answer: Option<Duration>,
    /// The relay's answer to that refusal, when it wrote one after it.
    pub answered: Option<Answered>,
    /// The route the relay credited with the last answer, or the one it would
    /// use now.
    pub route: Option<String>,
}

/// About three refused turns (each writes four lines) with no answer in
/// between: past that, "send it again" has been tried and did not help.
const STUCK_LINES: usize = 12;

/// How long a refusal is "happening now" and the card says «Чиним». After it,
/// with no answer since, the card asks for a check instead: nothing is being
/// fixed any more, and nothing has been shown to work either.
const FIXING_FOR: Duration = Duration::from_secs(10 * 60);

pub fn headline(f: &Facts) -> Headline {
    // Nothing to work on. Said first: every other card assumes Antigravity is
    // installed, and a button that patches nothing is not an action.
    if !f.installs_found && !f.patch_on && !f.bypass_on {
        return Headline {
            tone: Tone::Off,
            title: "Antigravity не найден".into(),
            detail: "Установите Antigravity или укажите папку с ним ниже — карандаш рядом с нужной строкой."
                .into(),
            action: None,
        };
    }

    // The bypass needs an administrator to install and to repair. Asked for
    // only when there is something it would install or repair.
    let service_broken = f.bypass_on && (!f.relay_running || f.relay_outdated || !f.rules);
    if !f.admin && (!f.bypass_on || service_broken) {
        return Headline {
            tone: Tone::Action,
            title: "Нужны права администратора".into(),
            detail: "Без них обход ошибки 400 не установить и не починить. Разблокировка входа \
                     работает и так."
                .into(),
            action: Some(Action::Elevate),
        };
    }

    if !f.patch_on || !f.bypass_on {
        let what = match (f.patch_on, f.bypass_on) {
            (false, false) => "Разблокировка входа и обход ошибки 400 выключены.",
            (false, true) => "Разблокировка входа выключена.",
            _ => "Обход ошибки 400 выключен.",
        };
        let closes = if f.patch_on {
            ""
        } else {
            " Antigravity закроется на пару секунд."
        };
        return Headline {
            tone: Tone::Off,
            title: "Включено не всё".into(),
            detail: format!("{what}{closes}"),
            action: Some(Action::EnableAll),
        };
    }

    if service_broken {
        return Headline {
            tone: Tone::Action,
            title: if !f.relay_running {
                "Служба обхода не работает".into()
            } else if f.relay_outdated {
                "Служба обхода устарела".into()
            } else {
                "Служба обхода работает не полностью".into()
            },
            detail: "Без неё ошибка 400 не обходится. Кнопка переустановит и запустит её.".into(),
            action: Some(Action::Repair),
        };
    }

    // A refusal newer than the newest answer is the live problem.
    let refused_last = match (f.refusal, f.answer) {
        (Some((r, _)), Some(a)) => r < a,
        (Some(_), None) => true,
        _ => false,
    };
    if refused_last {
        let (ago, lines) = f.refusal.unwrap_or_default();
        if ago > FIXING_FOR {
            return Headline {
                tone: Tone::Wait,
                title: "Нужна проверка".into(),
                detail: format!(
                    "Последний раз была ошибка 400 — {}, и после неё модель не отвечала. \
                     Напишите что-нибудь в Antigravity: если ошибка повторится, обход её поймает \
                     и сам сменит путь.",
                    ago_text(ago)
                ),
                action: None,
            };
        }
        if lines >= STUCK_LINES {
            return Headline {
                tone: Tone::Action,
                title: "Ошибка 400 повторяется".into(),
                detail: format!(
                    "Последняя — {}. Обход перебирает пути сам, но ни один пока не сработал. \
                     Если у вас включён VPN или прокси, попробуйте выключить его или сменить сервер, \
                     и отправьте сообщение ещё раз. Не помогло — нажмите «Скопировать отчёт» и \
                     пришлите его в группу.",
                    ago_text(ago)
                ),
                action: None,
            };
        }
        let detail = match &f.answered {
            Some(a) if a.bypassed => format!(
                "Ошибка 400 — {}. Antigravity обратился к Google мимо обхода; через минуту его \
                 запросы пойдут через обход. Отправьте сообщение ещё раз — не помогло, \
                 перезапустите Antigravity.",
                ago_text(ago)
            ),
            Some(a) if !a.acted.is_empty() => format!(
                "Ошибка 400 — {}. {}. Отправьте сообщение ещё раз.",
                ago_text(ago),
                capitalise(&a.acted)
            ),
            _ => format!(
                "Ошибка 400 — {}. Обход её видит и перестраивается. Отправьте сообщение ещё раз.",
                ago_text(ago)
            ),
        };
        return Headline {
            tone: Tone::Fixing,
            title: "Чиним".into(),
            detail,
            action: None,
        };
    }

    if let Some(ago) = f.answer {
        let route = f
            .route
            .as_deref()
            .filter(|r| !r.is_empty())
            .map(|r| format!(" · путь: {r}"))
            .unwrap_or_default();
        return Headline {
            tone: Tone::Ok,
            title: "Работает".into(),
            detail: format!("Модель ответила {}{route}.", ago_text(ago)),
            action: None,
        };
    }

    if !f.relay_reporting {
        return Headline {
            tone: Tone::Wait,
            title: "Служба запускается".into(),
            detail: "Это занимает до минуты. Потом напишите что-нибудь в Antigravity.".into(),
            action: None,
        };
    }

    Headline {
        tone: Tone::Wait,
        title: "Всё включено".into(),
        detail: "Напишите что-нибудь в чат Antigravity — здесь появится подтверждение, что модель \
                 ответила."
            .into(),
        action: None,
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// «только что», «22 секунды назад», «3 минуты назад», «2 часа назад».
pub fn ago_text(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 15 {
        return "только что".to_string();
    }
    if secs < 60 {
        return format!(
            "{} {} назад",
            secs,
            plural(secs, "секунду", "секунды", "секунд")
        );
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!(
            "{} {} назад",
            mins,
            plural(mins, "минуту", "минуты", "минут")
        );
    }
    let hours = mins / 60;
    format!("{} {} назад", hours, plural(hours, "час", "часа", "часов"))
}

/// Russian counts in three forms.
pub fn plural(n: u64, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    if n % 100 / 10 == 1 {
        return many;
    }
    match n % 10 {
        1 => one,
        2..=4 => few,
        _ => many,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn working() -> Facts {
        Facts {
            admin: true,
            installs_found: true,
            patch_on: true,
            bypass_on: true,
            relay_running: true,
            relay_outdated: false,
            rules: true,
            relay_reporting: true,
            ..Facts::default()
        }
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    /// G50, the report that started this: a refusal, then quiet. Quiet is not
    /// proof - the card must not turn green without a model answer after it.
    #[test]
    fn silence_after_a_refusal_is_not_working() {
        let f = Facts {
            refusal: Some((secs(300), 4)),
            answered: Some(Answered {
                acted: "адреса серверов Google подбираются заново".into(),
                bypassed: false,
            }),
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Fixing);
        assert!(
            h.detail.contains("Отправьте сообщение ещё раз"),
            "{}",
            h.detail
        );
    }

    #[test]
    fn an_answer_after_the_refusal_is_working_and_says_by_which_path() {
        let f = Facts {
            refusal: Some((secs(300), 4)),
            answer: Some(secs(60)),
            route: Some("встроенный выход".into()),
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Ok);
        assert_eq!(h.title, "Работает");
        assert!(h.detail.contains("1 минуту назад"), "{}", h.detail);
        assert!(h.detail.contains("встроенный выход"), "{}", h.detail);
    }

    /// Found live: an answer, then a refusal, then quiet. Once the refusal is
    /// older than the fixing window the card must not fall back to the answer
    /// before it and say «Работает».
    #[test]
    fn an_old_refusal_after_the_last_answer_asks_for_a_check_not_a_green_card() {
        let f = Facts {
            refusal: Some((secs(13 * 60), 0)),
            answer: Some(secs(14 * 60)),
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Wait);
        assert_eq!(h.title, "Нужна проверка");
        // And an answer after that same refusal is working again.
        let f = Facts {
            refusal: Some((secs(13 * 60), 0)),
            answer: Some(secs(60)),
            ..working()
        };
        assert_eq!(headline(&f).tone, Tone::Ok);
    }

    #[test]
    fn a_refusal_after_the_last_answer_is_the_live_problem() {
        let f = Facts {
            refusal: Some((secs(20), 4)),
            answer: Some(secs(600)),
            ..working()
        };
        assert_eq!(headline(&f).tone, Tone::Fixing);
    }

    #[test]
    fn set_up_and_unproven_asks_for_a_message_and_claims_nothing() {
        let h = headline(&working());
        assert_eq!(h.tone, Tone::Wait);
        assert!(!h.detail.contains("работает"), "{}", h.detail);
        assert_eq!(h.action, None);
    }

    #[test]
    fn a_client_that_went_around_us_is_told_so_and_what_to_do() {
        let f = Facts {
            refusal: Some((secs(10), 4)),
            answered: Some(Answered {
                acted: String::new(),
                bypassed: true,
            }),
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Fixing);
        assert!(h.detail.contains("мимо обхода"), "{}", h.detail);
        assert!(
            h.detail.contains("перезапустите Antigravity"),
            "{}",
            h.detail
        );
    }

    #[test]
    fn many_refusals_with_no_answer_stop_promising_and_say_what_to_try() {
        let f = Facts {
            refusal: Some((secs(10), STUCK_LINES)),
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Action);
        assert!(h.detail.contains("Скопировать отчёт"), "{}", h.detail);
    }

    #[test]
    fn switched_off_offers_the_one_button_and_warns_when_it_closes_antigravity() {
        let f = Facts {
            patch_on: false,
            bypass_on: false,
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.action, Some(Action::EnableAll));
        assert!(h.detail.contains("закроется"), "{}", h.detail);
        // Only the bypass off: nothing gets closed.
        let f = Facts {
            bypass_on: false,
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.action, Some(Action::EnableAll));
        assert!(!h.detail.contains("закроется"), "{}", h.detail);
    }

    #[test]
    fn without_admin_the_button_is_elevation_but_only_when_there_is_work_for_it() {
        let f = Facts {
            admin: false,
            bypass_on: false,
            ..working()
        };
        assert_eq!(headline(&f).action, Some(Action::Elevate));
        // Everything is installed and running: an unelevated window has nothing
        // to ask for.
        let f = Facts {
            admin: false,
            answer: Some(secs(30)),
            ..working()
        };
        assert_eq!(headline(&f).tone, Tone::Ok);
    }

    #[test]
    fn a_dead_or_old_service_is_repaired_by_one_button() {
        let f = Facts {
            relay_running: false,
            ..working()
        };
        let h = headline(&f);
        assert_eq!((h.tone, h.action), (Tone::Action, Some(Action::Repair)));
        let f = Facts {
            relay_outdated: true,
            ..working()
        };
        assert_eq!(headline(&f).title, "Служба обхода устарела");
        // Running, current, and still useless: nothing asks it anything.
        let f = Facts {
            rules: false,
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.action, Some(Action::Repair));
        assert_eq!(h.title, "Служба обхода работает не полностью");
    }

    #[test]
    fn nothing_installed_is_said_without_a_button() {
        let f = Facts {
            installs_found: false,
            patch_on: false,
            bypass_on: false,
            ..working()
        };
        let h = headline(&f);
        assert_eq!(h.tone, Tone::Off);
        assert_eq!(h.action, None);
    }

    #[test]
    fn russian_counts_in_three_forms() {
        let minutes = |n| plural(n, "минуту", "минуты", "минут");
        assert_eq!(minutes(1), "минуту");
        assert_eq!(minutes(2), "минуты");
        assert_eq!(minutes(5), "минут");
        assert_eq!(minutes(11), "минут");
        assert_eq!(minutes(21), "минуту");
        assert_eq!(minutes(22), "минуты");
        assert_eq!(minutes(0), "минут");
    }

    #[test]
    fn an_age_reads_as_a_person_would_say_it() {
        assert_eq!(ago_text(secs(3)), "только что");
        assert_eq!(ago_text(secs(22)), "22 секунды назад");
        assert_eq!(ago_text(secs(61)), "1 минуту назад");
        assert_eq!(ago_text(secs(9 * 60)), "9 минут назад");
        assert_eq!(ago_text(secs(2 * 3600 + 5)), "2 часа назад");
        assert_eq!(ago_text(secs(5 * 3600)), "5 часов назад");
    }
}
