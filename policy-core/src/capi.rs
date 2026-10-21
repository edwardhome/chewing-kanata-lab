use std::ffi::CStr;
use std::os::raw::c_char;
use crate::{PolicyContext, should_bypass_key, transform_commit};

#[no_mangle]
pub unsafe extern "C" fn policy_should_bypass_key(
    ascii_key: c_char,
    ctx: *const PolicyContext,
) -> bool {
    if ctx.is_null() {
        return false;
    }
    let context = &*ctx;
    let ch = (ascii_key as u8) as char;
    should_bypass_key(ch, context)
}

#[no_mangle]
pub unsafe extern "C" fn policy_transform_commit(
    utf8_text: *const c_char,
    ctx: *const PolicyContext,
) -> *const c_char {
    if utf8_text.is_null() {
        return std::ptr::null();
    }
    if ctx.is_null() {
        return utf8_text;
    }

    let c_str = CStr::from_ptr(utf8_text);
    if let Ok(rust_str) = c_str.to_str() {
        let context = &*ctx;
        let converted = transform_commit(rust_str, context);
        if converted == ":" {
            static COLON: &[u8] = b":\0";
            return COLON.as_ptr() as *const c_char;
        }
    }

    utf8_text
}
