"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const {
  STORAGE_KEY,
  createOrder,
  advanceOrder,
  markPartial,
  retryOrder,
  seedOrders,
  readOrders,
  saveOrders,
} = require("../app.js");

function memoryStorage(initialValue = null) {
  const entries = new Map();
  if (initialValue !== null) entries.set(STORAGE_KEY, initialValue);
  return {
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => entries.set(key, value),
  };
}

test("new orders trim titles, receive unique IDs, and start queued", () => {
  const first = createOrder("  Ship the dashboard  ");
  const second = createOrder("Ship the dashboard");
  assert.equal(first.title, "Ship the dashboard");
  assert.equal(first.status, "queued");
  assert.equal(first.stage, null);
  assert.notEqual(first.id, second.id);
});

test("blank and non-text titles are rejected", () => {
  for (const title of ["", " \n\t ", null, undefined, 123]) {
    assert.throws(() => createOrder(title), /Describe the order first\./);
  }
});

test("four advances traverse the workflow and finish ready", () => {
  const original = createOrder("Ship the dashboard");
  let order = original;
  for (const [stage, status] of [
    ["plan", "running"],
    ["sangha", "running"],
    ["implement", "running"],
    ["review", "ready"],
  ]) {
    order = advanceOrder(order);
    assert.equal(order.stage, stage);
    assert.equal(order.status, status);
    assert.equal(order.title, original.title);
    assert.equal(order.id, original.id);
  }
  assert.equal(original.status, "queued");
  assert.throws(() => advanceOrder(order), /cannot advance/i);
});

test("partial and retry retain each running stage across storage reloads", () => {
  let order = createOrder("Check recovery");
  const storage = memoryStorage();
  for (const stage of ["plan", "sangha", "implement"]) {
    order = advanceOrder(order);
    const paused = markPartial(order);
    assert.equal(paused.status, "partial");
    assert.equal(paused.stage, stage);
    assert.equal(order.status, "running");
    assert.throws(() => advanceOrder(paused), /cannot advance/i);
    saveOrders(storage, [paused]);
    order = retryOrder(readOrders(storage)[0]);
    assert.equal(order.stage, stage);
    assert.equal(order.status, "running");
  }
  assert.equal(advanceOrder(order).status, "ready");
});

test("actions reject inapplicable order states", () => {
  const queued = createOrder("Check guards");
  assert.throws(() => markPartial(queued), /running/i);
  assert.throws(() => retryOrder(queued), /partial/i);
  const ready = seedOrders().find((order) => order.status === "ready");
  assert.throws(() => markPartial(ready), /running/i);
  assert.throws(() => retryOrder(ready), /partial/i);
});

test("first use seeds exactly one ready and one partial order", () => {
  const orders = readOrders(memoryStorage());
  assert.equal(orders.length, 2);
  assert.deepEqual(orders.map((order) => order.status).sort(), [
    "partial",
    "ready",
  ]);
  assert.equal(new Set(orders.map((order) => order.id)).size, 2);
});

test("stored orders survive reload and an empty saved list stays empty", () => {
  const storage = memoryStorage();
  const orders = [createOrder("Keep this order")];
  saveOrders(storage, orders);
  assert.deepEqual(readOrders(storage), orders);
  saveOrders(storage, []);
  assert.deepEqual(readOrders(storage), []);
});

test("user text survives storage as literal text", () => {
  const title = '<img src=x onerror="alert(1)"> & <script>bad()</script>';
  const storage = memoryStorage();
  saveOrders(storage, [createOrder(title)]);
  assert.equal(readOrders(storage)[0].title, title);
});

test("corrupt or invalid saved data is reported and left untouched", () => {
  const valid = createOrder("Valid", "order-1");
  for (const raw of [
    "{broken",
    "null",
    "{}",
    "[null]",
    JSON.stringify([{ ...valid, title: " " }]),
    JSON.stringify([{ ...valid, status: "unknown" }]),
    JSON.stringify([{ ...valid, status: "ready", stage: "plan" }]),
    JSON.stringify([{ ...valid, status: "partial", stage: null }]),
    JSON.stringify([{ ...valid, status: "running", stage: "missing" }]),
    JSON.stringify([{ ...valid, stage: "plan" }]),
    JSON.stringify([valid, valid]),
  ]) {
    const storage = memoryStorage(raw);
    assert.throws(() => readOrders(storage), /saved orders/i);
    assert.equal(storage.getItem(STORAGE_KEY), raw);
  }
});

test("read and write failures propagate so the desk can explain them", () => {
  assert.throws(
    () =>
      readOrders({
        getItem() {
          throw new Error("Storage blocked");
        },
      }),
    /Storage blocked/,
  );
  assert.throws(
    () =>
      saveOrders(
        {
          setItem() {
            throw new Error("Storage full");
          },
        },
        [],
      ),
    /Storage full/,
  );
});
