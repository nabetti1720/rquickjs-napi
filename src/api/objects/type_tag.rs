// SPDX-License-Identifier: MIT OR Apache-2.0
//! Object type tags.
use super::*;

#[repr(C)]
pub struct napi_type_tag {
    lower: u64,
    upper: u64,
}

impl napi_type_tag {
    fn key(&self) -> String {
        format!("{}:{}", self.lower, self.upper)
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_type_tag_object(
    env: napi_env,
    value: napi_value,
    tag: *const napi_type_tag,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(value)?;
        let key = tag.as_ref().ok_or(NAPI_INVALID_ARG)?.key();
        if st::<_, bool>(env, "tagSet", (o.clone(), key))? {
            Ok(())
        } else {
            Err(NAPI_INVALID_ARG)
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_check_object_type_tag(
    env: napi_env,
    value: napi_value,
    tag: *const napi_type_tag,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(value)?;
        let key = tag.as_ref().ok_or(NAPI_INVALID_ARG)?.key();
        let found: Option<String> = st(env, "tagGet", (o.clone(),))?;
        out(result, found.as_deref() == Some(key.as_str()))
    })
}
