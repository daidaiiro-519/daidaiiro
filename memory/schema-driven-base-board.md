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
- 13回目：論点3を B″ で出し直し、承認待ち。基盤は描画の部品（見本 viewer.html の見せ方）と頁の型を読んで組む道具、concrete は種類ごとの頁の型（sample/pages/*.page.json）をデータで書く。頁の UI ・ UX は見本の水準を下回らない（利用者の指摘：注釈だけの汎用描画 page7 は見本に届かない）。サブドメインの頁2枚が compose7 で見本と同じ HTML になった。条件の文は x-view の文の型で64件一致
- 前提に足した：スキーマ駆動は新しく定義する概念で、すべての基盤として一から作る。いまの Skill は基盤より前の試しで、根拠にしない（基盤のあとで移す）
- 論点4の前提：基盤から作るものには、基盤の能力の複製を転写する（skills-creator の雛形の契約と同じ形）
- 試作 `trial/spike/` は契約の動作確認だけに使う
