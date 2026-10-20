# 02. Semantic Contract Core の概念候補

## 暫定の核
現時点の提案は `Contract / Schema / Relation / Constraint / Interpretation`。最上位にDDDのEntityやUseCaseを埋め込まず、それらは後から追加するVocabularyとして検討する。

- **Contract**: 一意に識別・版管理できる意味のある契約単位。
- **Schema**: JSONインスタンスの構造と値の許容範囲（標準JSON Schemaを参照・内包）。
- **Relation**: 契約間の参照、利用、生成、依存など。参照解決と整合性が必要。
- **Constraint**: 不変条件・振る舞い上の条件。表現しただけで検証済みとみなさない。
- **Interpretation / Projection**: 契約の具体化規則。例: OpenAPI、Rust DTO、TS型、フォーム、検証計画。

## Coreを小さくする
全てを `kind: entity` のような固定分類にしない。拡張Vocabulary側で `Entity / Operation / State / UI / Workflow / Architecture` などを定義する案。反面、汎用化しすぎると独自解釈器が巨大化するリスクがある。Vocabularyごとに意味、検証方法、表現できないものを宣言する。

## SchemaとSemanticの違い
JSON Schemaの `type`, `required`, `enum`, `minimum` などはデータの適合条件を表現する。一方、「この登録は重複してはならない」「停止中は送信しない」「DomainはInfrastructureに依存できない」といった主張には、DB制約、状態機械、依存ルールなど別の意味解釈と検証が要る。

## データ自由度の捉え方
- **構造の自由**: JSONで新たな構成を自由に記述できる。
- **許容空間の制限**: Schemaが未知の形や値を検出する。
- **意味の拡張**: Vocabularyによって新しい概念を導入できる。
この三者は競合する場合がある。自由度を増やすだけでは保証力は向上しない。

## 未確定の本質的問題
「Semanticとは何か」。語彙の名前や自然言語の説明なのか、検証可能な意味論・制約・状態遷移なのか。機械的に検査可能な意味と、人の解釈を要する意味をどう分離するかが最優先。
