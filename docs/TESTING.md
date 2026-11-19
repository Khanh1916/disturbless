# Testing

## Automatic

```powershell
. .\scripts\dev\env.ps1
cargo test --workspace            # engine, stats, rules, tabs, hosts, profile, store, persona, phrases, i18n
cargo clippy --workspace --all-targets
node scripts/dev/check-i18n.js
```

`core/tests/ported.rs` holds the original JS test-suite cases (stats, blocker, tabs, hosts) translated one-to-one. Engine tests use a fake clock.

## Manual, against the real app (safe recipes)

Start a sandbox: `.\scripts\dev\run-sandbox.ps1` (data in `%TEMP%\disturbless-sandbox`, hosts untouched, debug port 9333, trace log in `trace.log`). Drive it with `node scripts/dev/cdp.js 9333 http://tauri.localhost/ "<js>" [shot.png]`. Stop it with `Stop-Process -Id (Get-Content $env:TEMP\disturbless-sandbox.pid)`.

**Never start a session with the default block groups**, it would close the tester's real chat/music apps. Use a plan that blocks something harmless. Paste into a file and run via `cdp.js`:

```js
(async () => window.api.startSession({ plan: {
  name: 'Test', goal: 'study', mode: 'apps', workMin: 2, breakMin: 0, rounds: 1,
  appMode: 'block', blockedApps: ['charmap'], blockedSites: [], blockedTitles: [],
  closeTabs: false, strictness: 'easy', intent: 'Testing' } }))()
```

Then `Start-Process charmap.exe`: expect the cat scold, the pause gate (`http://tauri.localhost/gate.html`), and Character Map closed after 5 s (or at once via the gate button). Always finish with `window.api.emergencyStop('other')`.

Other recipes:
- **Tab blocking:** plan with `appMode:'off', blockedTitles:['YouTube'], closeTabs:true`; open Edge with a *private profile* and a single tab titled "... - YouTube" (`msedge.exe --user-data-dir=<tmp> --no-first-run --new-window "data:text/html;charset=utf-8,%3Ctitle%3EX%20-%20YouTube%3C/title%3E"`, no raw spaces in the URL). Expect one gate, then the window closes within ~6 s. Kill only that Edge (filter by its `--user-data-dir` command line).
- **Lock screen:** a `mode:'fullscreen'` plan covers every monitor, end it within seconds; `window.api.emergencyStop` works from the main page (`http://tauri.localhost/`).
- **Hosts file:** needs a human to click the UAC prompt. Back up `%SystemRoot%\System32\drivers\etc\hosts` first, use a fake domain, and compare file hashes afterwards. Start the app *without* `DL_NOHOSTS`.
- **Both languages:** `window.api.saveSettings({lang:'en'})` then reload the page (`location.reload()`); check text, the cat bubble (comes from Rust), and the retype phrase.
- **Font size:** `window.api.saveSettings({fontScale:1.3})`, reload, check nothing is cut off at the 860x620 minimum window.

## Gotchas learned the hard way

- `cdp.js` matches pages by exact URL first; matching by substring silently attached to the mini clock, which is destroyed when the session ends (script then exits with no output).
- A window that is destroyed during `Runtime.evaluate` produces no output and exit code 0.
- Hidden-window timers and non-activating windows behave differently: the main window is hidden while a session runs, but its page stays alive.
- Foreground tracking: our own windows being in front must not reset the pending tab; an untitled browser window is "no information", not "not banned".
- Date handling in `stats` uses JS-style months (0-based) and overflow; use `tz::make` / `tz::parts`, not `chrono` directly.
- Edge's first-run pages can steal the active tab; always pass `--no-first-run` and a private profile.
