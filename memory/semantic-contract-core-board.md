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

第13回まで出した（2026-10-17）。いま見る論点は Q0 目的と設計原則（回答待ち）。Q0 の目的は「意味 ・ 構造 ・ 関係 ・ 制約を1つの契約（Schema）に宣言し、その契約でインスタンスを解釈して検証する。機械で決まる検証も意味の判断が要る検証も同じ契約から行い、同じ形の結果で返す」。標準は JSON Schema だけ、x- の拡張は基盤の契約で基盤のコマンドが確かめ、標準へは写さない（第9・10回）。Agents 契約で呼ぶ AI は Claude Code ・ Codex ・ Kiro ・ GitHub Copilot（Copilot CLI 1.0.98 を WSL に npm で入れ、help を measured/copilot-cli/ に保存）。

持ち越し：AI ごとの許可の設定は、いま brainstorming（Concrete）が一時に持つ（ACDR 0188、.brainstorming/.settings/agents.json と brainstorming/agents@v1）。Q3 で Agents 契約が決まったら、形を基盤（Abstract）へ移し、brainstorming には値だけを残す。組み立ては figures/board_r<回>.py。
