---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、3回目）：
- 論点1（宣言の構成）・ 論点2（書き方）は決着。ACDR 0094（usecase-modeling-coding の表）・ 0095（述部を漢語にする規則を外す）は承認・適用済み
- 論点3（種類ごとの項目）は B′ で回答待ち：3つのアドバイザー（usecase ・ ddd ・ ux）の見直しを反映。導ける項目（整合性の境界 ・ 書き換える集約など）を外し、実装の都合（識別子の型 ・ データの型 ・ サーガ）を実装側へ、競合 ・ 拒否の例 ・ 所属するサブドメイン ・ 端数の扱いを足し、画面で見せる/畳む項目を決めた。見直しの全文は advice/review/
- 次：論点3が承認されたら、usecase-modeling-coding の論点1（書き換える集約を ID で指す）・ 論点7（サーガのテスト）を見直す ACDR を起こし、論点4（テストとのつなぎ方。テストの ID、テストのレベル、指摘3件）を開く。qa-advisor（required）を呼ぶ必要がある。アドバイザーの2つの例は ID の付け方がそろっていないので、論点4のあと1つの例に書き直す
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
