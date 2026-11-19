# AGENTS.md: Disturbless

Disturbless is a Windows focus-lock app: the user picks a duration and cannot quit until it is over. It blocks apps, websites and browser tabs, and a cat mascot scolds or "slaps" on attempts to escape. Rust + Tauri 2 (WebView2), web UI in `renderer/`, UI languages Vietnamese (default) and English. Target users are non-technical people, so wording and UX matter as much as code.

## Commands (Windows, PowerShell)

```powershell
. .\scripts\dev\env.ps1                    # tools live on E:\tools; this sets env vars only, never PATH/registry
cargo test --workspace                     # 48 tests, must pass
cargo clippy --workspace --all-targets     # must be warning-free
cargo fmt --all
node scripts/dev/check-i18n.js             # vi/en keys and {placeholders} must match
cargo build --release -p disturbless       # -> $env:CARGO_TARGET_DIR\release\disturbless.exe
Copy-Item "$env:CARGO_TARGET_DIR\release\disturbless.exe" release\Disturbless.exe -Force
.\scripts\dev\run-sandbox.ps1              # run the built exe on throw-away data, debug port 9333
node scripts/dev/cdp.js 9333 http://tauri.localhost/ "<js or file.js>" [shot.png]   # drive/inspect a window
```

The UI is embedded into the exe at build time: after editing anything in `renderer/`, rebuild before testing.

## Layout

- `core/`, pure logic, no UI, no Win32. `engine` (session state machine, monotonic clock), `stats`, `rules` (which windows violate; system processes are never closed), `tabs` (browser-tab title matching), `hosts_text`, `profile` (sanitising), `store` (atomic JSON), `persona`/`phrases` (cat lines and retype sentences, vi + en), `jsstr`/`tz` (JS-compatible string and calendar helpers).
- `app/`, Tauri shell. `main.rs` (windows, tray, session controller, IPC), `winapi.rs` (all Win32: enumerate/close windows, Ctrl+W, icons, hosts write via UAC, autostart), `i18n.rs` (Rust-side messages), `shim.js` (provides `window.api` to the UI).
- `renderer/`, HTML/CSS/JS UI. Text lives in `i18n.js`; icons in `icons.js`; pages: `index.html` (main), `overlay.html` (lock screen + mini clock), `scold.html` (cat scene), `gate.html` (pause gate).
- `core-esm/`, `rust/`, verification harness: original JS of four core modules and rust-anything crates holding the recorded JS behaviour. See `docs/ARCHITECTURE.md`.
- `docs/`, `ARCHITECTURE.md` (design, data flow, why), `TESTING.md` (safe manual test recipes, gotchas). Read the relevant one before changing behaviour.

## Rules for code

- Keep `core/` free of UI and Win32; put anything that talks to Windows in `app/src/winapi.rs`. New behaviour in `core/` needs a unit test next to it.
- `core/src/{tabs,hosts_text,rules,stats}.rs` are verified ports: each mirrors `rust/<name>/src/port.rs`. If you change one, change the other and re-run `rust-anything diff` (see `docs/ARCHITECTURE.md`).
- Every user-visible string exists in Vietnamese and English: `renderer/i18n.js` (both dictionaries), `app/src/i18n.rs` (both columns), `core/src/persona.rs` and `phrases.rs`. Never hard-code UI text in `app.js` or `main.rs`. No emoji in the UI; add an SVG to `renderer/icons.js` instead.
- IPC goes through one command, `ipc(channel, args)`, handled in `main.rs::handle`. A new channel needs: handler, `shim.js` entry, and validation of anything the page sends (the page must never be able to name an arbitrary process or path).
- Safety invariants that must never regress: the app never closes itself or `SYSTEM_SAFE` processes; in allow-mode it never closes Windows-folder or unverifiable programs; Ctrl+W is only sent while the same browser window with the same title is in front; the hosts block lives between the `# >>> DISTURBLESS BEGIN/END` markers and is removed when the session ends.
- Rust edition 2024; `windows` crate 0.61 (`BOOL` is `windows::core::BOOL`). `unsafe` only in `winapi.rs`; `core/` is `forbid(unsafe_code)`.

## Boundaries

**Always:** run the commands above before saying work is done; test the real app in the sandbox for anything touching windows, processes or UI; report what you did not verify.

**Ask first:** anything that triggers a Windows UAC prompt (hosts file), a human must click it; changing the data format in `%APPDATA%\disturbless\disturbless.json` (existing users' profiles and history live there); adding dependencies.

**Never:**
- Start a real session with default block groups on the user's machine: it closes their open Discord/Zalo/Spotify/etc. Tests use a plan that blocks only `charmap` (Character Map), see `docs/TESTING.md`.
- Leave a fullscreen lock, gate or scold window up: end test sessions at once with `emergencyStop`.
- Run `rust-anything setup` (edits agent configs), change PATH, the registry, or Windows settings, or install tools outside `E:\tools`.
- Commit secrets; there are none in this project, keep it that way.

## Definition of done

Tests, clippy, fmt and `check-i18n` pass; release exe rebuilt and smoke-tested in the sandbox; both languages checked if text changed; README or `docs/` updated if behaviour or structure changed.

## Which agent reads what

This file is the single source of instructions. Claude Code reads CLAUDE.md, which just imports this file (@AGENTS.md) and adds Claude-only notes. Google Antigravity, Codex, Cursor, Copilot and similar tools read AGENTS.md from the repository root natively, so there is deliberately no duplicate GEMINI.md (Antigravity would load both and risk conflicting copies). Antigravity loads nested AGENTS.md files only when enabled in its settings, so keep everything in this root file and link to docs/ for detail. Edit instructions here only.
