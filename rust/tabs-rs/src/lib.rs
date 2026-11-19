//! JS bindings for `core-esm/tabs.js`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;

/// `pageTitle`.
fn page_title_inner(title: String) -> Result<String, String> {
    let title = crate::js::from_json(&title)?;
    let out = port::page_title(title)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "pageTitle")]
pub fn page_title(title: String) -> Result<String, JsError> {
    page_title_inner(title).map_err(|e| JsError::new(&e))
}

/// `brandOf`.
fn brand_of_inner(domain: String) -> Result<String, String> {
    let domain = crate::js::from_json(&domain)?;
    let out = port::brand_of(domain)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "brandOf")]
pub fn brand_of(domain: String) -> Result<String, JsError> {
    brand_of_inner(domain).map_err(|e| JsError::new(&e))
}

/// `matchBanned`.
fn match_banned_inner(
    title: String,
    brands: Vec<String>,
    domains: Vec<String>,
) -> Result<String, String> {
    let out = port::match_banned(title, brands, domains)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "matchBanned")]
pub fn match_banned(
    title: String,
    brands: Vec<String>,
    domains: Vec<String>,
) -> Result<String, JsError> {
    match_banned_inner(title, brands, domains).map_err(|e| JsError::new(&e))
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

static S_PAGE_TITLE_0: wire::Schema = wire::Schema::Dyn;
static S_PAGE_TITLE_OUT: wire::Schema = wire::Schema::Dyn;

/// `pageTitle` over the binary wire.
#[wasm_bindgen(js_name = "pageTitleWire")]
pub fn page_title_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let title: serde_json::Value = wire::read_value(r, &S_PAGE_TITLE_0)?;
        let out = &port::page_title(title)?;
        wire::write_value(w, out, &S_PAGE_TITLE_OUT)?;
        Ok(())
    })
}

static S_BRAND_OF_0: wire::Schema = wire::Schema::Dyn;
static S_BRAND_OF_OUT: wire::Schema = wire::Schema::Dyn;

/// `brandOf` over the binary wire.
#[wasm_bindgen(js_name = "brandOfWire")]
pub fn brand_of_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let domain: serde_json::Value = wire::read_value(r, &S_BRAND_OF_0)?;
        let out = &port::brand_of(domain)?;
        wire::write_value(w, out, &S_BRAND_OF_OUT)?;
        Ok(())
    })
}

static S_MATCH_BANNED_OUT: wire::Schema = wire::Schema::Dyn;

/// `matchBanned` over the binary wire.
#[wasm_bindgen(js_name = "matchBannedWire")]
pub fn match_banned_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let title: String = r.string16()?;
        let brands: Vec<String> = r.list(|r| r.string16())?;
        let domains: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::match_banned(title, brands, domains)?;
        wire::write_value(w, out, &S_MATCH_BANNED_OUT)?;
        Ok(())
    })
}
