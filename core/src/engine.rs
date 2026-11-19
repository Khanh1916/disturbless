//! State machine of one focus session. Pure logic, no UI.
//!
//! Elapsed time runs on a monotonic clock so changing the system time mid-session cannot
//! shorten it. Time values are `f64` milliseconds like the JS original.

use std::sync::OnceLock;
use std::time::Instant;

use serde_json::{Map, Value, json};
use unicode_normalization::UnicodeNormalization;

use crate::jsstr;
use crate::phrases::pick_phrase;
use crate::tz;

/// Monotonic milliseconds since the first call in this process.
pub fn mono_now() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as f64
}

pub type Clock = Box<dyn Fn() -> f64 + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PhaseKind {
    Work,
    Break,
}

impl PhaseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Break => "break",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Phase {
    pub kind: PhaseKind,
    pub ms: f64,
    pub round: u32,
}

/// `value || default` for numeric JSON fields.
fn num(v: Option<&Value>, default: f64) -> f64 {
    v.and_then(Value::as_f64)
        .filter(|n| *n != 0.0)
        .unwrap_or(default)
}

pub fn build_phases(profile: &Value) -> Vec<Phase> {
    let rounds = num(profile.get("rounds"), 1.0).floor().max(1.0) as u32;
    let work_ms = num(profile.get("workMin"), f64::NAN).max(1.0) * 60_000.0;
    let break_ms = num(profile.get("breakMin"), 0.0).max(0.0) * 60_000.0;
    let mut phases = Vec::new();
    for i in 0..rounds {
        phases.push(Phase {
            kind: PhaseKind::Work,
            ms: work_ms,
            round: i + 1,
        });
        if break_ms > 0.0 && i < rounds - 1 {
            phases.push(Phase {
                kind: PhaseKind::Break,
                ms: break_ms,
                round: i + 1,
            });
        }
    }
    phases
}

pub struct Located {
    pub index: usize,
    pub phase: Option<Phase>,
    pub into_ms: f64,
}

pub fn locate(phases: &[Phase], elapsed_ms: f64) -> Located {
    let mut acc = 0.0;
    for (i, p) in phases.iter().enumerate() {
        if elapsed_ms < acc + p.ms {
            return Located {
                index: i,
                phase: Some(*p),
                into_ms: elapsed_ms - acc,
            };
        }
        acc += p.ms;
    }
    Located {
        index: phases.len(),
        phase: None,
        into_ms: 0.0,
    }
}

/// Milliseconds of actual work (breaks excluded) within `elapsed_ms`.
pub fn work_elapsed_ms(phases: &[Phase], elapsed_ms: f64) -> f64 {
    let (mut acc, mut work) = (0.0, 0.0);
    for p in phases {
        let used = (elapsed_ms - acc).min(p.ms).max(0.0);
        if p.kind == PhaseKind::Work {
            work += used;
        }
        acc += p.ms;
    }
    work
}

/// NFC, collapse any whitespace run to one space, trim: what "typed the same text" means.
pub fn normalize(s: &str) -> String {
    let nfc: String = s.nfc().collect();
    let units: Vec<u16> = nfc.encode_utf16().collect();
    let mut out: Vec<u16> = Vec::with_capacity(units.len());
    let mut in_space = false;
    for &u in &units {
        if jsstr::is_space(u) {
            if !in_space {
                out.push(0x20);
            }
            in_space = true;
        } else {
            out.push(u);
            in_space = false;
        }
    }
    String::from_utf16_lossy(jsstr::trim(&out))
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub cooldown_sec: f64,
    pub force_kill: bool,
    pub pause_ms: f64,
    pub sentences: Option<usize>,
    /// Language of the phrase the user must retype: "vi" or "en".
    pub lang: String,
}

impl Settings {
    fn from_json(v: Option<&Value>) -> Self {
        let get = |k: &str| v.and_then(|s| s.get(k));
        Self {
            cooldown_sec: get("cooldownSec").and_then(Value::as_f64).unwrap_or(120.0),
            force_kill: get("forceKill").and_then(Value::as_bool).unwrap_or(true),
            pause_ms: get("pauseMs")
                .and_then(Value::as_f64)
                .unwrap_or(10.0 * 60_000.0),
            sentences: get("sentences").and_then(Value::as_u64).map(|n| n as usize),
            lang: crate::persona::norm_lang(get("lang").and_then(Value::as_str).unwrap_or("vi"))
                .to_string(),
        }
    }

    fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("cooldownSec".into(), json!(self.cooldown_sec));
        m.insert("forceKill".into(), json!(self.force_kill));
        m.insert("pauseMs".into(), json!(self.pause_ms));
        m.insert("lang".into(), json!(self.lang));
        if let Some(n) = self.sentences {
            m.insert("sentences".into(), json!(n));
        }
        Value::Object(m)
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub done: bool,
    pub paused: bool,
    pub pause_used: bool,
    pub pause_left_ms: f64,
    pub phase_index: usize,
    pub phase: PhaseKind,
    pub round: u32,
    pub rounds: u32,
    pub phase_remaining_ms: f64,
    pub phase_total_ms: f64,
    pub total_remaining_ms: f64,
    pub total_ms: f64,
    pub elapsed_ms: f64,
}

impl State {
    pub fn to_json(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("done".into(), json!(self.done));
        m.insert("paused".into(), json!(self.paused));
        m.insert("pauseUsed".into(), json!(self.pause_used));
        m.insert("pauseLeftMs".into(), json!(self.pause_left_ms));
        m.insert("phaseIndex".into(), json!(self.phase_index));
        m.insert("phase".into(), json!(self.phase.as_str()));
        m.insert("round".into(), json!(self.round));
        m.insert("rounds".into(), json!(self.rounds));
        m.insert("phaseRemainingMs".into(), json!(self.phase_remaining_ms));
        m.insert("phaseTotalMs".into(), json!(self.phase_total_ms));
        m.insert("totalRemainingMs".into(), json!(self.total_remaining_ms));
        m.insert("totalMs".into(), json!(self.total_ms));
        m.insert("elapsedMs".into(), json!(self.elapsed_ms));
        m
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExitStatus {
    pub requested: bool,
    pub ready: bool,
    pub cooldown_left_ms: f64,
}

impl ExitStatus {
    pub fn to_json(&self) -> Value {
        json!({ "requested": self.requested, "ready": self.ready, "cooldownLeftMs": self.cooldown_left_ms })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExitError {
    NotRequested,
    Cooldown,
    Mismatch,
}

impl ExitError {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::Cooldown => "cooldown",
            Self::Mismatch => "mismatch",
        }
    }
}

pub struct Session {
    mono: Clock,
    pub id: String,
    pub profile: Value,
    pub settings: Settings,
    pub started_at: f64,
    pub elapsed_ms: f64,
    pub violations: u32,
    pub restarts: u32,
    /// Attempts to quit / open forbidden apps: decides how hard the cat slaps.
    pub offences: u32,
    pub phrase: String,
    phases: Vec<Phase>,
    total_ms: f64,
    last: f64,
    exit_at: Option<f64>,
    paused_used: bool,
    paused_at: Option<f64>,
}

impl Session {
    /// `snap` has the shape of `to_json()` (and of the first-start request: `profile` + `settings`).
    pub fn new(snap: &Value, mono: Option<Clock>) -> Self {
        let mono: Clock = mono.unwrap_or_else(|| Box::new(mono_now));
        let settings = Settings::from_json(snap.get("settings"));
        let profile = snap.get("profile").cloned().unwrap_or(Value::Null);
        let phases = build_phases(&profile);
        let total_ms = phases.iter().map(|p| p.ms).sum();
        let phrase = snap
            .get("phrase")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                pick_phrase(
                    fastrand::f64,
                    settings.sentences.unwrap_or(2),
                    &settings.lang,
                )
            });
        let count = |k: &str| snap.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
        let last = mono();
        Self {
            id: snap
                .get("id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map_or_else(|| uuid::Uuid::new_v4().to_string(), str::to_string),
            profile,
            settings,
            started_at: num(snap.get("startedAt"), tz::now_ms()),
            elapsed_ms: snap.get("elapsedMs").and_then(Value::as_f64).unwrap_or(0.0),
            violations: count("violations"),
            restarts: count("restarts"),
            offences: count("offences"),
            phrase,
            phases,
            total_ms,
            last,
            exit_at: None,
            paused_used: snap
                .get("pausedUsed")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            paused_at: None,
            mono,
        }
    }

    pub fn total_ms(&self) -> f64 {
        self.total_ms
    }

    pub fn tick(&mut self) -> State {
        let now = (self.mono)();
        let delta = now - self.last;
        self.last = now;
        if self.paused_at.is_some() {
            return self.state(); // paused: the clock stands still
        }
        if delta > 0.0 {
            self.elapsed_ms = self.total_ms.min(self.elapsed_ms + delta);
        }
        self.state()
    }

    // ── emergency pause ──
    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }
    pub fn can_pause(&self) -> bool {
        !self.paused_used && self.paused_at.is_none()
    }
    pub fn pause_used(&self) -> bool {
        self.paused_used
    }
    pub fn pause(&mut self) -> bool {
        if !self.can_pause() {
            return false;
        }
        self.tick();
        self.paused_at = Some((self.mono)());
        self.paused_used = true;
        true
    }
    pub fn resume(&mut self) -> bool {
        if self.paused_at.is_none() {
            return false;
        }
        self.paused_at = None;
        self.last = (self.mono)();
        true
    }
    pub fn pause_left_ms(&self) -> f64 {
        self.paused_at.map_or(0.0, |at| {
            (self.settings.pause_ms - ((self.mono)() - at)).max(0.0)
        })
    }

    pub fn state(&self) -> State {
        let loc = locate(&self.phases, self.elapsed_ms);
        let rounds = self
            .phases
            .iter()
            .filter(|p| p.kind == PhaseKind::Work)
            .count() as u32;
        State {
            done: self.elapsed_ms >= self.total_ms,
            paused: self.paused_at.is_some(),
            pause_used: self.paused_used,
            pause_left_ms: self.pause_left_ms(),
            phase_index: loc.index,
            phase: loc.phase.map_or(PhaseKind::Work, |p| p.kind),
            round: loc.phase.map_or(rounds, |p| p.round),
            rounds,
            phase_remaining_ms: loc.phase.map_or(0.0, |p| p.ms - loc.into_ms),
            phase_total_ms: loc.phase.map_or(0.0, |p| p.ms),
            total_remaining_ms: (self.total_ms - self.elapsed_ms).max(0.0),
            total_ms: self.total_ms,
            elapsed_ms: self.elapsed_ms,
        }
    }

    pub fn work_done_ms(&self) -> f64 {
        work_elapsed_ms(&self.phases, self.elapsed_ms)
    }
    pub fn work_total_ms(&self) -> f64 {
        work_elapsed_ms(&self.phases, self.total_ms)
    }

    pub fn request_exit(&mut self) -> ExitStatus {
        if self.exit_at.is_none() {
            self.exit_at = Some((self.mono)());
        }
        self.exit_status()
    }
    pub fn cancel_exit(&mut self) {
        self.exit_at = None;
    }

    pub fn exit_status(&self) -> ExitStatus {
        let Some(at) = self.exit_at else {
            return ExitStatus {
                requested: false,
                ready: false,
                cooldown_left_ms: 0.0,
            };
        };
        let left = (at + self.settings.cooldown_sec * 1000.0 - (self.mono)()).max(0.0);
        ExitStatus {
            requested: true,
            ready: left == 0.0,
            cooldown_left_ms: left,
        }
    }

    /// The phrase, only once the cooldown has passed.
    pub fn exit_phrase(&self) -> Option<&str> {
        self.exit_status().ready.then_some(self.phrase.as_str())
    }

    pub fn confirm_exit(&self, typed: &str) -> Result<(), ExitError> {
        let st = self.exit_status();
        if !st.requested {
            return Err(ExitError::NotRequested);
        }
        if !st.ready {
            return Err(ExitError::Cooldown);
        }
        if normalize(typed) != normalize(&self.phrase) {
            return Err(ExitError::Mismatch);
        }
        Ok(())
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "profile": self.profile,
            "settings": self.settings.to_json(),
            "startedAt": self.started_at,
            "elapsedMs": self.elapsed_ms,
            "violations": self.violations,
            "restarts": self.restarts,
            "offences": self.offences,
            "pausedUsed": self.paused_used,
            "paused": self.paused_at.is_some(), // only so a restore knows (time away is not counted while paused)
            "phrase": self.phrase,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    use super::*;

    fn profile() -> Value {
        json!({ "name": "t", "workMin": 25, "breakMin": 5, "rounds": 3 })
    }

    /// A fake clock like `fakeClock()` in the JS tests.
    #[derive(Clone)]
    struct FakeClock(Arc<AtomicI64>);
    impl FakeClock {
        fn new() -> Self {
            Self(Arc::new(AtomicI64::new(1000)))
        }
        fn advance(&self, ms: i64) {
            self.0.fetch_add(ms, Ordering::SeqCst);
        }
        fn clock(&self) -> Clock {
            let c = self.0.clone();
            Box::new(move || c.load(Ordering::SeqCst) as f64)
        }
    }

    fn mk_with(extra: Value) -> (Session, FakeClock) {
        let c = FakeClock::new();
        let mut snap =
            json!({ "profile": profile(), "settings": { "cooldownSec": 120, "forceKill": true } });
        for (k, v) in extra.as_object().unwrap() {
            snap[k] = v.clone();
        }
        (Session::new(&snap, Some(c.clock())), c)
    }
    fn mk() -> (Session, FakeClock) {
        mk_with(json!({}))
    }
    const MIN: i64 = 60_000;

    #[test]
    fn build_phases_has_no_break_after_the_last_round() {
        let p = build_phases(&profile());
        let kinds: Vec<_> = p.iter().map(|x| x.kind.as_str()).collect();
        assert_eq!(kinds, ["work", "break", "work", "break", "work"]);
        assert_eq!(
            p.iter().map(|x| x.ms).sum::<f64>(),
            ((25 * 3 + 5 * 2) * 60_000) as f64
        );
    }

    #[test]
    fn build_phases_without_break_has_only_work_rounds() {
        assert_eq!(
            build_phases(&json!({ "workMin": 30, "breakMin": 0, "rounds": 1 })).len(),
            1
        );
    }

    #[test]
    fn session_changes_phase_on_time() {
        let (mut s, c) = mk();
        assert_eq!(s.tick().phase, PhaseKind::Work);
        c.advance(25 * MIN);
        let st = s.tick();
        assert_eq!(st.phase, PhaseKind::Break);
        assert_eq!(st.phase_remaining_ms, (5 * MIN) as f64);
        c.advance(5 * MIN);
        let st2 = s.tick();
        assert_eq!(st2.phase, PhaseKind::Work);
        assert_eq!(st2.round, 2);
    }

    #[test]
    fn session_ends_when_time_is_up_and_never_overshoots() {
        let (mut s, c) = mk();
        c.advance(10 * 3600 * 1000);
        let st = s.tick();
        assert!(st.done);
        assert_eq!(st.total_remaining_ms, 0.0);
        assert_eq!(s.elapsed_ms, s.total_ms());
    }

    #[test]
    fn work_elapsed_counts_work_only() {
        let phases = build_phases(&profile());
        assert_eq!(
            work_elapsed_ms(&phases, (27 * MIN) as f64),
            (25 * MIN) as f64
        );
        assert_eq!(
            work_elapsed_ms(&phases, (32 * MIN) as f64),
            (27 * MIN) as f64
        );
    }

    #[test]
    fn a_clock_running_backwards_does_not_reduce_elapsed_time() {
        let (mut s, c) = mk();
        c.advance(60_000);
        s.tick();
        c.advance(-30_000);
        s.tick();
        assert_eq!(s.elapsed_ms, 60_000.0);
    }

    #[test]
    fn early_exit_needs_request_cooldown_and_the_exact_phrase() {
        let (mut s, c) = mk_with(json!({ "phrase": "xin  chào   thế giới" }));
        assert_eq!(s.confirm_exit("x"), Err(ExitError::NotRequested));
        assert_eq!(s.exit_phrase(), None);
        s.request_exit();
        assert_eq!(
            s.confirm_exit("xin chào thế giới"),
            Err(ExitError::Cooldown)
        );
        assert_eq!(s.exit_phrase(), None);
        c.advance(119_000);
        assert!(!s.exit_status().ready);
        c.advance(1000);
        assert!(s.exit_status().ready);
        assert_eq!(s.exit_phrase(), Some("xin  chào   thế giới"));
        assert_eq!(s.confirm_exit("sai"), Err(ExitError::Mismatch));
        assert_eq!(s.confirm_exit("  xin chào   thế giới "), Ok(()));
    }

    #[test]
    fn cancelling_an_exit_request_resets_the_cooldown() {
        let (mut s, c) = mk();
        s.request_exit();
        c.advance(100_000);
        s.cancel_exit();
        s.request_exit();
        c.advance(50_000);
        assert!(!s.exit_status().ready);
    }

    #[test]
    fn emergency_pause_stops_the_clock_once_per_session_for_ten_minutes() {
        let (mut s, c) = mk();
        c.advance(60_000);
        s.tick();
        assert_eq!(s.elapsed_ms, 60_000.0);
        assert!(s.can_pause());
        assert!(s.pause());
        assert!(s.is_paused());
        c.advance(5 * MIN);
        s.tick();
        assert_eq!(s.elapsed_ms, 60_000.0, "paused: no time passes");
        assert!(s.state().paused);
        assert_eq!(s.pause_left_ms(), (5 * MIN) as f64);
        assert!(!s.pause(), "no stacked pauses");
        c.advance(5 * MIN);
        assert_eq!(s.pause_left_ms(), 0.0);
        assert!(s.resume());
        c.advance(30_000);
        s.tick();
        assert_eq!(s.elapsed_ms, 90_000.0, "resumes from where it stopped");
        assert!(!s.can_pause(), "the session's pause is used up");
        assert!(!s.pause());
        let r = Session::new(&s.to_json(), Some(c.clock()));
        assert!(
            !r.can_pause(),
            "a restored session remembers the pause was used"
        );
    }

    #[test]
    fn to_json_then_restore_keeps_progress() {
        let (mut s, c) = mk();
        c.advance(90_000);
        s.tick();
        s.violations = 3;
        let r = Session::new(&s.to_json(), Some(c.clock()));
        assert_eq!(r.elapsed_ms, 90_000.0);
        assert_eq!(r.violations, 3);
        assert_eq!(r.phrase, s.phrase);
    }

    #[test]
    fn session_uses_as_many_sentences_as_the_strictness_asks() {
        let mk1 = |n: usize| {
            Session::new(
                &json!({ "profile": profile(), "settings": { "cooldownSec": 1, "sentences": n } }),
                Some(Box::new(|| 0.0)),
            )
        };
        assert!(mk1(1).phrase.chars().count() < mk1(3).phrase.chars().count());
    }

    #[test]
    fn normalize_composes_unicode_and_collapses_spaces() {
        assert_eq!(normalize("  ca\u{0301}c \u{a0}\t hai\n"), "các hai");
        assert_eq!(normalize(""), "");
    }
}
