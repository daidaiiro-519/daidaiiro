---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、2回目）：
- 論点1（宣言の構成）・ 論点2（書き方）は承認済み
- ACDR 0094（usecase-modeling-coding の表の論点1の行を改める、https://claude.ai/artifact/ApwXuUDmK5DA3AF4aiBomt ）と ACDR 0095（述部を漢語にする規則とゲート1の判定を外す、https://claude.ai/artifact/Uj8U7tthVbSt7rCpyvzJ1B ）を提示中・承認待ち。変更は作業ツリーにあり未コミット
- 0095 を承認したら：predicates.json ・ predicates.schema.json を git rm し、doc-writing-skills の bin を cargo install で組み直す（いまの bin は古い版なので、ファイルを消すと Stop フックが動かない）
- その次：論点3（種類ごとの項目）を開く。論点4（テストとのつなぎ方）は qa-advisor が required
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
