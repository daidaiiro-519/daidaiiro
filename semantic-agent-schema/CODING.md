# semantic-agent-schema の Go の規約

semantic-agent-schema の Go のコードと試験を書くときに守る決まりである。どの決まりにも出どころを付ける。

## 1 この文書の使い方

### 1.1 誰が・いつ読むか

| 読む人 | いつ | 何をする |
|---|---|---|
| Go のコードを書く AI | コードを書く前に毎回 | 2・4・5 の決まりに従って書く |
| 試験を書く AI | 試験を書く前に毎回 | 3 の決まりに従って書く |
| レビューする AI と人 | 差分を見るとき | 違反を決まりの ID（`K01` など）で指す |

### 1.2 決まりの出どころの種類

| 種類 | 何か | 出どころの書き方 |
|---|---|---|
| Go 公式 | Go チームの文書（go.dev ・ golang/go ・ golang/wiki ・ golang/tools）と、go 1.27.2 の `go doc` の出力 | 原文のファイルと行。引用は英語のまま |
| Google | Google Go Style Guide（guide ・ decisions ・ best-practices）。Go チームの文書ではない | 原文のファイルと行。引用は英語のまま |
| 使うライブラリ | このプロジェクトが使うライブラリの doc（exhaustive） | 原文のファイルと行。引用は英語のまま |
| このプロジェクト | 本質（`docs/essence/essence.json`）・抽象のページ ・ 具体のページ ・ 利用者の指示 | 本質の ID（`W7` など）、ページの行の ID（`D1` など）、ID の無い行は節と行の名前、利用者の指示は指示の日 |

### 1.3 食い違ったときの決め方

| 食い違い | 採るもの |
|---|---|
| Go 公式と Google | Go 公式。Google は Go 公式に無いことだけを足す |
| Go 公式・Google と、このプロジェクト | 1.4 の表に挙げた所だけ、表の「採るもの」に従う |
| 1.4 の表に無い食い違いを見つけた | コードを書く前に、食い違いの箇所を人へ伝える |

### 1.4 一次情報どうしが食い違う所

| 箇所 | 一方 | もう一方 | 採るもの | 理由 |
|---|---|---|---|---|
| 値オブジェクトの空の値 | go-proverbs.github.io L18「Make the zero value useful.」 | 具体のページ 8節「値オブジェクト」：型を非公開にし、作る関数だけを出す | このプロジェクト（`K70`） | 空の値では、path が working tree の外を指さない（`D1`）などの不変条件を守れない |
| 4つの `application` の名前 | package-names.md L258–260：一緒に使う package は別の名前にする | 具体のページ 8節「package の名前」：層は `domain` ・ `application` の名前で示す | このプロジェクト（`K03`）。衝突する所だけ別名にする（`K10`・`K11`） | 業務領域は import path の1段目が示し、層は package の名前が示す |
| 試験の代役 | CodeReviewComments.md L337–344：使う側の試験で `fakeThinger` を作る例。decisions.md L2336–2339：本物で試せないときは代役を認める | 具体のページ 2節 ・ 9節「環境」：代役を作らず、本物のアダプターで試す | このプロジェクト（`T50`） | アダプターの相手（ファイル ・ SQLite ・ Git）は、どれも一時ディレクトリで本物を使える。decisions.md の「本物で試せないとき」に当たらない |
| `import _` の置き場所 | CodeReviewComments.md L190–192 ・ decisions.md L953–955：`main` か試験だけで使う | 具体のページ 1節「依存の決まり」：`modernc.org/sqlite` を使ってよいのは `outbound/recordstore` だけ | このプロジェクト（`K122`） | decisions.md L963–966 の理由は、試験で別の import に差し替えられるようにすること。この試験は差し替えず、本物のドライバーを使う |
| 試験とページの行の結び付け | 具体のページ 9節「ページの行の id」・「完成の条件」5：試験がページの行の ID を持ち、`go test` が対応を照らす | 利用者の指示（2026-11-05）：識別子は古くなるので、コメント ・ 試験に書かない | 利用者の指示（`K59` ・ `T09` ・ `T40`）。具体のページ 9節の2行を直す | 識別子の対応は、ページが変わるたびに古くなり、保てない |
| 構造の比べ方 | TestComments.md L115–118：`cmp` を使う | 具体のページ 3節「試験の中だけ」：`github.com/google/go-cmp` が無い | Go 公式（`T13`）。具体のページ 3節と 1節の表に `github.com/google/go-cmp/cmp` を足す | `reflect.DeepEqual` は非公開のフィールドに左右される（TestComments.md L126–127） |

### 1.5 原文の一覧

原文は、この文書と同じフォルダの `sources/` に置く。`go doc` の出力は go 1.27.2 で書き出し、exhaustive の doc は Go のモジュールキャッシュのファイルを読んだ。ほかは fact-check で取得した。引用は、どれも原文の文字列で照合した。

| 文書 | 種類 | ファイル | 取得した日 |
|---|---|---|---|
| Effective Go | Go 公式 | `raw.githubusercontent.com_golang_website_master__content_doc_effective_go.html` | 2026-11-05 |
| Go Code Review Comments | Go 公式 | `raw.githubusercontent.com_golang_wiki_master_CodeReviewComments.md` | 2026-11-05 |
| Go Test Comments | Go 公式 | `raw.githubusercontent.com_golang_wiki_master_TestComments.md` | 2026-11-05 |
| Go Doc Comments | Go 公式 | `raw.githubusercontent.com_golang_website_master__content_doc_comment.md` | 2026-11-05 |
| Organizing a Go module | Go 公式 | `raw.githubusercontent.com_golang_website_master__content_doc_modules_layout.md` | 2026-11-05 |
| Package names | Go 公式 | `raw.githubusercontent.com_golang_website_master__content_blog_package-names.md` | 2026-11-05 |
| Go 1 compatibility | Go 公式 | `raw.githubusercontent.com_golang_website_master__content_doc_go1compat.html` | 2026-11-05 |
| testing の package の doc | Go 公式 | `raw.githubusercontent.com_golang_go_master_src_testing_testing.go` | 2026-11-05 |
| go コマンドの doc | Go 公式 | `raw.githubusercontent.com_golang_go_master_src_cmd_go_alldocs.go` | 2026-11-05 |
| deadcode の doc | Go 公式 | `raw.githubusercontent.com_golang_tools_master_cmd_deadcode_doc.go` | 2026-11-05 |
| `go doc` の出力（go 1.27.2） | Go 公式 | `go-doc_errors.Join.txt` ・ `go-doc_testing.T.Setenv.txt` ・ `go-doc_testing.T.Chdir.txt` | 2026-11-05 |
| Go Proverbs | Go 公式 | `go-proverbs.github.io` | 2026-11-04 |
| Google Go Style Guide：guide | Google | `raw.githubusercontent.com_google_styleguide_gh-pages_go_guide.md` | 2026-11-05 |
| Google Go Style Guide：decisions | Google | `raw.githubusercontent.com_google_styleguide_gh-pages_go_decisions.md` | 2026-11-05 |
| Google Go Style Guide：best-practices | Google | `raw.githubusercontent.com_google_styleguide_gh-pages_go_best-practices.md` | 2026-11-05 |
| exhaustive v0.17.1 の doc | 使うライブラリ | モジュールキャッシュの `github.com/nishanths/exhaustive@v0.17.1/passes/exhaustive/doc.go` ・ `passes/enumerated/doc.go` | 2026-11-05 |

下の表では、ファイルを短い名前（`CodeReviewComments.md` ・ `decisions.md` ・ `testing.go` など）で書く。Go のソースのコメントからの引用は、行の頭の `//` も原文のまま残す。

## 2 コーディング規約

### 2.1 名前

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K01 | package の名前は、短い小文字の1語にする。`_` と mixedCaps を使わない | Go 公式 | effective_go.html L200–202 | By convention, packages are given lower case, single-word names; there should be no need for underscores or mixedCaps. |
| K02 | `util` ・ `common` ・ `misc` ・ `api` ・ `types` ・ `interfaces` という package を作らない | Go 公式 | CodeReviewComments.md L528 | Avoid meaningless package names like util, common, misc, api, types, and interfaces. |
| K03 | 層は package の名前 `domain` ・ `application` で示す | このプロジェクト | 具体のページ 8節「package の名前」 | |
| K04 | 公開の名前に package の名前を繰り返さない（`files.Documents` にし、`files.FileDocuments` にしない） | Go 公式 | CodeReviewComments.md L524–525 | All references to names in your package will be done using the package name, so you can omit that name from the identifiers. |
| K05 | 2語以上の名前は MixedCaps で書く。非公開の定数も同じ | Go 公式 | CodeReviewComments.md L391 | For example an unexported constant is `maxLength` not `MaxLength` or `MAX_LENGTH`. |
| K06 | 頭字語は、全部を大文字か全部を小文字にする（`SchemaID` ・ `AbstractID` ・ `URL` ・ `JSON`） | Go 公式 | CodeReviewComments.md L309 | so write "appID" instead of "appId". |
| K07 | getter に `Get` を付けない（`Counts` にし、`GetCounts` にしない） | Google | decisions.md L258–260 | Prefer starting the name with the noun directly, for example use `Counts` over `GetCounts`. |
| K08 | レシーバーの名前は、型の1〜2文字の略にする。`me` ・ `this` ・ `self` を使わない | Go 公式 | CodeReviewComments.md L537 | Don't use generic names such as "me", "this" or "self", identifiers typical of object-oriented languages that gives the method a special meaning. |
| K09 | 同じ型のレシーバーは、どのメソッドでも同じ名前にする | Go 公式 | CodeReviewComments.md L537 | Be consistent, too: if you call the receiver "c" in one method, don't call it "cl" in another. |
| K10 | import に別名を付けるのは、名前が衝突するときだけにする | Go 公式 | CodeReviewComments.md L166 | Avoid renaming imports except to avoid a name collision; |
| K11 | 別名は、同じ名前の `application` を2つ以上使う入口とアダプターの中だけで付ける | このプロジェクト | 具体のページ 8節「import の別名」 | |
| K12 | 同じ package の別名は、どのファイルでも同じにする（業務領域の名前 `authoring` ・ `rendering` ・ `search` ・ `record`） | Google | decisions.md L869–870 | Try to be [consistent](guide#consistency) by always using the same local name for the same imported package. |
| K13 | 変数の名前は、宣言から遠くで使うものほど長くし、狭い範囲の変数は短くする | Go 公式 | CodeReviewComments.md L588 | The basic rule: the further from its declaration that a name is used, the more descriptive the name must be. |

### 2.2 package の分け方と internal

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K20 | Go モジュールは1つにする | このプロジェクト | 具体のページ 1節「構成：1つの Go モジュール」 | |
| K21 | `main` 以外の package は、すべて `internal/` の下に置く | Go 公式 | modules/layout.md L182 | It's recommended to keep packages in `internal` as much as possible. |
| K22 | `internal/` の1段目は業務領域（`authoring` ・ `rendering` ・ `search` ・ `record`）、2段目は層（`domain` ・ `application`）にする | このプロジェクト | 具体のページ 1節 | |
| K23 | アダプターは `internal/adapters/inbound/` と `internal/adapters/outbound/` の下に、1つずつ package にして置く | このプロジェクト | 具体のページ 1節 | |
| K24 | アダプターの package の名前は、扱う内容の名前にする。使うライブラリと同じ名前にしない | このプロジェクト | 具体のページ 8節「アダプターの名前」 | |
| K25 | よく使う標準ライブラリの package と同じ名前を付けない | Go 公式 | package-names.md L261–262 | avoid using the same name as popular standard packages like `io` or `http`. |
| K26 | `cmd/semantic-agent-schema/main.go` は、working tree と操作の記録のファイルを決め、アダプターを作って渡すことだけをする | このプロジェクト | 具体のページ 1節 | |
| K27 | 公開された言葉（`v1/*.json`）は `internal/publishedlanguage/` に置き、`go:embed` でバイナリに埋め込む | このプロジェクト | 具体のページ 5節 | |

### 2.3 interface は使う側に置く

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K30 | ポートの interface は、使う側の application の `ports.go` に宣言する | Go 公式 | CodeReviewComments.md L315–316 | Go interfaces generally belong in the package that uses values of the interface type, not the package that implements those values. |
| K31 | ポートは、そのユースケースが使うメソッドだけを持つ | Google | decisions.md L2346–2347 | The consumer of the interface should define it (not the package implementing the interface), ensuring it includes only the methods they actually use. |
| K32 | アダプターの作る関数は、具体の型（構造体へのポインタ）を返す | Go 公式 | CodeReviewComments.md L316–318 | The implementing package should return concrete (usually pointer or struct) types |
| K33 | 試験の代役のための interface を作らない | Go 公式 | CodeReviewComments.md L321 | Do not define interfaces on the implementor side of an API "for mocking"; |
| K34 | 使う所ができる前に interface を作らない | Go 公式 | CodeReviewComments.md L325 | Do not define interfaces before they are used |
| K35 | ポートは外部（ファイル ・ 操作の記録 ・ ライブラリ）の分だけ作る | このプロジェクト | 具体のページ 2節 | |
| K36 | domain の中で interface を使うのは、値オブジェクトを外から作らせないためだけにする | Google | decisions.md L2356–2357 | Sometimes returning an interface is acceptable for encapsulation (e.g., `error` interface) |

### 2.4 エラー

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K40 | 失敗しうる関数は、`error` を最後の戻り値で返す | Google | decisions.md L1030–1031 | Use `error` to signal that a function can fail. By convention, `error` is the last result parameter. |
| K41 | 戻り値の型は `error` にし、`*domain.Failure` などの具体の型にしない。ただし、必ず値を返し nil を返さない作る関数（`domain.Fail` など）は、具体の型 `*Failure` を返してよい | Google | decisions.md L1057 | Exported functions that return errors should return them using the `error` type. |
| K42 | 違反は domain の値 `*domain.Failure` で作る。`Failure` は code ・ path ・ instance location ・ absolute keyword location ・ message ・ Line（テンプレートの行）・ Argument（違反した JSON の引数名）を持つ | このプロジェクト | 具体のページ 8節「エラー」 ・ 抽象のページ 9節 | |
| K43 | `Failure` は `error` を実装する | Go 公式 | effective_go.html L3145–3147 | A library writer is free to implement this interface with a richer model under the covers, making it possible not only to see the error but also to provide some context. |
| K44 | 呼ぶ側は、`Failure` を `errors.As` で取り出し、code で分ける。error の文で分けない | Google | best-practices.md L813–814 | give the error value structure so that this can be done programmatically rather than having the caller perform string matching. |
| K45 | 1回の処理で違反が2つ以上あるときは、`errors.Join` で1つの `error` にする。cli は `Unwrap() []error` で1つずつ取り出す | Go 公式 | go-doc_errors.Join.txt L9 | A non-nil error returned by Join implements the Unwrap() []error method. |
| K46 | 外の error を包むときは `fmt.Errorf` の `%w` を使い、`%w` を文の末尾に置く | Google | best-practices.md L1117 | Prefer to place `%w` at the end of an error string |
| K47 | 包むときの文に「失敗した」「できなかった」を足さない | Google | best-practices.md L960–961 | Stop words such as "failed to", "couldn't", or "unable to" add no useful information |
| K48 | error を `_` で捨てない | Go 公式 | CodeReviewComments.md L162 | Do not discard errors using `_` variables. |
| K49 | 捨ててよい error（閉じるときの error など）を捨てるときは、なぜ安全かを同じ行のコメントに書く | Google | decisions.md L1129–1131 | an accompanying comment should explain why this is safe. |
| K50 | Go の error の文は小文字で始め、句読点で終えない | Go 公式 | CodeReviewComments.md L135 | Error strings should not be capitalized (unless beginning with proper nouns or acronyms) or end with punctuation |
| K51 | error を先に処理して return し、正常の流れの字下げを浅くする | Go 公式 | CodeReviewComments.md L263 | Try to keep the normal code path at a minimal indentation, and indent the error handling, dealing with it first. |
| K52 | panic を使わない | Go 公式 | CodeReviewComments.md L131 | Don't use panic for normal error handling. Use error and multiple return values. |
| K53 | `Must` の関数（`regexp.MustCompile` など）は、package の変数の初期化だけで呼ぶ | Google | decisions.md L2127–2129 | In general, they should only be called early on program startup, not on things like user input where normal Go error handling is preferred. |
| K54 | panic を `recover` で止めない | Google | best-practices.md L1363 | resist the temptation to recover panics to avoid crashes |

### 2.5 コメントと doc

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K60 | 公開の名前すべてと、自明でない非公開の型と関数に doc コメントを書く | Go 公式 | CodeReviewComments.md L127 | All top-level, exported names should have doc comments, as should non-trivial unexported type or function declarations. |
| K61 | doc コメントは、説明する名前で始め、句点で終える（`// Validate は、検証のユースケースの手続き。`） | Go 公式 | CodeReviewComments.md L25 | Comments should begin with the name of the thing being described and end in a period |
| K66 | 型の doc コメントには、その型が何を表すかを書く | Go 公式 | comment.md L152 | A type's doc comment should explain what each instance of that type represents or provides. |
| K67 | 関数の doc コメントには、何を返すか（副作用の関数は何をするか）を書く | Go 公式 | comment.md L240–241 | A function's doc comment should explain what the function returns or, for functions called for side effects, what it does. |
| K68 | 型・関数・package のコメントには、それが設計の中で何を担うかを、具体モデリング概念の言葉（集約 ・ 値オブジェクト ・ ドメインサービス ・ ポート ・ アダプター ・ 機械的な検証 ・ 意味的な検証 ・ 変更点 ・ ログイベント ・ instruction と next など）で書く。抽象モデリング概念を実装としてどう具体にしたかが読めるようにする | このプロジェクト | 利用者の指示（2026-11-05） | |
| K69 | コメントの設計の言葉は、部品に名前を付けて書く | Go 公式 | go-proverbs.github.io L30 | Design the architecture, name the components, document the details. |
| K59 | コメントに、ページの行の識別子（`UC3` ・ `D1` ・ `S4` ・ `W5` など）を書かない | このプロジェクト | 利用者の指示（2026-11-05） | |
| K62 | package コメントは `package` 句の直前に、空行を空けずに書く | Go 公式 | CodeReviewComments.md L453 | Package comments, like all comments to be presented by godoc, must appear adjacent to the package clause, with no blank line. |
| K63 | package コメントは「Package <名前> は、」で始める。`main` は「Command semantic-agent-schema は、」で始める | Go 公式 | comment.md L64–65 | For a package comment, that means the [first sentence](/pkg/go/doc/#Package.Synopsis) begins with “Package <name>”. |
| K64 | package コメントは、package の中の1つのファイルにだけ書く | Go 公式 | comment.md L67 | For multi-file packages, the package comment should only be in one source file. |
| K65 | コメントの語は、5 の用語に従う | このプロジェクト | 抽象のページ 4節 | |

### 2.6 値オブジェクトと空の値

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K70 | 値オブジェクトの型は非公開にし、外へは作る関数と interface だけを出す | このプロジェクト | 具体のページ 8節「値オブジェクト」 | |
| K71 | 値オブジェクトの interface は、非公開のメソッドを1つ持つ | このプロジェクト | 具体のページ 8節「値オブジェクト」 | |
| K72 | 作る関数は不変条件を検証し、違反なら `error` を返す | このプロジェクト | 具体のページ 8節「値オブジェクト」 | |
| K73 | 抽象のページ 7節の値オブジェクト（path ・ ハッシュ ・ JSON Pointer ・ change ・ Concrete ・ rule ・ judgment ・ prompt）は、K70〜K72 の形で作る | このプロジェクト | 抽象のページ 7節「値オブジェクト」 | |
| K74 | 不変条件を持たない型（verdict ごとの件数など）は、空の値でそのまま使える形にする | Go 公式 | go-proverbs.github.io L18 | Make the zero value useful. |
| K75 | domain は、JSON Schema で書けない不変条件だけを持つ。JSON Schema で書ける検証を domain に書かない | このプロジェクト | 抽象のページ 7節 ・ 本質 `W5` ・ `W10` | |
| K76 | ほかの package の構造体の値は、フィールドの名前を書いて作る | Go 公式 | go1compat.html L94–95 | We therefore recommend that composite literals whose type is defined in a separate package should use the keyed notation. |

### 2.7 区分

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K80 | 閉じた一覧（verdict ・ code など）は、非公開の名前付きの型（`string`）と公開の定数で書く | このプロジェクト | 具体のページ 8節「区分」 | |
| K81 | 区分の型と定数は、同じ `const` のブロックで宣言する | 使うライブラリ | enumerated/doc.go L32–34 | An enumerated type and // the constants of the type must be declared in the same [block] to // be considered by the analysis. |
| K82 | 区分で分ける `switch` は、すべての値を `case` に書く。`default` で網羅とみなさない | 使うライブラリ | exhaustive/doc.go L54 | The presence of a default case will not satisfy missing cases. |
| K83 | 網羅は、`architecture_test.go` の `TestExhaustive` が `checker.Analyze` で `exhaustive.Analyzer` を走らせて確かめる。`-d` は付けない | このプロジェクト | 具体のページ 8節「区分」 | |

### 2.8 context

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K90 | I/O をするポートのメソッドとアダプターのメソッドは、`ctx context.Context` を第1引数で受け取る | Go 公式 | CodeReviewComments.md L45 | Most functions that use a Context should accept it as their first parameter: |
| K91 | `context.Background()` は `main` だけで作る | Google | decisions.md L2848–2851 | In other entrypoint functions (see below for examples of such functions), use [`context.Background()`]. |
| K92 | 構造体に `Context` のフィールドを持たない | Go 公式 | CodeReviewComments.md L56 | Don't add a Context member to a struct type; |
| K93 | domain は `context` を受け取らない。ポートを呼ばず、I/O をしない | このプロジェクト | 抽象のページ 7節 | |

### 2.9 並行

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K100 | goroutine を起こさず、同期の関数で書く | Go 公式 | CodeReviewComments.md L555 | Prefer synchronous functions - functions which return their results directly or finish any callbacks or channel ops before returning - over asynchronous ones. |
| K101 | 書く操作の排他は、`Records.WithLock`（`BEGIN IMMEDIATE`）の中で行う | このプロジェクト | 具体のページ 6節 | |

### 2.10 依存の向き

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K110 | package の依存は、具体のページ 1節「依存の決まり」の表にあるものだけにする | このプロジェクト | 具体のページ 1節「依存の決まり」 | |
| K111 | `authoring/domain` は、ほかの package を使わない。標準ライブラリだけを使う | このプロジェクト | 具体のページ 1節「依存の決まり」 | |
| K112 | `*/application` は `authoring/domain` だけを使う。ほかの application を使わない。ただし application の試験は、外の試験 package（`<package>_test`）として、本物のアダプター（`adapters/outbound/*`）と `publishedlanguage` を import してよい | このプロジェクト | 具体のページ 1節「依存の決まり」 | |
| K113 | アダプターどうしは使い合わない | このプロジェクト | 具体のページ 1節 | |
| K114 | package を足したら、具体のページ 1節の表と `architecture_test.go` の `rules` に同じ行を足す。`TestDependencies` は、表に無い package を落とす | このプロジェクト | 具体のページ 1節「依存の決まり」 | |

### 2.11 外のライブラリを使ってよい package

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K120 | 外のライブラリは、下の表の package だけで使う。試験の中だけで使ってよいのは `golang.org/x/tools` ・ `github.com/nishanths/exhaustive` ・ `github.com/google/go-cmp/cmp` ・ `github.com/santhosh-tekuri/jsonschema/v6`（出力を `event.schema.json` で検証するため） | このプロジェクト | 具体のページ 1節 ・ 3節 | |
| K121 | 標準ライブラリで足りるときは、外のライブラリを足さない | Google | guide.md L186–187 | Where there are several ways to express the same idea, prefer the one that uses the most standard tools. |
| K122 | `import _` は、`outbound/recordstore` の `modernc.org/sqlite` と、`//go:embed` を使うファイルの `embed` だけにする | このプロジェクト | 具体のページ 1節（1.4 の表） | |

| package | 使ってよい外のライブラリ |
|---|---|
| `adapters/inbound/cli` | `github.com/spf13/pflag` |
| `outbound/schemacheck` | `github.com/santhosh-tekuri/jsonschema/v6` |
| `outbound/patching` | `github.com/evanphx/json-patch/v5` |
| `outbound/recordstore` | `modernc.org/sqlite` |
| `outbound/templates` | `github.com/tdewolff/parse/v2` ・ `github.com/tdewolff/parse/v2/js`（テンプレートの `<script>` の JS を構文木にする。純粋な Go で、依存はテストの外に無い） |
| 試験の中だけ | `golang.org/x/tools` ・ `github.com/nishanths/exhaustive` ・ `github.com/google/go-cmp/cmp` ・ `github.com/santhosh-tekuri/jsonschema/v6`（出力を `event.schema.json` で検証するため） |
| ほかの package | 無し（標準ライブラリだけ） |

### 2.12 ファイル名と整形

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K130 | application のファイルは、ユースケースごとの `usecase_<名前>.go` と、ポートの `ports.go` にする | このプロジェクト | 具体のページ 8節「ファイルの名前」 | |
| K131 | domain とアダプターのファイルは、扱う内容の名前にする（`document.go` ・ `judgment.go` など） | このプロジェクト | 具体のページ 8節「ファイルの名前」 | |
| K132 | 試験のファイルは、名前を `_test.go` で終える | Go 公式 | testing.go L18 | and give that file a name ending in "_test.go". |
| K133 | すべてのファイルを `gofmt` の出力のままにする | Go 公式 | CodeReviewComments.md L19 | Run [gofmt](https://pkg.go.dev/cmd/gofmt/) on your code to automatically fix the majority of mechanical style issues. |

### 2.13 Concrete に書いてよい format の名前

`schemacheck` は `Compiler.AssertFormat` で、`format` を注釈でなく違反として検証する。ライブラリは知らない名前の `format` を黙って飛ばすので、Abstract（`abstract.schema.json` の `$defs/keywords`）が、下の名前だけを `enum` で許す。名前は santhosh-tekuri/jsonschema v6.0.3 の `format.go` の `formats`（L20–39）と、`objcompiler.go` の `regex`（L186–192）で確かめた。

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K140 | Concrete の `format` には、`date` ・ `date-time` ・ `duration` ・ `email` ・ `hostname` ・ `ipv4` ・ `ipv6` ・ `iri` ・ `iri-reference` ・ `json-pointer` ・ `period` ・ `regex` ・ `relative-json-pointer` ・ `semver` ・ `time` ・ `uri` ・ `uri-reference` ・ `uri-template` ・ `uuid` だけを書く | 使うライブラリ | format.go L20–39 ・ objcompiler.go L186–192 | `var formats = map[string]*Format{` |
| K141 | ライブラリを上げたら、`format.go` の `formats` と Abstract の `enum` を照らし直す | このプロジェクト | 利用者の指示（2026-11-05） | |
| K142 | Concrete に `$ref` を書かない。Abstract が `"$ref": false` で違反にする。`$ref` の先を引く処理を作らない | このプロジェクト | 利用者の指示（2026-11-05） | |

### 2.14 描画の参照の静的な照合

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K150 | 描く前に、テンプレートが読む `.Value` と `.Tokens` の欄を、Concrete の properties ・ items とデザイントークンに照らす（ドメインサービス「レンダラー」）。構文木は `text/template/parse`（html/template が使う標準の構文木）で読み、`{{range}}` ・ `{{with}}` の中の `.` と変数もたどる | このプロジェクト | 利用者の指示（2026-11-05）「機械的な検証ができる時はしてくださいね」 | |
| K151 | テンプレートの `<script>` の中（type が無いか JavaScript ・ module のもの）は `tdewolff/parse` の js で読み、`{{.Value…}}` ・ `{{.Tokens…}}` を初期値にした変数から始まるメンバーの参照を照らす。構文の誤りは `template_invalid` | このプロジェクト | 利用者の指示（2026-11-05） | |
| K152 | JS で追うのは、変数の初期値（`const doc = {{.Value}}`、`const x = doc.a`）・ `for (const d of doc.items)` の要素 ・ 名前か数のリテラルで引くメンバー（`doc.a` ・ `doc["a"]` ・ `doc.items[0]`）だけ。変数を作り直す ・ 計算した名前で引く ・ 関数の戻り値をたどる ・ 分割代入 ・ 名前の隠れ（同じ名前の別の変数）は追わない。JS の構文木は位置を持たないので、行は読んだ式の文字列をテンプレートの中で探して出す | このプロジェクト | 利用者の指示（2026-11-05） | |
| K153 | properties を持たないサブスキーマ（配列 ・ 文字列など）の下の名前は、組み込みの読み方（`length` など）があるので照らさない。値が無い任意の欄は静的な照合では通り、描くときの `missingkey=error` で止まる | このプロジェクト | 利用者の指示（2026-11-05） | |

### 2.15 デザイントークンの契約

デザインシステムの `tokens` は、W3C Design Tokens Community Group の Design Tokens Format Module 2025.10（https://www.designtokens.org/TR/2025.10/format/ 、色は Color module 2025.10 https://www.designtokens.org/TR/2025.10/color/）の形にする。識別子は原文の文字列で照合した。

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K160 | トークンは `$value` を持つオブジェクト、グループは `$value` を持たないオブジェクト。性質は `$type` ・ `$description` ・ `$deprecated` ・ `$extensions`、グループの根のトークンは `$root`。名前は `$` で始めず、`{` ・ `}` ・ `.` を含まない | Design Tokens Format Module | format 5.1 ・ 5.1.1 ・ 6.1 ・ 6.2 | An object with a $value property is a token. |
| K161 | 型は、値の形を機械的に確かめられる `color` ・ `dimension` ・ `fontFamily` ・ `fontWeight` ・ `duration` ・ `cubicBezier` ・ `number` だけ。合成の型（shadow など）は扱わない。値の形は `design-tokens.schema.json` の `$defs` の型ごとの形で確かめる（color は colorSpace の一覧 ・ 成分3つが数か none ・ alpha が 0〜1 ・ hex が6桁。色空間ごとの成分の範囲は確かめない。dimension の unit は px か rem、duration の unit は ms か s、fontWeight は 1〜1000 か原文の名前、cubicBezier の x は 0〜1） | Design Tokens Format Module | format 8 ・ color 4.1 ・ 4.2 | |
| K162 | 別名は `{group.token}` の形だけを扱う。`$ref`（JSON Pointer）・ `$extends` は扱わない（JSON Schema が拒む） | このプロジェクト | 利用者の指示（2026-11-05） | |
| K163 | JSON Schema で書けない決まり（グループから受け継ぐ `$type` ・ 別名の先が在るトークンか ・ 輪になっていないか ・ 別名の先の型が合うか ・ 型が決まらないトークン）は、ドメインサービス「レンダラー」の `ResolveTokens` が確かめる。別名の違反は `token_alias_invalid`、型が決まらないトークンと、受け継いだ型の値の形の違反は `schema_violation` | このプロジェクト | 利用者の指示（2026-11-05） | |
| K164 | テンプレートには、別名を解いた値の木（グループは名前から子へ、トークンは値。`$` で始まる性質は除く）を `.Tokens` で渡す。テンプレートに `$value` を書かせない。テンプレートと JS の参照は、この木に照らす | このプロジェクト | 利用者の指示（2026-11-05） | |
| K165 | 描画は翻訳ツールの役を担い、トークンの型から CSS の文字列 `css` を作って、別名を解いた値の木の各トークンに足す（format 3.4「A translation tool might reference a token's type to convert the source value into the correct platform-specific format.」）。dimension ・ duration は数と単位、color は hex（alpha が 1 でないか hex が無いときは CSS Color 4 の関数：hsl ・ hwb ・ lab ・ lch ・ oklab ・ oklch はその名前の関数、ほかは `color(<色空間> …)`、alpha は ` / a`）、fontFamily は空白を含む名前を引用符で囲んで `, ` でつなぎ、fontWeight は数（名前は表の数）、cubicBezier は `cubic-bezier(a, b, c, d)`、number は数。値がオブジェクトの型（color ・ dimension ・ duration）はその値に `css` を足し、ほかの型は `{"value": 値, "css": …}` にする | Design Tokens Format Module | format 3.4 ・ 8 | A translation tool might reference a token's type to convert the source value into the correct platform-specific format. |

### 2.16 拡張項目を書いてよい所

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| K170 | Concrete の `x-prompt` は、値を書く所（ルート ・ `properties` ・ `patternProperties` ・ `additionalProperties` ・ `unevaluatedProperties` ・ `items` ・ `prefixItems` ・ `unevaluatedItems` のサブスキーマ）と、分岐の枝（`oneOf` ・ `anyOf` の各枝 ・ `then` ・ `else` ・ `dependentSchemas` の枝）に必ず書く。`const` か要素1つの `enum` だけで値が決まる所は、値を書く所でも省ける。`if` ・ `allOf` の枝 ・ `not` ・ `contains` ・ `propertyNames` には書かなくてよい。これは `abstract.schema.json` が JSON Schema（`writing` ・ `valueSchema` ・ `branchSchema`）で課し、ドメインのコードで同じことを確かめない。`$defs` は Concrete に書かせない（`$ref` を禁じているので使われない） | このプロジェクト | 利用者の指示（2026-11-06） | |
| K171 | 値を書く所のサブスキーマは `true` にせずオブジェクトにする。`type` が `object` なら `additionalProperties` か `unevaluatedProperties`（`false` かサブスキーマ）を、`array` なら `items` ・ `prefixItems` ・ `unevaluatedItems` のどれかを書く。分岐（`then` ・ `oneOf` の枝など）の中でキーを定義するときは、`additionalProperties: false` がそのキーを弾くので、`unevaluatedProperties: false` を使う | このプロジェクト | 利用者の指示（2026-11-06） | |
| K172 | 判定で当たる `x-prompt`（スキーマ検査アダプターの `Evaluate`）は、JSON Schema の注釈と同じに集める。拡張項目の呼び出しの中で、そのサブスキーマの検証をやり直し、通ったときだけ組にする。通った `anyOf` の枝は、ライブラリが打ち切った枝も検証し直して、すべて組にする。ドメインは組を JSON Schema の実装から受け取るだけで、スキーマをたどり直さない | このプロジェクト | JSON Schema 2020-12 Core 10.2.1.2 ・ 7.7 | all subschemas MUST be examined so that annotations are collected from each subschema that validates successfully ・ annotations are not retained for failing schemas |

## 3 テスト規約

### 3.1 試験の置き場所と入口

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T01 | 試験は外の package（`<package の名前>_test`）に置き、公開の名前だけを使う | Go 公式 | testing.go L39–41 | must be imported explicitly and only its exported identifiers may be used. |
| T02 | 試験は、バイナリ ・ application の手続き ・ domain の公開の関数から実行する。非公開の関数を直に試さない | このプロジェクト | 具体のページ 9節「試験の入口」 | |
| T03 | 試験の入力のファイルは、その package の `testdata/` に置く | Go 公式 | alldocs.go L1644–1645 | The go tool will ignore a directory named "testdata", making it available // to hold ancillary data needed by the tests. |

### 3.2 テーブル駆動

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T05 | 同じ確かめ方の場合が並ぶときは、表にして1つの試験関数で回す | Go 公式 | TestComments.md L248–250 | tests should be used whenever many different test cases can be tested using similar testing logic |
| T06 | 確かめ方が違う場合は、別の試験関数にする（違反を返す場合の表と、返さない場合の表を分ける） | Go 公式 | TestComments.md L255–256 | When some test cases need to be checked using different logic from other test cases, it is more appropriate to write multiple test functions. |
| T07 | 表の行に、何を確かめるかの名前を持たせ、`t.Run` の名前にする。行の番号で代えない | Go 公式 | TestComments.md L171 | Do not use the index of the test in the test table as a substitute for naming your tests or printing the inputs. |
| T08 | サブテストは、ほかの場合の実行に頼らない | Google | decisions.md L3488–3490 | Subtests should not depend on the execution of other cases for success or initial state |
| T09 | 試験関数 ・ サブテスト ・ システムテストの場面の名前は、確かめる振る舞いの名前にする（`working tree の外を指す path は作れない` など）。ページの行の識別子（`UC3-2a` ・ `D1` など）を名前にもフィールドにも書かない | このプロジェクト | 利用者の指示（2026-11-05） | |
| T04 | サブテストの名前は、エスケープした後も読める名前にする | Go 公式 | TestComments.md L60–62 | choose subtest names that will remain useful and readable after escaping. |

### 3.3 t.Helper

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T10 | `*testing.T` を受け取る準備と後始末の関数は、最初に `t.Helper()` を呼ぶ | Go 公式 | TestComments.md L204–206 | If you pass a `*testing.T`, call [`t.Helper`](https://pkg.go.dev/testing#T.Helper) to attribute failures in the test helper to the line where the helper is called. |
| T11 | `t.Helper` で assert の関数を作らない | Go 公式 | TestComments.md L227–228 | `t.Helper` should not be used to implement assert libraries. |

### 3.4 比べ方

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T12 | `==` で比べられる値は `==` で、slice と map は `slices.Equal` ・ `maps.Equal` で比べる | Google | guide.md L186–187 | Where there are several ways to express the same idea, prefer the one that uses the most standard tools. |
| T13 | それ以外の構造は `cmp.Diff(want, got)` で比べる。`reflect.DeepEqual` を使わない | Go 公式 | TestComments.md L115–118 | Use the [cmp](https://pkg.go.dev/github.com/google/go-cmp/cmp) package. |
| T14 | 構造体はフィールドごとに比べず、期待する値を作って1回で比べる | Go 公式 | TestComments.md L72–74 | Instead, construct the struct that you're expecting your function to return, and compare in one shot |
| T15 | JSON のバイト列を文字列で比べず、読んでから比べる | Go 公式 | TestComments.md L103–105 | Instead, a more robust test would parse the contents of the JSON string and ensure that it is semantically equivalent to some expected data structure. |
| T16 | 出力は JSON Pointer と値の組で比べ、`message` の文は比べない | このプロジェクト | 具体のページ 9節「比べ方」 | |
| T17 | error は文で比べず、code で比べる | Go 公式 | TestComments.md L278 | don't use string comparison to check what type of error your function returns. |

### 3.5 失敗のメッセージ

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T20 | 失敗の文は `関数(入力) = got, want want` の形にする | Go 公式 | TestComments.md L136–137 | A usual format for printing test outputs is "`YourFunc(%v) = %v, want %v`". |
| T21 | got（関数が返した値）を先に、want（期待した値）を後に書く | Go 公式 | CodeReviewComments.md L571 | Note that the order here is actual != expected, and the message uses that order too. |
| T22 | 失敗の文に、失敗した関数の名前を入れる | Go 公式 | TestComments.md L148–149 | In most tests, failure messages should include the name of the function that failed, even though it seems obvious from the name of the test function. |
| T23 | 入力が短ければ、失敗の文に入力を入れる。長ければ、行の名前を入れる | Go 公式 | TestComments.md L167–169 | you should name your test cases with a description of what's being tested, and print the description as part of your error message. |
| T24 | diff を出すときは、向き `(-want +got)` を書く | Go 公式 | TestComments.md L238 | Something like "`diff -want +got`" is good when you're using the `cmp` package |
| T25 | diff は、改行してから出す | Go 公式 | TestComments.md L243–244 | The diff will span multiple lines, so you should print a newline before you print the diff. |
| T26 | 比べた結果ごとに `t.Error` を呼び、1回の実行で全部の失敗を出す | Go 公式 | TestComments.md L182 | On a practical level, prefer calling `t.Error` over `t.Fatal`. |
| T27 | `t.Fatal` は、準備に失敗して先へ進めないときだけ呼ぶ | Go 公式 | TestComments.md L186–187 | `t.Fatal` is usually only appropriate when some piece of test setup fails, without which you cannot run the test at all. |
| T28 | 試験が起こした goroutine の中で `t.Fatal` を呼ばない | Google | best-practices.md L3039–3040 | If your test starts new goroutines, they must not call these functions from inside these goroutines. |

### 3.6 t.Parallel

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T30 | 試験とサブテストは、`t.TempDir()` で自分だけのディレクトリを持ち、`t.Parallel()` を呼ぶ | このプロジェクト | 具体のページ 9節「環境」 | |
| T31 | `t.Setenv` を使う試験では `t.Parallel()` を呼ばない | Go 公式 | go-doc_testing.T.Setenv.txt L7–8 | Because Setenv affects the whole process, it cannot be used in parallel tests or tests with parallel ancestors. |
| T32 | `t.Chdir` を使う試験では `t.Parallel()` を呼ばない | Go 公式 | go-doc_testing.T.Chdir.txt L8–9 | Because Chdir affects the whole process, it cannot be used in parallel tests or tests with parallel ancestors. |
| T33 | 試験の中の context は `t.Context()` から取る | Google | decisions.md L2844–2846 | In test functions (e.g. `TestXXX`, `BenchmarkXXX`, `FuzzXXX`), where the context comes from |

### 3.7 期待値

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T40 | 試験とページの行を識別子で結び付けない。識別子の対応を `go test` で照らす試験を作らない | このプロジェクト | 利用者の指示（2026-11-05） | |
| T43 | 期待値は、実装より先にページの行から書く。出力から写さない | このプロジェクト | 具体のページ 9節「期待値」 | |
| T44 | ハッシュの期待値は、試験の中でファイルのバイト列から求める | このプロジェクト | 具体のページ 9節「期待値」 | |
| T45 | 試験の正解は、本質と2枚のページだけにする | このプロジェクト | 具体のページ 9節 | |

### 3.8 代役を作らず本物のアダプターで試す

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T50 | 試験の代役（mock ・ stub ・ fake）を作らず、本物のアダプターで試す。本物のアダプターを包んで、決まった時点に別の書き手の書き込みだけを足す型は、代役に数えない（読んでから書くまでの競合の場面を作るため） | このプロジェクト | 具体のページ 2節 ・ 9節「環境」 | |
| T51 | 本物の実装の公開の API で試せるように作る | Go 公式 | CodeReviewComments.md L322–323 | design the API so that it can be tested using the public API of the real implementation. |
| T52 | 試験ごとに新しい一時ディレクトリで `git init` し、操作の記録は一時ファイルの SQLite にする | このプロジェクト | 具体のページ 9節「環境」 | |
| T53 | 子プロセスの環境変数は、空から作る | このプロジェクト | 具体のページ 9節「環境」 | |

### 3.9 試験のレベルとシステムテストの場面

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T60 | domain はコンポーネントテストにする。変更点の求め方はデシジョンテーブル、値オブジェクトと作法の検査は同値分割法で場合を作る | このプロジェクト | 具体のページ 9節「レベル」 | |
| T61 | application とアダプターは、コンポーネント統合テストにする | このプロジェクト | 具体のページ 9節「レベル」 | |
| T62 | システムテストは、UC の主成功シナリオか拡張1つを1場面にする | このプロジェクト | 具体のページ 9節「レベル」 | |
| T63 | 場面は、`cmd/semantic-agent-schema/testdata/` の JSON のファイル1つに1場面で書く | このプロジェクト | 具体のページ 9節「レベル」 ・ T03 | |
| T64 | 場面の各ステップで、終了コード ・ 出力の行の数 ・ 行ごとの JSON Pointer と値の組を比べる | このプロジェクト | 具体のページ 4節 ・ 9節「比べ方」 | |
| T65 | 全部の出力を `event.schema.json` で検証する。`*_requested` のログイベントの prompt は、そこから参照する `prompt.schema.json` で検証する | このプロジェクト | 具体のページ 9節「比べ方」 | |
| T66 | バイナリは `TestMain` で1回だけビルドする。`TestMain` はシステムテストの package だけで使う | Go 公式 | testing.go L394–395 | TestMain is a low-level primitive and should not be necessary for casual // testing needs, where ordinary test functions suffice. |

### 3.10 完成の条件

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| T70 | `deadcode ./cmd/semantic-agent-schema` を `-test` を付けずに実行し、報告される関数を0件にする。試験からしか呼ばれない関数を残さない | このプロジェクト | 具体のページ 9節「完成の条件」5 | |
| T71 | deadcode が報告するのは、`main` から届かない関数である | Go 公式 | deadcode/doc.go L12–13 | Any functions that are not reachable are reported as dead code, grouped by package. |
| T72 | `go vet` ・ `gofmt -l` ・ `go test ./...`（`architecture_test.go` と exhaustive を含む）を全部通す | このプロジェクト | 具体のページ 9節「完成の条件」5 | |

## 4 このプロジェクトのログイベントと error の書き方

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| L01 | 出力はすべてログイベントにし、標準出力へ JSON を1行に1つ書く。読むのは AI | このプロジェクト | 抽象のページ 9節 ・ 本質 `W5` | |
| L02 | ログイベントの形は `event.schema.json` が決める。Go の構造体の JSON のキーは、その形と同じにする。event の名前は、ユースケースで起きたことの過去形（`generation_requested` ・ `created` ・ `update_requested` ・ `updated` ・ `validated` ・ `rejected`）。prompt は `*_requested` のログイベントの `prompt` の欄に入れる | このプロジェクト | 具体のページ 5節 | |
| L03 | rejected のログイベント（理由は code）は、`code` ・ `instanceLocation` ・ `absoluteKeywordLocation` ・ `message` ・ `instruction` ・ `next` を持つ | このプロジェクト | 抽象のページ 9節 ・ 具体のページ 5節「error」 | |
| L04 | `code` は、`error-codes.json` のキーの名前だけを使う | このプロジェクト | 具体のページ 5節 | |
| L05 | `instruction` の定型文と `next` の形は `error-codes.json` に置き、Go のコードに文を書かない | このプロジェクト | 具体のページ 5節「エラーの定型」 | |
| L06 | Go のコードは、定型の `{…}`（`{instanceLocation}` ・ `{absoluteKeywordLocation}` ・ `{message}` ・ `{violations}` ・ `{path}` ・ `{command}` ・ `{paths}` ・ `{schemas}` ・ `{schema}` ・ `{range}` ・ `{all}`）を埋めるだけにする | このプロジェクト | 具体のページ 5節「エラーの定型」 | |
| L07 | `next` の `<…>` は、AI が埋める部分として残す | このプロジェクト | 抽象のページ 9節「next」 | |
| L08 | `next` には、受け取った `--schemas` ・ `--range` ・ `--schema` ・ `--all` を引き継ぐ。ハッシュは引数で渡さない（楽観ロックは update の中で、最初に読んだハッシュと書く直前のハッシュを比べて完結する。判定した内容のハッシュは、AI が judgment_requested の prompt から judgments の JSON に写す） | このプロジェクト | 具体のページ 4節「next への引き継ぎ」 | |
| L15 | コマンドは2つの層に分ける。`semantic <ユースケース>`（意味的な操作）は prompt を返し、`<ユースケース>`（機械的な操作）は AI が作ったものを受け取って機械的に検証する：`semantic create <path> --schemas … --schema <$id>` ・ `create <path> --schemas … --json <JSON｜->` ・ `semantic update <path> --schemas …` ・ `update <path> --schemas … --patch <JSON｜->` ・ `semantic validate <path> --schemas … [--all]` ・ `validate <path>... --schemas …` ・ `validate <path> --schemas … --judgments <JSON｜-> [--all]` | このプロジェクト | 利用者の指示（2026-11-05） | |
| L16 | 意味的な操作の next は、同じユースケースの機械的なコマンドにする。`semantic` の有無も引数の JSON に入れ、`arguments.schema.json` で検証する | このプロジェクト | 利用者の指示（2026-11-05） | |
| L17 | `--help`（`-h`）は、コマンドと引数の一覧を `help` のログイベント（読むのは AI）で返す。説明は `arguments.schema.json` の description から作り、契約と説明をずらさない。引数を足したら description も書く | このプロジェクト | 利用者の指示（2026-11-05） | |
| L18 | prompt では、x-prompt を `subschemas` にサブスキーマ（keyword location）ごとに1回だけ置き、更新と判定の step は instance location ・ keyword location（判定は rule の id も）だけを持つ。`response` には、response の形の JSON Schema の中身（`$id` だけでなく）を入れる。更新の prompt は、いまの内容の直前の検証の violated の judgment を `violations` に持ち、Concrete の更新の next は `--range` を引き継ぐ形にする | このプロジェクト | 利用者の指示（2026-11-05） | |
| L19 | rule は、それを置いたサブスキーマが当たるインスタンスの位置に当たる。インスタンスの判定（Concrete の rule）も Concrete の判定（Abstract の rule）も同じ決め方にする。サブスキーマとインスタンスの位置の対応は JSON Schema の検証に決めさせ、コードで properties ・ items をたどらない：`schemacheck` が `RegisterVocabulary` で x-prompt を持つサブスキーマに SchemaExt を返し（`CompilerContext.Enqueue(nil)` で自分の `*Schema` を引く）、`Validate` のたびに「`Schema.Location` × `ValidatorContext.ValueLocation()`」の組を集めて、ポート `Validator.Evaluate` で domain に渡す。step の keyword location は、検証に使ったスキーマの中の位置（`#/properties/title`、`#/$defs/keywords/properties/x-prompt/properties/meaning` など）。Abstract の rule は、`$defs/keywords` の `properties.x-prompt` の構造（Concrete に書く x-prompt の値に当たる）の `meaning` ・ `when` ・ `rules.items` に置き、その形（type ・ required）は vocabulary の x-prompt の1か所だけが持つ | このプロジェクト | 利用者の指示（2026-11-05） | |
| L20 | JSON Schema の違反と JSON として読めないことは、違反した JSON が引数（`--json` ・ `--patch` ・ `--judgments` ・ `--matches`）なら `error-codes.json` の `argumentInstruction`（「渡した {argument} の JSON の "<位置>" を直す」）で、文書なら `instruction` で示す | このプロジェクト | 利用者の指示（2026-11-05） | |
| L21 | 値なしの `--range` は、更新する Concrete に従うインスタンスが無いことを示す（`range` が空の並び） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L22 | AI の判定を受け取っていない検証（`validate` と、判定の prompt の counts）では、変更点に当たる rule と instance location の組（判定の prompt の組と同じ）を、`not-judged`（by は `ai`）の judgment にして数える。violated が無く not-judged があれば、validated に `error-codes.json` の `not_judged` の instruction と、`semantic validate` の next を付ける（終了コードは 0） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L23 | 判定する組が無ければ、`semantic validate` は判定の prompt を返さず、検証を操作の記録に追記して `validated` で返す（`--judgments` の next を出さない） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L24 | violated の judgment がある validated と judged の instruction は、judgment ごとに rule の id と、引用符で囲んだ instance location を並べる（`{violations}`）。Concrete を直す next の update は、更新の prompt と同じく `--range` を足す形にする（`{range}`） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L25 | 判定と更新の prompt には、`error-codes.json` の `prompt.instruction` から、path のファイルを読んで判定すること（判定）・ violations の reason に従って直すこと（更新）を `instruction` で書く（内容は貼らない） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L26 | prompt の x-prompt は、スキーマに書いた順のまま置く。Concrete の書いたままのバイト列から `json.RawMessage` で取り出し、`map` を通さない | このプロジェクト | 利用者の指示（2026-11-05） | |
| L27 | 例の related のキーが related_fields に無い作法の違反は、同じ rule の `related_fields` の位置を instance location にし、bad と good の例をまたいで rule ごとに1件にまとめる（message に、例の位置と足りないキーを並べる） | このプロジェクト | 利用者の指示（2026-11-05） | |
| L28 | 更新の prompt に violations があれば、steps を violations の instance location にする（同じ位置は1つ）。keyword location は、AI の判定の違反ならその rule を置いたサブスキーマ、機械的な検証の違反ならインスタンスと Concrete を一緒にたどって、その位置か一番深い親の位置のサブスキーマにする。violations が無ければ、title ・ description ・ x-prompt を持つサブスキーマに当たる位置を steps にする | このプロジェクト | 利用者の指示（2026-11-05） | |
| L09 | 引数は pflag で読んで JSON にし、`arguments.schema.json` で検証する。Go のコードで引数の形を確かめない | このプロジェクト | 具体のページ 4節 ・ 本質 `W5` | |
| L10 | JSON Schema の違反は、`schemacheck` が `ValidationError` の葉ごとに、`InstanceLocation` を `instanceLocation`、キーワードの絶対の位置を `absoluteKeywordLocation` にして1行ずつ出す | このプロジェクト | 具体のページ 5節「error」 | |
| L11 | JSON Schema の違反には、独自の判定も独自の文も足さない。ライブラリの出力を変換するだけにする | このプロジェクト | 抽象のページ 9節 ・ 具体のページ 5節「error」 | |
| L12 | `propertyNames` の違反の instance location は、親の instance location で置き換える。`Causes` は葉までたどる | このプロジェクト | 具体のページ 3節 | |
| L13 | `message` は、JSON Schema の実装の出力か、不変条件の違反の文にする | このプロジェクト | 抽象のページ 9節 | |
| L14 | 終了コードは、error か violated の judgment があれば 1、ほかは 0 にする | このプロジェクト | 具体のページ 4節 | |

## 5 用語

| ID | こうする | 種類 | 出どころ | 原文の引用 |
|---|---|---|---|---|
| U01 | コードの名前 ・ コメント ・ 試験の名前 ・ ログイベントの文は、抽象のページ 4節「ユビキタス言語」の語だけを使う | このプロジェクト | 抽象のページ 4節 | |
| U02 | 1つの概念に1つの語を使う。同じものを別の語で呼ばない | このプロジェクト | 抽象のページ 4節 | |
| U03 | キーと識別子は、ユビキタス言語の英語の語をそのまま使う（下の表） | このプロジェクト | 抽象のページ 4節 | |
| U04 | DDD とユースケースの語（集約 ・ 値オブジェクト ・ ドメインサービス ・ 主アクター ・ 拡張など）は、原典の訳語のまま使う | このプロジェクト | 抽象のページ 4節 | |

| ユビキタス言語の語 | Go の識別子 |
|---|---|
| JSON ドキュメント | `Document` |
| Abstract ・ Concrete | `Abstract` ・ `Concrete` |
| instance location | `InstanceLocation` |
| absolute keyword location | `AbsoluteKeywordLocation` |
| ハッシュ | `Hash` |
| path | `Path` |
| judgment ・ verdict | `Judgment` ・ `verdict` |
| rule | `Rule` |
| 前回の検証 | `Previous` |
| change | `Change` |
| prompt（コマンドが x-prompt から作って AI に返すもの。スキーマの側の語 x-prompt とは混ぜない） | `Prompt` |
| response ・ next（AI が prompt に従って作り、コマンドへ渡すもの ・ 次に実行するコマンド行） | `Response` ・ `Next` |
| code ・ message ・ 指示 | `code` ・ `Message` ・ `Instruction` |
