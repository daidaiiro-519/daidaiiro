# Semantic Agent Schema の設計

この中のものは、正（いま守るもの）と仮説（上から決めた設計で、作りながら検証するもの）に分かれる。

| 置き場所 | 中身 | 扱い |
|---|---|---|
| `arch/arch-v15.json`・`arch-v14.json` | 業務領域・集約・crate 14・ポート9・ディレクトリの木・決まり | 過去の仮説として残し、書き換えない。いまの決まりは `../../CODING.md` |
| `advisors/impl-ddd.json` | 上から決めて下から確かめた結果（集約・値・サービス・スクリプトのまま） | 仮説 |
| `advisors/re-uc.json` | ユースケース8件（UC0〜UC7・SF1）と意味の検証の流れ | 正 |
| `advisors/general-ddd.json` | 一般の能力の契約（セッションの4つの型・差分・JSON Schema の検査・デザインシステム） | 正 |
| `schema-db/design.json`・`design.md` | schemas/v1 の14ファイル・x- 10個・DB 5表 | 過去の仮説として残し、書き換えない。いまの形は具体モデリング概念（`../concrete-modeling/`）の公開された言葉と記録 |
| `schema-db/factcheck-*.json`・`sources.json` | 原典との照合の結果と、原文の取得の記録 | 正 |
| `rust/factcheck-rust.json`・`sources.json` | Rust の一次情報との照合 | 正 |
| `plan/plan.json`・`review.json` | 作り直しの計画と、その反証 | 正 |
| `exploration/` | 探索の記録の形（`exploration.schema.json`）と、ステップごとの記録 | 正 |

## 仮説を直すとき

作ってみて仮説と食い違ったら、`exploration/step-NN.json` の reconciliation に書き、仮説の側（抽象モデリング概念 `../abstract-modeling/` と具体モデリング概念 `../concrete-modeling/`）を直す。部品の置き場所は作った結果を、境界と公開する契約は仮説を採る。どちらにも実物が無いものはスクリプトに残す。
