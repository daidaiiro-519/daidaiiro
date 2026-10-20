# Semantic Contract Core — 構想アーカイブ v0.3

**更新:** 2026-10-17　**状態:** ブレインストーミング記録／仕様未確定　**実装方針:** Rustを第一候補、公開契約は言語非依存

> **中核原則:** 私たちの表現モデルが主であり、既存の成熟技術は交換可能な部品。JSON Schema / JSON-LD / SHACL / OpenAPIが表せない概念を、既存標準に合わせて削除・歪曲・黙って縮退させない。

## 入口（今回の更新）

- **[最新版の一枚絵：正確なテキスト版 PNG](images/semantic-contract-core-overview-v0.3.png)** ／ [拡大可能なSVG](images/semantic-contract-core-overview-v0.3.svg)
- **[ビジュアル版の全体構成図](images/semantic-contract-core-overview-illustrated.png)** — 参考用画像。語句と境界の正確さは上のSVG/PNGと本文を優先。
- [概念・境界の整理](docs/11_semantic-layered-model.md)
- [成熟標準を「部品」として使う原則（表現能力の非従属性）](docs/14_standards-as-optional-adapters.md)
- [JSON-LD / RDF / SHACLの取り込み候補](docs/12_jsonld-rdf-shacl-concept-review.md)
- [Rust実装・言語非依存プロトコル](docs/13_rust-engine-language-neutral-protocol.md)
- [決定事項／仮説／未解決論点](docs/17_decision-ledger-v0.3.md)

## このZIPの読み方

1. `images/semantic-contract-core-overview-v0.3.png` で全体像を確認。
2. `docs/11` で Abstract / Concrete / Foundation / Platform と Vocabulary / Interpreter の区別を読む。
3. `docs/14` で**既存標準に表現を従属させない**原則を読む。
4. `docs/12` でJSON-LD・SHACLから取り入れる概念を確認。
5. `docs/13` と `docs/15` でRust・JSON・Graph・交換プロトコルを検討。
6. `docs/17` で確定事項と未決事項を確認して議論を再開する。

## これまでの議論を保持

- v0.2の全21ファイルを引き継ぎ、旧 `docs/01`〜`docs/10`、旧PNG・Mermaid・例を保存。
- v0.1由来の検討: AI駆動開発、型安全性、Rust / Go / TS / Python、Angular / Vue / React / Svelte、Schema-first / OpenAPI、PoC。
- v0.2の境界整理: Abstract Schema、用途別Concrete Schema、Schema駆動開発基盤／Runtimeの分離。
- v0.3の追加: Semanticを「宣言（Vocabulary）」と「解釈（Interpreter）」へ分離。JSON-LD・RDF・SHACLの分析。Rustエンジンと言語非依存の意味仕様を分離。**既存標準に表現能力を制限されない**原則を明文化。

## 用語の最短整理

| 用語 | 役割 |
|---|---|
| Abstract Schema | 意味契約を定義・参照・合成・解釈するための抽象的な規則・形式。今回の構想独自の用語。|
| Vocabulary | 識別可能な用語と意味、適用規則、要求する解釈能力を宣言する。|
| Concrete Schema | Vocabularyを使い、特定領域の表現可能範囲と契約を宣言する。|
| Interpreter | 宣言済みの語彙・契約を解決、検証、変換、実行等する処理能力。|
| Semantic Contract Core | 上の抽象モデルを成立させる基盤・その仕様。Rust Engineはその一実装。|
| Schema駆動開発基盤 | 開発領域Concrete Schemaを使い、AI実装、生成、ビルド、検証、修正を支援する利用側システム。|

**注意:** これは逐語録ではなく議論内容を構造化した記録。コード・JSONは主として設計試案で、実装済みの仕様だとは主張しない。異なる時点の提案が衝突した場合は新しい `docs/17` を優先して読む。
