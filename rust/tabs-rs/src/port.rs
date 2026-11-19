//! The Rust port of `core-esm/tabs.js`.
//!
//! The JS regexes are not translated to the `regex` crate. They run on UTF-16 code units
//! with `/i` (no `u` flag) semantics, `\s` = JS whitespace, `.` = anything but a line
//! terminator, `\d` = ASCII digit. The `regex` crate differs on all four, so the three
//! patterns are matched by hand and checked against recorded cases.

#![forbid(unsafe_code)]

use serde_json::Value;

use crate::js;

// ---- JS string primitives (UTF-16 code units) -------------------------------------------------

type U = u16;

fn units(s: &str) -> Vec<U> {
    s.encode_utf16().collect()
}

/// JS `\s` (also the set `String.prototype.trim` removes).
fn is_space(c: U) -> bool {
    matches!(c, 0x09..=0x0D | 0x20 | 0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF)
}

/// JS line terminators, the only code units `.` does not match.
fn is_line_terminator(c: U) -> bool {
    matches!(c, 0x0A | 0x0D | 0x2028 | 0x2029)
}

fn is_digit(c: U) -> bool {
    (0x30..=0x39).contains(&c)
}

fn is_dash(c: U) -> bool {
    matches!(c, 0x2D | 0x2013 | 0x2014)
}

/// ECMAScript `Canonicalize` for a regex without the `u` flag but with `i`.
fn canon(c: U) -> U {
    let Some(ch) = char::from_u32(u32::from(c)) else {
        return c; // lone surrogate
    };
    let mut up = ch.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => {
            let Ok(u16c) = U::try_from(u32::from(u)) else {
                return c; // maps outside the BMP
            };
            if c >= 128 && u16c < 128 { c } else { u16c }
        }
        _ => c,
    }
}

/// Whether `lit` matches `s` at `pos` ignoring case the way JS `/i` does.
fn lit_at(s: &[U], pos: usize, lit: &[U]) -> bool {
    pos + lit.len() <= s.len()
        && lit
            .iter()
            .zip(&s[pos..])
            .all(|(a, b)| canon(*a) == canon(*b))
}

fn ws_run_end(s: &[U], mut pos: usize) -> usize {
    while pos < s.len() && is_space(s[pos]) {
        pos += 1;
    }
    pos
}

fn trim(s: &[U]) -> &[U] {
    let start = s.iter().position(|c| !is_space(*c)).unwrap_or(s.len());
    let end = s
        .iter()
        .rposition(|c| !is_space(*c))
        .map_or(start, |i| i + 1);
    &s[start..end]
}

// ---- String(x), x || '' ------------------------------------------------------------------------

fn truthy(v: &Value) -> bool {
    if js::is_undefined(v) {
        return false;
    }
    if let Some(u) = js::string_units(v) {
        return !u.is_empty();
    }
    if let Some(n) = js::as_number(v) {
        return n != 0.0 && !n.is_nan();
    }
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        _ => true,
    }
}

/// JS `String(v)`.
fn to_units(v: &Value) -> Vec<U> {
    if js::is_undefined(v) {
        return units("undefined");
    }
    if let Some(u) = js::string_units(v) {
        return u;
    }
    if let Some(n) = js::as_number(v) {
        return units(&js::number_to_string(n));
    }
    match v {
        Value::Null => units("null"),
        Value::Bool(b) => units(if *b { "true" } else { "false" }),
        Value::Array(items) => {
            let mut out = Vec::new();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(u16::from(b','));
                }
                if !(item.is_null() || js::is_undefined(item)) {
                    out.extend(to_units(item));
                }
            }
            out
        }
        _ => units("[object Object]"),
    }
}

/// `String(x || '')`.
fn string_or_empty(v: &Value) -> Vec<U> {
    if truthy(v) { to_units(v) } else { Vec::new() }
}

// ---- pageTitle ---------------------------------------------------------------------------------

/// `/\s+and\s+\d+\s+more\s+pages?/i`, first match: its `(start, end)`.
fn find_more_pages(s: &[U]) -> Option<(usize, usize)> {
    let and = units("and");
    let more = units("more");
    let page = units("page");
    let es = units("s");
    for start in 0..s.len() {
        let mut p = ws_run_end(s, start);
        if p == start || !lit_at(s, p, &and) {
            continue;
        }
        p += and.len();
        let q = ws_run_end(s, p);
        if q == p {
            continue;
        }
        let mut p = q;
        let d = p;
        while p < s.len() && is_digit(s[p]) {
            p += 1;
        }
        if p == d {
            continue;
        }
        let q = ws_run_end(s, p);
        if q == p || !lit_at(s, q, &more) {
            continue;
        }
        let p = q + more.len();
        let q = ws_run_end(s, p);
        if q == p || !lit_at(s, q, &page) {
            continue;
        }
        let mut end = q + page.len();
        if lit_at(s, end, &es) {
            end += 1;
        }
        return Some((start, end));
    }
    None
}

/// Where `BROWSER_TAIL` starts, if it matches: `\s[-–—]\s(<browser>)\s*$` with `/i`.
fn find_browser_tail(s: &[U]) -> Option<usize> {
    let names: [&str; 6] = [
        "Google Chrome",
        "Mozilla Firefox",
        "Brave",
        "Opera",
        "Vivaldi",
        "Chromium",
    ];
    let ms = units("Microsoft");
    let edge = units("Edge");
    let all_space_from = |p: usize| ws_run_end(s, p) == s.len();
    for start in 0..s.len() {
        if !is_space(s[start]) {
            continue;
        }
        let d = start + 1;
        if d >= s.len() || !is_dash(s[d]) {
            continue;
        }
        let w = d + 1;
        if w >= s.len() || !is_space(s[w]) {
            continue;
        }
        let p = w + 1;
        let mut ok = false;
        for name in names {
            let lit = units(name);
            if lit_at(s, p, &lit) && all_space_from(p + lit.len()) {
                ok = true;
                break;
            }
        }
        if !ok && lit_at(s, p, &ms) {
            let after = p + ms.len();
            for n in 0..=2 {
                // `.{0,2}`: n code units that are not line terminators
                if after + n > s.len() || s[after..after + n].iter().any(|c| is_line_terminator(*c))
                {
                    break;
                }
                if lit_at(s, after + n, &edge) && all_space_from(after + n + edge.len()) {
                    ok = true;
                    break;
                }
            }
        }
        if ok {
            return Some(start);
        }
    }
    None
}

fn page_title_units(raw: &Value) -> Vec<U> {
    let s = string_or_empty(raw);
    let s = match find_more_pages(&s) {
        Some((a, b)) => [&s[..a], &s[b..]].concat(),
        None => s,
    };
    let s = match find_browser_tail(&s) {
        Some(a) => s[..a].to_vec(),
        None => s,
    };
    trim(&s).to_vec()
}

/// Port of `pageTitle` (`core-esm/tabs.js:15`).
pub fn page_title(title: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(js::utf16_string(&page_title_units(&title)))
}

// ---- brandOf -----------------------------------------------------------------------------------

/// Port of `brandOf` (`core-esm/tabs.js:20`).
pub fn brand_of(domain: serde_json::Value) -> Result<serde_json::Value, String> {
    let s = String::from_utf16_lossy(&string_or_empty(&domain));
    let lower = s.to_lowercase();
    let parts: Vec<&str> = lower.split('.').filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return Ok(Value::Null);
    }
    let label = parts[parts.len() - 2];
    Ok(if label.encode_utf16().count() >= 3 {
        Value::String(label.to_string())
    } else {
        Value::Null
    })
}

// ---- matchBanned -------------------------------------------------------------------------------

/// `^(https?://)?(www\.|m\.)?<d>(/.*)?$` with `/i`.
fn domain_matches(t: &[U], d: &[U]) -> bool {
    let http = units("http");
    let sch = units("://");
    let ess = units("s");
    let www = units("www.");
    let m = units("m.");
    // positions after the optional `https?://`
    let mut starts = vec![0usize];
    if lit_at(t, 0, &http) {
        let mut p = http.len();
        let mut tails = vec![p];
        if lit_at(t, p, &ess) {
            p += 1;
            tails.push(p);
        }
        for p in tails {
            if lit_at(t, p, &sch) {
                starts.push(p + sch.len());
            }
        }
    }
    let mut after_host = Vec::new();
    for s in starts {
        after_host.push(s);
        if lit_at(t, s, &www) {
            after_host.push(s + www.len());
        }
        if lit_at(t, s, &m) {
            after_host.push(s + m.len());
        }
    }
    after_host.into_iter().any(|p| {
        if !lit_at(t, p, d) {
            return false;
        }
        let e = p + d.len();
        e == t.len()
            || (t[e] == u16::from(b'/') && t[e + 1..].iter().all(|c| !is_line_terminator(*c)))
    })
}

/// `[-|–—/·:]`
fn is_separator(c: U) -> bool {
    matches!(c, 0x2D | 0x7C | 0x2013 | 0x2014 | 0x2F | 0xB7 | 0x3A)
}

/// `^(\(\d+\)\s*)?(.*?\s*[-|–—/·:]\s*)?<b>(\s*[-–—]\s[^-–—]{1,30})?$` with `/i`.
fn brand_matches(t: &[U], b: &[U]) -> bool {
    let n = t.len();
    // after group 1
    let mut at1 = vec![0usize];
    if n > 0 && t[0] == u16::from(b'(') {
        let mut p = 1;
        while p < n && is_digit(t[p]) {
            p += 1;
        }
        if p > 1 && p < n && t[p] == u16::from(b')') {
            let run_end = ws_run_end(t, p + 1);
            at1.extend(p + 1..=run_end);
        }
    }
    // after group 2 (optional): `.*?\s*SEP\s*`
    let mut at2 = vec![false; n + 1];
    for &p in &at1 {
        at2[p] = true;
        let mut q = p;
        loop {
            // `.*?` has consumed t[p..q]; `\s*` takes w units, then a separator
            let ws_end = ws_run_end(t, q);
            for sep in q..=ws_end {
                if sep < n && is_separator(t[sep]) {
                    let end = ws_run_end(t, sep + 1);
                    at2[sep + 1..=end].fill(true);
                }
            }
            if q < n && !is_line_terminator(t[q]) {
                q += 1;
            } else {
                break;
            }
        }
    }
    (0..=n).filter(|&p| at2[p]).any(|p| {
        if !lit_at(t, p, b) {
            return false;
        }
        let r = p + b.len();
        if r == n {
            return true;
        }
        // `(\s*[-–—]\s[^-–—]{1,30})?$`
        let ws_end = ws_run_end(t, r);
        (r..=ws_end).any(|dash| {
            if dash >= n || !is_dash(t[dash]) || dash + 1 >= n || !is_space(t[dash + 1]) {
                return false;
            }
            let rest = &t[dash + 2..];
            (1..=30).contains(&rest.len()) && rest.iter().all(|c| !is_dash(*c))
        })
    })
}

/// Port of `matchBanned` (`core-esm/tabs.js:33`).
pub fn match_banned(
    title: String,
    brands: Vec<String>,
    domains: Vec<String>,
) -> Result<serde_json::Value, String> {
    let t = page_title_units(&Value::String(title));
    if t.is_empty() {
        return Ok(Value::Null);
    }
    for d in &domains {
        if domain_matches(&t, &units(d)) {
            return Ok(Value::String(d.clone()));
        }
    }
    for b in &brands {
        if brand_matches(&t, &units(b)) {
            return Ok(Value::String(b.clone()));
        }
    }
    Ok(Value::Null)
}
