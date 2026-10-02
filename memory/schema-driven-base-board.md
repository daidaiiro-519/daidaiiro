---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、7回目）：
- 論点1・2 は決着。基盤の契約は7つ（作成 ・ 取得 JMESPath ・ 更新と削除 JSON Patch ・ 検証 ・ 描画 ・ 案内）。concrete は Schema の注釈（x-test-spec ・ x-ref）、宣言の集合の検証、ID は「宣言の ID.項目の ID」、テストの報告は JSON（id と result）
- ACDR 0093（9月の論点8・11の行を改訂）は承認・適用済み（26a79d39）
- 論点3（描画の持ち方）は B で回答待ち。基盤は実体1件と違反を描画（HTML は版2 の view）、concrete は集合を目的の連鎖として頁に組む。上位の目的は x-ref に付ける x-purpose（upward ・ downward）。AI ツールの形へは変換せず、案内と取得で読む
- 論点3が承認されたら：9月の論点2（射影の契約2本）を、AI ツールの形を置かない形へ改訂する ACDR を起こす。次は論点4（配り方と名前）
- 試作 `trial/spike/`：引数なし（論点1）・ q2 ・ q3、出力は output*.txt。基盤に注入の口 update_with ・ render_with を足した
- crate（jmespath 0.5.0 ・ json-patch 4.2.0 ・ jsonschema 0.58.4）の仕様への準拠の範囲は未照合
