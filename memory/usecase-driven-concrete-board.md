---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、5回目、コミット 90949194）：
- 論点1 ・ 論点2 は決着。論点3（B′＋完成イメージで足した8項目）と論点4（テストとのつなぎ方、A）が回答待ち
- 論点4 の中身：テスト条件（JSTQB）を単位にし、ID「宣言ID.種類-連番」とハッシュ値（期待する結果を決める欄の SHA-256）で報告と突き合わせる。判定は欠け ・ 余り ・ 不合格 ・ 古い ・ 未実行 ・ レベル違い。qa-advisor の回答は advice/qa.json
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）に、テスト条件の ID ・ 突き合わせの頁 ・ report.schema.json がある
- 次：承認されたら ACDR を起こす（usecase-modeling-coding 論点1 の writes→invokes、論点7 のサーガのテストと集約のレベル、ACDR 0093 の報告の形）。design-svg の exchange の自分宛てメッセージ（作業コピー、未コミット）も ACDR が要る。qa-advisor の validate が findings があっても ok:true を返す不具合は未報告
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
