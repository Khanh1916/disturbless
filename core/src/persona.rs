//! The cat's single personality: the sharp-tongued best friend, in Vietnamese and English.
//! `exit[0..2]` / `app[0..2]` are levels 1 (nudge), 2 (scold), 3+ (harsh). `{app}` is replaced by the
//! offending app's name.

use serde_json::{Value, json};

pub struct Persona {
    pub id: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
    pub desc: &'static str,
    pub pitch: f64,
    pub rate: f64,
    pub unknown_app: &'static str,
    pub start: &'static [&'static str],
    pub cheer: &'static [&'static str],
    pub break_: &'static [&'static str],
    pub exit: [&'static [&'static str]; 3],
    pub app: [&'static [&'static str]; 3],
}

pub const DEFAULT_PERSONA: &str = "bff";

/// `"en"` or `"vi"`; anything else is Vietnamese (the original language of the app).
pub fn norm_lang(lang: &str) -> &'static str {
    if lang.eq_ignore_ascii_case("en") || lang.to_ascii_lowercase().starts_with("en-") {
        "en"
    } else {
        "vi"
    }
}

pub static BFF_VI: Persona = Persona {
    id: "bff",
    name: "Bạn thân mỏ hỗn",
    emoji: "",
    desc: "Trêu chọc, mỉa mai nhẹ — nhưng vẫn quý bạn.",
    pitch: 1.3,
    rate: 1.1,
    unknown_app: "ứng dụng đó",
    start: &[
        "Ừ thì học đi, xem bạn trụ được bao lâu nào.",
        "Hôm nay có chăm thật không đấy? Tui ngồi canh đây.",
    ],
    cheer: &[
        "Ôi, chăm thế cơ à? Cũng được đấy.",
        "Giỏi đó nha. Nhưng đừng có kiêu.",
        "Trụ được tới giờ này là ngầu rồi đó.",
        "Làm tiếp đi, xong tui mời trà sữa tinh thần.",
        "Nhìn bạn học trông nghiêm túc phết.",
        "Đừng có nhìn điện thoại nha, tui thấy hết.",
    ],
    break_: &[
        "Nghỉ đi bạn ơi, nghỉ cho đàng hoàng vào.",
        "Giải lao 5 phút thôi nha, đừng có lố.",
    ],
    exit: [
        &[
            "Ủa, đi đâu vậy? Mới học có tí mà.",
            "Lại lười nữa rồi hả bạn ơi?",
        ],
        &[
            "Bạn định bỏ cuộc thật đó hả? Mất mặt ghê.",
            "Ơ kìa, hứa là học mà? Nuốt lời hả?",
        ],
        &[
            "TRỜI ƠI lại nữa?! Ngồi xuống đi chứ!",
            "Bạn thân kiểu này tui cũng xin chịu luôn. Ngồi xuống!",
        ],
    ],
    app: [
        &[
            "Ủa, mở {app} làm gì vậy má?",
            "Tui thấy {app} rồi nha. Lén lút hả?",
        ],
        &[
            "{app} nữa hả?! Hứa học mà bạn?",
            "Tui đóng {app} giùm luôn cho đỡ tốn công.",
        ],
        &[
            "{app} LẦN NỮA?! Bạn hết thuốc chữa rồi!",
            "Tui không nói nữa, tui đóng {app} luôn!",
        ],
    ],
};

pub static BFF_EN: Persona = Persona {
    id: "bff",
    name: "Sharp-tongued best friend",
    emoji: "",
    desc: "Teasing and a little sarcastic, but still on your side.",
    pitch: 1.2,
    rate: 1.05,
    unknown_app: "that app",
    start: &[
        "Fine, study then. Let's see how long you last.",
        "Really going to focus today? I'm sitting right here, watching.",
    ],
    cheer: &[
        "Oh, so diligent? Not bad, not bad.",
        "Nice work. Just don't get cocky.",
        "Still going at this hour. That's pretty cool.",
        "Keep it up and I'll treat you to a bubble tea. In spirit.",
        "You look suspiciously serious right now.",
        "Don't even look at your phone. I see everything.",
    ],
    break_: &[
        "Take a break, a proper one this time.",
        "Five minutes only. Don't push your luck.",
    ],
    exit: [
        &[
            "Huh, where are you going? You just started.",
            "Getting lazy again, are we?",
        ],
        &[
            "You're really going to give up? How embarrassing.",
            "Hey, you promised to study. Breaking your word?",
        ],
        &[
            "OH COME ON, again?! Sit down already!",
            "I give up on you as a friend. Sit down!",
        ],
    ],
    app: [
        &[
            "Why are you opening {app}, huh?",
            "I saw {app}. Sneaking around?",
        ],
        &[
            "{app} again?! You promised to study!",
            "I'll close {app} for you to save the effort.",
        ],
        &[
            "{app} AGAIN?! You're hopeless!",
            "I'm done talking. I'm closing {app} myself!",
        ],
    ],
};

/// Only one persona exists; any id falls back to it. The language picks the wording.
pub fn persona_of(_id: &str, lang: &str) -> &'static Persona {
    if norm_lang(lang) == "en" {
        &BFF_EN
    } else {
        &BFF_VI
    }
}

pub fn pick<'a>(list: &[&'a str], mut rand: impl FnMut() -> f64) -> &'a str {
    list[((rand() * list.len() as f64).floor() as usize).min(list.len() - 1)]
}

/// The scolding line for `kind` (`"app"` or anything else = exit), `level` 1..=3.
pub fn scold_line(
    persona_id: &str,
    lang: &str,
    kind: &str,
    level: u32,
    app_name: &str,
    rand: impl FnMut() -> f64,
) -> String {
    let p = persona_of(persona_id, lang);
    let lv = (level.clamp(1, 3) - 1) as usize;
    let pool = if kind == "app" { p.app[lv] } else { p.exit[lv] };
    let name = if app_name.is_empty() {
        p.unknown_app
    } else {
        app_name
    };
    pick(pool, rand).replace("{app}", name)
}

/// The persona summaries the renderer lists.
pub fn summaries(lang: &str) -> Value {
    let p = persona_of(DEFAULT_PERSONA, lang);
    json!([{ "id": p.id, "name": p.name, "emoji": p.emoji, "desc": p.desc, "start": p.start }])
}

pub fn voice(p: &Persona) -> Value {
    json!({ "pitch": p.pitch, "rate": p.rate })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scold_line_fills_app_name_and_clamps_level() {
        let line = scold_line("bff", "vi", "app", 9, "Discord", || 0.0);
        assert_eq!(line, "Discord LẦN NỮA?! Bạn hết thuốc chữa rồi!");
        let none = scold_line("whatever", "vi", "app", 1, "", || 0.0);
        assert!(none.contains("ứng dụng đó"));
        // level 0 is treated as level 1; rand 0.99 picks the last line of that pool
        assert_eq!(
            scold_line("bff", "vi", "exit", 0, "", || 0.99),
            "Lại lười nữa rồi hả bạn ơi?"
        );
    }

    #[test]
    fn english_lines_exist_for_every_level_and_kind() {
        assert_eq!(
            scold_line("bff", "en", "app", 9, "Discord", || 0.0),
            "Discord AGAIN?! You're hopeless!"
        );
        assert!(scold_line("bff", "en", "app", 1, "", || 0.0).contains("that app"));
        assert_eq!(
            scold_line("bff", "en-US", "exit", 1, "", || 0.0),
            "Huh, where are you going? You just started."
        );
        for p in [&BFF_VI, &BFF_EN] {
            assert!(
                p.exit
                    .iter()
                    .chain(p.app.iter())
                    .all(|pool| pool.len() == 2)
            );
            assert!(
                p.app
                    .iter()
                    .all(|pool| pool.iter().all(|l| l.contains("{app}")))
            );
        }
    }

    #[test]
    fn norm_lang_defaults_to_vietnamese() {
        assert_eq!(norm_lang("EN"), "en");
        assert_eq!(norm_lang("en-GB"), "en");
        assert_eq!(norm_lang("vi"), "vi");
        assert_eq!(norm_lang("de"), "vi");
        assert_eq!(norm_lang(""), "vi");
    }
}
