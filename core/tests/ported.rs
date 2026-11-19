//! The JS tests of `test/core.test.js` (stats, blocker, tabs, hosts), ported one to one.

use disturbless_core::{hosts_text as hosts, rules, stats, tabs};
use serde_json::{Value, json};

fn local_ms(y: i32, m1: u32, d: u32, h: u32) -> f64 {
    use chrono::{Local, TimeZone};
    Local
        .with_ymd_and_hms(y, m1, d, h, 0, 0)
        .unwrap()
        .timestamp_millis() as f64
}

fn summarize(h: Value, now: f64) -> Value {
    stats::summarize(h, Some(json!(now))).unwrap()
}

#[test]
fn stats_streak_total_completion_rate() {
    let now = local_ms(2026, 11, 16, 15);
    let d = |back: u32, completed: bool, focused: u32, violations: u32| {
        let t = local_ms(2026, 11, 16 - back, 10);
        json!({ "startedAt": t, "endedAt": t + 3_600_000.0, "focusedMin": focused, "completed": completed, "violations": violations })
    };
    let h = json!([
        d(0, true, 50, 0),
        d(1, true, 50, 0),
        d(2, true, 50, 0),
        d(4, true, 50, 0),
        d(5, false, 10, 2)
    ]);
    let s = summarize(h, now);
    assert_eq!(s["streak"], 3);
    assert_eq!(s["bestStreak"], 3);
    assert_eq!(s["todayMin"], 50);
    assert_eq!(s["totalMin"], 210);
    assert_eq!(s["completed"], 4);
    assert_eq!(s["abandoned"], 1);
    assert_eq!(s["completionRate"], 80);
    assert_eq!(s["violations"], 2);
    assert_eq!(s["days"].as_array().unwrap().len(), 7);
    assert_eq!(s["days"][6]["key"], stats::day_key(json!(now)).unwrap());
}

#[test]
fn stats_streak_survives_when_only_yesterday_has_a_session() {
    let now = local_ms(2026, 11, 16, 9);
    let t = local_ms(2026, 11, 15, 10);
    let s = summarize(
        json!([{ "startedAt": t, "endedAt": t, "focusedMin": 30, "completed": true }]),
        now,
    );
    assert_eq!(s["streak"], 1);
}

fn at(back: u32, h: u32, completed: bool, extra: Value) -> Value {
    let t = local_ms(2026, 11, 16 - back, h);
    let mut v = json!({ "startedAt": t, "endedAt": t, "focusedMin": 30, "completed": completed });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

#[test]
fn stats_quitting_today_before_finishing_one_loses_the_streak() {
    let now = local_ms(2026, 11, 16, 15);
    let base = vec![at(1, 10, true, json!({})), at(2, 10, true, json!({}))];
    assert_eq!(summarize(json!(base), now)["streak"], 2);
    let mut quit = base.clone();
    quit.push(at(0, 9, false, json!({})));
    assert_eq!(summarize(json!(quit), now)["streak"], 0);
    let mut both = quit.clone();
    both.push(at(0, 11, true, json!({})));
    assert_eq!(summarize(json!(both), now)["streak"], 3);
}

#[test]
fn stats_emergency_sessions_do_not_count() {
    let now = local_ms(2026, 11, 16, 15);
    let h = json!([
        at(1, 10, true, json!({})),
        at(2, 10, true, json!({})),
        at(
            0,
            9,
            false,
            json!({ "emergency": true, "focusedMin": 0, "violations": 5 })
        )
    ]);
    let s = summarize(h, now);
    assert_eq!(s["streak"], 2);
    assert_eq!(s["emergencies"], 1);
    assert_eq!(s["sessions"], 2);
    assert_eq!(s["abandoned"], 0);
    assert_eq!(s["todayMin"], 0);
    assert_eq!(s["violations"], 0);
}

#[test]
fn stats_giving_up_keeps_no_streak() {
    let now = local_ms(2026, 11, 16, 9);
    let s = summarize(
        json!([{ "startedAt": now, "endedAt": now, "focusedMin": 5, "completed": false }]),
        now,
    );
    assert_eq!(s["streak"], 0);
}

// ---- rules (blocker.evaluate) ----
const SR: &str = "C:\\Windows";
fn w(name: &str, pid: u32, path: &str) -> Value {
    json!({ "pid": pid, "name": name, "title": name, "path": path })
}
fn wd(name: &str, pid: u32) -> Value {
    w(name, pid, &format!("C:\\Program Files\\Apps\\{name}.exe"))
}
fn names(v: Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x["name"].as_str().unwrap().to_string())
        .collect()
}
fn eval(windows: Vec<Value>, cfg: Value) -> Vec<String> {
    names(rules::evaluate(json!(windows), cfg).unwrap())
}

#[test]
fn allow_list_flags_unknown_apps_but_not_allowed_or_system_ones() {
    let v = eval(
        vec![
            wd("chrome", 1),
            wd("Discord", 2),
            w("explorer", 3, "C:\\Windows\\explorer.exe"),
            wd("disturbless", 99),
        ],
        json!({ "appMode": "allow", "allowed": ["Chrome.exe"], "blocked": [], "selfPids": [99], "systemRoot": SR }),
    );
    assert_eq!(v, ["Discord"]);
}

#[test]
fn allow_list_never_closes_windows_programs_or_unverifiable_ones() {
    let sys = vec![
        w("mmc", 10, "C:\\Windows\\System32\\mmc.exe"),
        w("regedit", 11, "c:/windows/regedit.exe"),
        w("mystery", 12, ""),
        wd("msiexec", 13),
        wd("svchost", 14),
        wd("consent", 15),
        wd("winlogon", 16),
        wd("Discord", 17),
    ];
    let v = eval(
        sys,
        json!({ "appMode": "allow", "allowed": [], "blocked": [], "selfPids": [], "systemRoot": SR }),
    );
    assert_eq!(v, ["Discord"]);
}

#[test]
fn allow_list_folder_named_like_windows_is_an_ordinary_app() {
    let v = eval(
        vec![w("tool", 20, "C:\\Windows Apps\\tool.exe")],
        json!({ "appMode": "allow", "allowed": [], "selfPids": [], "systemRoot": SR }),
    );
    assert_eq!(v, ["tool"]);
}

#[test]
fn block_list_never_closes_system_processes_even_when_listed() {
    let v = eval(
        vec![
            w("explorer", 1, "C:\\Windows\\explorer.exe"),
            wd("svchost", 2),
            wd("steam", 3),
        ],
        json!({ "appMode": "block", "blocked": ["explorer", "svchost", "steam"], "selfPids": [], "systemRoot": SR }),
    );
    assert_eq!(v, ["steam"]);
}

#[test]
fn block_list_can_block_a_windows_folder_app_that_is_not_core() {
    let v = eval(
        vec![w("notepad", 5, "C:\\Windows\\System32\\notepad.exe")],
        json!({ "appMode": "block", "blocked": ["notepad"], "selfPids": [], "systemRoot": SR }),
    );
    assert_eq!(v, ["notepad"]);
}

#[test]
fn block_list_only_forbidden_apps_offend() {
    let v = eval(
        vec![wd("chrome", 1), wd("steam", 1)],
        json!({ "appMode": "block", "allowed": [], "blocked": ["steam"], "selfPids": [] }),
    );
    assert_eq!(v, ["steam"]);
}

#[test]
fn app_mode_off_blocks_nothing_and_never_blocks_itself() {
    assert!(
        eval(
            vec![wd("steam", 1)],
            json!({ "appMode": "off", "allowed": [], "blocked": ["steam"] })
        )
        .is_empty()
    );
    assert!(
        eval(
            vec![wd("electron", 7)],
            json!({ "appMode": "allow", "allowed": [], "selfPids": [7] })
        )
        .is_empty()
    );
}

// ---- tabs ----
fn page_title(s: &str) -> String {
    tabs::page_title(json!(s))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}
fn matched(title: &str, brands: &[&str], domains: &[&str]) -> Option<String> {
    let r = tabs::match_banned(
        title.into(),
        brands.iter().map(|s| s.to_string()).collect(),
        domains.iter().map(|s| s.to_string()).collect(),
    )
    .unwrap();
    r.as_str().map(str::to_string)
}

#[test]
fn tabs_page_title_strips_browser_and_more_pages() {
    assert_eq!(page_title("Mèo - YouTube - Google Chrome"), "Mèo - YouTube");
    assert_eq!(
        page_title("Mèo - YouTube and 10 more pages - Personal - Microsoft\u{200b} Edge"),
        "Mèo - YouTube - Personal"
    );
    assert_eq!(page_title("Trang — Mozilla Firefox"), "Trang");
}

#[test]
fn tabs_match_when_the_site_name_is_last() {
    let brands = ["YouTube", "Facebook", "Steam", "Disney+"];
    assert_eq!(
        matched("Mèo nhảy - YouTube - Google Chrome", &brands, &[]).as_deref(),
        Some("YouTube")
    );
    assert_eq!(
        matched(
            "(3) Facebook - Personal - Microsoft\u{200b} Edge",
            &brands,
            &[]
        )
        .as_deref(),
        Some("Facebook")
    );
    assert_eq!(
        matched("Facebook - Microsoft Edge", &brands, &[]).as_deref(),
        Some("Facebook")
    );
    assert_eq!(
        matched("Phim hay | Disney+ - Mozilla Firefox", &brands, &[]).as_deref(),
        Some("Disney+")
    );
    assert_eq!(
        matched("Cat - YouTube - Person 1 - Google Chrome", &brands, &[]).as_deref(),
        Some("YouTube")
    );
}

#[test]
fn tabs_do_not_confuse_articles_or_documents() {
    let brands = ["YouTube", "Steam"];
    assert_eq!(
        matched("Steam engine - Wikipedia - Google Chrome", &brands, &[]),
        None
    );
    assert_eq!(
        matched(
            "Kịch bản video YouTube tuần này - Google Docs - Google Chrome",
            &brands,
            &[]
        ),
        None
    );
    assert_eq!(matched("Bài tập toán - Google Chrome", &brands, &[]), None);
    assert_eq!(matched("", &brands, &[]), None);
}

#[test]
fn tabs_error_pages_showing_only_the_domain_match() {
    assert_eq!(
        matched(
            "youtube.com - Microsoft\u{200b} Edge",
            &[],
            &["youtube.com"]
        )
        .as_deref(),
        Some("youtube.com")
    );
    assert_eq!(
        matched("www.youtube.com - Google Chrome", &[], &["youtube.com"]).as_deref(),
        Some("youtube.com")
    );
    assert_eq!(
        matched("notyoutube.com - Google Chrome", &[], &["youtube.com"]),
        None
    );
}

#[test]
fn tabs_brand_of_takes_the_label_before_the_tld() {
    let b = |d: &str| tabs::brand_of(json!(d)).unwrap();
    assert_eq!(b("tinhte.vn"), json!("tinhte"));
    assert_eq!(b("store.steampowered.com"), json!("steampowered"));
    assert_eq!(b("x.com"), Value::Null);
    assert!(tabs::is_browser("msedge"));
}

// ---- hosts ----
#[test]
fn hosts_clean_and_expand_domains() {
    assert_eq!(
        hosts::clean_domain(json!("https://www.YouTube.com/watch?v=1")).unwrap(),
        json!("youtube.com")
    );
    assert_eq!(
        hosts::clean_domain(json!("không hợp lệ")).unwrap(),
        Value::Null
    );
    assert_eq!(
        hosts::clean_domain(json!("localhost")).unwrap(),
        Value::Null
    );
    let mut e: Vec<String> = hosts::expand_domains(json!([
        "youtube.com",
        "www.youtube.com",
        "music.youtube.com",
        "rác"
    ]))
    .unwrap()
    .as_array()
    .unwrap()
    .iter()
    .map(|v| v.as_str().unwrap().to_string())
    .collect();
    e.sort();
    assert_eq!(
        e,
        [
            "m.youtube.com",
            "music.youtube.com",
            "www.youtube.com",
            "youtube.com"
        ]
    );
}

#[test]
fn hosts_block_is_added_replaced_and_removed_cleanly() {
    let orig = "127.0.0.1 localhost\r\n# my comment\r\n10.0.0.5 nas\r\n";
    let s = |v: Value| v.as_str().unwrap().to_string();
    let blocked = s(hosts::with_block(json!(orig), json!(["youtube.com"])).unwrap());
    assert!(hosts::has_block(json!(blocked)).unwrap().as_bool().unwrap());
    assert!(blocked.contains("0.0.0.0 youtube.com"));
    assert!(blocked.starts_with("127.0.0.1 localhost"));
    let again = s(hosts::with_block(json!(blocked), json!(["facebook.com"])).unwrap());
    assert_eq!(
        again.matches(hosts::BEGIN).count(),
        1,
        "the block is not duplicated"
    );
    assert!(!again.contains("youtube.com"));
    let clean = s(hosts::strip_block(json!(again)).unwrap());
    assert!(!hosts::has_block(json!(clean)).unwrap().as_bool().unwrap());
    assert_eq!(clean.trim(), orig.trim());
}
