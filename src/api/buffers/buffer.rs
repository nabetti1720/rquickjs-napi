// SPDX-License-Identifier: MIT OR Apache-2.0
//! Buffer APIs.
use super::*;
use rquickjs::{Constructor, Function};
use std::slice;

const FALLBACK_BUFFERS_KEY: &str = "__rquickjs_napi_fallback_buffers__";

fn fallback_buffers(env: napi_env) -> Status<rquickjs::Object<'static>> {
    let globals = ctx(env).globals();
    let buffers: V = js(env, globals.get(FALLBACK_BUFFERS_KEY))?;
    if let Some(buffers) = buffers.as_object() {
        return Ok(buffers.clone());
    }

    let constructor: Constructor = js(env, globals.get("WeakSet"))?;
    let buffers: rquickjs::Object = js(env, constructor.construct(()))?;
    let descriptor: rquickjs::Object = js(env, rquickjs::Object::new(ctx(env)))?;
    js(env, descriptor.set("value", buffers.clone()))?;
    let object: rquickjs::Object = js(env, globals.get("Object"))?;
    let define: Function = js(env, object.get("defineProperty"))?;
    js(
        env,
        define.call::<_, V>((globals, FALLBACK_BUFFERS_KEY, descriptor)),
    )?;
    Ok(buffers)
}

fn call_weak_set_method(
    env: napi_env,
    set: &rquickjs::Object<'static>,
    method: &str,
    value: V,
) -> Status<V> {
    let method: Function = js(env, set.get(method))?;
    let mut args = Args::new(ctx(env), 1);
    js(env, args.this(set.clone().into_value()))?;
    js(env, args.push_arg(value))?;
    js(env, method.call_arg(args))
}

fn mark_fallback_buffer(env: napi_env, buffer: V) -> Status<V> {
    let set = fallback_buffers(env)?;
    call_weak_set_method(env, &set, "add", buffer.clone())?;
    Ok(buffer)
}

fn buffer_from_arraybuffer(env: napi_env, arraybuffer: V, length: usize) -> Status<V> {
    let c = ctx(env);
    let globals = c.globals();
    let buffer: Value = js(env, globals.get("Buffer"))?;
    if buffer.is_function() {
        let object = buffer.as_object().ok_or(NAPI_FUNCTION_EXPECTED)?;
        let from: Function = js(env, object.get("from"))?;
        let mut args = Args::new(c, 3);
        js(env, args.this(buffer))?;
        js(env, args.push_arg(arraybuffer))?;
        js(env, args.push_arg(0))?;
        js(env, args.push_arg(length as f64))?;
        js(env, from.call_arg(args))
    } else {
        let constructor: Constructor = js(env, globals.get("Uint8Array"))?;
        mark_fallback_buffer(
            env,
            js(env, constructor.construct((arraybuffer, 0, length as f64)))?,
        )
    }
}

use rquickjs::ArrayBuffer;

pub(super) unsafe fn make_buffer(
    env: napi_env,
    bytes: Vec<u8>,
    data: *mut *mut c_void,
    result: *mut napi_value,
) -> Status<()> {
    let len = bytes.len();
    let ab = js(env, ArrayBuffer::new(ctx(env), bytes))?;
    let ptr = ab.as_raw().map(|r| r.as_ptr() as *mut c_void);
    let buf = buffer_from_arraybuffer(env, extend(ab.into_value()), len)?;
    if !data.is_null() {
        *data = ptr.unwrap_or(ptr::null_mut());
    }
    out(result, push(env, buf))
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_buffer(
    env: napi_env,
    size: usize,
    data: *mut *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || make_buffer(env, vec![0u8; size], data, result))
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_buffer_copy(
    env: napi_env,
    length: usize,
    data: *const c_void,
    result_data: *mut *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let bytes = if length == 0 {
            Vec::new()
        } else if data.is_null() {
            return Err(NAPI_INVALID_ARG);
        } else {
            slice::from_raw_parts(data as *const u8, length).to_vec()
        };
        make_buffer(env, bytes, result_data, result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_external_buffer(
    env: napi_env,
    length: usize,
    data: *mut c_void,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ab = create_external_arraybuffer(env, data, length, finalize_cb, finalize_hint)?;
        let buf = buffer_from_arraybuffer(env, ab, length)?;
        out(result, push(env, buf))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_buffer_info(
    env: napi_env,
    value: napi_value,
    data: *mut *mut c_void,
    length: *mut usize,
) -> napi_status {
    run(env, || {
        if !is_buffer(env, get(value)?)? {
            return Err(NAPI_INVALID_ARG);
        }
        let (byte_length, offset, buffer) = buf_info(env, value)?;
        if !data.is_null() {
            *data = view_ptr(buffer, offset)?;
        }
        if !length.is_null() {
            *length = byte_length;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_buffer(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || out(result, is_buffer(env, get(value)?)?))
}

unsafe fn is_buffer(env: napi_env, value: &V) -> Status<bool> {
    let globals = ctx(env).globals();
    let buffer: V = js(env, globals.get("Buffer"))?;
    if let Some(buffer) = buffer.as_object() {
        if let Some(is_buffer) = js(env, buffer.get::<_, Option<Function>>("isBuffer"))? {
            return js(env, is_buffer.call((value.clone(),)));
        }
    }

    let buffers: V = js(env, globals.get(FALLBACK_BUFFERS_KEY))?;
    let Some(buffers) = buffers.as_object() else {
        return Ok(false);
    };
    Ok(call_weak_set_method(env, buffers, "has", value.clone())?.as_bool() == Some(true))
}
