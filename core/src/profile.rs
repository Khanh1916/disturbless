//! Profile sanitising (what `sanitizeProfile` did in `main.js`) and the three strictness levels.

use serde_json::{Map, Value, json};

use crate::jsstr;
use crate::{hosts_text, js, rules, tabs};

#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub cooldown_sec: f64,
    pub force_kill: bool,
    pub sentences: usize,
}

pub fn level(id: &str) -> Option<Level> {
    match id {
        "easy" => Some(Level {
            cooldown_sec: 30.0,
            force_kill: false,
            sentences: 1,
        }),
        "normal" => Some(Level {
            cooldown_sec: 120.0,
            force_kill: true,
            sentences: 2,
        }),
        "hard" => Some(Level {
            cooldown_sec: 300.0,
            force_kill: true,
            sentences: 3,
        }),
        _ => None,
    }
}

pub fn is_level(id: &str) -> bool {
    level(id).is_some()
}

/// JS `Number(v)` for the values a profile can carry.
pub fn to_number(v: &Value) -> f64 {
    if js::is_undefined(v) {
        return f64::NAN;
    }
    if let Some(n) = js::as_number(v) {
        return n;
    }
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::String(s) => {
            let t = String::from_utf16_lossy(jsstr::trim(&jsstr::units(s)));
            if t.is_empty() {
                0.0
            } else {
                t.parse().unwrap_or(f64::NAN)
            }
        }
        _ => f64::NAN,
    }
}

fn clamp(v: Option<&Value>, lo: f64, hi: f64, default: f64) -> f64 {
    let n = v.map_or(f64::NAN, to_number);
    if n.is_finite() {
        n.max(lo).min(hi)
    } else {
        default
    }
}

fn str_of(v: &Value) -> Option<String> {
    jsstr::as_string(v).map(|u| String::from_utf16_lossy(&u))
}

fn uniq(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in items {
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// `/^[\w .\-+()]+$/`
fn plausible_app_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '_' | ' ' | '.' | '-' | '+' | '(' | ')')
        })
}

fn list(p: &Value, key: &str) -> Vec<Value> {
    match p.get(key) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    }
}

fn apps(p: &Value, key: &str) -> Vec<Value> {
    let names = list(p, key)
        .iter()
        .filter_map(|v| rules::norm_name(v.clone()).ok().and_then(|n| str_of(&n)))
        .filter(|n| plausible_app_name(n))
        .collect();
    uniq(names).into_iter().map(Value::String).collect()
}

fn sites(p: &Value) -> Vec<String> {
    let cleaned = list(p, "blockedSites")
        .iter()
        .filter_map(|v| {
            hosts_text::clean_domain(v.clone())
                .ok()
                .and_then(|d| str_of(&d))
        })
        .collect();
    uniq(cleaned)
}

fn slice_trim(v: &Value, max_units: usize) -> String {
    let u = jsstr::to_units(v);
    let t = jsstr::trim(&u);
    String::from_utf16_lossy(&t[..t.len().min(max_units)])
}

/// Clean any profile coming from the UI or the wizard into a safe, complete one.
/// `dev` allows sessions as short as three seconds (used only for testing).
pub fn sanitize(p: &Value, id: Option<&str>, dev: bool) -> Value {
    let min_work = if dev { 0.05 } else { 1.0 };
    let site_list = sites(p);
    let mut titles: Vec<String> = list(p, "blockedTitles")
        .iter()
        .map(|s| slice_trim(s, 40))
        .filter(|s| !s.is_empty())
        .collect();
    for d in &site_list {
        if let Some(b) = tabs::brand_of(Value::String(d.clone()))
            .ok()
            .and_then(|b| str_of(&b))
        {
            titles.push(b);
        }
    }
    let mut titles = uniq(titles);
    titles.truncate(120);

    let get = |k: &str| p.get(k);
    let is = |k: &str, want: &str| get(k).and_then(Value::as_str) == Some(want);
    let one_of = |k: &str, options: &[&str], default: &str| {
        get(k)
            .and_then(Value::as_str)
            .filter(|s| options.contains(s))
            .unwrap_or(default)
            .to_string()
    };
    let name = slice_trim(
        &get("name").cloned().unwrap_or(Value::Null).pipe_truthy(),
        60,
    );
    let id = id
        .map(str::to_string)
        .or_else(|| {
            get("id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let mut m = Map::new();
    m.insert("id".into(), json!(id));
    m.insert(
        "name".into(),
        json!(if name.is_empty() {
            "Hồ sơ mới"
        } else {
            &name
        }),
    );
    m.insert(
        "mode".into(),
        json!(if is("mode", "fullscreen") {
            "fullscreen"
        } else {
            "apps"
        }),
    );
    m.insert(
        "workMin".into(),
        js::number(clamp(get("workMin"), min_work, 480.0, 25.0)),
    );
    m.insert(
        "breakMin".into(),
        js::number(js::math_round(clamp(get("breakMin"), 0.0, 120.0, 0.0))),
    );
    m.insert(
        "rounds".into(),
        js::number(js::math_round(clamp(get("rounds"), 1.0, 12.0, 1.0))),
    );
    m.insert(
        "appMode".into(),
        json!(one_of("appMode", &["allow", "block", "off"], "off")),
    );
    m.insert("allowedApps".into(), Value::Array(apps(p, "allowedApps")));
    m.insert("blockedApps".into(), Value::Array(apps(p, "blockedApps")));
    m.insert("blockedSites".into(), json!(site_list));
    m.insert("blockedTitles".into(), json!(titles));
    m.insert(
        "closeTabs".into(),
        json!(get("closeTabs") != Some(&Value::Bool(false))),
    );
    m.insert(
        "goal".into(),
        json!(one_of("goal", &["study", "work", "read"], "study")),
    );
    // what this session is for (shown on the clock and lock screen); optional
    let intent = slice_trim(
        &get("intent").cloned().unwrap_or(Value::Null).pipe_truthy(),
        80,
    );
    if !intent.is_empty() {
        m.insert("intent".into(), json!(intent));
    }
    if let Some(s) = get("strictness")
        .and_then(Value::as_str)
        .filter(|s| is_level(s))
    {
        m.insert("strictness".into(), json!(s));
    }
    Value::Object(m)
}

/// `String(v || '')` as a value, so `slice_trim` can take it.
trait PipeTruthy {
    fn pipe_truthy(self) -> Value;
}
impl PipeTruthy for Value {
    fn pipe_truthy(self) -> Value {
        if jsstr::truthy(&self) {
            self
        } else {
            Value::String(String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_numbers_and_defaults_the_rest() {
        let p = sanitize(
            &json!({ "workMin": 9999, "breakMin": -3, "rounds": 2.6, "appMode": "weird", "goal": "x", "mode": "?" }),
            Some("a"),
            false,
        );
        assert_eq!(p["id"], "a");
        assert_eq!(p["workMin"], 480);
        assert_eq!(p["breakMin"], 0);
        assert_eq!(p["rounds"], 3);
        assert_eq!(p["appMode"], "off");
        assert_eq!(p["goal"], "study");
        assert_eq!(p["mode"], "apps");
        assert_eq!(p["name"], "Hồ sơ mới");
        assert_eq!(p["closeTabs"], true);
        assert!(p.get("strictness").is_none());
    }

    #[test]
    fn short_sessions_only_in_dev_mode() {
        assert_eq!(
            sanitize(&json!({ "workMin": 0.1 }), None, false)["workMin"],
            1
        );
        assert_eq!(
            sanitize(&json!({ "workMin": 0.1 }), None, true)["workMin"],
            0.1
        );
        assert_eq!(
            sanitize(&json!({ "workMin": "abc" }), None, false)["workMin"],
            25
        );
    }

    #[test]
    fn intent_is_trimmed_cut_to_80_and_optional() {
        assert!(sanitize(&json!({}), None, false).get("intent").is_none());
        assert!(
            sanitize(&json!({ "intent": "   " }), None, false)
                .get("intent")
                .is_none()
        );
        assert_eq!(
            sanitize(&json!({ "intent": "  Ôn chương 3  " }), None, false)["intent"],
            "Ôn chương 3"
        );
        let long = "a".repeat(200);
        assert_eq!(
            sanitize(&json!({ "intent": long }), None, false)["intent"]
                .as_str()
                .unwrap()
                .len(),
            80
        );
    }

    #[test]
    fn sanitize_cleans_apps_sites_and_derives_tab_titles() {
        let p = sanitize(
            &json!({
                "allowedApps": ["Chrome.exe", "chrome", "bad;name", "", 5],
                "blockedSites": ["https://www.YouTube.com/x", "youtube.com", "rác", "tinhte.vn"],
                "blockedTitles": ["  Facebook  ", "", "YouTube"],
                "closeTabs": false,
                "strictness": "hard"
            }),
            None,
            false,
        );
        assert_eq!(p["allowedApps"], json!(["chrome", "5"]));
        assert_eq!(p["blockedSites"], json!(["youtube.com", "tinhte.vn"]));
        // de-duplication is case-sensitive, exactly like the original `new Set`
        assert_eq!(
            p["blockedTitles"],
            json!(["Facebook", "YouTube", "youtube", "tinhte"])
        );
        assert_eq!(p["closeTabs"], false);
        assert_eq!(p["strictness"], "hard");
    }
}
