---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード19回目、未 push）：
- 19回目：論点6 A′ を承認後、fact-check の「原文を取得する」を移して試した（sample/migration-trial/fact-check/、ずれ0件 ・ 欠け8件を mig6 が数える）。直した論点6 A″（出どころの境目 ・ assert 単位 ・ 欠けは道具が数える）と、開き直した論点7 A′（拡張の別の道筋での成功 ・ 終わり方「成功」「終了」・ ends_with ・ データ要求と measure: length ・ してはならないは成功時保証）が承認待ち。助言は advice/q7b/。見本の道具の不具合4件は直した
- 論点7（宣言の欄の決め直し）決着。見本を data6 ・ gen6 ・ spec6 ・ drift6 ・ render6 で作り直した（2352178c）。検査は合格72 ・ レビュー1、ずれ7種類を両方の向きで検知
- 論点1〜5 は決着。ACDR 0107 ・ 0108 は承認 ・ 適用済み。集約のキーは business_rules（BR-n）・ state_changes（CHG-n）・ results（RES-n）に改めた（a4ba8124）
- 論点6（セマンティックマイグレーション）は A′ で承認待ち：移す作業＝既存の材料から、スキーマの x-prompt.write（このやり方で書くための基準）どおりに欄を埋める。移すときだけ足すのは出どころ（spec/migration.json）・既存のテストの扱い・移す範囲の3つ。回答は advice/mig/
- 論点6の要求事項に「x-prompt.write は助言役なしで足りる」を足した（ACDR 0109）。承認されたら：要求事項（論点3の欄をスキーマのファイルに起こし x-prompt.write を書く、migration.json の形、出どころごとの承認画面）を ACDR へ。共通 view トークンの不足は ux-advisor の修正後に ACDR で直す
- 見本（7ff9441e）：範囲の In/Out ・ エンティティの開閉 ・ サブドメインの問いをスキーマ sample/schema/subdomain.schema.json（title が問い、x-derive が判定の決まり）へ移し、宣言は答えだけ ・ 表の見た目（sample4.css 末尾）。画面の文面に書名 ・ 章 ・ 頁を出さない
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）と実行の記録 sample/realsim/
- 関連：[[schema-driven-base-board]] の論点3は保留
