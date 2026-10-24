// SPDX-License-Identifier: MIT OR Apache-2.0
//! Errors and exceptions.
use super::*;

use std::process;

use rquickjs::{Constructor, String as JsString};

#[no_mangle]
pub unsafe extern "C" fn napi_get_last_error_info(
    env: napi_env,
    result: *mut *const napi_extended_error_info,
) -> napi_status {
    if env.is_null() || result.is_null() {
        return NAPI_INVALID_ARG;
    }
    *result = &(*env).last_error;
    NAPI_OK
}

pub(super) fn make_error(
    env: napi_env,
    class: &str,
    code: napi_value,
    msg: napi_value,
) -> Status<V> {
    unsafe {
        let c = ctx(env);
        let msg = get(msg)?.as_string().ok_or(NAPI_STRING_EXPECTED)?;
        let msg = js(env, msg.to_string())?;
        let ctor: Constructor = js(env, c.globals().get(class))?;
        let err: Object = js(env, ctor.construct((msg,)))?;
        if let Some(code) = code.as_ref() {
            if !code.is_undefined() {
                js(env, err.set("code", code.clone()))?;
            }
        }
        Ok(extend(err.into_value()))
    }
}

pub(super) fn throw_error_class(
    env: napi_env,
    class: &str,
    code: *const c_char,
    msg: *const c_char,
) -> napi_status {
    run(env, || unsafe {
        let c = ctx(env);
        let msg = cstr(msg, NAPI_AUTO_LENGTH)?;
        let msg = push(
            env,
            extend(js(env, JsString::from_str(c.clone(), &msg))?.into_value()),
        );
        let code = if code.is_null() {
            ptr::null_mut()
        } else {
            let code = cstr(code, NAPI_AUTO_LENGTH)?;
            push(
                env,
                extend(js(env, JsString::from_str(c, &code))?.into_value()),
            )
        };
        let err = make_error(env, class, code, msg)?;
        (*env).exception = Some(err);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_error(
    env: napi_env,
    code: *const c_char,
    msg: *const c_char,
) -> napi_status {
    throw_error_class(env, "Error", code, msg)
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_type_error(
    env: napi_env,
    code: *const c_char,
    msg: *const c_char,
) -> napi_status {
    throw_error_class(env, "TypeError", code, msg)
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw_range_error(
    env: napi_env,
    code: *const c_char,
    msg: *const c_char,
) -> napi_status {
    throw_error_class(env, "RangeError", code, msg)
}

#[no_mangle]
pub unsafe extern "C" fn napi_throw(env: napi_env, error: napi_value) -> napi_status {
    run(env, || {
        (*env).exception = Some(get(error)?.clone());
        Ok(())
    })
}

macro_rules! create_error_fn {
    ($name:ident, $class:literal) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            env: napi_env,
            code: napi_value,
            msg: napi_value,
            result: *mut napi_value,
        ) -> napi_status {
            run(env, || {
                let e = make_error(env, $class, code, msg)?;
                out(result, push(env, e))
            })
        }
    };
}

create_error_fn!(napi_create_error, "Error");
create_error_fn!(napi_create_type_error, "TypeError");
create_error_fn!(napi_create_range_error, "RangeError");

#[no_mangle]
pub unsafe extern "C" fn napi_is_error(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || {
        let error: V = js(env, ctx(env).globals().get("Error"))?;
        let instance = qjs::JS_IsInstanceOf(
            ctx(env).as_raw().as_ptr(),
            get(value)?.as_raw(),
            error.as_raw(),
        );
        out(result, qjs_status(env, instance)?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_exception_pending(
    env: napi_env,
    result: *mut bool,
) -> napi_status {
    run(env, || out(result, (*env).exception.is_some()))
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_and_clear_last_exception(
    env: napi_env,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let v = (*env)
            .exception
            .take()
            .unwrap_or_else(|| extend(Value::new_undefined(ctx(env))));
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_fatal_exception(env: napi_env, err: napi_value) -> napi_status {
    run_js(env, || st::<_, ()>(env, "fatal", (get(err)?.clone(),)))
}

#[no_mangle]
pub unsafe extern "C" fn napi_fatal_error(
    location: *const c_char,
    location_len: usize,
    message: *const c_char,
    message_len: usize,
) -> ! {
    let location = cstr(location, location_len).unwrap_or_default();
    let message = cstr(message, message_len).unwrap_or_default();
    eprintln!("FATAL ERROR: {location} {message}");
    process::abort()
}
