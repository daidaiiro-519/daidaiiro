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

第8回を出した（2026-10-17）。完成イメージを論点ごとに厚くし（Q0 承認の前の画面 ・ 境界と向き、Q1 仕様の対応 ・ @v2 の宣言、Q2 自前の語彙 ・ 写しの報告、Q3 足すファイル ・ 結果の enum、Q4 要求と応答 ・ 適合の試験）、画像に名前を付けた（案のファイルは proposals/、組み立ては figures/board_r8.py）。第7回で廃語「口」をやめた。第6回：Rust の仕組み（workspace ・ crate ・ module ・ trait ・ feature ・ enum）を Q3 ・ Q4 に足した（原典は sources/rust/、組み立ては figures/board_r6.py）。基盤の CODING.md の土台にする。第5回：利用者の問い（HTTP ・ WASM ・ JSON-LD などは構想に要る概念か）に答え直し、Q2 を「標準技術の概念と写し」（概念は自前で定義し、標準へは写して相互運用）、Q4 を「protocol と外部インターフェース」（CLI ・ MCP ・ HTTP ・ WASM を同じ protocol の口に）へ直した（figures/board_r5.py）。第4回：全体構成図の層に沿って組み直した：Q0 目的と設計原則（回答待ち）→ Q1 抽象の仕様 → Q2 標準技術アダプターの技術選定 → Q3 エンジンの部品と Agents 契約 → Q4 外部インターフェースと移し方（組み立ては figures/board_r4.py）。第3回：基盤の実物（abstract@v1 ・ 注釈 ・ エンジン）を読み直して、足りている／足りないと完成イメージを実物に合わせた（組み立ては figures/board_r3.py）。第2回：利用者の指摘で、基盤だけを対象に論点を依存の順に組み直した：Q0 基盤の目的と境界（回答待ち）→ Q1 意味と最小単位 → Q2 抽象と具体の関係と正本の形 → Q3 解釈と結果、Agents 契約 → Q4 公開の口 ・ 移し方 ・ 完成の条件。Concrete（brainstorming ・ ACDR）は基盤の決着のあと別のボードで。組み立ては figures/board_r2.py（引用は組み立て時に原文から切り出す）。
