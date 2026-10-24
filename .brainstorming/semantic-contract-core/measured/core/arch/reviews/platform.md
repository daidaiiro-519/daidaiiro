# platform-advisor の回答：実装のアーキテクチャ（非機能）

## 相談種別

判断相談（judgment）

## 相談

Semantic Agent Schema の基盤を Rust で実装するアーキテクチャの下書き（arch/draft.md の案 A ・ B ・ C、依存の向きを守る仕組み、アダプターの crate の分け方）を、ビルドの時間 ・ 配布する実行ファイル ・ 外の技術の依存の閉じ込め ・ 差し替えやすさ ・ 保守の費用で判定する

## 前置き

| 前提 | 内容 | 出所 |
|---|---|---|
| 判定の範囲 | ビルド ・ 配布 ・ 依存の閉じ込め ・ 検査の仕組み ・ 保守の費用だけを判定する。業務領域どうしの依存の向きが正しいか、workflow の crate を置くか、判定の記録の集約をどこに置くか（draft の6. 未決の1つ目と2つ目）は判定しない | platform-advisor のガードレール「アプリケーション内部の層の配置と依存の向きの判定は範囲外である」 |
| 配る物 | ほかの言語の Concrete もバイナリを呼んで基盤を使う。案 A ・ B ・ C のどれでも、配るのは OS ごとの実行ファイルである | premises/board-answers.json Q4 #8「ほかの言語の Concrete も、バイナリを呼んで基盤を使う。言語に依らないことは、バイナリと protocol で満たす」 |
| 検証の中で外へ取りに行かない | jsonschema の取りに行く機能は feature で外す、が決着している | board-answers.json Q4 #9「jsonschema の取りに行く機能は feature で外す」 |
| AI ツールの差し替え | AI ツールは CLI をプロセスとして起動し、利用者の環境で替わる | board-answers.json Q0 #19「CLI でプロセスとして起動する」・ Q0 #16「利用者ごとに違う AI ツールはアダプターで差し替える」 |
| 実測の条件 | scratchpad/platform-arch/ に、案 A（3 crate）・ 案 B（23 crate）・ 案 C（32 crate）を同じ量の試作のコード（モジュールごとに関数120個）と同じ外の crate（serde ・ serde_json ・ jsonschema 0.58.6 の default-features = false ・ rusqlite 0.32 の bundled）で作り、cargo 1.99.1 ・ 20 コアでビルドした。試作のコードは本物ではないので、時間の絶対値ではなく案どうしの差だけを読む | 実測（gen/gen.py ・ gen/measure.sh ・ measure.txt） |
| 印の意味 | 「実測」は scratchpad/platform-arch/ の試作で確かめたこと。「前提の外」は sources/rust/ に原文の行が無く、実測だけに依ること。注意の「（推測）」は、Skill の規則どおり判断基準に無い注意を回答の側で作った印で、中身の多くは実測である | arch-common.txt の守ること ・ platform-advisor の Step 3 |
| Git のアダプターの作り方 | 未判定。git の CLI をプロセスで呼ぶか、Rust のライブラリ（gix など）を使うかが決まっていない。後者なら重い外の依存を持つ | 前提の外（core のどのファイルにも無い） |

## 結論

| 判定 | 結論 | なぜ |
|---|---|---|
| 条件付きで採る | 案 B を採る。ただし AI ツール5つは1つの crate にまとめ、案 C の domain と application の分割は外し、依存の向きの検査は Cargo.toml の辺だけでなく dev 辺 ・ 外の crate ・ feature の3つも突き合わせる条件で採る | 実測では A ・ B ・ C のビルドの時間と配る実行ファイルに差が無く（どれも1つの実行ファイル、クリーンビルドの差は2秒以内）、差が出るのは外の依存の閉じ込めと、ビルドが止める範囲だけだった。一方、Cargo.toml の辺と循環の禁止だけでは、dev の循環 ・ workspace の feature の統合 ・ pub use による外の crate の持ち出し ・ clippy の allow の4つの書き方がビルドを通ることを試作で確かめたので、判断基準の「変更をテストし、検証する」を自動の検査として足さなければ決定的にならない。 |

## 図で確認する

採る形：業務の側は外の crate を名指しで許したものだけ持ち、外の技術はアダプターの4つの crate に閉じる。破線は試作でビルドを通った、向きの検査で止まらない書き方

- contexts/<業務領域>（9） → contexts/kernel（許す辺）
- contexts/<業務領域>（9） → jsonschema（default-features = false）（validation だけに許す外の crate）
- adapters/fs（ファイル ・ 設定 ・ スナップショット） → contexts/<業務領域>（9）（ポートを実装）
- adapters/sqlite（rusqlite bundled） → contexts/<業務領域>（9）（ポートを実装）
- adapters/git → contexts/<業務領域>（9）（ポートを実装）
- adapters/ai-cli（5ツールをモジュールで） → contexts/<業務領域>（9）（ポートを実装）
- entry/cli（compose を含む） → adapters/fs（ファイル ・ 設定 ・ スナップショット）
- entry/cli（compose を含む） → adapters/sqlite（rusqlite bundled）
- entry/cli（compose を含む） → adapters/git
- entry/cli（compose を含む） → adapters/ai-cli（5ツールをモジュールで）
- contexts/<業務領域>（9） → adapters/fs（ファイル ・ 設定 ・ スナップショット）（dev-dependencies の循環（止まらない書き方1））　［破線］
- entry/cli（compose を含む） → jsonschema（default-features = false）（default の feature が validation に統合される（止まらない書き方2））　［破線］

## 比較した案

| 案 | 何が起きるか | 採否 |
|---|---|---|
| 案 A（業務1 ＋ アダプター1 ＋ cli。実測 3 crate） | 実測：クリーンビルド（debug）9.8秒、本体だけの作り直し 2.9秒、kernel を触った後 0.60秒、release 32.8秒、実行ファイル1つ。rusqlite がアダプターの crate 全体から見えるので、ファイルや AI のアダプターから SQLite を直接呼べる。業務領域どうしの向きはビルドでは止まらない | 採らない |
| 案 B（業務領域ごと ＋ 外の技術ごと ＋ 結合 ＋ 入口。実測 23 crate） | 実測：クリーンビルド 8.6秒、本体だけの作り直し 1.25秒、kernel を触った後 0.37秒、release 31.5秒、実行ファイル1つ。rusqlite は store-sqlite に依存する crate からしか見えない。crate の数はビルドの時間を増やさなかった（並列に作られる） | 採る |
| 案 C（B の業務領域を domain と application に分ける。実測 32 crate） | 実測：クリーンビルド 8.1秒、本体だけの作り直し 1.45秒、kernel を触った後 0.41秒、release 31.7秒、実行ファイル1つ。B より閉じ込める外の依存は増えない（外の crate はもともとアダプターにある）。Cargo.toml と許す辺の表が9つ増えるだけ | 採らない |
| 案 B のアダプター10 crate（AI ツール5 ・ 変更の仕組み2 ・ SQLite ・ ファイル ・ 設定）をそのまま | AI ツールの5つは外の crate を持たず std::process だけで動き、利用者の環境で替わるので5つとも同じ実行ファイルに入る。分けてもビルドの時間（アダプターを触った後 0.22秒）も配る物も変わらず、Cargo.toml と辺が5つずつ増える | 採らない |
| アダプターを外の技術ごとの4 crate（fs ・ sqlite ・ git ・ ai-cli）にまとめる | 外の crate の閉じ込めは10 crate と同じ（rusqlite は sqlite だけ、Git のライブラリを使うなら git だけ）。AI ツールどうしの中身はモジュールの非公開で隠れる。辺の表が短くなる | 採る |

## 判断の道筋

| # | 問い | 答え |
|---|---|---|
| 1 | 案ごとにビルドの時間は変わるか | 変わらない（実測）。クリーンビルドは A 9.8秒 ・ B 8.6秒 ・ C 8.1秒で、大半は外の crate（単独で測って jsonschema 7.5秒 ・ libsqlite3-sys 4.2秒 ・ serde 2.3秒）。本体だけの作り直しは A 2.9秒 ・ B 1.25秒 ・ C 1.45秒で、crate を分けたほうが並列になり短い |
| 2 | 配る実行ファイルは変わるか | 変わらない。どの案も cli の実行ファイル1つで、workspace の crate はそこへ静的にリンクされる（実測で A ・ B ・ C とも1つ）。大きさの差は今回の試作では測れていない（試作の main は関数を呼ばず、使わないコードは除かれる） |
| 3 | 外の技術の依存を閉じ込められるか | B ・ C は rusqlite を store-sqlite の中に閉じ込められ、A はアダプター全体に広がる。ただし B でも、Cargo.toml に外の crate を書けば業務の crate が rusqlite を直接持てるので、外の crate も許す辺の表に入れなければ閉じ込めは保証されない |
| 4 | Cargo.toml の辺と循環の禁止で、向きは決定的に守れるか | 守れない（実測）。①dev-dependencies は循環を許す（change → files → change がビルドを通った）②workspace をまとめてビルドすると feature が統合され、change 単独ではビルドが通らないコードが workspace では通った。jsonschema では cli が default で足すと validation の jsonschema にも resolve-http と resolve-file が入った ③change が pub use した外の crate を、files が Cargo.toml に書かずに使えた ④clippy の disallowed は deny のとき #[allow] で外せ、依存先の中の入出力（extlib::fetch の中の std::fs）は見ない。辺を明示すること（ch14-03:150-151「Cargo doesn’t assume that crates in a workspace will depend on each other」）と循環の禁止（resolver:568「Usually the resolver does not allow cycles in the graph, but it does allow them for dev-dependencies」）は原文どおりで、①は同じ行が許している。②は resolver:451-453「When building multiple packages in a workspace … the features of the dependencies of all of those packages are unified」。③④は前提の外（実測だけ） |
| 5 | #![no_std] で業務の crate から std を外せば決定的になるか | ならない（実測 ・ 前提の外）。#![no_std] の crate でも extern crate std; の1行で std::fs が通った。jsonschema は std を使うので validation には付けられない。守りにしない |
| 6 | 止まらない書き方を何で止めるか | cargo metadata の辺を種類（normal ・ dev ・ build）ごとに突き合わせる ・ 外の crate も crate ごとに許すものだけ ・ 業務の crate を1つずつ -p で単独にビルドする ・ jsonschema の feature を cargo tree -e features で確かめる ・ clippy の disallowed は forbid にする（allow を書くとコンパイルが止まることを実測）。どれも CI で自動にする |
| 7 | アダプターをどう分けるか | 外の技術ごとに分け、AI ツールの5つは1つにまとめる。分けて得るものは外の crate の閉じ込めだけで、AI ツールは外の crate を持たない。Git は CLI で呼ぶならファイルと同じ std だけで済むが、ライブラリを使うなら単独の crate にする（未判定） |
| 8 | 外せるものは何か | ①案 C の domain と application の分割（ビルドも閉じ込めも変わらず、表が9つ増えるだけ）②AI ツールの5 crate を1つへ ③settings-file と changes-snapshot を fs へ（どちらも std::fs だけで、外の crate を持たない）④compose の crate は、実行ファイルが cli の1つなら cli の中のモジュールで足りる。MCP を別の実行ファイルで出すなら（いまの Skill は bin/<名前> と bin/<名前>-mcp の2つを配っている）2つの入口が共有するので残す ⑤clippy の disallowed の一覧を長くすること（外の crate の表で足りる分は列挙しない）。これで案 B は 23 crate から 業務10〜11（workflow を置くかで変わる）＋ アダプター4 ＋ 入口1〜2 の 15〜17 crate になる |

## 根拠

| 判断基準 | 引用 |
|---|---|
| ops-05 | ベストプラクティス：「変更をテストし、検証する」 ・ 望ましい成果：ソフトウェアの変更は、届ける前にすべてテストされている。デベロッパーがテストの結果と検証を見られる。どのソフトウェア変更にも当てはまるテスト基準が組織にある |
| ops-05 | ベストプラクティス：「構築およびデプロイ管理システムを使用する」 ・ 望ましい成果：組織の構築およびデプロイ管理システムが、CI/CD システムを支え、そのシステムが正しい設定で安全にロールアウトするところまで自動で行う |
| ops-05 | ベストプラクティス：「設計標準を共有する」 ・ 望ましい成果：設計標準がチームの間で共有されている。設計標準は文書として残り、ベストプラクティスが変わるたびに書き直される |
| sec-11 | ベストプラクティス：「開発およびリリースライフサイクル全体を通じてテストを自動化する」 ・ 何をするか：開発とリリースのライフサイクルを通して、セキュリティ特性のテストを自動にする。リリースの前にソフトウェアの問題を、いつも同じように繰り返し見つけやすくなり、出すソフトウェアのセキュリティの問題のリスクが下がる |
| sec-11 | ベストプラクティス：「パッケージと依存関係のサービスを一元化する」 ・ メリット：作っているソフトウェアで使うパッケージをよく分かる。誰がどのパッケージを使っているかが分かり、更新がいるときにワークロードのチームに知らせられる。問題のあるパッケージが使われるリスクが下がる |
| sec-11 | 種類 ・ ツール：ソフトウェア部品表 (SBOM) ・ 何をするか：ソフトウェアを作る部品の細部と関係を持つ正式な記録を取り出す。脆弱性管理に役立ち、依存先とサプライチェーンのリスクをすぐつかめる |
| rel-10 | 障害分離：コンポーネントやシステムの障害の影響を、決めた境界の内側に限ること。うまく分離できていれば、境界の外のコンポーネントは巻き込まれない。複数の境界にまたがって動かすと、回復力が上がる見込みがある。 |
| sus-03 | ベストプラクティス：「使用率が低い、またはまったく使用しないワークロードのコンポーネントを削除またはリファクタリングする」 ・ 目指す姿：ワークロードの動きを見て、コンポーネントごとの使用率の移り変わりをつかむ。使わないか不要なコンポーネントは消し、使用率の低いものはリファクタリングして、無駄を最小にする |
| cost-11 | 条件：自動化の効果を考えるとき ・ 対応：デプロイ ・ 管理 ・ 運用を自動にし、顧客に安定して信頼できる体験を届ける ・ 理由：インフラストラクチャの手作業から人を解き放ち、ビジネス成果がよくなる |

## 注意

| アンチパターン | リスク | 代替案 |
|---|---|---|
| （推測）dev-dependencies で業務の crate からアダプターへ辺を張る | cargo は dev の循環を許すので（resolver:568-569）、業務の試験がアダプターに依存しても、ビルドも許す辺の表（normal だけ見たとき）も止めない。試験の中で業務がアダプターを知る形が育つ。実測で change → files（dev）→ change が通った | 許す辺の表を種類ごとに持ち、contexts/* から adapters/* への dev 辺と build 辺を0件にする。アダプターを使う試験は entry の結合テストに置く |
| （推測）workspace をまとめてビルドしたときだけ通る | feature は workspace でまとめてビルドすると統合される（resolver:451-455）。jsonschema の default（resolve-http ・ resolve-file）を workspace のどこか1か所が足すだけで、validation の検証が外の $ref を取りに行けるようになり、Q4 #9 が黙って破れる。実測で cli に jsonschema の default を足すと validation 側にも resolve-http が入った | jsonschema は [workspace.dependencies] に default-features = false で1か所だけ書き、各 crate は workspace = true で継ぐ。CI で cargo tree --workspace -e features -i jsonschema に resolve-http ・ resolve-file が無いことを確かめ、業務の crate を -p で1つずつ単独にビルドする |
| （推測）clippy の disallowed を向きの主な守りにする | disallowed は列挙した API しか見ず、依存先の crate の中の入出力（実測で extlib::fetch の中の std::fs は見逃した）と、列挙し忘れた API（std::net ・ std::env など）を通す。deny のままだと #[allow] 1行で外せる（実測） | 主な守りは「業務の crate に許す外の crate の表」にする（入出力をする crate を持てなければ呼べない）。clippy は contexts/clippy.toml で業務の側だけに当て、Cargo.toml の [lints.clippy] で forbid にする（allow を書くと E0453 で止まることを実測） |
| （推測）pub use で外の crate を持ち出す | 許した辺の先の crate が外の crate を pub use すると、その先の crate は Cargo.toml に書かずに外の crate を使える（実測で files が change 経由で extlib を使えた）。外の crate の表が、書いた辺しか見ないと漏れる | 業務の crate から外の crate の pub use を禁じる（grep で検出できる）。公開するのは kernel の型だけにする |

## 差し戻し

| 疑われる上流（advisor ・ 判断） | 疑う理由 |
|---|---|
| ddd-advisor（業務領域どうしの許す辺と workflow の crate） | draft の4. の許す辺（change → render など）と workflow を置くかは、層と依存の向きの判定で、この Skill の範囲外である。crate の数が業務領域の数で決まるので、そこが変わると辺の表も変わる |
| qa-advisor（受け入れの試験で全アダプターを回す） | AI ツールの5つは CI に CLI が無いと回せない。全アダプターを回す試験と、偽の judge で回す試験をどう分けるかは試験の設計で、この判定では決めていない |

## 次にすること

- 許す辺の表を、crate の層（contexts ・ adapters ・ entry）ごとの規則と、辺の種類（normal ・ dev ・ build）と、crate ごとに許す外の crate の3つを持つ形で書く
- cargo metadata と許す辺の表を突き合わせる試験を、最初の crate を作るときに CI へ入れる
- jsonschema を [workspace.dependencies] に default-features = false で1か所だけ書き、feature を確かめる検査を CI へ入れる
- AI ツールの5つのアダプターを ai-cli の1つの crate に、ツールごとのモジュールとして置く
- Git のアダプターを CLI で呼ぶかライブラリを使うかを決め、ライブラリなら git の crate を単独にする
- 案 C の domain と application の分割を draft から外す

## 関連する基準

| 判断基準 | 関係 |
|---|---|
| ops-06 | 実行ファイルを OS ごとに出すときの、デプロイのテストと戻し方（今回は判定に使っていない） |

## AI が作業するときの注意

| 違反しやすい点 | 検査する方法 |
|---|---|
| AI が試験を書くとき、業務の crate の [dev-dependencies] にアダプターを足して試験を通す | cargo metadata --format-version 1 の resolve.nodes の deps を dep_kinds ごとに読み、contexts/* から adapters/* と entry/* への辺（normal ・ dev ・ build のどれも）が0件であることを CI で確かめる |
| AI が便利な crate（reqwest ・ jsonschema の default など）を業務の crate や cli にそのまま足す | 業務の crate ごとに許す外の crate の表と cargo metadata を突き合わせ、表に無い外の crate を0件にする。cargo tree --workspace -e features -i jsonschema の出力に resolve-http ・ resolve-file が無いことを確かめる |
| clippy が止めたとき、AI が #[allow(clippy::disallowed_methods)] を書いて通す | [lints.clippy] の disallowed_methods ・ disallowed_types を forbid にし、cargo clippy --workspace が E0453 で止まることを確かめる。あわせて contexts/ の下で allow(clippy::disallowed の文字列を grep し0件にする |

---

回答の JSON は validate に合格（`platform-advisor validate --kind answer`）。JSON と描画した HTML、試作と実測の記録は scratchpad の platform-arch/（answer.json ・ answer.html ・ gen/ ・ measure.txt）に置いた。
