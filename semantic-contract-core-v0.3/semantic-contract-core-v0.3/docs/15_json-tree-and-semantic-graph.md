# 15. Canonical Representation：JSONツリーとSemantic Graphの共存

## 現時点の仮説

**外部交換はJSON、元JSONを保持、意味解決後のGraphは必要に応じた派生表現。** ただし、正式な内部Canonical Representationは未決。

### 方式A：JSONツリー中心
- 長所: AIとの相性、シリアライズ容易、既存JSON Schemaの適用が明快。
- 短所: 非局所的な多対多参照・複雑な関係の検証に別処理が必要。

### 方式B：RDF/Graph中心
- 長所: 識別子と関係中心、SHACLやSPARQLの恩恵。
- 短所: JSON固有のツリー構造・配列・順序等との完全往復が自明ではなく、全用途のGraph化は過剰になり得る。

### 方式C：ハイブリッド（現時点の推薦仮説）
- Authoring/Exchange: JSON
- Structural Validation: JSON Schema
- Meaning normalization: IRI/Contextなどで名前解決
- Relation projection: Graph（必要な部分だけ）
- Relation verification: SHACLなど（必要時）
- Runtime application: 個別Interpreter
- Original contract: 元JSONを失わず保持する

## 完全な可逆性を無条件には要求しない

`JSON -> Semantic Graph -> JSON` の往復は一般には情報を失う可能性がある。だから変換結果の`coverage`と`lossless`を明示し、**原本のJSONがCanonical Sourceとして残る**構造を検討する。

## 判断に使う実験

同じConcrete Schemaについて、ネスト・循環参照・異名同義語・配列順序・複合関係・複合値の例を作り、JSONモデルとGraphモデルで情報損失、参照解決、検証表現の必要性を比較する。
