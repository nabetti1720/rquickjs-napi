// SPDX-License-Identifier: MIT OR Apache-2.0
const addon = require("./fixtures/napi/addon.node");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function equal(actual, expected, message) {
  assert(Object.is(actual, expected), `${message}: expected ${expected}, got ${actual}`);
}

function arrayEqual(actual, expected, message) {
  assert(actual.length === expected.length, `${message}: length mismatch`);
  for (let i = 0; i < expected.length; i++) {
    equal(actual[i], expected[i], `${message}[${i}]`);
  }
}

async function main() {
  equal(addon.add(2, 3), 5, "native callback");
  equal(addon.greet("World"), "Hello, World!", "UTF-8 string callback");
  equal(addon.callTwice(() => 42), 42, "calling a JS callback");
  equal(addon.escaped(), 7, "escapable handle scope");
  equal(addon.hidden, 7, "non-enumerable property");
  assert(!Object.keys(addon).includes("hidden"), "hidden property must not enumerate");
  assert(Object.keys(addon).includes("visible"), "visible property must enumerate");

  const removable = { value: 1 };
  equal(addon.hasDel(removable, "value"), 111, "has/has-own/delete property");
  assert(!Object.hasOwn(removable, "value"), "property should be deleted");
  assert(Object.isFrozen(addon.freezeObject({})), "native object freeze");
  assert(Object.isSealed(addon.sealObject({})), "native object seal");

  class Parent {}
  const child = new Parent();
  equal(addon.isInstance(child, Parent), true, "instanceof");
  assert(new addon.Sample() instanceof addon.Sample, "native class construction");
  equal(addon.counter(), 1, "instance data first access");
  equal(addon.counter(), 2, "instance data second access");

  const buffer = addon.arrayBuffer(8);
  const views = addon.views(buffer);
  assert(views[0] instanceof Uint16Array, "typed array creation");
  assert(views[1] instanceof DataView, "DataView creation");
  equal(views[0].length, 4, "typed array length");
  equal(views[1].byteLength, 8, "DataView length");
  equal(views[2], 81028, "array buffer contents and view info");

  const externalArrayBuffer = new Uint8Array(addon.externalArrayBuffer());
  externalArrayBuffer[0] = 41;
  equal(addon.externalArrayBufferValue(), 41, "external ArrayBuffer shares backing memory");

  const externalBuffer = addon.externalBuffer();
  externalBuffer[0] = 42;
  equal(addon.externalBufferValue(), 42, "external Buffer shares backing memory");
  equal(addon.externalFinalizeCount(), 0, "external finalizers must not run at creation");

  const coerced = addon.coerce("12");
  equal(coerced[0], 12, "number coercion");
  equal(coerced[1], true, "boolean coercion");
  equal(typeof coerced[2], "symbol", "symbol creation");
  equal(coerced[3], "12", "string coercion");
  equal(String(coerced[4]), "12", "object coercion");
  equal(addon.strictEquals(NaN, NaN), false, "strict equality with NaN");
  const sameObject = {};
  equal(addon.strictEquals(sameObject, sameObject), true, "strict equality with objects");
  equal(addon.isPromise(Promise.resolve()), true, "Promise detection");
  equal(addon.isPromise({}), false, "non-Promise detection");
  equal(addon.isArrayBuffer(addon.arrayBuffer(4)), true, "ArrayBuffer detection");
  equal(addon.isArrayBuffer(new Uint8Array(4)), false, "non-ArrayBuffer detection");
  equal(addon.isError(new Error("error")), true, "Error detection");
  equal(addon.isError(new TypeError("type error")), true, "TypeError detection");
  equal(addon.isError({}), false, "non-Error detection");

  const napiBuffer = addon.createBuffer(4);
  equal(addon.isBuffer(napiBuffer), true, "N-API buffer detection");
  equal(addon.isBuffer(new Uint8Array(4)), false, "unmarked Uint8Array is not a Buffer");
  equal(addon.isBuffer(new Uint16Array(2)), false, "typed array is not a Buffer");
  equal(addon.isBuffer(new DataView(new ArrayBuffer(4))), false, "DataView is not a Buffer");
  equal(addon.bufferInfoStatus(napiBuffer), 0, "buffer info accepts a Buffer");
  equal(addon.bufferInfoStatus(new Uint8Array(4)), 1, "buffer info rejects an unmarked view");
  equal(addon.bufferInfoStatus(new DataView(new ArrayBuffer(4))), 1, "buffer info rejects DataView");
  equal(await addon.resolvedPromise(), "native promise", "native deferred resolution");

  const bigint = addon.bigints(5n);
  equal(bigint[1], 10n, "BigInt int64 round trip");
  equal(bigint[0], -36893488147419103233n, "BigInt creation from words");
  equal(bigint[2], 1, "BigInt words count");
  equal(bigint[3], true, "BigInt lossless flag");
  equal(bigint[4], 0, "BigInt sign");
  equal(bigint[5], 5, "BigInt low word");
  equal(bigint[6], 5n, "BigInt uint64 conversion");
  equal(bigint[7], true, "BigInt uint64 lossless flag");
  const largeBigint = addon.bigints(18446744073709551621n);
  equal(largeBigint[1], 10n, "BigInt int64 wraps to 64 bits");
  equal(largeBigint[2], 2, "large BigInt words count");
  equal(largeBigint[3], false, "large BigInt is not lossless as int64");
  equal(largeBigint[5], 5, "large BigInt low word");
  equal(largeBigint[6], 5n, "large BigInt uint64 wraps to 64 bits");
  equal(largeBigint[7], false, "large BigInt is not lossless as uint64");
  const negativeBigint = addon.bigints(-18446744073709551623n);
  equal(negativeBigint[1], -14n, "negative BigInt int64 wraps to 64 bits");
  equal(negativeBigint[4], 1, "negative BigInt sign");
  equal(negativeBigint[5], 7, "negative BigInt magnitude low word");
  equal(negativeBigint[6], 18446744073709551609n, "negative BigInt uint64 wraps to 64 bits");

  const misc = addon.misc();
  equal(misc[0].getTime(), 86400000, "Date creation");
  equal(misc[1], 86400000, "Date value");
  equal(misc[2], 8042, "external value and typeof");
  equal(misc[3], "café", "Latin-1 string creation");
  equal(misc[4], 4233, "UTF-16 string extraction");
  equal(misc[5], 7, "script execution");

  const base = { inherited: true, suppressed: true };
  const prototype = Object.create(base);
  Object.defineProperty(prototype, "suppressed", { value: true });
  const object = Object.create(prototype);
  object.own = 1;
  object[2] = "index";
  object[Symbol("ignored")] = 3;
  arrayEqual(addon.allKeys(object), [2, "own", "inherited"], "all property names");
  arrayEqual(
    addon.propertyNames(object),
    ["2", "own", "inherited"],
    "enumerable property names",
  );

  const threadValues = [];
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("threadsafe function timed out")), 5000);
    addon.startThread((value) => {
      threadValues.push(value);
      if (threadValues.length === 3) {
        clearTimeout(timeout);
        resolve();
      }
    });
  });
  arrayEqual(threadValues, [1, 2, 3], "threadsafe function calls");
  equal(addon.tsfnFinalizeCount(), 1, "unref'd threadsafe function finalizer");

  console.log("napi-cjs: ok");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
