// SPDX-License-Identifier: MIT OR Apache-2.0
//! Script execution.
use super::*;

#[no_mangle]
pub unsafe extern "C" fn napi_run_script(
    env: napi_env,
    script: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let s = get(script)?.as_string().ok_or(NAPI_STRING_EXPECTED)?;
        let source = js(env, s.to_string())?;
        let v: V = js(env, ctx(env).eval(source))?;
        out(result, push(env, v))
    })
}
