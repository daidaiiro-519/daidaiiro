// 集約「出荷指示」(AGG-01J7Q4K) のシナリオを検査する。
import { test } from "node:test";
import assert from "node:assert/strict";
import { Order } from "./order.mjs";
import { traceScenario } from "./scenario-trace.mjs";

test("SC-01J7Q4M 明細が0件のとき、確定できない", () => {
  traceScenario("SC-01J7Q4M");
  // given: 作成する
  const order = new Order([]);
  assert.equal(order.status, "準備中");
  // when: 確定する / then: 失敗する（事前条件 lines.length > 0 に違反）
  assert.throws(() => order.confirm());
  assert.equal(order.status, "準備中");
});

test("SC-01J7Q4N 明細が1件あれば、確定できる", () => {
  traceScenario("SC-01J7Q4N");
  // given: 作成する
  const order = new Order([]);
  // given: 明細を足す
  order.lines.push({ qty: 1 });
  assert.equal(order.lines.length, 1);
  // when: 確定する
  order.confirm();
  // then: 失敗せず、status == 確定済
  assert.equal(order.status, "確定済");
});
