# 12. JSON-LD / RDF / SHACLから借りたい概念

## 前提
JSON-LDはJSONとLinked Dataを接続する規格、RDFは識別可能な資源間の関係をトリプルとして表現するデータモデル、SHACLはRDFグラフをShapeに照らして検証する規格。**SHACLは一般のJSONをそのまま検証する仕様ではない**。

## JSON-LDからの候補

| 仕組み | 何が嬉しいか | 私たちの採用案 |
|---|---|---|
| `@context` | ローカルキーを意味識別子（IRI）に対応付ける | Contextを使ったVocabulary alias解決の参考 |
| `@id` | 対象の識別 | Contract / Term / Instance間の参照識別子の参考 |
| `@type` | 資源がどの概念に属するかを記述 | Semantic typeの区別に利用（静的型保証と混同しない） |
| `@protected` | 重要な用語の再定義からの保護 | 名前空間の競合制御・定義の保護に応用 |
| Expansion / Compaction | 人間向け略記を意味解決済みの形へ正規化 | 短いAI入力とCanonical formを分離 |
| `@graph`, IRI references | 入れ子以外の関係を表現 | Relationの表現設計に応用 |

**限界:** `@context`は名前の対応付けを行うのであって、業務上の意味や不変条件を自動的に証明しない。JSON-LD展開時に通常のJSON構造・順序を完全に保つとは限らない。語彙の対応付けがないフィールドの扱いにも注意が必要。

## RDFからの候補

- NodeとRelationshipを構造上対等に扱う（Subject–Predicate–Object）。
- ネストに依存しない参照と多対多の関係が表せる。
- JSONツリーとは異なる**意味関係の射影モデル**として使える。
- 本体のJSONを捨ててRDFへ全面移行することは**未決／当面不要**。

## SHACLからの候補

| 仕組み | 取り込みたい設計原則 |
|---|---|
| NodeShape / PropertyShape | 値だけでなくノードや関係に制約を置ける |
| `sh:path` / Property Paths | 参照の経路をたどる関係制約 |
| `sh:minCount`, `sh:maxCount`, `sh:class` | 個数・型・関係先を限定 |
| `sh:closed` | 追加情報を許す／閉じるを用途別に選択 |
| Validation Report | focus node、path、source constraint、severity、messageを返す |

**限界:** SHACLが動作するにはRDFグラフとShapeの用意が必要。ソースコードの依存関係を直接検証するわけではなく、ソースからグラフを抽出するAdapterが必要。SHACLの制約体系で表せない意味まで無理に矮小化しない。

## 推奨：取り込み順

1. IRI形式や明示的な識別子、Contextの概念、構造化診断という**意味上の良い設計**を学ぶ。
2. まず元JSONを正本に保持し、必要な契約・参照・関係だけを内部で解決する。
3. RDFへの射影・SHACL検証は**必要なConcrete Schemaに限定して**追加する。
4. 未表現・非互換・情報損失は診断として可視化。黙って無視しない。

## 一次資料

- JSON-LD 1.1 (W3C): https://www.w3.org/TR/json-ld11/
- JSON-LD 1.1 Processing Algorithms (W3C): https://www.w3.org/TR/json-ld11-api/
- RDF 1.1 Concepts (W3C): https://www.w3.org/TR/rdf11-concepts/
- SHACL Recommendation (W3C): https://www.w3.org/TR/shacl/
