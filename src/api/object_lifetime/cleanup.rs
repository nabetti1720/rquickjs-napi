// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleanup hooks for environment shutdown.
//!
//! Cleanup hooks are unsupported until environment teardown can run them safely.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_add_env_cleanup_hook(
    env: napi_env,
    _fun: *mut c_void,
    _arg: *mut c_void,
) -> napi_status {
    run(env, || Err(NAPI_GENERIC_FAILURE))
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_env_cleanup_hook(
    env: napi_env,
    _fun: *mut c_void,
    _arg: *mut c_void,
) -> napi_status {
    run(env, || Err(NAPI_GENERIC_FAILURE))
}

#[no_mangle]
pub unsafe extern "C" fn napi_add_async_cleanup_hook(
    env: napi_env,
    _hook: *mut c_void,
    _arg: *mut c_void,
    remove_handle: *mut *mut c_void,
) -> napi_status {
    let _ = remove_handle;
    run(env, || Err(NAPI_GENERIC_FAILURE))
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_async_cleanup_hook(handle: *mut c_void) -> napi_status {
    let _ = handle;
    NAPI_GENERIC_FAILURE
}
