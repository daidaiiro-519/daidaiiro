# 反証：ドメインモデル ・ JSON Schema ・ DB（第2回）

見たもの：domain-model-v2.json（decisions-in-session.md と answers-v2.json を当てた姿として読む）・ abstract-draft/ の abstract.json ・ change.schema.json ・ judgment.schema.json ・ concrete.json ・ instances.json ・ records.example.json ・ db.sql ・ build.py の場面。

実際に動かした検査（scratchpad/adversarial/ の db2.py ・ js.py）：

- DDL：SQLite のメモリの DB に db.sql を流し、壊す操作を15件当てた。
- JSON Schema：Draft 2020-12 の jsonschema で、change と judgment のスキーマを本体で、records.example をそれぞれのスキーマで検査した。壊した Concrete と記録も当てた。

判定の書き方：重さは「答えを変える必要がある」「直せば残る」「問題なし」の3つで書く。前提の番号は board-answers.json の details を1から数えた番号（例：Q1 #30＝Q1 の details の30番目）で書く。

---

## 1. 前提に反して壊れる場面

| # | 壊れる場面（アクター ・ 操作 ・ 事前 → 事後） | 依拠した前提 | 重さ |
|---|---|---|---|
| 1-1 | 人が README.md をエディタで直す。または AI が Edit の道具で HTML や Rust のソースを直す。事前：基盤の入口を通らない。事後：変更の記録が無いので、変更を見せると「記録に無い」（I31）が毎回出る。どちらの案でも、基盤の外で書いたファイルの意図を残す入口が無い | user-instructions.md「既存のこのSemanticSchemaで作成していないマークダウンドキュメントやHTMLやソースコードに対しても差分を記録できるように」／ domain-model-v2 I31「記録の無い書き換えは「記録に無い」…として出る」 | 答えを変える必要がある（A2 に、入口を通らずに書いたファイルへ意図を付ける操作を足す） |
| 1-2 | 1-1 の続き。ソースコードの変更で I33（意図と変更が合っているか）を判定すると、前後の値として基盤がソースを読む | user-instructions.md（同上）と、board Q1 #8「基盤はソースを読まず、テストが走ったときに書く記録（JSON）でつながる」・ I36「見ていないもの（ソースコード ・ テストのコード）を示す」が食い違う（7節 G1） | 答えを変える必要がある（優先順で利用者の指示が勝つので、Q1 #8 と I36 を直す） |
| 1-3 | 目的のセッションの AI が、Bash で基盤の CLI の「判定する」を呼び、status を accepted にする。事後：writer_kind は human、人の判定の記録も残る。人の入口を通ったかどうかを、基盤は見分けられない | board Q0 #3「作業した本人が承認するのは基盤の操作で、人の判定の記録として残す」／ domain-model-v2 ports.inbound「判定する…人の操作の入口だけ（MCP には見せない）」／ Q1 #18「基盤は許可の境界を持たない」／ Q8 #9「Out は…本人確認」 | 直せば残る（「人の入口」を守るのは各 AI ツールの設定ファイルで、基盤は保証しない、と I18 ・ I21 に書く） |
| 1-4 | A1「セッションの中で本人が承認する」。事前：セッションは AI ツールの会話。人がチャットで「承認」と書く。事後：基盤を呼ぶのは AI（MCP か CLI）になる。人の入口は MCP に見せないので、セッションの中から承認を呼べない | answers-v2 A1「作業した本人がセッションの中で承認するのは基盤の操作」と、ports.inbound「判定する：人の操作の入口だけ（MCP には見せない）」が食い違う | 答えを変える必要がある（A1 か入口の、どちらかを直す） |
| 1-5 | 場面5（build.py の CH1）で、AI が ACDR 0197 を作る。status は既定値 proposed で、変更の項目に /status が入る。事後：x-decided-by: human の欄を AI が書いたので、作るたびに「人が決める欄の書き換え」が出る | domain-model-v2 I21「決定権が人の欄を、人の入口を通らずに変えた変更は、取り込めない理由に出る」／ concrete.json status の x-decided-by: "human" と default: "proposed" | 直せば残る（作成時の既定値は人の入口を通ったものとして扱う、と I21 に書く） |
| 1-6 | 場面10（build.py の J4）で、本人の承認を判定の記録「status/human-approves」に残している。この行は by: machine。事後：人が machine の行の結果を書いている | board Q1 #57「人が判定の記録に残すのは、判断待ちと AI の rule の結果だけ」／ I18「人が覆せるのは AI と人の rule の結果だけ」 | 答えを変える必要がある（承認を残す形が無い。by: human の行を置くか、承認を判定の記録とは別の形にする） |
| 1-7 | オーケストレーターのセッションが CLAUDE.md や memory を書く。I38 で目的の参照を写すが、オーケストレーターは目的を持たない（DDL で purpose_ref は NULL）。事後：change.schema の purpose_ref（必須 ・ minLength 1）を満たせず、記録が書けない | I38「その呼び出しで書くすべての変更の記録に（目的の参照を）写し」／ db.sql「purpose_ref TEXT, -- オーケストレーターのセッションは持たない」／ change.schema.json の required の purpose_ref | 直せば残る（オーケストレーターは書かない、と不変条件にするか、書くときの目的を決める） |
| 1-8 | 人やハーネスや CI が、セッション ID を渡さずに書く。I38 は「セッションが無いと索引に残す」と言うが、session_changes は tool と id が NOT NULL で外部キーがある。事後：残せない（db2.py で NOT NULL により断られた） | I38「セッション ID を受け取らなければ…セッションが無いと索引に残す」 | 直せば残る（索引の行の形を直すか、I38 の後半を外す） |
| 1-9 | 端末1で claude-code のオーケストレーター 0a9d が開いている。端末2で利用者が Codex を始めると、DB が断る（dbcheck.py の想定どおり）。しかし Codex は claude-code の ID では再開できない | user-instructions.md「他のターミナルで開きたくなったらそのsessionIDでResumeすればいいしね」／ db.sql one_open_orchestrator_session（tool を問わず1つ） | 直せば残る（オーケストレーターの1つを tool ごとにするか、別の AI ツールでは閉じてから立てる、と決める） |
| 1-10 | 古いオーケストレーター 0a9d が閉じたあとも、その子の目的のセッション 7c1e（ACDR 0197）は開いている。新しいオーケストレーター 0a9e が ACDR 0197 を進める。事後：「再開か新規かを決める」は親が違うので「新しく立てる」を返す。DB は one_open_purpose_session で登録を断る。A5 に従って 7c1e を再開しても、キャッシュの有効時間が切れていれば、Q11 が避けたい高い再開になる。7c1e を閉じてよい者（利用者か親）は居ない | Q11 #3「目的 ・ 親 ・ 役割が同じで開いていて…再開する」／ decisions「同じ目的の目的のセッションは親が違っても1つまで」／ Q11 #2「閉じるのは、利用者か親が閉じると決めたとき」／ domain_services「再開か新規かを決める」（直されていない） | 答えを変える必要がある（ドメインサービスの規則を I46 の新しい形に直し、親が閉じた目的のセッションの扱いを決める） |
| 1-11 | 1つの worktree で、オーケストレーターが ACDR 0197 と ACDR 0198 の目的のセッションを並べて動かす。両方が CLAUDE.md を書く。0197 で本人が承認すると、A3 (4)「確定する」にファイルの一覧を渡す。事後：ファイル単位の確定なので、0198 が書いた CLAUDE.md の差分も 0197 の単位に入る。0198 の変更の記録は確定されないまま残り、A6 で「この前後には無い」と出る。0197 の単位では、記録の無い書き換えとして出る | A1「承認したら、基盤がポートの『確定する』を呼ぶ」／ A3 (4)「確定する：対象のファイルの一覧と説明 → 単位の参照」／ I31 | 直せば残る（同じ対象を開いている2つの目的が書いたら、確定する前に例外として出す） |
| 1-12 | 規則 state-based（rule は「ある時点のインスタンスを見れば判定できる状態」として書く）を当てると、id を変えない ・ 記録を書き換えない ・ 人の入口から来た値だけ、といった行は前と後の2つの時点を比べるので、1時点の状態では書けない。案の中の never-change-id ・ no-rewrite ・ human-approves ・ from-session は「〜しない」「〜を写す」と操作の形で書かれており、本体自身の行も命令の形で、本体の judge に当てると violated になる | decisions「TLA+ ・ Z3 から採るのは「決まりは状態で書く」」／ abstract.json $defs/rule/rule の state-based「ある時点のインスタンスを見れば判定できる状態として書き」 | 直せば残る（「前の状態と後の状態の関係」も状態の決まりに含める。TLA+ の action の制約にあたる） |
| 1-13 | 判定の記録を「対象 × rule」で上書きする（A4）。同じ値で、別のモデルに聞き直して逆の結果が出る。事後：前の記録は消え、2件が並ばないので「判定が割れた」を出せない | Q7 #19「同じインスタンス ・ rule ・ 値に逆の判定が並んだら、unverified（根拠「判定が割れた」）」／ I14「割れた答えは unverified」 | 直せば残る（割れたことを、1件の記録の中で出せる欄を持つ。取り込みでは I22 の衝突で出る） |
| 1-14 | ACDR 0197 の slug を直してフォルダの名前が変わる。事後：判定の記録の target（ファイルのパス）と、変更の項目の target が無いファイルを指す。A4 の識別（対象 × rule）も変わり、効いていた判定がすべて別の対象のものになる | Q1 #35「1件は id と kind」／ judgment.schema target「判定したインスタンスのファイル」 | 直せば残る（対象をインスタンスの id と kind で指す。ファイルでしか指せない対象だけパスにする） |

## 2. ドメインモデル ・ スキーマ ・ DB の食い違い

| # | 食い違い | 依拠した前提 | 重さ |
|---|---|---|---|
| 2-1 | judgment.schema に「意味」（意図全体の1文と対象ごとの1文。各文に JSON Pointer と意図の id）の欄が無い。I33 の判定の記録が書けない | Q1 #49「記録は結果 ・ 判定した者 ・ 意味 ・ 判定の理由を持ち、意味と判定の理由は別の欄にする」／ value_objects「意味（I33）」 | 直せば残る |
| 2-2 | judged_values に、読んだ x-prompt の文 ・ 引き継ぎのプロンプト ・ I33 の前後の値 ・ term ・ 参照先の値が無い。reads と delegate と model だけ | Q0 #4「渡したものは判定した値に残す」／ Q1 #51「前の値 ・ 後の値 ・ 読んだ欄の値…読んだ x-prompt の文 ・ 任せる先とモデル」 | 直せば残る（足さないと、x-prompt を直しても判定が効いたままになる） |
| 2-3 | judgment.schema に「何が分かれば決められるか（問い）」の欄が無い。場面6の「least-privilege は unverified で、何が分かれば決められるかが付く」を書く場所が無い | decisions「AI の答えは rule ごとの5つの値と問い」 | 直せば残る |
| 2-4 | judgment.schema は id（ULID）を必須にし、置き場所を {id} にしている。A4 は「ULID の id を持たない」 | answers-v2 A4 | 直せば残る（A4 を採るなら、id と x-instance-path を直す） |
| 2-5 | 役割の形が3通りある。value_objects「役割：親の任せる先の名前」、db.sql「<Concrete の URN>#<x-delegates の名前>」、場面7の行「urn:…platform-advisor:judgment:v1#advisor」。場面7の URN は任せる側（ACDR）ではなく任せられる側のもので、ACDR の x-delegates には advisor が無い | Q1 #38「名前が子のセッションの役割になる」／ decisions「中身は自分の agent の定義（ほかの Concrete の URN は書かない）」 | 直せば残る（役割は親の Concrete の x-delegates にある名前に限る。advisor を ACDR の x-delegates に working で足す） |
| 2-6 | 集約「セッション」は目的の参照を必須にしている。DDL では NULL（オーケストレーター）にしている。ドメインモデルにはオーケストレーターのセッションが無い | aggregates セッション「セッション ID ・ 目的の参照 ・ 親のセッション ID（任意）」／ decisions の呼び名3つ | 直せば残る |
| 2-7 | I44「子の目的の参照は親と同じ」はすべての子に掛かる。決定は任せる先のセッションだけに当てる。DDL はどちらも守らない（任せる先の目的が親と違っても通った） | I44 ／ decisions「Q11 の不変条件①は任せる先のセッションにだけ当てる」／ A5「I44…を一員の rule として書き、書き込みを断るのはアダプターの一意の索引 ・ トリガー」 | 直せば残る（トリガーを足す） |
| 2-8 | DDL が通してしまう行（db2.py で確認）：親の無い目的のセッション（最上位が2つになる）、目的のセッションの下の目的のセッション、任せる先の下の任せる先、目的も役割も無い親付きの行、目的の無い任せる先の重複（NULL は一意の索引で別の値）、閉じた親の下の新しい子 | user-instructions.md「最上位のセッションはでもオーケストレーターではあるよね」「入れ子にしないでOK」／ decisions「呼ばれた子はほかの agent を呼ばず」 | 直せば残る（CHECK とトリガーで、3つの種類の形を閉じる） |
| 2-9 | INSERT OR REPLACE で閉じたセッションが開く（db2.py：D5 が closed から open に戻った）。DELETE してから同じ ID で入れ直しても開く。子の無いオーケストレーターは REPLACE で別のツールの行に入れ替わる。A5 の「閉じた行の削除も断る」トリガーが DDL に無い | I45「閉じたセッションは開き直さない」／ A5「閉じた行の削除も断る」 | 直せば残る（BEFORE DELETE のトリガーを足し、REPLACE を使わないことをアダプターの決まりにする） |
| 2-10 | 外部キーを有効にしていない接続では、無いセッションの索引の行が入る（db2.py で確認）。A5 はアダプターが必ず有効にすると書くが、DDL だけでは守れない | A5「アダプターは外部キーを必ず有効にし」 | 問題なし（A5 のとおりアダプターが守る。場面の検査に1件足すとよい） |
| 2-11 | Abstract の一員「session」は案が無く、A5 が書く2つ（session と session-change）と、Q6 の「session（集約 セッションと、手元のセッションの索引の行）」の1ファイルの数が合わない | Q6 #4「2群 値オブジェクト：…session（集約 セッションと、手元のセッションの索引の行）」 | 直せば残る |
| 2-12 | 変更の項目の対象が、ドメインモデルでは6種類（Concrete ・ インスタンス ・ 判定の記録 ・ glossary ・ ビュー ・ デザインシステム）に限られる。スキーマは任意の文字列。利用者の指示は任意のファイル | value_objects「変更の項目」／ user-instructions.md（1-1 と同じ） | 直せば残る（ドメインモデルを利用者の指示に合わせる） |
| 2-13 | 任せる先の形：ドメインモデルの entities は「名前 ・ role ・ Concrete の URN ・ モデル」、スキーマは role ・ answer ・ run。judged_values.model（records.example では "opus"）は、どこにも出どころが無い | entities 任せる先／ agreed.json「x-delegates の中身は自分の任せる先の定義（role ・ answer ・ run）」 | 直せば残る（モデルはアダプターが起動した結果として返す、と書く） |
| 2-14 | ports.outbound「変更の前後を受け取る」に、A3 の (4)「確定する」が無い。G1 の直し方の一覧にも入っていない | answers-v2 A3 ・ G1 | 直せば残る |
| 2-15 | change.schema と judgment.schema の行は agent: judge を指す。しかし、どちらのスキーマにも x-delegates が無い。本体の agent-exists「名前が x-delegates にあるかを見る」を当てると violated になる | decisions「基盤自身も x-delegates に judge を持つ」／ abstract.json rule.agent の agent-exists | 直せば残る（一員は本体の x-delegates を引く、と書く） |
| 2-16 | 場面8の CH2 の欄が "/shift/0/to" で、配列の添字まで下がっている。change.schema の unit-field にも反するが、スキーマは通す | Q1 #30「欄は x-prompt を持つ書く単位まで（配列の添字までは下げない）」 | 直せば残る（シミュレーションを直す） |
| 2-17 | 場面の記録が二重になっている。ACDR の docs（path と marks。acdr-inspect が Git の差分を読む）と、Abstract の変更の記録（items）が、同じ変更を別々に持つ。ACDR は Abstract の変更のまとまりの形を $ref で使っていない | user-instructions.md「ACDRはこれをより拡張して見やすくしたものである理解です」／ Q1 #46「Concrete はこの形を $ref で使って見せ方を足す」 | 直せば残る（ACDR の docs と acdr-inspect は、まとまりの形から導く） |
| 2-18 | ACDR の docs の行（why-per-change）の test に「git の差分のまとまりごとに」と書いてある。変更の仕組みを Git 以外へ替えると、この行は判定できない | user-instructions.md「変更差分のとり方が変わったとしてもアダプターが変わったとしても機能するように」 | 直せば残る（2-17 と一緒に直す） |

## 3. JSON Schema の構文と能力

| # | 壊れる場面 | 依拠した前提 | 重さ |
|---|---|---|---|
| 3-1 | Concrete の入れ子の欄（shift.items.properties.what）に、壊れた x-prompt（{"oops":1}）と未登録の x-storage を書いても、本体の検査は0件で通る。本体は $dynamicAnchor を持たないので、入れ子の欄には本体の形が掛からない。x-prompt の形と9つの x- の値の形は値だけで決まるのに、x-prompt の行 nested-fields（by: machine）に残っている | decisions「値だけで決まる決まりは if ・ then ・ dependentRequired ・ contains ・ oneOf ・ readOnly などで書き、x-prompt には書けないもの…だけを残す」／ user-instructions.md「このあたりはねフルに活用したほうがいいよ？」 | 直せば残る（$dynamicAnchor: meta で 2020-12 のメタスキーマを広げる。OpenAPI 3.1 の dialect と同じ作り方） |
| 3-2 | 根に未登録の x-db-table を書いた Concrete も0件で通る。保存の形の宣言が入る | abstract.json の abstract-keywords-closed の why「保存の形（DB ・ テーブル）の宣言も通る」／ I40「Concrete は保存の形を宣言しない」 | 問題なし（自分の x-extensions に依るので、値だけでは決まらない。x-prompt に残すのは正しい） |
| 3-3 | reads に "why のこと" と書いても通る。format: json-pointer は format-annotation の語彙では検査されない。format を検査させると1件出る。items.field ・ grounds.at ・ judged_values.reads.at は format すら無い | decisions（3-1 と同じ） | 直せば残る（JSON Pointer の pattern `^(/([^~/]|~[01])*)*$` を $defs に置いて使う） |
| 3-4 | 値だけで決まるのに x-prompt に残っている行：answer-for-ai（argv があれば answer を書かない）・ argv-no-shell（argv にシェルの記号を書かない）・ by が human なら agent を書かない ・ judged_by が ai なら delegate と model が要る ・ ULID の先頭は 0〜7。どれも壊した入力が0件で通った | decisions（3-1 と同じ） | 直せば残る（if ・ then ・ not ・ pattern で書く） |
| 3-5 | JSON Schema と同じことを x-prompt に重ねて書いている行：fixed-kind（const）・ five-values（enum）・ kind-only（enum）・ read-and-write（required と minItems）・ means-first（required）・ instance-path-and-kind と only-for-instances（根の if ・ then）。$defs/prompt が write に1件以上を求めるので、JSON Schema だけで決まる欄にも行を書かされる | decisions（3-1 と同じ）／ abstract.json $defs/prompt の write の minItems: 1 | 直せば残る（write を0件でもよいとするか、その欄には x-prompt を求めない） |
| 3-6 | 本体の x-extensions には9つの x- が登録されている。同じ x-extensions の行 no-abstract-names は「Abstract の9つの x- と同じ名前を使わない」。本体は自分で自分を検査するので、この行を本体に当てると violated になる。propertyNames で書いても、本体の自己検査が通らない | decisions「9つの x- を根の x-extensions に、Concrete と同じ形で登録する」 | 直せば残る（「本体の外のスキーマでは」と範囲を書く） |
| 3-7 | $defs/fields が "properties" と "$defs" を外している。fields の中のキーはインスタンスの欄の名前なので、欄の名前が "$defs" や "x-foo" だと、description と x-prompt が無くても通る（js.py で確認） | decisions「$defs/fields は $schema ・ 入れ物（properties ・ $defs）・ x- のキーを除く」 | 直せば残る（入れ物を外すのは Concrete の根の側で、fields の側ではない） |
| 3-8 | 本体の relationship は urn:semantic-agent-schema:base:annotations:v1 を $ref で指す。このファイルは案に無く、selfcheck.py が仮の中身を登録している。x-relationship を書いた Concrete を検査すると解決できない（推測：referencing が Unresolvable を出す。x-relationship を持つ Concrete が案に無いので、試していない） | Q6 #2「annotations（複数のファイルで使う。x-relationship ・ 判定する者）」 | 直せば残る |
| 3-9 | 本体は $vocabulary に urn:semantic-agent-schema:base:vocab:v1: true を書く。python の jsonschema は語彙を見ずに通した。語彙を見ない実装では、知らない語彙のスキーマを断るという効き目が出ない（推測：Rust の jsonschema の振る舞いは確かめていない） | Q2 #3「検証せずに合格と返す実装を止められる」 | 直せば残る（効く実装を適合の試験で確かめる） |
| 3-10 | concrete.json の x-instance-path は ".acdr/{no}-{slug}/acdr.json" だが、slug は properties に無い。本体の machine の行 level1-template に反するのに、selfcheck は0件と返す。machine の行を実行する仕組みがまだ無いので、案の例が自分の契約を破っても見つからない | abstract.json x-instance-path の level1-template「{…} の中が根の properties にある欄の名前」 | 直せば残る |
| 3-11 | 本体の then は kind と $schema の const を求めるが、id は求めない。ACDR のインスタンスは id を持たず no を持つ。x-relationship（指す先はインスタンスの id）で ACDR を指せない。supersedes も x-relationship を使わず、番号の pattern だけで書かれている | Q1 #35「書き方（種類はスキーマ、1件は id と kind、関係は x-relationship…）は基盤の共通の契約」 | 答えを変える必要がある（共通の契約が案に入っていない。id を求めるか、id の役をする欄を宣言させる） |
| 3-12 | change.replaces と judgment.target は、ほかの記録を指すのに x-relationship を持たない。存在の検査が x-prompt の行（by: machine）に残っている | Q7 #8「欄の x-relationship：この欄が指すインスタンスの種類…指す先が無ければ違反」 | 直せば残る |
| 3-13 | concrete.json の x-delegates.reviewer.answer に {"$ref": "#/$defs/review-body"} がある。知らないキーワードの中の $ref なので、実装ごとに解決するかどうかが違う。本体は x-extensions の中についてだけ、これを禁じている | abstract.json no-ref-into-extensions の why「知らないキーワードの中にある指す先は、JSON Schema の実装ごとに振る舞いが決まっていない」 | 直せば残る（同じ決まりを x-delegates にも当てる。答えの形は $defs に置き、名前で指す） |
| 3-14 | 本体の部品のうち、x-extensions の1件（schema ・ description ・ prompt ・ published）と run.kind ・ run.argv は x-prompt を持たない。Concrete の作者が書く単位なのに、本体の自己検査は $defs/shape が additionalProperties の中まで見ないので通る | decisions「本体の部品の欄はすべて description と x-prompt を持つ」 | 直せば残る |
| 3-15 | 構文の確かめ：本体 ・ change ・ judgment ・ ACDR の Concrete ・ インスタンス3つ ・ records.example は、どれも Draft 2020-12 で0件だった。$schema は自分の $id、$id は URN の pattern を満たす。$ref は、annotations（3-8）を除いて解決できた | ― | 問題なし |

## 4. A2 の2案で壊れる場面

| # | 案 | 壊れる場面 | 依拠した前提 | 重さ |
|---|---|---|---|---|
| 4-1 | X | 項目は「入口がその呼び出しで書いた対象」なので、入口を通らずに書いた Markdown ・ HTML ・ ソースは項目に入らない。どれも「記録に無い」になる（1-1） | user-instructions.md（1-1）／ A2 案 X | 答えを変える必要がある（宣言だけを受け取る入口を足す） |
| 4-2 | X | change.schema は field を必須にしている。A2 X は「欄（JSON の対象だけ）」。Markdown を対象にした項目は field が無いので断られる（js.py で確認） | A2 案 X | 直せば残る（JSON でない対象では field を書かない、を if ・ then で書く） |
| 4-3 | X | 検証で判定の記録を書く。判定の記録も変更の項目の対象（value_objects）なのに、判定の記録を書く呼び出しの意図と目的が決まっていない。項目に入れなければ「記録に無い」、入れれば検証のたびに変更の記録が増える | value_objects 変更の項目「対象（…判定の記録…）」／ I31 | 直せば残る（基盤が書く判定の記録は I31 の突き合わせから外す、と書く） |
| 4-4 | X | 1-11 のとおり、同じファイルを2つの目的が書くと、ファイル単位の確定で、片方の差分がもう片方の単位に入る | A3 (4) | 直せば残る |
| 4-5 | Y | 「未確定の変更に入る意図は1つまで」。同じ worktree で、2つの目的のセッション（または同じ目的の2つの working の任せる先）が並んで書くと、すぐに破れる。CLAUDE.md の運用は、呼ぶ Skill を並列に呼ぶ | A2 案 Y ／ decisions「任せる先のセッションは目的 ・ 親 ・ 役割が同じで1つ」（役割が違えば並ぶ） | 答えを変える必要がある（Y は worktree を目的ごとに分けることを前提にする。そうしないなら Y は成り立たない） |
| 4-6 | Y | AI の作業が未確定のうちに本人が承認すると、承認の変更（status）が2つ目の意図になる。AI の作業を先に確定し、承認を書いて、もう一度確定するという順番が要る。この順番は A1 に書かれていない | A1 ／ A2 案 Y | 直せば残る |
| 4-7 | Y | Y は「1単位＝1意図」を未確定の変更の上で不変条件にする。A1 は「1単位＝1意図は不変条件にせず」と言う。単位をまとめて取り込むと（squash など）、複数の意図が1つの単位に入り、意図ごとの欄を導けない。A8 は X の場合しか書いていない | A1「1単位＝1意図は不変条件にせず」／ A8 | 答えを変える必要がある（Y を採るなら、A1 と A8 も直す） |
| 4-8 | Y | 対象ごとの経緯（U13）を担う者が居ない（A7 が自分で認めている）。U13 は決着した使う場面で、外していない | Q8 #13「人の場面：…経緯（U13）」／ A7 | 答えを変える必要がある（Y を採るなら、A3 に「対象の履歴」を足すか、U13 を外す） |
| 4-9 | Y | I31 の両方向の突き合わせ（記録に無い ・ 変更の無い記録）と、I21 の欄ごとの書いた者が、項目なしでは「単位に入った意図1つが全部を書いた」としか言えない | I31 ・ I21 | 直せば残る |
| 4-10 | X と Y | 4-1 の場面では、Y のほうが強い。入口を通らない書き換えも、未確定の変更に入れば意図1つに結び付く。X は入口を足さないと利用者の指示を満たさない。Y は並行の作業（4-5）で満たさない | user-instructions.md | （所見へ） |

## 5. 外せるもの

| # | 外せるもの | 理由 | 依拠した前提 |
|---|---|---|---|
| 5-1 | JSON Schema と同じことを書いた x-prompt の行（3-5 の7件） | 値だけで決まるので、行に残すと2か所になる | decisions（3-1） |
| 5-2 | $defs/fields の "properties" と "$defs" の除外（3-7） | 入れ物ではない場所で外しているので、検査の穴にしかならない | decisions「$defs/fields は…入れ物（properties ・ $defs）…を除く」 |
| 5-3 | judgment の id（A4 を採るなら） | 対象 × rule で識別するなら要らない | A4 |
| 5-4 | ACDR の docs と acdr-inspect の Git の読み取り（2-17） | Abstract の変更のまとまりから導けるので、ACDR が自分で持つ必要が無い | user-instructions.md「ACDRはこれをより拡張して見やすくしたもの」 |
| 5-5 | 外せないと見たもの：オーケストレーターのセッションの行。セッションの索引の「親」。replaces | 利用者が「オーケストレーターは一つ」と決めている。I35 は親をたどる。I32 は書き換えずに置き換える | user-instructions.md「そのリポジトリ内でオーケストレーターは一つ」／ I35 ／ I32 |

## 6. 前提の外の指摘

| # | 壊れる場面 | 重さ |
|---|---|---|
| 6-1 | fixed_at_creation のトリガーは、値が同じでも UPDATE OF の列に入っていれば断る（db2.py：purpose_ref=purpose_ref で断られた）。全部の列を書き戻すアダプターだと、last_used_at を更新するだけで失敗する | 直せば残る（WHEN OLD.x IS NOT NEW.x を付ける） |
| 6-2 | 同じ ID のオーケストレーターを2つの端末で同時に再開すると、AI ツールの側で1つの会話に2つの書き手が並ぶ（推測：Claude Code の resume を2つの端末で同時に使ったときの振る舞いは確かめていない） | 確かめてから決める |
| 6-3 | A3 (1) の「未確定の後の姿には、まだ変更の仕組みに載っていない新しいファイルも含む」。Git の無視の一覧にあるファイル（ビルドの成果物や、リポジトリの中に置いた手元の DB）が入るかが決まっていない | 直せば残る |
| 6-4 | writer_kind は human と ai-agent だけ。基盤が自分で書く記録（承認のあとの status、判定の記録）の書いた者を表せない | 直せば残る |

## 7. 前提どうしの食い違い

| # | 食い違い | 優先順での扱い |
|---|---|---|
| G1 | user-instructions.md「…ソースコードに対しても差分を記録できるように」と、Q1 #8「基盤はソースを読まず」・ I36「見ていないもの（ソースコード ・ テストのコード）を示す」 | 利用者の指示が勝つ。Q1 #8 と I36 を直す必要がある（1-2） |
| G2 | Q1 #46「変更のまとまりは Abstract の一員の形」と、Q6 の答え「10個で組む」と #1 ・ #4 の一覧（変更のまとまりが無い） | ボードの中で食い違っている。一員を足すなら Q6 の数も変わる |
| G3 | Q1 #9 ・ Q6 #19 ・ #20 ・ domain-model-v2 naming は持ち主を core と書く。decisions は base | decisions が勝つ。ボードと v2 が直されていない |
| G4 | Q7 #17「Abstract の AI の rule は agent に予約した reviewer を使い、どの Concrete に割り当てるかは各 Concrete の x-delegates が決める」と、decisions「基盤自身も x-delegates に judge を持つ」 | decisions が勝つ。Q7 #17 ・ #18 と entities を直す必要がある |
| G5 | Q1 #22 ・ I26「1つの kind に向くビューは1つ」と、decisions「ビューは任意」・ A9「0か1つ」 | decisions が勝つ。I26 が直されていない |
| G6 | I44 ・ Q11 #9 ①「子の目的は親と同じ」と、decisions「不変条件①は任せる先のセッションにだけ当てる」。①には「ID ・ 目的 ・ 親 ・ 役割は変えない」も入るので、これも任せる先だけに当てるのかが読み分けられない（DDL はすべての行に当てている） | decisions が勝つが、どこまでを任せる先だけに当てるのかを1文で決める必要がある |
| G7 | 前半の decisions「rule の by: machine → program」と、後半「program は種類にしない」 | 後半が勝つ（README の決まり）。問題なし |

## 8. 見つからなかったもの

- Abstract の一員 session ・ session-change ・ 変更のまとまりの形 ・ annotations ・ protocol の call のスキーマ。どれも案に無い。
- 本人の承認を残す判定の記録の形（どの rule に対する判定か）。1-6 のとおり、どこにも決まっていない。
- 基盤の judge を、どの AI ツールのどのモデルで起動するかを決める場所。
- x-prompt の machine の行を実行する仕組み。selfcheck.py は JSON Schema だけを当てている。

---

## 所見

1. **答えを変える必要があるのは6か所。** 1-1 と 4-1（入口を通らない書き換えに意図を付ける操作が無い）、1-2 と G1（ソースを読まないという決着が、利用者の指示と食い違う）、1-4（セッションの中の承認と、MCP に見せない人の入口）、1-6（本人の承認を残す判定の記録の形が無い）、1-10（親が閉じた目的のセッションと、直されていない再開の規則）、3-11（1件は id と kind という共通の契約が案に入っていない）。
2. **A2 はどちらの案のままでも、利用者の指示を満たさない。** X は入口を通らない書き換えを拾えない（4-1）。Y は並行の作業で「意図は1つまで」が破れる（4-5）。さらに U13 の担い手を失い（4-8）、A1 ・ A8 と食い違う（4-7）。X に「書いたファイルと意図を宣言だけ受け取る入口」を足す形なら、4-1 は直り、4-5 ・ 4-7 ・ 4-8 は起きない。残るのは 4-3 と 4-4 で、どちらも直せば残る。これは案であり、反証役の推奨ではない。
3. **JSON Schema の能力は、値だけで決まる決まりの半分ほどしか使っていない。** 一番大きいのは入れ子の欄（3-1、$dynamicAnchor を使っていない）。ほかに if ・ then で書ける行が x-prompt に残っており（3-4）、JSON Schema と同じことを行に重ねる形を、$defs/prompt の minItems: 1 が強いている（3-5）。
4. **DB は、案の場面（dbcheck.py）は通るが、場面の外の行を多く通す**（2-8 ・ 2-9）。3つのセッションの種類の形（親 ・ 目的 ・ 役割の組）を CHECK とトリガーで閉じていない。
5. **ドメインモデル v2 は、decisions と answers-v2 の多くをまだ取り込んでいない**（2-5 ・ 2-6 ・ 2-7 ・ 2-13 ・ 2-14 ・ G3〜G6）。G1 の直し方の一覧は Git の語に限られており、こうした形のずれは入っていない。
