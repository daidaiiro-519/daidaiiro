---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、2回目）：
- 論点1を B′ に組み直し、回答待ち。抽象の基盤の契約は3つ（スキーマ検証 ・ Markdown と HTML への描画 ・ 項目ごとの x-prompt の read と write）。concrete は同じ3つを特化した表現で持ち、ユースケース駆動の concrete はテスト仕様（シナリオと条件）とテスト実装のドリフト検知の契約を足す
- 1回目の答え（kind ・ id ・ name ・ body と JSONPath の在りかの申告）は、基盤が宣言の構造を知るので撤回した（前の答えとしてボードに残る）
- 論点2（concrete の契約）・ 3（描画の持ち方）・ 4（配り方と名前）は待ち

抽象の3つの契約は、既存の Skill の references の契約の版2（validate ・ view ・ x-view）と no-more-spaghetti の schema-meta.schema.json（x-prompt の read ・ write）に実装がある。
