# Third-party material

The MIT License in `LICENSE` covers the source code written for this repository. The items below are not covered by it.

## Cat photo

`renderer/cat/happy.jpg` is a meme image added by the repository owner. Its original author and license are not known to this project, so no license is granted for it. If you are the rights holder and want it removed, open an issue and it will be taken down. The app works without it: when no photo is present it falls back to a cat drawn in SVG (`renderer/cat.js`). To use your own picture, put files named `happy`, `cheer`, `worried`, `angry` or `slapped` (png, jpg, webp, gif or avif) in `renderer/cat/` and rebuild.

## App icon

`assets/icon.png` and `assets/icon.ico` were produced for this project and may contain the cat photo above, so the same caveat applies.

## Icons in the UI

The shapes in `renderer/icons.js` follow the style of [Lucide](https://lucide.dev) (ISC License). They are redrawn by hand and not copied from the Lucide package, but credit belongs there.

## Rust dependencies

The binary links crates such as `tauri`, `serde`, `chrono`, `windows`, `png` and `unicode-normalization`. Each keeps its own license (MIT, Apache-2.0 or similar). `cargo tree` and `Cargo.lock` list them all.

## rust-anything

`core-esm/` and `rust/` come from a migration done with [rust-anything](https://github.com/erkamyaman/rust-anything). The recorded cases in `rust/*/cases.jsonl` are generated data. The tool itself is not part of this repository.
