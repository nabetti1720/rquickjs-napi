// SPDX-License-Identifier: MIT OR Apache-2.0
//! Async work executed on a worker thread.
use super::*;

use std::{
    mem,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    thread,
};

use tokio::sync::oneshot;

const WORK_CREATED: u8 = 0;
const WORK_QUEUED: u8 = 1;
const WORK_RUNNING: u8 = 2;
const WORK_CANCELLED: u8 = 3;
const WORK_COMPLETED: u8 = 4;

pub struct AsyncWork {
    pub(super) execute: napi_async_execute,
    pub(super) complete: Option<napi_async_complete>,
    pub(super) data: usize,
    state: Arc<AtomicU8>,
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
                state: Arc::new(AtomicU8::new(WORK_CREATED)),
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
    work: *mut AsyncWork,
) -> napi_status {
    run(env, || {
        let work = work.as_ref().ok_or(NAPI_INVALID_ARG)?;
        work.state
            .compare_exchange(
                WORK_QUEUED,
                WORK_CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| NAPI_GENERIC_FAILURE)
    })
}

/// Runs `execute` on a dedicated thread and `complete` back on the JS thread.
#[no_mangle]
pub unsafe extern "C" fn napi_queue_async_work(env: napi_env, work: *mut AsyncWork) -> napi_status {
    run(env, || {
        let w = work.as_ref().ok_or(NAPI_INVALID_ARG)?;
        w.state
            .compare_exchange(
                WORK_CREATED,
                WORK_QUEUED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| NAPI_GENERIC_FAILURE)?;
        let (execute, complete, data, state) = (
            w.execute as usize,
            w.complete.map(|c| c as usize),
            w.data,
            Arc::clone(&w.state),
        );
        let env_addr = env as usize;
        ctx(env).spawn(async move {
            let status = match state.compare_exchange(
                WORK_QUEUED,
                WORK_RUNNING,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let (tx, rx) = oneshot::channel();
                    thread::spawn(move || {
                        let execute: napi_async_execute = mem::transmute(execute);
                        execute(ptr::null_mut(), data as *mut c_void);
                        let _ = tx.send(());
                    });
                    if rx.await.is_ok() {
                        NAPI_OK
                    } else {
                        NAPI_GENERIC_FAILURE
                    }
                }
                Err(WORK_CANCELLED) => NAPI_CANCELLED,
                Err(_) => NAPI_GENERIC_FAILURE,
            };
            state.store(WORK_COMPLETED, Ordering::Release);
            let env = env_addr as napi_env;
            if let Some(complete) = complete {
                let complete: napi_async_complete = mem::transmute(complete);
                let mark = (*env).handles.len();
                complete(env, status, data as *mut c_void);
                (*env).handles.truncate(mark);
                if (*env).exception.take().is_some() {
                    eprintln!("Uncaught exception in native async work completion");
                }
            }
        });
        Ok(())
    })
}
