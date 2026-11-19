## What changed

<!-- One or two sentences. Link the issue if there is one: Closes #123 -->

## How I tested it

<!-- Commands you ran, and what you checked in the real app if the change touches windows, blocking or the UI. -->

## Checklist

- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass
- [ ] `node scripts/dev/check-i18n.js` passes, and new text exists in Vietnamese and English
- [ ] `core/` stays free of UI and Win32 code
- [ ] `CHANGELOG.md` is updated if users will notice the change