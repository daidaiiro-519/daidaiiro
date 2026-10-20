# 10. Schema駆動基盤（Abstract）とSchema駆動開発（Concrete）の境界

> **議論日**: 2026-10-17  /  **状態**: 概念整理の暫定合意。実装設計の決定ではない。
>
> **参照図**: [`../images/schema-abstraction-concept.png`](../images/schema-abstraction-concept.png)（会話で作成した一枚絵をそのまま同梱）
>
> **補助図（編集可能）**: [`../diagrams/abstract-concrete-boundary.mmd`](../diagrams/abstract-concrete-boundary.mmd)

## 1. 今回、新しく明確になった境界

これまで「Semantic Contract Core」という一つの呼称に、抽象的なSchemaの定義・解釈と、コード生成・ビルド・AI修正ループを含む開発基盤の責務を重ねていた。以下を分ける。

1. **Schema駆動基盤（Abstract Schema / Semantic Contract Core）**: 特定用途や言語に先立つ意味・契約・構造・関係・制約・インスタンスの扱い方を規定する、抽象側の基盤。
2. **Concrete Schema（用途別スキーマ）**: 抽象基盤で定義された形式・ルールに基づき、具体的な用途の語彙、契約、表現空間を定義したもの。
3. **Schema駆動開発基盤（Development Platform）**: 開発に特化したConcrete Schemaを利用して、コード生成・検証・ビルド・AI修正ループ等を行う**実行・運用システム**。

**用語の厳密な区別**: 「Schema駆動開発 ＝ Concrete Schemaそのもの」と言い切ると、**スキーマ（定義）**と**そのスキーマを利用する開発基盤（ソフトウェア）**が混ざる。会話中の図は大枠の方向を説明する模式図であり、厳密には次の関係で捉える方がよい。

```
Schema駆動基盤 / Abstract Schema（抽象ルール）
        ↓ 準拠・具体化
開発用 Concrete Schema（開発領域の語彙・制約）
        ↓ 利用・解釈
Schema駆動開発基盤（コード生成・検証・AI支援）
        ↓
Rust / Go / TypeScript / Python / Angular / Svelte / ...
```

用途別Concrete Schemaは開発領域だけに限られない。UI、Workflow、設定、データ連携などにも個別のConcrete Schemaがあり得る。UI RendererやWorkflow Engineは、それぞれのSchemaを利用する**Runtime**であり、Schemaそのものとは区別する。

## 2. なぜこれが必要か

- **抽象を保護する**: Foundationをソフトウェア開発用語（Project、Source Code、Build、UseCase等）で規定してしまうと、用途に閉じた「具体」が上位の「抽象」を拘束する。
- **低依存な共通表現を維持する**: JSONを人間・AI・各実装技術が交換する表現に採用し、JSON Schemaなど成熟した標準の検証機能を再利用できるようにする。
- **同一原則の別用途展開**: Schema駆動開発は最初の重要なユースケースであって、Foundationの存在理由すべてではない。
- **検証可能性を分離する**: Foundationが定義する適合性と、開発基盤が追加で行うコンパイル・アーキテクチャ検査・テストは別の保証である。

## 3. 役割表

| 境界 | 主に定めること | 主に定めないこと | 主な成果物・手段 |
|---|---|---|---|
| Abstract Schema / Foundation | 契約の識別、構造、意味の参照、関係、制約の定義法、Instanceとの適合規則 | Rustのクレート構成、HTTPのPath、特定UIのコンポーネント | 抽象メタモデル、標準JSON Schemaとの接続、共通検証機構 |
| Concrete Schema: Development | 開発領域の語彙、Artifact、Dependency、Verification、許容される変更など | 基盤全体の普遍的な意味 | 開発用Schema、依存契約、検証ポリシー |
| Schema-driven Development Platform | Concrete Schemaを消費してコード・テスト・診断結果を作成、AI修正を回す | 抽象メタモデルの普遍的な定義 | Generators、Adapters、CLI、Harness |
| Concrete Schema: UI / Workflow など | データバインディング、許される操作、配置、状態遷移など | 言語固有の実装詳細 | UI Schema、Workflow Schema |
| Schema-driven Runtime | 定義済みSchemaとInstanceの解釈・描画・実行 | 抽象の普遍的定義 | UI Renderer、Form Engine、Workflow Engine |

## 4. 用語上の注意点（今後の議論に残す）

- **Abstract Schema / Concrete Schemaは、今回の構想内での暫定名称**。JSON Schema標準における正式な階層名だと誤解しない。
- **SchemaとPlatformは異なる**。前者は定義・契約、後者はそれを操作・解釈するシステム。
- **抽象→具体は常に一方向の完全自動生成とは限らない**。「準拠」「特殊化」「組成」「解釈」「射影」のどの関係なのかを明確にする必要がある。
- **Semanticの意味は未定義**。意味をどこまで形式化するか、どの保証をFoundationに置くかは次のブレストで確定する。
- **JSON Schemaのメタスキーマとの混同に注意**。抽象モデルに対するメタ定義（メタスキーマ）の構造、契約インスタンス、ユーザーデータインスタンスの関係は別途定義が必要。

## 5. 現時点の整理（合意・仮説）

### 方向性として共有されたこと
- Schema駆動が抽象基盤であり、その上に具体領域としてSchema駆動開発が載る。
- JSONをAIと実装技術の低依存な共通表現とし、JSON SchemaやOpenAPIなどの成熟した標準は活用する。
- 技術固有の保証（Rustコンパイル、TSの型検査、Pythonの実行時検証など）は、用途側で適切に活用する。

### まだ設計上の仮説であること
- Coreを `Contract / Schema / Relation / Constraint / Interpretation / Instance` 等で組み立てること。
- 各用途をVocabularyとConcrete Schemaとして表現し、どこまで機械的に適合性検証できるか。
- RuntimeとDevelopmentを完全に独立したBounded Contextとするか、共通のProjection/Adapterを共有するか。

## 6. 次のブレストへの問い

1. **Semanticとは何か？** 名称・参照・制約・状態・振る舞いをどこまで含むか。
2. Abstract SchemaとConcrete Schemaの間の関係は、継承・合成・準拠・型付きインスタンス化のどれが本質か。
3. どの内容がFoundation固有で、どの内容が開発・UIなどの領域固有か。
4. Schemaは何を「表現可能」にし、どこまでを「検証可能」にするか。
5. 既存のJSON Schemaメタスキーマ、Vocabulary拡張、OpenAPI参照をどの境界で利用するか。

## 7. 旧文書の読み替え

`02_semantic-contract-model.md` と `04_architecture-and-verification.md` はv0.1時点のブレスト記録として原文を維持した。そこではCore、コード生成、Renderer、Harnessが近接して図示されているが、**本書（v0.2）ではそれらの所属境界を明確に分離した**。旧文書を誤りとして削除するのではなく、思考過程として保存する。
