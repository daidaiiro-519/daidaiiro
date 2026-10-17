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

第2回を出した（2026-10-17）。利用者の指摘で、基盤だけを対象に論点を依存の順に組み直した：Q0 基盤の目的と境界（回答待ち）→ Q1 意味と最小単位 → Q2 抽象と具体の関係と正本の形 → Q3 解釈と結果、Agents 契約 → Q4 公開の口 ・ 移し方 ・ 完成の条件。Concrete（brainstorming ・ ACDR）は基盤の決着のあと別のボードで。組み立ては figures/board_r2.py（引用は組み立て時に原文から切り出す）。
