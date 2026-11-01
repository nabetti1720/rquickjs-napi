// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleanup hooks for environment shutdown.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_add_env_cleanup_hook(
    env: napi_env,
    fun: *mut c_void,
    arg: *mut c_void,
) -> napi_status {
    run(env, || {
        if fun.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let mut hooks = (*env)
            .cleanup_hooks
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?;
        let hook = CleanupHook {
            callback: fun as usize,
            arg: arg as usize,
        };
        if hooks
            .iter()
            .any(|registered| registered.callback == hook.callback && registered.arg == hook.arg)
        {
            return Err(NAPI_INVALID_ARG);
        }
        hooks.push(hook);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_env_cleanup_hook(
    env: napi_env,
    fun: *mut c_void,
    arg: *mut c_void,
) -> napi_status {
    run(env, || {
        if fun.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let mut hooks = (*env)
            .cleanup_hooks
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?;
        if let Some(position) = hooks
            .iter()
            .rposition(|hook| hook.callback == fun as usize && hook.arg == arg as usize)
        {
            hooks.remove(position);
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_add_async_cleanup_hook(
    env: napi_env,
    hook: *mut c_void,
    _arg: *mut c_void,
    _remove_handle: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        if hook.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        Err(NAPI_GENERIC_FAILURE)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_remove_async_cleanup_hook(handle: *mut c_void) -> napi_status {
    if handle.is_null() {
        NAPI_INVALID_ARG
    } else {
        NAPI_GENERIC_FAILURE
    }
}
