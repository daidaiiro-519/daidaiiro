# sources：ボード semantic-contract-core の原典

ボード semantic-contract-core（構想 Semantic Contract Core v0.3 を、基盤 schema-driven とどう関係させるかを決める）の根拠にする原典の置き場所である。原文は fact-check の `fetch` で取得し、`verify` で照合した。git で追跡するのは `*.meta.json` とこの README だけで、原文のファイルは追跡しない（`.gitignore` 58〜60行）。

確かめた反証の仮説は次のとおりである。

> 自前のモデルを正本にしなくても、JSON Schema 2020-12 の Vocabulary の仕組み（`$vocabulary`・メタスキーマ）の中で、表現を制限されずに独自の意味を足せるのではないか。未知の必須語彙を処理できない実装は失敗にする（fail-closed）仕組みが、もともと規格にあるのではないか。

---

## 1. 取得したもの

取得した日はすべて 2026-10-17（JST）。HTML はタグを剥がさずに保存してある。

| ファイル | URL | 行数 | sha256 |
|---|---|---|---|
| `www.ietf.org_archive_id_draft-bhutton-json-schema-01.txt` | https://www.ietf.org/archive/id/draft-bhutton-json-schema-01.txt | 4,369 | `03834e2a60caff313476df9913dac2ccf9a34aad569df570f8ab80db7bd6d24a` |
| `www.ietf.org_archive_id_draft-bhutton-json-schema-validation-01.txt` | https://www.ietf.org/archive/id/draft-bhutton-json-schema-validation-01.txt | 1,681 | `1f8832490b4b8a5876b9f571afce4051a5ccbae2811220d70f8e86c3c08da414` |
| `json-schema.org_draft_2020-12_json-schema-core` | https://json-schema.org/draft/2020-12/json-schema-core | 5,822 | `6edfa81f5b11070c1e391775d9e1368a2cc759fd3a93d0284341d6f7bda2ce76` |
| `json-schema.org_draft_2020-12_json-schema-validation` | https://json-schema.org/draft/2020-12/json-schema-validation | 3,185 | `c637d0a9640d54ace07395a677a6c3f1ad760f8df2eed57d5ca967f007e7e538` |
| `www.w3.org_TR_json-ld11` | https://www.w3.org/TR/json-ld11/ | 13,368 | `9e2c9972d0f60bc744e975731643a9a63d410afc6b682eb8898ad2720e452866` |
| `www.w3.org_TR_shacl` | https://www.w3.org/TR/shacl/ | 6,385 | `ccca77caed771f42ea649c03d190b9c7ea0b82286c22c457b9edc8087c9ad0f8` |

- json-schema.org の2つの HTML は、IETF の2つの `.txt` と同じ版である。HTML 側に `<meta content="draft-bhutton-json-schema-01" name="ietf.draft">`（`json-schema.org_draft_2020-12_json-schema-core:29`）と `<meta content="draft-bhutton-json-schema-validation-01" name="ietf.draft">`（`json-schema.org_draft_2020-12_json-schema-validation:24`）がある。**引用と行番号は、改行が文の区切りに近い `.txt` 側で取った。**
- JSON Schema 2020-12 は Internet-Draft である。`.txt` の6行目は `Intended status: Informational`、8行目は `Expires: 12 December 2022`。
- JSON-LD 1.1 は `16 July 2020`（`www.w3.org_TR_json-ld11:1261`）、SHACL は 2017-07-20 の W3C Recommendation（`www.w3.org_TR_shacl:615` の `datetime="2017-07-20"`）の版である。
- `.md` は試していない。4件とも規格の頁で、`fetch` が本体（HTML またはテキスト）を返した。

---

## 2. 照合した主張

照合の種類は、引用はすべて `quote`、識別子はすべて `identifier` である。HTML の原文は、タグを含む文字列のまま引用している（タグを除いた文字列は原文に無い）。

### a. `$vocabulary` の値 true／false の意味

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| true のとき、その語彙を認識しない実装は、そのメタスキーマを `$schema` で宣言するスキーマの処理を拒否しなければならない | If the value is true, then implementations that do not recognize the vocabulary MUST refuse to process any schemas that declare this meta-schema with "$schema". | draft-bhutton-json-schema-01.txt:1421 |
| false のとき、認識しない実装は処理を続けるべきである | If the value is false, implementations that do not recognize the vocabulary SHOULD proceed with processing such schemas. | 同:1424 |
| 語彙を理解している実装には、値は影響しない | The value has no impact if the implementation understands the vocabulary. | 同:1426 |
| 理解している語彙は、その語彙の意味の定義どおりに処理しなければならない | Any vocabulary that is understood by the implementation MUST be processed in a manner consistent with the semantic definitions contained within the vocabulary. | 同:1391 |
| `$vocabulary` は、メタスキーマとして処理されない文書では無視しなければならない | The "$vocabulary" keyword MUST be ignored in schema documents that are not being processed as a meta-schema. | 同:1439 |
| `$vocabulary` の宣言は `$ref` で参照したメタスキーマから継承されない | Note that the processing restrictions on "$vocabulary" mean that meta-schemas that reference other meta-schemas using "$ref" or similar keywords do not automatically inherit the vocabulary declarations of those other meta-schemas. | 同:1474 |
| 局所でだけ使うメタスキーマは `$vocabulary` を省いてよい | Meta-schemas intended for local use, with no need to test for vocabulary support in arbitrary implementations, can safely omit "$vocabulary" entirely. | 同:3879 |
| 規格の中の前例：Format-Assertion 語彙を true で宣言したとき、すべての形式を検証できない実装は処理を拒否しなければならない | When the Format-Assertion vocabulary is declared with a value of true, implementations MUST provide full validation support for all of the formats defined by this specificaion. | draft-bhutton-json-schema-validation-01.txt:651 |
| （同上） | Implementations that cannot provide full validation support MUST refuse to process the schema. | 同:653 |
| （同上）Format-Assertion を指定したときは、未知の形式で失敗しなければならない | When the Format-Assertion vocabulary is specified, implementations MUST fail upon encountering unknown formats. | 同:714 |

`specificaion` は原文の綴りのままである。

### b. 未知のキーワードの扱い

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| 認識しない個々のキーワードは、値を annotation として集める | Unrecognized individual keywords simply have their values collected as annotations, while the behavior with respect to an unrecognized vocabulary can be controlled when declaring which vocabularies are in use. | draft-bhutton-json-schema-01.txt:261 |
| 対応していないキーワードは、値をそのまま annotation の値として扱うべきである | Implementations SHOULD treat keywords they do not support as annotations, where the value of the keyword is the value of the annotation. | 同:720 |
| 認識しない語彙のキーワードも同じく annotation として扱う | Per 6.5, unrecognized keywords SHOULD be treated as annotations. | 同:1429 |
| （同上） | This remains the case for keywords defined by unrecognized vocabularies. | 同:1430 |
| 語彙に属する未知のキーワードと、どの語彙にも属さない未知のキーワードは、いまは区別できない | It is not currently possible to distinguish between unrecognized keywords that are defined in vocabularies from those that are not part of any vocabulary. | 同:1431 |

### c. 方言・メタスキーマで独自の語彙を足す方法

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| 方言は、語彙の集合と、メタスキーマで示した必須かどうかの組である | A dialect is defined as a set of vocabularies and their required support identified in a meta-schema. | draft-bhutton-json-schema-01.txt:256 |
| 語彙の仕様は、使う範囲の外へ公開しなくてよい | In particular, in order to facilitate vocabulary use within non-public organizations, a vocabulary specification need not be published outside of its scope of use. | 同:527 |
| メタスキーマは、どの語彙にも属さないキーワードも記述・検証できる | Meta-schemas may also describe and validate additional keywords that are not part of a formal vocabulary. | 同:541 |
| 明示の合意が無ければ、追加した語彙を実装が扱うと期待してはならない | Save for explicit agreement, schema authors SHALL NOT expect these additional keywords and vocabularies to be supported by implementations that do not explicitly document such support. | 同:717 |
| 対応していない語彙の処理を登録・読み込む機能を、実装は持ってよい | Implementations MAY provide the ability to register or load handlers for vocabularies that they do not support directly. | 同:733 |
| その登録の仕組みは実装依存である | The exact mechanism for registering and implementing such handlers is implementation-dependent. | 同:734 |
| 語彙の作者は、その語彙のキーワードを検証するメタスキーマを用意すべきである | Vocabulary authors SHOULD provide a meta-schema that validates the expected usage of the vocabulary's keywords on their own. | 同:3847 |
| 正式な名前空間の仕組みは無い | JSON Schema does not provide any formal namespacing system, but also does not constrain keyword names, allowing for any number of namespacing approaches. | 同:3827 |
| 衝突する語彙を宣言したときの振る舞いは未定義 | If conflicting vocabularies are declared, the resulting behavior is undefined. | 同:3844 |
| 語彙の外でキーワードの意味を表す仕組みは無い | JSON Schema does not define any mechanism for expressing keyword semantics outside of vocabularies, making them unsuitable for use except in a specific environment in which they are understood. | 同:3902 |
| 例のメタスキーマは、独自の語彙を true で宣言している | "https://example.com/vocab/example-vocab": true | 同:3933 ・ 3960 |
| 標準の語彙でなく独自の語彙にキーワードを足すことを勧める（format の文脈） | it is RECOMMENDED to define additional keywords in a custom vocabulary rather than additional | draft-bhutton-json-schema-validation-01.txt:719 |

### d. annotation と assertion の区別

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| assertion は、インスタンスが制約を満たすかを真偽で返す | Assertions validate that an instance satisfies constraints, producing a boolean result. | draft-bhutton-json-schema-01.txt:741 |
| annotation は、アプリケーションが自由に使う情報を付ける | Annotations attach information that applications may use in any way they see fit. | 同:742 |
| 拡張キーワードはこの分類にとどまるべきである | Extension keywords SHOULD stay within these categories, keeping in mind that annotations in particular are extremely flexible. | 同:746 |
| 複雑な振る舞いは、annotation をもとにアプリケーションへ任せるほうがよい | Complex behavior is usually better delegated to applications on the basis of annotation data than implemented directly as schema keywords. | 同:747 |
| ただし、特別な目的のために別の振る舞いを定義してもよい | However, extension keywords MAY define other behaviors for specialized purposes. | 同:750 |
| インスタンスが失敗するのは、スキーマに在る assertion に対してだけである | An instance can only fail an assertion that is present in the schema. | 同:967 |
| 実装は、集めた annotation をアプリケーションの代わりに使うことは期待されない | JSON Schema implementations are not expected to make use of the collected information on behalf of applications. | 同:1029 |
| assertion が false のスキーマは annotation を出さない | Schema objects that produce a false assertion result MUST NOT produce any annotation results, whether from their own keywords or from keywords in subschemas. | 同:1206 |
| 意味の検証はアプリケーションで行うのが勧める実践である（format の文脈） | encourage relying only on the annotation behavior and performing | draft-bhutton-json-schema-validation-01.txt:645 |
| （同上） | semantic validation in the application, which is the recommended | 同:646 |

### e. 出力の形とエラーの場所

| 主張 | 引用（原文そのまま） | 出どころ |
|---|---|---|
| 出力の形は4つ | This specification defines four output formats. | draft-bhutton-json-schema-01.txt:2836 |
| Flag | Flag - A boolean which simply indicates the overall validation result with no further details. | 同:2839 |
| Basic | Basic - Provides validation information in a flat list structure. | 同:2842 |
| Detailed | Detailed - Provides validation information in a condensed hierarchical structure based on the structure of the schema. | 同:2844 |
| Verbose | Verbose - Provides validation information in an uncondensed hierarchical structure that matches the exact structure of the schema. | 同:2847 |
| キーワードの相対位置は JSON Pointer で、`$ref` などを含む | The value MUST be expressed as a JSON Pointer, and it MUST include any by-reference applicators such as "$ref" or "$dynamicRef". | 同:2881 |
| キー名 | The JSON key for this information is "keywordLocation". | 同:2892 |
| キー名 | The JSON key for this information is "absoluteKeywordLocation". | 同:2924 |
| キー名 | The JSON key for this information is "instanceLocation". | 同:2931 |
| キー名 | The JSON key for failed validations is "error"; for successful validations it is "annotation". | 同:2943 |
| エラーの文言は規格が決めない | For errors, the specific wording for the message is not defined by this specification. | 同:2937 |
| 出力は真偽の `valid` を持つ | The output MUST be an object containing a boolean property named "valid". | 同:2958 |

### f. JSON-LD の `@context`・`@protected`・expansion

```
主張      context は term を IRI へ対応付ける
引用      <p>Simply speaking, a <a href="#dfn-context" class="internalDFN" data-link-type="dfn">context</a> is used to map <a href="#dfn-term" class="internalDFN" data-link-type="dfn">terms</a> to <a data-link-type="dfn" href="https://tools.ietf.org/html/rfc3987#section-2">IRIs</a>.
出どころ  www.w3.org_TR_json-ld11:2401

主張      @protected が true の term 定義は、後の context による上書きを防ぎ、試みると誤りとして処理を止める
引用      <p>A <dfn data-dfn-type="dfn" id="dfn-protected-term-definition">protected term definition</dfn> is a term definition with an <a data-link-type="dfn" href="https://infra.spec.whatwg.org/#map-entry">entry</a> <code>@protected</code> set to <code>true</code>.
出どころ  www.w3.org_TR_json-ld11:4901
引用      It generally prevents further contexts from overriding this term definition,     either through a new definition of the same term,     or through clearing the context with <code>"@context": null</code>.     Such attempts will raise an error and abort the processing
出どころ  www.w3.org_TR_json-ld11:4902-4905

主張      同じ定義での再定義は許される。理由は、term の意味を変えないため
引用      The first exception is that a context is allowed to redefine a protected term     if the new definition is identical to the protected term definition     (modulo the <code>@protected</code> flag).     The rationale is that the new definition does not violate the protection,     as it does not change the semantics of the protected term.
出どころ  www.w3.org_TR_json-ld11:5019-5023

主張      Expansion は、context を当てて IRI・型・値を展開し、@context を要らなくする処理である
引用      <dfn data-dfn-type="dfn" id="dfn-expansion" data-no-export=""><a href="https://www.w3.org/TR/json-ld11-api/#dfn-expanded">Expansion</a></dfn> is the process of taking a JSON-LD document and applying a     <a href="#dfn-context" class="internalDFN" data-link-type="dfn">context</a> such that all IRIs, types, and values     are expanded so that the <code>@context</code> is no longer necessary.</p>
出どころ  www.w3.org_TR_json-ld11:9920-9922
```

- 上の3つの節（3.1 The Context・4.1.11 Protected Term Definitions・5.1 Expanded Document Form）は、どれも `This section is non-normative.` と書いてある（`www.w3.org_TR_json-ld11:2390`・`4888`・`9916`）。
- **「業務の意味を保証する」と書いた箇所は、読めた範囲に無い。** 取得した JSON-LD 1.1 の原文に `validat` という文字列は1か所も無い（`grep -c -i validat` で 0。これは `text` 相当の探索で、照合の証しではない）。原文が `semantics` に触れるのは、`@protected` の再定義が term の意味を変えない、という文脈（5023行）などである。
- expansion の手順そのもの（アルゴリズム）は JSON-LD 1.1 Processing Algorithms and API にあり、**取得していない**（4章を参照）。

### g. SHACL の Validation Report と `sh:closed`

```
主張      SHACL に準拠する処理系は、必要な検証結果をすべて含む validation report を返せなければならない
引用      SHACL-compliant processors <em class="rfc2119" title="MUST">MUST</em> be capable of returning a validation report with all required <a href="#dfn-validation-results" class="internalDFN" data-link-type="dfn">validation results</a> described
出どころ  www.w3.org_TR_shacl:2440

主張      sh:conforms は、検証結果が1件も無いときだけ true
引用      Each SHACL instance of <code>sh:ValidationReport</code> in the results graph has exactly one value for the property <code>sh:conforms</code> and the value is of datatype <code>xsd:boolean</code>. It represents the outcome of the                         <a href="#dfn-conformance-checking" class="internalDFN" data-link-type="dfn">conformance checking</a>. The value of <code>sh:conforms</code> is <code>true</code> if and only if the <a href="#dfn-validation" class="internalDFN"                             data-link-type="dfn">validation</a> did not produce any <a href="#dfn-validation-results" class="internalDFN" data-link-type="dfn">validation results</a>, and <code>false</code> otherwise.
出どころ  www.w3.org_TR_shacl:2479-2481

主張      必須の項目は sh:focusNode・sh:resultSeverity・sh:sourceConstraintComponent の3つだけ
引用      All the properties described in the remaining sub-sections of this section can be specified in a <code>sh:ValidationResult</code>. The properties <code>sh:focusNode</code>, <code>sh:resultSeverity</code> and <code>sh:sourceConstraintComponent</code>                    are the only properties that are mandatory for all validation results.
出どころ  www.w3.org_TR_shacl:2513

主張      sh:focusNode はちょうど1つ
引用      Each validation result has exactly one value for the property <code>sh:focusNode</code> that is equal to the <a href="#dfn-focus-node" class="internalDFN" data-link-type="dfn">focus node</a> that has caused the result. This is
出どころ  www.w3.org_TR_shacl:2520

主張      sh:resultPath は持ってもよい（may）
引用      Validation results may have a value for the property <code>sh:resultPath</code> pointing at a <a href="#dfn-well-formed" class="internalDFN" data-link-type="dfn">well-formed</a> <a href="#dfn-shacl-property-path" class="internalDFN"
出どころ  www.w3.org_TR_shacl:2528

主張      sh:sourceConstraintComponent はちょうど1つで、原因になった constraint component の IRI
引用      Validation results have exactly one value for the property <code>sh:sourceConstraintComponent</code> and this value is the <a href="#dfn-iri" class="internalDFN" data-link-type="dfn">IRI</a> of the <a href="#dfn-constraint-component"
出どころ  www.w3.org_TR_shacl:2552

主張      sh:resultMessage は持ってもよい。人へ伝えるため
引用      Validation results may have values for the property <code>sh:resultMessage</code>, for example to communicate additional textual details to humans. While <code>sh:resultMessage</code> may have multiple values, there should not
出どころ  www.w3.org_TR_shacl:2570

主張      sh:resultMessage の具体の値は SHACL が決めない
引用      The following graph represents an example of a validation report for the validation of a data graph that does not conform to a shapes graph. Note that the specific value of <code>sh:resultMessage</code> is not mandated by SHACL and considered                 implementation-specific.
出どころ  www.w3.org_TR_shacl:2451-2452

主張      sh:resultSeverity はちょうど1つで、shape の sh:severity、無ければ sh:Violation
引用      Each validation result has exactly one <a href="#dfn-value" class="internalDFN" data-link-type="dfn">value</a> for the property <code>sh:resultSeverity</code>, and this value is an <a href="#dfn-iri" class="internalDFN" data-link-type="dfn">IRI</a>.                         The value is equal to the <a href="#dfn-value" class="internalDFN" data-link-type="dfn">value</a> of <a href="#severity"><code>sh:severity</code></a> of the <a href="#dfn-shape" class="internalDFN" data-link-type="dfn">shape</a>                        in the <a href="#dfn-shapes-graph" class="internalDFN" data-link-type="dfn">shapes graph</a> that caused the result, defaulting to <code>sh:Violation</code> if no <code>sh:severity</code> has been specified for the shape.
出どころ  www.w3.org_TR_shacl:2579-2580

主張      sh:closed は、sh:property で列挙した property shape の property だけを値に持つという条件を指定する
引用      property called <code>sh:closed</code> that can be used to specify the condition that each value node has <a href="#dfn-value" class="internalDFN" data-link-type="dfn">values</a> only for those properties that have                     been explicitly enumerated via the                     <a href="#dfn-property-shape" class="internalDFN" data-link-type="dfn">property shapes</a> specified for the shape via <code>sh:property</code>.
出どころ  www.w3.org_TR_shacl:4230-4232
引用      Set to <code>true</code> to close the shape.
出どころ  www.w3.org_TR_shacl:4248

主張      $closed が true なら、列挙されていない述語の triple ごとに検証結果を出す。$ignoredProperties の要素も許される
引用      If <code>$closed</code> is <code>true</code> then there is a <a href="#dfn-validation-results" class="internalDFN" data-link-type="dfn">validation result</a> for each <a href="#dfn-rdf-triple" class="internalDFN" data-link-type="dfn">triple</a>                        that has a <a href="#dfn-value-nodes" class="internalDFN" data-link-type="dfn">value node</a> as its                         <a href="#dfn-subject" class="internalDFN" data-link-type="dfn">subject</a> and a <a href="#dfn-predicate" class="internalDFN" data-link-type="dfn">predicate</a> that is not explicitly enumerated as a <a href="#dfn-value" class="internalDFN"                             data-link-type="dfn">value</a> of <code>sh:path</code> in any of the <a href="#dfn-property-shape" class="internalDFN" data-link-type="dfn">property shapes</a> declared via <code>sh:property</code> at the current shape. If <code>$ignoredProperties</code>                        has a value then the properties enumerated as <a href="#dfn-members" class="internalDFN" data-link-type="dfn">members</a> of this <a href="#dfn-shacl-list" class="internalDFN" data-link-type="dfn">SHACL list</a> are also permitted
出どころ  www.w3.org_TR_shacl:4265-4267
```

- 構想の docs/12 の34行目「Validation Report | focus node、path、source constraint、severity、messageを返す」について：原文では、必ず持つのは focusNode・resultSeverity・sourceConstraintComponent の3つで（2513行）、resultPath と resultMessage は may である（2528・2570行）。**docs/12 の書き方は、5つとも返すと読める。**

---

## 3. 構想の文書の識別子の照合（`--as identifier`）

| 文書 | 識別子 | 照合した原文 | 結果 |
|---|---|---|---|
| docs/12:10〜15 | `@context` `@id` `@type` `@protected` `@graph` | `www.w3.org_TR_json-ld11` | 5／5 在る（例：`@protected` 22か所、2173行ほか） |
| docs/12:14 | `Expansion` `Compaction` | `www.w3.org_TR_json-ld11` | 2／2 在る（Expansion 9か所、Compaction 10か所） |
| docs/12:30〜33 | `NodeShape` `PropertyShape` `sh:path` `sh:minCount` `sh:maxCount` `sh:class` `sh:closed` | `www.w3.org_TR_shacl` | 7／7 在る（`sh:NodeShape` `sh:PropertyShape` の形でも在る） |
| 本書 2.g | `sh:ignoredProperties` `sh:ValidationReport` `sh:ValidationResult` `sh:focusNode` `sh:resultPath` `sh:sourceConstraintComponent` `sh:resultSeverity` `sh:resultMessage` `sh:sourceShape` `sh:value` `sh:conforms` `sh:Violation` `sh:ClosedConstraintComponent` | `www.w3.org_TR_shacl` | 13／13 在る |
| 本書 2.a〜e | `$vocabulary` `$schema` `$id` `$dynamicAnchor` `$dynamicRef` `keywordLocation` `absoluteKeywordLocation` `instanceLocation` `valid` `errors` `annotations` `error` `annotation` | `draft-bhutton-json-schema-01.txt` | 13／13 在る |

合わせて 40個を照合し、40個が原文に在った。docs/14・docs/19 には、規格の識別子（`@` や `sh:` や `$` で始まる語）が無かった。docs/19 は URL の一覧である。

---

## 4. 照合できなかったもの・読めなかった範囲

| 項目 | 状態 |
|---|---|
| 照合して原文に無かった識別子 | 無い |
| 最初に `quote` で照合に失敗した引用 | 9件（JSON-LD 2件・SHACL 7件）。原因はすべて、こちらがタグを除いた文字列を渡したこと。タグを含む原文の文字列で照合し直し、全件が一致した（2.f・2.g に載せたもの） |
| 読めなかったファイル | 無い。取得した6件は全部読めた |
| 取得していない原典 | JSON-LD 1.1 Processing Algorithms and API（https://www.w3.org/TR/json-ld11-api/）・RDF 1.1 Concepts・JSON Schema のメタスキーマ本体（https://json-schema.org/draft/2020-12/schema など）・json-schema.org の Output 形式の後継の文書・実装（Rust の jsonschema など）。**これらについての主張はここに書いていない** |
| 読んだ範囲 | JSON Schema Core は 3〜4.3・6.5〜7.7・8.1・12・付録D を読んだ。Validation は 7.2 を読んだ。JSON-LD は 3.1・4.1.11・5.1・11〜13 を読んだ。SHACL は 3.6・4.8.1 を読んだ。ほかの節は読んでいない |

---

## 5. 反証の仮説について、原文で言えること・言えないこと

### 原文で言えること

1. **独自の語彙は、誰でも定義でき、公開しなくてもよい。** 717行（定義は誰でもよい）・527行（公開しなくてよい）。
2. **必須の語彙を認識しない実装に処理を拒否させる仕組みは、規格にある。** `$vocabulary` の値を true にすると、その語彙を認識しない実装は、そのメタスキーマを宣言するスキーマの処理を拒否しなければならない（1421行）。規格自身も、Format-Assertion 語彙で同じ形を使っている（validation 651・653・714行）。
3. **ただし、その仕組みには条件がある。**
   - `$vocabulary` が効くのは、メタスキーマとして処理する文書の中だけである（1439行）。
   - 宣言は `$ref` で継承されないので、メタスキーマごとに書き直す（1474行）。
   - 局所用のメタスキーマは `$vocabulary` を省いてよい（3879行）。省けば、この拒否は働かない。
   - 語彙に入れずにキーワードを足しただけなら、そのキーワードは annotation として集められる（261・720行）。つまり、この場合は失敗にならない。
   - 語彙に属する未知のキーワードと、属さない未知のキーワードは、いまは区別できない（1431行）。
4. **認識している語彙は、その語彙の定義どおりに処理しなければならない**（1391行）。
5. **独自の語彙の処理を実装へ登録する仕組みは、規格では決まっていない。** 実装は持ってもよい（MAY）、仕組みは実装依存である（733・734行）。
6. **拡張キーワードの振る舞いは、assertion・annotation・applicator の外も許される**（750行、MAY）。ただし「この分類にとどまるべき」（746行、SHOULD）で、「複雑な振る舞いは annotation をもとにアプリケーションへ任せるほうがよい」（747行）と書いてある。
7. **名前空間の仕組みは無く、衝突する語彙を宣言したときの振る舞いは未定義である**（3827・3844行）。
8. **出力は `valid` を必ず持ち、場所を `keywordLocation`・`absoluteKeywordLocation`・`instanceLocation` で表す**（2958・2892・2924・2931行）。エラーの文言は規格が決めない（2937行）。
9. **JSON Schema 2020-12 は Internet-Draft で、`Intended status: Informational` である**（6行）。

### 原文では言えないこと

1. **「表現を制限されずに」** ── 原文にこの趣旨の記述は無い。言えるのは、拡張キーワードが別の振る舞いを定義してもよい（750行）ことまでである。時間の条件（docs/14:88 の例）のような意味を、語彙のキーワードとして書いて検証させられるかは、原文からは決まらない。（推測：書くことはできるが、検証するのは、その語彙を理解する処理を持つ実装だけである。）
2. **実際の実装が、未知の必須語彙で処理を拒否するか** ── 原文は実装への要件（MUST）を書いているだけである。Rust の jsonschema などの実装は調べていない。
3. **「一部だけ検証できた」を報告する仕組み** ── 2020-12 の出力形式に、語彙ごとの対応の有無（docs/14 の `coverage`・`notRepresented` のようなもの）を返す項目は、読んだ範囲（12章）に無かった。12章以外は読み通していないので、「規格に無い」とは結論しない。（推測：拒否は処理全体の単位で起きるので、「語彙Aは検証した、語彙Bは検証できなかった」という部分の報告は、2020-12 の仕組みだけでは出せない。）
4. **JSON-LD が業務の意味を保証するか** ── 保証すると書いた箇所は、読めた範囲に無い。原文が書くのは、context が term を IRI へ対応付けること（2401行）と、`@protected` が term 定義の上書きを止めること（4901〜4905行）で、どちらの節も non-normative である。
5. **SHACL を JSON に直接当てられるか** ── この調査では読んでいない。SHACL の節で読んだのは Validation Report と `sh:closed` だけである。

### 仮説への答え（原文で言える範囲）

- 後半「未知の必須語彙を処理できない実装は失敗にする仕組みが、もともと規格にある」── **原文で支えられる**（1421行）。ただし上の3の条件の内側に限る。
- 前半「自前のモデルを正本にしなくても、表現を制限されずに独自の意味を足せる」── **足す仕組み（語彙・メタスキーマ）があることは原文で支えられる**（717・527・541行）。**「表現を制限されずに」と「検証まで含めて足せる」は、原文では支えられない**（1・3）。
