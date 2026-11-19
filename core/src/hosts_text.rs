//! The Rust port of `core-esm/hosts_pure.js`.
//!
//! Text is handled as UTF-16 code units like the JS original. The regex uses are matched by
//! hand: `DOMAIN_RE` needs a lookahead (not in the `regex` crate) and the block regex is a
//! literal marker pair with a lazy gap.

#![forbid(unsafe_code)]

use serde_json::Value;

use crate::js;

use crate::jsstr;
use jsstr::{U, find, string_or_empty, string_value, to_lower, trim, trim_end, units};

pub const BEGIN: &str = "# >>> DISTURBLESS BEGIN (không sửa đoạn này, tự gỡ khi phiên kết thúc)";
pub const END: &str = "# <<< DISTURBLESS END";

const CR: U = 0x0D;
const LF: U = 0x0A;

fn alnum(c: U) -> bool {
    matches!(c, 0x30..=0x39 | 0x61..=0x7A)
}

/// `^(?=.{1,253}$)([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$`
fn is_domain(d: &[U]) -> bool {
    if d.is_empty() || d.len() > 253 {
        return false;
    }
    let labels: Vec<&[U]> = d.split(|c| *c == u16::from(b'.')).collect();
    let Some((tld, hosts)) = labels.split_last() else {
        return false;
    };
    if hosts.is_empty() || tld.len() < 2 || !tld.iter().all(|c| matches!(c, 0x61..=0x7A)) {
        return false;
    }
    hosts.iter().all(|l| {
        !l.is_empty()
            && l.len() <= 63
            && alnum(l[0])
            && alnum(l[l.len() - 1])
            && l.iter().all(|c| alnum(*c) || *c == u16::from(b'-'))
    })
}

fn clean_domain_units(raw: &Value) -> Option<Vec<U>> {
    let s = to_lower(trim(&string_or_empty(raw)));
    // .replace(/^[a-z]+:\/\//, '')
    let mut start = 0;
    let mut p = 0;
    while p < s.len() && matches!(s[p], 0x61..=0x7A) {
        p += 1;
    }
    if p > 0 && s[p..].starts_with(&units("://")) {
        start = p + 3;
    }
    // .split(/[/?#:]/)[0]
    let rest = &s[start..];
    let cut = rest
        .iter()
        .position(|c| matches!(*c, 0x2F | 0x3F | 0x23 | 0x3A))
        .unwrap_or(rest.len());
    let mut d = &rest[..cut];
    // .replace(/^www\./, '')
    if d.starts_with(&units("www.")) {
        d = &d[4..];
    }
    is_domain(d).then(|| d.to_vec())
}

/// Port of `cleanDomain` (`core-esm/hosts_pure.js:8`).
pub fn clean_domain(raw: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(clean_domain_units(&raw).map_or(Value::Null, |d| string_value(&d)))
}

/// Port of `expandDomains` (`core-esm/hosts_pure.js:15`).
pub fn expand_domains(list: serde_json::Value) -> Result<serde_json::Value, String> {
    let items: Vec<Value> = if !jsstr::truthy(&list) {
        Vec::new()
    } else if let Value::Array(a) = &list {
        a.clone()
    } else if let Some(s) = jsstr::as_string(&list) {
        // a string iterates by code point; a single character is never a valid domain
        String::from_utf16_lossy(&s)
            .chars()
            .map(|c| Value::String(c.to_string()))
            .collect()
    } else {
        return Err("(list || []) is not iterable".into());
    };
    let mut out: Vec<Vec<U>> = Vec::new();
    let mut add = |d: Vec<U>| {
        if !out.contains(&d) {
            out.push(d);
        }
    };
    for raw in &items {
        let Some(d) = clean_domain_units(raw) else {
            continue;
        };
        let labels = d.iter().filter(|c| **c == u16::from(b'.')).count() + 1;
        add(d.clone());
        if labels <= 2 {
            add([units("www."), d.clone()].concat());
            add([units("m."), d].concat());
        }
    }
    Ok(Value::Array(out.iter().map(|d| string_value(d)).collect()))
}

fn text_units(text: &Value, method: &str) -> Result<Vec<U>, String> {
    if text.is_null() {
        return Err(format!(
            "Cannot read properties of null (reading '{method}')"
        ));
    }
    jsstr::as_string(text).ok_or_else(|| format!("text.{method} is not a function"))
}

fn eat(s: &[U], p: usize, c: U) -> usize {
    if s.get(p) == Some(&c) { p + 1 } else { p }
}

/// Replace every `\r?\n?BEGIN[\s\S]*?END\r?\n?` match with `\r\n`.
fn replace_blocks(t: &[U]) -> Vec<U> {
    let (begin, end) = (units(BEGIN), units(END));
    let mut out = Vec::with_capacity(t.len());
    let mut i = 0;
    let mut copied = 0;
    while i <= t.len() {
        // `\r?\n?` then BEGIN: the optional breaks sit right before the marker
        let after_cr = eat(t, i, CR);
        let after_lf = eat(t, after_cr, LF);
        let mut found = None;
        for p in [after_lf, after_cr, i] {
            if t[p..].starts_with(&begin)
                && let Some(e) = find(t, &end, p + begin.len())
            {
                found = Some(eat(t, eat(t, e + end.len(), CR), LF));
                break;
            }
        }
        match found {
            Some(stop) => {
                out.extend_from_slice(&t[copied..i]);
                out.extend_from_slice(&[CR, LF]);
                i = stop;
                copied = stop;
            }
            None => i += 1,
        }
    }
    out.extend_from_slice(&t[copied.min(t.len())..]);
    out
}

/// `.replace(/(\r?\n){3,}/g, '\r\n\r\n')`
fn squeeze_breaks(t: &[U]) -> Vec<U> {
    let mut out = Vec::with_capacity(t.len());
    let mut i = 0;
    while i < t.len() {
        let mut p = i;
        let mut count = 0;
        loop {
            let q = eat(t, p, CR);
            if t.get(q) == Some(&LF) {
                p = q + 1;
                count += 1;
            } else {
                break;
            }
        }
        if count >= 3 {
            out.extend_from_slice(&[CR, LF, CR, LF]);
            i = p;
        } else {
            out.push(t[i]);
            i += 1;
        }
    }
    out
}

fn strip_units(t: &[U]) -> Vec<U> {
    let replaced = squeeze_breaks(&replace_blocks(t));
    let mut out = trim_end(&replaced).to_vec();
    out.extend_from_slice(&[CR, LF]);
    out
}

/// Port of `stripBlock` (`core-esm/hosts_pure.js:26`).
pub fn strip_block(text: serde_json::Value) -> Result<serde_json::Value, String> {
    let t = text_units(&text, "replace")?;
    Ok(string_value(&strip_units(&t)))
}

/// Port of `withBlock` (`core-esm/hosts_pure.js:32`).
pub fn with_block(
    text: serde_json::Value,
    domains: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let base = strip_units(&text_units(&text, "replace")?);
    // `domains.length`, then `for (const d of domains)`
    let items: Vec<Vec<U>> = match &domains {
        Value::Array(a) => a.iter().map(jsstr::to_units).collect(),
        Value::Null => return Err("Cannot read properties of null (reading 'length')".into()),
        v if js::is_undefined(v) => {
            return Err("Cannot read properties of undefined (reading 'length')".into());
        }
        v => match jsstr::as_string(v) {
            Some(s) => String::from_utf16_lossy(&s)
                .chars()
                .map(|c| units(&c.to_string()))
                .collect(),
            None => Vec::new(),
        },
    };
    if items.is_empty() {
        return Ok(string_value(&base));
    }
    let crlf = units("\r\n");
    let mut out = base;
    out.extend(&crlf);
    let mut lines: Vec<Vec<U>> = vec![units(BEGIN)];
    for d in &items {
        lines.push([units("0.0.0.0 "), d.clone()].concat());
        lines.push([units(":: "), d.clone()].concat());
    }
    lines.push(units(END));
    out.extend(lines.join(crlf.as_slice()));
    out.extend(&crlf);
    Ok(string_value(&out))
}

/// Port of `hasBlock` (`core-esm/hosts_pure.js:41`).
pub fn has_block(text: serde_json::Value) -> Result<serde_json::Value, String> {
    let begin = units(BEGIN);
    if let Value::Array(a) = &text {
        return Ok(Value::Bool(
            a.iter()
                .any(|v| jsstr::as_string(v).is_some_and(|s| s == begin)),
        ));
    }
    let t = text_units(&text, "includes")?;
    Ok(Value::Bool(find(&t, &begin, 0).is_some()))
}
