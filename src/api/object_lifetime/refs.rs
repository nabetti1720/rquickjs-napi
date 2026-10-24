// SPDX-License-Identifier: MIT OR Apache-2.0
//! References to JavaScript values.
use super::*;

pub struct RefInfo {
    pub(super) id: u64,
    pub(super) count: u32,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_reference(
    env: napi_env,
    value: napi_value,
    initial_refcount: u32,
    result: *mut *mut RefInfo,
) -> napi_status {
    run(env, || {
        let id = next_id();
        st::<_, ()>(
            env,
            "refNew",
            (id as f64, get(value)?.clone(), initial_refcount > 0),
        )?;
        out(
            result,
            Box::into_raw(Box::new(RefInfo {
                id,
                count: initial_refcount,
            })),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_reference(env: napi_env, r: *mut RefInfo) -> napi_status {
    run(env, || {
        if r.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let info = Box::from_raw(r);
        st::<_, ()>(env, "refDel", (info.id as f64,))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_reference_ref(
    env: napi_env,
    r: *mut RefInfo,
    result: *mut u32,
) -> napi_status {
    run(env, || {
        let info = r.as_mut().ok_or(NAPI_INVALID_ARG)?;
        info.count += 1;
        if info.count == 1 {
            st::<_, ()>(env, "refMode", (info.id as f64, true))?;
        }
        if !result.is_null() {
            *result = info.count;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_reference_unref(
    env: napi_env,
    r: *mut RefInfo,
    result: *mut u32,
) -> napi_status {
    run(env, || {
        let info = r.as_mut().ok_or(NAPI_INVALID_ARG)?;
        if info.count == 0 {
            return Err(NAPI_GENERIC_FAILURE);
        }
        info.count -= 1;
        if info.count == 0 {
            st::<_, ()>(env, "refMode", (info.id as f64, false))?;
        }
        if !result.is_null() {
            *result = info.count;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_reference_value(
    env: napi_env,
    r: *mut RefInfo,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let info = r.as_ref().ok_or(NAPI_INVALID_ARG)?;
        let v: V = st(env, "refGet", (info.id as f64,))?;
        out(
            result,
            if v.is_undefined() {
                ptr::null_mut()
            } else {
                push(env, v)
            },
        )
    })
}

pub(super) unsafe fn new_ref(env: napi_env, value: &V, count: u32) -> Status<*mut RefInfo> {
    let id = next_id();
    st::<_, ()>(env, "refNew", (id as f64, value.clone(), count > 0))?;
    Ok(Box::into_raw(Box::new(RefInfo { id, count })))
}
