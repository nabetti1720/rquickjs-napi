// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared access to ArrayBufferView backing stores.
use super::*;

use rquickjs::{ArrayBuffer, Function, Object};

pub(super) unsafe fn view_ptr(buffer: V, offset: usize, byte_length: usize) -> Status<*mut c_void> {
    let ab = ArrayBuffer::from_value(buffer).ok_or(NAPI_INVALID_ARG)?;
    let raw = ab.as_raw().ok_or(NAPI_INVALID_ARG)?;
    let end = offset.checked_add(byte_length).ok_or(NAPI_INVALID_ARG)?;
    if end > raw.len() {
        return Err(NAPI_INVALID_ARG);
    }
    Ok((raw.as_ptr() as *mut u8).add(offset) as *mut c_void)
}

/// Returns `(byteLength, byteOffset, buffer)` for any ArrayBufferView.
pub(super) unsafe fn buf_info(env: napi_env, value: napi_value) -> Status<(usize, usize, V)> {
    let value = get(value)?;
    if !is_array_buffer_view(env, value)? {
        return Err(NAPI_INVALID_ARG);
    }
    let view = value.as_object().ok_or(NAPI_INVALID_ARG)?;
    let byte_length = js(env, view.get("byteLength"))?;
    let byte_offset = js(env, view.get("byteOffset"))?;
    let buffer: V = js(env, view.get("buffer"))?;
    view_ptr(buffer.clone(), byte_offset, byte_length)?;
    Ok((byte_length, byte_offset, buffer))
}

pub(super) unsafe fn is_array_buffer_view(env: napi_env, value: &V) -> Status<bool> {
    let globals = ctx(env).globals();
    let array_buffer: Object = js(env, globals.get("ArrayBuffer"))?;
    let is_view: Function = js(env, array_buffer.get("isView"))?;
    js(env, is_view.call((value.clone(),)))
}

pub(super) unsafe fn view_info(
    env: napi_env,
    value: napi_value,
) -> Status<(u32, usize, usize, usize, V)> {
    let value = get(value)?;
    let kind = qjs::JS_GetTypedArrayType(value.as_raw());
    if kind < 0 {
        return Err(NAPI_INVALID_ARG);
    }

    let mut offset = 0;
    let mut byte_length = 0;
    let mut bytes_per_element = 0;
    let raw_buffer = qjs::JS_GetTypedArrayBuffer(
        ctx(env).as_raw().as_ptr(),
        value.as_raw(),
        &mut offset,
        &mut byte_length,
        &mut bytes_per_element,
    );
    if qjs::JS_IsException(raw_buffer) {
        (*env).exception = Some(extend(ctx(env).catch()));
        return Err(NAPI_PENDING_EXCEPTION);
    }
    if bytes_per_element == 0 {
        qjs::JS_FreeValue(ctx(env).as_raw().as_ptr(), raw_buffer);
        return Err(NAPI_GENERIC_FAILURE);
    }
    let buffer = extend(Value::from_raw(ctx(env), raw_buffer));
    let napi_kind = match kind as u32 {
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT8 => 0,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT8 => 1,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT8C => 2,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT16 => 3,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT16 => 4,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_INT32 => 5,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_UINT32 => 6,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_FLOAT32 => 7,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_FLOAT64 => 8,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_BIG_INT64 => 9,
        qjs::JSTypedArrayEnum_JS_TYPED_ARRAY_BIG_UINT64 => 10,
        _ => 1,
    };
    Ok((
        napi_kind,
        (byte_length / bytes_per_element) as usize,
        offset as usize,
        byte_length as usize,
        buffer,
    ))
}
