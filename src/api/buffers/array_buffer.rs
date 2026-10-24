// SPDX-License-Identifier: MIT OR Apache-2.0
//! ArrayBuffer APIs.
use super::*;

use rquickjs::{ArrayBuffer, ArrayBufferSource};

struct ExternalArrayBuffer {
    data: usize,
    length: usize,
    env: usize,
    finalize_data: usize,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: usize,
}

unsafe impl ArrayBufferSource for ExternalArrayBuffer {
    fn as_ptr(&self) -> *mut u8 {
        self.data as *mut u8
    }

    fn len(&self) -> usize {
        self.length
    }
}

impl Drop for ExternalArrayBuffer {
    fn drop(&mut self) {
        if let Some(finalize_cb) = self.finalize_cb {
            unsafe {
                finalize_cb(
                    self.env as napi_env,
                    self.finalize_data as *mut c_void,
                    self.finalize_hint as *mut c_void,
                );
            }
        }
    }
}

pub(super) unsafe fn create_external_arraybuffer(
    env: napi_env,
    data: *mut c_void,
    byte_length: usize,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
) -> Status<V> {
    if byte_length > 0 && data.is_null() {
        return Err(NAPI_INVALID_ARG);
    }
    let backing = ExternalArrayBuffer {
        data: if data.is_null() {
            ptr::NonNull::<u8>::dangling().as_ptr() as usize
        } else {
            data as usize
        },
        length: byte_length,
        env: env as usize,
        finalize_data: data as usize,
        finalize_cb,
        finalize_hint: finalize_hint as usize,
    };
    let ab = js(env, ArrayBuffer::from_source(ctx(env), backing))?;
    Ok(extend(ab.into_value()))
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_arraybuffer(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        out(
            result,
            ArrayBuffer::from_value(get(value)?.clone()).is_some(),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_arraybuffer(
    env: napi_env,
    byte_length: usize,
    data: *mut *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ab = js(env, ArrayBuffer::new(ctx(env), vec![0u8; byte_length]))?;
        if !data.is_null() {
            *data = ab
                .as_raw()
                .map_or(ptr::null_mut(), |r| r.as_ptr() as *mut u8 as *mut c_void);
        }
        out(result, push(env, extend(ab.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_external_arraybuffer(
    env: napi_env,
    data: *mut c_void,
    byte_length: usize,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ab = create_external_arraybuffer(env, data, byte_length, finalize_cb, finalize_hint)?;
        out(result, push(env, ab))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_arraybuffer_info(
    env: napi_env,
    arraybuffer: napi_value,
    data: *mut *mut c_void,
    byte_length: *mut usize,
) -> napi_status {
    run(env, || {
        let ab = ArrayBuffer::from_value(get(arraybuffer)?.clone()).ok_or(NAPI_INVALID_ARG)?;
        let raw = ab.as_raw();
        if !data.is_null() {
            *data = raw.map_or(ptr::null_mut(), |r| r.as_ptr() as *mut u8 as *mut c_void);
        }
        if !byte_length.is_null() {
            *byte_length = raw.map_or(0, |r| r.len());
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_detach_arraybuffer(
    env: napi_env,
    arraybuffer: napi_value,
) -> napi_status {
    run(env, || {
        let mut ab = ArrayBuffer::from_value(get(arraybuffer)?.clone()).ok_or(NAPI_INVALID_ARG)?;
        ab.detach();
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_detached_arraybuffer(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        let detached =
            ArrayBuffer::from_value(get(value)?.clone()).is_some_and(|ab| ab.as_raw().is_none());
        out(result, detached)
    })
}
