// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native data wrapping and finalizers.
use super::*;

use std::{
    collections::HashMap,
    mem,
    sync::{LazyLock, Mutex},
};

pub(super) struct Finalizer {
    env: usize,
    cb: usize,
    data: usize,
    hint: usize,
}

pub(super) static FINALIZERS: LazyLock<Mutex<HashMap<u64, Finalizer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub(super) static WRAPS: LazyLock<Mutex<HashMap<u64, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(super) unsafe fn call_finalizer(f: Finalizer) {
    let env = f.env as napi_env;
    let cb: napi_finalize = mem::transmute(f.cb);
    let mark = (*env).handles.len();
    cb(env, f.data as *mut c_void, f.hint as *mut c_void);
    (*env).handles.truncate(mark);
    (*env).exception.take();
}

pub(in crate::api) fn run_finalizer(id: u64) {
    match WRAPS.lock() {
        Ok(mut wraps) => {
            wraps.remove(&id);
        }
        Err(poisoned) => {
            eprintln!("Recovering poisoned N-API wrap map");
            poisoned.into_inner().remove(&id);
        }
    };
    let finalizer = match FINALIZERS.lock() {
        Ok(mut finalizers) => finalizers.remove(&id),
        Err(poisoned) => {
            eprintln!("Recovering poisoned N-API finalizer map");
            poisoned.into_inner().remove(&id)
        }
    };
    if let Some(f) = finalizer {
        unsafe { call_finalizer(f) };
    }
}

pub(super) fn register_finalizer(
    env: napi_env,
    target: &V,
    id: u64,
    cb: napi_finalize,
    data: *mut c_void,
    hint: *mut c_void,
) -> Status<()> {
    FINALIZERS.lock().map_err(|_| NAPI_GENERIC_FAILURE)?.insert(
        id,
        Finalizer {
            env: env as usize,
            cb: cb as usize,
            data: data as usize,
            hint: hint as usize,
        },
    );
    st::<_, ()>(env, "finReg", (target.clone(), id as f64))
}

#[no_mangle]
pub unsafe extern "C" fn napi_wrap(
    env: napi_env,
    js_object: napi_value,
    native_object: *mut c_void,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
    result: *mut *mut RefInfo,
) -> napi_status {
    run(env, || {
        let o = get(js_object)?;
        if !o.is_object() {
            return Err(NAPI_OBJECT_EXPECTED);
        }
        let id = next_id();
        if !st::<_, bool>(env, "wrapSet", (o.clone(), id as f64))? {
            return Err(NAPI_INVALID_ARG);
        }
        WRAPS
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?
            .insert(id, native_object as usize);
        if let Some(cb) = finalize_cb {
            register_finalizer(env, o, id, cb, native_object, finalize_hint)?;
        }
        if !result.is_null() {
            *result = new_ref(env, o, 0)?;
        }
        Ok(())
    })
}

pub(super) unsafe fn wrap_id(env: napi_env, js_object: napi_value) -> Status<u64> {
    let o = get(js_object)?;
    if !o.is_object() {
        return Err(NAPI_OBJECT_EXPECTED);
    }
    let id: Option<f64> = st(env, "wrapGet", (o.clone(),))?;
    id.map(|i| i as u64).ok_or(NAPI_INVALID_ARG)
}

#[no_mangle]
pub unsafe extern "C" fn napi_unwrap(
    env: napi_env,
    js_object: napi_value,
    result: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        let id = wrap_id(env, js_object)?;
        let data = WRAPS
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?
            .get(&id)
            .copied();
        out(result, data.ok_or(NAPI_INVALID_ARG)? as *mut c_void)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_wrap(
    env: napi_env,
    js_object: napi_value,
    result: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        let id = wrap_id(env, js_object)?;
        let mut wraps = WRAPS.lock().map_err(|_| NAPI_GENERIC_FAILURE)?;
        let mut finalizers = FINALIZERS.lock().map_err(|_| NAPI_GENERIC_FAILURE)?;
        let data = wraps.remove(&id);
        finalizers.remove(&id);
        drop(finalizers);
        drop(wraps);
        st::<_, ()>(env, "wrapDel", (get(js_object)?.clone(),))?;
        if !result.is_null() {
            *result = data.unwrap_or(0) as *mut c_void;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_add_finalizer(
    env: napi_env,
    js_object: napi_value,
    finalize_data: *mut c_void,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
    result: *mut *mut RefInfo,
) -> napi_status {
    run(env, || {
        let o = get(js_object)?;
        if !o.is_object() {
            return Err(NAPI_OBJECT_EXPECTED);
        }
        let cb = finalize_cb.ok_or(NAPI_INVALID_ARG)?;
        register_finalizer(env, o, next_id(), cb, finalize_data, finalize_hint)?;
        if !result.is_null() {
            *result = new_ref(env, o, 0)?;
        }
        Ok(())
    })
}
