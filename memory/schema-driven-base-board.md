---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、4回目）：
- 論点1は決着（B″）。基盤の契約は7つ ── 操作の4つ（作成 ・ 取得 ・ 更新 ・ 削除）と形の3つ（検証 ・ 描画 ・ 案内）。取得は JMESPath、更新と削除は JSON Patch。ユースケース駆動の concrete はドリフト検知の契約を足す
- 試作 `trial/spike/`（Rust。base.rs ＝基盤、usecase.rs ＝ concrete）で10の手順を実行し、出力は output.txt。base.rs の本体に宣言の語は0件
- 次にすること：論点2（concrete の契約）を開く。材料は試作で分かった3件 ── 削除で残った参照の扱い ・ テスト仕様の ID の組み方（試作は「宣言の ID.項目の ID」）・ テストの報告の形（試作は1行に ID と pass ・ fail）
- 論点3（描画の持ち方）・ 4（配り方と名前）は待ち。論点3には「検証の違反の文が英語」も渡す
- crate（jmespath 0.5.0 ・ json-patch 4.2.0 ・ jsonschema 0.58.4）の仕様への準拠の範囲は、まだ照合していない
