@AGENTS.md

## Claude Code notes

- Shell is PowerShell (Windows 11). Prefer writing multi-line JS or scripts to files and passing the path; non-ASCII text and quoting inside `-Command` strings is fragile. `$home` is read-only in PowerShell, do not use it as a variable name.
- When driving the app with `scripts/dev/cdp.js`, match the main window by the exact URL `http://tauri.localhost/`; the mini clock and lock screen URLs also contain that text.
- Long builds (`cargo build --release`, about 1 minute) are fine in the foreground; run `cargo test` first, it is much faster.
- The user writes in Vietnamese; answer in Vietnamese unless asked otherwise. Keep UI copy friendly and plain for non-technical people.
- Project memory about the E: tool locations is kept in the user's Claude memory (`build-environment`); `scripts/dev/env.ps1` is the source of truth in the repo.
