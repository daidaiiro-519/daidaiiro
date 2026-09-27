// テスト契約の補助関数: SCENARIO_TRACE が設定されていれば、検査するシナリオIDを1行追記する。
import { appendFileSync } from "node:fs";

export function traceScenario(scenarioId) {
  const path = process.env.SCENARIO_TRACE;
  if (!path) return;
  appendFileSync(path, `${scenarioId}\n`);
}
