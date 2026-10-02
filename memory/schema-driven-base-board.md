---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、6回目）：
- 論点1・2 は決着。基盤の契約は7つ（作成 ・ 取得 JMESPath ・ 更新と削除 JSON Patch ・ 検証 ・ 描画 ・ 案内）。concrete は Schema の注釈2つ（x-test-spec ・ x-ref）、宣言の集合の検証（Schema ・ ID の一意 ・ 参照の解決）、ID は「宣言の ID.項目の ID」、テストの報告は JSON（id と result）、ドリフトは欠け ・ 余り ・ pass でないもの
- ACDR 0093（9月のボードの論点8と論点11の行を改める）を提示中・承認待ち（Artifact https://claude.ai/artifact/7Y53u1vXeixyAYwr8EKByA ）。改めた usecase-schema-driven/board.json は作業ツリーにあり未コミット。承認されたら accepted にして組み直し、9月のボードの board.html も render してコミット
- その次：論点3（描画の持ち方。検証の違反の文が英語・参照をたどれる描画を含む）を開く。論点4（配り方と名前）は待ち
- 試作 `trial/spike/`：引数なしで論点1の10手順（output.txt）、`-- q2` で論点2の7手順（output-q2.txt）
- crate（jmespath 0.5.0 ・ json-patch 4.2.0 ・ jsonschema 0.58.4）の仕様への準拠の範囲は未照合
