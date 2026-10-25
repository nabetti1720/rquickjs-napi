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
const invalidDataView = new DataView(new ArrayBuffer(4));
Object.defineProperty(invalidDataView, "byteOffset", { value: Number.MAX_SAFE_INTEGER });
assert(
  addon.dataviewInfoStatus(invalidDataView) !== 0,
  "DataView info rejects an out-of-bounds byte offset",
);

assert(
  addon.cleanupHookStatuses().every((status) => status !== 0),
  "unsupported cleanup hooks must report failure",
);

const completion = { threadsafe: false, promise: false };
globalThis.napiIntegrationDone = false;
const completeIfReady = () => {
  globalThis.napiIntegrationDone = completion.threadsafe && completion.promise;
};
addon.resolvedPromise().then((value) => {
  assert(value === "native promise", "native promise resolution");
  completion.promise = true;
  completeIfReady();
});

const values = [];
addon.startThread((value) => {
  values.push(value);
  if (values.length === 3) {
    assert(values.join(",") === "1,2,3", "threadsafe function calls");
    completion.threadsafe = true;
    completeIfReady();
  }
});
