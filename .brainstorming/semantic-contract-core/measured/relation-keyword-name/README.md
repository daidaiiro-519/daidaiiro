# x-ref の新しい名前の候補を、原典の語義で比べる

## この文書が答える問い

annotations@v1 の注釈 x-ref の新しい名前を、原典にある語から選ぶ。そのために、各候補の語が原典で何を指すかを原文で示し、x-ref の仕事を名前から読み取れるかを候補ごとに評価する。どの候補を採るかは、この文書では決めない。

## x-ref がしていること

定義は `.claude/skills/schema-driven/assets/annotations.schema.json` の `$defs/x-ref` にある。

| 仕事 | 定義の中の根拠 |
|---|---|
| ① 指す先が在るかを確かめる | description「検査は、指す先がある ・ 指す先が受け付ける値か ・ 指される数 ・ 承認のあとの変化を出す」 |
| ② 指される数を、指される側から確かめる | `inverse` の description「指される側から見た数の決まり」 |
| ③ 指す ・ 指されるのつながりの一覧を作る | 描画と「意味で探す」が使う（依頼の記述による。annotations.schema.json には書かれていない） |
| ④ instance のデータを指す | x-prompt.read.means「このプロパティの値の1つ1つが、指す先を表す文字列であることを示す」。`to` は「指す先のインスタンスの kind」 |

紛れる相手は JSON Schema の `$ref` である。

> The "$ref" keyword is an applicator that is used to reference a statically identified schema.
> ── JSON Schema Core 2020-12, §8.2.3.1（照合済み）

`$ref` はスキーマを指し、x-ref は instance を指す。名前で区別したいのはこの1点である。

## 原典での語義

照合の欄：「照合済み」は fact-check verify で原文と一致したもの。「画像で目視」は、PDF が文字として読めず、ページ画像を読んで書き写したもの。

### relationship

| 出どころ | 原文 | 照合 |
|---|---|---|
| Chen 1976, §2.2.2（p.11–12） | "A *relationship set*, R<sub>i</sub>, is a mathematical relation [5] among n entities, each taken from an entity set: {[e1, e2, ..., en] \| e1 ∈ E1, e2 ∈ E2, ..., en ∈ En}, and each tuple of entities, [e1, e2, ..., en], is a *relationship*." | 画像で目視 |
| Chen 1976, §3.1（p.20） | "The diagram can distinguish between 1:n, m:n, and 1:1 mappings." | 画像で目視 |
| Chen 1976, §3.1（p.20） | "Information about the number of entities in each entity set which is allowed in a relationship set is indicated by specifying "1", "m", "n" in the diagram." | 画像で目視 |
| JSON:API 1.1, Relationships | "Each member of a relationships object represents a “relationship” from the resource object in which it has been defined to other resource objects." | 照合済み（前半）。後半は HTML のタグをまたぐので Read で目視 |
| JSON:API 1.1, Relationships | "Relationships may be to-one or to-many." | 照合済み |
| JSON:API 1.1, Resource Linkage | "Resource linkage in a compound document allows a client to link together all of the included resource objects without having to GET any URLs via links." | Read で目視（タグをまたぐ） |

読み取れること：Chen では、relationship は実体（entity）の組そのもので、型の側は relationship set と呼ぶ。数の決まり（1:n ・ m:n ・ 1:1）は relationship set に付く。Chen の読んだ範囲（p.11–21）に cardinality という語は出てこず、数の決まりは mapping と呼ばれている。JSON:API では、relationship は resource object（データ）の名前付きの欄で、ほかの resource object を指す。

### foreign key

| 出どころ | 原文 | 照合 |
|---|---|---|
| PostgreSQL 公式ドキュメント, Constraints の Foreign Keys の節 | "A foreign key constraint specifies that the values in a column (or a group of columns) must match the values appearing in some row of another table." | 照合済み |
| 同上 | "We say this maintains the referential integrity between two related tables." | Read で目視（`<em>` をまたぐ） |
| 同上 | "We say that in this situation the orders table is the referencing table and the products table is the referenced table." | referencing ・ referenced は照合済み。文全体は Read で目視 |

読み取れること：foreign key は「列の値が、別の表のどれかの行の値と一致しなければならない」という制約で、指す先が在ることを確かめる仕事そのものを指す。指される側の数（「必ず1つから指される」）は、この定義に入っていない。

### link

| 出どころ | 原文 | 照合 |
|---|---|---|
| OpenAPI 3.1.0, §4.8.20 Link Object | "The Link object represents a possible design-time link for a response. The presence of a link does not guarantee the caller’s ability to successfully invoke it, rather it provides a known relationship and traversal mechanism between responses and other operations." | 照合済み |
| JSON Hyper-Schema 2019-09, §5.2 links | "The "links" property of schemas is used to associate Link Description Objects with instances." | 照合済み |
| JSON:API 1.1, relationship object | "`links`: a links object containing at least one of the following: ..." （`self` は relationship link、`related` は related resource link） | Read で目視 |

読み取れること：OpenAPI の link は、応答から次の操作へのつながり（操作を指す）で、データの id を指すものではない。JSON Schema の系統（Hyper-Schema）には `links` というキーワードが既にあり、instance に Link Description Object（URI テンプレートで作るハイパーリンク）を付ける。JSON:API の links は URL を持つ欄である。

### relation

| 出どころ | 原文 | 照合 |
|---|---|---|
| Chen 1976, §2.2.2（p.11） | "A *relationship set*, R<sub>i</sub>, is a mathematical relation [5] among n entities" | 画像で目視 |
| Chen 1976, §2.3.2（p.16） | "The whole table in Figure 7 is an *entity relation*, and each row is an *entity tuple*." | 画像で目視 |

読み取れること：Chen では、relation は数学の関係（組の集合）か、レベル2の表（entity relation ・ relationship relation）を指す。欄が別のものを指すことは指さない。

### 足した候補

| 語 | 出どころ | 原文 | 照合 |
|---|---|---|---|
| sh:class | W3C SHACL, §4.1.1 | "The condition specified by sh:class is that each value node is a SHACL instance of a given type." | Read で目視（タグをまたぐ） |
| sh:node | W3C SHACL, §4.7.1 | "sh:node specifies the condition that each value node conforms to the given node shape." | Read で目視 |
| sh:minCount | W3C SHACL, §4.2.1 | "sh:minCount specifies the minimum number of value nodes that satisfy the condition. If the minimum cardinality value is 0 then this constraint is always satisfied and so may be omitted." | 「minimum number of value nodes」は照合済み。文全体は Read で目視 |
| sh:qualifiedValueShape | W3C SHACL, §4.7.3 | "sh:qualifiedValueShape specifies the condition that a specified number of value nodes conforms to the given shape." | Read で目視 |
| sh:inversePath | W3C SHACL, Property Paths の節の例 | "SPARQL Property path: ^ex:parent / SHACL Property path: [ sh:inversePath ex:parent ]" | sh:inversePath は照合済み |
| node reference | JSON-LD 1.1, 用語の定義 | "A node object used to reference a node having only the @id key." | 「node reference」は照合済み。定義文は Read で目視 |
| association | UML 2.5.1 | ── | 確かめられなかった |

読み取れること：SHACL では、指す先の型は sh:class、数は sh:minCount ・ sh:maxCount、指される側からの数は sh:inversePath と sh:minCount の組で書く。1つの語が x-ref の仕事全体を受け持つことはなく、x-ref の下のキー（`to` ・ `inverse` ・ `min`）に当たる語が並ぶ。JSON-LD の node reference は「@id だけを持つ node object」で、instance を指す。ただし動詞 reference は、`$ref` の定義文（"used to reference a statically identified schema"）と同じ語である。

## 名前から仕事を読み取れるか

○＝原典の語義に入っている　△＝語義に近いが、原典はそこまで言っていない　×＝語義に入っていない

| 候補 | ① 指す先が在る | ② 指される数 | ③ つながりの一覧 | ④ instance を指す | `$ref` と紛れるか |
|---|---|---|---|---|---|
| x-relationship | △ JSON:API は存在を保証しない | ○ Chen の 1:n ・ m:n は relationship set に付く。JSON:API は to-one ・ to-many | ○ Chen の relationship は両側の実体の組 | ○ Chen は実体の組、JSON:API は resource object から resource object へ | 紛れない |
| x-foreign-key | ○ referential integrity そのもの | × 指される側の数は定義に無い | △ referencing ・ referenced の対はあるが、一覧は言っていない | ○ 行の値を指す | 紛れない |
| x-link | × OpenAPI は "does not guarantee" と書く | × | △ 語は「つながり」だが、一覧の出力 `links` と名前が重なる | × OpenAPI は操作を、Hyper-Schema は URI を指す | 紛れないが、JSON Schema 系の `links` と紛れる |
| x-relation | × | × | × | × Chen では表（組の集合）そのもの | 紛れない |
| x-association（UML） | 確かめられなかった | 確かめられなかった | 確かめられなかった | 確かめられなかった | 紛れない |
| x-class（SHACL） | △ 型が合うかの検査 | × 数は別のキー | × | △ value node の型を見る | 紛れないが、`to`（kind）1つ分の意味しか無い |
| x-reference（JSON-LD） | △ | × | △ | ○ node reference | 紛れる。`$ref` の定義文が同じ動詞 reference を使う |

## 結論

| 候補 | 結論 | 理由 |
|---|---|---|
| x-relationship | 採れる | Chen と JSON:API の両方で、データ（実体 ・ resource object）どうしのつながりを指し、数の決まり（1:n ・ to-one ・ to-many）も relationship に付く。①の存在の確認は語義に入らないが、基盤の検査が受け持つので名前と食い違わない |
| x-foreign-key | 条件付きで採れる | ①は語義そのもの。条件は、②の inverse（指される側の数）と③の一覧が foreign key の語義の外にあると説明で補うこと。表と列の語なので、文書の中の項目（`to: self`）を指す場合にも読み手が当てはめ直す必要がある |
| x-link | 採れない | OpenAPI では操作へのつながり、JSON Hyper-Schema では `links` キーワードとして instance にハイパーリンクを付ける別の意味がある。`$ref` と紛れるのを避けたい理由が、JSON Schema 系の `links` との間でそのまま残る |
| x-relation | 採れない | Chen では relation は表（組の集合）を指し、欄が別のものを指すことを指さない |
| x-association | 判断できない | UML 2.5.1 の原文を確かめられなかった |
| x-class | 採れない（キーワード全体の名前として） | sh:class は値の型の条件だけで、x-ref の下の `to` に当たる。数とつながりは別のキー（sh:minCount ・ sh:inversePath）が持つ |
| x-reference | 採れない | `$ref` の定義文と同じ動詞 reference で、今の紛れが残る |

## 確かめられなかったこと

| 対象 | 何が起きたか |
|---|---|
| UML 2.5.1 の association と multiplicity | OMG の PDF（18 MB）は取得できたが、fact-check verify では「大きすぎる」で読めず、Read でもページを開けなかった。原文の引用は無い |
| Chen 1976 の原文の文字列照合 | NUS と LSU の PDF は取得できたが、スキャン画像で文字として読めなかった。引用は Read でページ画像を目視して書き写したもので、文字列の照合はしていない。cardinality という語が論文全体に出てこないかは、p.11–16 と p.19–21 しか読んでいないので言えない |
| SQL 標準（ISO/IEC 9075）の FOREIGN KEY | 有償の規格で取得していない。PostgreSQL の公式ドキュメントで代えた |

## 取得した原典

置き場所は `sources/`。

| 原典 | URL | ファイル |
|---|---|---|
| Chen 1976（ACM TODS 1(1)） | https://www.comp.nus.edu.sg/~lingtw/papers/tods76.chen.pdf | `www.comp.nus.edu.sg_lingtw_papers_tods76.chen.pdf` |
| Chen（5ページ版） | https://www.csc.lsu.edu/~chen/pdf/erd-5-pages.pdf | `www.csc.lsu.edu_chen_pdf_erd-5-pages.pdf`（使っていない） |
| PostgreSQL 5.5 Constraints | https://www.postgresql.org/docs/current/ddl-constraints.html | `www.postgresql.org_docs_current_ddl-constraints.html` |
| JSON:API 1.1 | https://jsonapi.org/format/1.1/ | `jsonapi.org_format_1.1` |
| OpenAPI 3.1.0 | https://spec.openapis.org/oas/v3.1.0.html | `spec.openapis.org_oas_v3.1.0.html` |
| W3C SHACL | https://www.w3.org/TR/shacl/ | `www.w3.org_TR_shacl` |
| JSON-LD 1.1 | https://www.w3.org/TR/json-ld11/ | `www.w3.org_TR_json-ld11` |
| JSON Schema Core 2020-12 | https://json-schema.org/draft/2020-12/json-schema-core | `json-schema.org_draft_2020-12_json-schema-core` |
| JSON Hyper-Schema 2019-09 | https://json-schema.org/draft/2019-09/json-schema-hypermedia | `json-schema.org_draft_2019-09_json-schema-hypermedia` |
| UML 2.5.1 | https://www.omg.org/spec/UML/2.5.1/PDF | `www.omg.org_spec_UML_2.5.1_PDF`（読めなかった） |
