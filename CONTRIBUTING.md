# Contributing

Thanks for taking the time. Issues, ideas and pull requests are all welcome.

## Before you start

Open an issue for anything bigger than a small fix, so we can agree on the direction before you spend time on it. Bug reports are most useful with the Windows version, the app version, what you did and what you expected.

If you found a security problem, do not open a public issue. Follow [SECURITY.md](SECURITY.md).

## Set up

You need Windows 10 or 11, Rust with the MSVC toolchain, and the Visual Studio Build Tools with the "Desktop development with C++" workload.

```powershell
git clone https://github.com/Khanh1916/disturbless.git
cd disturbless
cargo test --workspace
```

`scripts/dev/` has the helpers. `run-sandbox.ps1` starts the built app on throw-away data without touching your hosts file, and `cdp.js` drives its windows. `docs/TESTING.md` shows how to test blocking without closing your own apps.

## Checks that must pass

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node scripts/dev/check-i18n.js
```

CI runs the same four commands on every pull request.

## Rules for changes

- Keep `core/` free of UI and Win32 calls. Anything that talks to Windows goes in `app/src/winapi.rs`.
- Every text the user can see exists in Vietnamese and English: `renderer/i18n.js`, `app/src/i18n.rs`, `core/src/persona.rs` and `core/src/phrases.rs`. Do not add emoji, add an icon to `renderer/icons.js`.
- `core/src/{tabs,hosts_text,rules,stats}.rs` mirror `rust/<name>/src/port.rs`. Change both and re-run the comparison described in `docs/ARCHITECTURE.md`.
- New behaviour in `core/` needs a unit test.
- Never weaken a safety rule: the app must not close itself or Windows system processes, and it must send Ctrl+W only to the browser window it flagged.
- Do not start a real session with the default block groups while testing, because it closes your own apps. Use the recipe in `docs/TESTING.md`.

## Pull requests

Branch from `main`, keep one topic per pull request, and describe what changed and how you tested it. Commit messages follow [Conventional Commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`). Update `CHANGELOG.md` under "Unreleased" for anything a user would notice.

## Coding agents

`AGENTS.md` holds the project instructions for coding agents. It is the single source, so edit it there and nowhere else.
