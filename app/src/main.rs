//! Disturbless, Tauri edition. The pure logic lives in `disturbless-core`; this file wires it to
//! Windows (`winapi`), to the windows/tray, and to the web UI (`shim.js` provides `window.api`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod winapi;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use disturbless_core::engine::{PhaseKind, Session, State};
use disturbless_core::store::Store;
use disturbless_core::{hosts_text, persona, profile as prof, rules, stats, tabs, tz};
use serde_json::{Map, Value, json};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::PageLoadEvent;
use tauri::window::Color;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, RunEvent, WebviewUrl,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

const GRACE_MS: u64 = 5000; // warning before a forbidden app is closed
const FORCE_MS: u64 = 4000; // after a polite close, kill it if it is still alive
const TAB_GRACE_MS: u64 = 4000;
const CAT_IMAGES: &str = env!("DL_CAT_IMAGES");
const SHIM: &str = include_str!("shim.js");

const EMERGENCY_REASONS: [&str; 6] = ["health", "call", "need", "mistake", "bug", "other"];

struct PendingApp {
    first: Instant,
    closed: bool,
    forced: bool,
    closed_at: Instant,
}

struct PendingTab {
    title: String,
    pid: u32,
    first: Instant,
    closed: bool,
}

struct Ctl {
    store: Store,
    session: Option<Session>,
    dev: bool,
    skip_hosts: bool,
    hosts_status: Value,
    stale_hosts: bool,
    last_key: Option<String>,
    last_phase: Option<PhaseKind>,
    tick_count: u64,
    pending: HashMap<u32, PendingApp>,
    tab_pending: Option<PendingTab>,
    quitting: bool,
    scold_active: bool,
    watcher_on: bool,
    gate_active: bool,
    /// Apps and tabs that pulled the user away this session, with the reason they gave at the pause gate.
    distractions: Vec<Value>,
}

type Shared = Arc<Mutex<Ctl>>;

/// Run `f` with the controller locked.
fn with<R>(app: &AppHandle, f: impl FnOnce(&mut Ctl) -> R) -> R {
    let state = app.state::<Shared>();
    let mut g = state.lock().unwrap_or_else(|p| p.into_inner());
    f(&mut g)
}

// ───────────────────────── helpers ─────────────────────────

fn s_of<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn arr_len(v: &Value, k: &str) -> usize {
    v.get(k).and_then(Value::as_array).map_or(0, Vec::len)
}

fn dev_number(name: &str) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

fn level_cfg(ctl: &Ctl, id: &str) -> Value {
    let l = prof::level(id)
        .or_else(|| prof::level("normal"))
        .expect("normal exists");
    let cd = dev_number("DL_COOLDOWN");
    let pm = dev_number("DL_PAUSE_MS");
    json!({
        "cooldownSec": if ctl.dev && cd > 0.0 { cd } else { l.cooldown_sec },
        "forceKill": l.force_kill,
        "sentences": l.sentences,
        "lang": lang(ctl),
        "pauseMs": if ctl.dev && pm > 0.0 { pm } else { 10.0 * 60_000.0 },
    })
}

fn settings_str(ctl: &Ctl, key: &str) -> String {
    ctl.store
        .settings()
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// UI language of the user: "vi" or "en".
fn lang(ctl: &Ctl) -> &'static str {
    persona::norm_lang(
        ctl.store
            .settings()
            .get("lang")
            .and_then(Value::as_str)
            .unwrap_or("vi"),
    )
}

fn lang_of(app: &AppHandle) -> &'static str {
    with(app, |c| lang(c))
}

/// A message in the user's language.
fn tx(ctl: &Ctl, key: &str, args: &[(&str, String)]) -> String {
    i18n::t(lang(ctl), key, args)
}

fn toast(app: &AppHandle, text: &str, kind: &str) {
    let _ = app.emit("toast", json!({ "text": text, "kind": kind }));
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

fn on_main(app: &AppHandle, f: impl FnOnce(&AppHandle) + Send + 'static) {
    let a = app.clone();
    let _ = app.run_on_main_thread(move || f(&a));
}

fn after(app: &AppHandle, ms: u64, f: impl FnOnce(&AppHandle) + Send + 'static) {
    let a = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(ms));
        f(&a);
    });
}

// ───────────────────────── state shown to the UI ─────────────────────────

fn public_session(ctl: &Ctl) -> Value {
    let Some(s) = &ctl.session else {
        return json!({ "active": false });
    };
    let p = &s.profile;
    let mut m = Map::new();
    m.insert("active".into(), json!(true));
    m.insert("id".into(), json!(s.id));
    m.insert("profileName".into(), json!(s_of(p, "name")));
    m.insert("goal".into(), json!(s_of(p, "goal")));
    m.insert("intent".into(), json!(s_of(p, "intent")));
    m.insert("mode".into(), json!(s_of(p, "mode")));
    m.insert("appMode".into(), json!(s_of(p, "appMode")));
    m.insert(
        "closeTabs".into(),
        json!(p.get("closeTabs").and_then(Value::as_bool).unwrap_or(false)),
    );
    let strict = Some(s_of(p, "strictness"))
        .filter(|x| !x.is_empty())
        .map_or_else(|| settings_str(ctl, "strictness"), str::to_string);
    m.insert("strictness".into(), json!(strict));
    m.insert("sites".into(), json!(arr_len(p, "blockedSites")));
    m.extend(s.state().to_json());
    m.insert("violations".into(), json!(s.violations));
    m.insert("restarts".into(), json!(s.restarts));
    m.insert("exit".into(), s.exit_status().to_json());
    m.insert("hosts".into(), ctl.hosts_status.clone());
    m.insert("graceMs".into(), json!(GRACE_MS));
    Value::Object(m)
}

fn full_state(ctl: &Ctl) -> Value {
    let history = ctl
        .store
        .data
        .get("history")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let recent: Vec<Value> = history
        .as_array()
        .map(|h| h.iter().rev().take(100).cloned().collect())
        .unwrap_or_default();
    json!({
        "profiles": ctl.store.data.get("profiles").cloned().unwrap_or_else(|| json!([])),
        "settings": Value::Object(ctl.store.settings().clone()),
        "history": recent,
        "stats": stats::summarize(history, None).unwrap_or(Value::Null),
        "staleHosts": ctl.stale_hosts,
        "hosts": ctl.hosts_status,
        "session": public_session(ctl),
        "dev": ctl.dev,
        "lastPlan": ctl.store.data.get("lastPlan").cloned().unwrap_or(Value::Null),
        "personas": persona::summaries(lang(ctl)),
    })
}

fn push_data(app: &AppHandle, ctl: &Ctl) {
    let _ = app.emit("data", full_state(ctl));
    rebuild_tray(app, ctl.session.is_some());
}

fn persist(ctl: &mut Ctl) {
    if let Some(s) = &ctl.session {
        let mut a = s.to_json();
        a["savedAt"] = json!(tz::now_ms());
        ctl.store.data.insert("active".into(), a);
    }
    let _ = ctl.store.save();
}

// ───────────────────────── windows ─────────────────────────

fn shim(lock: bool, query: &str) -> String {
    SHIM.replace("__CAT_IMAGES__", CAT_IMAGES)
        .replace("__LOCK__", if lock { "true" } else { "false" })
        .replace("__QUERY__", query)
}

fn show_main(app: &AppHandle) {
    on_main(app, |app| {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
            return;
        }
        let built = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("Disturbless")
            .inner_size(1040.0, 740.0)
            .min_inner_size(860.0, 620.0)
            .background_color(Color(15, 18, 32, 255))
            .initialization_script(shim(false, ""))
            .build();
        if let Ok(w) = built {
            let _ = w.set_focus();
        }
    });
}

fn hide_main(app: &AppHandle) {
    on_main(app, |app| {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    });
}

fn overlay_labels(app: &AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|l| l.starts_with("overlay"))
        .cloned()
        .collect()
}

fn show_overlay(app: &AppHandle) {
    on_main(app, |app| {
        if !overlay_labels(app).is_empty() {
            return;
        }
        let monitors = app.available_monitors().unwrap_or_default();
        let primary = app.primary_monitor().ok().flatten();
        for (i, m) in monitors.iter().enumerate() {
            let is_primary = primary
                .as_ref()
                .is_some_and(|p| p.position() == m.position() && p.size() == m.size());
            let label = if is_primary {
                "overlay-primary".to_string()
            } else {
                format!("overlay-{i}")
            };
            let query = format!("?mode=overlay&primary={}", u8::from(is_primary));
            let built =
                WebviewWindowBuilder::new(app, &label, WebviewUrl::App("overlay.html".into()))
                    .decorations(false)
                    .resizable(false)
                    .maximizable(false)
                    .minimizable(false)
                    .closable(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .visible(false)
                    .background_color(Color(11, 14, 26, 255))
                    .initialization_script(shim(true, &query))
                    .build();
            if let Ok(w) = built {
                let _ = w.set_position(PhysicalPosition::new(m.position().x, m.position().y));
                let _ = w.set_size(PhysicalSize::new(m.size().width, m.size().height));
                let _ = w.show();
                if is_primary {
                    let _ = w.set_focus();
                }
            }
        }
    });
}

fn hide_overlay(app: &AppHandle) {
    on_main(app, |app| {
        for l in overlay_labels(app) {
            if let Some(w) = app.get_webview_window(&l) {
                let _ = w.destroy();
            }
        }
    });
}

/// Keep the lock screen on top and focused (it re-asserts itself twice a second).
fn keep_overlay_on_top(app: &AppHandle) {
    on_main(app, |app| {
        for l in overlay_labels(app) {
            if let Some(w) = app.get_webview_window(&l) {
                let _ = w.set_always_on_top(true);
                if l == "overlay-primary" && !w.is_focused().unwrap_or(true) {
                    let _ = w.set_focus();
                }
            }
        }
    });
}

fn show_mini(app: &AppHandle) {
    on_main(app, |app| {
        if app.get_webview_window("mini").is_some() {
            return;
        }
        let built = WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("overlay.html".into()))
            .decorations(false)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .focused(false)
            .inner_size(330.0, 150.0)
            .background_color(Color(18, 22, 42, 255))
            .initialization_script(shim(true, "?mode=mini"))
            .build();
        if let Ok(w) = built {
            if let Ok(Some(m)) = app.primary_monitor() {
                let scale = m.scale_factor();
                let wa = m.work_area();
                let x = wa.position.x + wa.size.width as i32 - (346.0 * scale) as i32;
                let y = wa.position.y + (16.0 * scale) as i32;
                let _ = w.set_position(PhysicalPosition::new(x, y));
            }
            let _ = w.show();
        }
    });
}

fn hide_mini(app: &AppHandle) {
    on_main(app, |app| {
        if let Some(w) = app.get_webview_window("mini") {
            let _ = w.destroy();
        }
    });
}

fn scold_ms(level: u32) -> u64 {
    match level {
        1 => 3800,
        2 => 4600,
        _ => 6200,
    }
}

fn persona_payload(ctl: &Ctl, text: &str) -> Value {
    let p = persona::persona_of(&settings_str(ctl, "persona"), lang(ctl));
    json!({
        "text": text,
        "lang": lang(ctl),
        "persona": { "id": p.id, "name": p.name, "voice": persona::voice(p) },
        "voiceOn": ctl.store.settings().get("voice").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// A transparent, click-through, full-screen scene of the cat scolding or slapping.
fn show_scold(app: &AppHandle, ctl: &mut Ctl, kind: &str, level: u32, app_name: &str) -> bool {
    if ctl.scold_active {
        return false;
    }
    ctl.scold_active = true;
    let line = persona::scold_line(
        &settings_str(ctl, "persona"),
        lang(ctl),
        kind,
        level,
        app_name,
        fastrand::f64,
    );
    let mut payload = persona_payload(ctl, &line);
    payload["kind"] = json!(kind);
    payload["level"] = json!(level);
    on_main(app, move |app| {
        let handle = app.clone();
        let built = WebviewWindowBuilder::new(app, "scold", WebviewUrl::App("scold.html".into()))
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .resizable(false)
            .focused(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .initialization_script(shim(true, ""))
            .on_page_load(move |wv, p| {
                if matches!(p.event(), PageLoadEvent::Finished) {
                    let _ = wv.show();
                    let _ = handle.emit_to("scold", "scold", payload.clone());
                }
            })
            .build();
        match built {
            Ok(w) => {
                if let Ok(Some(m)) = app.primary_monitor() {
                    let _ = w.set_position(PhysicalPosition::new(m.position().x, m.position().y));
                    let _ = w.set_size(PhysicalSize::new(m.size().width, m.size().height));
                }
                let _ = w.set_ignore_cursor_events(true);
            }
            Err(_) => with(app, |c| c.scold_active = false),
        }
    });
    after(app, scold_ms(level), |app| {
        on_main(app, |app| {
            if let Some(w) = app.get_webview_window("scold") {
                let _ = w.destroy();
            }
        });
        with(app, |c| c.scold_active = false);
    });
    true
}

/// One more offence in this session: the cat reacts according to how many there have been.
fn offend(app: &AppHandle, ctl: &mut Ctl, kind: &str, app_name: &str) {
    let Some(s) = &mut ctl.session else { return };
    s.offences += 1;
    let level = s.offences.min(3);
    show_scold(app, ctl, kind, level, app_name);
}

const GATE_REASONS: [&str; 5] = ["bored", "habit", "notify", "curious", "need"];

fn destroy_gate_window(app: &AppHandle) {
    on_main(app, |app| {
        if let Some(w) = app.get_webview_window("gate") {
            let _ = w.destroy();
        }
    });
}

/// The pause gate: when a forbidden app or tab shows up, a small card appears for a few seconds so the
/// user can stop, breathe and say why they reached for it. It never lets the app stay open: the only
/// way out is to close it (or to use the emergency button).
fn show_gate(app: &AppHandle, ctl: &mut Ctl, kind: &str, name: &str, pid: u32, title: &str) {
    let grace = if kind == "tab" {
        TAB_GRACE_MS
    } else {
        GRACE_MS
    };
    ctl.distractions
        .push(json!({ "name": name, "kind": kind, "reason": null }));
    if ctl.gate_active {
        let key = if kind == "tab" {
            "toast_tab"
        } else {
            "toast_app"
        };
        let msg = tx(
            ctl,
            key,
            &[
                ("name", name.to_string()),
                ("sec", (grace / 1000).to_string()),
            ],
        );
        toast(app, &msg, "warn");
        return;
    }
    ctl.gate_active = true;
    let is_tab = kind == "tab";
    let payload =
        json!({ "kind": kind, "name": name, "pid": pid, "title": title, "graceMs": grace });
    on_main(app, move |app| {
        let handle = app.clone();
        let built = WebviewWindowBuilder::new(app, "gate", WebviewUrl::App("gate.html".into()))
            .decorations(false)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            // the tab gate appears without taking the focus, so the browser stays in front and Ctrl+W can reach it
            .visible(is_tab)
            .focused(!is_tab)
            .center()
            .inner_size(540.0, 590.0)
            .background_color(Color(28, 23, 18, 255))
            .initialization_script(shim(true, ""))
            .on_page_load(move |wv, p| {
                if matches!(p.event(), PageLoadEvent::Finished) {
                    if !is_tab {
                        let _ = wv.show();
                        let _ = wv.set_focus();
                    }
                    let _ = handle.emit_to("gate", "gate", payload.clone());
                }
            })
            .build();
        if built.is_err() {
            with(app, |c| c.gate_active = false);
        }
    });
    let (tab_pid, tab_title) = (pid, title.to_string());
    after(app, grace + 600, move |app| {
        if is_tab {
            // time is up and nobody pressed "close": hand the focus back to the browser while we still own it,
            // so that closing the tab does not depend on where Windows puts the focus after the gate is gone
            let pending = with(app, |c| {
                c.tab_pending
                    .as_mut()
                    .filter(|t| t.pid == tab_pid && t.title == tab_title && !t.closed)
                    .map(|t| t.closed = true)
                    .is_some()
            });
            winapi::focus_process(tab_pid);
            destroy_gate_window(app);
            with(app, |c| c.gate_active = false);
            if pending {
                std::thread::sleep(Duration::from_millis(350));
                let r = winapi::close_active_tab(tab_pid, &tab_title);
                log(&format!("gate timeout close tab pid={tab_pid} -> {r}"));
            }
        } else {
            destroy_gate_window(app);
            with(app, |c| c.gate_active = false);
        }
    });
}

#[derive(Clone, Copy)]
enum Pool {
    Start,
    Cheer,
    Break,
}

fn cheer(app: &AppHandle, ctl: &Ctl, pool: Pool) {
    let p = persona::persona_of("", lang(ctl));
    let list = match pool {
        Pool::Start => p.start,
        Pool::Cheer => p.cheer,
        Pool::Break => p.break_,
    };
    let target = ["mini", "overlay-primary"]
        .into_iter()
        .find(|l| app.get_webview_window(l).is_some());
    if let Some(label) = target {
        let _ = app.emit_to(
            label,
            "cheer",
            persona_payload(ctl, persona::pick(list, fastrand::f64)),
        );
    }
}

fn rebuild_tray(app: &AppHandle, locked: bool) {
    on_main(app, move |app| {
        let Some(tray) = app.tray_by_id("main") else {
            return;
        };
        let l = lang_of(app);
        let open = MenuItem::with_id(
            app,
            "open",
            i18n::t(l, "tray_open", &[]),
            true,
            None::<&str>,
        );
        let quit = MenuItem::with_id(
            app,
            "quit",
            i18n::t(l, if locked { "tray_locked" } else { "tray_quit" }, &[]),
            !locked,
            None::<&str>,
        );
        let sep = PredefinedMenuItem::separator(app);
        if let (Ok(open), Ok(quit), Ok(sep)) = (open, quit, sep)
            && let Ok(menu) = Menu::with_items(app, &[&open, &sep, &quit])
        {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

// ───────────────────────── the session ─────────────────────────

fn sync_visuals(app: &AppHandle, ctl: &mut Ctl, st: &State) {
    let Some(s) = &ctl.session else { return };
    let (mode, break_min) = (
        s_of(&s.profile, "mode").to_string(),
        s.profile
            .get("breakMin")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    );
    let key = format!("{mode}:{}:{}", st.phase.as_str(), u8::from(st.paused));
    if ctl.last_key.as_deref() == Some(&key) {
        return;
    }
    let first = ctl.last_phase.is_none();
    let phase_changed = !first && ctl.last_phase != Some(st.phase);
    ctl.last_key = Some(key);
    ctl.last_phase = Some(st.phase);
    if mode == "fullscreen" && st.phase == PhaseKind::Work && !st.paused {
        hide_mini(app);
        show_overlay(app);
    } else {
        hide_overlay(app);
        show_mini(app);
    }
    if !(st.phase == PhaseKind::Work && mode == "apps") {
        ctl.pending.clear();
    }
    let (phase, round, rounds) = (st.phase, st.round, st.rounds);
    if first {
        after(app, 2500, |app| with(app, |c| cheer(app, c, Pool::Start)));
    } else if phase_changed && !st.paused {
        if phase == PhaseKind::Break {
            notify(
                app,
                &tx(ctl, "n_break_title", &[]),
                &tx(ctl, "n_break_body", &[("min", break_min.to_string())]),
            );
        } else {
            notify(
                app,
                &tx(ctl, "n_back_title", &[]),
                &tx(
                    ctl,
                    "n_back_body",
                    &[("round", round.to_string()), ("rounds", rounds.to_string())],
                ),
            );
        }
        after(app, 2500, move |app| {
            with(app, |c| {
                cheer(
                    app,
                    c,
                    if phase == PhaseKind::Break {
                        Pool::Break
                    } else {
                        Pool::Cheer
                    },
                )
            });
        });
    }
}

fn tick_locked(app: &AppHandle, ctl: &mut Ctl) {
    let Some(s) = &mut ctl.session else { return };
    if s.is_paused() && s.pause_left_ms() <= 0.0 {
        resume_session(app, ctl, true);
    }
    let Some(s) = &mut ctl.session else { return };
    let st = s.tick();
    if st.done {
        finish(app, ctl, true, None);
        return;
    }
    sync_visuals(app, ctl, &st);
    ctl.tick_count += 1;
    if ctl.tick_count.is_multiple_of(5) {
        persist(ctl);
    }
    let every = ctl
        .store
        .settings()
        .get("cheerEverySec")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if every > 0 && st.phase == PhaseKind::Work && ctl.tick_count.is_multiple_of(every) {
        cheer(app, ctl, Pool::Cheer);
    }
    let _ = app.emit("session", public_session(ctl));
}

fn resume_session(app: &AppHandle, ctl: &mut Ctl, auto: bool) -> bool {
    let Some(s) = &mut ctl.session else {
        return false;
    };
    if !s.resume() {
        return false;
    }
    ctl.last_key = None; // redraw the lock screen / mini window
    spawn_apply_hosts(app); // block the sites again (Windows asks permission)
    let msg = tx(
        ctl,
        if auto {
            "toast_resume_auto"
        } else {
            "toast_resume"
        },
        &[],
    );
    toast(app, &msg, "ok");
    persist(ctl);
    tick_locked(app, ctl);
    true
}

enum Act {
    Soft(u32),
    Force(u32),
    CloseTab(u32, String),
}

fn run_actions(app: &AppHandle, actions: Vec<Act>) {
    for a in actions {
        match a {
            Act::Soft(pid) => {
                winapi::close_process(pid, false);
            }
            Act::Force(pid) => {
                winapi::close_process(pid, true);
            }
            Act::CloseTab(pid, title) => {
                let _ = app;
                std::thread::spawn(move || {
                    let r = winapi::close_active_tab(pid, &title);
                    log(&format!("close tab pid={pid} title={title:?} -> {r}"));
                });
            }
        }
    }
}

fn windows_json(list: &[winapi::WinInfo]) -> Value {
    Value::Array(
        list.iter()
            .map(|w| json!({ "pid": w.pid, "name": w.name, "title": w.title, "path": w.path }))
            .collect(),
    )
}

fn rules_cfg(p: &Value) -> Value {
    json!({
        "appMode": s_of(p, "appMode"),
        "allowed": p.get("allowedApps").cloned().unwrap_or_else(|| json!([])),
        "blocked": p.get("blockedApps").cloned().unwrap_or_else(|| json!([])),
        "selfPids": [std::process::id()],
        "systemRoot": std::env::var("SystemRoot").unwrap_or_default(),
    })
}

fn on_windows(app: &AppHandle, ctl: &mut Ctl, list: &[winapi::WinInfo]) -> Vec<Act> {
    let Some(s) = &mut ctl.session else {
        return Vec::new();
    };
    let st = s.state();
    let mode = s_of(&s.profile, "mode").to_string();
    if mode != "apps" || st.phase != PhaseKind::Work || st.done || st.paused {
        ctl.pending.clear();
        return Vec::new();
    }
    let viol = rules::evaluate(windows_json(list), rules_cfg(&s.profile))
        .ok()
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let force_kill = s.settings.force_kill;
    let seen: Vec<u32> = viol
        .iter()
        .filter_map(|v| v["pid"].as_u64())
        .map(|p| p as u32)
        .collect();
    ctl.pending.retain(|pid, _| seen.contains(pid));
    let now = Instant::now();
    let mut actions = Vec::new();
    let mut new_offences = Vec::new();
    for v in &viol {
        let Some(pid) = v["pid"].as_u64().map(|p| p as u32) else {
            continue;
        };
        let name = v["name"].as_str().unwrap_or("").to_string();
        let fresh = !ctl.pending.contains_key(&pid);
        let e = ctl.pending.entry(pid).or_insert(PendingApp {
            first: now,
            closed: false,
            forced: false,
            closed_at: now,
        });
        if fresh {
            let path = v["path"].as_str().unwrap_or("").to_string();
            new_offences.push((pid, name, path));
        }
        if !e.closed && now.duration_since(e.first) >= Duration::from_millis(GRACE_MS) {
            e.closed = true;
            e.closed_at = now;
            actions.push(Act::Soft(pid));
        } else if e.closed
            && !e.forced
            && force_kill
            && now.duration_since(e.closed_at) >= Duration::from_millis(FORCE_MS)
        {
            e.forced = true;
            actions.push(Act::Force(pid));
        }
    }
    for (pid, name, path) in new_offences {
        if let Some(s) = &mut ctl.session {
            s.violations += 1;
        }
        let label = winapi::display_name(&path, &name);
        offend(app, ctl, "app", &label);
        show_gate(app, ctl, "app", &label, pid, "");
    }
    actions
}

fn on_foreground(app: &AppHandle, ctl: &mut Ctl, fg: Option<winapi::WinInfo>) -> Vec<Act> {
    let (Some(s), Some(fg)) = (&ctl.session, fg) else {
        ctl.tab_pending = None;
        return Vec::new();
    };
    let st = s.state();
    let p = &s.profile;
    let close_tabs = p.get("closeTabs").and_then(Value::as_bool).unwrap_or(false);
    if s_of(p, "mode") != "apps"
        || st.phase != PhaseKind::Work
        || st.done
        || st.paused
        || !close_tabs
    {
        ctl.tab_pending = None;
        return Vec::new();
    }
    if fg.pid == std::process::id() {
        // our own window (the pause gate) is in front: keep the pending tab and wait for the browser to return
        return Vec::new();
    }
    let norm = rules::norm_name(json!(fg.name))
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    if !tabs::is_browser(&norm) {
        ctl.tab_pending = None;
        return Vec::new();
    }
    if fg.title.trim().is_empty() {
        // browsers briefly show an untitled window (e.g. while another window opens): not a verdict either way
        return Vec::new();
    }
    let strings = |k: &str| -> Vec<String> {
        p.get(k)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let hit = tabs::match_banned(
        fg.title.clone(),
        strings("blockedTitles"),
        strings("blockedSites"),
    )
    .ok()
    .and_then(|v| v.as_str().map(str::to_string));
    let Some(hit) = hit else {
        ctl.tab_pending = None;
        return Vec::new();
    };
    let now = Instant::now();
    let mut actions = Vec::new();
    match &mut ctl.tab_pending {
        Some(t) if t.title == fg.title => {
            let waited = now.duration_since(t.first);
            if !t.closed && waited >= Duration::from_millis(TAB_GRACE_MS) {
                t.closed = true;
                actions.push(Act::CloseTab(t.pid, t.title.clone()));
            } else if t.closed && waited >= Duration::from_millis(9000) {
                ctl.tab_pending = None; // could not close it (or you reopened it): count it as new
            }
        }
        _ => {
            ctl.tab_pending = Some(PendingTab {
                title: fg.title.clone(),
                pid: fg.pid,
                first: now,
                closed: false,
            });
            if let Some(s) = &mut ctl.session {
                s.violations += 1;
            }
            offend(app, ctl, "app", &hit);
            show_gate(app, ctl, "tab", &hit, fg.pid, &fg.title);
        }
    }
    actions
}

// ── hosts file (blocks websites) ──

fn set_hosts_status(app: &AppHandle, state: &str, message: &str) {
    with(app, |c| {
        c.hosts_status = json!({ "state": state, "message": message });
        push_data(app, c);
    });
}

fn hosts_blocked() -> bool {
    hosts_text::has_block(json!(winapi::read_hosts()))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn apply_block(domains: &[String]) -> bool {
    let Ok(content) = hosts_text::with_block(json!(winapi::read_hosts()), json!(domains)) else {
        return false;
    };
    let wrote = winapi::write_hosts_elevated(content.as_str().unwrap_or("")).is_ok();
    wrote && (hosts_blocked() != domains.is_empty())
}

fn spawn_apply_hosts(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let (domains, skip, sites) = with(&app, |c| {
            let sites = c.session.as_ref().map_or_else(
                || json!([]),
                |s| {
                    s.profile
                        .get("blockedSites")
                        .cloned()
                        .unwrap_or_else(|| json!([]))
                },
            );
            let domains: Vec<String> = hosts_text::expand_domains(sites.clone())
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|a| {
                    a.iter()
                        .filter_map(|d| d.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            (domains, c.skip_hosts, sites.as_array().map_or(0, Vec::len))
        });
        if domains.is_empty() || skip {
            with(&app, |c| {
                c.hosts_status = json!({ "state": "none", "message": "" })
            });
            return;
        }
        if hosts_blocked() {
            with(
                &app,
                |c| {
                    c.hosts_status =
                        json!({ "state": "active", "message": tx(c, "hosts_active", &[]) })
                },
            );
            return;
        }
        let l = lang_of(&app);
        set_hosts_status(&app, "pending", &i18n::t(l, "hosts_pending", &[]));
        let ok = apply_block(&domains);
        if ok {
            set_hosts_status(
                &app,
                "active",
                &i18n::t(l, "hosts_on", &[("n", sites.to_string())]),
            );
        } else {
            set_hosts_status(&app, "failed", &i18n::t(l, "hosts_failed", &[]));
        }
    });
}

fn spawn_release_hosts(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let skip = with(&app, |c| c.skip_hosts);
        if skip || !hosts_blocked() {
            with(&app, |c| {
                c.stale_hosts = false;
                c.hosts_status = json!({ "state": "none", "message": "" });
                push_data(&app, c);
            });
            return;
        }
        set_hosts_status(
            &app,
            "pending",
            &i18n::t(lang_of(&app), "hosts_releasing", &[]),
        );
        let ok = apply_block(&[]);
        with(&app, |c| {
            c.stale_hosts = !ok;
            c.hosts_status = if ok {
                json!({ "state": "none", "message": "" })
            } else {
                json!({ "state": "failed", "message": tx(c, "hosts_release_failed", &[]) })
            };
            push_data(&app, c);
        });
    });
}

fn engage(app: &AppHandle, ctl: &mut Ctl) {
    ctl.pending.clear();
    ctl.last_key = None;
    ctl.last_phase = None;
    ctl.tick_count = 0;
    ctl.tab_pending = None;
    ctl.distractions.clear();
    if let Some(s) = &ctl.session {
        let p = &s.profile;
        let has_tab_rules = arr_len(p, "blockedTitles") > 0 || arr_len(p, "blockedSites") > 0;
        let close_tabs = p.get("closeTabs").and_then(Value::as_bool).unwrap_or(false);
        ctl.watcher_on = s_of(p, "mode") == "apps"
            && (s_of(p, "appMode") != "off" || (close_tabs && has_tab_rules));
    }
    tick_locked(app, ctl);
    spawn_apply_hosts(app);
    push_data(app, ctl);
}

fn resolve_profile(ctl: &Ctl, req: &Value) -> Option<Value> {
    if let Some(plan) = req.get("plan").filter(|p| !p.is_null()) {
        return Some(prof::sanitize(plan, Some("plan"), ctl.dev));
    }
    let id = req.get("profileId").and_then(Value::as_str)?;
    let base = ctl
        .store
        .data
        .get("profiles")?
        .as_array()?
        .iter()
        .find(|p| s_of(p, "id") == id)?
        .clone();
    let mut merged = base.clone();
    if let Some(w) = req.get("workMin").filter(|w| !w.is_null()) {
        merged["workMin"] = w.clone();
    }
    Some(prof::sanitize(&merged, Some(id), ctl.dev))
}

fn start_session(app: &AppHandle, ctl: &mut Ctl, req: &Value) -> Value {
    if ctl.session.is_some() {
        return json!({ "ok": false, "error": tx(ctl, "err_running", &[]) });
    }
    let Some(mut profile) = resolve_profile(ctl, req) else {
        return json!({ "ok": false, "error": tx(ctl, "err_plan", &[]) });
    };
    if s_of(&profile, "mode") == "apps"
        && s_of(&profile, "appMode") == "allow"
        && arr_len(&profile, "allowedApps") == 0
    {
        return json!({ "ok": false, "error": tx(ctl, "err_allow", &[]) });
    }
    if s_of(&profile, "strictness").is_empty() {
        profile["strictness"] = json!(settings_str(ctl, "strictness"));
    }
    let strict = s_of(&profile, "strictness").to_string();
    let settings = level_cfg(ctl, &strict);
    ctl.session = Some(Session::new(
        &json!({ "profile": profile.clone(), "settings": settings }),
        None,
    ));
    if req.get("plan").is_some_and(|p| !p.is_null()) {
        let mut plan = profile.clone();
        if let Some(m) = plan.as_object_mut() {
            m.remove("id");
        }
        ctl.store.data.insert("lastPlan".into(), plan);
        ctl.store
            .settings_mut()
            .insert("strictness".into(), json!(strict));
    }
    persist(ctl);
    hide_main(app);
    engage(app, ctl);
    json!({ "ok": true })
}

fn finish(app: &AppHandle, ctl: &mut Ctl, completed: bool, emergency: Option<&str>) {
    let Some(s) = ctl.session.take() else { return };
    ctl.watcher_on = false;
    ctl.pending.clear();
    ctl.tab_pending = None;
    ctl.last_key = None;
    ctl.last_phase = None;
    hide_overlay(app);
    hide_mini(app);
    on_main(app, |app| {
        if let Some(w) = app.get_webview_window("scold") {
            let _ = w.destroy();
        }
    });
    ctl.scold_active = false;
    ctl.gate_active = false;
    destroy_gate_window(app);
    let planned = (s.work_total_ms() / 60_000.0).round();
    let mut entry = json!({
        "id": s.id, "profileName": s_of(&s.profile, "name"), "startedAt": s.started_at, "endedAt": tz::now_ms(),
        "plannedMin": planned,
        "focusedMin": if emergency.is_some() { 0.0 } else { (s.work_done_ms() / 60_000.0).round() },
        "completed": completed, "violations": s.violations, "restarts": s.restarts,
        "pauses": u8::from(s.pause_used()),
    });
    if !s_of(&s.profile, "intent").is_empty() {
        entry["intent"] = json!(s_of(&s.profile, "intent"));
    }
    if !ctl.distractions.is_empty() {
        entry["distractions"] = Value::Array(std::mem::take(&mut ctl.distractions));
    }
    if let Some(r) = emergency {
        entry["emergency"] = json!(true);
        entry["reason"] = json!(r);
    }
    let history = ctl.store.array_mut("history");
    history.push(entry);
    if history.len() > 1000 {
        history.drain(..history.len() - 1000);
    }
    ctl.store.data.insert("active".into(), Value::Null);
    let _ = ctl.store.save();
    if completed {
        notify(
            app,
            &tx(ctl, "n_done_title", &[]),
            &tx(ctl, "n_done_body", &[("min", planned.to_string())]),
        );
    }
    show_main(app);
    push_data(app, ctl);
    let (key, kind) = if emergency.is_some() {
        ("toast_emergency", "ok")
    } else if completed {
        ("toast_done", "ok")
    } else {
        ("toast_early", "warn")
    };
    toast(app, &tx(ctl, key, &[]), kind);
    spawn_release_hosts(app);
}

// ───────────────────────── IPC ─────────────────────────

fn capitalize(n: &str) -> String {
    let mut c = n.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

fn app_label(r: &winapi::RichApp, n: &str) -> String {
    let d = r.desc.trim();
    if d.is_empty() {
        capitalize(n)
    } else {
        d.to_string()
    }
}

fn icon_url(icon: &str) -> String {
    if icon.is_empty() {
        String::new()
    } else {
        format!("data:image/png;base64,{icon}")
    }
}

fn norm_of(name: &str) -> String {
    rules::norm_name(json!(name))
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn locked_err(ctl: &Ctl) -> Option<Value> {
    ctl.session
        .is_some()
        .then(|| json!({ "ok": false, "error": tx(ctl, "err_locked", &[]) }))
}

fn handle(app: &AppHandle, channel: &str, args: &[Value]) -> Value {
    let arg = |i: usize| args.get(i).cloned().unwrap_or(Value::Null);
    with(app, |ctl| match channel {
        "state:get" => full_state(ctl),
        "profile:save" => {
            if let Some(l) = locked_err(ctl) {
                return l;
            }
            let p = arg(0);
            let known = s_of(&p, "id");
            let exists = ctl
                .store
                .array_mut("profiles")
                .iter()
                .any(|x| s_of(x, "id") == known)
                && !known.is_empty();
            let clean = prof::sanitize(&p, exists.then_some(known), ctl.dev);
            let profiles = ctl.store.array_mut("profiles");
            match profiles
                .iter()
                .position(|x| s_of(x, "id") == s_of(&clean, "id"))
            {
                Some(i) => profiles[i] = clean.clone(),
                None => profiles.push(clean.clone()),
            }
            let _ = ctl.store.save();
            push_data(app, ctl);
            json!({ "ok": true, "profile": clean })
        }
        "profile:delete" => {
            if let Some(l) = locked_err(ctl) {
                return l;
            }
            let id = arg(0);
            ctl.store
                .array_mut("profiles")
                .retain(|p| p.get("id") != Some(&id));
            let _ = ctl.store.save();
            push_data(app, ctl);
            json!({ "ok": true })
        }
        "settings:save" => {
            if let Some(l) = locked_err(ctl) {
                return l;
            }
            let s = arg(0);
            let cur = ctl.store.settings().clone();
            let truthy = |v: &Value| disturbless_core::jsstr::truthy(v);
            let mut next = cur.clone();
            if let Some(x) = s
                .get("strictness")
                .and_then(Value::as_str)
                .filter(|x| prof::is_level(x))
            {
                next.insert("strictness".into(), json!(x));
            }
            if let Some(x) = s.get("autostart") {
                next.insert("autostart".into(), json!(truthy(x)));
            }
            if let Some(x) = s.get("introSeen") {
                next.insert("introSeen".into(), json!(truthy(x)));
            }
            next.insert("persona".into(), json!(persona::DEFAULT_PERSONA));
            if let Some(x) = s.get("voice") {
                next.insert("voice".into(), json!(truthy(x)));
            }
            if let Some(x) = s.get("lang").and_then(Value::as_str) {
                next.insert("lang".into(), json!(persona::norm_lang(x)));
            }
            if let Some(n) = s
                .get("fontScale")
                .map(prof::to_number)
                .filter(|n| [1.0, 1.15, 1.3].contains(n))
            {
                next.insert("fontScale".into(), json!(n));
            }
            if let Some(n) = s
                .get("cheerEverySec")
                .map(prof::to_number)
                .filter(|n| [0.0, 60.0, 120.0, 240.0, 600.0].contains(n))
            {
                next.insert("cheerEverySec".into(), json!(n as u64));
            }
            let autostart = next
                .get("autostart")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            *ctl.store.settings_mut() = next;
            let _ = winapi::set_autostart(autostart);
            let _ = ctl.store.save();
            push_data(app, ctl);
            json!({ "ok": true })
        }
        "session:start" => start_session(app, ctl, &arg(0)),
        "exit:request" => {
            let Some(s) = &mut ctl.session else {
                return Value::Null;
            };
            let first = !s.exit_status().requested;
            let r = s.request_exit().to_json();
            if first {
                offend(app, ctl, "exit", "");
            }
            r
        }
        "exit:cancel" => {
            if let Some(s) = &mut ctl.session {
                s.cancel_exit();
            }
            json!(true)
        }
        "exit:phrase" => ctl
            .session
            .as_ref()
            .and_then(|s| s.exit_phrase())
            .map_or(Value::Null, |p| json!(p)),
        "exit:confirm" => {
            let Some(s) = &ctl.session else {
                return json!({ "ok": false, "error": "no-session" });
            };
            let typed = arg(0);
            match s.confirm_exit(typed.as_str().unwrap_or("")) {
                Ok(()) => {
                    finish(app, ctl, false, None);
                    json!({ "ok": true })
                }
                Err(e) => json!({ "ok": false, "error": e.code() }),
            }
        }
        "scold:test" => {
            let level = prof::to_number(&arg(0));
            let level = if level.is_finite() { level as u32 } else { 1 }.clamp(1, 3);
            json!(show_scold(app, ctl, "exit", level, ""))
        }
        "emergency:pause" => {
            let Some(s) = &mut ctl.session else {
                return json!({ "ok": false, "error": tx(ctl, "err_nosession", &[]) });
            };
            if !s.can_pause() {
                return json!({ "ok": false, "error": tx(ctl, "err_pause_used", &[]) });
            }
            s.pause();
            let minutes = (s.settings.pause_ms / 60_000.0).round();
            ctl.pending.clear();
            ctl.tab_pending = None;
            persist(ctl);
            let msg = tx(ctl, "toast_pause", &[("min", minutes.to_string())]);
            toast(app, &msg, "ok");
            spawn_release_hosts(app); // reopen the sites during the pause (Windows asks permission)
            tick_locked(app, ctl);
            json!({ "ok": true })
        }
        "emergency:resume" => json!({ "ok": resume_session(app, ctl, false) }),
        "emergency:stop" => {
            if ctl.session.is_none() {
                return json!({ "ok": false, "error": tx(ctl, "err_nosession", &[]) });
            }
            let reason = arg(0);
            let Some(r) = reason.as_str().filter(|r| EMERGENCY_REASONS.contains(r)) else {
                return json!({ "ok": false, "error": tx(ctl, "err_reason", &[]) });
            };
            finish(app, ctl, false, Some(r));
            json!({ "ok": true })
        }
        "hosts:clean" => {
            if ctl.session.is_none() {
                spawn_release_hosts(app);
            }
            json!({ "ok": true })
        }
        "gate:close" => {
            let a = arg(0);
            let pid = a.get("pid").and_then(Value::as_u64).unwrap_or(0) as u32;
            let mut acts = Vec::new();
            // only ever act on what the watcher itself flagged: the page cannot name an arbitrary process
            if s_of(&a, "kind") == "tab" {
                if let Some(t) = &mut ctl.tab_pending
                    && t.pid == pid
                {
                    // Ctrl+W only works while the browser is in front. The gate was just clicked, so Windows
                    // lets us hand the focus straight back: wait for the gate to go, refocus, then close.
                    t.closed = true;
                    let title = t.title.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(350));
                        winapi::focus_process(pid);
                        std::thread::sleep(Duration::from_millis(250));
                        let r = winapi::close_active_tab(pid, &title);
                        log(&format!("gate close tab pid={pid} -> {r}"));
                    });
                }
            } else if let Some(e) = ctl.pending.get_mut(&pid) {
                if !e.closed {
                    e.closed = true;
                    e.closed_at = Instant::now();
                }
                acts.push(Act::Soft(pid));
            }
            ctl.gate_active = false;
            destroy_gate_window(app);
            run_actions(app, acts);
            json!({ "ok": true })
        }
        "gate:reason" => {
            let a = arg(0);
            let name = s_of(&a, "name").to_string();
            if let Some(r) = a
                .get("reason")
                .and_then(Value::as_str)
                .filter(|r| GATE_REASONS.contains(r))
                && let Some(d) = ctl
                    .distractions
                    .iter_mut()
                    .rev()
                    .find(|d| s_of(d, "name") == name && d["reason"].is_null())
            {
                d["reason"] = json!(r);
            }
            json!({ "ok": true })
        }
        "gate:done" => {
            ctl.gate_active = false;
            destroy_gate_window(app);
            json!({ "ok": true })
        }
        "window:openMain" => {
            show_main(app);
            json!(true)
        }
        _ => Value::Null,
    })
}

#[tauri::command]
async fn ipc(app: AppHandle, channel: String, args: Vec<Value>) -> Result<Value, String> {
    // Listing apps reads icons and file info: do it off the UI thread, without holding the lock.
    match channel.as_str() {
        "apps:list" => {
            let rich = tauri::async_runtime::spawn_blocking(winapi::list_apps_rich)
                .await
                .map_err(|e| e.to_string())?;
            let mut out: Vec<(String, Value)> = Vec::new();
            for r in rich {
                let n = norm_of(&r.info.name);
                if r.info.pid == std::process::id() || rules::is_system_safe(&n) {
                    continue;
                }
                let label = app_label(&r, &n);
                out.push((label.to_lowercase(), json!({ "name": n, "label": label, "title": r.info.title, "icon": icon_url(&r.icon) })));
            }
            out.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(Value::Array(out.into_iter().map(|(_, v)| v).collect()))
        }
        "session:preview" => {
            let req = args.first().cloned().unwrap_or(Value::Null);
            let Some(profile) = with(&app, |c| resolve_profile(c, &req)) else {
                return Ok(json!({ "ok": false, "error": with(&app, |c| tx(c, "err_plan", &[])) }));
            };
            let mut to_close = Vec::new();
            if s_of(&profile, "mode") == "apps" && s_of(&profile, "appMode") != "off" {
                let rich = tauri::async_runtime::spawn_blocking(winapi::list_apps_rich)
                    .await
                    .map_err(|e| e.to_string())?;
                let infos: Vec<winapi::WinInfo> = rich.iter().map(|r| r.info.clone()).collect();
                let viol = rules::evaluate(windows_json(&infos), rules_cfg(&profile))
                    .ok()
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                for v in viol {
                    let pid = v["pid"].as_u64().unwrap_or(0) as u32;
                    if let Some(r) = rich.iter().find(|r| r.info.pid == pid) {
                        let n = norm_of(&r.info.name);
                        to_close.push(json!({ "name": n, "label": app_label(r, &n), "icon": icon_url(&r.icon) }));
                    }
                }
            }
            let has_sites = hosts_text::expand_domains(
                profile
                    .get("blockedSites")
                    .cloned()
                    .unwrap_or_else(|| json!([])),
            )
            .ok()
            .and_then(|v| v.as_array().map(|a| !a.is_empty()))
            .unwrap_or(false);
            Ok(json!({ "ok": true, "toClose": to_close, "sites": has_sites, "profile": profile }))
        }
        _ => {
            let a = app.clone();
            tauri::async_runtime::spawn_blocking(move || handle(&a, &channel, &args))
                .await
                .map_err(|e| e.to_string())
        }
    }
}

// ───────────────────────── startup ─────────────────────────

fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("DL_DATA") {
        return PathBuf::from(d); // for tests: never touches real data
    }
    // Same folder the Electron version used, so existing profiles and history carry over.
    std::env::var_os("APPDATA")
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join("disturbless")
}

fn spawn_loops(app: &AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            with(&a, |c| tick_locked(&a, c));
        }
    });
    let a = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            if !with(&a, |c| c.watcher_on) {
                continue;
            }
            let list = winapi::list_windows();
            let fg = winapi::foreground();
            let mut actions = with(&a, |c| on_windows(&a, c, &list));
            actions.extend(with(&a, |c| on_foreground(&a, c, fg)));
            run_actions(&a, actions);
        }
    });
    let a = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if !overlay_labels(&a).is_empty() {
                keep_overlay_on_top(&a);
            }
        }
    });
}

fn build_tray(app: &AppHandle) {
    let mut b = TrayIconBuilder::with_id("main")
        .tooltip("Disturbless")
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    let tray = b
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "quit" => {
                let can = with(app, |c| {
                    let ok = c.session.is_none();
                    if ok {
                        c.quitting = true;
                    }
                    ok
                });
                if can {
                    app.exit(0);
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app);
    let _ = tray;
    rebuild_tray(app, false);
}

fn restore_or_start(app: &AppHandle, hidden: bool) {
    let restored = with(app, |ctl| {
        let Some(a) = ctl
            .store
            .data
            .get("active")
            .filter(|a| !a.is_null())
            .cloned()
        else {
            if !ctl.skip_hosts && hosts_blocked() {
                ctl.stale_hosts = true; // leftover website block from an earlier session
            }
            return false;
        };
        // An interrupted session (app killed, power cut): the time away still counts, unless it was paused.
        let saved = a
            .get("savedAt")
            .and_then(Value::as_f64)
            .unwrap_or_else(tz::now_ms);
        let downtime = (tz::now_ms() - saved).max(0.0);
        let paused = a.get("paused").and_then(Value::as_bool).unwrap_or(false);
        let mut snap = a.clone();
        snap["elapsedMs"] = json!(
            a.get("elapsedMs").and_then(Value::as_f64).unwrap_or(0.0)
                + if paused { 0.0 } else { downtime }
        );
        snap["restarts"] = json!(a.get("restarts").and_then(Value::as_u64).unwrap_or(0) + 1);
        let mut s = Session::new(&snap, None);
        if s.tick().done {
            ctl.session = Some(s);
            finish(app, ctl, true, None);
        } else {
            ctl.session = Some(s);
            engage(app, ctl);
        }
        true
    });
    if !hidden || !restored {
        show_main(app);
    }
}

fn main() {
    let dir = data_dir();
    let store = Store::open(dir.join("disturbless.json"));
    let ctl = Ctl {
        store,
        session: None,
        dev: std::env::var("DL_DEV").is_ok_and(|v| v == "1"),
        skip_hosts: std::env::var("DL_NOHOSTS").is_ok_and(|v| v == "1"),
        hosts_status: json!({ "state": "none", "message": "" }),
        stale_hosts: false,
        last_key: None,
        last_phase: None,
        tick_count: 0,
        pending: HashMap::new(),
        tab_pending: None,
        quitting: false,
        scold_active: false,
        watcher_on: false,
        gate_active: false,
        distractions: Vec::new(),
    };
    let hidden = std::env::args().any(|a| a == "--hidden");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app)
        }))
        .plugin(tauri_plugin_notification::init())
        .manage::<Shared>(Arc::new(Mutex::new(ctl)))
        .invoke_handler(tauri::generate_handler![ipc])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label().to_string();
                let app = window.app_handle();
                if label == "main" {
                    if !with(app, |c| c.quitting) {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                } else if with(app, |c| c.session.is_some()) {
                    api.prevent_close();
                }
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            build_tray(&handle);
            spawn_loops(&handle);
            restore_or_start(&handle, hidden);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Disturbless could not start");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            // Closing the last window must not quit: the app lives in the tray. Only an explicit quit exits.
            if code.is_none() || with(app, |c| c.session.is_some()) {
                api.prevent_exit();
            }
        }
    });
}

/// Debug trace to the file named by `DL_LOG` (off unless set; used when testing).
fn log(line: &str) {
    use std::io::Write;
    if let Some(path) = std::env::var_os("DL_LOG")
        && let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    {
        let _ = writeln!(f, "{} {line}", tz::now_ms());
    }
}
