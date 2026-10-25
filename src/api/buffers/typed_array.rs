// SPDX-License-Identifier: MIT OR Apache-2.0
//! TypedArray and DataView APIs.
use super::*;

use rquickjs::{ArrayBuffer, Constructor};

const TYPED_ARRAY_CONSTRUCTORS: [&str; 11] = [
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];

#[no_mangle]
pub unsafe extern "C" fn napi_is_typedarray(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        out(result, qjs::JS_GetTypedArrayType(get(value)?.as_raw()) >= 0)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_typedarray_info(
    env: napi_env,
    typedarray: napi_value,
    kind: *mut c_int,
    length: *mut usize,
    data: *mut *mut c_void,
    arraybuffer: *mut napi_value,
    byte_offset: *mut usize,
) -> napi_status {
    run(env, || {
        let (k, len, offset, byte_length, buffer) = view_info(env, typedarray)?;
        if !kind.is_null() {
            *kind = k as c_int;
        }
        if !length.is_null() {
            *length = len;
        }
        if !data.is_null() {
            *data = view_ptr(buffer.clone(), offset, byte_length)?;
        }
        if !arraybuffer.is_null() {
            *arraybuffer = push(env, buffer);
        }
        if !byte_offset.is_null() {
            *byte_offset = offset;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_dataview(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        out(result, qjs::JS_IsDataView(get(value)?.as_raw()))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_typedarray(
    env: napi_env,
    kind: c_int,
    length: usize,
    arraybuffer: napi_value,
    byte_offset: usize,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        if !(0..=10).contains(&kind) {
            return Err(NAPI_INVALID_ARG);
        }
        let ab = get(arraybuffer)?;
        if ArrayBuffer::from_value(ab.clone()).is_none() {
            return Err(NAPI_INVALID_ARG);
        }
        let constructor: Constructor = js(
            env,
            ctx(env)
                .globals()
                .get(TYPED_ARRAY_CONSTRUCTORS[kind as usize]),
        )?;
        let v: V = js(
            env,
            constructor.construct((ab.clone(), byte_offset as f64, length as f64)),
        )?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_dataview(
    env: napi_env,
    length: usize,
    arraybuffer: napi_value,
    byte_offset: usize,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ab = get(arraybuffer)?;
        if ArrayBuffer::from_value(ab.clone()).is_none() {
            return Err(NAPI_INVALID_ARG);
        }
        let constructor: Constructor = js(env, ctx(env).globals().get("DataView"))?;
        let v: V = js(
            env,
            constructor.construct((ab.clone(), byte_offset as f64, length as f64)),
        )?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_dataview_info(
    env: napi_env,
    dataview: napi_value,
    byte_length: *mut usize,
    data: *mut *mut c_void,
    arraybuffer: *mut napi_value,
    byte_offset: *mut usize,
) -> napi_status {
    run(env, || {
        if !qjs::JS_IsDataView(get(dataview)?.as_raw()) {
            return Err(NAPI_INVALID_ARG);
        }
        let (len, offset, buffer) = buf_info(env, dataview)?;
        if !byte_length.is_null() {
            *byte_length = len;
        }
        if !data.is_null() {
            *data = view_ptr(buffer.clone(), offset, len)?;
        }
        if !arraybuffer.is_null() {
            *arraybuffer = push(env, buffer);
        }
        if !byte_offset.is_null() {
            *byte_offset = offset;
        }
        Ok(())
    })
}
