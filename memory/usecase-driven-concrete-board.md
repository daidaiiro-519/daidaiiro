---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-04、ボード22回目、未 push）：
- 論点1〜7 すべて決着。論点7 A″：アプリケーション層の操作を外し、要求と設計は宣言の要素どうしの従属関係でつなぐ（文脈はサブドメインを「対象とする」。包含ではない）。手順とコードの対応は仕様に書かず、宣言の各種類から取り出したテスト条件をテストが確かめる。ACDR 0114 承認済み
- 見本のスキーマ sample/schema/（build.py で組み、check.py で検査）：宣言9種類 ＋ 共通の形 ＋ 記録の1行 ・ 承認した時点の記録 ・ 移行の記録。見本と fact-check の試しはどれも検出0件
- 次にすること（ボードの現在地の表が正本）：欠けを数える道具（mig6）と入れ子の拡張の番号を道具へ ・ 出どころごとの承認画面 ・ スキーマと道具を正本の Skill へ入れる ACDR
- 保留：役割の分担（どちらの Skill が設計の側の宣言を書くか）は usecase-modeling-coding で決める
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）と実行の記録 sample/realsim/
- 関連：[[schema-driven-base-board]] の論点3は保留
