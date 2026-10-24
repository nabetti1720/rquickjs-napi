// SPDX-License-Identifier: MIT OR Apache-2.0
//! Node-API and host version reporting.
use super::*;

#[repr(C)]
pub struct napi_node_version {
    major: u32,
    minor: u32,
    patch: u32,
    release: *const c_char,
}

struct NodeVersion(napi_node_version);
unsafe impl Sync for NodeVersion {}

static NODE_VERSION: NodeVersion = NodeVersion(napi_node_version {
    major: 20,
    minor: 0,
    patch: 0,
    release: c"node".as_ptr(),
});

#[no_mangle]
pub unsafe extern "C" fn napi_get_version(env: napi_env, result: *mut u32) -> napi_status {
    run(env, || out(result, 8))
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_node_version(
    env: napi_env,
    result: *mut *const napi_node_version,
) -> napi_status {
    run(env, || out(result, &NODE_VERSION.0 as *const _))
}
