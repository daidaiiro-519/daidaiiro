---
name: semantic-contract-core-board
description: 構想 Semantic Contract Core v0.3 を、いまの基盤 schema-driven とどう関係させるかを決めるボードの現在地
metadata:
  type: project
---

## 現在地（2026-10-17）

- 利用者が構想 Semantic Contract Core v0.3（リポジトリの根の semantic-contract-core-v0.3.zip。写しは .brainstorming/semantic-contract-core/sources/v0.3/、git では追跡しない）を出し、基盤の次の形かどうかを含めて進め方を相談した。
- 決めた進め方：新しいボード semantic-contract-core を立てる。論点 Q0 v0.3 でできるようになること ／ Q1 位置づけ（基盤の次の形・上の層・横の層）／ Q2 Semantic と最小単位 ／ Q3 正本の形（JSON Schema を土台のまま広げるか、自前のモデルか）／ Agents 契約 ／ Q4 移し方 ／ Q5 完成の条件と最初の試作。
- 利用者の補足：基盤が Agents 契約（AI をプロセスで呼ぶ契約）を持つという概念は、v0.3 でも必要で、v0.3 に足りない部分である。
- ボード agents-in-base は保留にした（hold、理由をボードに記録）。新しいボードの Q1 が決着したら再開を判断する。
- 論点を出す前の調べ：fact-check（JSON Schema 2020-12 の $vocabulary ・ JSON-LD ・ SHACL を原文で照合。反証の仮説「JSON Schema の Vocabulary の仕組みの中で独自の意味を足せる」）・ 4つのアドバイザー（meta-thinking ・ ddd ・ platform ・ qa、回答は advisors/）・ 小さな実験（brainstorming のボードの決まりを v0.3 の形で書き直して数える、measured/v03-experiment/）。

## 次にすること

第26回まで出した（2026-10-17）。いま見る論点は Q0 目的と設計原則と、Q5 用語（validator ・ 検証結果 ・ capability ・ インバウンド ・ アウトバウンドアダプター ・ ルール ・ Semantic Validator を提案）。Q1〜Q4 は Q0 と Q5 が前提。構想 v0.3 は利用者が Codex でブレストした検討中の案で、原典ではない（根拠の種類は「前提」）。Q5 が承認されたら、ほかの論点の名前をその表に揃える（いまの Interpreter ・ outcome ・ inbound などは仮の名前）。

決まったこと：基盤のインターフェースはコマンドと protocol（CLI ・ MCP）。インバウンド ・ アウトバウンドのアダプターを差し替えて、手元でもクラウドでも同じ振る舞い（クラウドは後で載せる）。HTTP などの API は Concrete が持つ。WASM は外した。

持ち越し：AI ごとの許可の設定は、いま brainstorming（Concrete）が一時に持つ（ACDR 0188）。Q3 で Agents 契約が決まったら形を基盤へ移す。ボードの更新は figures/board_r<回>.py と apply_patch.py（引数の上限を超えないように分けて渡す）。
