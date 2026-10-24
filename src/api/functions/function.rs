// SPDX-License-Identifier: MIT OR Apache-2.0
//! Function creation and callback APIs.
use super::*;

use std::mem;

struct CbInfo {
    this: napi_value,
    new_target: napi_value,
    args: Vec<napi_value>,
    data: *mut c_void,
}

pub(in crate::api) fn make_function<'js>(
    env: napi_env,
    ctx: &Ctx<'js>,
    name: &str,
    cb: napi_callback,
    data: *mut c_void,
) -> Result<Function<'js>> {
    let (env_addr, cb_addr, data_addr) = (env as usize, cb as usize, data as usize);
    let f = Function::new(
        ctx.clone(),
        move |ctx: Ctx<'js>,
              this: This<Value<'js>>,
              args: Rest<Value<'js>>|
              -> Result<Value<'js>> {
            let env = env_addr as napi_env;
            let cb: napi_callback = unsafe { mem::transmute(cb_addr) };
            let mark = unsafe { (*env).handles.len() };
            let state: Object<'js> = ctx.globals().get("__rquickjs_napi")?;
            let pending_target: Value<'js> = state.get("pendingNewTarget")?;
            state.set("pendingNewTarget", Value::new_undefined(ctx.clone()))?;
            let new_target = if pending_target.is_undefined() {
                ptr::null_mut()
            } else {
                push(env, unsafe { extend(pending_target) })
            };
            let this = push(env, unsafe { extend(this.0) });
            let args = args
                .0
                .into_iter()
                .map(|a| push(env, unsafe { extend(a) }))
                .collect();
            let mut info = CbInfo {
                this,
                new_target,
                args,
                data: data_addr as *mut c_void,
            };
            let ret = unsafe { cb(env, &mut info as *mut CbInfo as napi_callback_info) };
            let result = unsafe { ret.as_ref().map(|v| narrow(v.clone())) }
                .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
            let exception = unsafe {
                (*env).handles.truncate(mark);
                (*env).exception.take()
            };
            match exception {
                Some(e) => Err(ctx.throw(unsafe { narrow(e) })),
                None => Ok(result),
            }
        },
    )?;
    f.set_name(name)?;
    Ok(f)
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_function(
    env: napi_env,
    utf8name: *const c_char,
    length: usize,
    cb: Option<napi_callback>,
    data: *mut c_void,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let cb = cb.ok_or(NAPI_INVALID_ARG)?;
        let name = cstr(utf8name, length)?;
        let c = ctx(env);
        let f = js(env, make_function(env, &c, &name, cb, data))?;
        out(result, push(env, extend(f.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_cb_info(
    env: napi_env,
    cbinfo: napi_callback_info,
    argc: *mut usize,
    argv: *mut napi_value,
    this_arg: *mut napi_value,
    data: *mut *mut c_void,
) -> napi_status {
    run(env, || {
        let info = (cbinfo as *mut CbInfo).as_ref().ok_or(NAPI_INVALID_ARG)?;
        if !argc.is_null() {
            let capacity = *argc;
            if !argv.is_null() {
                for i in 0..capacity {
                    *argv.add(i) = match info.args.get(i) {
                        Some(a) => *a,
                        None => push(env, extend(Value::new_undefined(ctx(env)))),
                    };
                }
            }
            *argc = info.args.len();
        }
        if !this_arg.is_null() {
            *this_arg = info.this;
        }
        if !data.is_null() {
            *data = info.data;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_call_function(
    env: napi_env,
    recv: napi_value,
    func: napi_value,
    argc: usize,
    argv: *const napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let f = get(func)?.as_function().ok_or(NAPI_FUNCTION_EXPECTED)?;
        let mut args = Args::new(ctx(env), argc);
        let this = match recv.as_ref() {
            Some(v) => v.clone(),
            None => Value::new_undefined(ctx(env)),
        };
        js(env, args.this(this))?;
        for i in 0..argc {
            js(env, args.push_arg(get(*argv.add(i))?.clone()))?;
        }
        let v: V = js(env, f.call_arg(args))?;
        if !result.is_null() {
            *result = push(env, v);
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_new_target(
    env: napi_env,
    cbinfo: napi_callback_info,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let info = (cbinfo as *mut CbInfo).as_ref().ok_or(NAPI_INVALID_ARG)?;
        out(result, info.new_target)
    })
}
