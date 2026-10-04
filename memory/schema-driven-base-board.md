---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-04、8回目、未 push）：
- 論点1（基盤の7つの契約）は決着。ACDR 0093 は適用済み
- 9回目：論点1を B‴（基盤は開発で使う道具で、ドリフト検知も持つ）、論点2を B″（x-ref ・ x-test-spec ・ x-derive は基盤の注釈。concrete に残るのは頁の組み立てだけ）に開き直し、承認待ち。usecase-driven-concrete の見本（sample/concrete7.py ・ sample/schema/）で試し、道具が持つ名前は id と kind だけになった
- 注釈で書けない検査4つと、文を組む処理 ・ 頁の組み立ては論点3（描画の持ち方）で扱う。論点3は論点2のあとに出し直す
- 試作 `trial/spike/` は契約の動作確認だけに使う
