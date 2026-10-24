// SPDX-License-Identifier: MIT OR Apache-2.0
//! Async contexts, callback scopes and event-loop access.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_async_init(
    env: napi_env,
    _async_resource: napi_value,
    _async_resource_name: napi_value,
    result: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        out(result, Box::into_raw(Box::new(0u8)) as *mut c_void)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_async_destroy(env: napi_env, context: *mut c_void) -> napi_status {
    run(env, || {
        if !context.is_null() {
            drop(Box::from_raw(context as *mut u8));
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_make_callback(
    env: napi_env,
    _async_context: *mut c_void,
    recv: napi_value,
    func: napi_value,
    argc: usize,
    argv: *const napi_value,
    result: *mut napi_value,
) -> napi_status {
    napi_call_function(env, recv, func, argc, argv, result)
}

#[no_mangle]
pub unsafe extern "C" fn napi_open_callback_scope(
    env: napi_env,
    _resource_object: napi_value,
    _context: *mut c_void,
    result: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        out(result, Box::into_raw(Box::new(0u8)) as *mut c_void)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_close_callback_scope(
    env: napi_env,
    scope: *mut c_void,
) -> napi_status {
    run(env, || {
        if !scope.is_null() {
            drop(Box::from_raw(scope as *mut u8));
        }
        Ok(())
    })
}

/// There is no libuv loop to expose.
#[no_mangle]
pub unsafe extern "C" fn napi_get_uv_event_loop(
    env: napi_env,
    event_loop: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        if !event_loop.is_null() {
            *event_loop = ptr::null_mut();
        }
        Err(NAPI_GENERIC_FAILURE)
    })
}
