// SPDX-License-Identifier: MIT OR Apache-2.0
//! Objects and arrays.
use super::*;

use rquickjs::{Filter, String as JsString};

fn to_property_atom(env: napi_env, key: &V) -> Status<qjs::JSAtom> {
    let c = ctx(env);
    let atom = unsafe { qjs::JS_ValueToAtom(c.as_raw().as_ptr(), key.as_raw()) };
    if atom == qjs::JS_ATOM_NULL {
        unsafe {
            (*env).exception = Some(extend(c.catch()));
        }
        Err(NAPI_PENDING_EXCEPTION)
    } else {
        Ok(atom)
    }
}

fn object_keys(env: napi_env, object: &Object<'static>, symbols: bool) -> Status<Vec<V>> {
    let filter = if symbols {
        Filter::new().string().symbol()
    } else {
        Filter::new().string()
    };
    js(env, object.own_keys::<V>(filter).collect())
}

fn own_property_descriptor(
    env: napi_env,
    object: &Object<'static>,
    key: &V,
) -> Status<Option<Object<'static>>> {
    let object_constructor: Object = js(env, ctx(env).globals().get("Object"))?;
    let get_descriptor: Function = js(env, object_constructor.get("getOwnPropertyDescriptor"))?;
    js(env, get_descriptor.call((object.clone(), key.clone())))
}

fn was_seen(env: napi_env, keys: &[V], key: &V) -> bool {
    let raw_ctx = ctx(env).as_raw().as_ptr();
    keys.iter()
        .any(|seen| unsafe { qjs::JS_IsStrictEqual(raw_ctx, seen.as_raw(), key.as_raw()) })
}

fn enumerable_property_names(env: napi_env, object: &Object<'static>) -> Status<V> {
    let mut keys = Vec::new();
    let result = js(env, Array::new(ctx(env)))?;
    let mut length = 0usize;
    let mut prototype = Some(object.clone());

    while let Some(current) = prototype {
        for key in object_keys(env, &current, false)? {
            if was_seen(env, &keys, &key) {
                continue;
            }
            keys.push(key.clone());
            let Some(descriptor) = own_property_descriptor(env, &current, &key)? else {
                continue;
            };
            if js(env, descriptor.get::<_, bool>("enumerable"))? {
                js(env, result.set(length, key))?;
                length += 1;
            }
        }
        prototype = current.get_prototype();
    }

    Ok(unsafe { extend(result.into_value()) })
}

fn all_property_names(
    env: napi_env,
    object: &Object<'static>,
    own_only: bool,
    filter: c_int,
    conversion: c_int,
) -> Status<V> {
    let mut seen = Vec::new();
    let result = js(env, Array::new(ctx(env)))?;
    let mut length = 0usize;
    let mut prototype = Some(object.clone());

    while let Some(current) = prototype {
        for key in object_keys(env, &current, true)? {
            if was_seen(env, &seen, &key) {
                continue;
            }
            seen.push(key.clone());

            let symbol = key.is_symbol();
            if (symbol && filter & 16 != 0) || (!symbol && filter & 8 != 0) {
                continue;
            }
            let Some(descriptor) = own_property_descriptor(env, &current, &key)? else {
                continue;
            };
            if (filter & 1 != 0 && !js(env, descriptor.get::<_, bool>("writable"))?)
                || (filter & 2 != 0 && !js(env, descriptor.get::<_, bool>("enumerable"))?)
                || (filter & 4 != 0 && !js(env, descriptor.get::<_, bool>("configurable"))?)
            {
                continue;
            }

            let key = if conversion == 0 && !symbol {
                let string = key.as_string().ok_or(NAPI_GENERIC_FAILURE)?;
                let string = js(env, string.to_string())?;
                match string.parse::<u32>() {
                    Ok(index) if index != u32::MAX && index.to_string() == string => {
                        Value::new_float(ctx(env), index as f64)
                    }
                    _ => key,
                }
            } else {
                key
            };
            js(env, result.set(length, key))?;
            length += 1;
        }

        if own_only {
            break;
        }
        prototype = current.get_prototype();
    }

    Ok(unsafe { extend(result.into_value()) })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_object(env: napi_env, result: *mut napi_value) -> napi_status {
    run(env, || {
        let o = js(env, Object::new(ctx(env)))?;
        out(result, push(env, extend(o.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_array(env: napi_env, result: *mut napi_value) -> napi_status {
    run(env, || {
        let a = js(env, Array::new(ctx(env)))?;
        out(result, push(env, extend(a.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_array_with_length(
    env: napi_env,
    length: usize,
    result: *mut napi_value,
) -> napi_status {
    run(env, || {
        let a = js(env, Array::new(ctx(env)))?;
        js(env, a.as_object().set("length", length as u32))?;
        out(result, push(env, extend(a.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_is_array(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    run(env, || out(result, get(value)?.is_array()))
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_array_length(
    env: napi_env,
    value: napi_value,
    result: *mut u32,
) -> napi_status {
    run(env, || {
        let a = get(value)?.as_array().ok_or(NAPI_ARRAY_EXPECTED)?;
        out(result, a.len() as u32)
    })
}

pub(in crate::api) unsafe fn object_arg<'a>(v: napi_value) -> Status<&'a Object<'static>> {
    get(v)?.as_object().ok_or(NAPI_OBJECT_EXPECTED)
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_property(
    env: napi_env,
    object: napi_value,
    key: napi_value,
    value: napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        js(env, o.set(get(key)?.clone(), get(value)?.clone()))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_property(
    env: napi_env,
    object: napi_value,
    key: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let v: V = js(env, o.get(get(key)?.clone()))?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_named_property(
    env: napi_env,
    object: napi_value,
    name: *const c_char,
    value: napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let name = cstr(name, NAPI_AUTO_LENGTH)?;
        js(env, o.set(name, get(value)?.clone()))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_named_property(
    env: napi_env,
    object: napi_value,
    name: *const c_char,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let name = cstr(name, NAPI_AUTO_LENGTH)?;
        let v: V = js(env, o.get(name))?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_has_named_property(
    env: napi_env,
    object: napi_value,
    name: *const c_char,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let name = cstr(name, NAPI_AUTO_LENGTH)?;
        out(result, js(env, o.contains_key(name))?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_set_element(
    env: napi_env,
    object: napi_value,
    index: u32,
    value: napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        js(env, o.set(index, get(value)?.clone()))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_element(
    env: napi_env,
    object: napi_value,
    index: u32,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let v: V = js(env, o.get(index))?;
        out(result, push(env, v))
    })
}

pub(in crate::api) unsafe fn apply_property(
    env: napi_env,
    target: &Object<'static>,
    p: &napi_property_descriptor,
) -> Status<()> {
    let c = ctx(env);
    let (key, name): (V, String) = if p.utf8name.is_null() {
        (get(p.name)?.clone(), String::new())
    } else {
        let name = cstr(p.utf8name, NAPI_AUTO_LENGTH)?;
        let key = js(env, JsString::from_str(c.clone(), &name))?;
        (extend(key.into_value()), name)
    };
    let desc = js(env, Object::new(c.clone()))?;
    if p.getter.is_some() || p.setter.is_some() {
        if let Some(g) = p.getter {
            let f = js(env, make_function(env, &c, &name, g, p.data))?;
            js(env, desc.set("get", f))?;
        }
        if let Some(s) = p.setter {
            let f = js(env, make_function(env, &c, &name, s, p.data))?;
            js(env, desc.set("set", f))?;
        }
    } else {
        let value: V = if let Some(m) = p.method {
            extend(js(env, make_function(env, &c, &name, m, p.data))?.into_value())
        } else {
            get(p.value)?.clone()
        };
        js(env, desc.set("value", value))?;
        js(env, desc.set("writable", p.attributes & 1 != 0))?;
    }
    js(env, desc.set("enumerable", p.attributes & 2 != 0))?;
    js(env, desc.set("configurable", p.attributes & 4 != 0))?;
    let object_ctor: Object = js(env, c.globals().get("Object"))?;
    let define: Function = js(env, object_ctor.get("defineProperty"))?;
    js(env, define.call::<_, Value>((target.clone(), key, desc)))?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn napi_define_properties(
    env: napi_env,
    object: napi_value,
    count: usize,
    props: *const napi_property_descriptor,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        if count > 0 && props.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        for p in (0..count).map(|i| &*props.add(i)) {
            apply_property(env, o, p)?;
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_prototype(
    env: napi_env,
    object: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let c = ctx(env);
        let raw = qjs::JS_GetPrototype(c.as_raw().as_ptr(), get(object)?.as_raw());
        if qjs::JS_IsException(raw) {
            (*env).exception = Some(extend(c.catch()));
            return Err(NAPI_PENDING_EXCEPTION);
        }
        let v = extend(Value::from_raw(c, raw));
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_has_own_property(
    env: napi_env,
    object: napi_value,
    key: napi_value,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let k = get(key)?;
        if !k.is_string() && !k.is_symbol() {
            return Err(4); // napi_name_expected
        }
        let object = object_arg(object)?;
        let atom = to_property_atom(env, k)?;
        let c = ctx(env);
        let mut descriptor = qjs::JSPropertyDescriptor {
            flags: 0,
            value: qjs::JS_UNDEFINED,
            getter: qjs::JS_UNDEFINED,
            setter: qjs::JS_UNDEFINED,
        };
        let found =
            qjs::JS_GetOwnProperty(c.as_raw().as_ptr(), &mut descriptor, object.as_raw(), atom);
        qjs::JS_FreeAtom(c.as_raw().as_ptr(), atom);
        if found < 0 {
            (*env).exception = Some(extend(c.catch()));
            return Err(NAPI_PENDING_EXCEPTION);
        }
        if found > 0 {
            qjs::JS_FreeValue(c.as_raw().as_ptr(), descriptor.value);
            qjs::JS_FreeValue(c.as_raw().as_ptr(), descriptor.getter);
            qjs::JS_FreeValue(c.as_raw().as_ptr(), descriptor.setter);
        }
        out(result, found > 0)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_property_names(
    env: napi_env,
    object: napi_value,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let object = object_arg(object)?;
        let v = enumerable_property_names(env, object)?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_all_property_names(
    env: napi_env,
    object: napi_value,
    key_mode: c_int,
    key_filter: c_int,
    key_conversion: c_int,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let v = all_property_names(env, o, key_mode == 1, key_filter, key_conversion)?;
        out(result, push(env, v))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_has_property(
    env: napi_env,
    object: napi_value,
    key: napi_value,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let key = get(key)?;
        let atom = to_property_atom(env, key)?;
        let qjs_result = qjs::JS_HasProperty(ctx(env).as_raw().as_ptr(), o.as_raw(), atom);
        qjs::JS_FreeAtom(ctx(env).as_raw().as_ptr(), atom);
        out(result, qjs_status(env, qjs_result)?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_property(
    env: napi_env,
    object: napi_value,
    key: napi_value,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let key = get(key)?;
        let atom = to_property_atom(env, key)?;
        let qjs_result = qjs::JS_DeleteProperty(ctx(env).as_raw().as_ptr(), o.as_raw(), atom, 0);
        qjs::JS_FreeAtom(ctx(env).as_raw().as_ptr(), atom);
        let r = qjs_status(env, qjs_result)?;
        if result.is_null() {
            return Ok(());
        }
        out(result, r)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_has_element(
    env: napi_env,
    object: napi_value,
    index: u32,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let key = Value::new_float(ctx(env), index as f64);
        let atom = to_property_atom(env, &key)?;
        let qjs_result = qjs::JS_HasProperty(ctx(env).as_raw().as_ptr(), o.as_raw(), atom);
        qjs::JS_FreeAtom(ctx(env).as_raw().as_ptr(), atom);
        out(result, qjs_status(env, qjs_result)?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_element(
    env: napi_env,
    object: napi_value,
    index: u32,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let o = object_arg(object)?;
        let key = Value::new_float(ctx(env), index as f64);
        let atom = to_property_atom(env, &key)?;
        let qjs_result = qjs::JS_DeleteProperty(ctx(env).as_raw().as_ptr(), o.as_raw(), atom, 0);
        qjs::JS_FreeAtom(ctx(env).as_raw().as_ptr(), atom);
        let r = qjs_status(env, qjs_result)?;
        if result.is_null() {
            return Ok(());
        }
        out(result, r)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_instanceof(
    env: napi_env,
    object: napi_value,
    constructor: napi_value,
    result: *mut bool,
) -> napi_status {
    run_js(env, || {
        let c = get(constructor)?;
        if !c.is_function() {
            return Err(NAPI_FUNCTION_EXPECTED);
        }
        let r = qjs::JS_IsInstanceOf(
            ctx(env).as_raw().as_ptr(),
            get(object)?.as_raw(),
            c.as_raw(),
        );
        out(result, qjs_status(env, r)?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_object_freeze(env: napi_env, object: napi_value) -> napi_status {
    run_js(env, || {
        qjs_status(
            env,
            qjs::JS_FreezeObject(ctx(env).as_raw().as_ptr(), object_arg(object)?.as_raw()),
        )?;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_object_seal(env: napi_env, object: napi_value) -> napi_status {
    run_js(env, || {
        qjs_status(
            env,
            qjs::JS_SealObject(ctx(env).as_raw().as_ptr(), object_arg(object)?.as_raw()),
        )?;
        Ok(())
    })
}
