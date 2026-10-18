# x-ref の新しい名前：候補ごとの反証

役割：反証役。どの候補にも肩入れしない。
対象：`.claude/skills/schema-driven/assets/annotations.schema.json` の `x-ref`（`to` ・ `in` ・ `item` ・ `bare` ・ `only` ・ `accept` ・ `when` ・ `label` ・ `unique` ・ `covered_by` ・ `inverse`）。

> 出どころの注記：ほかの技術の語の意味は、反証役の記憶による。原文の文字列とはまだ照合していない。ボードへ取り込む前に fact-check で原典を取得して照合すること。

---

## 1. 置き換えの元にした実際の使い方

| 記号 | 場所 | いまの断片 | 何を指すか |
|---|---|---|---|
| A | board.schema.json `waits_for`（配列） | `"x-ref": {"to": "board"}` | ほかのインスタンス（ボード）の id |
| B | board.schema.json `queue[].topic` ・ `changes[].topic` | `"x-ref": {"to": "self", "in": "topics", "bare": true}` | 同じ文書の中の項目（論点） |
| C | board.schema.json `topics[].depends`（配列） | `"x-ref": {"to": "self", "in": "topics", "bare": true}` | 同じ文書の中の項目（前提の論点）。複数 |
| D | annotations.schema.json の例 | `{"to": "glossary", "in": "terms", "accept": {"at": "meanings/kind", "in": ["集約"]}, "label": "word"}` | ほかのインスタンスの中の項目。種類まで確かめる |
| E | annotations.schema.json の inverse の例 | `"inverse": {"group": "ユースケースを束ねるサブドメインは1つ", "min": 1, "max": 1}` | 指される数の決まり |
| F | annotations.schema.json の to の例 | `"to": ["aggregate", "domain_service"]` | 2種類のどれか |

board.schema.json には inverse ・ accept の使用が無いので、E ・ D ・ F は annotations.schema.json の例から取った。

---

## 2. 候補ごとの置き換えと誤読

### 2.1 x-relationship

```json
"topic": {
  "$ref": "board/glossary@v1#topic-id",
  "x-relationship": {"to": "self", "in": "topics", "bare": true}
}
```

| 観点 | 読めるか | 誤読の場面 |
|---|---|---|
| $ref と並べたとき | △ | 「この欄のスキーマと、`$ref` の先（用語集の topic-id）との関係」と読める。relationship という語は、スキーマどうしの関係かデータどうしの関係かを言わない |
| inverse | △ | ORM では、relationship の inverse は「相手側の属性の名前」である（JPA の `mappedBy`、Doctrine の owning side ／ inverse side、SQLAlchemy の `back_populates`）。`"inverse": {"group": …, "min": 1}` を見た ORM 経験者は、`inverse` に相手側のプロパティ名を書くものと思う |
| 同じ文書の中（`to: self`） | ○ | 自分の中の項目どうしの関係として違和感が無い |
| 存在を確かめるか | △ | 名前からは分からない。ER の relationship も JSON:API の relationships も、相手の存在の検査を名前では約束しない。「関係を描くための宣言」と読まれ、検査が走ることに驚く |
| ほかの技術 | ○ | JSON:API の resource object の `relationships`（`data` に `{type, id}` を持つ resource linkage）が、`to`＋id の形とよく合う。ER（Chen 1976）の relationship は両側の数（cardinality ・ participation）を持ち、inverse の min ・ max と合う。UML の Relationship は Generalization ・ Dependency も含む抽象メタクラスで、範囲が広すぎる |

### 2.2 x-foreign-key

```json
"topic": {
  "$ref": "board/glossary@v1#topic-id",
  "x-foreign-key": {"to": "self", "in": "topics", "bare": true}
}
```

| 観点 | 読めるか | 誤読の場面 |
|---|---|---|
| $ref と並べたとき | ○ | 4候補で最もはっきり分かれる。「値の形は `$ref`、値が別の記録のキーであることは x-foreign-key」と読める |
| inverse | × | 外部キーは参照する側の制約で、「必ず1件以上から指される」のような指される側の数は外部キーの概念に無い。E を `"x-foreign-key": {"to": "use_case", "inverse": {"min": 1, "max": 1}}` と書くと、DB 経験者は inverse を ON DELETE の振る舞いか、逆向きの has_many と読む |
| 同じ文書の中（`to: self`） | △ | 自己参照の外部キー（社員表の manager_id など）は DB では普通だが、「foreign ／ 外部」と `self` が並ぶと、DB を知らない書き手や AI は矛盾と読む |
| 存在を確かめるか | ◎ | 参照整合性そのものの名前で、存在の検査を強く約束する。SQL の `REFERENCES` 句とも、いまの説明文の「参照」とも合う |
| ほかの技術 | × が多い | 名前が SQL の外部キーの前提を持ち込み、x-ref はその前提から4か所で外れる（下の表） |

外部キーの前提から外れる箇所：

| x-ref の機能 | 断片 | 外部キーとして読んだときの誤読 |
|---|---|---|
| `to` が配列（F） | `"x-foreign-key": {"to": ["aggregate", "domain_service"]}` | 外部キーは参照先の表を1つに決める。多態の関連（Rails の polymorphic association）は外部キー制約を張れないことで知られる。複合キーの列の並びと読まれるおそれもある |
| 配列の欄（A ・ C） | `"waits_for": {"type": "array", …, "x-foreign-key": {"to": "board"}}` | 配列の要素ごとの外部キーは SQL 標準に無い（PostgreSQL も配列の要素への外部キーを持たない）。配列そのものがキーと読まれる |
| `item: true`（"インスタンスの id.項目の id" の1文字列） | `"x-foreign-key": {"to": "other_requirements", "item": true, "in": "business_rules"}` | 外部キーの複合キーは列を分ける。1つの文字列を `.` で割る形は外部キーの読み方に無い |
| `only`（参照と組み込みの値が混ざる） | `"x-foreign-key": {"to": "value_object", "only": "^VO-"}` | 「キーでない値も入る外部キー」になる。外部キーで検査を飛ばすのは NULL だけである |
| `accept`（指す先の種類） | D | 外部キーは指す先の行の中身を見ない。CHECK 制約の領分と読まれる |

先例：Frictionless Data の Table Schema が `foreignKeys`（`fields` と `reference: {resource, fields}`、`resource` が空文字なら同じ資源）を持つ。JSON で書くスキーマに外部キーを置く先例ではあるが、表と列の世界の語である。

### 2.3 x-link

```json
"topic": {
  "$ref": "board/glossary@v1#topic-id",
  "x-link": {"to": "self", "in": "topics", "bare": true}
}
```

| 観点 | 読めるか | 誤読の場面 |
|---|---|---|
| $ref と並べたとき | × | `$ref` 自体が「スキーマへのリンク」とよく呼ばれる。さらに同じ JSON Schema の系統に JSON Hyper-Schema の `links`（Link Description Object：`rel` ・ `href` ・ `targetSchema`）がある。`x-link` を見た読み手は、Hyper-Schema の `links` の自前版で、`href` のような URI テンプレートを書くものと思う |
| inverse | ○ | 「被リンク（この項目にリンクしているもの）」の数として読める。4候補で最も自然 |
| 同じ文書の中（`to: self`） | ○ | 文書内リンク（アンカー）として読める |
| 存在を確かめるか | × | リンクは切れていてもよいもの、という読み方が普通である。OpenAPI 3.x の Link Object は「設計時の、ありうるリンク」で、呼べることを保証しないと書かれている。描画や移動のための注釈と読まれ、「ずれ（指す先が無い）」で検査が落ちる理由が名前から読めない |
| ほかの技術 | × | JSON Hyper-Schema `links`、OpenAPI の Response の `links`、JSON:API の `links`（self ・ related の URL）、HTML の `<link>`、RFC 8288 Web Linking。いずれも URI と移動の語で、id の照合ではない。UML では Link は Association の実体（インスタンス側）で、スキーマ側の宣言の名前としては段がずれる |

### 2.4 x-relation

```json
"topic": {
  "$ref": "board/glossary@v1#topic-id",
  "x-relation": {"to": "self", "in": "topics", "bare": true}
}
```

| 観点 | 読めるか | 誤読の場面 |
|---|---|---|
| $ref と並べたとき | △ | x-relationship と同じく、スキーマどうしの関係とも読める |
| inverse | △ | 数学の逆関係（R⁻¹）として読め、「逆向きの関係を定義する欄」と思われる。数の決まりとは読みにくい |
| 同じ文書の中（`to: self`） | ○ | 問題なし |
| 存在を確かめるか | △ | 名前からは分からない |
| ほかの技術 | × | 関係モデル（Codd 1970）では relation は「表（タプルの集合）」そのもので、表どうしのつながりではない。RFC 8288 と Hyper-Schema では relation type（`rel`）は `"author"` ・ `"next"` のような文字列である。`"x-relation": {"to": "board"}` を見た DB 経験者は「board という表」、Web の経験者は「`rel` の値を書く欄なのにオブジェクトが入っている」と読む |

---

## 3. 足して検討した候補（原典に在る語だけ）

| 候補 | 原典 | 合うところ | 誤読の場面 | 結論 |
|---|---|---|---|---|
| x-association | UML 2.5 の Association（両端に multiplicity）。Rails の Active Record Associations（Rails 5 以降、`belongs_to` は既定で相手の存在を検査する） | 両端の多重度が inverse の min ・ max とそのまま合う。スキーマ側の宣言の名前として段が合う | このリポジトリでは `to: "aggregate"`（DDD の集約）を書く。`"x-association": {"to": "aggregate"}` は UML の aggregation（共有集約）と並び、DDD の集約と UML の集約が1行で衝突する | 条件付き。x-relationship より誤読の種が1つ多い |
| x-idref ／ x-keyref | XML 1.0 の ID ・ IDREF（同じ文書の中で、ID の値が在ることを妥当性制約にする）。XML Schema の `xs:key` ・ `xs:keyref`（selector ・ field の path で範囲を決め、存在を検査する） | 意味は x-ref に最も近い。同じ文書の中 ・ 存在の検査 ・ path で範囲を決める、の3つがそろう | 名前に ref が残り、`$ref` と紛れるという今回の問題を解かない | 採れない（目的に合わない） |

---

## 4. 結論

| 候補 | 判定 | 理由 |
|---|---|---|
| x-relationship | 条件付きで採れる | 外の語との衝突が最も少なく、JSON:API の relationships と ER の relationship（両側の数を持つ）に合う。条件は2つ：(1) 名前は存在の検査を約束しないので、description の1文目で「指す先が在ることを検査する」と書く (2) ORM の inverse side（相手側の属性名）と読まれないよう、inverse の description で「指される数の決まり」と言い切る |
| x-foreign-key | 採れない | 存在の検査と `$ref` との区別は最も強いが、名前が SQL の外部キーの前提を持ち込み、x-ref はその前提から外れる（`to` の配列 ・ 配列の欄 ・ `item` の1文字列 ・ `only` ・ `accept` ・ inverse）。`to: self` も「外部」と矛盾して読まれる |
| x-link | 採れない | 同じ JSON Schema ・ OpenAPI の系統に `links`（Hyper-Schema ・ OpenAPI Link Object）が在り、どちらも URI での移動を表し、存在を保証しない。`$ref` との紛れも解けない |
| x-relation | 採れない | 関係モデルでは「表」、RFC 8288 では `rel` の文字列で、どちらにも読まれる。同じ意味なら x-relationship のほうが衝突が少ない |

決め手の誤読（1つずつ）：

| 候補 | 決め手の断片 | 誤読 |
|---|---|---|
| x-relationship | `"x-relationship": {"to": "use_case", "inverse": {"group": "…", "min": 1, "max": 1}}` | ORM の経験者が inverse に相手側のプロパティ名を書こうとする（条件 (2) で防げる） |
| x-foreign-key | `"x-foreign-key": {"to": ["aggregate", "domain_service"]}` | 外部キーは参照先の表を1つに決めるので、DB の経験者は誤りか複合キーと読む |
| x-link | `"$ref": "…#topic-id", "x-link": {"to": "self", …}` | Hyper-Schema の `links` の仲間と読まれ、検査が走らない移動用の注釈と思われる |
| x-relation | `"x-relation": {"to": "board"}` | DB の経験者は「board という表」と読む |
