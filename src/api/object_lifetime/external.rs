// SPDX-License-Identifier: MIT OR Apache-2.0
//! External values carrying opaque native pointers.
use super::*;

/// Whether `v` was created by `napi_create_external`.
pub(in crate::api) fn is_external(env: napi_env, v: &V) -> bool {
    let Some(obj) = v.as_object().filter(|o| !o.is_function()) else {
        return false;
    };
    let Some(proto) = obj.get_prototype() else {
        return false;
    };
    state(env)
        .and_then(|s| js(env, s.get::<_, Object>("extProto")))
        .is_ok_and(|ext| ext == proto)
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_external(
    env: napi_env,
    data: *mut c_void,
    finalize_cb: Option<napi_finalize>,
    finalize_hint: *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ext_proto: Object = js(env, state(env)?.get("extProto"))?;
        let obj = extend(js(env, Object::new_proto(ctx(env), Some(&ext_proto)))?.into_value());
        let id = next_id();
        st::<_, bool>(env, "wrapSet", (obj.clone(), id as f64))?;
        WRAPS
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?
            .insert(id, data as usize);
        if let Some(cb) = finalize_cb {
            register_finalizer(env, &obj, id, cb, data, finalize_hint)?;
        }
        out(result, push(env, obj))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_external(
    env: napi_env,
    value: napi_value,
    result: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        if !is_external(env, get(value)?) {
            return Err(NAPI_INVALID_ARG);
        }
        let id = wrap_id(env, value)?;
        let data = WRAPS
            .lock()
            .map_err(|_| NAPI_GENERIC_FAILURE)?
            .get(&id)
            .copied();
        out(result, data.ok_or(NAPI_INVALID_ARG)? as *mut c_void)
    })
}
