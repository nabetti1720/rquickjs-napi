<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Node-API support

This document describes the Node-API (N-API) surface implemented by
`rquickjs-napi`. It is a compatibility guide, not a guarantee that every addon
using Node-API will work. Addons that depend on Node.js, V8, NAN, libuv, or
Node-specific runtime behavior are not supported unless the needed behavior is
provided by this crate.

## Platform and loading requirements

- Native `.node` loading is implemented for Unix and Windows platforms. The CI
  and integration tests currently cover Linux, macOS and Windows.

- The host must export the `napi_*` symbols so that the native library can
  resolve them. On Linux, this commonly requires linking the executable with
  `-rdynamic`. Configure the equivalent symbol-export behavior on other
  platforms, including Windows.
- Use `NapiLoader` with the `rquickjs` `loader` feature, and configure a
  resolver to resolve `.node` paths. Register `NapiLoader` before a general
  script loader.
- Addons must use N-API entry points supported by this crate and be built for
  the host operating system and architecture.
- The implementation reports N-API version 8 and Node version 20.0.0 through
  the version APIs. The reported Node version is compatibility metadata; this
  crate does not embed or run Node.js.

## Implemented API areas

The implementation provides N-API entry points in these areas:

| Area | Examples |
| --- | --- |
| Values and types | Undefined, null, booleans, numbers, strings (UTF-8, Latin-1, UTF-16), BigInt, symbols, dates, type checks, coercion, strict equality |
| Objects and properties | Object/array creation, property access and definition, property enumeration, property checks/deletion, prototypes, freeze/seal, type tags |
| Functions and classes | Native functions, callback arguments and `this`, JS function calls, class definition, construction and `new.target` |
| Errors | Error creation and throwing, pending-exception queries and retrieval, Error type checks |
| References and lifetimes | Handle scopes, escapable scopes, references, object wrapping/unwrapping, external values, finalizers |
| Binary data | ArrayBuffers, external ArrayBuffers, typed arrays, DataViews, Buffers, data and length accessors |
| Async APIs | Promises and deferreds, async work, callback scopes, async context helpers, thread-safe functions |
| Other | Script execution, instance data access, synchronous environment cleanup hooks, external-memory adjustment, ArrayBuffer detach checks |

This list describes implemented API families, not complete Node.js compatibility.
Check the implementation and tests for the exact behavior of a particular API.

## Known limitations

| API or behavior | Current behavior |
| --- | --- |
| Synchronous environment cleanup hooks (`napi_add_env_cleanup_hook`, `napi_remove_env_cleanup_hook`) | Hooks and arguments are stored per addon environment and execute in reverse registration order when the owning QuickJS context is destroyed. Reusing a callback with different arguments is supported; an identical callback/argument pair is rejected with `napi_invalid_arg` rather than Node.js's process-aborting behavior. Removing an unregistered pair is a successful no-op. Null environment/function arguments return `napi_invalid_arg`. |
| Asynchronous environment cleanup hooks (`napi_add_async_cleanup_hook`, `napi_remove_async_cleanup_hook`) | Unsupported because this crate cannot pause QuickJS context destruction while asynchronous cleanup is pending. A null hook or null remove handle returns `napi_invalid_arg`; otherwise calls return `napi_generic_failure`. |
| `napi_get_uv_event_loop` | No libuv loop is exposed; the call returns `napi_generic_failure`. |
| Async context tracking (`napi_async_init`, `napi_async_destroy`, `napi_make_callback`) | Async context handles have no Node.js `async_hooks` or `AsyncLocalStorage` propagation. `napi_make_callback` invokes the function but does not restore Node.js async context. |
| Callback scopes (`napi_open_callback_scope`, `napi_close_callback_scope`) | Handle lifetimes are scoped and cleaned up on close. The resource and async-context arguments are ignored because async-hooks integration is not provided. |
| `napi_cancel_async_work` | Queued work can be cancelled before its worker starts; its completion callback receives `napi_cancelled` and the execute callback is skipped. Cancellation after work has started, before queueing, or repeated cancellation returns `napi_generic_failure`. |
| Instance-data finalizer in `napi_set_instance_data` | The current finalizer is called once during environment teardown with its associated data and hint, after synchronous cleanup hooks. Replacing instance data overwrites the prior finalizer without invoking it. |
| Thread-safe function queue options | A `max_queue_size` of zero is unbounded; a positive limit is enforced. Nonblocking calls return `napi_queue_full` when full, while blocking calls from worker threads wait for capacity. Blocking calls from the JS execution thread or while an unref'd TSFN is parked return `napi_queue_full` rather than deadlocking. |
| Buffer fallback | When a global Node.js `Buffer` is absent, buffers created by this implementation are `Uint8Array` instances tracked internally. Arbitrary typed arrays and DataViews are not Buffers. |
| Node.js and libuv integration | Node.js modules, the Node event loop, and libuv handles are not provided. |

Do not rely on a function merely being present in the N-API symbol table as proof
of full Node.js-equivalent semantics. Synchronous cleanup hooks run at QuickJS
context destruction; asynchronous cleanup hooks, Node.js async-hooks integration,
and Node.js environment teardown integration are not provided.

## Testing

Run `make test-napi` to compile the C fixture and load it through `NapiLoader` in
an `rquickjs` host. `make ci` includes this integration test on the supported CI
platforms. The fixture exercises representative APIs; it is not an exhaustive
conformance suite.
