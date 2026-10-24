// SPDX-License-Identifier: MIT OR Apache-2.0
use std::time::Duration;

use rquickjs::{
    loader::{ImportAttributes, Resolver},
    AsyncContext, AsyncRuntime, Ctx, Error, Module, Result,
};
use rquickjs_napi::NapiLoader;

struct AddonResolver;

impl Resolver for AddonResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String> {
        if name != "./addon.node" {
            return Err(Error::new_resolving(base, name));
        }
        Ok(format!(
            "{}/tests/fixtures/napi/addon.node",
            env!("CARGO_MANIFEST_DIR")
        ))
    }
}

#[test]
#[ignore = "requires the native sample addon; run with `make test-napi`"]
fn loads_and_runs_napi_addon_in_rquickjs() {
    let tokio = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("create Tokio runtime");
    let local = tokio::task::LocalSet::new();

    local.block_on(&tokio, async {
        let runtime = AsyncRuntime::new().expect("create QuickJS runtime");
        runtime.set_loader(AddonResolver, NapiLoader).await;

        let context = AsyncContext::full(&runtime)
            .await
            .expect("create QuickJS context");
        context
            .async_with(async |ctx| {
                let promise =
                    Module::evaluate(ctx.clone(), "tests/test.mjs", include_str!("test.mjs"))?;
                promise.into_future::<()>().await
            })
            .await
            .expect("evaluate N-API integration module");

        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                runtime
                    .execute_pending_job()
                    .await
                    .expect("execute QuickJS pending job");
                let done = context
                    .with(|ctx| ctx.eval::<bool, _>("globalThis.napiIntegrationDone"))
                    .await
                    .expect("read integration completion state");
                if done {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("N-API integration test timed out");

        tokio::time::timeout(Duration::from_secs(10), runtime.idle())
            .await
            .expect("QuickJS runtime did not become idle");
        let finalize_count = context
            .with(|ctx| ctx.eval::<i32, _>("globalThis.napiIntegrationAddon.tsfnFinalizeCount()"))
            .await
            .expect("read TSFN finalizer count");
        assert_eq!(finalize_count, 1, "unref'd threadsafe function finalizer");
    });
}
