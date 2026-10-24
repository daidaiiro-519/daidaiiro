# 反証：arch/draft.md（実装のアーキテクチャの案）

読んだもの：arch/draft.md、premises/README.md ・ board-answers.json ・ decisions-in-session.md ・ user-instructions.md、model/model.json、ports.json、organize.html、sources/rust/ の原典。

記法：draft の 42行目の「X ← Y」は「Y が X に依存する」と読んだ（「kernel ← declaration」から）。以下の「A → B」は「A の Cargo.toml に B を書く（A が B を使う）」の意味で書く。

## 結論（先に）

| # | 所見 | 重さ |
|---|---|---|
| 1 | 案 B のまま核のドメインサービスを置くと、change ⇄ validation、change ⇄ render、declaration ⇄ instance の3つの循環が要る。Cargo は循環を許さないので、ビルドが通らない | 高 |
| 2 | 循環を外すと、インスタンス ・ 判定の記録 ・ 変更の記録の型が kernel へ移り、kernel が核のモデルのほぼ全部を持つ | 高 |
| 3 | 循環を外す3つとも、「集約（下）とそれを使うドメインサービス（上）を別の crate に分ける」ことで解ける。これは案 C の分け方である。案 B は推しとして弱い | 高 |
| 4 | 許す辺の表は、足りない辺が8本以上あり、session の「kernel だけ」は成り立たない | 高 |
| 5 | 同じ入力から同じ結果（Q0 #14）を、Cargo の feature の統合が崩せる。draft の検査（clippy ・ 辺の表）はこれを止めない | 高 |
| 6 | 「判定するは人の入口だけ（MCP には見せない）」を、案 B の workflow 1つではビルドで守れない | 中 |
| 7 | 「決定的」：循環が起きる2点（workflow を置くか ・ 判定の記録をどこに置くか）を未決に残している | 中 |
| 8 | 「セマンティックなディレクトリ構成」：contexts/ ・ kernel の名前が、決着 Q9 の「1つの区切られた文脈」と食い違う。semantic-agent-schema/ の根と schemas/ の置き場所が無い | 中 |

---

## 1. 案 B が壊れる場面

| # | 壊れる場面（ドメインサービスと要る依存の辺） | 依拠した前提（ファイル ・ 行 ・ 原文） | 重さ |
|---|---|---|---|
| B1 | **change ⇄ validation の循環**。「変更を見せる」（change）は全インスタンスを検証するので change → validation。一方「Semantic Schema Validator」（validation）は判定の記録を使い直し、I33 を変更の記録ごとに判定するので validation → change（判定の記録 ・ 変更の記録は change にある）。さらに I33 の「判定した値」の前の値は最後に確定した姿から取るので、validation は changes のポート（change の crate が定義）も要る | model.json:628「判定した値がいまと同じ判定の記録は使い直す。…（I33）は Abstract の rule として、変更の記録ごとに変更の中でだけ判定する」／model.json:648「変更（取り込みを含む）の後の姿で全インスタンスを検証し」／model.json:504 判定した値「前の値は、その記録が入った単位の前の姿（未確定なら最後に確定した姿）から取る」／draft:38「change（…判定の記録…changes のポート）」／resolver.html:568「Usually the resolver does not allow cycles in the graph」 | 高 |
| B2 | **change ⇄ render の循環**。「変更を見せる」は描画の差分を出すので change → render。「Semantic Schema Renderer」は状態と一緒に描くので render → change（「状態を導く」は change）。draft:42 の「change → render」は、同じ行の「←」の記法と向きの読み方が逆で、どちらに依存するかが読めない | model.json:648「…検証結果の差分 → 描画の差分（ビューが無い kind は既定の描き方で）の順で出す」／model.json:679「インスタンスを、状態と rule ごとの結果と一緒に人が読む形にする」／model.json:640 状態を導く の業務領域「変更の確かめと例外の判断」／draft:42「declaration ← render、change → render（描画の差分）」 | 高 |
| B3 | **declaration ⇄ instance の循環**。「作成する」（declaration）は Concrete からインスタンスの雛形を作るので declaration → instance（集約インスタンス）。「取り込む」（instance）は既存のファイルに $schema を当てるので instance → declaration（Concrete）、さらに $schema が1つに決まるかを確かめるなら instance → validation | model.json:602 作成する「Abstract から Concrete の雛形、Concrete からインスタンスの雛形を作る（2段）」業務領域「意味の宣言」／model.json:410 集約インスタンスの業務領域「インスタンスの作成と変更」／model.json:693 取り込む「既存のファイルに $schema を当ててインスタンスにし」 | 高 |
| B4 | **kernel が太る**。B1〜B3 を crate を足さずに外す手は、両側が使う型を kernel へ下ろすことだけである。下ろす型は、インスタンス（validation ・ change ・ render ・ search ・ instance ・ migration が使う）、判定の記録と検証結果（validation ・ change ・ search ・ render）、変更の記録の対象（judgment が change の $defs/target を借りる）。kernel は「全部の領域が使う型だけ」の条件を満たしたまま、核の集約の半分を持つ | draft:35「kernel：5つの値 ・ URN ・ id ・ JSON Pointer など、全部の領域が使う型だけ」／organize.html:137「judgment … change の $defs/target … 対象の指し方をそろえる」／model.json:668 意味で探して取り出す「判定の結果 ・ 状態で探し」 | 高 |
| B5 | **orphan rule に当たる**。B1 を依存の逆転で外す（change に trait「全インスタンスを検証する」を置き、validation の Validator に実装させる）と、実装を書ける crate は change か validation だけである。validation に書けば validation → change で循環に戻る。workflow に書くと trait も型も外の crate なので orphan rule に反する。newtype を workflow に作れば通るが、逆転させる trait ごとに包む型が1つ増える | reference implementations.md:155「a trait implementation is only allowed if either the trait or at least one of the types in the implementation is defined in the current crate」／ch10-02:101「we can implement a trait on a type only if either the trait or the type, or both, are local to our crate」 | 中 |
| B6 | **1つの使う場面が4〜6の crate をまたぐ**。U25 変更を確かめる：change（変更を見せる ・ 例外をまとめる）＋ validation（Validator）＋ render（描画の差分）＋ session（手元の索引で引く）＋ declaration（ビューの kind が指す Concrete）。U2 Concrete を変える：declaration（照らす）＋ change ＋ validation ＋ render ＋ session。U16 判定する：change（例外をまとめる ・ 判定の記録 ・ confirm）＋ validation（I33）＋ session。核の中核の使う場面は、どれも workflow を通る。業務領域ごとの crate は、ほぼ workflow からしか呼ばれない | model.json の use_cases U2 ・ U16 ・ U25 と domain_services の担う使う場面（U25 は 変更を見せる・Validator・意味で探して取り出す に付く）／board-answers Q9 #12「同じデータを操作するユースケースを複数の文脈に分散させない」 | 中 |
| B7 | **アダプターの依存が「満たすポートの crate と kernel だけ」に収まらない**。files は「契約を読む ・ 書く」（declaration）・「インスタンスを読む ・ 書く」（instance）・「判定の記録」「変更の記録」（change）の4つのポートを満たすので、3つの業務の crate に依存する。ai-claude-code ほか5つは judge（validation）と ai_session（session）の両方を満たす。store-sqlite は「session の形から表を作る」ので、Abstract のスキーマのファイルを持つ declaration にも依存する | model.json ports.outbound の1〜4行目「文脈のリポジトリのファイル」／ports.json「judge」「ai_session」／organize.html「DB の表の形は Abstract のファイル session から、SQLite のアダプターが作る」／draft:36「declaration … Abstract のスキーマのファイルを持つ」／draft:44「各々は自分が満たすポートの crate と kernel にだけ依存する」 | 中 |
| B8 | **SQLite の置き場所の既定が Git に依る**。「Git のアダプターを使うときの既定はすべての worktree が共有する場所」を決めるには Git の情報が要る。store-sqlite → changes-git（アダプターどうし）か、compose が両方を知るかのどちらかで、draft はどちらとも書いていない | model.json ports.outbound 5行目「置き場所は起動の設定で渡し、Git のアダプターを使うときの既定はすべての worktree が共有する場所」／draft:29「アダプターどうしの全部の辺を Cargo.toml が持ち」 | 低 |

## 2. 案 A ・ 案 C のほうが良い場面

| # | 場面 | 依拠した前提 | 重さ |
|---|---|---|---|
| A1 | **案 C は B1〜B3 の循環を外せる**。循環はどれも「集約（使われる側）とそれを組み立てるドメインサービス（使う側）が同じ crate にある」ことで起きる。C で分けると：validation-domain → change-domain（判定の記録 ・ 変更の記録）、change-app（変更を見せる）→ validation-domain ＋ render-domain、render-domain → change-domain（状態）、declaration-app（作成する）→ instance-domain、instance-app（取り込む）→ declaration-domain ＋ validation-domain。どれも一方向で、kernel を太らせない（推測：各ドメインサービスを domain と app のどちらに置くかは、この反証の中で決めた仮の割り当て） | model.json:628 ・ 640 ・ 648 ・ 679 ・ 602 ・ 693／resolver.html:584「try to split your package into multiple packages and restructure it so that it remains strictly acyclic」 | 高 |
| A2 | **案 A は決着 Q9 の言い方にそのまま合う**。Q9 は「基盤の内側は1つの区切られた文脈にし、中に業務領域ごとの論理的な境界（名前空間 ・ モジュール）を置く」とし、根拠を「同じデータを操作するユースケースを複数の文脈に分散させない」とした。B6 のとおり使う場面が4〜6の業務領域をまたぐので、1つの crate の中のモジュールに置くと、場面を1か所で読める | board-answers Q9 #11 ・ #12 | 中 |
| A3 | **案 A は Q3 のいまの答え（参考）と同じ**。「crate は core と adapters の2つ」「adapters から core への向きだけをビルドで守る」。A を外すなら、外す理由を書く必要がある | board-answers Q3 #3（open。決着ではない） | 低 |
| A4 | 案 A の弱さ：業務領域どうしの向きはビルドで守れない。draft:28 のとおり。モジュールどうしの循環が書けてしまう（前提の外：モジュールの相互参照を許すことを示す原典の行は sources/rust/ に無い） | draft:28「業務領域どうしの向きはレビュー頼み」 | ― |

## 3. 利用者の指示を満たしていない点

| # | 満たしていない点 | 依拠した前提 | 重さ |
|---|---|---|---|
| D1 | **決定的でない**。循環が起きる2点（workflow を置くか、判定の記録を change と validation のどちらに置くか）と、AI のアダプターの切り方を未決に残し、A ・ B ・ C も「推し」で止めている。B1 ・ B2 は未決の2点そのものから起きる | user-instructions.md:58「望むのは決定的なアーキテクチャであり、依存の向きなども考慮してほしいんですよね。」／draft:54-56 | 中 |
| D2 | **ヘキサゴナルが適しているかを比べていない**。1節は採る理由だけで、適さない場面や別の構成と比べていない | user-instructions.md:57「ヘキサゴナルが適しているのかは改めて考えた上で」 | 中 |
| D3 | **ディレクトリの名前が決着と食い違う**。contexts/ は「区切られた文脈」の語で、決着は内側を1つの文脈とした。業務領域は文脈ではない。kernel（共有カーネル）も文脈どうしの間の語で、1つの文脈の中では指すものが無い | board-answers Q9 #11「基盤の内側は1つの区切られた文脈にし」／draft:34「業務の側（contexts/）」 | 中 |
| D4 | **根のフォルダとスキーマの置き場所が無い**。利用者は semantic-agent-schema というフォルダを作るよう指示し、model.json の naming は Abstract の正本を src/semantic-agent-schema/schemas/ に置くとした。draft は declaration が「スキーマのファイルを持つ」とし、置き場所が crate の中か schemas/ かが決まらない | user-instructions.md:59「このsemantic-agent-schemaというフォルダを作成して」／model.json:1366「Semantic Abstract Schema の正本は src/semantic-agent-schema/schemas/ に置く（Q6）」／draft:36 | 中 |
| D5 | **アダプターの名前がポートを示さない**。adapters/ を技術名（ai-claude-code ・ files）で切り、どのポートを満たすかがディレクトリから読めない。files は4つ、ai-* は2つのポートを満たす（B7） | ports.json の5つのポート／model.json ports.outbound 12行 | 低 |
| D6 | **Rust の性質のうち、向きを守るのに効くものを使っていない**。crate の中の公開範囲（pub(crate)）・ feature の統合（下の F1）を書かず、効かない orphan rule の行（ポートを業務の crate に置く限り何も制約しない）を置いている | ch07-03:285「everything being private by default unless annotated with `pub`」／features.html:386「Cargo will use the union of all features enabled on that dependency」 | 低 |

## 4. 許す辺の表（draft:42 ・ 44）の誤り

| # | 誤り | 種類 | 依拠した前提 |
|---|---|---|---|
| E1 | validation → change（判定の記録の使い直し ・ I33 の対象 ・ changes のポート）。表は change → validation だけ | 足りない（入れると循環） | model.json:628 ・ 504 |
| E2 | render → change（状態）・ render → validation（rule ごとの結果） | 足りない | model.json:679 |
| E3 | 「change → render」は、同じ行の「←」の記法では render が change に依存する向きになる。「描画の差分」の説明は change が render を使う向き | 向きが読めない | draft:42 |
| E4 | search → change（判定の結果 ・ 状態で探す）・ search → declaration（term ・ glossary ・ Concrete の拡張項目）。Cargo では依存先の依存先を直接は使えないので、validation 経由では足りない（前提の外：sources/rust/ に直接の原文は無い。ch14-03:150「we need to be explicit about the dependency relationships」が近い） | 足りない | model.json:668 |
| E5 | validation → publish（Resolver が公開された言葉の版を解決する）・ publish → declaration（Concrete の公開する部分） | 足りない | model.json:618 ・ model.json の 公開された言葉の版を出す |
| E6 | migration → declaration（Abstract の版 ・ Concrete を照らす） | 足りない | model.json:686 |
| E7 | instance → declaration ・ validation、declaration → instance（B3 の循環） | 足りない（入れると循環） | model.json:602 ・ 693 |
| E8 | session → declaration（行の形は Abstract の session。基盤が行を検査する）。「session は kernel だけ」は成り立たない | 要る辺が無い | organize.html「手元の DB の行 … SQLite のアダプターが session の形から表を作り、基盤が行を検査する」 |
| E9 | アダプター側：files → declaration ・ instance ・ change、ai-* → validation ・ session、store-sqlite → session ・ declaration | 足りない | B7 |
| E10 | 足りないアダプターとポート：公開された言葉を取りに行く ・ 描画したものを書き出す（HTML ・ Markdown）・ 外のツールを呼ぶ（Concrete が宣言した validator。argv で起動）・ ULID を決める（時刻と乱数）。1節は「時刻もポートか値で」とするが、change_id の ULID は時刻と乱数の両方を要る | 足りない | model.json ports.outbound 7 ・ 9 ・ 10行／decisions-in-session.md:98「change_id）は、基盤が書いた時に1回だけ決める ULID」 |
| E11 | 入口：「人の操作の入口」と CI が無い（cli ・ mcp だけ） | 足りない | model.json:768「人の操作の入口だけ（MCP には見せない）」／ports.inbound 3行「CLI ・ MCP ・ CI」 |
| E12 | 要らない辺は見つからなかった | ― | ― |

## 5. 決定的にするための検査（draft 5節）が止めないもの

| # | 壊れる場面 | 依拠した前提 | 重さ |
|---|---|---|---|
| F1 | **feature の統合で、検証が外のスキーマを取りに行く**。Q4 は jsonschema の取りに行く機能を feature で外すとした。公開された言葉を取りに行くアダプター（または別の crate）が jsonschema を取りに行く feature 付きで使うと、`--workspace` でビルドしたとき validation の jsonschema も同じ feature で作られる。辺の表の試験も clippy も、これを見ない | board-answers Q4 #9「jsonschema の取りに行く機能は feature で外す」／resolver.html:451-453「When building multiple packages in a workspace … the features of the dependencies of all of those packages are unified」／features.html:386-388 | 高 |
| F2 | **「判定するは人の入口だけ」をビルドで守れない**。workflow が1つの crate なら、mcp も承認の関数に届く。workflow を「人の入口だけが依存する crate」と分けるか、mcp から見えない形にしないと、この不変条件はレビュー頼みになる | model.json:768／U16「承認は人の入口からだけ入る」 | 中 |
| F3 | **受け入れの試験で全アダプターを回すと、同じ入力から同じ結果にならない**。AI のアダプター（Claude Code ほか）は実物の AI を呼ぶ。場面の JSON を受け入れの試験にするなら、judge と ai_session は決まった答えを返す試験用のアダプターで回す必要がある | board-answers Q0 #14「同じ入力から同じ結果を返し」／Q4 #10「試験は、入力と期待結果の JSON の組で持ち」／draft:51 | 中 |
| F4 | ジェネリクスと dyn の分け方の基準が合っていない。「実装が1つに決まるポートはジェネリクス」とするが、changes（Git ・ スナップショット）も利用者の環境で替わる | draft:22／model.json ports.outbound 6行「Git ・ スナップショット（前と後のディレクトリ）など」 | 低 |

## 6. 外せるもの

| # | 外せるもの | 理由 |
|---|---|---|
| G1 | 2節の orphan rule の行 | ポートの trait を業務の crate に置く限り、何も制約せず、構成を決める根拠になっていない（効くのは B5 の場面だけ） |
| G2 | kernel（推測） | 中身の5つの値は検証結果（validation）、URN は naming（declaration）に持ち主がある。最下段の crate（案 C なら declaration-domain）へ置けば、持ち主の無い crate を作らずに済む |
| G3 | AI のアダプターをツールごとの5 crate に分けること | Q3 #11 はツールごとの違いを「アダプターが吸収し、契約は AI ツールごとに分けない」とし、違いは起動の旗（Q0 #20 ・ #21）。1つの crate の中のモジュールで足りる（推測：旗以外の違いが無いことは未確認） |
| G4 | draft 4節の許す辺の文 | 5節の試験が持つ表と2か所になり、ずれる。表は試験の側だけに置く |
| ― | 上の4つのほかは見つからなかった | ― |

## 前提の外として分けたもの

- 案 A でモジュールどうしの循環が書けること（A4）、依存先の依存先を直接使えないこと（E4）は、sources/rust/ に原文の行が無い。
- crate を増やしたときのビルド時間は、前提に材料が無いので比べていない。
