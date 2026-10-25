# rquickjs-napi

Node-API (N-API) support for loading native `.node` addons in
[rquickjs](https://github.com/DelSkayn/rquickjs).

`rquickjs-napi` provides an `rquickjs` module loader that loads a native addon
and exposes its `module.exports` to JavaScript. It is intended for embedding
Node-API addons in an `rquickjs` runtime; it does not run Node.js.

## Status and platform support

- Native addon loading is currently supported on Unix platforms.
- Addons must be built for the host platform and architecture and use Node-API
  functions supported by this crate.
- The host executable must export the N-API symbols so that the addon can
  resolve them at load time. On Linux, this commonly requires linking the
  executable with `-rdynamic`; the equivalent linker option may differ on
  other platforms.
- Compatibility depends on the APIs used by each addon; compatibility with
  every Node.js native addon is not guaranteed.
- Environment cleanup hooks are unsupported because safe execution requires
  integration with full environment teardown. The add/remove operations for
  both synchronous and asynchronous cleanup hooks return `napi_generic_failure`
  rather than reporting success.

See [NAPI.md](NAPI.md) for the implemented API areas and known limitations.

## Usage

Add `rquickjs` with its module loader feature and this crate to your
application:

```toml
[dependencies]
rquickjs = { version = "0.14", features = ["loader"] }
rquickjs-napi = { version = "0.0.1" }
```

Register `NapiLoader` before general-purpose loaders, and add the `.node`
pattern to `FileResolver` so it can resolve addon paths:

```rust
use rquickjs::{
    loader::{FileResolver, ScriptLoader},
    Context, Module, Runtime,
};
use rquickjs_napi::NapiLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new()?;
    runtime.set_loader(
        FileResolver::default().with_pattern("{}.node"),
        (NapiLoader::default(), ScriptLoader::default()),
    );

    let context = Context::full(&runtime)?;
    context.with(|ctx| {
        Module::evaluate(
            ctx,
            "main.js",
            r#"
                import addon from "./addon.node";
                import { hello } from "./addon.node";
                globalThis.addon = addon;
                globalThis.hello = hello;
            "#,
        )?
        .finish::<()>()?;
        Ok::<(), rquickjs::Error>(())
    })?;

    Ok(())
}
```

Replace `hello` with an export provided by the addon. The addon's default
export is its `module.exports`. If `module.exports` is an object, enumerable
string keys that are valid JavaScript identifiers are also available as named
exports.

## Development

The Makefile provides common build and validation targets. Run `make help` to
list them. `make lint` checks formatting and runs Clippy; `make build` builds
all targets; and `make test` runs Rust unit tests. `make ci` runs these checks
and the N-API integration test. Run `make test-napi` to run just that
integration test; it builds the C addon fixture under `tests/fixtures/napi` and
requires a C compiler. `make check` checks all targets without producing
artifacts, while `make fix` applies Cargo and Clippy suggestions and formats
Rust sources. JavaScript test scripts are kept in `tests/`. To cross-compile,
pass an installed Rust target, for example
`make build TARGET=aarch64-unknown-linux-gnu`.

## License

Licensed under either of

- [MIT](LICENSE-MIT), or
- [Apache License, Version 2.0](LICENSE-APACHE),

at your option.
