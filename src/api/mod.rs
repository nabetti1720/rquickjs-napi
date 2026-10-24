// SPDX-License-Identifier: MIT OR Apache-2.0
//! Node-API (N-API) implementation for loading `.node` addons.
//!
//! Core addon loading and shared N-API state live here. Implementations are grouped by
//! Node-API domain.
#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]

mod async_ops;
mod buffers;
mod error;
mod functions;
mod object_lifetime;
mod objects;
mod runtime;
mod script;
mod state;
mod values;

use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    mem, ptr, result, slice,
    sync::{atomic::Ordering, Mutex},
};

use libc::{dlerror, dlopen, dlsym, RTLD_LAZY, RTLD_LOCAL};
use rquickjs::{
    function::{Args, Rest, This},
    qjs, Array, Ctx, Error, Exception, Function, Object, Result, Value,
};

use self::{
    async_ops::*, buffers::*, error::*, functions::*, object_lifetime::*, objects::*, runtime::*,
    script::*, state::*, values::*,
};

type V = Value<'static>;
type napi_status = c_int;
type napi_env = *mut Env;
type napi_value = *mut V;
type napi_callback_info = *mut c_void;
type napi_callback = unsafe extern "C" fn(napi_env, napi_callback_info) -> napi_value;
type napi_finalize = unsafe extern "C" fn(napi_env, *mut c_void, *mut c_void);
type napi_async_execute = unsafe extern "C" fn(napi_env, *mut c_void);
type napi_async_complete = unsafe extern "C" fn(napi_env, napi_status, *mut c_void);
type napi_addon_init = unsafe extern "C" fn(napi_env, napi_value) -> napi_value;

const NAPI_OK: napi_status = 0;
const NAPI_INVALID_ARG: napi_status = 1;
const NAPI_OBJECT_EXPECTED: napi_status = 2;
const NAPI_STRING_EXPECTED: napi_status = 3;
const NAPI_FUNCTION_EXPECTED: napi_status = 5;
const NAPI_NUMBER_EXPECTED: napi_status = 6;
const NAPI_BOOLEAN_EXPECTED: napi_status = 7;
const NAPI_ARRAY_EXPECTED: napi_status = 8;
const NAPI_GENERIC_FAILURE: napi_status = 9;
const NAPI_PENDING_EXCEPTION: napi_status = 10;
const NAPI_ESCAPE_CALLED_TWICE: napi_status = 12;
const NAPI_STATIC: c_int = 1 << 10;

type Status<T> = result::Result<T, napi_status>;

const NAPI_AUTO_LENGTH: usize = usize::MAX;
static ADDON_LOAD_LOCK: Mutex<()> = Mutex::new(());

#[repr(C)]
pub struct napi_extended_error_info {
    error_message: *const c_char,
    engine_reserved: *mut c_void,
    engine_error_code: u32,
    error_code: napi_status,
}

#[repr(C)]
pub struct napi_property_descriptor {
    utf8name: *const c_char,
    name: napi_value,
    method: Option<napi_callback>,
    getter: Option<napi_callback>,
    setter: Option<napi_callback>,
    value: napi_value,
    attributes: c_int,
    data: *mut c_void,
}

pub struct Env {
    // Raw pointer: holding a `Ctx` would keep the context alive and leak at runtime shutdown.
    ctx: ptr::NonNull<qjs::JSContext>,
    // Boxed so that `napi_value` pointers stay stable when the vector grows.
    #[allow(clippy::vec_box)]
    handles: Vec<Box<V>>,
    exception: Option<V>,
    last_error: napi_extended_error_info,
    instance_data: *mut c_void,
}

unsafe fn extend<'js>(v: Value<'js>) -> V {
    mem::transmute::<Value<'js>, V>(v)
}

unsafe fn narrow<'js>(v: V) -> Value<'js> {
    mem::transmute::<V, Value<'js>>(v)
}

fn push(env: napi_env, v: V) -> napi_value {
    unsafe {
        let mut boxed = Box::new(v);
        let p: napi_value = &mut *boxed;
        (*env).handles.push(boxed);
        p
    }
}

fn ctx(env: napi_env) -> Ctx<'static> {
    unsafe { Ctx::from_raw((*env).ctx) }
}

/// Runs `f` and records the resulting status in the environment.
fn run(env: napi_env, f: impl FnOnce() -> Status<()>) -> napi_status {
    if env.is_null() {
        return NAPI_INVALID_ARG;
    }
    let status = f().err().unwrap_or(NAPI_OK);
    unsafe { (*env).last_error.error_code = status };
    status
}

/// Like `run`, but fails with `napi_pending_exception` if an exception is already pending.
fn run_js(env: napi_env, f: impl FnOnce() -> Status<()>) -> napi_status {
    if !env.is_null() && unsafe { (*env).exception.is_some() } {
        unsafe { (*env).last_error.error_code = NAPI_PENDING_EXCEPTION };
        return NAPI_PENDING_EXCEPTION;
    }
    run(env, f)
}

/// Converts a JS error into a pending exception on the environment.
fn js<T>(env: napi_env, r: Result<T>) -> Status<T> {
    r.map_err(|err| {
        let c = ctx(env);
        let exception = if matches!(err, Error::Exception) {
            c.catch()
        } else {
            Exception::from_message(c.clone(), &err.to_string())
                .map(|e| e.into_value())
                .unwrap_or_else(|_| Value::new_undefined(c))
        };
        unsafe { (*env).exception = Some(extend(exception)) };
        NAPI_PENDING_EXCEPTION
    })
}

fn qjs_status(env: napi_env, status: c_int) -> Status<bool> {
    if status < 0 {
        unsafe {
            (*env).exception = Some(extend(ctx(env).catch()));
        }
        Err(NAPI_PENDING_EXCEPTION)
    } else {
        Ok(status != 0)
    }
}

unsafe fn get<'a>(v: napi_value) -> Status<&'a V> {
    v.as_ref().ok_or(NAPI_INVALID_ARG)
}

unsafe fn out<T>(p: *mut T, v: T) -> Status<()> {
    if p.is_null() {
        return Err(NAPI_INVALID_ARG);
    }
    *p = v;
    Ok(())
}

unsafe fn cstr(p: *const c_char, len: usize) -> Status<String> {
    if p.is_null() {
        return if len == 0 {
            Ok(String::new())
        } else {
            Err(NAPI_INVALID_ARG)
        };
    }
    let bytes = if len == NAPI_AUTO_LENGTH {
        CStr::from_ptr(p).to_bytes()
    } else {
        slice::from_raw_parts(p as *const u8, len)
    };
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// Loads a native addon and returns its `module.exports`.
pub(crate) fn load_addon<'js>(ctx: &Ctx<'js>, path: &str) -> Result<Value<'js>> {
    runtime::retain_exported_symbols();

    let throw = |msg: String, code: &str| {
        Exception::from_message(ctx.clone(), &msg)
            .and_then(|e| {
                e.set("code", code)?;
                Ok(e.throw())
            })
            .unwrap_or_else(|e| e)
    };

    let c_path =
        CString::new(path).map_err(|_| throw("Invalid path".into(), "ERR_DLOPEN_FAILED"))?;

    let (_handle, init) = {
        let _load_guard = ADDON_LOAD_LOCK
            .lock()
            .map_err(|_| throw("Addon loader lock is poisoned".into(), "ERR_DLOPEN_FAILED"))?;
        PENDING_MODULE.store(ptr::null_mut(), Ordering::Relaxed);
        let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY | RTLD_LOCAL) };
        if handle.is_null() {
            let err = unsafe {
                let e = dlerror();
                if e.is_null() {
                    "unknown error".to_string()
                } else {
                    CStr::from_ptr(e).to_string_lossy().into_owned()
                }
            };
            return Err(throw(err, "ERR_DLOPEN_FAILED"));
        }

        let symbol = unsafe { dlsym(handle, c"napi_register_module_v1".as_ptr()) };
        let init: napi_addon_init = if !symbol.is_null() {
            unsafe { mem::transmute::<*mut c_void, napi_addon_init>(symbol) }
        } else {
            let module = PENDING_MODULE.load(Ordering::Acquire);
            match unsafe { module.as_ref().and_then(|m| m.nm_register_func) } {
                Some(init) => init,
                None => {
                    return Err(throw(
                        format!("Module did not self-register: '{}'.", path),
                        "ERR_DLOPEN_FAILED",
                    ))
                }
            }
        };
        (handle, init)
    };

    // The environment lives for as long as the addon's functions can be called.
    let env: napi_env = Box::into_raw(Box::new(Env {
        ctx: ctx.as_raw(),
        handles: Vec::new(),
        exception: None,
        instance_data: ptr::null_mut(),
        last_error: napi_extended_error_info {
            error_message: ptr::null(),
            engine_reserved: ptr::null_mut(),
            engine_error_code: 0,
            error_code: NAPI_OK,
        },
    }));

    let exports_value = Object::new(ctx.clone())?.into_value();
    let exports_handle = push(env, unsafe { extend(exports_value.clone()) });
    let ret = unsafe { init(env, exports_handle) };
    let result = unsafe { ret.as_ref().map(|v| narrow::<'js>(v.clone())) }.unwrap_or(exports_value);
    let exception = unsafe {
        (*env).handles.clear();
        (*env).exception.take()
    };
    match exception {
        Some(e) => Err(ctx.throw(unsafe { narrow(e) })),
        None => Ok(result),
    }
}
