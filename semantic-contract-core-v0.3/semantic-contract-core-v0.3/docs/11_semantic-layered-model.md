# 11. Semanticの宣言と解釈を分離する — レイヤーと境界

## 位置づけ
- **合意済みの方向:** Semanticの宣言と解釈を分離し、VocabularyとInterpreterで両方を扱う。
- **合意済みの方向:** Schema駆動基盤が抽象側にあり、用途別Concrete Schemaを通じて開発・UI・Workflow等へ具体化する。
- **未決:** 「Semantic」の完全な形式意味論、最小プリミティブ、Graph/JSONのCanonical Representation。

## 最重要の区別 — 契約とそれを動かすソフトウェア

- **Abstract Schema** は、契約・語彙の宣言、識別、参照、適合、拡張に関する抽象の**規則／形式**。
- **Concrete Schema** は、特定ドメインの語彙を使って表現・制約を**宣言した成果物**。
- **Semantic Contract Engine** は、それらを解釈・検証する**実装**。まずRustで実装したい。
- **Schema駆動開発基盤** は開発用Concrete Schemaを利用する**アプリケーション／プラットフォーム**。スキーマそのものではない。
- **Schema駆動Runtime** はUI、フォーム、Workflow等のConcrete Schemaを処理する実行側。開発基盤とは別の境界。

## Vocabulary・Concrete Schema・Interpreterの関係

1. Vocabularyは、識別子・概念・関係・単位・制約などの**意味宣言**を行う。
2. Concrete Schemaは、特定領域に対してVocabularyを参照し、追加の構造・許容範囲・制約を宣言する。
3. InterpreterはVocabulary・Concrete Schemaを解決し、対応するCapabilityに従って解釈／検証する。
4. Interpreter未対応の意味を`verified`とみなさず、`unsupported`／`unverified`として報告する。
5. AIはJSONを生成・変更できるが、JSONが妥当なことと業務上の意味が正しいことは別。

## Semanticの4つの側面（分析のための仮分類）

| 側面 | 例 | 検証上の限界 |
|---|---|---|
| Descriptive | 名前・説明・意図 | 人手/外部根拠なしには真偽確定困難 |
| Structural | 型・必須・パターン | JSON Schema等で一部機械検証可能 |
| Formal | 単位関係・関係制約・状態条件 | 形式化された部分について検査可能 |
| Operational | 状態遷移・変換・実行 | Interpreterとテスト等によって保証範囲が変化 |

上記は仕様の規定階層ではなく、議論を進めるための視点。Semanticを単なるラベルや説明文に還元しない。

## Coreを過度に具体化しない

- DDDのEntity / UseCase、API Path、Angularコンポーネント、Rust crateをAbstract Schemaの必須プリミティブにしない。
- これらはConcrete Vocabulary / Schema / Adapterの語彙として扱えるようにする。
- 最小単位の候補はTerm, Identity, Definition, Relation, Constraint, Interpretationなど。ただし**未決**。
- Coreが全Vocabularyの意味を静的に知るのではなく、語彙ごとのInterpreterへ委譲できる仕組みを持つ。

## 境界を誤らないための反例

- 「JSON Schemaで定義できないから、その関係は表現禁止」→ **不採用**。独自契約を保持し、別Interpreterで解釈する。
- 「Rustのtraitを外部プラグインの唯一の契約にする」→ **不採用**。Rust内部ではよいが外部契約は言語非依存にする。
- 「開発用Concrete Schema = 開発プラットフォーム」→ **誤り**。定義と利用者を分離。
- 「全JSONを必ずRDFへ可逆変換する」→ **未保証**。元JSONを保持し、グラフは必要時の派生表現とする仮説。
