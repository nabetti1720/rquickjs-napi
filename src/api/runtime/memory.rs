// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native external-memory accounting.
use super::*;

use std::sync::atomic::{AtomicI64, Ordering};

static EXTERNAL_MEMORY: AtomicI64 = AtomicI64::new(0);

#[no_mangle]
pub unsafe extern "C" fn napi_adjust_external_memory(
    env: napi_env,
    change_in_bytes: i64,
    adjusted_value: *mut i64,
) -> napi_status {
    run(env, || {
        let total = EXTERNAL_MEMORY.fetch_add(change_in_bytes, Ordering::Relaxed) + change_in_bytes;
        out(adjusted_value, total)
    })
}
