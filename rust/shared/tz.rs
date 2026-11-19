//! Local-time calendar functions with JS `Date` semantics (month 0-based, overflow carries over,
//! invalid dates are NaN). On wasm32 (used only by rust-anything's record/diff harness) they
//! call the host's `Date`; elsewhere they use chrono with the machine's time zone.

#![allow(dead_code)]

#[derive(Clone, Copy)]
pub struct Parts {
    pub year: i64,
    pub month0: i64,
    pub day: i64,
    pub dow: i64,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
    pub milli: i64,
}

/// ECMAScript TimeClip.
pub fn time_clip(t: f64) -> f64 {
    if !t.is_finite() || t.abs() > 8.64e15 {
        f64::NAN
    } else {
        t.trunc() + 0.0
    }
}

#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(target_arch = "wasm32")]
pub fn parts(ts: f64) -> Option<Parts> {
    let t = time_clip(ts);
    if t.is_nan() {
        return None;
    }
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(t));
    Some(Parts {
        year: i64::from(d.get_full_year()),
        month0: i64::from(d.get_month()),
        day: i64::from(d.get_date()),
        dow: i64::from(d.get_day()),
        hour: i64::from(d.get_hours()),
        minute: i64::from(d.get_minutes()),
        second: i64::from(d.get_seconds()),
        milli: i64::from(d.get_milliseconds()),
    })
}

#[cfg(target_arch = "wasm32")]
pub fn make(y: f64, m0: f64, d: f64, h: f64, mi: f64, s: f64, ms: f64) -> f64 {
    if ![y, m0, d, h, mi, s, ms].iter().all(|v| v.is_finite()) {
        return f64::NAN;
    }
    let date = js_sys::Date::new_with_year_month_day_hr_min_sec_milli(
        y as u32, m0 as i32, d as i32, h as i32, mi as i32, s as i32, ms as i32,
    );
    time_clip(date.get_time())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_millis() as f64)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parts(ts: f64) -> Option<Parts> {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    let t = time_clip(ts);
    if t.is_nan() {
        return None;
    }
    let dt = Local.timestamp_millis_opt(t as i64).single()?;
    Some(Parts {
        year: i64::from(dt.year()),
        month0: i64::from(dt.month0()),
        day: i64::from(dt.day()),
        dow: i64::from(dt.weekday().num_days_from_sunday()),
        hour: i64::from(dt.hour()),
        minute: i64::from(dt.minute()),
        second: i64::from(dt.second()),
        milli: i64::from(dt.nanosecond() / 1_000_000 % 1000),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn make(y: f64, m0: f64, d: f64, h: f64, mi: f64, s: f64, ms: f64) -> f64 {
    use chrono::{Duration, LocalResult, NaiveDate, TimeZone};
    if ![y, m0, d, h, mi, s, ms].iter().all(|v| v.is_finite()) {
        return f64::NAN;
    }
    let ym = y.trunc() + (m0.trunc() / 12.0).floor();
    let mn = m0.trunc().rem_euclid(12.0);
    let naive = (|| {
        let first = NaiveDate::from_ymd_opt(ym as i32, mn as u32 + 1, 1)?;
        let date = first.checked_add_signed(Duration::try_days((d.trunc() - 1.0) as i64)?)?;
        let time_ms =
            h.trunc() * 3_600_000.0 + mi.trunc() * 60_000.0 + s.trunc() * 1000.0 + ms.trunc();
        date.and_hms_opt(0, 0, 0)?
            .checked_add_signed(Duration::try_milliseconds(time_ms as i64)?)
    })();
    let Some(naive) = naive else { return f64::NAN };
    let ts = match chrono::Local.from_local_datetime(&naive) {
        LocalResult::Single(t) => t.timestamp_millis(),
        LocalResult::Ambiguous(first, _) => first.timestamp_millis(),
        LocalResult::None => {
            // inside a DST gap: JS reads the time with the offset from before the gap
            let before = naive - Duration::hours(3);
            match chrono::Local.from_local_datetime(&before) {
                LocalResult::Single(t) | LocalResult::Ambiguous(t, _) => {
                    let off = t.offset().local_minus_utc();
                    naive.and_utc().timestamp_millis() - i64::from(off) * 1000
                }
                LocalResult::None => return f64::NAN,
            }
        }
    };
    time_clip(ts as f64)
}
