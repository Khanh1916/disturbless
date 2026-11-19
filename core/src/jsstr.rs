//! JS string and value semantics shared by the ports (UTF-16 code units, `String(x)`, `x || ''`).
//! Included from each crate's `port.rs` with `#[path]`; relies on that crate's `crate::js`.

#![allow(dead_code)]

use serde_json::Value;

use crate::js;

pub type U = u16;

pub fn units(s: &str) -> Vec<U> {
    s.encode_utf16().collect()
}

/// JS `\s`, and the set `String.prototype.trim` removes.
pub fn is_space(c: U) -> bool {
    matches!(
        c,
        0x09..=0x0D | 0x20 | 0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF
    )
}

pub fn is_line_terminator(c: U) -> bool {
    matches!(c, 0x0A | 0x0D | 0x2028 | 0x2029)
}

pub fn trim_start(s: &[U]) -> &[U] {
    let start = s.iter().position(|c| !is_space(*c)).unwrap_or(s.len());
    &s[start..]
}

pub fn trim_end(s: &[U]) -> &[U] {
    let end = s.iter().rposition(|c| !is_space(*c)).map_or(0, |i| i + 1);
    &s[..end]
}

pub fn trim(s: &[U]) -> &[U] {
    trim_end(trim_start(s))
}

/// `toLowerCase()` on code units (lone surrogates become U+FFFD).
pub fn to_lower(s: &[U]) -> Vec<U> {
    String::from_utf16_lossy(s)
        .to_lowercase()
        .encode_utf16()
        .collect()
}

pub fn find(hay: &[U], needle: &[U], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return (from <= hay.len()).then_some(from);
    }
    if hay.len() < needle.len() {
        return None;
    }
    (from..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// A JS string value: plain, or the marked form with lone surrogates.
pub fn as_string(v: &Value) -> Option<Vec<U>> {
    js::string_units(v)
}

pub fn truthy(v: &Value) -> bool {
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
pub fn to_units(v: &Value) -> Vec<U> {
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
pub fn string_or_empty(v: &Value) -> Vec<U> {
    if truthy(v) { to_units(v) } else { Vec::new() }
}

/// `v.<key>`: `undefined` for primitives and missing keys, a TypeError for null/undefined.
pub fn prop(v: &Value, key: &str) -> Result<Value, String> {
    if js::is_undefined(v) || v.is_null() {
        let what = if v.is_null() { "null" } else { "undefined" };
        return Err(format!(
            "Cannot read properties of {what} (reading '{key}')"
        ));
    }
    if js::string_units(v).is_some() || js::as_number(v).is_some() {
        return Ok(js::undefined());
    }
    Ok(match v {
        Value::Object(o) => o.get(key).cloned().unwrap_or_else(js::undefined),
        _ => js::undefined(),
    })
}

/// JS `SameValueZero` for values that crossed as JSON (arrays and objects never compare equal).
pub fn same_value_zero(a: &Value, b: &Value) -> bool {
    if js::is_undefined(a) || js::is_undefined(b) {
        return js::is_undefined(a) && js::is_undefined(b);
    }
    if let (Some(x), Some(y)) = (js::as_number(a), js::as_number(b)) {
        return x == y || (x.is_nan() && y.is_nan());
    }
    if let (Some(x), Some(y)) = (js::string_units(a), js::string_units(b)) {
        return x == y;
    }
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        _ => false,
    }
}

/// A JS string from code units.
pub fn string_value(u: &[U]) -> Value {
    js::utf16_string(u)
}
