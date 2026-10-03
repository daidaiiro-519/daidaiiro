---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、3回目）：
- 論点1（宣言の構成）・ 論点2（書き方）は決着。ACDR 0094（usecase-modeling-coding の表）・ 0095（述部を漢語にする規則を外す）は承認・適用済み
- 論点3（種類ごとの項目）は B で回答待ち：7種類の項目の表、1宣言1ファイルで所属は context、判断基準の質問の答えを enum / true-false で持つ、名前は用語集の TERM を指す、キーは英語の snake_case
- 次：論点3が承認されたら論点4（テストとのつなぎ方。テストの ID、テストのレベル、指摘3件）を開く。qa-advisor（required）を呼ぶ必要がある。アドバイザーの2つの例は ID の付け方がそろっていないので、論点4のあと1つの例に書き直す
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
