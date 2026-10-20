# 03. 標準とSchema-first

## JSON
共通の交換形式・AIの構造化出力形式として有力。JSONという表現形式そのものは意味的な正しさを保証しない。

## JSON Schema
`2020-12` を候補に、Schema自身の検証とインスタンス検証を行う。`additionalProperties: false`、`required`、`enum` などを利用。適切なSchemaと実際の検証の双方が必要で、「想定外の値は絶対入らない」とは言えない。生成物の型への投影で全制約が保存されるとは限らない。

## OpenAPI
HTTP APIのpath / method / request / response / status / security等を契約化する標準。Rustとも併用可能。OpenAPIからRust DTO/サーバー骨格やTypeScriptクライアントを生成する方向（contract-first）と、RustからOpenAPIを生成する方向（code-first）がある。本構想では前者がより整合的。

代表候補: utoipa（Rust→OpenAPI）、OpenAPI Generator（OpenAPI→各言語）、Typify（JSON Schema→Rust型）、openapi-typescript / openapi-fetch（フロント向け）。生成器のカバレッジと互換性は実際に検証する。

OpenAPIはHTTPの具体である。Domain UseCaseがHTTPと同一とは限らない。MQTTやCLIなど別アダプターから同一ユースケースを呼び得る。

## TypeSpecの位置づけ
Microsoftの宣言的API仕様言語。OpenAPI/JSON SchemaへのEmitterの発想は参考になるが、上位の意味モデルの正本には採用しない方針が示された。抽象が既にAPI設計の文脈で具体化されているため。技術的にTypeSpecへ投影するAdapterは不可能ではない。

## 独自化する範囲
**再利用:** JSON Schema規格とValidator、OpenAPI、コンパイラ、各言語の型システム、成熟した生成器。
**独自設計:** Semantic Contractのメタモデル、Vocabulary、参照解決、契約間制約、投影・検証能力の宣言、構造化診断、必要なAdapter。

## 正本の選択肢
1. JSON Schemaを正本にして意味を付加する。
2. 小さな独自Semantic Contractを正本にしてJSON Schemaを参照する。
議論は2へ傾いているが、抽象と標準の責務境界は未確定。
