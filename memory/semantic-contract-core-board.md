---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-18、第59回）

- 決着：Q0 ・ Q5 ・ Q8（使う場面 U0〜U15）・ Q9（業務領域と文脈。事業領域はプロダクト、中核は意味の宣言 ・ 意味の検証 ・ 承認と陳腐化の管理 ・ 意味で探す、文脈の境界は1つ）。
- 回答待ち：Q1（抽象のドメインモデル）。第59回で、使う場面から導いた集約5つ（Concrete スキーマ ・ instance ・ 承認の記録 ・ 変更の提案 ・ 検証の記録）・ 不変条件65行 ・ 能力18 ・ 承認の状態 × 出来事の状態表（6 × 10）で組み直した。材料は .brainstorming/semantic-contract-core/measured/domain-model.json、図は figures/make18.py と make14.py。
- 前提待ち：Q6（一員8つ）・ Q7（$vocabulary を採ると書いた）・ Q2 ・ Q3（ポート7つ）・ Q4。Q1 が決着したら、Q7 ・ Q3 ・ Q6 の順に開く。
- Q1 の未決（domain-model.json の open）：時間切れの値 ・ 答えが割れたかを見る回数 ・ 確かめ直しの検証のやり直しの時期 ・ まとめて承認し直す範囲 ・ 承認の依頼を出した instance を止めるか。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
