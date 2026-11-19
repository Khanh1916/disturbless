//! Messages the Rust side shows (toasts, notifications, tray, errors), in Vietnamese and English.
//! Texts may contain `{name}` placeholders filled by `t`.

use disturbless_core::persona::norm_lang;

fn table(key: &str) -> Option<(&'static str, &'static str)> {
    // (vi, en)
    Some(match key {
        "tray_open" => ("Mở Disturbless", "Open Disturbless"),
        "tray_quit" => ("Thoát", "Quit"),
        "tray_locked" => ("Đang tập trung — không thể thoát", "Focusing — cannot quit"),
        "n_break_title" => ("Giờ nghỉ", "Break time"),
        "n_break_body" => (
            "Nghỉ {min} phút. Ứng dụng đã được mở khoá tạm thời (website vẫn bị chặn).",
            "Take {min} minutes. Apps are unlocked for now (websites stay blocked).",
        ),
        "n_back_title" => ("Hết giờ nghỉ", "Break is over"),
        "n_back_body" => (
            "Vào vòng {round}/{rounds} — quay lại tập trung.",
            "Round {round}/{rounds} — back to focus.",
        ),
        "n_done_title" => ("Hoàn thành!", "Done!"),
        "n_done_body" => (
            "Bạn đã tập trung đủ {min} phút. Giờ được tự do rồi.",
            "You focused for the full {min} minutes. You're free now.",
        ),
        "toast_resume_auto" => (
            "Hết giờ tạm dừng, mèo canh lại nha!",
            "Pause is over, the cat is watching again!",
        ),
        "toast_resume" => ("Tiếp tục học thôi!", "Back to it!"),
        "toast_pause" => (
            "Tạm dừng {min} phút. Hết giờ mèo sẽ canh lại nha.",
            "Paused for {min} minutes. The cat resumes watching afterwards.",
        ),
        "toast_app" => (
            "“{name}” không được phép lúc này — sẽ bị đóng sau {sec} giây. Lưu việc đang làm!",
            "“{name}” is not allowed right now — it will be closed in {sec} seconds. Save your work!",
        ),
        "toast_tab" => (
            "Tab “{name}” không được xem lúc này — mèo sẽ đóng tab đó sau {sec} giây.",
            "The “{name}” tab is off limits right now — the cat will close it in {sec} seconds.",
        ),
        "toast_emergency" => (
            "Mèo hiểu rồi, lo việc của bạn đi nhé. Phiên này không tính giờ, chuỗi ngày vẫn giữ.",
            "Understood, go take care of it. This session is not counted and your streak is kept.",
        ),
        "toast_done" => ("Hoàn thành phiên tập trung!", "Focus session complete!"),
        "toast_early" => (
            "Phiên đã kết thúc sớm — không tính vào streak.",
            "The session ended early and does not count towards your streak.",
        ),
        "hosts_active" => ("Đang chặn website.", "Websites are blocked."),
        "hosts_pending" => (
            "Windows sắp hỏi xin phép để chặn trang web. Bạn bấm “Có” nhé.",
            "Windows is about to ask for permission to block websites. Please click “Yes”.",
        ),
        "hosts_on" => ("Đang chặn {n} trang web", "Blocking {n} websites"),
        "hosts_failed" => (
            "Chưa chặn được trang web (bạn đã bấm “Không”?). Ứng dụng vẫn được chặn bình thường.",
            "Could not block websites (did you click “No”?). Apps are still blocked as usual.",
        ),
        "hosts_releasing" => (
            "Đang mở lại các trang web… Windows sẽ hỏi xin phép, bạn bấm “Có” nhé.",
            "Unblocking websites… Windows will ask for permission, please click “Yes”.",
        ),
        "hosts_release_failed" => (
            "Chưa mở lại được các trang web.",
            "Could not unblock the websites.",
        ),
        "err_running" => ("Đang có một phiên chạy.", "A session is already running."),
        "err_plan" => ("Không tìm thấy kế hoạch.", "Plan not found."),
        "err_allow" => (
            "Hãy chọn ít nhất một ứng dụng được phép dùng.",
            "Choose at least one app you are allowed to use.",
        ),
        "err_locked" => (
            "Không thể thay đổi khi đang có phiên.",
            "Cannot be changed during a session.",
        ),
        "err_nosession" => ("Không có phiên nào đang chạy.", "No session is running."),
        "err_pause_used" => (
            "Phiên này đã dùng lượt tạm dừng rồi.",
            "This session has already used its pause.",
        ),
        "err_reason" => ("Hãy chọn một lý do.", "Please choose a reason."),
        _ => return None,
    })
}

/// The message for `key` in `lang`, with `{name}` placeholders replaced from `args`.
pub fn t(lang: &str, key: &str, args: &[(&str, String)]) -> String {
    let Some((vi, en)) = table(key) else {
        return key.to_string();
    };
    let mut s = if norm_lang(lang) == "en" { en } else { vi }.to_string();
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &[&str] = &[
        "tray_open",
        "tray_quit",
        "tray_locked",
        "n_break_title",
        "n_break_body",
        "n_back_title",
        "n_back_body",
        "n_done_title",
        "n_done_body",
        "toast_resume_auto",
        "toast_resume",
        "toast_pause",
        "toast_app",
        "toast_tab",
        "toast_emergency",
        "toast_done",
        "toast_early",
        "hosts_active",
        "hosts_pending",
        "hosts_on",
        "hosts_failed",
        "hosts_releasing",
        "hosts_release_failed",
        "err_running",
        "err_plan",
        "err_allow",
        "err_locked",
        "err_nosession",
        "err_pause_used",
        "err_reason",
    ];

    #[test]
    fn every_key_has_both_languages_with_the_same_placeholders() {
        for k in KEYS {
            let (vi, en) = table(k).unwrap_or_else(|| panic!("missing {k}"));
            fn ph(s: &'static str) -> Vec<&'static str> {
                let mut v: Vec<&str> = s
                    .match_indices('{')
                    .map(|(i, _)| &s[i..i + s[i..].find('}').unwrap() + 1])
                    .collect();
                v.sort();
                v
            }
            assert_eq!(ph(vi), ph(en), "placeholders differ for {k}");
            assert!(!vi.is_empty() && !en.is_empty());
        }
    }

    #[test]
    fn fills_placeholders_and_falls_back_to_vietnamese() {
        assert_eq!(
            t(
                "en",
                "n_back_body",
                &[("round", "2".into()), ("rounds", "3".into())]
            ),
            "Round 2/3 — back to focus."
        );
        assert!(t("de", "tray_quit", &[]) == "Thoát");
        assert_eq!(t("en", "no_such_key", &[]), "no_such_key");
    }
}
