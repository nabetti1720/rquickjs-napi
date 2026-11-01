// SPDX-License-Identifier: MIT OR Apache-2.0
//! Environment instance data.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_set_instance_data(
    env: napi_env,
    data: *mut c_void,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
) -> napi_status {
    run(env, || {
        (*env).instance_data = data;
        (*env).instance_finalize_cb = finalize_cb;
        (*env).instance_finalize_hint = finalize_hint as usize;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_instance_data(
    env: napi_env,
    data: *mut *mut c_void,
) -> napi_status {
    run(env, || out(data, (*env).instance_data))
}
