// SPDX-License-Identifier: MIT OR Apache-2.0
//! UTF-8, Latin-1 and UTF-16 strings.
use super::*;

use std::slice;

use rquickjs::{Coerced, String as JsString};

unsafe fn slice<'a, T>(p: *const T, len: usize) -> Status<&'a [T]> {
    if p.is_null() {
        return if len == 0 {
            Ok(&[])
        } else {
            Err(NAPI_INVALID_ARG)
        };
    }
    Ok(slice::from_raw_parts(p, len))
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_string_utf8(
    env: napi_env,
    s: *const c_char,
    length: usize,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let s = cstr(s, length)?;
        let v = js(env, JsString::from_str(ctx(env), &s))?;
        out(result, push(env, extend(v.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_string_latin1(
    env: napi_env,
    s: *const c_char,
    length: usize,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let bytes = if length == NAPI_AUTO_LENGTH && !s.is_null() {
            CStr::from_ptr(s).to_bytes()
        } else {
            slice(s as *const u8, length)?
        };
        let text: String = bytes.iter().map(|&b| b as char).collect();
        let v = js(env, JsString::from_str(ctx(env), &text))?;
        out(result, push(env, extend(v.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_string_utf16(
    env: napi_env,
    s: *const u16,
    length: usize,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let len = if length == NAPI_AUTO_LENGTH && !s.is_null() {
            (0..).take_while(|&i| *s.add(i) != 0).count()
        } else {
            length
        };
        let text = String::from_utf16_lossy(slice(s, len)?);
        let v = js(env, JsString::from_str(ctx(env), &text))?;
        out(result, push(env, extend(v.into_value())))
    })
}

/// Copies `units` into `buf` as a NUL-terminated string of at most `bufsize - 1` units.
unsafe fn copy_out<T: Copy + Default>(
    units: &[T],
    buf: *mut T,
    bufsize: usize,
    result: *mut usize,
) -> Status<()> {
    if buf.is_null() {
        return out(result, units.len());
    }
    if bufsize == 0 {
        return if result.is_null() {
            Ok(())
        } else {
            out(result, 0)
        };
    }
    let n = units.len().min(bufsize - 1);
    ptr::copy_nonoverlapping(units.as_ptr(), buf, n);
    *buf.add(n) = T::default();
    if result.is_null() {
        Ok(())
    } else {
        out(result, n)
    }
}

unsafe fn string_arg(env: napi_env, value: napi_value) -> Status<String> {
    let s = get(value)?.as_string().ok_or(NAPI_STRING_EXPECTED)?;
    js(env, s.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_utf8(
    env: napi_env,
    value: napi_value,
    buf: *mut c_char,
    bufsize: usize,
    result: *mut usize,
) -> napi_status {
    run(env, || {
        let s = string_arg(env, value)?;
        let bytes = s.as_bytes();
        if buf.is_null() {
            return out(result, bytes.len());
        }
        if bufsize == 0 {
            if !result.is_null() {
                *result = 0;
            }
            return Ok(());
        }
        let mut n = bytes.len().min(bufsize - 1);
        while n > 0 && n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n -= 1;
        }
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, n);
        *buf.add(n) = 0;
        if !result.is_null() {
            *result = n;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_latin1(
    env: napi_env,
    value: napi_value,
    buf: *mut c_char,
    bufsize: usize,
    result: *mut usize,
) -> napi_status {
    run(env, || {
        let bytes: Vec<u8> = string_arg(env, value)?
            .chars()
            .map(|c| if (c as u32) < 256 { c as u8 } else { b'?' })
            .collect();
        copy_out(&bytes, buf as *mut u8, bufsize, result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_string_utf16(
    env: napi_env,
    value: napi_value,
    buf: *mut u16,
    bufsize: usize,
    result: *mut usize,
) -> napi_status {
    run(env, || {
        let units: Vec<u16> = string_arg(env, value)?.encode_utf16().collect();
        copy_out(&units, buf, bufsize, result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_string(
    env: napi_env,
    value: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let value = get(value)?;
        let coerced: Coerced<JsString> = js(env, value.get())?;
        out(result, push(env, extend(coerced.0.into_value())))
    })
}
