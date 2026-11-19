# Architecture

## Big picture

```
 renderer/ (HTML/JS in WebView2)  <--- window.api (shim.js) ---> app/src/main.rs  --->  core/ (pure logic)
   index / overlay / scold / gate        invoke('ipc', {channel,args})       |             engine, rules, stats...
                                         events: data, session, toast,       +--> app/src/winapi.rs (Win32)
                                         cheer, scold, gate                  +--> %APPDATA%\disturbless\disturbless.json
```

- **One controller, one lock.** `main.rs` keeps all state in `Ctl` behind a `Mutex`, accessed via `with(app, |ctl| ...)`. Never call `with` while already inside `with` (deadlock). Window creation/destruction is queued to the main thread with `on_main`; delayed work uses `after`.
- **Three background threads** (started in `spawn_loops`): a 1 s session tick, a ~1 s watcher (`winapi::list_windows` + `foreground`, only while `watcher_on`), and a 500 ms keeper that re-asserts the lock screen on top. Hosts-file work runs in its own threads because UAC can take a long time.
- **Time** is monotonic (`engine::mono_now`), so changing the system clock cannot shorten a session. Wall-clock time is only used for stats and for crediting time while the app was closed (`restore_or_start`, except while paused).
- **Persistence** is JSON, written atomically (`store.rs`), same shape as the old Electron version so existing data keeps working. The running session is saved every 5 ticks as `active` and restored on start.

## Session flow

`start_session` -> sanitize plan (`profile::sanitize`, strictness presets in `profile::level`) -> `Session::new` -> `engage` (watcher on if app/tab rules exist, hosts block requested) -> ticks -> `finish(completed|early|emergency)` writes a history entry and releases the hosts block.

Leaving early: request -> cooldown (30 s / 2 min / 5 min by strictness) -> retype 1/2/3 sentences (`phrases`, language-specific, NFC + whitespace normalised, paste blocked in the UI) -> confirm. Emergency: pause 10 min once per session, or stop with a reason (hold 3 s); both are logged, neither breaks the streak.

## Blocking

- **Apps** (`rules::evaluate`): block-mode closes only listed apps; allow-mode closes everything not listed except Windows-folder programs and programs whose path cannot be read. `SYSTEM_SAFE` and the app itself are never touched in either mode. A violating app gets a pause gate and a 5 s grace (`GRACE_MS`), then `WM_CLOSE`; after 4 s more, if strictness allows, `TerminateProcess` on it and its children.
- **Tabs** (`tabs::match_banned`): the foreground window of a known browser has its title parsed; a banned brand/domain at the END of the title matches (so "Steam engine - Wikipedia" does not). After the grace, Ctrl+W is sent only if the same browser pid and exact title are still in front.
- **Websites**: hosts file with `0.0.0.0` and `::` lines for each domain (plus `www.` and `m.`), written through a hidden elevated `cmd` (UAC). Failure is non-fatal; apps and tabs are still blocked.
- **Pause gate** (`gate.html`, `show_gate`): shown for a new violation, never keeps the app open, records the reason in `ctl.distractions` -> history entry `distractions`. The tab gate must not take focus (the browser has to stay in front for Ctrl+W); on timeout `show_gate`'s timer refocuses the browser and closes the tab itself. Untitled browser windows are ignored by the watcher (they flicker while other windows open).

## Language

`settings.lang` is `vi` or `en` (`persona::norm_lang`). New installs follow the Windows language; existing data without the key stays Vietnamese. UI text: `renderer/i18n.js` (`t(key, params)`; values may be functions for plurals). Rust text: `app/src/i18n.rs` (`t(lang, key, args)`, a test checks both columns and placeholders match). Cat lines and retype sentences: `core/src/persona.rs`, `phrases.rs`. Font size is `settings.fontScale` (1, 1.15, 1.3) applied as CSS `zoom` (not on the fixed-size mini window).

## Verified ports and the rust-anything harness

The project was converted from Electron/JS with rust-anything. Four modules are *verified ports*:

| core module | JS original (`core-esm/`) | harness crate | recorded cases |
|---|---|---|---|
| `tabs.rs` | `tabs.js` | `rust/tabs-rs` | 1426/1426 match |
| `hosts_text.rs` | `hosts_pure.js` | `rust/hosts-pure-rs` | 249/249 |
| `rules.rs` | `blocker_pure.js` | `rust/blocker-pure-rs` | 349/349 |
| `stats.rs` | `stats.js` | `rust/stats-rs` | 128/160 (the rest are nonsensical inputs: timestamps given as strings/arrays) |

`rust/*/src/port.rs` is the same code as `core/src/<module>.rs` (they share `rust/shared/jsstr.rs` and `tz.rs` with `core/src/jsstr.rs`, `tz.rs`). To re-verify after a change: copy the edited code into the matching `port.rs`, then from the repo root

```powershell
rust-anything build rust\tabs-rs; rust-anything diff rust\tabs-rs     # repeat per crate
```

(needs Node 22.6+, `rustup target add wasm32-unknown-unknown`, `cargo install wasm-bindgen-cli --version 0.2.130`). Never run `rust-anything setup`. The size gate (`rust-anything gate`) is expected to complain: the matchers are hand-written on purpose, because the `regex` crate differs from JS on `\s`, `.`, `\d` and case folding for Vietnamese text.

DST caveat: the stats logic was cross-checked against JS `Date` in four DST time zones through the wasm path; the native path uses `chrono` with the machine time zone and has only been run in UTC+7 (no DST).

## Known limits

- Windows cannot stop an app before it opens; blocking is "detect then close", which is why the pause gate exists.
- Closing the main window hides it to the tray; quitting is disabled during a session.
- The renderer uses CSS `zoom`; if WebView2 ever drops it, switch to `font-size` scaling with rem units.
