use std::ffi::CStr;
use std::os::raw::c_char;
use crate::{PolicyContext, PolicyMode, should_bypass_key, transform_commit};

#[no_mangle]
pub unsafe extern "C" fn policy_context_default_rs() -> PolicyContext {
    PolicyContext::default_legacy()
}

#[no_mangle]
pub unsafe extern "C" fn policy_context_create_rs(mode: i32) -> PolicyContext {
    let policy_mode = match mode {
        1 => PolicyMode::AsciiFirst,
        2 => PolicyMode::FormalZh,
        _ => PolicyMode::Legacy,
    };
    PolicyContext::new(policy_mode)
}

#[no_mangle]
pub unsafe extern "C" fn policy_should_bypass_key_rs(
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
pub unsafe extern "C" fn policy_transform_commit_rs(
    utf8_input: *const c_char,
    ctx: *const PolicyContext,
    out_buffer: *mut c_char,
    out_buffer_size: usize,
) -> usize {
    if utf8_input.is_null() || out_buffer.is_null() || out_buffer_size == 0 {
        return 0;
    }

    let c_str = match CStr::from_ptr(utf8_input).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let context = if ctx.is_null() {
        PolicyContext::default_legacy()
    } else {
        *ctx
    };

    let transformed = transform_commit(c_str, &context);
    let bytes = transformed.as_bytes();
    if bytes.len() + 1 > out_buffer_size {
        return 0;
    }

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buffer as *mut u8, bytes.len());
    *out_buffer.add(bytes.len()) = 0;

    bytes.len()
}
