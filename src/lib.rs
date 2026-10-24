// SPDX-License-Identifier: MIT OR Apache-2.0
//! Node-API (N-API) support for loading native `.node` addons.
//!
//! [`NapiLoader`] is an `rquickjs` module loader; add it to the loader tuple before
//! the generic file loaders. The executable must export its symbols (`-rdynamic`) so
//! that addons can resolve the `napi_*` functions at load time.
#[cfg(unix)]
mod api;
mod loader;

pub use self::loader::NapiLoader;
