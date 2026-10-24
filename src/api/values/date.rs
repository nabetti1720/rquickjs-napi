// SPDX-License-Identifier: MIT OR Apache-2.0
//! Date values.
use super::*;
use rquickjs::{Constructor, Function};

const NAPI_DATE_EXPECTED: napi_status = 18;

#[no_mangle]
pub unsafe extern "C" fn napi_create_date(
    env: napi_env,
    time: f64,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let constructor: Constructor = js(env, ctx(env).globals().get("Date"))?;
        let v: V = js(env, constructor.construct((time,)))?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_date(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || out(result, qjs::JS_IsDate(get(value)?.as_raw())))
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_date_value(
    env: napi_env,
    value: napi_value,
    result: *mut f64,
) -> napi_status {
    run(env, || {
        let v = get(value)?;
        if !qjs::JS_IsDate(v.as_raw()) {
            return Err(NAPI_DATE_EXPECTED);
        }
        let object = v.as_object().ok_or(NAPI_DATE_EXPECTED)?;
        let value_of: Function = js(env, object.get("valueOf"))?;
        out(
            result,
            js(env, value_of.call::<_, f64>((This(v.clone()),)))?,
        )
    })
}
