---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード15回目）：
- 論点1〜5 は決着。ACDR 0107（テストとのつなぎを合否を持たない記録の契約に）・ 0108（usecase-modeling-coding の論点1 ・ 7 を concrete の決着に）は承認 ・ 適用済み（eba25746、未 push）
- 次：論点6（セマンティックマイグレーション＝既存の文書から宣言を起こす）を開く。集約のスキーマのキーを validations ・ state_changes に改める。共通 view トークンの不足（地の段 ・ 強い線 ・ 差し色と警告の明るさ）は ux-advisor の修正後に ACDR で直す
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ 、amber、スマホ ・ PC 対応済み）と実行の記録 sample/realsim/
- 関連：[[schema-driven-base-board]] の論点2は ACDR 0107 で記録の契約に改めた。論点3は保留
