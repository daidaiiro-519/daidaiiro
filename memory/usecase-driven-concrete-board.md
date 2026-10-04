---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード20回目、未 push）：
- 論点1〜7 すべて決着。論点6 A″ と論点7 A′ は、fact-check の「原文を取得する」を移した試し（sample/migration-trial/fact-check/）で使ってから承認
- 宣言の種類ごとの JSON Schema（10種類 ＋ 共通の形）を sample/schema/ に起こした（build.py で組み、check.py で見本と試しの宣言を検査、どちらも0件）。完成イメージに「書き方（スキーマ）」の頁。正本（Skill）へ入れるのは ACDR のあと
- 次にすること（ボードの現在地の表が正本）： 記録の契約と承認時点の記録の形 ・ spec/migration.json の形 ・ 欠けを数える道具と入れ子の拡張の番号 ・ 出どころごとの承認画面。正本へ入れる前に ACDR
- 保留：役割の分担（どちらの Skill が設計の側の宣言を書くか）は usecase-modeling-coding で決める
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）と実行の記録 sample/realsim/
- 関連：[[schema-driven-base-board]] の論点3は保留
