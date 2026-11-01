// SPDX-License-Identifier: MIT OR Apache-2.0
import addon, { add, greet } from "./addon.node";
globalThis.napiIntegrationAddon = addon;

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

assert(addon.add(4, 6) === 10, "CJS-style default export");
assert(add(4, 6) === 10, "ESM named function export");
assert(greet("ESM") === "Hello, ESM!", "ESM named string export");

assert(addon.isError(new Error("error")), "Error detection");
assert(addon.isError(new TypeError("type error")), "TypeError detection");
assert(!addon.isError({}), "non-Error detection");

const buffer = addon.createBuffer(4);
assert(addon.isBuffer(buffer), "N-API buffer detection");
assert(!addon.isBuffer(new Uint8Array(4)), "unmarked Uint8Array is not a Buffer");
assert(!addon.isBuffer(new Uint16Array(2)), "typed array is not a Buffer");
assert(!addon.isBuffer(new DataView(new ArrayBuffer(4))), "DataView is not a Buffer");
assert(addon.bufferInfoStatus(buffer) === 0, "buffer info accepts a Buffer");
assert(addon.bufferInfoStatus(new Uint8Array(4)) !== 0, "buffer info rejects an unmarked view");
assert(addon.bufferInfoStatus(new DataView(new ArrayBuffer(4))) !== 0, "buffer info rejects DataView");
const validDataView = new DataView(new ArrayBuffer(4));
assert(addon.dataviewInfoStatus(validDataView) === 0, "DataView info accepts an in-bounds view");
const invalidDataView = new DataView(new ArrayBuffer(4));
Object.defineProperty(invalidDataView, "byteOffset", { value: Number.MAX_SAFE_INTEGER });
assert(
  addon.dataviewInfoStatus(invalidDataView) === 1,
  "DataView info rejects an out-of-bounds byte offset",
);

assert(
  addon.cleanupHookStatuses().join(",") === "1,1,9,9,1",
  "cleanup hook argument validation, registration, removal, and async-hook limitation",
);
assert(addon.cleanupHookEvents().length === 0, "removed cleanup hooks must not run");
assert(
  addon.callbackScopeStatuses().join(",") === "1,1,0,0,0,0",
  "callback scopes validate handles and close nested handle lifetimes",
);

const completion = {
  threadsafe: false,
  boundedTsfn: false,
  blockingTsfn: false,
  promise: false,
  asyncWork: false,
};
globalThis.napiIntegrationDone = false;
const completeIfReady = () => {
  globalThis.napiIntegrationDone =
    completion.threadsafe &&
    completion.boundedTsfn &&
    completion.blockingTsfn &&
    completion.promise &&
    completion.asyncWork;
};
addon.resolvedPromise().then((value) => {
  assert(value === "native promise", "native promise resolution");
  completion.promise = true;
  completeIfReady();
});

const asyncWorkResult = await addon.asyncWork();
assert(asyncWorkResult.executeEnvWasNull, "async execute callback receives a null env");
assert(asyncWorkResult.executed, "uncancelled async work executes");
assert(asyncWorkResult.status === 0, "uncancelled async work completes with napi_ok");
completion.asyncWork = true;
completeIfReady();

const cancelledWorkResult = await addon.asyncWorkCancel();
assert(cancelledWorkResult.cancelStatus === 0, "queued async work can be cancelled");
assert(cancelledWorkResult.secondCancelStatus === 9, "a cancelled work cannot be cancelled twice");
assert(cancelledWorkResult.status === 11, "cancelled work completes with napi_cancelled");
assert(!cancelledWorkResult.executed, "cancelled async work does not execute");

const values = [];
addon.startThread((value) => {
  values.push(value);
  if (values.length === 3) {
    assert(values.join(",") === "1,2,3", "threadsafe function calls");
    completion.threadsafe = true;
    completeIfReady();
  }
});

assert(
  addon
    .tsfnQueueStatuses((value) => {
      assert(value === 1, "only the accepted bounded-queue item is delivered");
      completion.boundedTsfn = true;
      completeIfReady();
    })
    .join(",") === "0,0,15,15,1,0",
  "bounded queue reports full status and rejects unknown call mode",
);

const boundedValues = [];
addon.startBoundedThread((value) => {
  boundedValues.push(value);
  if (boundedValues.length === 3) {
    assert(
      boundedValues.join(",") === "1,2,3",
      "blocking TSFN producers wait for bounded queue capacity",
    );
    assert(
      addon.boundedBlockingStatuses().join(",") === "0,0,0",
      "blocking calls from a worker thread succeed as queue slots open",
    );
    completion.blockingTsfn = true;
    completeIfReady();
  }
});
