---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-18、第66回）

- 基盤を最小の形にした：契約 ・ instance ・ 機械の検証（計算し直す関数）・ 判定の記録（AI と人の判定。入力のハッシュに結び付き、変更に入れて残す）。承認 ・ 版の状態 ・ 名簿 ・ 受信箱 ・ 信頼できる実行者は外した。
- 不変条件 M1〜M10 と試験18本は measured/model/（model.py ・ tests.py ・ invariants.json）。前のモデルは measured/model-r65/。
- 回答待ち：Q10（チームで使うとき。基盤は変更後の姿の検証と、人が見るものを出すだけ。Git はアダプター）。Q1 は Q10 を前提にして待つ。
- 決着：Q0 ・ Q5 ・ Q8 ・ Q9。前提待ち：Q6（一員6つ）・ Q7 ・ Q2 ・ Q3（ポート5つ）・ Q4。
- 未決：AI に何回聞くか ・ 時間切れの値（validator の割り当てに書く案）、判定の記録が増えたときの読む速さ。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
