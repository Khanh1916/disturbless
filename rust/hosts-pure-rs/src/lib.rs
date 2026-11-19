//! JS bindings for `core-esm/hosts_pure.js`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;

/// `cleanDomain`.
fn clean_domain_inner(raw: String) -> Result<String, String> {
    let raw = crate::js::from_json(&raw)?;
    let out = port::clean_domain(raw)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "cleanDomain")]
pub fn clean_domain(raw: String) -> Result<String, JsError> {
    clean_domain_inner(raw).map_err(|e| JsError::new(&e))
}

/// `expandDomains`.
fn expand_domains_inner(list: String) -> Result<String, String> {
    let list = crate::js::from_json(&list)?;
    let out = port::expand_domains(list)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "expandDomains")]
pub fn expand_domains(list: String) -> Result<String, JsError> {
    expand_domains_inner(list).map_err(|e| JsError::new(&e))
}

/// `stripBlock`.
fn strip_block_inner(text: String) -> Result<String, String> {
    let text = crate::js::from_json(&text)?;
    let out = port::strip_block(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "stripBlock")]
pub fn strip_block(text: String) -> Result<String, JsError> {
    strip_block_inner(text).map_err(|e| JsError::new(&e))
}

/// `withBlock`.
fn with_block_inner(text: String, domains: String) -> Result<String, String> {
    let text = crate::js::from_json(&text)?;
    let domains = crate::js::from_json(&domains)?;
    let out = port::with_block(text, domains)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "withBlock")]
pub fn with_block(text: String, domains: String) -> Result<String, JsError> {
    with_block_inner(text, domains).map_err(|e| JsError::new(&e))
}

/// `hasBlock`.
fn has_block_inner(text: String) -> Result<String, String> {
    let text = crate::js::from_json(&text)?;
    let out = port::has_block(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "hasBlock")]
pub fn has_block(text: String) -> Result<String, JsError> {
    has_block_inner(text).map_err(|e| JsError::new(&e))
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

static S_CLEAN_DOMAIN_0: wire::Schema = wire::Schema::Dyn;
static S_CLEAN_DOMAIN_OUT: wire::Schema = wire::Schema::Dyn;

/// `cleanDomain` over the binary wire.
#[wasm_bindgen(js_name = "cleanDomainWire")]
pub fn clean_domain_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let raw: serde_json::Value = wire::read_value(r, &S_CLEAN_DOMAIN_0)?;
        let out = &port::clean_domain(raw)?;
        wire::write_value(w, out, &S_CLEAN_DOMAIN_OUT)?;
        Ok(())
    })
}

static S_EXPAND_DOMAINS_0: wire::Schema = wire::Schema::Dyn;
static S_EXPAND_DOMAINS_OUT: wire::Schema = wire::Schema::Dyn;

/// `expandDomains` over the binary wire.
#[wasm_bindgen(js_name = "expandDomainsWire")]
pub fn expand_domains_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let list: serde_json::Value = wire::read_value(r, &S_EXPAND_DOMAINS_0)?;
        let out = &port::expand_domains(list)?;
        wire::write_value(w, out, &S_EXPAND_DOMAINS_OUT)?;
        Ok(())
    })
}

static S_STRIP_BLOCK_0: wire::Schema = wire::Schema::Dyn;
static S_STRIP_BLOCK_OUT: wire::Schema = wire::Schema::Dyn;

/// `stripBlock` over the binary wire.
#[wasm_bindgen(js_name = "stripBlockWire")]
pub fn strip_block_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: serde_json::Value = wire::read_value(r, &S_STRIP_BLOCK_0)?;
        let out = &port::strip_block(text)?;
        wire::write_value(w, out, &S_STRIP_BLOCK_OUT)?;
        Ok(())
    })
}

static S_WITH_BLOCK_0: wire::Schema = wire::Schema::Dyn;
static S_WITH_BLOCK_1: wire::Schema = wire::Schema::Dyn;
static S_WITH_BLOCK_OUT: wire::Schema = wire::Schema::Dyn;

/// `withBlock` over the binary wire.
#[wasm_bindgen(js_name = "withBlockWire")]
pub fn with_block_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: serde_json::Value = wire::read_value(r, &S_WITH_BLOCK_0)?;
        let domains: serde_json::Value = wire::read_value(r, &S_WITH_BLOCK_1)?;
        let out = &port::with_block(text, domains)?;
        wire::write_value(w, out, &S_WITH_BLOCK_OUT)?;
        Ok(())
    })
}

static S_HAS_BLOCK_0: wire::Schema = wire::Schema::Dyn;
static S_HAS_BLOCK_OUT: wire::Schema = wire::Schema::Dyn;

/// `hasBlock` over the binary wire.
#[wasm_bindgen(js_name = "hasBlockWire")]
pub fn has_block_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: serde_json::Value = wire::read_value(r, &S_HAS_BLOCK_0)?;
        let out = &port::has_block(text)?;
        wire::write_value(w, out, &S_HAS_BLOCK_OUT)?;
        Ok(())
    })
}
