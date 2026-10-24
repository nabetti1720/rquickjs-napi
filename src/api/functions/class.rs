// SPDX-License-Identifier: MIT OR Apache-2.0
//! Class definition and instance construction APIs.
use super::*;
use rquickjs::Constructor;

#[no_mangle]
pub unsafe extern "C" fn napi_define_class(
    env: napi_env,
    utf8name: *const c_char,
    length: usize,
    constructor: Option<napi_callback>,
    data: *mut c_void,
    property_count: usize,
    properties: *const napi_property_descriptor,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let ctor = constructor.ok_or(NAPI_INVALID_ARG)?;
        if property_count > 0 && properties.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let name = cstr(utf8name, length)?;
        let c = ctx(env);
        let native = js(env, make_function(env, &c, &name, ctor, data))?;
        let class: Object = st(env, "defClass", (native, name))?;
        let prototype: Object = js(env, class.get("prototype"))?;
        for p in (0..property_count).map(|i| &*properties.add(i)) {
            let target = if p.attributes & NAPI_STATIC != 0 {
                &class
            } else {
                &prototype
            };
            apply_property(env, target, p)?;
        }
        out(result, push(env, extend(class.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_new_instance(
    env: napi_env,
    constructor: napi_value,
    argc: usize,
    argv: *const napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        if argc > 0 && argv.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let constructor = get(constructor)?;
        if !constructor.is_function() {
            return Err(NAPI_FUNCTION_EXPECTED);
        }
        let class: Constructor = js(env, constructor.get())?;
        let mut args = Args::new(ctx(env), argc);
        for i in 0..argc {
            js(env, args.push_arg(get(*argv.add(i))?.clone()))?;
        }
        let v: V = js(env, class.construct_args(args))?;
        out(result, push(env, v))
    })
}
