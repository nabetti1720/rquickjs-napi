// SPDX-License-Identifier: MIT OR Apache-2.0
//! Thread-safe functions: calls queued from any thread and run on the JS thread.
//!
//! A ref'd function keeps the event loop alive through a spawned task. When it is
//! unref'd the task parks (keeping its queue) and resumes on ref or final release.
use super::*;

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Condvar, Mutex, MutexGuard,
};
use std::thread::{self, ThreadId};

use tokio::sync::mpsc::{
    error::TryRecvError, unbounded_channel, UnboundedReceiver, UnboundedSender,
};

const NAPI_CLOSING: napi_status = 16;
const NAPI_QUEUE_FULL: napi_status = 15;
const NAPI_TSFN_NONBLOCKING: c_int = 0;
const NAPI_TSFN_BLOCKING: c_int = 1;

type napi_threadsafe_function_call_js =
    unsafe extern "C" fn(napi_env, napi_value, *mut c_void, *mut c_void);

enum Msg {
    Data(usize),
    Close,
    Wake,
}

pub struct Tsfn {
    env: usize,
    func_id: u64,
    context: usize,
    call_js: Option<napi_threadsafe_function_call_js>,
    finalize_cb: Option<napi_finalize>,
    finalize_data: usize,
    tx: UnboundedSender<Msg>,
    parked: Mutex<Option<UnboundedReceiver<Msg>>>,
    max_queue_size: usize,
    queue_size: Mutex<usize>,
    queue_space: Condvar,
    js_thread: ThreadId,
    want_ref: AtomicBool,
    threads: AtomicUsize,
    closing: AtomicBool,
}

fn lock_queue_size(t: &Tsfn) -> MutexGuard<'_, usize> {
    match t.queue_size.lock() {
        Ok(size) => size,
        Err(poisoned) => {
            eprintln!("Recovering poisoned threadsafe function queue size");
            poisoned.into_inner()
        }
    }
}

fn enqueue_data(t: &Tsfn, data: usize, blocking: bool) -> napi_status {
    let mut size = lock_queue_size(t);
    loop {
        if t.closing.load(Ordering::SeqCst) {
            return NAPI_CLOSING;
        }
        if t.max_queue_size == 0 || *size < t.max_queue_size {
            break;
        }
        let unref_parked = !t.want_ref.load(Ordering::SeqCst) && lock_parked(t).is_some();
        if !blocking || thread::current().id() == t.js_thread || unref_parked {
            return NAPI_QUEUE_FULL;
        }
        size = match t.queue_space.wait(size) {
            Ok(size) => size,
            Err(poisoned) => {
                eprintln!("Recovering poisoned threadsafe function queue size");
                poisoned.into_inner()
            }
        };
    }
    if t.max_queue_size > 0 {
        *size += 1;
    }
    match t.tx.send(Msg::Data(data)) {
        Ok(()) => NAPI_OK,
        Err(_) => {
            if t.max_queue_size > 0 {
                *size = size.saturating_sub(1);
                t.queue_space.notify_one();
            }
            NAPI_CLOSING
        }
    }
}

fn release_queue_slot(t: &Tsfn) {
    let mut size = lock_queue_size(t);
    *size = size.saturating_sub(1);
    t.queue_space.notify_one();
}

fn release_slot_for_message(t: &Tsfn, message: &Msg) {
    if matches!(message, Msg::Data(_)) && t.max_queue_size > 0 {
        release_queue_slot(t);
    }
}

fn lock_parked(t: &Tsfn) -> MutexGuard<'_, Option<UnboundedReceiver<Msg>>> {
    match t.parked.lock() {
        Ok(parked) => parked,
        Err(poisoned) => {
            eprintln!("Recovering poisoned threadsafe function receiver");
            poisoned.into_inner()
        }
    }
}

fn start(t: &'static Tsfn, rx: UnboundedReceiver<Msg>) {
    ctx(t.env as napi_env).spawn(run_loop(t, rx));
}

async fn run_loop(t: &'static Tsfn, mut rx: UnboundedReceiver<Msg>) {
    loop {
        let msg = if t.want_ref.load(Ordering::SeqCst) {
            match rx.recv().await {
                Some(m) => {
                    release_slot_for_message(t, &m);
                    m
                }
                None => break,
            }
        } else {
            match rx.try_recv() {
                Ok(m) => {
                    release_slot_for_message(t, &m);
                    m
                }
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    let resume = {
                        let mut parked = lock_parked(t);
                        *parked = Some(rx);
                        // Ref or release may race with parking the receiver.
                        if t.want_ref.load(Ordering::SeqCst) || t.closing.load(Ordering::SeqCst) {
                            parked.take()
                        } else {
                            None
                        }
                    };
                    if let Some(r) = resume {
                        rx = r;
                        continue;
                    }
                    return;
                }
            }
        };
        match msg {
            Msg::Data(data) => unsafe { deliver(t, data) },
            Msg::Wake => {}
            Msg::Close => break,
        }
    }
    unsafe { finish(t) };
}

unsafe fn deliver(t: &Tsfn, data: usize) {
    let env = t.env as napi_env;
    let mark = (*env).handles.len();
    let func: Option<V> = st(env, "refGet", (t.func_id as f64,)).ok().flatten();
    match t.call_js {
        Some(cb) => {
            let f = func.map_or(ptr::null_mut(), |v| push(env, v));
            cb(env, f, t.context as *mut c_void, data as *mut c_void);
        }
        None => {
            if let Some(f) = func.as_ref().and_then(|v| v.as_function()) {
                let _ = js(env, f.call::<_, Value>(()));
            }
        }
    }
    (*env).handles.truncate(mark);
    if (*env).exception.take().is_some() {
        eprintln!("Uncaught exception in threadsafe function callback");
    }
}

unsafe fn finish(t: &Tsfn) {
    t.closing.store(true, Ordering::SeqCst);
    let env = t.env as napi_env;
    let _ = st::<_, ()>(env, "refDel", (t.func_id as f64,));
    if let Some(cb) = t.finalize_cb {
        let mark = (*env).handles.len();
        cb(
            env,
            t.finalize_data as *mut c_void,
            t.context as *mut c_void,
        );
        (*env).handles.truncate(mark);
        (*env).exception.take();
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_threadsafe_function(
    env: napi_env,
    func: napi_value,
    _async_resource: napi_value,
    _async_resource_name: napi_value,
    max_queue_size: usize,
    initial_thread_count: usize,
    thread_finalize_data: *mut c_void,
    thread_finalize_cb: Option<napi_finalize>,
    context: *mut c_void,
    call_js_cb: Option<napi_threadsafe_function_call_js>,
    result: *mut *mut Tsfn,
) -> napi_status {
    run_js(env, || {
        if result.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        if initial_thread_count == 0 || (func.is_null() && call_js_cb.is_none()) {
            return Err(NAPI_INVALID_ARG);
        }
        let func_id = next_id();
        if let Some(f) = func.as_ref() {
            if !f.is_function() {
                return Err(NAPI_FUNCTION_EXPECTED);
            }
            st::<_, ()>(env, "refNew", (func_id as f64, f.clone(), true))?;
        }
        let (tx, rx) = unbounded_channel();
        // Never freed: other threads may still hold the pointer after the function closes.
        let t: &'static Tsfn = Box::leak(Box::new(Tsfn {
            env: env as usize,
            func_id,
            context: context as usize,
            call_js: call_js_cb,
            finalize_cb: thread_finalize_cb,
            finalize_data: thread_finalize_data as usize,
            tx,
            parked: Mutex::new(None),
            max_queue_size,
            queue_size: Mutex::new(0),
            queue_space: Condvar::new(),
            js_thread: thread::current().id(),
            want_ref: AtomicBool::new(true),
            threads: AtomicUsize::new(initial_thread_count),
            closing: AtomicBool::new(false),
        }));
        start(t, rx);
        out(result, t as *const Tsfn as *mut Tsfn)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_threadsafe_function_context(
    func: *mut Tsfn,
    result: *mut *mut c_void,
) -> napi_status {
    match (func.as_ref(), result.is_null()) {
        (Some(t), false) => {
            *result = t.context as *mut c_void;
            NAPI_OK
        }
        _ => NAPI_INVALID_ARG,
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_call_threadsafe_function(
    func: *mut Tsfn,
    data: *mut c_void,
    is_blocking: c_int,
) -> napi_status {
    let Some(t) = func.as_ref() else {
        return NAPI_INVALID_ARG;
    };
    if is_blocking != NAPI_TSFN_NONBLOCKING && is_blocking != NAPI_TSFN_BLOCKING {
        return NAPI_INVALID_ARG;
    }
    enqueue_data(t, data as usize, is_blocking == NAPI_TSFN_BLOCKING)
}

#[no_mangle]
pub unsafe extern "C" fn napi_acquire_threadsafe_function(func: *mut Tsfn) -> napi_status {
    let Some(t) = func.as_ref() else {
        return NAPI_INVALID_ARG;
    };
    if t.closing.load(Ordering::SeqCst) {
        return NAPI_CLOSING;
    }
    t.threads.fetch_add(1, Ordering::SeqCst);
    NAPI_OK
}

/// `mode`: 0 = release, 1 = abort.
#[no_mangle]
pub unsafe extern "C" fn napi_release_threadsafe_function(
    func: *mut Tsfn,
    mode: c_int,
) -> napi_status {
    let Some(t) = func.as_ref() else {
        return NAPI_INVALID_ARG;
    };
    let previous = t
        .threads
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1));
    match previous {
        Err(_) => NAPI_INVALID_ARG,
        Ok(n) => {
            if n == 1 || mode == 1 {
                let _size = lock_queue_size(t);
                t.closing.store(true, Ordering::SeqCst);
                t.queue_space.notify_all();
                let _ = t.tx.send(Msg::Close);
                if let Some(rx) = lock_parked(t).take() {
                    start(t, rx);
                }
            }
            NAPI_OK
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_ref_threadsafe_function(
    env: napi_env,
    func: *mut Tsfn,
) -> napi_status {
    run(env, || {
        let t: &'static Tsfn = func.as_ref().ok_or(NAPI_INVALID_ARG)?;
        t.want_ref.store(true, Ordering::SeqCst);
        let parked = t.parked.lock().map_err(|_| NAPI_GENERIC_FAILURE)?.take();
        match parked {
            Some(rx) => start(t, rx),
            None => {
                let _ = t.tx.send(Msg::Wake);
            }
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_unref_threadsafe_function(
    env: napi_env,
    func: *mut Tsfn,
) -> napi_status {
    run(env, || {
        let t = func.as_ref().ok_or(NAPI_INVALID_ARG)?;
        t.want_ref.store(false, Ordering::SeqCst);
        let _ = t.tx.send(Msg::Wake);
        Ok(())
    })
}
