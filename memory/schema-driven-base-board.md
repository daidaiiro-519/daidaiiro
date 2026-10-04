---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、AI が扱うデータを構造化して持つための基盤（開発に限らない）として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-04、8回目、未 push）：
- 論点1（基盤の7つの契約）は決着。ACDR 0093 は適用済み
- 10回目：論点1を B⁗（開発に限らない AI の構造化データ管理の基盤。正本は構造化データ、Markdown と HTML は描画の結果。7つの契約 ＋ 注釈 x-ref ・ x-derive を読む道具）、論点2を B‴（x-test-spec とテストの記録の突き合わせは開発の concrete）に開き直し、承認済み（11回目で決着）。見本の道具は base7.py（基盤）と concrete7.py（開発）に分けた
- 12回目：論点3を B′ で出し直し、承認待ち（描画は基盤がすべて持つ。文の組み方は x-view の文の型、目的の向きは x-ref の purpose）。見本 sample/text7.py が条件64件を gen6 と同じ文にした。注釈で書けなかった検査のうち2つは x-derive の expect と inverse の where_derive で書けた。残る例を組む処理は開発の concrete
- 前提に足した：スキーマ駆動は新しく定義する概念で、すべての基盤として一から作る。いまの Skill は基盤より前の試しで、根拠にしない（基盤のあとで移す）
- 論点4の前提：基盤から作るものには、基盤の能力の複製を転写する（skills-creator の雛形の契約と同じ形）
- 試作 `trial/spike/` は契約の動作確認だけに使う
