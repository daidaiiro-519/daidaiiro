# v0.3 の形で、ボードの決まりを書き直して比べる実験

ブレストボード semantic-contract-core の Q0（v0.3 で何ができるようになるか）と Q1（基盤の次の形か、別の層か）の根拠にするために、ボード（`brainstorming/board@v1`）に課している決まりを実物から数え、v0.3 の形で書き直して、いまの基盤と比べた。

**この実験の JSON（`v03/` の下）は、v0.3 の `examples/` の形にならった試案である。正式な仕様ではない。**

## 結論

- ボードの決まりは、出どころごとに数えて **397件** ある。形（JSON Schema のキーワード）が259件、それ以外が138件である。
- 形以外の138件のうち **62件はどこでも検査されていない**。内訳は、機械で見られる37件、意味の判断を必要とする12件、インスタンスの外の事実に関わる13件である。
- 検査されていない決まりは、実物のボードで破れていた。2枚のボードとも論点が6件あり、x-prompt の「3〜5件」を外れている。agents-in-base には、2文の caption が3件ある。どちらのボードも、inspect と check の指摘は0件だった。
- 中身の無い22件（「説明のとおりに書く」）を除いた116件を v0.3 の形に分けると、次のとおりである。
  - (a) 宣言にでき、汎用の Interpreter で検査できるもの：54件。いまも宣言で検査されているもの18件を除くと、新しく宣言にできるのは36件である。
  - (b) 宣言にはできるが、検査に専用の Interpreter（Rust ・ AI ・ 人）を必要とするもの：35件
  - (c) v0.3 の試案の形に置き場所が無いもの：27件
- (a) の36件のうち16件は、v0.3 を待たずに、**いまの基盤のまま宣言にできた**。標準の JSON Schema のキーワードで書けたものが11件、x-ref で書けたものが3件、すでに x-ref が検査しているものが2件である。inspect.rs の検査13件のうち9件が、この16件に入る。
- 実物で確かめた範囲では、v0.3 が加えるのは「表せる範囲」ではなかった。加えるのは「検査しなかった決まりを `unverified` として報告すること」である。いまの基盤は、知らない注釈を申告さえすれば受け付け、検査しないまま何も報告しない（keyword-trial）。
- x-prompt の文で書いた決まり105件のうち、AI か人に意味を判断させないと検査できないものは10件である。SKILL.md のガードレールを足すと12件になる。
- JSON Schema 2020-12 の独自のキーワードで書く場合と、v0.3 の自前のモデルで書く場合とで、**表せる範囲に差が出た決まりは0件**だった。差が出たのは、検査しなかったことの報告のしかたである。

## 手順

リポジトリの根から、次の順に実行する。どのスクリプトも、この実験のフォルダの外には書かない。

| 順 | 実行するもの | すること | 書くもの |
|---|---|---|---|
| 1 | `python3 -I <このフォルダ>/measure_unchecked.py .brainstorming/ai-driven-process.json .brainstorming/agents-in-base.json > <このフォルダ>/measure_unchecked.out.json` | 検査されていない決まりのうち機械で数えられる12個を、実物のボード2枚に適用する。文の切り方は `text.rs` の `sentences` を Python に写したもの | `measure_unchecked.out.json` |
| 2 | `python3 -I <このフォルダ>/make_trials.py` | いまの基盤で、決まりを宣言へ移せるかを試す（xref-trial ・ jsonschema-trial ・ keyword-trial）。写したスキーマを、Skill の名前のフォルダに置いて `--skill` で渡す | `xref-trial/` ・ `jsonschema-trial/` ・ `keyword-trial/` ・ `trials.out.json` |
| 3 | `python3 -I <このフォルダ>/count.py` | スキーマの形と x-prompt の rule を機械で拾い、コードと SKILL.md の決まりと合わせて、1件ずつ分類する | `inventory.json` ・ `inventory.md` |
| 4 | `python3 -I <このフォルダ>/build_v03.py` | 116件を v0.3 の形で書き直す。どの決まりも1か所にだけ置かれたことを確かめてから書く | `v03/*.trial.json` |

外の情報源は、JSON Schema 2020-12 の Core の仕様だけを使った。原文は `sources/` に取得し、引用は原文の文字列で照合した（fact-check の verify で4件とも一致）。

### 数え方

| 出どころ | 1件と数える単位 | 拾い方 |
|---|---|---|
| `board.schema.json` の形 | 検証のキーワード1つ。`required` は名前1つを1件、`additionalProperties` は `false` のときだけ、x-ref は1つを1件 | count.py が JSON を歩いて拾う |
| `board.schema.json` の x-prompt | `write` の要素1つ（rule と test の組） | count.py が拾う。行番号は `"rule":` の行 |
| `inspect.rs` | 指摘を出す条件1つ | 読んで書き出した |
| `status.rs` | ステータスを決める分かれ道1つ | 読んで書き出した |
| `lock.rs` | 決まりの定数と判断1つ | 読んで書き出した |
| `SKILL.md` | ガードレールの1行 | 読んで書き出した |

同じ決まりが、別の出どころにも書かれていることがある。件数は**出どころごとの記述の数**であり、重なりを除いた数ではない。重なりが分かったものは、表の「同じ決まり」の列に書いた。

### 分類の記号

**いまの検査**（どこで検査されているか）

| 記号 | 意味 |
|---|---|
| S | JSON Schema で検査される（update と check） |
| R | x-ref を check が検査する |
| I | inspect.rs が検査する |
| W | 書く側 ・ 動かす側のコード（advance ・ render ・ serve ・ ロック）が守る。検査ではない |
| M | 検査されていない。機械で見られる |
| A | 検査されていない。意味の判断を必要とする |
| O | 検査されていない。インスタンスの外の事実（ファイル ・ 実行 ・ 誰が書いたか）に関わる |
| D | 決まりの中身が無い（「説明のとおりに書く」） |

x-prompt の決まりは、その要素の `test`（どう確かめるか）が何を見ると書いているかで分けた。

**v0.3**（v0.3 の形にしたとき）

| 記号 | 意味 |
|---|---|
| a | 宣言にでき、汎用の Interpreter で検査できる |
| b | 宣言にできるが、検査はこの決まり専用の Interpreter（Rust ・ AI ・ 人）を必要とする。宣言は語の名前と引数だけになる |
| c | v0.3 の試案の形に置き場所が無い。1件のインスタンスの外 ・ 時間 ・ 誰が書いたか ・ 値を導く決まりがこれに当たる |
| - | 数えない（中身が無い） |

c は「v0.3 の `examples/` に置き場所が無い」という判定である。v0.3 の構想で表せないという判定ではない。docs/14 は、時間の制約も「独自Contract上は**そのまま表現できる**ことを目指す（形式は未決）」としている。

## 表1 出どころごとの件数と、いまの検査

| 出どころ | 件数 | S | R | I | W | M | A | O | D |
|---|---|---|---|---|---|---|---|---|---|
| 形（JSON Schema のキーワードと x-ref） | 259 | 255 | 4 | | | | | | |
| x-prompt の rule | 105 | 15 | 2 | 5 | 4 | 37 | 10 | 10 | 22 |
| inspect.rs | 13 | | | 13 | | | | | |
| status.rs | 5 | | | | 5 | | | | |
| lock.rs | 4 | | | | 4 | | | | |
| SKILL.md のガードレール | 11 | 2 | | | 4 | | 2 | 3 | |
| **計** | **397** | 272 | 6 | 18 | 17 | 37 | 12 | 13 | 22 |

形の259件の内訳は、type 99 ・ required 56 ・ minLength 43 ・ additionalProperties 20 ・ pattern 9 ・ const 8 ・ enum 6 ・ x-ref 4 ・ minimum 4 ・ uniqueItems 3 ・ minItems 2 ・ maximum 2 ・ maxItems 1 ・ oneOf 1 ・ contentMediaType 1 である。1件ずつの行番号は `inventory.json` にある。

表し方で並べ直すと、次のとおりになる。

| いま表している所 | 件数 | そのうち検査されていないもの |
|---|---|---|
| JSON Schema の形 | 255 | 0 |
| x-ref（独自の注釈） | 4 | 0 |
| x-prompt の文 | 105 | 57（M 37 ・ A 10 ・ O 10） |
| Rust のコード（inspect.rs ・ status.rs ・ lock.rs） | 22 | 0（status.rs と lock.rs の9件は、検査ではなくコードがその振る舞いをする） |
| SKILL.md の文 | 11 | 5（A 2 ・ O 3） |

## 表2 v0.3 の形にしたときの分類

中身の無い22件を除いた116件。

| いまの検査 | a | b | c | 計 |
|---|---|---|---|---|
| S（形で検査済み。x-prompt か SKILL.md が同じことを文でも言う） | 17 | | | 17 |
| R（x-ref） | 1 | | 1 | 2 |
| I（inspect.rs） | 14 | 4 | | 18 |
| W（コードが守る） | 1 | 2 | 14 | 17 |
| M（検査されていない。機械で見られる） | 20 | 17 | | 37 |
| A（検査されていない。意味の判断） | | 12 | | 12 |
| O（検査されていない。インスタンスの外） | 1 | | 12 | 13 |
| **計** | **54** | **35** | **27** | **116** |

### (a) 宣言として外に出せるようになる決まり

54件のうち、S と R の18件はいまも宣言である。**新しく宣言にできるのは36件**で、内訳は I 14 ・ M 20 ・ W 1 ・ O 1 である。

この36件を、いまの基盤で試した結果は次のとおりである。

| いまの基盤での扱い | 件数 | 決まり |
|---|---|---|
| 標準のキーワードで書けた（jsonschema-trial で検査が効いた） | 11 | P2 ・ P48（rejected が1件以上）・ P6（題は40字まで）・ P26（論点は3〜5件）・ P36 ・ I12（open の論点に完成イメージ）・ P78 ・ I11（code の言語が text でない）・ I1（open は5件まで）・ I2（open があれば queue がある）・ I5（ボードに status を書かない） |
| x-ref の when と accept で書けた（xref-trial で検査が効いた） | 3 | P15 ・ I4（queue が指す論点は open）・ I8（open の論点の前提は決着） |
| すでに x-ref が検査している | 2 | I3 ・ I7（指す先がある） |
| 試していない | 20 | 「60行くらいまで」11件 ・ 日付の新しい順（P23）・ 論点の id が重ならない（P28）・ 名前は10字くらい（P29）・ ステータスと queue の対応（P30）・ 言語の名前（P77 ・ P87）・ open が無ければ全部決着（I6）・ ロックの形（L4）・ AI の「検査した」で合格としない（G7） |

I5 は、根の `additionalProperties: false` がすでに断る（xref-trial の `board-status.json`）。inspect.rs の同じ検査は、形の検査と重なっている。

試していない20件のうち、標準のキーワードで書けない見込みのものは、並びの順（P23）と、キーでの重なり（P28）である。`uniqueItems` は要素全体を比べるので、id だけの重なりは見られない。

### (b) それでも検査に Interpreter（Rust ・ AI ・ 人）を必要とする決まり

35件。いま検査されていないもの（M 17 ・ A 12）が29件ある。

| 必要とする Interpreter | 件数 | 決まり |
|---|---|---|
| 日本語の文の切り方（1文か ・ 1文の長さ） | 17 | caption ・ note ・ intro が1文（15件）・ 1文は120字まで（I10 ・ P32） |
| AI（意味の判断） | 11 | 事実を聞かない ・ 選ばせない ・ 1つのことを決める ・ 問いに答えている ・ 1件に1つのこと ・ ファイルとして成り立つ ・ 強調は十分少ない ・ 優先の印は少数 など |
| 人（描いて目で見る） | 1 | 画面の高さが中身に合う（P72） |
| 本文から論点の id を拾う | 1 | I9 |
| 逆向きの参照と文字列の「または」 | 1 | I13 |
| tree の深さの前後 | 1 | P94 |
| 描いた結果 ・ SVG ・ HTML を読む | 3 | P52 ・ P67 ・ P71 |

1文の判定は、いまの inspect.rs が文の長さ（SentenceAtMost）しか持たない。「1文か」（OneSentence）を見る Interpreter はどこにも無い。

### (c) v0.3 でも表しにくい決まり

27件。v0.3 の要求の例（`interpreter-request.example.json`）が渡すのは、Vocabulary ・ Contract ・ インスタンス1件だけである。

| 理由 | 件数 | 決まり |
|---|---|---|
| インスタンスの外のファイルを必要とする | 8 | id とファイル名（P5）・ waits_for のほかのボード（P12）・ path が在るか（P76 ・ P86 ・ P95）・ 原文と同じか（P88 ・ P89）・ cwd が在るか（P102） |
| 前の回の記録を必要とする | 3 | round（P13）・ changes（P18）・ 外した案を消さない（G3） |
| 誰が書いたかを必要とする | 3 | 保留は AI が書かない（P8）・ 手で書き換えない（G4）・ rounds/ と answers/ を書き換えない（G5） |
| 実行と時間 | 9 | 実行できるか（P103）・ 出力と同じか（P104）・ ロック（L1〜L3）・ 更新中（D2）・ フォルダの外（G6）・ ロックを残さない（G9）・ Kiro は読むだけ（G10） |
| 値を導く決まり | 4 | 論点のステータス（D1）・ 保留（D3）・ 決着（D4）・ 回答待ち（D5） |

このうち2件は、**いまの基盤のほうが v0.3 の試案より多くを表せる**。

- waits_for（P12）：いまの check はディレクトリの全部のボードを読み、x-ref でほかのボードを指せる。v0.3 の要求の例は、インスタンスを1件しか渡さない。
- 論点のステータス（D1）：いまの基盤の x-derive（`annotations.schema.json:745`）は、`rules` と `otherwise` で値を導く決まりを宣言できる。v0.3 の試案の operation は `validate` だけが例にある。

### AI に意味を見させる決まり

x-prompt の105件は、どれも AI が書くときに読む文である。そのうち、test を確かめるのに意味の判断を必要とするものは **10件**（board.schema.json の 14 ・ 197 ・ 544 ・ 626 ・ 746 ・ 767 ・ 1143 ・ 1447 ・ 1574 ・ 1596 行）である。1447行だけは「描いて目で見る」で、人か、画像を読める AI が見る。SKILL.md のガードレールの2件（119 ・ 120 行）を足すと12件になる。12件とも、いまは検査として呼ばれていない。書く AI が自分で読むだけである。

残りの95件のうち、37件は機械で見られるのに検査されていない。22件は中身が無い。

## 試案の JSON

| ファイル | 中身 |
|---|---|
| `v03/vocabulary.text-ja.trial.json` | 日本語の文の語（OneSentence ・ SentenceAtMost ・ CharsAtMost ・ LinesAtMost） |
| `v03/vocabulary.constraint-core.trial.json` | どのボードにも依らない制約の語（Count ・ Implies ・ RefersTo ・ UniqueKey ・ OrderedBy ・ NotIn ・ InList） |
| `v03/vocabulary.brainstorming.trial.json` | このボードにしか無い語（TopicId ・ TopicStatus ・ MentionedTopicsDeclared など） |
| `v03/vocabulary.judgment.trial.json` | AI と人に判断させる語（Judgment ・ VisualJudgment）。宣言は問いの文だけになる |
| `v03/concrete.board.trial.json` | ボードの Concrete Contract。dataSchema はいまのスキーマを名前で引く。116件を、dataSchema（21）・ constraints（34個の制約に66件）・ プロトコル（1）・ unplaced（27）・ ロックの形（1）に分けた |
| `v03/concrete.lock.trial.json` | ロックの形（L4）。いまはスキーマが無く、`lock.rs:190` の中にだけある |
| `v03/interpreters.trial.json` | Interpreter と Capability。この実験のときに動くものがあるかを `present` に書いた |
| `v03/interpreter-request.trial.json` ・ `v03/interpreter-response.trial.json` | 実物のボード ai-driven-process への要求と応答 |
| `v03/projection-report.trial.json` | 標準の JSON Schema 2020-12 への射影。34個の制約のうち8個を表せる。7個は動かし、1個（I6）は試していない |

応答は、この実験で実物のボード（またはその写し）に実際に動かした結果だけで作った。動かしていない制約は `unverified` にした。

| 状態 | 件数 | 中身 |
|---|---|---|
| verified | 11 | inspect の3つ ・ x-ref の2つ ・ 標準のキーワードへ射影した6つ |
| violated | 1 | 論点は3〜5件（実物は6件） |
| unverified | 22 | AI の判断9 ・ 人の判断1 ・ Interpreter が無いもの10 ・ 言語の名前の一覧が無いもの1 ・ この実験で render を動かしていないもの1 |

同じボードに、いまの inspect と check は指摘0件を返す。応答の全体の状態は `violated` になる。violated が無くても、必須の judge-meaning が無いので `verified` にはならない。

## 独自のキーワードと v0.3 の自前のモデルで、表せる範囲に差が出た決まり

**0件**だった。116件のどれも、片方で表せて片方で表せない、ということは無かった。c の27件は、どちらの形でも同じように表しにくい。入力がインスタンスの外にあるからである。

差が出なかった理由は、JSON Schema 2020-12 の Core の仕様に3つある（原文は `sources/` に取得した）。

| 仕様の箇所 | 原文 | この実験での意味 |
|---|---|---|
| 7.9 Loading Instance Data | "Keywords MAY be defined to use JSON Pointers or Relative JSON Pointers to examine parts of an instance outside the current evaluation location." | 独自のキーワードは、いまの場所の外（ボード全体）を見る決まりも持てる。I1 ・ I2 ・ I6 のような全体の決まりも置ける |
| 8.1.2 The "$vocabulary" Keyword | "If the value is true, then implementations that do not recognize the vocabulary MUST refuse to process any schemas that declare this meta-schema with "$schema"." | 知らない語彙を必須にすれば、知らない実装は処理を断る。v0.3 の fail-closed に近いことが、語彙の単位でできる |
| 8.1.2 | "unrecognized keywords SHOULD be treated as annotations." | 語彙を必須にしなければ、知らないキーワードは注釈として読み流される |

独自のキーワードの値は任意の JSON なので、v0.3 の Constraint に書ける JSON は、そのまま独自のキーワードの値にも書ける。いまの基盤には、そのための口がすでにある。`meta-schema.json:195` の `x-annotations` に名前と仕様のファイルを申告すれば、check-schemas は通る（keyword-trial）。

差が出たのは、表せる範囲ではなく次の3つである。

| 違い | いまの基盤（独自のキーワード） | v0.3 の自前のモデル |
|---|---|---|
| 検査しなかった決まりの報告 | 申告した独自の注釈を、検査せずに黙って通す。keyword-trial で、40字の決まりを `x-constraints` に書いた50字の題は、指摘0件だった。基盤の abstract スキーマは `$vocabulary` を使っていない | 制約ごとに `unverified` と、足りない Capability を返す |
| 断る単位 | `$vocabulary` を使っても、語彙の単位でスキーマ全体の処理を断るだけ | 制約1つの単位で、検査できたものとできなかったものを分けて返す |
| 語の使い回し | x-prompt は節点ごとに文を写している（「60行くらいまで」11件 ・ 「1文で書く」7件 ・ 「種類の名前をそのまま」6件）。`$ref` で共有する形は、仕様では書けるが使っていない | Vocabulary の Term を、Contract の外に1つだけ置いて指す |

3つとも、JSON Schema の上で独自のキーワードとして足せないものではない。ただし、いまの基盤は持っていない。

## 分かったこと

1. **書いただけの決まりは守られていない。** 検査されていない62件のうち、数えられる12個を実物に適用すると、2つが破れていた（論点の件数 ・ caption の文の数）。inspect と check はどちらも0件だった。
2. **inspect.rs の13件のうち9件は、いまの基盤の宣言で書ける。** 標準のキーワードで5件（I1 ・ I2 ・ I5 ・ I11 ・ I12）、x-ref で4件（I3 ・ I4 ・ I7 ・ I8）。I5 はすでに形の検査と重なっている。Rust に残る4件は、日本語の文（I10）・ 本文の id（I9）・ 「または」の組み合わせ（I13）と、試していない I6 である。
3. **v0.3 にしても、検査の数は増えない。** b の35件は、宣言が語の名前になるだけで、検査の中身は専用の Interpreter が持つ。そのうち、いま動く Interpreter があるのは5件（I9 ・ I10 ・ P32 ・ I13 ・ P67）である。
4. **v0.3 で増えるのは、検査しなかったことが見えることである。** 同じボードに、いまのツールは指摘0件を返し、v0.3 の形の応答は「1件違反 ・ 22件未検証」を返す。
5. **c の27件は、v0.3 の試案では置けない。** 理由は、どれも1件のインスタンスの外の事実を必要とすることである。そのうち2件（waits_for ・ 論点のステータスの導き方）は、いまの基盤のほうが表せる。
6. **表せる範囲の差は、独自のキーワードと自前のモデルのあいだに見つからなかった。** 差は報告のしかたと語の使い回しにあり、どちらも JSON Schema の上に足せる種類のものである。

Q0 と Q1 への根拠としては、次のことが言える。

| 論点 | この実験から言えること |
|---|---|
| Q0 v0.3 で何ができるようになるか | 制約ごとに、検査できなかったことを `unverified` と足りない Capability で返せるようになる。新しく宣言にできる36件のうち16件は、v0.3 を待たずにいまの基盤で宣言にできた |
| Q1 基盤の次の形か、別の層か | v0.3 で増えるもの（制約ごとの未検証の報告 ・ Capability ・ 共有の語）は、どれも JSON Schema の独自のキーワードとエンジンの応答として、いまの基盤に足せる種類のものだった。逆に、いまの基盤にあって v0.3 の試案に無いもの（ディレクトリをまたぐ参照 ・ 導く決まり）が2件あった |

## 言えないこと

- **分類は書き手の判断である。** a と b の境（「汎用」の線）、approx（「くらい」）の扱い、test の読み方は、別の人が分けると動く。根拠は表のメモに書いたが、2人で分けて比べてはいない。
- **v0.3 には Constraint の正式な形が無い**（docs/17 の未決2）。Count ・ Implies などの形は、この実験で置いた試案である。v0.3 がこの形を採るとは言えない。
- **c は「v0.3 の examples に置き場所が無い」という判定である。** v0.3 の構想（docs/14）は、時間の制約も表すことを目指している。v0.3 が将来これらを表せないとは言えない。
- **重なりを除いた「異なる決まりの数」は出していない。** 397件は出どころごとの記述の数であり、同じ決まりが2〜3か所に書かれているものがある。
- **数えたのはボードのスキーマだけである。** 回答 ・ 回の記録 ・ ダッシュボード ・ チャットのスキーマと、advance.rs ・ answer.rs ・ scope.rs ・ agent.rs の決まりは数えていない。読んだのは、W の分類に必要とする箇所だけである。
- **「検査されていない」は、この実験で読んだ範囲で検査を見つけていないという意味である。** 外の URL を読まない HTML（P71）と、rounds/ ・ answers/ を書き換えない（G5）は、ほかのコードが見ている可能性を消していない。
- **実物のボードは2枚だけである。** 決まりが破れる頻度を一般化できない。
- **試していない20件**（a の残り）が、標準のキーワードで書けるかは確かめていない。I6 は anyOf と contains で書けるはずだが、動かしていない。
- **AI に意味を判断させたときの当たり外れ**は測っていない。12件は、Interpreter として呼んだことが無い。
- **measure_unchecked.py の文の切り方**は text.rs を Python に写したものである。Rust の結果と同じかは、同じ入力で比べていない。
- **表せる範囲の差が0件**というのは、このボードの116件についてである。ほかの Skill の決まりで差が出ないとは言えない。

## 置いてあるもの

| 場所 | 中身 |
|---|---|
| `README.md` | この文書 |
| `count.py` ・ `inventory.json` ・ `inventory.md` | 決まりの数え上げ。`inventory.json` が397件の全部、`inventory.md` が下の表 |
| `measure_unchecked.py` ・ `measure_unchecked.out.json` | 検査されていない決まりを実物に適用した結果 |
| `make_trials.py` ・ `trials.out.json` | いまの基盤で試した結果 |
| `xref-trial/` ・ `jsonschema-trial/` ・ `keyword-trial/` | 試しに写したスキーマとボード。実物は書き換えていない |
| `build_v03.py` ・ `v03/` | v0.3 の形の試案 |
| `sources/` | JSON Schema 2020-12 の Core の仕様の原文（取得した日 2026-10-17） |

## 表3 形以外の決まり（138件）

出どころの `brainstorming/` は `.claude/skills/brainstorming/`、`domain/` は `src/brainstorming/crates/core/src/domain/` を指す。形の259件は `inventory.json` にある。

| id | 出どころ | 決まり | いま表している所 | いまの検査 | v0.3 | いまの基盤で試した結果 | 同じ決まり | メモ |
|---|---|---|---|---|---|---|---|---|
| P1 | brainstorming/assets/board.schema.json:14 | `/` decisions-only：利用者に渡すのは決定だけにし、事実は調べて根拠に書く。（test：問いに、調べれば分かることが入っていないかを見る。） | x-prompt の文 | A | b |  |  | test は「調べれば分かることが入っていないか」。意味の判断 |
| P2 | brainstorming/assets/board.schema.json:20 | `/` refuted-first：案を複数出して反証し、残った1つだけを答えに書く。除外した案は rejected に残す。（test：rejected が1件以上あるかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「rejected が1件以上か」。rejected に minItems が無く、どこも数えていない |
| P3 | brainstorming/assets/board.schema.json:50 | `/properties/$schema` this-name：brainstorming/board@v1 と書く。（test：値がこの文字列と同じかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P4 | brainstorming/assets/board.schema.json:67 | `/properties/kind` const：board と書く。（test：値が同じかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P5 | brainstorming/assets/board.schema.json:84 | `/properties/id` file-name：.brainstorming/<id>.json のファイル名と同じにする。（test：ファイル名と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「ファイル名と同じか」。xref-trial の id-differs-file で、check も inspect も見ないことを確かめた |
| P6 | brainstorming/assets/board.schema.json:102 | `/properties/title` short：40字くらいまでの名詞句で書く。（test：40字を超えていないかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「40字を超えていないか」。title は minLength 1 だけ |
| P7 | brainstorming/assets/board.schema.json:120 | `/properties/intro` one-sentence：何を決めるかを1文で書く。決まったことの一覧は書かない。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P8 | brainstorming/assets/board.schema.json:138 | `/properties/hold` by-user：利用者が画面の「保留にする」で書き、「保留を解く」で消す。AI は書かない。（test：date があるかを見る。） | x-prompt の文 | O | c |  |  | rule は「AI は書かない」。誰が書いたかはインスタンスに残らない。test の date の有無は required が見る |
| P9 | brainstorming/assets/board.schema.json:159 | `/properties/hold/properties/date` today：保留にした日を書く。（test：日付の形かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の pattern が検査する |
| P10 | brainstorming/assets/board.schema.json:177 | `/properties/hold/properties/why` optional：利用者が書いたときだけ入れる。（test：空の文字でないかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の minLength が検査する |
| P11 | brainstorming/assets/board.schema.json:197 | `/properties/flag` rare：ほかの作業を止めているときだけ true にする。（test：true のボードが少数かを見る。） | x-prompt の文 | A | b |  |  | test は「true のボードが少数か」。少数の線が決まっておらず、ほかの作業を止めているかは意味の判断 |
| P12 | brainstorming/assets/board.schema.json:214 | `/properties/waits_for` by-id：ほかのボードの id を書く。（test：指す先のボードがあるかを check で見る。） | x-prompt の文 | R | c |  |  | x-ref（to: board）を check がディレクトリの全部のボードで検査する。v0.3 の要求の例はインスタンスを1件しか渡さない |
| P13 | brainstorming/assets/board.schema.json:239 | `/properties/round` by-advance：advance が書く。手で書き換えない。（test：rounds/ の最後の番号と同じかを見る。） | x-prompt の文 | W | c |  |  | advance が書く（advance.rs:142）。test の「rounds/ の最後の番号と同じか」は誰も見ない。rounds/ はインスタンスの外 |
| P14 | brainstorming/assets/board.schema.json:257 | `/properties/opened` by-advance：advance が書く。（test：日付の形かを見る。） | x-prompt の文 | S | a |  |  | test は「日付の形か」で pattern が検査する。書くのは advance |
| P15 | brainstorming/assets/board.schema.json:275 | `/properties/queue` ready-only：前提が片付いた論点だけを並べ、なぜいま開いたかを1文で書く。（test：並べた論点の depends がすべて決着しているかを見る。） | x-prompt の文 | I | a | x-ref で書けた |  | inspect.rs:55 と :74 が見る。xref-trial で x-ref の accept と when でも書けた |
| P16 | brainstorming/assets/board.schema.json:306 | `/properties/queue/items/properties/topic` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P17 | brainstorming/assets/board.schema.json:324 | `/properties/queue/items/properties/why` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P18 | brainstorming/assets/board.schema.json:343 | `/properties/changes` by-advance：advance が、前の回と比べて書く。理由には差し戻しの言葉をそのまま入れる。（test：前の回の記録と比べて、変わった欄だけが並んでいるかを見る。） | x-prompt の文 | W | c |  |  | advance が前の回と比べて書く（advance.rs:162-205）。前の回はインスタンスの外 |
| P19 | brainstorming/assets/board.schema.json:375 | `/properties/changes/items/properties/topic` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P20 | brainstorming/assets/board.schema.json:403 | `/properties/changes/items/properties/part` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P21 | brainstorming/assets/board.schema.json:421 | `/properties/changes/items/properties/summary` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P22 | brainstorming/assets/board.schema.json:439 | `/properties/changes/items/properties/reason` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P23 | brainstorming/assets/board.schema.json:458 | `/properties/log` append：新しいものを先頭に足す。（test：日付が新しい順かを見る。） | x-prompt の文 | M | a |  |  | test は「日付が新しい順か」。どこも見ていない |
| P24 | brainstorming/assets/board.schema.json:483 | `/properties/log/items/properties/date` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P25 | brainstorming/assets/board.schema.json:500 | `/properties/log/items/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P26 | brainstorming/assets/board.schema.json:519 | `/properties/topics` three-to-five：3〜5件にし、依存の順に並べる。（test：件数を見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「件数」。minItems 1 だけで、実物のボード2枚とも6件（measure_unchecked.out.json） |
| P27 | brainstorming/assets/board.schema.json:544 | `/$defs/topic` one-decision：1つの論点で決めることを1つにする。（test：答えの1文が1つのことを決めているかを見る。） | x-prompt の文 | A | b |  |  | test は「答えの1文が1つのことを決めているか」。意味の判断 |
| P28 | brainstorming/assets/board.schema.json:569 | `/$defs/topic/properties/id` unique：ボードの中で重ならない id を書く。（test：重なりが無いかを見る。） | x-prompt の文 | M | a |  |  | test は「重なりが無いか」。xref-trial の dup-topic-id で、重なった id は参照されたときにだけ「1つに決まらない」で出た |
| P29 | brainstorming/assets/board.schema.json:587 | `/$defs/topic/properties/name` short：10字くらいの名前を書く。（test：長すぎないかを見る。） | x-prompt の文 | M | a |  |  | test は「長すぎないか」。線が「10字くらい」で決まっていない |
| P30 | brainstorming/assets/board.schema.json:608 | `/$defs/topic/properties/status` by-flow：前提が片付いていなければ waiting、AI が開いて利用者の回答を待つなら open、承認されて更新したら settled にする。（test：queue と合っているかを見る。） | x-prompt の文 | I | a |  |  | test は「queue と合っているか」。inspect.rs:50 ・ :55 ・ :65 が見る |
| P31 | brainstorming/assets/board.schema.json:626 | `/$defs/topic/properties/question` decidable：答えが1つに決まる問いを書く。（test：答えの1文が問いに答えているかを見る。） | x-prompt の文 | A | b |  |  | test は「答えの1文が問いに答えているか」。意味の判断 |
| P32 | brainstorming/assets/board.schema.json:652 | `/$defs/topic/properties/answer` one-sentence-first：承認してほしいことを1文で text に書き、詳しい決まりを details に並べる。（test：text が120字までかを見る。） | x-prompt の文 | I | b |  |  | test は「text が120字までか」。inspect.rs:105 が見るのは欄ではなく1文の長さ（引用を除く） |
| P33 | brainstorming/assets/board.schema.json:671 | `/$defs/topic/properties/answer/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P34 | brainstorming/assets/board.schema.json:686 | `/$defs/topic/properties/answer/properties/details` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P35 | brainstorming/assets/board.schema.json:701 | `/$defs/topic/properties/answer/properties/details/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P36 | brainstorming/assets/board.schema.json:740 | `/$defs/topic/properties/images` always：答えの完成イメージを1つ以上置く。（test：1つ以上あるかを見る。） | x-prompt の文 | I | a | 標準のキーワードで書けた |  | inspect.rs:123 が open の論点だけ見る。if/then で書けた |
| P37 | brainstorming/assets/board.schema.json:746 | `/$defs/topic/properties/images` kind-by-content：示すものから種類を選ぶ。比べ ・ 流れ ・ 構成は figure、変更前後は diff、触れるものは screen、コマンドは run、フォルダは tree、ファイルの本文は code にする。（test：code の完成イメージが、ファイルとして成り立つ本文（コード ・ 設定 ・ Markdown など）かを見る。） | x-prompt の文 | A | b |  |  | test は「code がファイルとして成り立つ本文か」。inspect.rs:113 は言語が text などかだけを見る |
| P38 | brainstorming/assets/board.schema.json:767 | `/$defs/topic/properties/assumptions` numbered：答えが前提にしたことを1件ずつ書く。（test：1件に1つのことかを見る。） | x-prompt の文 | A | b |  |  | test は「1件に1つのことか」。意味の判断 |
| P39 | brainstorming/assets/board.schema.json:782 | `/$defs/topic/properties/assumptions/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P40 | brainstorming/assets/board.schema.json:812 | `/$defs/topic/properties/checks` three：読み手に確かめてほしい箇所を3件まで書く。（test：3件以下かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の maxItems 3 が検査する |
| P41 | brainstorming/assets/board.schema.json:830 | `/$defs/topic/properties/checks/items/properties/target` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P42 | brainstorming/assets/board.schema.json:846 | `/$defs/topic/properties/checks/items/properties/ask` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P43 | brainstorming/assets/board.schema.json:877 | `/$defs/topic/properties/grounds` three-parts：答えのどこを支えるか ・ もとにしたこと ・ 出どころの3つを書く。（test：3つがそろっているかを見る。） | x-prompt の文 | S | a |  |  | items の required が4つを検査する |
| P44 | brainstorming/assets/board.schema.json:895 | `/$defs/topic/properties/grounds/items/properties/supports` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P45 | brainstorming/assets/board.schema.json:918 | `/$defs/topic/properties/grounds/items/properties/basis` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P46 | brainstorming/assets/board.schema.json:936 | `/$defs/topic/properties/grounds/items/properties/tag` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P47 | brainstorming/assets/board.schema.json:952 | `/$defs/topic/properties/grounds/items/properties/source` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P48 | brainstorming/assets/board.schema.json:981 | `/$defs/topic/properties/rejected` keep：反証で外した案と、何が壊れるかを書く。（test：1件以上あるかを見る。） | x-prompt の文 | M | a | 標準のキーワードで書けた |  | test は「1件以上あるか」。rejected に minItems が無い |
| P49 | brainstorming/assets/board.schema.json:999 | `/$defs/topic/properties/rejected/items/properties/option` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P50 | brainstorming/assets/board.schema.json:1015 | `/$defs/topic/properties/rejected/items/properties/reason` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P51 | brainstorming/assets/board.schema.json:1042 | `/$defs/topic/properties/depends` declared：前提にした論点の id を書く。（test：指す論点があるかを見る。） | x-prompt の文 | R | a |  |  | x-ref（to: self）を check が、inspect.rs:72 も検査する |
| P52 | brainstorming/assets/board.schema.json:1077 | `/$defs/topic/properties/verification` not-shown：書き手の検証の記録を書く。描画しない。（test：画面に出ていないかを見る。） | x-prompt の文 | W | b |  |  | x-view の hidden に render が従う。test の「画面に出ていないか」は描いてみないと分からない |
| P53 | brainstorming/assets/board.schema.json:1094 | `/$defs/topic/properties/verification/properties/passed` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P54 | brainstorming/assets/board.schema.json:1110 | `/$defs/topic/properties/verification/properties/passed/items/properties/option` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P55 | brainstorming/assets/board.schema.json:1129 | `/$defs/topic/properties/verification/properties/passed/items/properties/cost` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P56 | brainstorming/assets/board.schema.json:1143 | `/$defs/topic/properties/verification/properties/assumptions` numbered：答えが前提にしたことを1件ずつ書く。（test：1件に1つのことかを見る。） | x-prompt の文 | A | b |  |  | test は「1件に1つのことか」。意味の判断 |
| P57 | brainstorming/assets/board.schema.json:1170 | `/$defs/topic/properties/verification/properties/assumptions/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P58 | brainstorming/assets/board.schema.json:1190 | `/$defs/topic/properties/verification/properties/out_of_scope` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P59 | brainstorming/assets/board.schema.json:1207 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/item` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P60 | brainstorming/assets/board.schema.json:1223 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/treatment` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P61 | brainstorming/assets/board.schema.json:1242 | `/$defs/topic/properties/verification/properties/out_of_scope/items/properties/note` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P62 | brainstorming/assets/board.schema.json:1256 | `/$defs/topic/properties/verification/properties/history` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P63 | brainstorming/assets/board.schema.json:1271 | `/$defs/topic/properties/verification/properties/history/items` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P64 | brainstorming/assets/board.schema.json:1291 | `/$defs/figure` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P65 | brainstorming/assets/board.schema.json:1313 | `/$defs/figure/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P66 | brainstorming/assets/board.schema.json:1331 | `/$defs/figure/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P67 | brainstorming/assets/board.schema.json:1350 | `/$defs/figure/properties/svg` checked：design-svg の検査を通した SVG を書く。（test：render が断らないかを見る。） | x-prompt の文 | W | b |  |  | contentMediaType image/svg+xml の入力を render が検査する（schema-driven SKILL.md:173） |
| P68 | brainstorming/assets/board.schema.json:1370 | `/$defs/screen` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P69 | brainstorming/assets/board.schema.json:1392 | `/$defs/screen/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P70 | brainstorming/assets/board.schema.json:1410 | `/$defs/screen/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P71 | brainstorming/assets/board.schema.json:1428 | `/$defs/screen/properties/html` standalone：外のファイルを読まない1枚の HTML を書く。（test：外の URL を読んでいないかを見る。） | x-prompt の文 | M | b |  |  | test は「外の URL を読んでいないか」。この実験で読んだ範囲では、見ているコードを見つけていない |
| P72 | brainstorming/assets/board.schema.json:1447 | `/$defs/screen/properties/height` fit：中身が収まる高さを書く。（test：描いて目で見る。） | x-prompt の文 | A | b |  |  | test は「描いて目で見る」。人の目の判断 |
| P73 | brainstorming/assets/board.schema.json:1467 | `/$defs/code` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P74 | brainstorming/assets/board.schema.json:1490 | `/$defs/code/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P75 | brainstorming/assets/board.schema.json:1508 | `/$defs/code/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P76 | brainstorming/assets/board.schema.json:1526 | `/$defs/code/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P77 | brainstorming/assets/board.schema.json:1544 | `/$defs/code/properties/language` lowercase：言語の名前を小文字で書く。（test：知っている名前かを見る。） | x-prompt の文 | M | a |  |  | test は「知っている名前か」。言語の名前の一覧を持てば汎用の検査で足りる |
| P78 | brainstorming/assets/board.schema.json:1550 | `/$defs/code/properties/language` file-language：そのファイルの言語（rust ・ json ・ markdown ・ shell など）を書く。text は書かない。（test：inspect が、言語が text ・ plain ・ txt のコードを見つけないかを見る。） | x-prompt の文 | I | a | 標準のキーワードで書けた |  | inspect.rs:113 が見る。not enum で書けた（ただし大文字小文字を区別する） |
| P79 | brainstorming/assets/board.schema.json:1568 | `/$defs/code/properties/text` only-needed：見せたい部分だけを書く。（test：60行くらいまでかを見る。） | x-prompt の文 | M | a |  |  | test は「60行くらいまで」。行数は数えられるが、どこも数えていない |
| P80 | brainstorming/assets/board.schema.json:1574 | `/$defs/code/properties/text` real-file：実在するか作る予定のファイルの本文だけを書く。場面の比べや一覧は図にする。（test：path に書いたファイルの本文として成り立つかを見る。） | x-prompt の文 | A | b |  |  | test は「path のファイルの本文として成り立つか」。言語ごとの読み手を必要とする |
| P81 | brainstorming/assets/board.schema.json:1596 | `/$defs/code/properties/highlight` few：読んでほしい行だけを書く。（test：全体の行の数より十分少ないかを見る。） | x-prompt の文 | A | b |  |  | test は「全体の行の数より十分少ないか」。十分の線が無い |
| P82 | brainstorming/assets/board.schema.json:1614 | `/$defs/code/properties/note` why-highlight：強調した行が何かを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P83 | brainstorming/assets/board.schema.json:1634 | `/$defs/diff` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P84 | brainstorming/assets/board.schema.json:1658 | `/$defs/diff/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P85 | brainstorming/assets/board.schema.json:1676 | `/$defs/diff/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P86 | brainstorming/assets/board.schema.json:1694 | `/$defs/diff/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P87 | brainstorming/assets/board.schema.json:1712 | `/$defs/diff/properties/language` lowercase：言語の名前を小文字で書く。（test：知っている名前かを見る。） | x-prompt の文 | M | a |  |  | test は「知っている名前か」。言語の名前の一覧を持てば汎用の検査で足りる |
| P88 | brainstorming/assets/board.schema.json:1729 | `/$defs/diff/properties/before` verbatim：変更前の原文をそのまま書く。（test：原文と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「原文と同じか」。リポジトリのある時点のファイルを必要とする |
| P89 | brainstorming/assets/board.schema.json:1746 | `/$defs/diff/properties/after` verbatim：変更後の原文をそのまま書く。（test：原文と同じかを見る。） | x-prompt の文 | O | c |  |  | test は「原文と同じか」。リポジトリのある時点のファイルを必要とする |
| P90 | brainstorming/assets/board.schema.json:1766 | `/$defs/diff/properties/layout` default-unified：並べたいときだけ split と書く。（test：値を見る。） | x-prompt の文 | S | a |  |  | 同じ節点の enum が検査する |
| P91 | brainstorming/assets/board.schema.json:1786 | `/$defs/tree` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P92 | brainstorming/assets/board.schema.json:1808 | `/$defs/tree/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P93 | brainstorming/assets/board.schema.json:1826 | `/$defs/tree/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P94 | brainstorming/assets/board.schema.json:1856 | `/$defs/tree/properties/rows` top-down：上から順に、フォルダの中身を字下げで並べる。（test：depth が親より1つ深いかを見る。） | x-prompt の文 | M | b |  |  | test は「depth が親より1つ深いか」。並びの前後を比べる専用の検査を必要とする |
| P95 | brainstorming/assets/board.schema.json:1875 | `/$defs/tree/properties/rows/items/properties/path` as-in-repo：リポジトリの根からの場所で書く。（test：その場所が在るか、作る予定かを見る。） | x-prompt の文 | O | c |  |  | test は「その場所が在るか、作る予定か」。リポジトリと書き手の予定を必要とする |
| P96 | brainstorming/assets/board.schema.json:1893 | `/$defs/tree/properties/rows/items/properties/depth` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P97 | brainstorming/assets/board.schema.json:1914 | `/$defs/tree/properties/rows/items/properties/note` why-highlight：強調した行が何かを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P98 | brainstorming/assets/board.schema.json:1930 | `/$defs/tree/properties/rows/items/properties/mark` as-described：説明のとおりに書く。（test：説明と合っているかを見る。） | x-prompt の文 | D | - |  |  | description と同じことを言うだけで、独自の決まりを持たない |
| P99 | brainstorming/assets/board.schema.json:1950 | `/$defs/run` one-thing：この完成イメージで示すことを1つにし、caption にそれを1文で書く。（test：caption が1文かを見る。） | x-prompt の文 | M | b |  |  | test は「caption が1文か」。文の切り方（text.rs の sentences）を持つ Interpreter を必要とする |
| P100 | brainstorming/assets/board.schema.json:1973 | `/$defs/run/properties/kind` as-is：完成イメージの種類の名前をそのまま書く。（test：6つの名前のどれかかを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の const が検査する |
| P101 | brainstorming/assets/board.schema.json:1991 | `/$defs/run/properties/caption` one-sentence：何を示すかを1文で書く。（test：1文かを見る。） | x-prompt の文 | M | b |  |  | test は「1文か」。文の切り方を持つ Interpreter を必要とする |
| P102 | brainstorming/assets/board.schema.json:2009 | `/$defs/run/properties/cwd` where：実行した場所を書く。（test：場所が在るかを見る。） | x-prompt の文 | O | c |  |  | test は「場所が在るか」 |
| P103 | brainstorming/assets/board.schema.json:2027 | `/$defs/run/properties/command` as-typed：打ったとおりに書く。（test：実行できるかを見る。） | x-prompt の文 | O | c |  |  | test は「実行できるか」 |
| P104 | brainstorming/assets/board.schema.json:2044 | `/$defs/run/properties/output` as-printed：出たとおりに書き、長ければ途中を省いたと書く。（test：実物の出力と比べる。） | x-prompt の文 | O | c |  |  | test は「実物の出力と比べる」 |
| P105 | brainstorming/assets/board.schema.json:2061 | `/$defs/run/properties/exit` as-returned：返った終了コードを書く。（test：整数かを見る。） | x-prompt の文 | S | a |  |  | 同じ節点の type integer が検査する |
| I1 | domain/inspect.rs:40 | 回答待ち（open）の論点は5件まで | Rust | I | a | 標準のキーワードで書けた |  | contains と maxContains で書けた |
| I2 | domain/inspect.rs:50 | open の論点があれば queue は空でない | Rust | I | a | 標準のキーワードで書けた | x-prompt:608 | if/then で書けた |
| I3 | domain/inspect.rs:58 | queue の topic が指す論点がある | Rust | I | a |  | x-ref:/properties/queue/items/properties/topic | x-ref の check と同じことを見る |
| I4 | domain/inspect.rs:55 | queue の topic が指す論点は open | Rust | I | a | x-ref で書けた | x-prompt:275 | accept（at: status, in: [open]） |
| I5 | domain/inspect.rs:62 | ボードに status を書かない | Rust | I | a | 標準のキーワードで書けた | G8 | 根の additionalProperties:false がすでに断る（xref-trial/boards/board-status.json で確かめた） |
| I6 | domain/inspect.rs:65 | open が無いのに決着していない論点があれば誤り | Rust | I | a |  | x-prompt:608 | anyOf と contains で書けるはずだが試していない |
| I7 | domain/inspect.rs:72 | depends が指す論点がある | Rust | I | a |  | x-prompt:1042 | x-ref の check と同じことを見る |
| I8 | domain/inspect.rs:74 | open の論点の depends はすべて settled | Rust | I | a | x-ref で書けた | x-prompt:275 | when（status: [open]）と accept（at: status, in: [settled]） |
| I9 | domain/inspect.rs:92 | 答えの本文が指す論点（Q と数字）は depends か下流にある | Rust | I | b |  |  | 本文から id を拾う決まり（inspect.rs:12 の mentioned）を持つ Interpreter を必要とする |
| I10 | domain/inspect.rs:105 | 答え ・ 詳しい決まり ・ 外した理由の1文は120字まで（「」の中は数えない） | Rust | I | b |  | x-prompt:652 | 文の切り方は text.rs:7 ・ 上限は text.rs:4 |
| I11 | domain/inspect.rs:113 | code の完成イメージの言語は text ・ plain ・ txt ・ 空でない | Rust | I | a | 標準のキーワードで書けた | x-prompt:1550 | not enum で書けた |
| I12 | domain/inspect.rs:123 | open の論点には完成イメージがある | Rust | I | a | 標準のキーワードで書けた | x-prompt:740 | if/then で書けた |
| I13 | domain/inspect.rs:132 | open の論点は、下流の論点に使われるか、外の作業の行き先が out_of_scope にある | Rust | I | b |  |  | 逆向きの参照と文字列の中身の「または」。x-ref の inverse には「または」が無い |
| D1 | domain/status.rs:69 | 論点のステータス：settled→決着、open→回答待ち、ほか→前提待ち | Rust | W | c |  |  | いまの基盤の x-derive（rules ・ otherwise）なら書ける形。v0.3 の試案には導く決まりの置き場所が無い |
| D2 | domain/status.rs:79 | AI が動いていればボードは更新中 | Rust | W | c |  |  | ロックのファイルとプロセスの状態 |
| D3 | domain/status.rs:82 | hold があればボードは保留 | Rust | W | c |  |  | 導く決まり。x-derive の when は値の一致だけで、有無を見られない |
| D4 | domain/status.rs:86 | 論点がすべて settled（1件以上）ならボードは決着 | Rust | W | c |  |  | 導く決まり。全部についての条件 |
| D5 | domain/status.rs:89 | それ以外のボードは回答待ち | Rust | W | c |  |  | 導く決まり |
| L1 | domain/lock.rs:206 | ロックが新しいあいだは持ち主だけが書く | Rust | W | c |  |  | プロセスと時間 |
| L2 | domain/lock.rs:175 | serve のロックは60秒で古くなる | Rust | W | c |  |  | 時間 |
| L3 | domain/lock.rs:177 | ターミナルのロックは10分で古くなる | Rust | W | c |  |  | 時間 |
| L4 | domain/lock.rs:190 | ロックの中身は board ・ owner ・ what ・ since ・ beat | Rust | W | a |  |  | 形の決まりだがスキーマが無く、コードの中にだけある |
| G1 | brainstorming/SKILL.md:119 | 案を並べて選ばせない | SKILL.md の文 | A | b |  | x-prompt:20 |  |
| G2 | brainstorming/SKILL.md:120 | 事実を利用者に聞かない | SKILL.md の文 | A | b |  | x-prompt:14 |  |
| G3 | brainstorming/SKILL.md:121 | 外した案を黙って消さない | SKILL.md の文 | W | c |  |  | advance が数の増減を changes に書く（advance.rs:177）。前の回はインスタンスの外 |
| G4 | brainstorming/SKILL.md:122 | ボードの JSON を手で書き換えない | SKILL.md の文 | O | c |  |  | 誰がどう書いたか |
| G5 | brainstorming/SKILL.md:123 | rounds/ と answers/ を書き換えない | SKILL.md の文 | O | c |  |  | この実験で読んだ範囲では、見ているコードを見つけていない |
| G6 | brainstorming/SKILL.md:124 | ボードのフォルダの外を書き換えない | SKILL.md の文 | W | c |  |  | serve が作業の前後を比べて知らせる（scope.rs:14） |
| G7 | brainstorming/SKILL.md:125 | AI の「検査した」で合格としない | SKILL.md の文 | O | a |  |  | 決まりではなく手順。v0.3 では Interpreter の応答の status と providedCapabilities が持つ |
| G8 | brainstorming/SKILL.md:126 | ボードに status を書かない | SKILL.md の文 | S | a | 標準のキーワードで書けた | I5 | 根の additionalProperties:false |
| G9 | brainstorming/SKILL.md:127 | ロックを残したまま作業をやめない | SKILL.md の文 | W | c |  |  | 外し忘れは lock.rs の時間で外れる |
| G10 | brainstorming/SKILL.md:128 | 画面の AI に聞くで、Kiro は読むだけ | SKILL.md の文 | W | c |  |  | agent.rs:3 が起動のしかたで絞る |
| G11 | brainstorming/SKILL.md:129 | 扱わない範囲に扱いを付ける | SKILL.md の文 | S | a |  |  | out_of_scope の items の required と enum |
