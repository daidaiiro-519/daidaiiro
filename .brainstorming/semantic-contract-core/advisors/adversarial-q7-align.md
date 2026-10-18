# 反証：Q7 の答え（r91-q7-align-text ＋ r91-q6-q7-consistency を当てた後）

読んだもの：`patches/r91-q7-align-text.json`、`patches/r91-q6-q7-consistency.json`、`proposals/q7-semantic-abstract-schema.after.json`（以下「本体」）、`figures/q7-keywords.src.svg`（以下「Q7 の図」）、参考に `figures/q6-abstract.src.svg` ・ `q1-extension.src.svg` ・ `q1-model.src.svg`。
details の番号は、2つのパッチを当てた後の並び（0 始まり）。
前提は、直された前提を使う：Core の x- は Semantic Abstract Schema の中で1つずつ定義し、x-relationship と複数のファイルで使う値オブジェクト（term を指す欄 ・ 判定する者）は annotations、x-prompt ・ x-derive ・ view ・ rule は本体の $defs に置く。

## 1. 前提との食い違い

| # | 場所 | 何が食い違うか | どの前提とか | 重さ |
|---|---|---|---|---|
| 1-1 | 本体 5行目（根の description） | 「Core 自身の x- の形は annotations が持つ」が残っている | 直された前提（annotations に置くのは x-relationship と値オブジェクトだけ） | 直さないと出せない |
| 1-2 | 本体 $defs/rule の by（419–428行） | enum の `engine` ・ `check` は、いまの実装 schema-driven のツール名。5つの値が判定した者の「機械 ・ 基盤 ・ 外のツール ・ AI ・ 人」と1対1に対応しない（verifier は外のツールか AI か決まらない） | 名前を引き継がない ／ 判定した者の5つ | 直さないと出せない |
| 1-3 | 本体 $defs/rule の by | 「判定する者」は annotations の値オブジェクトなのに、本体が enum をその場で書いている。定義が2か所になる | Q6（判定する者は annotations） | 直さないと出せない |
| 1-4 | 本体 x-definitions の `verifier`（229–233行）・ rule.by の `verifier` ・ rule-validator（70–71行） | x- の検証役を validator と別の語 verifier で呼んでいる。Q1 のモデルは「validator の割り当て：rule と x- ごとの validator の名前」 | rule 1件の合否を出す側は validator | 直したほうがよい |
| 1-5 | Q7 の図 x-view ・ x-states の validator 欄（77 ・ 89行） | validator に Semantic Schema Renderer を挙げている。Renderer は人が読む形にする役で、合否を出さない | 名前の役割（合否を出すのは validator） | 直したほうがよい |
| 1-6 | Q7 の図 x-states の定義の置き場所「catalog」（83行） | Q6 の図は catalog.schema.json の $defs を「なし」とし、本体の行にも x-states が無い。直された前提の本体 $defs の列挙にも無い | Q6 のファイル構成 | 直したほうがよい（Q6 か Q7 のどちらかを直す） |
| 1-7 | details[1]「ほかを本体の $defs に置く」 | x-generates と x-definitions は本体の $defs ではなく本体の properties で定義している（166 ・ 191行）。x-states は図では catalog | 直された前提 ・ 本体の JSON | 直したほうがよい |
| 1-8 | 答えの文「いま検証していないもの」 | いまの実装の状態を起点にしている。新しい Core には「いま」の検証が無い | いまの実装は参考の事実 | 直したほうがよい |
| 1-9 | 答えの文「unsupported と返す」 | Core 自身の x- に unsupported を許す読み方になる | Core 自身の x- は Core が意味の検証まで持つ | 直したほうがよい（Concrete が足した x- に限ると書く） |
| 1-10 | details[6]「名前の出どころは x-view の label の1か所」 | term は名前を title に持ち、本体も根に title を求める。名前の出どころが3か所ある | term は名前を title に持つ | 直したほうがよい |
| 1-11 | details[15]「AI と人の rule」 | 作法の3は「AI に任せるときも Concrete が検証役を選び Agents 契約を通す」。人を検証役にする道が作法に無い | 作法の3 | 直したほうがよい |
| 1-12 | Q6 の annotations「term を指す欄」 | 欄から term を指すのは標準の $ref。本体はこの値オブジェクトをどこからも使っていない | x-term を持たない ・ 標準の $ref | 直したほうがよい（Q6 側。使い道を書くか外す） |

## 2. 案の中の矛盾

| # | 場所 | 何が食い違うか | 重さ |
|---|---|---|---|
| 2-1 | details[8] ・ [9] ・ [11] と本体 | details は「$vocabulary で Core の語彙と標準の7つを宣言」「語彙を2つに分ける」「版を語彙の URI に入れる」と書くが、本体に `$vocabulary` が無い | 直さないと出せない |
| 2-2 | details[9] と前提 ・ details[3] ・ Q7 の図 | x-relationship を `required: false`（知らない実装は無視してよい）にしているが、指す先が無ければ基盤が violated にする（合否を変える）。x-prompt の write も details[7] で検証結果を返すので合否に入るのに、どちらの語彙か書いていない | 直さないと出せない |
| 2-3 | details[12]「本体で閉じる」と本体 | field ・ shape ・ 根のどれも x- のキーを閉じていない（propertyNames も unevaluatedProperties も無い）。未定義の x-foo は本体の検査を通る。閉じているのは x-definitions の名前だけ | 直さないと出せない（閉じるのは Validator の rule だと書くか、形で閉じる） |
| 2-4 | details[15] と本体 x-definitions | details は検証の仕方を「Core の x- の組み合わせ ・ AI と人の rule ・ 外のツール。書けなければ unsupported」とするが、本体は `verifier` を required の文字列1つにしている。組み合わせや rule を書く欄が無く、verifier の無い定義は unsupported でなく形の違反になる（q1-extension は「検証役が無い → unsupported か error」） | 直さないと出せない |
| 2-5 | 作法の4（details[21]）と本体 | 「Concrete は従う作法の版を申告する」の欄が本体に無い | 直さないと出せない |
| 2-6 | details[22]「この論点で詰めること」・ 本体 $defs/derive ・ view の description（342 ・ 346行）「この論点で詰める」 | Q7 の答えなのに、作法の1〜4の形 ・ x-derive ・ view の形を「この論点で詰める」と残している。derive ・ view は `type: object` だけ | 直さないと出せない（詰めるか、送り先の論点を書く） |
| 2-7 | 本体の根の x-prompt の rule-validator と、noun ・ what-and-not ・ placeholder ・ context-kind-version | rule-validator は「意味の判断が要る決まりには by を書く」と言うが、本体自身の意味の判断が要る rule（題が名詞か ・ 書かないものがあるか など）に by が無い。rule-validator 自身の test も意味の判断が要るのに by が無い。by が無い rule を誰が検証するかの既定も無いので、details[7]「どの rule も検証結果を1件返す」が成り立たない | 直さないと出せない |
| 2-8 | rule.by の `verifier` と details[20]（作法の3） | x-prompt の write の rule が `by: verifier` のとき、どの x- の定義の検証役を呼ぶかを指す欄が無い | 直したほうがよい |
| 2-9 | details[2]「足すものはどれも description と x-prompt を持つ」と本体 $defs/derive ・ view | 新しく足した derive ・ view に x-prompt が無い。葉の例外は by ・ requires ・ required にしか書いていない | 直したほうがよい |
| 2-10 | Q7 の図 x-derive の「Concrete で書く場所：オブジェクト」と本体 field（332行） | 本体は x-derive を欄（field）にだけ置いている | 直したほうがよい |
| 2-11 | Q7 の図 x-view の「欄 ・ $defs」と本体 | 本体は x-view を field にだけ型付けし、shape（$defs の形）には置いていない | 直したほうがよい |
| 2-12 | Q7 の図 x-view「hidden を単独にする」 | $defs/view に hidden が無く、何を検証するかが読めない | 直したほうがよい |
| 2-13 | Q7 の図 rule の by「機械的 ・ validator の名前があるか」と rule-validator の test | 図は機械の検査とするが、by が要るかどうかは「意味の判断が要るか」で決まり、機械では決まらない | 直したほうがよい |
| 2-14 | 答えの文「1つごとに置く場所 ・ 形 ・ 検証の種類 ・ validator を持たせる」と本体 | 本体の定義（derive ・ view ・ x-definitions 以外の Core の x-）は検証の種類も validator も持たない。持っているのは Q7 の図だけ | 直したほうがよい（図の表を定義へ移すか、文を「図が示す」に直す） |
| 2-15 | rejected[5]「Core の x- の一覧を定義とは別の表として持つ」を退けた理由と、本体 x-definitions の `not.enum`（197–205行）・ details[12] の列挙 ・ Q7 の図 | 退けたはずの「定義と別の一覧」を本体の中に enum で持っている。Core の x- を足すと enum も直す必要があり、2か所が食い違う崩れを繰り返す | 直したほうがよい（残すなら、rejected の理由を書き直す） |
| 2-16 | Q7 の図 | x-definitions（Core の x- で not.enum にも入る）の行が無い | 直したほうがよい |
| 2-17 | details[14]「範囲は Concrete の $id で決まり」と本体 | x-definitions のキーは素の `x-foo` で、$id の範囲を表す欄が無い。ほかの Concrete を $ref したとき、どちらの x-foo の定義かが決まらない | 直したほうがよい |

## 3. 本体の JSON の誤り（JSON Schema 2020-12 として）

| # | 場所 | 何が誤りか | 重さ |
|---|---|---|---|
| 3-1 | 本体 3行目の $id ・ 85行目の enum ・ Concrete の $schema の値 | 2020-12 の Core 仕様 §8.1.1 は `$schema` の値を「scheme を含む URI」と定める。`semantic-schema-core/semantic-abstract-schema@v1` は scheme の無い相対参照なので、Concrete の `$schema` に書けない。`$vocabulary` のキーも URI が要る | 直さないと出せない |
| 3-2 | 本体 330行目 `"$ref": "semantic-schema-core/annotations@v1#/$defs/relationship"` | $id が相対なので、RFC 3986 の解決で `semantic-schema-core/semantic-schema-core/annotations@v1` になる（最後のセグメントを外して足す） | 直さないと出せない |
| 3-3 | glossary の例（images/2）の `$ref zip/glossary@v1#DurationSeconds` | 3-2 と同じ。Concrete の $id が `zip/…@v1` なら `zip/zip/glossary@v1` に解決される | 直さないと出せない |
| 3-4 | 本体全体 | 標準の 2020-12 メタスキーマを `allOf` の `$ref`（と `$dynamicAnchor: "meta"`）で取り込んでいない。Concrete の標準キーワード（`type: 5` など）を検査しない | 直したほうがよい |
| 3-5 | details[10]（閉じた一覧に無い $schema は断る）と本体2行目 | 本体自身の `$schema` は `https://json-schema.org/draft/2020-12/schema`。これを閉じた一覧に入れて同梱すると書いていないので、本体自身が断られる | 直したほうがよい |
| 3-6 | 本体 x-definitions の `schema`（218–221行） | `type: object` だけで、値が JSON Schema かを検査しない（標準のメタスキーマを $ref していない）。真偽値のスキーマも受けない | 直したほうがよい |
| 3-7 | 本体 $defs/field ・ shape の `type: object` | JSON Schema では properties の値に `true` ・ `false` も書けるが、それを断る | 好み（断るなら rule に書く） |
| 3-8 | 本体 x-definitions の `propertyNames` の `not.enum` | 2020-12 として正しい。Core の x- の漏れも無い（x-prompt ・ x-view ・ x-relationship ・ x-derive ・ x-generates ・ x-states ・ x-definitions）。持たないと決めた x-term ・ x-vocabularies ・ x-glossaries を Concrete が足せてしまう点は残る | 直したほうがよい（Q2 で退けた語を Concrete が足すと、$ref による term の指し方を迂回できる） |
| 3-9 | 本体 `properties.$schema` の enum（84–86行） | 閉じた一覧（データで出す $id の一覧）の写しを enum で持つ。版を足すと2か所を直す | 直したほうがよい |
| 3-10 | $ref と隣のキーワード | 前の版の field にあった `$ref` と `required` ・ `properties` の並びは無くなった。いまの $ref の書き方に 2020-12 上の問題は無い | 無し |

## 4. 確認（checks）の見解が成り立つか

| # | 場所 | 何が食い違うか | 重さ |
|---|---|---|---|
| 4-1 | checks[0] の why「x-derive は合否を変えるので…」 | 直された前提で x-derive は本体の $defs に移り、annotations に無い。why は、annotations に残った x-relationship（合否を変える）で書き直す必要がある | 直さないと出せない |
| 4-2 | checks[0] の view と grounds の追加分「x- は…合否に入らない」 | annotations に残る x-relationship は、指す先が無ければ基盤が violated にするので合否に入る。根拠が見解と逆を支える。annotations に置く「判定する者」「term を指す欄」はキーワードでなく値オブジェクトで、「注釈として集められる語」に当たらない。見解は成り立たない | 直さないと出せない |
| 4-3 | grounds の根拠の文 | 2020-12 は、知らないキーワードを注釈として集めることを SHOULD とする（必ずではない）。「集められるだけ」と言い切っている | 直したほうがよい |

## 5. 外せるもの

| # | 場所 | なぜ外せるか | 重さ |
|---|---|---|---|
| 5-1 | details[13] と details[16] | どちらも「Concrete の x- は Concrete の中の定義があれば断らない」で、同じことを言っている | 好み |
| 5-2 | details[14] と details[18]（作法の1） | どちらも定義の中身（形 ・ 意味 ・ 検証役 ・ 名前の範囲）を並べている | 直したほうがよい（作法の1に1本化する） |
| 5-3 | details[17]（責任） | 決まった前提と q1-extension の「責任」をそのまま写している | 好み |
| 5-4 | details[23] ・ [26] | x-relationship の名前と、x-term ・ x-vocabularies を持たないことは決着済みの前提（Q2）。Q7 の決めることではない | 好み |
| 5-5 | details[0] と details[2] | 足すもの（by ・ requires ・ required）を2行で繰り返している | 好み |
| 5-6 | details[9]（語彙を2つに分ける） | `required` は「知らない実装は処理を断る」かどうかを決める。閉じた一覧で Core の実装だけが読むので、分けても振る舞いが変わらない。2-2 の矛盾ごと外せる | 直したほうがよい |
| 5-7 | 本体 x-definitions の x-prompt の no-core-names | 同じことを `propertyNames.not.enum` が形で止めている（「機械で決まる rule は形へ」に当たる） | 好み |
| 5-8 | 本体 rule.required の名前 | JSON Schema の `required` と details[9] の $vocabulary の `required` とで、同じ語が3つの意味になる | 好み |

## 重さの集計

- 直さないと出せない：1-1 ・ 1-2 ・ 1-3 ・ 2-1 ・ 2-2 ・ 2-3 ・ 2-4 ・ 2-5 ・ 2-6 ・ 2-7 ・ 3-1 ・ 3-2 ・ 3-3 ・ 4-1 ・ 4-2
- 直したほうがよい：1-4〜1-12 ・ 2-8〜2-17 ・ 3-4〜3-6 ・ 3-8 ・ 3-9 ・ 4-3 ・ 5-2 ・ 5-6
- 好み：3-7 ・ 5-1 ・ 5-3 ・ 5-4 ・ 5-5 ・ 5-7 ・ 5-8
