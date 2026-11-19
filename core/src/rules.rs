//! The Rust port of `core-esm/blocker_pure.js`.
//!
//! Safety rules kept exactly as in the original: the app never closes itself or any
//! `SYSTEM_SAFE` process, and in allow-mode it also skips Windows' own programs and programs
//! whose path is unknown.

#![forbid(unsafe_code)]

use serde_json::Value;

use crate::jsstr;
use jsstr::{U, prop, same_value_zero, string_or_empty, string_value, to_lower, trim, units};

/// Processes that are never closed, even when the user lists them as blocked.
const SYSTEM_SAFE: &[&str] = &[
    "explorer",
    "applicationframehost",
    "searchhost",
    "searchapp",
    "searchui",
    "startmenuexperiencehost",
    "shellexperiencehost",
    "textinputhost",
    "ctfmon",
    "lockapp",
    "logonui",
    "dwm",
    "widgets",
    "systemsettings",
    "securityhealthsystray",
    "sihost",
    "runtimebroker",
    "screenclippinghost",
    "snippingtool",
    "taskhostw",
    "fontdrvhost",
    "csrss",
    "winlogon",
    "wininit",
    "smss",
    "services",
    "lsass",
    "svchost",
    "system",
    "registry",
    "userinit",
    "msiexec",
    "wusa",
    "trustedinstaller",
    "tiworker",
    "mousocoreworker",
    "musnotification",
    "musnotificationux",
    "usoclient",
    "wuauclt",
    "consent",
    "credentialuibroker",
    "dllhost",
];

fn norm_units(n: &Value) -> Vec<U> {
    let lower = to_lower(&string_or_empty(n));
    let exe = units(".exe");
    let cut = if lower.ends_with(&exe) {
        lower.len() - exe.len()
    } else {
        lower.len()
    };
    trim(&lower[..cut]).to_vec()
}

/// Port of `normName` (`core-esm/blocker_pure.js:18`).
pub fn norm_name(n: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(string_value(&norm_units(&n)))
}

fn is_slash(c: U) -> bool {
    c == u16::from(b'\\') || c == u16::from(b'/')
}

fn system_path(p: &Value, system_root: &Value) -> bool {
    let mut root = to_lower(&string_or_empty(system_root));
    while root.last().is_some_and(|c| is_slash(*c)) {
        root.pop();
    }
    if root.is_empty() {
        return false;
    }
    let path: Vec<U> = to_lower(&string_or_empty(p))
        .into_iter()
        .map(|c| {
            if c == u16::from(b'/') {
                u16::from(b'\\')
            } else {
                c
            }
        })
        .collect();
    root.push(u16::from(b'\\'));
    path.starts_with(&root)
}

/// Port of `isSystemPath` (`core-esm/blocker_pure.js:21`).
pub fn is_system_path(
    p: serde_json::Value,
    system_root: serde_json::Value,
) -> Result<serde_json::Value, String> {
    Ok(Value::Bool(system_path(&p, &system_root)))
}

/// `(list || []).map(normName)` as a set.
fn name_set(list: &Value) -> Result<Vec<Vec<U>>, String> {
    if !jsstr::truthy(list) {
        return Ok(Vec::new());
    }
    match list {
        Value::Array(a) => Ok(a.iter().map(norm_units).collect()),
        _ => Err("(cfg.list || []).map is not a function".into()),
    }
}

/// `new Set(cfg.selfPids || [])`.
fn pid_set(list: &Value) -> Result<Vec<Value>, String> {
    if !jsstr::truthy(list) {
        return Ok(Vec::new());
    }
    if let Value::Array(a) = list {
        return Ok(a.clone());
    }
    if let Some(s) = jsstr::as_string(list) {
        return Ok(String::from_utf16_lossy(&s)
            .chars()
            .map(|c| Value::String(c.to_string()))
            .collect());
    }
    Err("object is not iterable".into())
}

/// Port of `evaluate` (`core-esm/blocker_pure.js:34`).
pub fn evaluate(
    windows: serde_json::Value,
    cfg: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if !jsstr::truthy(&cfg) {
        return Ok(Value::Array(Vec::new()));
    }
    let mode = prop(&cfg, "appMode")?;
    let mode = jsstr::as_string(&mode).map(|m| String::from_utf16_lossy(&m));
    if mode.as_deref() == Some("off") {
        return Ok(Value::Array(Vec::new()));
    }
    let allowed = name_set(&prop(&cfg, "allowed")?)?;
    let blocked = name_set(&prop(&cfg, "blocked")?)?;
    let selfs = pid_set(&prop(&cfg, "selfPids")?)?;
    let system_root = prop(&cfg, "systemRoot")?;
    let block_mode = mode.as_deref() == Some("block");

    let Value::Array(windows) = &windows else {
        return Err("windows.filter is not a function".into());
    };
    let mut out = Vec::new();
    for w in windows {
        let n = norm_units(&prop(w, "name")?);
        let pid = prop(w, "pid")?;
        let name_text = String::from_utf16_lossy(&n);
        if selfs.iter().any(|s| same_value_zero(s, &pid))
            || SYSTEM_SAFE.contains(&name_text.as_str())
        {
            continue;
        }
        let hit = if block_mode {
            blocked.contains(&n)
        } else if allowed.contains(&n) {
            false
        } else {
            let path = prop(w, "path")?;
            jsstr::truthy(&path) && !system_path(&path, &system_root)
        };
        if hit {
            out.push(w.clone());
        }
    }
    Ok(Value::Array(out))
}

/// Whether a (normalised) process name is one that must never be closed.
pub fn is_system_safe(normalised_name: &str) -> bool {
    SYSTEM_SAFE.contains(&normalised_name)
}
