---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-18、第67回）

- ドメインモデルを使う場面（Q8）と業務領域（Q9）から導き直した案を Q1 に入れた：境界 ・ 拡張の契約 ・ 集約4つ（Semantic Abstract Schema ・ Semantic Concrete Schema ・ instance ・ 判定の記録）・ ドメインサービス12 ・ 駆動する側と駆動される側のポート ・ 不変条件29行。正本は measured/domain-model-v2.json。
- 利用者の決定：Concrete は x- を拡張できる（Core が拡張の仕方を言語に依らない契約で決める）。x- は JSON Schema で検証できない部分のためにある。判定の記録は判定した値を残す（ハッシュにしない）。外のツールは tool permissions に外出し。ほかの文脈へ見せる部分だけ版を持つ。Q9 の中核は「変更の確かめと例外の判断」。
- 回答待ち：Q10。Q1 は Q10 待ち。未決は D6（実行できない検証の仕方の返し方）だけ。
- 実行できるモデル measured/model/ は第66回の形のままで、案と食い違う。作り直す。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
