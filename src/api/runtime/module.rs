// SPDX-License-Identifier: MIT OR Apache-2.0
//! Legacy Node-API module registration used by addon loading.
use super::*;

use std::sync::atomic::{AtomicPtr, Ordering};

#[repr(C)]
pub struct napi_module {
    nm_version: c_int,
    nm_flags: u32,
    nm_filename: *const c_char,
    pub(in crate::api) nm_register_func: Option<napi_addon_init>,
    nm_modname: *const c_char,
    nm_priv: *mut c_void,
    reserved: [*mut c_void; 4],
}

pub(in crate::api) static PENDING_MODULE: AtomicPtr<napi_module> = AtomicPtr::new(ptr::null_mut());

#[no_mangle]
pub unsafe extern "C" fn napi_module_register(module: *mut napi_module) {
    PENDING_MODULE.store(module, Ordering::Release);
}
