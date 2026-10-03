---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、7回目、コミット 9988195e）：
- 論点1 は決着。論点2（書き方。C に組み直し：意味はすべて欄、自由文は説明だけ。回答 advice/q2b/）・ 論点3（B′）・ 論点4 ・ 論点5 が回答待ち。論点6（セマンティックマイグレーション＝既存の文書から宣言を起こす）は待ち
- 完成イメージに「シミュレーション」の頁（sample/sim.py、JSON Patch で9手）がある。論点2 C に合わせた見本の書き直し（手順3を2つに分ける、手順の data ・ verb ・ checks、拡張の fails）は未着手
- 論点4：テスト条件を単位に、ID とハッシュ値で報告と突き合わせる。終了基準は「宣言どうしのずれ0件」を前提にする
- 論点5：ずれを4つ（構造で検査 ・ 対応の欄で検査 ・ 上流の変更 ・ 人のレビュー）に分ける。ユースケースに handles ・ ensures ・ established_by ・ keeps を足し、guarantees_hold を外す。上流の変更は参照される項目ごとのハッシュ値を承認時に記録し、1段だけ伝える。回答は advice/q5/
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）の「突き合わせ」に、宣言どうし（spec_drift.py）と宣言とテスト実装（drift.py）の2つの検査がある。見本の宣言の13件のずれは検知の見本として残している
- 次：承認されたら ACDR を起こす（usecase-modeling-coding 論点1 の writes→invokes、論点7 のサーガのテストと集約のレベル、ACDR 0093 の報告の形）。design-svg の exchange の自分宛てメッセージ（作業コピー、未コミット）も ACDR が要る。qa-advisor の validate が findings があっても ok:true を返す不具合は未報告
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
