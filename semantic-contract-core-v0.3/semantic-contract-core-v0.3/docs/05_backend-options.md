# 05. バックエンド技術候補

言語は最初から確定せず、同一契約を各言語でどこまで保証できるかを比較する。対象は **Rust / Go / TypeScript / Python**。

| 言語 | 主要候補 | 静的な強み | 実行時契約 | 懸念 |
|---|---|---|---|---|
| Rust | Axum / Serde / SQLx | enum網羅性、所有権、公開範囲、型状態 | JSON Schema validator等 | 学習/ビルドコスト、生成器互換性 |
| Go | net/http / Chi | コンパイル、簡潔な型とinterface | validator等 | 一部の不変条件を型で表しづらい |
| TypeScript | Fastify / NestJS | 判別可能ユニオン、型推論、ビルドチェック | Ajv / TypeBox等 | 型は実行時に消える |
| Python | FastAPI / Pydantic | 型ヒント＋mypy/pyright | Pydantic / JSON Schema | 静的保証の範囲が限定的 |

Rustを選ぶ根拠は単純な速度ではなくコンパイル時に見つかるエラーと型状態設計。Goは一般的なAPIで開発効率が有利になり得る。TSはJSONとフロント型共有、Pythonはデータ・AIエコシステムと実行時検証の利点。

## Rust + OpenAPI
- Contract-first: OpenAPI/JSON Schema → Rust DTO/インターフェース → Axum Handler。
- Code-first: Rust + utoipa → OpenAPI。
- 注意: OpenAPI自体がRustコンパイラに内蔵されるわけではない。生成型と実装の整合性、境界の実行時検証が必要。

## 評価する数値
初回生成成功率、生成から検証成功までの修正回数・所要時間、構造/型/境界違反の検出率、残存の業務欠陥数、変更波及のカバレッジ、ビルドコスト、実行性能・メモリ。
