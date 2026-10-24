<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Node-API support

This document describes the Node-API (N-API) surface implemented by
`rquickjs-napi`. It is a compatibility guide, not a guarantee that every addon
using Node-API will work. Addons that depend on Node.js, V8, NAN, libuv, or
Node-specific runtime behavior are not supported unless the needed behavior is
provided by this crate.

## Platform and loading requirements

- Native `.node` loading is implemented for Unix platforms. The CI and
  integration tests currently cover Linux and macOS.
- The host must export the `napi_*` symbols so that the native library can
  resolve them. On Linux, this commonly requires linking the executable with
  `-rdynamic`. Use the equivalent export-dynamic linker option on other
  platforms.
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
| Other | Script execution, instance data access, external-memory adjustment, ArrayBuffer detach checks |

This list describes implemented API families, not complete Node.js compatibility.
Check the implementation and tests for the exact behavior of a particular API.

## Known limitations

| API or behavior | Current behavior |
| --- | --- |
| Environment cleanup hooks (`napi_add_env_cleanup_hook`, `napi_remove_env_cleanup_hook`, `napi_add_async_cleanup_hook`, `napi_remove_async_cleanup_hook`) | Unsupported. Calls return `napi_generic_failure`. |
| `napi_get_uv_event_loop` | No libuv loop is exposed; the call returns `napi_generic_failure`. |
| `napi_cancel_async_work` | Cancellation is unsupported and returns `napi_generic_failure`. |
| Instance-data finalizer in `napi_set_instance_data` | The data pointer can be stored and retrieved, but the finalizer and hint are not invoked. |
| Thread-safe function queue options | `max_queue_size` and the blocking mode are ignored; the queue is unbounded. Producers can enqueue faster than callbacks are processed, increasing memory use. |
| Buffer fallback | When a global Node.js `Buffer` is absent, buffers created by this implementation are `Uint8Array` instances tracked internally. Arbitrary typed arrays and DataViews are not Buffers. |
| Node.js and libuv integration | Node.js modules, the Node event loop, and libuv handles are not provided. |

Do not rely on a function merely being present in the N-API symbol table as proof
of full Node.js-equivalent semantics. In particular, review the limitations
above before using async work, thread-safe functions, or environment teardown.

## Testing

Run `make test-napi` to compile the C fixture and load it through `NapiLoader` in
an `rquickjs` host. `make ci` includes this integration test on the supported CI
platforms. The fixture exercises representative APIs; it is not an exhaustive
conformance suite.
