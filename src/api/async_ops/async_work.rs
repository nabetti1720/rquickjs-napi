// SPDX-License-Identifier: MIT OR Apache-2.0
//! Async work executed on a worker thread.
use super::*;

use std::{mem, thread};

use tokio::sync::oneshot;

pub struct AsyncWork {
    pub(super) execute: napi_async_execute,
    pub(super) complete: Option<napi_async_complete>,
    pub(super) data: usize,
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_async_work(
    env: napi_env,
    _async_resource: napi_value,
    _async_resource_name: napi_value,
    execute: Option<napi_async_execute>,
    complete: Option<napi_async_complete>,
    data: *mut c_void,
    result: *mut *mut AsyncWork,
) -> napi_status {
    run(env, || {
        let execute = execute.ok_or(NAPI_INVALID_ARG)?;
        out(
            result,
            Box::into_raw(Box::new(AsyncWork {
                execute,
                complete,
                data: data as usize,
            })),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_delete_async_work(
    env: napi_env,
    work: *mut AsyncWork,
) -> napi_status {
    run(env, || {
        if work.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        drop(Box::from_raw(work));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_cancel_async_work(
    env: napi_env,
    _work: *mut AsyncWork,
) -> napi_status {
    run(env, || Err(NAPI_GENERIC_FAILURE))
}

/// Runs `execute` on a dedicated thread and `complete` back on the JS thread.
#[no_mangle]
pub unsafe extern "C" fn napi_queue_async_work(env: napi_env, work: *mut AsyncWork) -> napi_status {
    run(env, || {
        let w = work.as_ref().ok_or(NAPI_INVALID_ARG)?;
        let (execute, complete, data) =
            (w.execute as usize, w.complete.map(|c| c as usize), w.data);
        let env_addr = env as usize;
        let (tx, rx) = oneshot::channel::<()>();
        thread::spawn(move || {
            let execute: napi_async_execute = mem::transmute(execute);
            execute(env_addr as napi_env, data as *mut c_void);
            let _ = tx.send(());
        });
        ctx(env).spawn(async move {
            let _ = rx.await;
            let env = env_addr as napi_env;
            if let Some(complete) = complete {
                let complete: napi_async_complete = mem::transmute(complete);
                let mark = (*env).handles.len();
                complete(env, NAPI_OK, data as *mut c_void);
                (*env).handles.truncate(mark);
                if (*env).exception.take().is_some() {
                    eprintln!("Uncaught exception in native async work completion");
                }
            }
        });
        Ok(())
    })
}
