# Semantic Contract Core — 構想アーカイブ v0.1

作成: 2026-10-11 / 位置づけ: ブレインストーミング記録（設計・採用決定ではない）

## 最初に読む順番
1. `docs/01_vision.md`: 目的、問題意識、中心仮説
2. `docs/02_semantic-contract-model.md`: Coreと抽象モデルの候補
3. `docs/03_standards-and-schema-first.md`: JSON Schema / OpenAPI / TypeSpecとの境界
4. `docs/04_architecture-and-verification.md`: 全体構成、検証・AI修正ループ
5. `docs/05_backend-options.md`: Rust / Go / TypeScript / Python
6. `docs/06_frontend-and-ui.md`: Angular / Svelte / Vue / React、Schema-driven UI
7. `docs/07_poc-roadmap.md`: PoC、評価軸、リスク
8. `docs/08_decisions-and-open-questions.md`: 決定に近い合意、未確定、次のテーマ
9. `docs/09_conversation-timeline.md`: 議論の流れ

`examples/` は概念説明用の試案です。標準JSON Schemaと独自メタモデルを意識的に分離しています。`diagrams/` にMermaid図を収録。

## ひと言で
**JSONをAIとソフトウェア間の低依存な共通表現に用い、既存JSON Schemaの構造検証を再利用しつつ、独自Semantic Contract Coreで意味・制約・参照・解釈を記述し、言語・UIの各Adapterを通じて検証可能な具体実装へ投影する。**

## 読み方についての注意
- 「構想として希望・方向性が示されたこと」と「実装方針として確定したこと」は異なる。
- Rust/Svelteや独自Coreの優位性は仮説であり、PoCで比較検証する。
- ここにあるのは**会話内容の主題別再構成**であって、発言の逐語録ではない。
- 「Semanticの定義」は意図的に未確定。次回の最優先議題。
