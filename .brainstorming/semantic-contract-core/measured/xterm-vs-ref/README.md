# 欄と term の結び付け方：x-term と $ref の反証

スキーマの欄と、用語集 glossary の term を結び付ける方法を1つに決めるための材料である。どの案も持ち上げず、各案が壊れる場面を断片つきで挙げる。

## 調べ方と限界

| 項目 | 内容 |
|---|---|
| 原典 | `../../sources/www.ietf.org_archive_id_draft-bhutton-json-schema-01.txt`（Core）と `../../sources/www.ietf.org_archive_id_draft-bhutton-json-schema-validation-01.txt`（Validation）。2026-10-17 に fact-check が取得した原文で、sha256 は同じ場所の `.meta.json` にある |
| 版の照合 | json-schema.org の 2020-12 の頁（`../../sources/json-schema.org_draft_2020-12_json-schema-core`）は `<meta content="draft-bhutton-json-schema-01" name="ietf.draft">`（29行）で、上の txt と同じ版である |
| 今回取得しなかった理由 | この作業では fact-check の fetch とコマンドの実行が許可されなかった。保存済みの原文を Read と Grep で読んだ |
| 実測していないもの | 実装（Rust の jsonschema 0.58.5 ・ Python の jsonschema など）での振る舞いは、どれも動かしていない。下の表で「実装：確かめられなかった」と書いた所がそれである |

## 1. 仕様の原文

行番号は Core の txt のもの。HTML の頁の行は括弧に書いた。

| # | 論点 | 原文 | 出どころ |
|---|---|---|---|
| S1 | $ref の隣のキーワード | "The "$ref" keyword is an applicator that is used to reference a statically identified schema.  Its results are the results of the referenced schema." / "// Note that this definition of how the results are determined means // that other keywords can appear alongside of "$ref" in the same // schema object." | Core 1654〜1659（HTML 3135） |
| S2 | 隣のキーワードの呼び名 | "Keywords which are properties within the same schema object are referred to as adjacent keywords." | Core 468〜469 |
| S3 | URI は取得先ではない | "The resolved URI produced by these keywords is not necessarily a network locator, only an identifier." / "implementations SHOULD NOT assume they should perform a network operation when they encounter a network-addressable URI." | Core 1646〜1650 |
| S4 | 事前に知っておく | "The use of URIs to identify remote schemas does not necessarily mean anything is downloaded, but instead JSON Schema implementations SHOULD understand ahead of time which schemas they will be using, and the URIs that identify them." | Core 1826〜1829 |
| S5 | 任意の URI を登録できる | "Implementations SHOULD be able to associate arbitrary URIs with an arbitrary schema and/or automatically associate a schema's "$id"- given URI, depending on the trust that the validator has in the schema." | Core 1835〜1838 |
| S6 | 渡されていない参照先 | "If a schema with that identifier has otherwise been supplied to the implementation, it can also be used automatically." / "// What should implementations do when the referenced schema is not // known?  Are there circumstances in which automatic network // dereferencing is allowed?" | Core 1936〜1941 |
| S7 | 標準外の場所を指す参照 | "Multi-level structures of unknown keywords are capable of introducing nested subschemas, which would be subject to the processing rules for "$id".  Therefore, having a reference target in such an unrecognized structure cannot be reliably implemented, and the resulting behavior is undefined." | Core 2176〜2180（HTML 3621） |
| S8 | 1段の独自キーワードの下 | "Note that single-level custom keywords with identical syntax and semantics to "$defs" do not allow for any intervening "$id" keywords, and therefore will behave correctly under implementations that attempt to use any reference target as a schema.  However, this behavior is implementation-specific and MUST NOT be relied upon for interoperability." | Core 2199〜2204 |
| S9 | $defs | "The "$defs" keyword reserves a location for schema authors to inline re-usable JSON Schemas into a more general schema." / "Each member value of this object MUST be a valid JSON Schema." | Core 1713〜1718 |
| S10 | 知らないキーワード | "Unknown keywords SHOULD be treated as annotations, where the value of the keyword is the value of the annotation." / "An empty schema is a JSON Schema with no properties, or only unknown properties." | Core 474〜479（HTML 1990） |
| S11 | 注釈は $ref の先から集まる | "In addition to possibly defining annotation results of their own, applicator keywords aggregate the annotations collected in their subschema(s) or referenced schema(s)." | Core 1244〜1246 |
| S12 | 注釈がどこから来たかが分かる | "The schema location path, indicating how reference keywords such as "$ref" were followed to reach the absolute schema location." | Core 1057〜1058 |
| S13 | 同じ注釈が2か所にあるとき | "The application programmer and the schema author need to agree on the usage." | Core 1169〜1170 |
| S14 | 失敗した所の注釈は捨てる | "Schema objects that produce a false assertion result MUST NOT produce any annotation results, whether from their own keywords or from keywords in subschemas." | Core 1206〜1208 |
| S15 | $anchor | "If present, the value of this keyword MUST be a string and MUST start with a letter ([A-Za-z]) or underscore ("_"), followed by any number of letters, digits ([0-9]), hyphens ("-"), underscores ("_"), and periods (".")." | Core 1611〜1614 |
| S16 | $anchor の利点 | "it may be preferable to use a plain name fragment that is not tied to any particular structural location.  This allows a subschema to be relocated without requiring JSON Pointer references to be updated." | Core 1583〜1586 |
| S17 | $id に断片を書けない | "Therefore, "$id" MUST NOT contain a non-empty fragment, and SHOULD NOT contain an empty fragment." | Core 1539〜1540 |
| S18 | $ref に意味を持たせるとき | "To make such use case-specific semantics explicit, the best practice is to create an annotation keyword for use in the same schema object alongside of a reference keyword such as "$ref"." | Core 4011〜4013 |
| S19 | 必須の語彙を知らない実装 | "If the value is true, then implementations that do not recognize the vocabulary MUST refuse to process any schemas that declare this meta-schema with "$schema"." | Core 1421〜1424 |
| S20 | title と description | "Both of these keywords can be used to decorate a user interface with information about the data produced by this user interface." | Validation 1136〜1137 |

いまの基盤のエンジンについては、リポジトリに次の実測がある（`../../sources/adapters/README.md` 107〜109行）。

| 内容 | 原文 |
|---|---|
| 登録に無い参照は構築の時点で失敗する | "[`ValidationOptions::offline`] disables retrieval, so a reference that is not already in the //! registry fails at construction instead of being fetched:" |
| そのときのメッセージ | "Err(format!("Retrieval is disabled, cannot fetch {uri}").into())" |
| 取得の仕方を差し替えられる | "A retriever is for documents fetched on demand — from a database, an embedded asset, a cache." |

## 2. 比べる案

| 案 | 欄に書くもの | 値の形の出どころ |
|---|---|---|
| A | `"x-term": "<term id>"`、根に `"x-glossaries": [...]` | 欄に別に書く |
| B | `"$ref": "<glossary の URI>#/terms/<term id>"` | term の値の形（$ref の先） |
| C1 | `"$ref": "<glossary の URI>#<term id>"`。glossary を JSON Schema の文書にし、term を `$defs` に置いて `$anchor` を付ける | term の値の形（$ref の先） |
| C2 | `"$ref"` と `"x-term"` を同じ欄に並べる（S18 の形） | term の値の形（$ref の先） |

B の結果は、glossary の中で term をどう持つかで変わる。次の2つの持ち方で確かめた。

```json
// 持ち方 P：term の値の形を "schema" の下に置く
{ "id": "order", "version": 2,
  "terms": { "quantity": { "id": "quantity", "description": "注文の数量", "unit": "個",
                            "schema": { "type": "integer", "minimum": 1 } } } }

// 持ち方 Q：term そのものを値の形として書く
{ "id": "order", "version": 2,
  "terms": { "quantity": { "description": "注文の数量", "x-unit": "個",
                            "type": "integer", "minimum": 1 } } }
```

## 3. 壊れる場面

### 3.1 値の形を持たない term（例：承認者）

| 案 | 断片 | 結果 |
|---|---|---|
| A | `"approver": { "x-term": "approver", "type": "string" }` | 壊れない。欄が自分の形を書く |
| B・持ち方 P | `"approver": { "$ref": ".../order/2#/terms/approver/schema" }` | **壊れる**。term に `schema` が無いので参照が解決しない。エンジンは構築の時点で止まる（上の実測と同じ形の失敗と見込むが、実装：確かめられなかった） |
| B・持ち方 P | `"approver": { "$ref": ".../order/2#/terms/approver" }` と書いて逃げる | **壊れる**。今度は値の形を持つ term（quantity）で、`schema` が知らないキーワードになり注釈として扱われる（S10）。値の形が検証されず、何も言われずに通る |
| B・持ち方 Q | `"approver": { "$ref": ".../order/2#/terms/approver", "type": "string" }` | 先は知らないキーワードだけの空のスキーマ（S10）になり、隣の `type` だけが効く。壊れない |
| C1 | `"$defs": { "approver": { "$anchor": "approver", "description": "承認する人" } }` を指す | 持ち方 Q と同じで壊れない |
| C2 | C1 と同じ | 壊れない |

### 3.2 範囲を絞った欄（数量を 1〜99 に絞る）

| 案 | 断片 | 結果 |
|---|---|---|
| A | `"qty": { "x-term": "order:quantity", "type": "string" }` | **壊れる**。term は integer なのに、欄は string と書けてしまう。一般の実装は x-term を見ないので検出しない。Core が term の形も一緒に当てる（欄の形と term の形の両方を満たすことを求める）決まりを持たないかぎり、同じ term の欄どうしで形がずれる |
| A | `"qty": { "x-term": "order:quantity", "type": "integer", "maximum": 99 }` | 欄の形だけを見る実装では `0` が通る（term の minimum 1 が効かない） |
| B・C1・C2 | `"qty": { "$ref": ".../order/2#quantity", "maximum": 99 }` | 壊れない。隣に置いた `maximum` は $ref の結果と一緒に効く（S1・S2）。`0` は term の `minimum` で、`100` は欄の `maximum` で落ちる |
| B・C1・C2 | `"qty": { "$ref": ".../order/2#quantity", "type": "string" }` | 矛盾した欄が書けてしまい、どの値も通らない欄になる。検証では落ちるが、スキーマを読んだだけでは知らされない |
| B・C1・C2 | 欄が term と別の description を持つ | 注釈が2つ集まる。どちらを使うかは決まっておらず、使う側と書く側で取り決める（S13）。位置は S12 で区別できる |

### 3.3 版を上げた term を指す欄

前提：order の glossary が版 2 を出し、版 1 は変えずに残っている。

| 案 | 断片 | 結果 |
|---|---|---|
| A | `{ "x-glossaries": ["order@1"], "properties": { "qty": { "x-term": "quantity", "type": "integer" } } }` を版 2 へ上げるとき | 根の1行を直すだけで済む。ただし欄の `type` は版 1 のときに書いたままで、版 2 で term の形が変わっても欄は変わらない。ずれは Core が照合しないかぎり残る |
| A | 1つのスキーマで、欄 a は order@1、欄 b は order@2 を使いたい | **扱えない**。`x-glossaries` に両方を並べると、どちらにも `quantity` があり、`"x-term": "quantity"` がどちらを指すか決まらない |
| B・C1・C2 | `"qty": { "$ref": "https://.../glossaries/order/2#quantity" }` | 欄ごとに版を持てる。版を上げるには、その term を指す欄をすべて直す |
| B・C1・C2 | 版 2 を登録していないエンジンで読む | 止まる（実測の「Retrieval is disabled」と同じ形）。ずれたまま通ることは無いが、Core が使う glossary の版を登録する一覧を持つことが前提になる（S4・S5） |
| B・C1・C2 | 版を URI の断片に入れる：`"$id": ".../order#v2"` | **書けない**。$id は断片を持てない（S17）。版は断片より前に入れる |

### 3.4 別の文脈の term を指す欄

| 案 | 断片 | 結果 |
|---|---|---|
| A | `{ "x-glossaries": ["shop", "order@2"], ... "x-term": "quantity" }` で、shop と order の両方に quantity がある | **壊れる**。どちらの term か決まらない。term id に文脈を付ける（`"order:quantity"` など）か、重なりを誤りにする決まりが要る |
| A | 欄が、別の文書（自分の `x-glossaries` を持つ）の定義を `$ref` で使う：`"line": { "$ref": "https://.../shared/line-item" }`、その先に `"x-term": "quantity"` | **壊れる**。根の `x-glossaries` が、$ref の先の文書にも効くのかが決まっていない。文書ごとに効くと決める必要がある |
| B・C1・C2 | `"$ref": "https://.../glossaries/order/2#quantity"` | 文脈は URI に入っているので、重なりは起きない |

### 3.5 意味で探すとき、欄から term を引けるか

| 案 | 断片 | 結果 |
|---|---|---|
| A | `"x-term": "order:quantity"` | スキーマを読めば欄ごとに term が1つ出る。一般の実装も、知らないキーワードを注釈として集めるなら（S10、SHOULD）出力に term id が載る |
| A | 不正なインスタンスを検証した出力から term を引く | **引けない**。失敗した所の注釈は捨てる（S14）。スキーマを読んで引く必要がある（B・C1・C2 も同じ） |
| B | `"$ref": "https://.../glossaries/order/2#/terms/quantity"` | URI を分解して文脈 ・ 版 ・ term を取り出す決まりが要る。term id は標準の注釈ではないので、出力からは引けない |
| B・C1 | 欄 → ローカルの `$defs` → glossary と2段で参照する：`"qty": { "$ref": "#/$defs/qty" }`、`"$defs": { "qty": { "$ref": ".../order/2#quantity", "maximum": 99 } }` | 欄の $ref だけを見ると term が見えない。$ref をたどる決まりが要る |
| B・C1 | 同じスキーマで、term を指す $ref と、構造を再利用するだけの $ref が混ざる | どれが term を指すかを、URI の前半（glossary の置き場所）で見分ける決まりが要る |
| B・C1 | `"qty": { "anyOf": [ { "$ref": "...#quantity" }, { "$ref": "...#count" } ] }` | 欄が term を2つ持つことになり、1つに決まらない |
| C1 | term 側に `"x-term": "quantity"` を置き、$ref でたどった注釈として集める（S11） | 出力にも term id が載る。ただし S14 で、失敗した所では消える |
| C2 | `"qty": { "$ref": ".../order/2#quantity", "x-term": "order:quantity" }` | 欄で直接読める。S18 が勧める形である |
| C2 | `"qty": { "$ref": ".../order/2#price", "x-term": "order:quantity" }` | **壊れる**。2か所が食い違っても、一般の実装は検出しない。Core が一致を検査する必要がある |

### 3.6 Concrete が自分で足した x- との衝突

| 案 | 断片 | 結果 |
|---|---|---|
| A | Concrete が、外から取り込んだスキーマに別の意味の `"x-term": "Glossary entry for UI"` を持っている | **壊れる**。Core が自分のキーワードとして読み、存在しない term として誤りにするか、誤った term に結び付ける。Abstract のメタスキーマで x-term の値の形（term id の形）を決め、Core の名前を予約しておく必要がある |
| A | Concrete が `"x-glossary"`（単数）を自分の意味で足す | Core の `x-glossaries` と1文字違いで並ぶ。メタスキーマでは誤りにならない |
| B・C1 | 結び付けに x- を使わない | 結び付けの部分では衝突しない。ただし C1 で term に付ける `x-unit` などは同じ問題を持つ |
| C2 | A と同じ | A と同じ |

### 3.7 JSON Schema の一般の実装（Core 以外）で読んだとき

| 案 | 断片 | 結果 |
|---|---|---|
| A | `"qty": { "x-term": "order:quantity", "type": "integer" }` | スキーマ1枚で閉じていて、取得なしで検証できる。x-term は注釈になるか無視される（S10 は SHOULD）。term の形は効かないので、Core と一般の実装で検証の結果が変わりうる（3.2 の例） |
| B・C1・C2 | glossary を渡さずに読む | 参照が解決しない。止めるか取得しに行くかは決まっていない（S6）。取得しに行かないことを勧めている（S3） |
| B | glossary を渡して読む | `#/terms/...` は標準のキーワードの下ではない。2段以上なら振る舞いは未定義（S7）、1段でも実装しだいで、相互運用に頼ってはならない（S8）。term の値の形が 2段（`/terms/<id>/schema`）にある持ち方 P は S7 に当たる |
| C1 | glossary を `$defs` に持つ JSON Schema として渡して読む | 標準の場所（S9）なので S7・S8 に当たらない。glossary に `$id` と `$schema` が要る |
| C1 | term id が `数量` や `2nd-qty` | **書けない**。$anchor は ASCII の英字か `_` で始める（S15）。日本語の id は JSON Pointer（`#/$defs/数量` を百分率符号化したもの）で指すことになる |
| A・C2 | Abstract のメタスキーマが Core の語彙を `$vocabulary` で `true` と宣言する | Core の語彙を知らない一般の実装は、そのスキーマを処理してはならない（S19）。`false` にすれば処理は続く |

## 4. 結論

| 案 | 結論 | 条件と理由 |
|---|---|---|
| A | 条件付きで採れる | 理由：スキーマ1枚で閉じていて、どの実装でも取得なしに検証でき、欄から term を直接読める。条件：(1) term id に文脈を含めるか、glossary の重なりを誤りにする（3.4）。(2) `x-glossaries` が効く範囲を、文書（スキーマの資源）ごとと決める（3.4）。(3) Core が欄の形と term の形の両方を当てる。当てないなら、同じ term の欄どうしで形がずれる（3.2）。(4) メタスキーマで x-term と x-glossaries の値の形を決める（3.6）。残る弱さ：一般の実装では term の形が効かず、Core との結果が変わる |
| B | 書かれた形（`#/terms/<id>`）のままでは採れない | 理由：参照先が標準外のキーワードの下にあり、仕様が相互運用に頼ってはならないとしている（S7・S8）。持ち方 P では、値の形を持たない term か持つ term のどちらかで壊れる（3.1）。参照先を `$defs` に移せば C1 になる |
| C1 | 条件付きで採れる | 理由：term の形が標準の仕組みで欄に効き、絞った範囲は隣に書ける（S1）。版と文脈が URI に入る。条件：(1) glossary を `$id` 付きの JSON Schema の文書にし、版を断片より前に入れる（S17）。(2) term id を $anchor の形に収める（S15）か、JSON Pointer で指す。(3) Core が閉じた一覧から glossary を事前に登録する（S4・S5）。登録が漏れると止まる。(4) term を指す $ref の見分け方と、$ref をたどる深さを決める（3.5）。残る弱さ：版を上げると欄をすべて直す。一般の実装は glossary を渡されないと検証できない |
| C2 | 条件付きで採れる | 理由：仕様が勧める形（S18）で、欄から term を読むことと、term の形を効かせることを両方持つ。条件：C1 の条件すべてと、$ref と x-term の一致を Core が検査すること（3.5）。残る弱さ：1つの事実を2か所に書く |

## 5. 確かめられなかったこと

| 項目 | 理由 |
|---|---|
| Rust の jsonschema 0.58.5 が、`#/terms/<id>` のように知らないキーワードの下を指す $ref を解決するか | コマンドの実行が許可されなかった |
| 同じく、知らないキーワード（x-term）を注釈として出力に載せるか | 同上 |
| Python の jsonschema など、ほかの一般の実装の振る舞い | 同上 |
| 今日の時点で json-schema.org の原文が 2026-10-17 の取得から変わっていないか | fact-check の fetch が許可されなかった |
