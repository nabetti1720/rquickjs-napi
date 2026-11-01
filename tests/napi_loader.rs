// SPDX-License-Identifier: MIT OR Apache-2.0
use std::time::Duration;

use rquickjs::{
    loader::{ImportAttributes, Resolver},
    AsyncContext, AsyncRuntime, Context, Ctx, Error, Module, Result, Runtime,
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

    {
        let runtime = Runtime::new().expect("create runtime for callback-scope validation");
        runtime.set_loader(AddonResolver, NapiLoader);
        let first = Context::full(&runtime).expect("create callback-scope owner context");
        first
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "callback-scope-owner.mjs",
                    "import addon from './addon.node'; \
                     globalThis.callbackScopeAddon = addon; \
                     if (addon.saveCallbackScope() !== 0) \
                       throw new Error('opening callback scope failed');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("save callback scope in owning environment");

        let second = Context::full(&runtime).expect("create callback-scope foreign context");
        second
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "callback-scope-foreign.mjs",
                    "import addon from './addon.node'; \
                     if (addon.closeSavedCallbackScope() !== 1) \
                       throw new Error('foreign callback scope was accepted');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("reject callback scope from another environment");

        first
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "if (callbackScopeAddon.closeSavedCallbackScope() !== 0) \
                     throw new Error('owning environment could not close callback scope');",
                )?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("close callback scope in owning environment");
    }

    {
        let runtime = Runtime::new().expect("create runtime for instance-data finalizers");
        runtime.set_loader(AddonResolver, NapiLoader);

        let first = Context::full(&runtime).expect("create first instance-data context");
        first
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "instance-data-first.mjs",
                    "import addon from './addon.node'; \
                     if (addon.setInstanceDataWithFinalizer(1, 10) !== 0 || \
                         addon.setInstanceDataWithFinalizer(2, 20) !== 0) \
                       throw new Error('setting instance data failed');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("set and replace instance data");

        let second = Context::full(&runtime).expect("create second instance-data context");
        second
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "instance-data-second.mjs",
                    "import addon from './addon.node'; \
                     if (addon.setInstanceDataWithFinalizer(3, 30) !== 0) \
                       throw new Error('setting instance data failed');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("set instance data in separate environment");
    }

    {
        let runtime = Runtime::new().expect("create runtime to inspect instance-data finalizers");
        runtime.set_loader(AddonResolver, NapiLoader);
        let context = Context::full(&runtime).expect("create instance-data verification context");
        context
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "instance-data-verify.mjs",
                    "import addon from './addon.node'; \
                     globalThis.instanceDataAddon = addon;",
                )?
                .finish::<()>()?;
                let values: String =
                    ctx.eval("instanceDataAddon.instanceDataFinalizeEvents().join(',')")?;
                let values: Vec<i32> = values
                    .split(',')
                    .map(|value| {
                        value
                            .parse::<i32>()
                            .expect("instance data value is integer")
                    })
                    .collect();
                assert_eq!(values.len(), 4, "two environments finalize once each");
                let (pairs, remainder) = values.as_chunks::<2>();
                assert!(remainder.is_empty());
                let mut pairs: Vec<_> = pairs.iter().map(|[data, hint]| (*data, *hint)).collect();
                pairs.sort_unstable();
                assert_eq!(
                    pairs,
                    [(2, 20), (3, 30)],
                    "only the current data finalizer runs once per environment"
                );
                Ok::<(), rquickjs::Error>(())
            })
            .expect("verify instance-data finalizers");
    }

    {
        let runtime = Runtime::new().expect("create sync runtime for cleanup-hook lifecycle");
        runtime.set_loader(AddonResolver, NapiLoader);

        let context = Context::full(&runtime).expect("create first cleanup-hook context");
        context
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "cleanup-hooks.mjs",
                    "import addon from './addon.node'; \
                     if (addon.addCleanupHook(1) !== 0 || \
                         addon.addCleanupHook(2) !== 0 || \
                         addon.addCleanupHook(4) !== 0 || \
                         addon.addCleanupHook(4) !== 1 || \
                         addon.removeCleanupHook(2) !== 0 || \
                         addon.removeCleanupHook(99) !== 0 || \
                         addon.addCleanupHook(3) !== 0 || \
                         addon.addCleanupHook(9) !== 0 || \
                         addon.removeCleanupHook(9) !== 0) \
                       throw new Error('cleanup hook registration failed');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("register cleanup hooks in first environment");

        let context = Context::full(&runtime).expect("create second cleanup-hook context");
        context
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "cleanup-hooks-second.mjs",
                    "import addon from './addon.node'; \
                     if (addon.addCleanupHook(9) !== 0 || \
                         addon.addCleanupHook(8) !== 0) \
                       throw new Error('cleanup hook registration failed');",
                )?
                .finish::<()>()?;
                Ok::<(), rquickjs::Error>(())
            })
            .expect("register cleanup hooks in second environment");
    }

    let runtime = Runtime::new().expect("create runtime to inspect cleanup hooks");
    runtime.set_loader(AddonResolver, NapiLoader);
    let context = Context::full(&runtime).expect("create cleanup-hook verification context");
    context
        .with(|ctx| {
            Module::evaluate(
                ctx.clone(),
                "cleanup-hooks-verify.mjs",
                "import addon from './addon.node'; \
                 globalThis.cleanupAddon = addon;",
            )?
            .finish::<()>()?;
            let events: String = ctx.eval("cleanupAddon.cleanupHookEvents().join(',')")?;
            let events: Vec<i32> = events
                .split(',')
                .map(|event| event.parse().expect("cleanup event is an integer"))
                .collect();
            let position = |marker| events.iter().position(|event| *event == marker).unwrap();
            assert_eq!(
                events.len(),
                5,
                "removed and distinct-argument hooks run once"
            );
            assert!(position(3) < position(4) && position(4) < position(1));
            assert!(position(8) < position(9));
            assert_eq!(
                events.iter().filter(|event| **event == 9).count(),
                1,
                "removal in one environment must not remove another environment's hook"
            );
            Ok::<(), rquickjs::Error>(())
        })
        .expect("verify cleanup-hook order and environment isolation");

    {
        let runtime = Runtime::new().expect("create runtime for cleanup-hook context access");
        runtime.set_loader(AddonResolver, NapiLoader);
        {
            let context = Context::full(&runtime).expect("create cleanup-hook context");
            context
                .with(|ctx| {
                    Module::evaluate(
                        ctx.clone(),
                        "cleanup-hook-context.mjs",
                        "import addon from './addon.node'; \
                         addon.resetCleanupLifecycleEvents(); \
                         if (addon.addCleanupHookUsingEnv(7) !== 0 || \
                             addon.setInstanceDataWithFinalizer(50, 0) !== 0) \
                           throw new Error('cleanup lifecycle setup failed');",
                    )?
                    .finish::<()>()?;
                    Ok::<(), rquickjs::Error>(())
                })
                .expect("register cleanup hook with environment access");
        }
        {
            let context = Context::full(&runtime).expect("create second cleanup-hook context");
            context
                .with(|ctx| {
                    Module::evaluate(
                        ctx.clone(),
                        "cleanup-hook-context-second.mjs",
                        "import addon from './addon.node'; \
                         if (addon.addCleanupHookUsingEnv(8) !== 0 || \
                             addon.setInstanceDataWithFinalizer(60, 0) !== 0) \
                           throw new Error('second cleanup lifecycle setup failed');",
                    )?
                    .finish::<()>()?;
                    Ok::<(), rquickjs::Error>(())
                })
                .expect("register second environment cleanup hook and finalizer");
        }
    }

    {
        let runtime = Runtime::new().expect("create runtime to verify cleanup-hook context access");
        runtime.set_loader(AddonResolver, NapiLoader);
        let context =
            Context::full(&runtime).expect("create cleanup-hook lifecycle verification context");
        context
            .with(|ctx| {
                Module::evaluate(
                    ctx.clone(),
                    "cleanup-hook-context-verify.mjs",
                    "import addon from './addon.node'; \
                     globalThis.cleanupLifecycleAddon = addon;",
                )?
                .finish::<()>()?;
                let events: String =
                    ctx.eval("cleanupLifecycleAddon.cleanupLifecycleEvents().join(',')")?;
                let events: Vec<i32> = events
                    .split(',')
                    .map(|event| {
                        event
                            .parse()
                            .expect("cleanup lifecycle event is an integer")
                    })
                    .collect();
                let position = |marker| events.iter().position(|event| *event == marker).unwrap();
                assert_eq!(
                    events.len(),
                    4,
                    "both environments run their hook and finalizer exactly once"
                );
                assert!(position(7) < position(-50));
                assert!(position(8) < position(-60));
                let statuses: String =
                    ctx.eval("cleanupLifecycleAddon.cleanupHookContextStatuses().join(',')")?;
                assert_eq!(
                    statuses, "0,0",
                    "both callbacks can use their N-API environment"
                );
                Ok::<(), rquickjs::Error>(())
            })
            .expect("verify cleanup callback context and finalizer order");
    }
}
