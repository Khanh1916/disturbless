//! The Rust port of `core-esm/stats.js`.
//!
//! Days are local-time calendar days, as in the original; the calendar maths lives in
//! `shared/tz.rs`. Timestamps stay `f64` like JS numbers so invalid dates behave the same
//! (`NaN-NaN-NaN` keys).

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use crate::js;

use crate::jsstr;
use crate::tz;
use jsstr::{prop, truthy};

fn pad(n: i64) -> String {
    format!("{n:0>2}")
}

fn key_of(ms: f64) -> String {
    match tz::parts(ms) {
        Some(p) => format!("{}-{}-{}", p.year, pad(p.month0 + 1), pad(p.day)),
        None => "NaN-NaN-NaN".to_string(),
    }
}

/// `d.setDate(d.getDate() + n)` on the timestamp.
fn add_days(ms: f64, n: i64) -> f64 {
    match tz::parts(ms) {
        Some(p) => tz::make(
            p.year as f64,
            p.month0 as f64,
            (p.day + n) as f64,
            p.hour as f64,
            p.minute as f64,
            p.second as f64,
            p.milli as f64,
        ),
        None => f64::NAN,
    }
}

/// `new Date(v).getTime()` for the values the app passes: numbers, null, booleans, undefined.
fn date_ms(v: &Value) -> Result<f64, String> {
    if js::is_undefined(v) {
        return Ok(f64::NAN);
    }
    if let Some(n) = js::as_number(v) {
        return Ok(tz::time_clip(n));
    }
    match v {
        Value::Null => Ok(0.0),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err("unsupported Date input (only numbers are used by the app)".into()),
    }
}

/// `x || 0` for a numeric field.
fn number_or_zero(v: &Value) -> Result<f64, String> {
    if !truthy(v) {
        return Ok(0.0);
    }
    js::as_number(v).ok_or_else(|| "unsupported non-numeric value".to_string())
}

/// Port of `dayKey` (`core-esm/stats.js:4`).
pub fn day_key(ts: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(Value::String(key_of(date_ms(&ts)?)))
}

/// Port of `summarize` (`core-esm/stats.js:18`).
pub fn summarize(
    history: serde_json::Value,
    now: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let now = match now {
        Some(v) if !js::is_undefined(&v) => date_ms(&v)?,
        _ => tz::now_ms(),
    };
    let items: Vec<Value> = match &history {
        Value::Array(a) => a.clone(),
        v => match jsstr::as_string(v) {
            Some(s) => String::from_utf16_lossy(&s)
                .chars()
                .map(|c| Value::String(c.to_string()))
                .collect(),
            None => return Err("history is not iterable".into()),
        },
    };

    let mut min_per_day: HashMap<String, f64> = HashMap::new();
    let mut done_days: HashSet<String> = HashSet::new();
    let mut quit_days: HashSet<String> = HashSet::new();
    let (mut total, mut violations) = (0.0, 0.0);
    let (mut completed, mut abandoned, mut emergencies) = (0u32, 0u32, 0u32);

    for h in &items {
        if truthy(&prop(h, "emergency")?) {
            emergencies += 1;
            continue;
        }
        let ended = prop(h, "endedAt")?;
        let started = prop(h, "startedAt")?;
        let k = key_of(date_ms(if truthy(&ended) { &ended } else { &started })?);
        let focused = number_or_zero(&prop(h, "focusedMin")?)?;
        *min_per_day.entry(k.clone()).or_insert(0.0) += focused;
        total += focused;
        violations += number_or_zero(&prop(h, "violations")?)?;
        if truthy(&prop(h, "completed")?) {
            completed += 1;
            done_days.insert(k);
        } else {
            abandoned += 1;
            quit_days.insert(k);
        }
    }

    // current streak, counted backwards from today (or yesterday when today has no session);
    // quitting a session today before finishing one resets it at once
    let today = key_of(now);
    let mut cursor = now;
    if !done_days.contains(&today) {
        cursor = add_days(cursor, -1);
    }
    let mut streak = 0u32;
    if !(quit_days.contains(&today) && !done_days.contains(&today)) {
        while !cursor.is_nan() && done_days.contains(&key_of(cursor)) {
            streak += 1;
            cursor = add_days(cursor, -1);
        }
    }

    // longest streak
    let mut sorted: Vec<&String> = done_days.iter().collect();
    sorted.sort();
    let (mut best, mut run) = (0u32, 0u32);
    let mut prev: Option<f64> = None;
    for k in sorted {
        let nums: Vec<f64> = k
            .split('-')
            .map(|p| p.parse::<f64>().unwrap_or(f64::NAN))
            .collect();
        let at = |i: usize| nums.get(i).copied().unwrap_or(f64::NAN);
        let ts = tz::make(at(0), at(1) - 1.0, at(2), 12.0, 0.0, 0.0, 0.0);
        run = match prev {
            Some(p) if key_of(add_days(p, 1)) == *k => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(ts);
    }

    let mut days = Vec::new();
    let mut week = 0.0;
    for i in (0..=6).rev() {
        let ts = add_days(now, -i);
        let k = key_of(ts);
        let min = min_per_day.get(&k).copied().unwrap_or(0.0);
        week += min;
        let mut d = Map::new();
        d.insert("key".into(), Value::String(k));
        d.insert(
            "dow".into(),
            js::number(tz::parts(ts).map_or(f64::NAN, |p| p.dow as f64)),
        );
        d.insert("min".into(), js::number(min));
        days.push(Value::Object(d));
    }

    let sessions = completed + abandoned;
    let rate = if sessions > 0 {
        js::math_round(f64::from(completed) / f64::from(sessions) * 100.0)
    } else {
        0.0
    };
    let mut out = Map::new();
    let mut put = |k: &str, v: f64| out.insert(k.to_string(), js::number(v));
    put("todayMin", min_per_day.get(&today).copied().unwrap_or(0.0));
    put("weekMin", week);
    put("totalMin", total);
    put("sessions", f64::from(sessions));
    put("completed", f64::from(completed));
    put("abandoned", f64::from(abandoned));
    put("completionRate", rate);
    put("emergencies", f64::from(emergencies));
    put("violations", violations);
    put("streak", f64::from(streak));
    put("bestStreak", f64::from(best));
    out.insert("days".into(), Value::Array(days));
    Ok(Value::Object(out))
}
