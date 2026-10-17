---
name: semantic-contract-core-board
description: Abstract スキーマの基盤 Semantic Schema Core（いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
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

第41回まで出した（2026-10-17）。基盤の名前は Semantic Schema Core に決めた（利用者が OK）。このボードは Abstract スキーマの基盤だけを決め、開発基盤（コード生成 ・ 言語の Adapter ・ AI の修正ループ、ボード ai-driven-process の話）は上に載る Concrete の側に置く。Q0 ・ Q5 は決着。いま答えを待っているのは Q1 の確認3つ（Concrete の意味 ・ 注釈の呼び名を x- keyword に ・ ユビキタス言語を <文脈>/terms@v1 の $defs と $ref で）。Q6 ・ Q7 ・ Q2〜Q4 は Q1 待ち。Q1 が決まったら Q6 ・ Q7 の vocabulary ・ x-term ・ x-vocabularies を terms@v1 に置き換える。いまのエンジンは別のスキーマへの $ref を解決できない（実測）。実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。構想 v0.3 は利用者が Codex でブレストした検討中の案で、原典ではない。

決まったこと：基盤のインターフェースはコマンドと protocol（CLI ・ MCP）。インバウンド ・ アウトバウンドのアダプターを差し替えて、手元でもクラウドでも同じ振る舞い（クラウドは後で載せる）。HTTP などの API は Concrete が持つ。WASM は外した。

持ち越し：ACDR 0191（長いパッチをファイルで渡す、schema-driven 0.1.2）は承認待ちで、作業ツリーにある。AI ごとの許可の設定は、いま brainstorming（Concrete）が一時に持つ（ACDR 0188）。ボードの更新は figures/board_r<回>.py と apply_patch.py（引数の上限を超えないように分けて渡す）。
