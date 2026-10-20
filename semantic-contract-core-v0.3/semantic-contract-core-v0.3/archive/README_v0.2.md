# Semantic Contract Core — 構想アーカイブ v0.2

更新: 2026-10-17  /  位置づけ: ブレインストーミングの作業記録（技術採用や抽象モデルは未確定）

## 今回の追加点：境界の再整理

**Schema駆動基盤（Abstract Schema）** を上位の抽象として扱い、その上に **用途別Concrete Schema** が載るという概念整理を追加した。Schema駆動開発はConcreteの重要な適用領域の一つ。ただし **開発用Concrete Schema（契約の定義）** と **Schema駆動開発基盤（生成・検証するシステム）** は厳密には異なる。

### まずここから

1. **[一枚絵：Schema駆動の概念整理](images/schema-abstraction-concept.png)** — 会話で作った図を収録
2. **[抽象と具体の境界整理](docs/10_abstract-concrete-schema-boundaries.md)** — 図の意図・用語上の注意・境界の責務
3. **[編集可能な補助図（Mermaid）](diagrams/abstract-concrete-boundary.mmd)** — SchemaとPlatformをより厳密に分けた図
4. **[変更履歴](CHANGELOG.md)** — v0.1からの差分

## 旧v0.1の全内容も収録

| 文書 | 内容 |
|---|---|
| `docs/01_vision.md` | 背景、問題意識、中心仮説 |
| `docs/02_semantic-contract-model.md` | Contract/Schema/Relation/Constraint等の試案 |
| `docs/03_standards-and-schema-first.md` | JSON Schema、OpenAPI、TypeSpecの位置付け |
| `docs/04_architecture-and-verification.md` | 機械的検証、AI修正ループ、保証範囲 |
| `docs/05_backend-options.md` | Rust / Go / TypeScript / Python |
| `docs/06_frontend-and-ui.md` | Angular / Svelte / Vue / React、Schema-driven UI |
| `docs/07_poc-roadmap.md` | PoCと評価指標 |
| `docs/08_decisions-and-open-questions.md` | 決定に近い方向性、仮説、未決論点 |
| `docs/09_conversation-timeline.md` | v0.1までの議論の流れ |
| `examples/` | 独自Contract・JSON Schema・UIの例（設計試案） |
| `diagrams/architecture.mmd` / `diagrams/contract-lifecycle.mmd` | 旧アーキテクチャ図（思考記録） |
| `archive/README_v0.1.md` | v0.1のREADME原文 |

## 中心思想

> **JSONをAIと各技術の低依存な共通表現とし、成熟した標準（JSON Schema / OpenAPI）を活かしながら、技術非依存のSchema駆動基盤に意味・構造・制約・関係を定義する。用途別Concrete Schemaを通じてUI、開発、Workflowなどへ具体化し、その上で機械的に保証できる範囲を最大化する。**

## 現段階で確定していないこと

- 「Semantic」の定義そのもの。
- Abstract Schema / Concrete Schemaの形式的な意味、準拠と具体化の仕組み。
- Coreの最小プリミティブや、JSON Schemaとの接続方式。
- 最適なBackend/Frontend技術スタック。

このZIPは**発言の逐語録ではなく、会話を構想資料に整理したアーカイブ**。v0.1の各文書は思考の履歴として残し、今回の境界整理は`docs/10`で上書きではなく補足・精緻化した。
