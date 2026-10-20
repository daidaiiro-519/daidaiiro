# 14. 最重要追加原則：成熟標準は「部品」であり、抽象を制限しない

## ユーザーが明示した要求

> 既存の枯れた技術を使用する場合に、私たちが考えている表現を制限されては困る。

今回の構想では、この要求を**最上位の設計制約**として扱う。

## 正本と部品の主従関係

**独自Abstract Semantic Model（表現できる世界の定義）が正本**。JSON Schema / JSON-LD / RDF / SHACL / OpenAPIは、それぞれ得意な領域を解釈・検証・射影・交換する**Adapter／ライブラリ／規格**。

「独自モデルがSHACLで表せないから、意味を削る」という設計はしない。SHACLで扱える部分だけをProjectionし、扱えない意味は原形のまま残し、別Interpreterまたは未対応として明示する。

## 必須にしたい7原則

1. **Expression-first:** 最初に私たちの表現モデルを設計。外部標準の制約が全体の上限にならない。
2. **Preserve source:** 元のContract/Instance JSONを保持し、派生形式への変換で置き換えない。
3. **Loss awareness:** 射影の情報損失・非可逆性を明示する（lossless / partial / unsupported）。
4. **Explicit capability:** Adapterが理解できる語彙・制約・操作を宣言する。
5. **Fail-closed on required semantics:** 必須の意味が検証不能なら成功判定しない。
6. **Open extension:** 任意のVocabulary/Interpreterを登録できる。ただし登録・実行権限は管理する。
7. **Stable public contract:** 外部の公開形式をRust型や特定OSSの内部構造に固定しない。

## Adapter別の責務と限界

| 標準・技術 | 借りる機能 | 抽象を制限しないための方針 |
|---|---|---|
| JSON Schema | JSONの構造・値制約、Meta-Schema、既存Validator | 表現できない意味条件は独自Constraintとして保持 |
| JSON-LD | IRI / Context、名称展開、意味識別、Linked Data表現 | JSON-LDを全Contractの正本にしない。Context未対応部分を落とさない |
| RDF | 関係グラフ、問い合わせ・関係照合 | JSONのあらゆる性質をRDFへ強制変換しない |
| SHACL | グラフ上のShape・Path・関係制約・診断 | 対応可能な関係部分にのみ使用。非対応の制約は別Interpreterへ |
| OpenAPI | HTTP通信契約、生成可能なAPI interface | Coreの概念をHTTP中心にしない。Concrete DevelopmentのAdapterとして利用 |
| Rust | 型・所有権・ビルド時検査 | Engine内部実装には活用。仕様と外部プロトコルをRust専用にしない |

## 具体例 — 表現不能時にどう扱うか

`Constraint = 「Gatewayが3分連続で不達ならSensorの警告状態へ遷移」` は単なるJSONの形状制約ではない。JSON Schemaにはこの時間的意味論を直接検証できる汎用機構がない。

- 独自Contract上は**そのまま表現できる**ことを目指す（形式は未決）。
- JSON Schema Adapterは構造面のみ検証し、時間的制約のCapabilityは`unsupported`と報告。
- 時間・状態のInterpreterが利用可能ならそこで検証する。
- Interpreter自体が存在しないなら「完全適合」と報告しない。

### Projection Report（構想案）

```json
{
  "target": "json-schema-2020-12",
  "coverage": "partial",
  "represented": ["type", "required", "minimum"],
  "notRepresented": ["temporal-transition-rule"],
  "lossless": false,
  "diagnostics": [
    {"code": "PROJECTION_UNSUPPORTED", "path": "/constraints/temporal"}
  ]
}
```

これは外部標準の定義ではなく、**独自Foundationの拡張機構と診断プロトコルの例**。

## 未決のトレードオフ

表現力を無制限に追加できても、検証能力まで自動的に増えるわけではない。表現可能だが解釈不能な領域が増えた場合、ユーザーやAIへどう示すかが重要。`expressible` / `interpretable` / `verifiable` / `executable` の区別をCapabilityとして記述する案を検討する。
