//! FFI-safe value DTO (`HudValue`) and conversions to/from `Value16`.
//!
//! The DTO is a plain `#[repr(C)]` tree the Dart side mirrors with a struct
//! of identical layout. Ownership rule: every non-null pointer inside a
//! `HudValue` is owned by that DTO and released by `hud_value_free`.
//!
//! Notes:
//! * Int is carried as `i64` verbatim so NaN-boxed integers survive the
//!   round trip without a lossy `f64` detour.
//! * BigInt has no cross-FFI constructor; it degrades to a decimal STRING.
//! * A rejected promise surfaces as `HUD_TAG_ERROR` with the reason in
//!   `str_val`. `HUD_TAG_PROMISE` carries the awaitable id in `promise_id`.
//! * Conversion depth is capped to protect the native stack from hostile
//!   nesting; deeper values become null.

use std::ffi::CString;
use std::os::raw::c_char;

use hudhudscript_bytecode::{PromiseState16, Value16};

pub const HUD_TAG_NULL: u8 = 0;
pub const HUD_TAG_BOOL: u8 = 1;
pub const HUD_TAG_INT: u8 = 2;
pub const HUD_TAG_FLOAT: u8 = 3;
pub const HUD_TAG_STRING: u8 = 4;
pub const HUD_TAG_LIST: u8 = 5;
pub const HUD_TAG_MAP: u8 = 6;
pub const HUD_TAG_PROMISE: u8 = 7;
pub const HUD_TAG_ERROR: u8 = 8;

const MAX_DEPTH: u8 = 64;

#[repr(C)]
pub struct HudMapEntry {
    pub key: *mut c_char,
    /// Separately boxed value (pointer, not inline) — keeps FFI-side
    /// consumers (Dart struct views) simple.
    pub value: *mut HudValue,
}

#[repr(C)]
pub struct HudValue {
    pub tag: u8,
    pub int_val: i64,
    pub float_val: f64,
    pub bool_val: u8,
    pub str_val: *mut c_char,
    pub items: *mut HudValue,
    pub entries: *mut HudMapEntry,
    pub len: u32,
    pub promise_id: *mut c_char,
}

fn null_dto() -> HudValue {
    HudValue {
        tag: HUD_TAG_NULL,
        int_val: 0,
        float_val: 0.0,
        bool_val: 0,
        str_val: std::ptr::null_mut(),
        items: std::ptr::null_mut(),
        entries: std::ptr::null_mut(),
        len: 0,
        promise_id: std::ptr::null_mut(),
    }
}

fn cstring_lossy(s: &str) -> *mut c_char {
    // Interior NUL bytes cannot cross the C boundary; strip them.
    let cleaned: String = s.chars().filter(|&c| c != '\0').collect();
    match CString::new(cleaned) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert a `Value16` into an owned DTO. Consumed by the caller via
/// `hud_value_free` (or interpreted as a borrowed `*const` input).
pub fn value16_to_dto(value: &Value16, depth: u8) -> HudValue {
    if depth >= MAX_DEPTH {
        return null_dto();
    }
    if value.is_null() {
        return null_dto();
    }
    if let Some(b) = value.as_bool() {
        let mut dto = null_dto();
        dto.tag = HUD_TAG_BOOL;
        dto.bool_val = u8::from(b);
        return dto;
    }
    if let Some(i) = value.as_int() {
        let mut dto = null_dto();
        dto.tag = HUD_TAG_INT;
        dto.int_val = i;
        return dto;
    }
    if let Some(f) = value.as_number() {
        let mut dto = null_dto();
        dto.tag = HUD_TAG_FLOAT;
        dto.float_val = f;
        return dto;
    }
    if let Some(s) = value.as_str() {
        let mut dto = null_dto();
        dto.tag = HUD_TAG_STRING;
        dto.str_val = cstring_lossy(s);
        return dto;
    }
    if let Some(items) = value.as_array() {
        let converted: Vec<HudValue> =
            items.iter().map(|item| value16_to_dto(item, depth + 1)).collect();
        let mut dto = null_dto();
        dto.tag = HUD_TAG_LIST;
        dto.len = converted.len() as u32;
        dto.items = if converted.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(converted.into_boxed_slice()) as *mut HudValue
        };
        return dto;
    }
    if let Some(obj) = value.as_object() {
        let converted: Vec<HudMapEntry> = obj
            .iter()
            .map(|(sym, val)| HudMapEntry {
                key: cstring_lossy(&hudhudscript_bytecode::interner::resolve(
                    hudhudscript_bytecode::interner::SymbolId(sym.0),
                )),
                value: Box::into_raw(Box::new(value16_to_dto(val, depth + 1))),
            })
            .collect();
        let mut dto = null_dto();
        dto.tag = HUD_TAG_MAP;
        dto.len = converted.len() as u32;
        dto.entries = if converted.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(converted.into_boxed_slice()) as *mut HudMapEntry
        };
        return dto;
    }
    if let Some(state) = value.as_promise_state() {
        match state {
            PromiseState16::Resolved(inner) => value16_to_dto(inner, depth + 1),
            PromiseState16::Rejected(msg) => {
                let mut dto = null_dto();
                dto.tag = HUD_TAG_ERROR;
                dto.str_val = cstring_lossy(msg);
                dto
            }
            PromiseState16::Pending => {
                let mut dto = null_dto();
                dto.tag = HUD_TAG_PROMISE;
                dto
            }
            PromiseState16::AsyncPending(id) => {
                let mut dto = null_dto();
                dto.tag = HUD_TAG_PROMISE;
                dto.promise_id = cstring_lossy(id);
                dto
            }
        }
    } else if value.is_bigint() {
        // No FFI bigint constructor exists; degrade to decimal string.
        let mut dto = null_dto();
        dto.tag = HUD_TAG_STRING;
        dto.str_val = cstring_lossy(&format!("{value:?}"));
        dto
    } else {
        null_dto()
    }
}

/// Convert a borrowed DTO back into a `Value16`. `depth` guards recursion.
pub fn dto_to_value16(dto: *const HudValue, depth: u8) -> Result<Value16, String> {
    if depth >= MAX_DEPTH {
        return Err("value nesting too deep".to_string());
    }
    if dto.is_null() {
        return Ok(Value16::null());
    }
    let d = unsafe { &*dto };
    match d.tag {
        HUD_TAG_NULL => Ok(Value16::null()),
        HUD_TAG_BOOL => Ok(Value16::bool_(d.bool_val != 0)),
        HUD_TAG_INT => Ok(Value16::int(d.int_val)),
        HUD_TAG_FLOAT => Ok(Value16::number(d.float_val)),
        HUD_TAG_STRING => unsafe {
            if d.str_val.is_null() {
                return Ok(Value16::string(""));
            }
            let s = std::ffi::CStr::from_ptr(d.str_val).to_string_lossy();
            Ok(Value16::string(s.into_owned()))
        },
        HUD_TAG_LIST => unsafe {
            if d.items.is_null() || d.len == 0 {
                return Ok(Value16::array(Vec::new()));
            }
            let slice = std::slice::from_raw_parts(d.items, d.len as usize);
            let mut items = Vec::with_capacity(slice.len());
            for item in slice {
                items.push(dto_to_value16(item, depth + 1)?);
            }
            Ok(Value16::array(items))
        },
        HUD_TAG_MAP => unsafe {
            if d.entries.is_null() || d.len == 0 {
                return Ok(Value16::object(Vec::<(String, Value16)>::new()));
            }
            let slice = std::slice::from_raw_parts(d.entries, d.len as usize);
            let mut pairs: Vec<(String, Value16)> = Vec::with_capacity(slice.len());
            for entry in slice {
                if entry.key.is_null() || entry.value.is_null() {
                    continue;
                }
                let key = std::ffi::CStr::from_ptr(entry.key).to_string_lossy();
                let value = dto_to_value16(entry.value, depth + 1)?;
                pairs.push((key.into_owned(), value));
            }
            Ok(Value16::object(pairs))
        },
        HUD_TAG_PROMISE => unsafe {
            if d.promise_id.is_null() {
                return Err("promise DTO without id".to_string());
            }
            let id = std::ffi::CStr::from_ptr(d.promise_id).to_string_lossy();
            Ok(Value16::promise(PromiseState16::AsyncPending(id.into_owned())))
        },
        HUD_TAG_ERROR => Err(unsafe {
            if d.str_val.is_null() {
                "rejected promise".to_string()
            } else {
                std::ffi::CStr::from_ptr(d.str_val).to_string_lossy().into_owned()
            }
        }),
        other => Err(format!("unknown HudValue tag: {other}")),
    }
}

/// Free the owned children of a DTO in place (strings, slices, entries).
/// Slice elements are array members, not individually boxed, so only their
/// contents are freed recursively — never `Box::from_raw`ed per element.
///
/// # Safety
/// All non-null pointers in `dto` must have been produced by this crate.
pub unsafe fn free_dto_contents(dto: &mut HudValue) {
    if !dto.str_val.is_null() {
        drop(CString::from_raw(dto.str_val));
        dto.str_val = std::ptr::null_mut();
    }
    if !dto.promise_id.is_null() {
        drop(CString::from_raw(dto.promise_id));
        dto.promise_id = std::ptr::null_mut();
    }
    if !dto.items.is_null() && dto.len > 0 {
        let slice = std::slice::from_raw_parts_mut(dto.items, dto.len as usize);
        for item in slice {
            free_dto_contents(item);
        }
        drop(Vec::from_raw_parts(dto.items, dto.len as usize, dto.len as usize));
        dto.items = std::ptr::null_mut();
    }
    if !dto.entries.is_null() && dto.len > 0 {
        let slice = std::slice::from_raw_parts_mut(dto.entries, dto.len as usize);
        for entry in slice {
            if !entry.key.is_null() {
                drop(CString::from_raw(entry.key));
                entry.key = std::ptr::null_mut();
            }
            if !entry.value.is_null() {
                let mut boxed = Box::from_raw(entry.value);
                free_dto_contents(&mut boxed);
                entry.value = std::ptr::null_mut();
            }
        }
        drop(Vec::from_raw_parts(
            dto.entries,
            dto.len as usize,
            dto.len as usize,
        ));
        dto.entries = std::ptr::null_mut();
    }
}

/// Recursively free a boxed `HudValue` returned by this library.
///
/// # Safety
/// `v` must originate from `Box::into_raw` inside this crate and must not
/// be freed twice.
#[no_mangle]
pub unsafe extern "C" fn hud_value_free(v: *mut HudValue) {
    if v.is_null() {
        return;
    }
    let mut boxed = Box::from_raw(v);
    free_dto_contents(&mut boxed);
}
