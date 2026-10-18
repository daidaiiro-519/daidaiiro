---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-18、第60回）

- 決着：Q0 ・ Q5 ・ Q8 ・ Q9。第60回で Q8 ・ Q9 を利用者の判断で改めた（U0 の成功＝最新の公開した版で全 rule が verified、U9＝契約の承認、U16＝判定できなかった rule の判断、U17＝取り込み、U15 は外した。中核は「契約の承認と例外の判断」）。
- 回答待ち：Q1。人が承認するのは Concrete の版と validator の割り当て（Concrete に1つ）だけ、人が判断するのは判定できなかった rule と AI の判定（覆す）だけ。状態の単位は rule の結果。集約4つ ・ 不変条件 J1〜J27。表 ・ 決定表 ・ 試験22本は measured/model/（model.py ・ gen.py ・ tests.py）から生成する。図は figures/make18.py。
- 反証は、別の文脈の4つの advisor（meta ・ ddd ・ ux ・ qa）に4回かけた。回答は advisors/r60-*.json。
- Q1 の未決：error を unverified にする回数 ・ 抜き取りの件数 ・ 外の AI が同じ名前でモデルを替えたことの検出（platform-advisor へ）・ 承認のあとに AI の誤りが見つかったときの残り（O5）。
- 前提待ち：Q6（一員7つ）・ Q7（$vocabulary）・ Q2 ・ Q3（集約4つ ・ ポート6つ）・ Q4。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
