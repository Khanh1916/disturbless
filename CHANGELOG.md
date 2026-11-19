# Changelog

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org).

## [Unreleased]

## [0.1.0] - 2026-11-20

First public release. The app began as an Electron prototype and was rewritten in Rust with Tauri.

### Added

- Focus sessions you cannot quit before the time is up, with quick starts of 25, 50 and 90 minutes and a step-by-step wizard.
- Blocking by block-list or allow-list, website blocking through the hosts file, and closing the active browser tab when its title matches a banned site.
- Pause gate: five seconds to stop and pick a reason when a blocked app or tab appears. Reasons feed the stats page.
- Early exit with a cooldown and a retyped paragraph, and an emergency button (10 minute pause or stop with a reason) that keeps the streak.
- Cat mascot with a sharp-tongued personality that cheers, scolds and slaps, with optional speech.
- Stats: streaks, daily minutes, completion rate, and what pulls you away most.
- Vietnamese and English interface, three text sizes, light and dark theme following Windows.
- A goal line shown on the clock and the lock screen.

### Changed

- The interface uses SVG icons instead of emoji and a softer layout.
- Core logic moved into a separate `core` crate. The tab matcher, hosts text, blocking rules and stats were checked against the original JavaScript with rust-anything.

[Unreleased]: https://github.com/Khanh1916/disturbless/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Khanh1916/disturbless/releases/tag/v0.1.0
