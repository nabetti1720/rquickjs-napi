// SPDX-License-Identifier: MIT OR Apache-2.0
#[cfg(not(unix))]
use rquickjs::Exception;
#[cfg(unix)]
use rquickjs::Object;
use rquickjs::{
    loader::{ImportAttributes, Loader},
    Ctx, Error, Module, Result,
};
use tracing::trace;

const ADDON_EXT: &str = ".node";

/// Loads `.node` files as modules whose default export is the addon's `module.exports`.
/// Enumerable string keys that are valid identifiers are also exposed as named exports.
#[derive(Debug, Default)]
pub struct NapiLoader;

#[cfg(unix)]
fn is_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c == '$' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
        && key != "default"
}

#[cfg(unix)]
fn load_module<'js>(ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
    let exports = crate::api::load_addon(ctx, name)?;

    let mut source = String::from("const __addon = import.meta.addon;\nexport default __addon;\n");
    if let Some(obj) = exports.as_object().filter(|o| !o.is_function()) {
        for key in obj.keys::<String>() {
            let key = key?;
            if is_identifier(&key) && key != "__addon" {
                source.push_str(&format!("export const {key} = __addon.{key};\n"));
            }
        }
    }

    let module = Module::declare(ctx.clone(), name, source)?;
    let meta: Object = module.meta()?;
    meta.set("addon", exports)?;
    Ok(module)
}

#[cfg(not(unix))]
fn load_module<'js>(ctx: &Ctx<'js>, _name: &str) -> Result<Module<'js>> {
    Err(Exception::throw_message(
        ctx,
        "Native addons are not supported on this platform",
    ))
}

impl Loader for NapiLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attr: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js>> {
        if !name.ends_with(ADDON_EXT) {
            return Err(Error::new_loading(name));
        }
        trace!("Loading native addon: {}", name);
        load_module(ctx, name)
    }
}
