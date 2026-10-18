# 反証：Q7 の答え（r91-urn-q6-reopen ＋ r91-q7-complete を当てた後）

読んだもの：`patches/r91-urn-q6-reopen.json`、`patches/r91-q7-complete.json`、`proposals/q7-semantic-abstract-schema.after.json`（以下「本体」）、`figures/q7-keywords.src.svg`（以下「Q7 の図」）、`figures/q6-abstract.src.svg`、`advisors/adversarial-q7-align.md`（前の反証）、原文 `measured/schema-uri/json-schema.org_draft_2020-12_json-schema-core` の §8・§8.1.1・§8.1.2・§8.2.1・§8.2.3.2・付録 A・付録 C。
details の番号は、パッチを当てた後の Q7 の並び（0 始まり）。
ボード本体（`semantic-contract-core.json`）は大きすぎて読めなかった。パッチが触らない行（Q6 の answer.text、Q7 の残りの grounds）に古い名前や古い見解が残っているかは確かめていない。

## 1. 前提との食い違い

| # | 場所 | 何が食い違うか | どの前提とか | 重さ |
|---|---|---|---|---|
| 1-1 | details[20] ・ [21]、本体 $defs/derive ・ view | x-derive の rules ・ otherwise ・ declared ・ expect と x-view の text ・ cases ・ parts ・ maps ・ label ・ hidden を、「いまの実装の形と同じ項目で足りる」として採っている。項目の名前の出どころが、いまの実装になっている | いまの実装は参考の事実で、名前を引き継がない | 直さないと出せない（項目ごとに、新しい構想の側の理由で名前を選び直す） |
| 1-2 | 本体 x-validators の description ・ every-delegated ・ x-definitions.validator.name | AI の validator に「モデルの名前まで」書かせる。Agents 契約を通さずにモデルを名指しする形になる。作法の3（details[19]）とも食い違う | AI の検証役は Agents 契約を通して呼ぶ | 直さないと出せない |
| 1-3 | answer.text ・ details[8] ・ x-definitions.validator の description | validator が「無ければ unsupported」だけを書いている。validator が答えない ・ 失敗したときの error が無く、details[22] は protocol へ送るだけ | 検証役が無い ・ 答えないときは unsupported か error | 直したほうがよい |
| 1-4 | 本体 rule.by の description ・ Q6 の annotations ・ answer.text は「判定する者」、checks は「判定した者の5つ」 | 同じ値オブジェクトを2つの名前で呼んでいる | 判定した者の5つ | 直したほうがよい |
| 1-5 | Q7 の図 x-relationship ・ x-derive の validator 欄（77 ・ 89行） | validator に Semantic Schema Resolver を挙げている。Resolver は名前を解決する役で、合否を出さない（前の 1-5 の Renderer と同じ型） | rule 1件の合否を出す側は validator | 直したほうがよい |
| 1-6 | 本体 x-definitions（no-core-names を含む） | Concrete が x-term や x-vocabularies を x-definitions に定義できる。no-core-names が止めるのは Core の x- の名前だけで、持たないと決めた語は止めない（前の 3-8 が残る） | x-term ・ x-vocabularies は持たない。term は標準の $ref で指す | 直したほうがよい |
| 1-7 | Q6 details[3] ・ Q6 の図 annotations の行 | x-relationship を「複数のファイルで使う」annotations に置いている。ところが本体で x-relationship を使うのは $defs/field だけで、ほかに使うファイルを挙げていない | 値オブジェクトは持ち主の集約の $defs、複数のファイルで使うものだけ annotations | 直したほうがよい（使うファイルを挙げるか、本体の $defs へ移す） |
| 1-8 | Q7 images/2（glossary の例）の `$schema: urn:semantic-schema-core:glossary:v1` | 本体の declare-version は「Abstract の版の $id（…semantic-abstract-schema:v1）を書く」と定め、Q6 の図も本体を「Concrete が $schema で指す」としている。glossary.schema.json もメタスキーマとして $vocabulary と $dynamicAnchor を持つのかが書かれていない（§8.1.2.2：語彙の宣言は $ref で引き継がれない） | Q6 のファイル構成 | 直したほうがよい |
| 1-9 | term-by-ref の test「閉じた一覧の glossary」と details[12]「Abstract と一緒にデータで出す $id の閉じた一覧」 | Concrete の glossary（urn:zip:glossary:v1）は Abstract と一緒には出さない。別々の2つの一覧を、同じ「閉じた一覧」という語で呼んでいる | glossary は閉じた一覧に事前に登録し、取得しない | 直したほうがよい |
| 1-10 | 本体 view の text ・ label（「JMESPath 式」） | JMESPath という2つ目の規格を持ち込むかどうかを決めていない | 標準は JSON Schema 2020-12 だけ | 直したほうがよい（持ち込むなら前提に足し、持ち込まないなら式の言語を外す） |

## 2. 案の中の矛盾

| # | 場所 | 何が食い違うか | 重さ |
|---|---|---|---|
| 2-1 | 本体 `$vocabulary` の `urn:semantic-schema-core:vocab:v1: true`（details[10] ・ [11]）と checks の view「機械（JSON Schema の実装）」 | §8.1.2-4 は「true の語彙を知らない実装は、このメタスキーマを $schema で宣言したスキーマの処理を MUST refuse」と定める。「機械」が汎用の JSON Schema の実装なら、どの Concrete も処理を断られ、by: machine の rule が1件も動かない | 直さないと出せない（false にするか、「機械」を Core に組み込んだ実装に限ると書く） |
| 2-2 | by: machine の only-for-instances ・ hidden-alone | 本体の JSON Schema に、対応する制約が無い（x-generates と「$defs だけを持つ」の組み合わせも、hidden と他のキーの排他も書いていない）。機械は検査しないまま通すので、合格に見える | 直さないと出せない（details[8]「検証しなかったことを隠さない」に反する。形で書くか、by を core に替える） |
| 2-3 | rule-validator の rule ・ test と、rule.by の description ・ rule-validator の why | rule と test は「意味の判断が要る決まりには by を書く」と言う。一方で by の無い rule は unsupported と返るので、機械 ・ 基盤の rule にも by が要る。rule と test が求める範囲が狭すぎる | 直したほうがよい |
| 2-4 | x-validators（キーは x- の名前でもよい）と x-definitions.validator.name | Concrete の x- の validator の名前を2か所に書く（every-delegated は「validator を持つ x- のそれぞれを x-validators に書く」）。name の説明「外のツールなら x-validators の名前」が、x-validators のキーと値のどちらを指すのかも決まらない | 直したほうがよい（どちらか1か所にする） |
| 2-5 | x-validators のキー ・ $defs/rule の id の pattern | rule の id が文書の中で一意だという決まりが無い（noun などは欄ごとに繰り返せる）。また id の pattern `^[a-z0-9]+(-[a-z0-9]+)*$` は `x-foo` も通すので、x- の名前とキーがぶつかる | 直したほうがよい |
| 2-6 | rule.by = human | x-validators は tool と ai だけを引く。人の rule の結果がどこから返るかが書かれていないので、details[8]「どの rule も検証結果を1件返す」が成り立たない | 直したほうがよい |
| 2-7 | rule.requires「validator に必要な capability」 | x-validators の値は名前の文字列だけで、validator がどの capability を持つかを書く欄が無い。requires と照らし合わせる相手が無い | 直したほうがよい（外せる候補でもある） |
| 2-8 | rule.required | 書かなかったときの既定値が無い。unsupported を全体の成功に含めるかどうかが決まらない | 直したほうがよい |
| 2-9 | details[17] と x-definitions.validator（by と name だけ） | 検証の仕方の1つ「Core の x- の組み合わせ」を書く欄が無い（前の 2-4 の残り） | 直したほうがよい |
| 2-10 | core-keywords-closed ・ no-core-names の test「Core の x- の定義」 | Core の x- は本体の properties ・ $defs/field ・ $defs/shape ・ annotations に散らばっている。not.enum を退けたので一覧が無く、どこを数えるかの決まりも無い | 直したほうがよい（例：本体の properties と field ・ shape の properties にある x- のキーと定める） |
| 2-11 | answer.text「1つずつ…置く場所 ・ 形 ・ 検証の種類 ・ validator を決める」 | 本体の定義は検証の種類も validator も持たない。持っているのは Q7 の図だけ（前の 2-14 が残る） | 直したほうがよい |
| 2-12 | Q7 の図の「検証の種類」列（機械的 ・ 形だけ ・ 両方 ・ 作法だけ）と details[3] ・ [4]、rule.by の5つ | 誰がどう検証するかを、2組の語で書いている。図の「機械的」が machine を指すのか core を指すのかも決まらない（every-delegated と x-relationship の存在の検査は core） | 直したほうがよい |
| 2-13 | Q7 の図 x-prompt の write の validator 欄「rule の by が名指す validator」 | by が表すのは判定する者の種類で、validator を名指さない。名指すのは x-validators | 直したほうがよい |
| 2-14 | Q7 の図 x-view「両方 ・ Validator ・ AI」 | 本体の view の rule は hidden-alone（machine）の1件だけで、AI の rule が無い | 直したほうがよい |
| 2-15 | 本体 view の cases ・ parts ・ maps と derive の when ・ expect | 中身の形が無い（cases は配列であることだけ、expect は items が無い、when は空でないオブジェクトであることだけ）。図が「機械的」とする検査を書けない | 直したほうがよい |
| 2-16 | details[22]「protocol で書く」 | protocol がどのファイル、どの論点を指すのかが無い。10ファイルにも protocol は無い | 直したほうがよい |
| 2-17 | details[18]（作法の2）「場所ごとの5つの値」 | 5つが何かを本体も details も書いていない。判定した者の5つと読み違えやすい | 直したほうがよい |

## 3. 本体の JSON の誤り（JSON Schema 2020-12 として）

| # | 場所 | 何が誤りか | 重さ |
|---|---|---|---|
| 3-1 | 本体の根の `$dynamicAnchor: "meta"` と、allOf による標準メタスキーマの $ref | 標準のメタスキーマは、部分スキーマ（properties の値 ・ $defs の値 ・ items など）に `$dynamicRef: "#meta"` を当てる。§8.2.3.2-4 により、解決先は動的スコープの一番外の "meta"、つまり本体の根になる。その結果、根の required [$schema, title, description]、$id の pattern、properties → fields、if/then が、Concrete のすべての部分スキーマに当たる。§8.1.1-4 は資源の根以外に $schema を書くことを MUST NOT とするので、どの Concrete も合格できない。glossary の例の `$defs/DurationSeconds` も $schema が無いので落ちる | 直さないと出せない（文書の根だけに当てる決まりを、$dynamicAnchor を付けない別の入口へ分ける） |
| 3-2 | 3-1 の帰結（properties → fields → field の required [description, x-prompt]） | fields が入れ子のすべての properties に当たり、葉にも x-prompt を求める。no-prompt-on-leaf と正面から食い違う | 直さないと出せない（3-1 と同じ直しで消える） |
| 3-3 | x-definitions.*.schema の `$ref: https://json-schema.org/draft/2020-12/schema` | 同じ動的スコープの中で評価されるので、x- の値の形にも本体の根（$schema ・ title ・ description）が当たる | 直さないと出せない（3-1 と同じ直しで消える） |
| 3-4 | 本体の根の `type: object` | 3-1 の帰結で、Concrete の中の真偽値のスキーマ（`"data": true` など）をすべて断る（前の 3-7 が好みから上がる） | 直したほうがよい |
| 3-5 | details[12]「標準の 2020-12 のメタスキーマも同梱」 | 標準のメタスキーマは meta/core ・ applicator ・ unevaluated ・ validation ・ meta-data ・ format-annotation ・ content の7つを $ref している。閉じた一覧にこの7つも入れるとは書いていない | 直したほうがよい |
| 3-6 | URN の NID（semantic-schema-core ・ zip ・ Skill の名前）と $id の pattern | RFC 8141 は NID を IANA に登録することを求めるが、どれも登録していない（RFC 8141 の原文は measured に無く、照合していない）。pattern は NID の長さ（2〜32）も見ない。Q6 の rejected には、登録の要らない形（urn:uuid、tag: URI）との比較も無い | 直したほうがよい |
| 3-7 | 本体 properties.$schema の `format: uri` | 宣言している語彙は format-annotation なので、format は検査されない。scheme があるかどうかの検査は、rule（core）に頼ることになる | 好み |
| 3-8 | $vocabulary の書き方（core を true にし、キーは scheme 付きの URI）と、URN の $id ・ `#/$defs/…` ・ `#DurationSeconds` | §8-2 ・ §8.1.2-2 ・ §8.2.2-6 ・ 付録 A に合っている | 無し |

## 4. 前の反証の「直さないと出せない」15件

| # | 解消したか | 1行 |
|---|---|---|
| 1-1 | 解消 | 根の description は「x-relationship と判定する者の形は annotations」になった |
| 1-2 | 解消 | by は machine ・ core ・ tool ・ ai ・ human になった（annotations の judge 本体は案に無く、enum は照合できない） |
| 1-3 | 解消 | rule.by も validator.by も annotations の judge を $ref している |
| 2-1 | 解消 | $vocabulary を足した。ただし required: true が新しい矛盾 2-1 を生んだ |
| 2-2 | 解消 | 語彙を1つにして required を分けない（rejected に理由がある） |
| 2-3 | 解消 | 閉じるのは Validator の rule（core-keywords-closed）だと書いた。Core の x- をどこで数えるかは 2-10 に残る |
| 2-4 | 解消 | validator は任意になり、書かなければ unsupported。「Core の x- の組み合わせ」を書く欄は 2-9 に残る |
| 2-5 | 解消 | 作法の版は $schema で申告すると書いた |
| 2-6 | 解消（別の問題が残る） | 形は書いた。ただし名前をいまの実装から採っており（1-1）、中身の形が無い項目もある（2-15） |
| 2-7 | 解消 | 本体のどの rule にも by がある。rule-validator の文言は狭いまま（2-3） |
| 3-1 | 解消 | $id と $schema は URN になった |
| 3-2 | 解消 | $ref は URN の全体で書いている |
| 3-3 | 解消 | glossary の例の $id は urn:zip:glossary:v1 になった |
| 4-1 | 解消 | checks を差し替え、x-derive を根拠にした確認は無くなった |
| 4-2 | 一部未確認 | checks は解消した。grounds に「x- は合否に入らない」が残っていないかは、ボード本体を読めず確かめていない（外したのは grounds[16] だけ） |

## 5. 外せるもの

| # | 場所 | なぜ外せるか |
|---|---|---|
| 5-1 | 根の declare-version と properties.$schema の closed-list | 同じ検査（$schema が閉じた一覧にあるか）を2か所に書いている |
| 5-2 | name-and-description | 根の `required: [title, description]` がすでに止めている |
| 5-3 | details[20]（作法の4）と declare-version | どちらも「$schema で版を示す」と言っている |
| 5-4 | details[11]（語彙は分けない）と rejected の追加分（語彙を2つに分ける） | 同じことを決まりと退けた案の両方に書いている。rejected だけで足りる |
| 5-5 | rejected の追加分「not enum に並べる」と、前からある rejected[5]「Core の x- の一覧を定義とは別の表で持つ」 | 同じ案を2回退けている |
| 5-6 | details[3] ・ [4]（機械的 ・ 両方の分類）と Q7 の図の「検証の種類」列 | rule.by の5つに1本化できる（2-12） |
| 5-7 | details[1] と Q7 の図の「定義の置き場所」列と Q6 details[7] | 同じ置き場所を3か所に書いている |
| 5-8 | x-validators か x-definitions.validator のどちらか | x- の validator の名前が重なっている（2-4） |
| 5-9 | details[23]〜[25] ・ [27] | x-relationship の名前 ・ description ・ inverse と URN の形は、Q2 と Q6 で決めたことの写し |
| 5-10 | rule.requires | 照らし合わせる相手が無いまま（2-7）なら、無くても困らない |

## 重さの集計

- 直さないと出せない：1-1 ・ 1-2 ・ 2-1 ・ 2-2 ・ 3-1 ・ 3-2 ・ 3-3（3-2 と 3-3 は 3-1 の直しで消える）
- 直したほうがよい：1-3〜1-10 ・ 2-3〜2-17 ・ 3-4〜3-6
- 好み：3-7
