---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、5回目）：
- 論点1は決着（B″）。基盤の契約は7つ ── 操作の4つ（作成 ・ 取得 ・ 更新 ・ 削除）と形の3つ（検証 ・ 描画 ・ 案内）。取得は JMESPath、更新と削除は JSON Patch。concrete はドリフト検知の契約を足す
- 論点2（concrete の契約）は B で回答待ち。Schema に注釈2つ（x-test-spec ・ x-ref）、宣言の集合の検証（Schema ・ ID の一意 ・ 参照の解決）、参照されている宣言は削除しない、テスト仕様の ID は「宣言の ID.項目の ID」、テストの報告は JSON（id と result：pass ・ fail ・ skip）
- 試作 `trial/spike/`：引数なしで論点1の10手順（output.txt）、`-- q2` で論点2の7手順（output-q2.txt）。基盤の更新に concrete の検証を注入する口 update_with を足した
- 論点2が承認されたら：9月の論点11（1行1ID）を JSON の報告へ改訂する ACDR を起こす。次は論点3（描画の持ち方。検証の違反の文が英語・参照をたどれる描画を含む）
- 論点4（配り方と名前）は待ち。crate（jmespath 0.5.0 ・ json-patch 4.2.0 ・ jsonschema 0.58.4）の仕様への準拠の範囲は未照合
