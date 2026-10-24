// SPDX-License-Identifier: MIT OR Apache-2.0
//! JS-side state object shared by the other N-API extension modules.
use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

use rquickjs::{function::IntoArgs, object::Property, FromJs};

pub(super) const STATE_JS: &str = r#"(onFinalize) => {
  const refs = new Map(), defs = new Map(), wraps = new WeakMap(), tags = new WeakMap();
  const fr = new FinalizationRegistry(onFinalize);
  const isObj = (v) => (typeof v === 'object' && v !== null) || typeof v === 'function';
  const state = {
    pendingNewTarget: undefined,
    refNew(id, v, strong) { refs.set(id, strong || !isObj(v) ? { v } : { w: new WeakRef(v) }); },
    refMode(id, strong) {
      const e = refs.get(id);
      if (!e) return;
      if (strong && e.w) { const v = e.w.deref(); if (v !== undefined) refs.set(id, { v }); }
      else if (!strong && 'v' in e && isObj(e.v)) refs.set(id, { w: new WeakRef(e.v) });
    },
    refGet(id) { const e = refs.get(id); return e ? ('v' in e ? e.v : e.w.deref()) : undefined; },
    refDel(id) { refs.delete(id); },
    defSet(id, resolve, reject) { defs.set(id, [resolve, reject]); },
    defSettle(id, ok, v) { const d = defs.get(id); defs.delete(id); if (d) d[ok ? 0 : 1](v); },
    wrapSet(o, id) { if (wraps.has(o)) return false; wraps.set(o, id); return true; },
    wrapGet(o) { return wraps.get(o); },
    wrapDel(o) { wraps.delete(o); },
    finReg(o, id) { fr.register(o, id); },
    defClass(native, name) {
      const C = function (...a) {
        if (new.target === undefined) throw new TypeError("Class constructor " + name + " cannot be invoked without 'new'");
        state.pendingNewTarget = new.target;
        const r = native.apply(this, a);
        return isObj(r) ? r : this;
      };
      Object.defineProperty(C, 'name', { value: name, configurable: true });
      return C;
    },
    tagSet(o, k) { if (tags.has(o)) return false; tags.set(o, k); return true; },
    tagGet(o) { return tags.get(o); },
    fatal(e) { setTimeout(() => { throw e; }, 0); },
  };
  return state;
}"#;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub(super) fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed) as u64
}

pub(super) fn state(env: napi_env) -> Status<Object<'static>> {
    let c = ctx(env);
    let globals = c.globals();
    if let Ok(o) = globals.get::<_, Object>("__rquickjs_napi") {
        return Ok(o);
    }
    let make: Function = js(env, c.eval(STATE_JS))?;
    let on_finalize = js(
        env,
        Function::new(c.clone(), |id: f64| run_finalizer(id as u64)),
    )?;
    let obj: Object = js(env, make.call((on_finalize,)))?;
    let ext_proto: Object = js(env, Object::new_proto(c.clone(), None))?;
    js(env, obj.set("extProto", ext_proto))?;
    js(
        env,
        globals.prop("__rquickjs_napi", Property::from(obj.clone())),
    )?;
    Ok(obj)
}

/// Calls a helper function of the JS-side state object.
pub(super) fn st<A, R>(env: napi_env, name: &str, args: A) -> Status<R>
where
    A: IntoArgs<'static>,
    R: FromJs<'static>,
{
    let f: Function = js(env, state(env)?.get(name))?;
    js(env, f.call::<A, R>(args))
}
