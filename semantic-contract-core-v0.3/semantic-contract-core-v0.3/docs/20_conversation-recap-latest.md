# 20. v0.2以後の議論サマリー（逐語録ではない）

## 2026-10-17：SemanticからVocabulary / Interpreterへ

- 「Semanticとは何か」を議論し、データ構造に対して解釈規則を与えるという仮定に到達。
- ユーザーは、**Semanticの宣言と解釈を分け、VocabularyとInterpreterで扱う**方針を選択。
- Abstract Schemaは語彙・解釈機構が成立する抽象規則、Concrete Schemaは用途の具体化。
- 「Termだけが最小単位か」「RelationとConstraintをどこへ置くか」は未決。

## JSON-LD / RDF / SHACLの深掘り

- JSON-LD: IRI、Context、`@id`, `@type`, `@protected`、Expansion/Compaction等から概念を借りる。
- RDF: Node–RelationのGraph表現として参考にする。
- SHACL: NodeShape、PropertyShape、Property Path、`sh:closed`、構造化Validation Reportを参考にする。
- これらが表現可能な範囲と、自分たちが表したい意味の全範囲は一致しない。全てをRDFにしない。

## Rust Engineの選定

- ユーザーはFoundation実装言語の第一候補をRustとした。
- ただしJSONを共通交換形式とし、抽象意味モデル、公開仕様、Interpreter Protocolは言語非依存。
- Rustのtraitは内部実装上のPortに利用可能だが、外部インターフェースの唯一の定義にはしない。
- CLI / IPC JSONなどで他言語と相互運用できるようにする。

## 最後に追加された最重要要求

- **成熟した既存技術が、私たちの意味表現や拡張能力の上限になってはいけない。**
- 技術は独自Foundationの下位Adapterに配置し、必要な機能だけ取り込み、変換不能な意味を保持・可視化する。
- これを図とZIPのv0.3に反映した。

## 次のブレスト候補

1. 意味の最小単位（Node / Term / Relation / Rule）を何に置くか。
2. VocabularyとInterpreterのバージョン／Capability整合ルール。
3. Core内CanonとJSON/Graphの関係。
4. 標準へ射影するときの非対応・情報損失の表現。
