// SPDX-License-Identifier: MIT OR Apache-2.0
//! Handle scopes and escaped values.
use super::*;

pub struct Scope {
    mark: usize,
    // Slot reserved before the scope so an escaped value outlives it.
    escape_slot: Option<*mut V>,
    escaped: bool,
}

pub(super) unsafe fn open_scope(
    env: napi_env,
    escapable: bool,
    result: *mut *mut Scope,
) -> napi_status {
    run(env, || {
        let escape_slot = escapable.then(|| push(env, extend(Value::new_undefined(ctx(env)))));
        let scope = Box::new(Scope {
            mark: (*env).handles.len(),
            escape_slot,
            escaped: false,
        });
        out(result, Box::into_raw(scope))
    })
}

pub(super) unsafe fn close_scope(env: napi_env, scope: *mut Scope) -> napi_status {
    run(env, || {
        if scope.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let scope = Box::from_raw(scope);
        (*env).handles.truncate(scope.mark);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_open_handle_scope(
    env: napi_env,
    result: *mut *mut Scope,
) -> napi_status {
    open_scope(env, false, result)
}

#[no_mangle]
pub unsafe extern "C" fn napi_close_handle_scope(env: napi_env, scope: *mut Scope) -> napi_status {
    close_scope(env, scope)
}

#[no_mangle]
pub unsafe extern "C" fn napi_open_escapable_handle_scope(
    env: napi_env,
    result: *mut *mut Scope,
) -> napi_status {
    open_scope(env, true, result)
}

#[no_mangle]
pub unsafe extern "C" fn napi_close_escapable_handle_scope(
    env: napi_env,
    scope: *mut Scope,
) -> napi_status {
    close_scope(env, scope)
}

#[no_mangle]
pub unsafe extern "C" fn napi_escape_handle(
    env: napi_env,
    scope: *mut Scope,
    escapee: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let scope = scope.as_mut().ok_or(NAPI_INVALID_ARG)?;
        if scope.escaped {
            return Err(NAPI_ESCAPE_CALLED_TWICE);
        }
        let slot = scope.escape_slot.ok_or(NAPI_INVALID_ARG)?;
        *slot = get(escapee)?.clone();
        scope.escaped = true;
        out(result, slot)
    })
}
