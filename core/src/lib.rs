//! Disturbless core: everything that is pure logic.
//!
//! `tabs`, `hosts_text`, `rules` and `stats` are the Rust ports verified against the original
//! JavaScript with rust-anything (see `rust/*/cases.jsonl`); `engine`, `persona`, `phrases`,
//! `profile` and `store` are written natively and covered by the tests ported from
//! `test/core.test.js`.

pub mod engine;
pub mod hosts_text;
pub mod js;
pub mod jsstr;
pub mod persona;
pub mod phrases;
pub mod profile;
pub mod rules;
pub mod stats;
pub mod store;
pub mod tabs;
pub mod tz;
