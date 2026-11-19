//! JS bindings for `core-esm/blocker_pure.js`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;

/// `normName`.
fn norm_name_inner(n: String) -> Result<String, String> {
    let n = crate::js::from_json(&n)?;
    let out = port::norm_name(n)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "normName")]
pub fn norm_name(n: String) -> Result<String, JsError> {
    norm_name_inner(n).map_err(|e| JsError::new(&e))
}

/// `isSystemPath`.
fn is_system_path_inner(p: String, system_root: String) -> Result<String, String> {
    let p = crate::js::from_json(&p)?;
    let system_root = crate::js::from_json(&system_root)?;
    let out = port::is_system_path(p, system_root)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isSystemPath")]
pub fn is_system_path(p: String, system_root: String) -> Result<String, JsError> {
    is_system_path_inner(p, system_root).map_err(|e| JsError::new(&e))
}

/// `evaluate`.
fn evaluate_inner(windows: String, cfg: String) -> Result<String, String> {
    let windows = crate::js::from_json(&windows)?;
    let cfg = crate::js::from_json(&cfg)?;
    let out = port::evaluate(windows, cfg)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "evaluate")]
pub fn evaluate(windows: String, cfg: String) -> Result<String, JsError> {
    evaluate_inner(windows, cfg).map_err(|e| JsError::new(&e))
}

// ---- the binary wire ------------------------------------------------------------------------

thread_local! {
    static WIRE_IN: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    static WIRE_OUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// This module's memory, where `index.cjs` reads and writes the wire buffers.
#[wasm_bindgen(js_name = "rustAnythingWireMemory")]
pub fn rust_anything_wire_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// The address of an argument buffer of `len` bytes.
#[wasm_bindgen(js_name = "rustAnythingWireIn")]
pub fn rust_anything_wire_in(len: u32) -> u32 {
    WIRE_IN.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr() as usize as u32
    })
}

/// The address of the last result.
#[wasm_bindgen(js_name = "rustAnythingWireOut")]
pub fn rust_anything_wire_out() -> u32 {
    WIRE_OUT.with(|b| b.borrow().as_ptr() as usize as u32)
}

/// Reads `len` argument bytes, runs `f`, and keeps its result for `rustAnythingWireOut`.
fn wire_io(
    len: u32,
    f: impl FnOnce(&mut wire::Reader, &mut wire::Writer) -> Result<(), String>,
) -> Result<u32, JsError> {
    let input = WIRE_IN.with(|b| std::mem::take(&mut *b.borrow_mut()));
    let mut w = wire::Writer::default();
    let result = f(
        &mut wire::Reader::new(&input[..(len as usize).min(input.len())]),
        &mut w,
    );
    WIRE_IN.with(|b| *b.borrow_mut() = input);
    match result {
        // A result of another shape than the TS type: JS runs the call through JSON.
        Err(e) if e.starts_with(wire::MISMATCH) => return Ok(u32::MAX),
        other => other.map_err(|e| JsError::new(&e))?,
    }
    let n = w.0.len();
    WIRE_OUT.with(|b| *b.borrow_mut() = w.0);
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

static S_NORM_NAME_0: wire::Schema = wire::Schema::Dyn;
static S_NORM_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `normName` over the binary wire.
#[wasm_bindgen(js_name = "normNameWire")]
pub fn norm_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let n: serde_json::Value = wire::read_value(r, &S_NORM_NAME_0)?;
        let out = &port::norm_name(n)?;
        wire::write_value(w, out, &S_NORM_NAME_OUT)?;
        Ok(())
    })
}

static S_IS_SYSTEM_PATH_0: wire::Schema = wire::Schema::Dyn;
static S_IS_SYSTEM_PATH_1: wire::Schema = wire::Schema::Dyn;
static S_IS_SYSTEM_PATH_OUT: wire::Schema = wire::Schema::Dyn;

/// `isSystemPath` over the binary wire.
#[wasm_bindgen(js_name = "isSystemPathWire")]
pub fn is_system_path_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let p: serde_json::Value = wire::read_value(r, &S_IS_SYSTEM_PATH_0)?;
        let system_root: serde_json::Value = wire::read_value(r, &S_IS_SYSTEM_PATH_1)?;
        let out = &port::is_system_path(p, system_root)?;
        wire::write_value(w, out, &S_IS_SYSTEM_PATH_OUT)?;
        Ok(())
    })
}

static S_EVALUATE_0: wire::Schema = wire::Schema::Dyn;
static S_EVALUATE_1: wire::Schema = wire::Schema::Dyn;
static S_EVALUATE_OUT: wire::Schema = wire::Schema::Dyn;

/// `evaluate` over the binary wire.
#[wasm_bindgen(js_name = "evaluateWire")]
pub fn evaluate_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let windows: serde_json::Value = wire::read_value(r, &S_EVALUATE_0)?;
        let cfg: serde_json::Value = wire::read_value(r, &S_EVALUATE_1)?;
        let out = &port::evaluate(windows, cfg)?;
        wire::write_value(w, out, &S_EVALUATE_OUT)?;
        Ok(())
    })
}
