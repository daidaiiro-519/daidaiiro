# 17. 決定・仮説・未解決事項（最新版）

> この文書がv0.3時点の設計状態の正本。旧文書との解釈の衝突があればここを優先。

## ユーザーが明確に示した要求・方向

1. AIに実装させる際、プロンプトとハーネスだけに制約を依存させず、コンパイル・型・機械的検証を活かしたい。
2. JSONはAIと各層の低依存な共通交換表現として重視。
3. Schemaによって自由な表現を許容範囲に閉じ込めたい。
4. Abstract Schema＝Schema駆動の抽象基盤、Concrete Schema＝用途別の具体化という整理。
5. Semanticの宣言と解釈を分離し、VocabularyとInterpreterで両者を扱う方針。
6. JSON Schema / OpenAPI等の成熟技術を利用したい。
7. TypeSpecの作者が選んだ具体抽象を正本にしたくない。
8. **既存の枯れた技術によって独自の意味表現能力を制限されては困る。**
9. Foundation Engineは当面Rustが有力。ただし言語非依存の意味仕様・公開Interfaceを望む。
10. 開発基盤のBackend候補にはRust / Go / TypeScript / Pythonを含める。

## 採用方針の提案（合意済みの正式仕様ではない）

- 独立したSemantic Contractモデルを正本にし、成熟標準をAdapter／Validatorとして使う。
- JSON原本を維持し、JSON-LD / RDF / SHACLは必要な機能だけを接続。
- InterpreterにはCapabilityを宣言させ、未対応を成功として隠さない。
- `Expression-first`、`Loss-aware Projection`、`Fail-closed for required semantics` を原則にする。
- Rust Engine内部のtraitやcrateは公開契約の一部としない。
- CLI / IPC + JSONを最初の外部Interpreter Protocolとして試す。

## 依然として未決の重要事項

1. **Semanticの形式的な定義**と独立な意味規則の検証範囲。
2. 最小プリミティブ（Term / Identity / Definition / Relation / Constraint等）。
3. Vocabularyの登録・互換性・バージョン・衝突解決。
4. Abstract/Concreteの正確な関係（準拠／特殊化／合成／型付きインスタンス化）。
5. Canonical Representation（JSONツリー／Graph／Hybrid）の正式決定。
6. 構造検証、意味解釈、実装検査の責務と結果ステータスの仕様。
7. 必須Vocabulary未対応時の失敗、権限、セキュリティ。
8. どこまで標準JSON SchemaのMeta-Schema / Vocabulary機能をそのまま使うか。
9. Backend/Frontendの最適構成、PoC選択、具体的な生成器。
10. Concrete Schemaでの拡張と破壊的変更の互換性モデル。

## 次の議論への問い

「意味を持つ対象と関係をどう定義するか」「意味の契約が保証できたと宣言するために何が必要か」「どの情報をCanonicalに固定し、どの情報をAdapterへのLoss-aware Projectionにするか」。
