---
name: usecase-driven-concrete-board
description: ブレストボード「ユースケース駆動の concrete の宣言」の現在地
metadata:
  type: project
---

`.brainstorming-board/usecase-driven-concrete/`（Artifact https://claude.ai/artifact/RCzv1XstzQ3Y5PHtFsT5Gr ）。宣言の種類ごとの欄と、AI が書いても出力が安定する書き方を、ddd-advisor と usecase-advisor の回答（`advice/` に保存）から決める。試作（trial）は根拠にしない。

現在地（2026-10-03、9回目）：
- 論点1 は決着。論点2（書き方。C に組み直し：意味はすべて欄、自由文は説明だけ。回答 advice/q2b/）・ 論点3（B″：6種類にも原則を当てはめ、例の構造 ・ 測れる基準。回答 advice/q3b/）・ 論点4 ・ 論点5 が回答待ち。論点6（セマンティックマイグレーション＝既存の文書から宣言を起こす）は待ち
- 完成イメージ sample/ は論点2 C ・ 論点3 B″ の形に書き直し済み（コミット 71d9bf21）。生成は data5 ・ gen5（文 ・ 例 ・ 境界値を組む）・ drift5 ・ spec5 ・ sim5 ・ render5。論点2・3 の決着は利用者の承認待ち
- 論点4（A′）：テストの合否は受け取らない。テストは走ったときに ID ・ ハッシュ値 ・ レベルを記録ファイル（CONCRETE_TRACE）へ1行追記し、道具は宣言と記録だけで欠け ・ 余り ・ 古い ・ レベル違いを出す。Python と Go で実際に流した記録は sample/realsim/（Artifact https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY ・ 仕組みの頁 https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh ）
- 論点5：ずれを4つ（構造で検査 ・ 対応の欄で検査 ・ 上流の変更 ・ 人のレビュー）に分ける。ユースケースに handles ・ ensures ・ established_by ・ keeps を足し、guarantees_hold を外す。上流の変更は参照される項目ごとのハッシュ値を承認時に記録し、1段だけ伝える。回答は advice/q5/
- 完成イメージ sample/（Artifact https://claude.ai/artifact/NdRwdgjvW1DJrV9RaDFFTJ ）の「突き合わせ」に、宣言どうし（spec_drift.py）と宣言とテスト実装（drift.py）の2つの検査がある。見本の宣言の13件のずれは検知の見本として残している
- 完成イメージから合否を外し、記録1行のスキーマ（sample/trace.schema.json）にした。次：論点2〜5 が承認されたら ACDR を起こす（usecase-modeling-coding 論点1 の writes→invokes、論点7 のサーガのテストと集約のレベル、ACDR 0093 を記録の契約へ改訂）。design-svg の exchange の自分宛てメッセージ（作業コピー、未コミット）も ACDR が要る。qa-advisor の validate が findings があっても ok:true を返す不具合は未報告
- 関連：[[schema-driven-base-board]] の論点2はこのボードの決定に合わせて組み直し、論点3は保留
