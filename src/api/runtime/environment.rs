// SPDX-License-Identifier: MIT OR Apache-2.0
//! Environment instance data.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_set_instance_data(
    env: napi_env,
    data: *mut c_void,
    _finalize_cb: *mut c_void,
    _finalize_hint: *mut c_void,
) -> napi_status {
    run(env, || {
        (*env).instance_data = data;
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
