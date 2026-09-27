import { test } from "node:test";
import assert from "node:assert/strict";
import { appendFileSync } from "node:fs";
import { Order } from "./order.mjs";
const scenario = (id) => { const p = process.env.SCENARIO_TRACE; if (p) appendFileSync(p, id + "\n"); };

test("確定", async (t) => {
  await t.test("明細が0件", () => { scenario("SC-01J7Q4M"); assert.throws(() => new Order().confirm()); });
  await t.test("明細が1件", () => { scenario("SC-01J7Q4N"); new Order(["りんご"]).confirm(); });
});
