---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード14回目、完成イメージは bf0b1e1b）：
- 論点1 は決着。論点2（C：意味はすべて欄）・ 論点3（種類ごとの項目。設計スコープ ・ 区切られた文脈の欄 ・ 中核以外のサブドメインの組み立て ・ 集約のコマンドは原典の語「業務ルール ・ 状態の変更」）・ 論点4（A′：記録の契約）・ 論点5（宣言どうしのずれ）が承認待ち。論点6（セマンティックマイグレーション）は待ち
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）は利用者が見やすいと確認済み。配色は共通 view トークンの amber、スマホ ・ PC で崩れなし。生成は data5 ・ gen5 ・ drift5 ・ spec5 ・ render5。実行の記録 sample/realsim/（Python と Go で9手）
- 次：論点2〜5 の承認を受けたら ACDR を起こす（0093 を記録の契約へ改訂、usecase-modeling-coding 論点1 の writes→invokes ・ 論点7 のサーガのテストと集約のレベル）。集約のスキーマのキーを validations ・ state_changes に改める。共通 view トークンの不足（地の段 ・ 強い線 ・ 差し色と警告の明るさ）は ux-advisor の修正後に ACDR で直す
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
