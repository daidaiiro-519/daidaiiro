# Semantic Agent Schema の設計

この中のものは、正（いま守るもの）と仮説（上から決めた設計で、作りながら検証するもの）に分かれる。

| 置き場所 | 中身 | 扱い |
|---|---|---|
| `concept/` | 概念のページ（版18）。責務・能力・内と外・事業領域と文脈・セッション・データ・スキーマと DB | 正。ただしアーキテクチャ・ドメインモデルのタブは仮説 |
| `arch/arch-v15.json` | 業務領域・集約・crate 14・ポート9・ディレクトリの木・決まり | 仮説。`arch-v14.json` は1つ前の版 |
| `advisors/impl-ddd.json` | 上から決めて下から確かめた結果（集約・値・サービス・スクリプトのまま） | 仮説 |
| `advisors/re-uc.json` | ユースケース8件（UC0〜UC7・SF1）と意味の検証の流れ | 正 |
| `advisors/general-ddd.json` | 一般の能力の契約（セッションの4つの型・差分・JSON Schema の検査・デザインシステム） | 正 |
| `schema-db/design.json`・`design.md` | schemas/v1 の14ファイル・x- 10個・DB 5表 | 仮説（版21 で Abstract を導き直し、公開された言葉は9つ、Concrete に書ける x- は5つになった。いまの形は概念のページのスキーマと DB のタブ） |
| `schema-db/factcheck-*.json`・`sources.json` | 原典との照合の結果と、原文の取得の記録 | 正 |
| `rust/factcheck-rust.json`・`sources.json` | Rust の一次情報との照合 | 正 |
| `plan/plan.json`・`review.json` | 作り直しの計画と、その反証 | 正 |
| `exploration/` | 探索の記録の形（`exploration.schema.json`）と、ステップごとの記録 | 正 |

## 仮説を直すとき

作ってみて仮説と食い違ったら、`exploration/step-NN.json` の reconciliation に書き、仮説の側（`arch/`・`concept/`）を直す。部品の置き場所は作った結果を、境界と公開する契約は仮説を採る。どちらにも実物が無いものはスクリプトに残す。

## 古くなっている記述

`arch/arch-v15.json` の rules_language_neutral の1行目「文脈の根はリポジトリの根」は古い。正しくは、リポジトリの根は製品の置き場で、文脈は根の直下の文脈の名前のフォルダ（CODING.md）。
