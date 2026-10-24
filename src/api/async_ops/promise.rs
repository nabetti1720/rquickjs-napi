// SPDX-License-Identifier: MIT OR Apache-2.0
//! Promises and deferreds.
use super::*;
use rquickjs::Promise;

#[no_mangle]
pub unsafe extern "C" fn napi_create_promise(
    env: napi_env,
    deferred: *mut *mut c_void,
    promise: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        if deferred.is_null() || promise.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let id = next_id();
        let c = ctx(env);
        let (promise_value, resolve, reject) = js(env, Promise::new(&c))?;
        let p = extend(promise_value.into_value());
        st::<_, ()>(env, "defSet", (id as f64, resolve, reject))?;
        out(deferred, id as usize as *mut c_void)?;
        out(promise, push(env, p))
    })
}

pub(super) unsafe fn settle(
    env: napi_env,
    deferred: *mut c_void,
    ok: bool,
    value: napi_value,
) -> napi_status {
    run_js(env, || {
        if deferred.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        st::<_, ()>(
            env,
            "defSettle",
            (deferred as usize as f64, ok, get(value)?.clone()),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_resolve_deferred(
    env: napi_env,
    deferred: *mut c_void,
    resolution: napi_value,
) -> napi_status {
    settle(env, deferred, true, resolution)
}

#[no_mangle]
pub unsafe extern "C" fn napi_reject_deferred(
    env: napi_env,
    deferred: *mut c_void,
    rejection: napi_value,
) -> napi_status {
    settle(env, deferred, false, rejection)
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_promise(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || out(result, get(value)?.is_promise()))
}
