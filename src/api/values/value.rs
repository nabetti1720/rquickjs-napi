// SPDX-License-Identifier: MIT OR Apache-2.0
//! Primitive values, type checks, symbols and coercions.
use super::*;
use rquickjs::{Coerced, Constructor, Symbol};

#[no_mangle]
pub unsafe extern "C" fn napi_get_undefined(env: napi_env, result: *mut napi_value) -> napi_status {
    run(env, || {
        out(result, push(env, extend(Value::new_undefined(ctx(env)))))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_null(env: napi_env, result: *mut napi_value) -> napi_status {
    run(env, || {
        out(result, push(env, extend(Value::new_null(ctx(env)))))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_global(env: napi_env, result: *mut napi_value) -> napi_status {
    run(env, || {
        out(result, push(env, extend(ctx(env).globals().into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_boolean(
    env: napi_env,
    value: bool,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        out(result, push(env, extend(Value::new_bool(ctx(env), value))))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_double(
    env: napi_env,
    value: f64,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        out(result, push(env, extend(Value::new_float(ctx(env), value))))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_int32(
    env: napi_env,
    value: i32,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        out(result, push(env, extend(Value::new_int(ctx(env), value))))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_uint32(
    env: napi_env,
    value: u32,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let v = match i32::try_from(value) {
            Ok(i) => Value::new_int(ctx(env), i),
            Err(_) => Value::new_float(ctx(env), value as f64),
        };
        out(result, push(env, extend(v)))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_int64(
    env: napi_env,
    value: i64,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        out(
            result,
            push(env, extend(Value::new_float(ctx(env), value as f64))),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_double(
    env: napi_env,
    value: napi_value,
    result: *mut f64,
) -> napi_status {
    run(env, || {
        let n = get(value)?.as_number().ok_or(NAPI_NUMBER_EXPECTED)?;
        out(result, n)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_int32(
    env: napi_env,
    value: napi_value,
    result: *mut i32,
) -> napi_status {
    run(env, || {
        let n = get(value)?.as_number().ok_or(NAPI_NUMBER_EXPECTED)?;
        out(result, if n.is_finite() { n as i64 as i32 } else { 0 })
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_uint32(
    env: napi_env,
    value: napi_value,
    result: *mut u32,
) -> napi_status {
    run(env, || {
        let n = get(value)?.as_number().ok_or(NAPI_NUMBER_EXPECTED)?;
        out(result, if n.is_finite() { n as i64 as u32 } else { 0 })
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_int64(
    env: napi_env,
    value: napi_value,
    result: *mut i64,
) -> napi_status {
    run(env, || {
        let n = get(value)?.as_number().ok_or(NAPI_NUMBER_EXPECTED)?;
        out(result, if n.is_finite() { n as i64 } else { 0 })
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bool(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        let b = get(value)?.as_bool().ok_or(NAPI_BOOLEAN_EXPECTED)?;
        out(result, b)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_typeof(
    env: napi_env,
    value: napi_value,
    result: *mut c_int,
) -> napi_status {
    run(env, || {
        let v = get(value)?;
        let t = if v.is_undefined() {
            0
        } else if v.is_null() {
            1
        } else if v.is_bool() {
            2
        } else if v.is_number() {
            3
        } else if v.is_string() {
            4
        } else if v.is_symbol() {
            5
        } else if v.is_function() {
            7
        } else if v.is_big_int() {
            9
        } else if v.is_object() {
            if is_external(env, v) {
                8
            } else {
                6
            }
        } else {
            return Err(NAPI_INVALID_ARG);
        };
        out(result, t)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_strict_equals(
    env: napi_env,
    lhs: napi_value,
    rhs: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        let left = get(lhs)?;
        let right = get(rhs)?;
        let eq = unsafe {
            qjs::JS_IsStrictEqual(ctx(env).as_raw().as_ptr(), left.as_raw(), right.as_raw())
        };
        out(result, eq)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_symbol(
    env: napi_env,
    description: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let v = match description.as_ref() {
            Some(d) => {
                if !d.is_string() {
                    return Err(NAPI_STRING_EXPECTED);
                }
                let description: String = js(env, d.get::<Coerced<String>>())?.0;
                js(env, Symbol::with_description(ctx(env), description))?.into_value()
            }
            None => js(env, Symbol::new(ctx(env)))?.into_value(),
        };
        out(result, push(env, extend(v)))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_bool(
    env: napi_env,
    value: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let value = get(value)?;
        let coerced: bool = js(env, value.get::<Coerced<bool>>())?.0;
        out(
            result,
            push(env, extend(Value::new_bool(ctx(env), coerced))),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_number(
    env: napi_env,
    value: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let value = get(value)?;
        let coerced: f64 = js(env, value.get::<Coerced<f64>>())?.0;
        out(
            result,
            push(env, extend(Value::new_float(ctx(env), coerced))),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_coerce_to_object(
    env: napi_env,
    value: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let constructor: Constructor = js(env, ctx(env).globals().get("Object"))?;
        let object: V = js(env, constructor.construct((get(value)?.clone(),)))?;
        out(result, push(env, object))
    })
}
