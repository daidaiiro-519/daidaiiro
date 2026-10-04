---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード16回目、e43779c6、未 push）：
- 論点1〜5 は決着。ACDR 0107 ・ 0108 は承認 ・ 適用済み。集約のキーは business_rules（BR-n）・ state_changes（CHG-n）・ results（RES-n）に改めた（a4ba8124）
- 論点6（セマンティックマイグレーション）は A′ で承認待ち：移す作業＝既存の材料から、スキーマの x-prompt.write（このやり方で書くための基準）どおりに欄を埋める。移すときだけ足すのは出どころ（spec/migration.json）・既存のテストの扱い・移す範囲の3つ。回答は advice/mig/
- 承認されたら：要求事項（論点3の欄をスキーマのファイルに起こし x-prompt.write を書く、migration.json の形、出どころごとの承認画面）を ACDR へ。共通 view トークンの不足は ux-advisor の修正後に ACDR で直す
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）と実行の記録 sample/realsim/
- 関連：[[schema-driven-base-board]] の論点3は保留
