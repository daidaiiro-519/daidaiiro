# semantic-agent-schema のコーディング規則

このファイルを置いたフォルダ（`semantic-agent-schema/`）より下のコードは、この規則に従う。

規則の1件は、**なぜ（守らないと何が起きるか）** と **検査** を持つ。検査は機械で見る方法を書き、機械で見られないものは「レビュー」と書く。なぜを書けない規則は置かない。原典の引用は、取得した原文と照合してある。

---

## 1. アーキテクチャと依存の向き

ポートとアダプター。中心は `crates/domain/` の8つの crate で、`crates/application/` が使う場面とポートを持ち、`crates/outbound/` と `crates/inbound/` が外側である。

| 規則 | なぜ | 検査 |
|---|---|---|
| crate の依存は `architecture.json` が許す辺だけにする | 依存の向きが外から中心へ揃っていないと、中心の変更理由が外の技術に引きずられる | `architecture-lint check --rules architecture.json --cargo .` が0件 |
| domain の crate は値を受け取り値を返す。ファイル ・ 通信 ・ プロセス ・ 時刻に触らない | 中心が入出力に触ると、代役なしの単体テストで確かめられなくなる | `crates/domain/clippy.toml` の `disallowed-methods` ・ `disallowed-types`（`std::fs` ・ `std::net` ・ `std::process` ・ `std::time::SystemTime`）に clippy が0件 |
| 基盤は AI を起動しない。AI が判定する rule は Semantic Schema Validator が依頼の一覧（needs）として返し、使う場面がポート（rule の判定を依頼する）で依頼して、答えを Validator の2段目に渡す | AI の起動が domain や Validator の中に入ると、判定の入力と出力が値として残らず、決定的な試験ができない | domain の crate に outbound の crate への辺が無い（architecture-lint）。Validator の公開関数が値だけを受け取る（単体テスト） |
| 起動時の結合は `crates/inbound/cli/src/compose.rs` の1か所に置く。`main.rs`（CLI）と `src/bin/mcp.rs`（MCP）は、引数を受け取って compose を呼ぶだけにする | 結合が2か所にあると、片方だけ直したときに振る舞いが食い違う。原典：「Rust projects that provide a binary have a straightforward _src/main.rs_ file that calls logic that lives in the _src/lib.rs_ file」（The Rust Programming Language 11.3） | ① outbound の外から outbound の crate に依存するのは cli だけ。outbound どうしの辺は architecture.json のとおり（architecture-lint）② cli の中で outbound の crate を `use` するのは compose.rs だけ（追跡の検査の grep） |

## 2. ワークスペースと crate

| 規則 | なぜ | 検査 |
|---|---|---|
| ワークスペースは virtual manifest にし、メンバーを `crates/` の下に層ごとのフォルダで置く（`members = ["crates/domain/*", "crates/application", "crates/outbound/*", "crates/inbound/*"]`）。`resolver` を明示する | 原典は「Keep all member packages in a flat directory (commonly `crates/`)」と平らを勧める。ここでは層（domain ・ outbound ・ inbound）をフォルダで見せるために、原典から外して2段にする（ACDR 0201 で承認した形）。原典：「resolver must be set explicitly in virtual workspaces」（Cargo Book Workspaces） | `cargo metadata --no-deps` のメンバーが crates/ の下だけ |
| 版 ・ edition ・ rust-version（1.97 以上。`build.warnings` が使える版）・ license と依存の版は `[workspace.package]` と `[workspace.dependencies]` の1か所に置き、メンバーは `workspace = true` で受け継ぐ | 版が2か所にあると、片方だけ上がって食い違う | メンバーの Cargo.toml に版の数字が0件 |
| lint の段階は `[workspace.lints]` に置き、メンバーは `[lints] workspace = true` だけを書く。lint の設定値は根の `clippy.toml`（試験のコードでの unwrap ・ expect ・ panic を許す `allow-unwrap-in-tests` など）に置き、domain の入出力の禁止は `crates/domain/clippy.toml` に置く（根の設定値も写す。Clippy は近いほうの1つだけを読む） | lint の段階が crate ごとに違うと、同じ書き方が crate によって通ったり落ちたりする。`workspace = true` と自分の `[lints]` を両方書くと Cargo が止まる（Cargo の実装 commit b5786332 のエラー「cannot override `workspace.lints` in `lints`」。Cargo Book には無い） | メンバーの Cargo.toml すべてに `[lints] workspace = true` があり、ほかの lint の行が0件 |
| 共有の crate（common ・ shared ・ utils）を作らない。2つ以上の crate が使う値は、その語の規則を持つ crate に置く | 共有の crate は持ち主の無い型の置き場になり、依存の辺が中心から外れる | crates/ の下に common ・ shared ・ utils の名前のフォルダが0件（追跡の検査の grep） |
| crate の名前は `semantic-agent-schema-<名前>`（例：`semantic-agent-schema-declaration`）。フォルダの名前は `<名前>` | ACDR 0201 で承認した名前。原典：「Crate names should not use -rs or -rust as a suffix or prefix」（Rust API Guidelines C-CASE）。フォルダで層が分かるので、フォルダに接頭辞は必要ない | architecture.json の components と Cargo.toml の name が一致 |

## 3. domain の crate の中

| 規則 | なぜ | 検査 |
|---|---|---|
| domain の crate の中は `values/`（値オブジェクト）・ `aggregates/`（集約）・ `services/`（ドメインサービス）に分ける。エンティティかファクトリを持つ集約は、集約の名前のフォルダ `aggregates/<集約>/` に `entities/` と `factories/` を置き、集約とエンティティの欄 ・ エンティティを作る関数を `pub(in crate::aggregates::<集約>)` にする。2つ以上の集約が持つエンティティだけ `aggregates/entities/` に置き `pub(in crate::aggregates)` にする | 原典：「pub(in path) makes an item visible within the provided path」「If an item is private, it may be accessed by the current module and its descendants.」（The Rust Reference, Visibility and privacy）。集約のフォルダの中にだけ見せれば、ファクトリは欄で集約を作れ、別の集約や services/ からは書き換えられない（実測：E0616 ・ E0624）。crate の直下の factories/ からは pub(crate) にしないと作れず、crate のどこからでも書き換えられる（実測 E0451） | 集約とエンティティの struct の欄に `pub` ・ `pub(crate)` ・ `pub(in crate::aggregates)`（共有のエンティティを除く）が0件（追跡の検査。syn でソースを解析する） |
| 集約の状態を変えるのは、集約のコマンド（メソッド）だけにする。欄は private で、読むのは借用を返すメソッド（`get_` を付けない）で行う | 原典：「the get_ prefix is not used for getters」（C-GETTER）。欄を開くと、どこからでも不変条件を壊せる | 上と同じ |
| 2つ以上の集約が持つエンティティ（aggregates/entities/）の欄は、持ち主の集約のコマンドからだけ書き換える | このエンティティだけは見せる範囲が aggregates/ 全体で、コンパイラが止めない | レビュー |
| ドメインサービスは `services/` に1つ1ファイルで置く（`resolver.rs` ・ `validator.rs` ・ `renderer.rs` ・ `selector.rs`） | ドメインサービスは集約をまたぐ処理で、集約を作るときは集約の公開の関数を使う | この4つのファイルが割り当てた crate の services/ にある（追跡の検査の grep） |
| 値オブジェクトは `values/` に1つ1ファイルで置き、中身は private にして `new` で作る | 値オブジェクトは自分で正しさを守り、ほかから中身に触らない | 値オブジェクトの struct の欄に `pub` が0件（追跡の検査。syn） |
| モジュールのファイルの形を `foo.rs` と `foo/` の1つにそろえ、`foo/mod.rs` の形を混ぜない | Rust はどちらの形も受け付けるが、`foo.rs` と `foo/mod.rs` を両方置くことはできない。原典：「It is encouraged to use the new naming convention as it is more consistent, and avoids having many files named mod.rs within a project.」（The Rust Reference, Modules）。mod.rs が禁止されているわけではない | crates/ の下に `mod.rs` が0件 |
| 不変条件を守る関数の doc コメントに `不変条件 I番号` を書く | どの不変条件をどの関数が守るかを、コードからたどれるようにする | `test-suite/trace.json` の不変条件の id が、それぞれ doc コメントか試験の traces に1回以上ある（追跡の検査） |

## 4. application

| 規則 | なぜ | 検査 |
|---|---|---|
| `src/use_cases/` はコマンドの組ごとに1ファイル、`src/ports/` は駆動されるポート13に1ファイルずつ置く（ACDR 0201 の形） | 使う場面とポートのファイルがコマンドの組とポートの一覧からずれると、決めていない入口や実装の無いポートが生まれる | ファイルの一覧と `test-suite/trace.json` のコマンドの組 ・ ポートの一覧が一致（追跡の検査） |
| 使う場面の1関数は「ポートで読む → 集約1つのコマンドかドメインサービス → ポートで書く」で終える | 1つの関数で2つの集約を書き換えると、片方だけ書けたときの状態が決まらない | レビュー |
| ポートの trait は application が定義し、outbound が実装する。application の中で自分が実装する trait は作らない | 実装が1つしかない trait は、差し替えがあると読む人に思わせる | application の中に、自分の trait への `impl … for` が0件（試験の代役を除く） |
| 手元のセッションの索引の型は `src/ports/sessions.rs` に置く | session と change の両方の id を持ち、domain のどの crate にも置けない | 追跡の検査（型がこのファイルにある） |

## 5. 名前

| 規則 | なぜ | 検査 |
|---|---|---|
| 識別子は RFC 430 の書き方にする（モジュール snake_case ・ 型 UpperCamelCase。略語も1語：`Ulid`） | 原典：Rust API Guidelines C-CASE | clippy が0件 |
| 集約 ・ エンティティ ・ 値オブジェクト ・ ドメインサービスの英語の型名と日本語の名前の対応は、型の doc コメントの1行目（日本語の名前）だけで持つ。別の対応表を作らない | 対応表をコードの外に持つと、コードと2か所になり、どちらが正しいかを毎回確かめることになる | 公開する型に doc コメントが無いものが0件（`missing_docs` を deny） |
| 変換は `as_` ・ `to_` ・ `into_` の決まりに従う | 原典：C-CONV | レビュー |

## 6. 失敗と panic

| 規則 | なぜ | 検査 |
|---|---|---|
| ライブラリの crate で `unwrap` ・ `expect` ・ `panic!` を使わない。失敗は `Result` で返す | panic すると CLI が値（JSON の結果と終了コード）を返さずに落ち、呼ぶ側の AI と CI が理由を受け取れない | `[workspace.lints.clippy]` の `unwrap_used` ・ `expect_used` ・ `panic` を deny。試験のコードは根の `clippy.toml` の `allow-unwrap-in-tests` ・ `allow-expect-in-tests` ・ `allow-panic-in-tests` で許す |
| `Result` を返す公開関数は、doc コメントに `# Errors` の節を書く | 原典：「Error conditions should be documented in an "Errors" section」（C-FAILURE） | clippy の `missing_errors_doc` を deny |
| 外から受け取る値の検査①（入力が入力のスキーマに合うか）で外れたら断り、状態に触れない。検査②（出力が形と rule に合うか）で外れても保存し、violated で返す | ACDR 0201 と Q8（保存は断らない） | システムテスト（①は `unchanged`、②は violated を期待する場面） |

## 7. テスト

| 規則 | なぜ | 検査 |
|---|---|---|
| 単体テストは確かめるコードと同じファイルの `#[cfg(test)] mod tests` に置く。domain の単体テストは代役を使わない | 原典：「put unit tests in the src directory in each file with the code that they're testing」（The Rust Programming Language 11.3）。domain は値の関数なので代役を必要としない | domain の crate の dev-dependencies に代役の crate が0件 |
| 結合テストは crate ごとに `tests/<名前>/main.rs` を1つ置き、中をモジュールに分ける | 原典：「Cargo will compile each of the files as an individual crate」。ファイルごとに crate ができると、ビルドが遅くなり補助が重複する | crate の tests/ の直下に .rs のファイルが0件 |
| 代役は `crates/application/tests/use_cases/doubles/` にポート1つ1ファイルで置く。保存のポートは `FakeXxx`、外から答えを受け取るポートは `StubXxx` にする。呼び出しの順は確かめない | 呼び出しの順を確かめる試験は、実装を変えるたびに落ちる | 代役の型名が Fake か Stub で始まる |
| テストの関数の名前は「条件_期待」の英語の snake_case にし、doc コメントに `/// traces: I45, U27:事後` を1行書く。本体に if ・ for ・ match を書かない | どの試験が落ちたら、どの不変条件と使う場面のリスクが残るかを引けるようにする。条件で分かれる試験は、何を確かめたかが読めない | 追跡の検査 |
| 決定表は1列に1テスト、状態遷移は1テストに無効な遷移1つ | 1つの試験で複数を確かめると、どれが落ちたかが分からない | レビュー |
| outbound の ai-cli の結合テストは、AI ツールの出力を記録したファイルで走らせる。記録は AI ツールの版を上げたときと月に1回取り直し、取り直した日と版を記録のファイルに書く | 本物の AI ツールを毎回呼ぶと、遅く、結果が揺れる。記録が古いと、AI ツールの出力の形が変わっても気づけない | 記録のファイルの取り直した日が31日以内（追跡の検査） |
| システムテストは `test-suite/scenarios/*.json`（given ・ when ・ after ・ then ・ unchanged）に書き、`crates/inbound/cli/tests/system/` が1場面を1件のテストとして走らせる。場面ごとに一時ディレクトリを作り、HOME と XDG もその下に向ける。本物の AI ツールは呼ばない | 場面どうしが手元の DB を共有すると、走らせる順で結果が変わる | 夜間に逆順で走らせて同じ結果 |
| 試験がまだ無い不変条件と使う場面の行は `test-suite/untested.json` に、why と move_when を付けて置く | 試験の無い行を黙って残さない | 追跡の検査：traces と untested.json の両方にある行 ・ どちらにも無い行 ・ trace.json に無い id を落とす |

## 8. 書式

| 規則 | なぜ | 検査 |
|---|---|---|
| 書式は rustfmt の既定（Rust style guide）にし、`rustfmt.toml` には `style_edition` だけを書く | 原典：「it is recommended to specify the style_edition in a rustfmt.toml configuration file」。cargo fmt と rustfmt で既定の edition が違う | `cargo fmt --check` が0件 |

## 9. 1つの変更を終えてよい条件

警告で止める方法は、原典の推奨に合わせて `build.warnings` にする。原典：「Since Cargo 1.97, the recommended way to do this is by using the build.warnings config option」「this will invalidate build caches and is therefore not recommended」（Clippy usage）

次の8段を、この順にすべて0件で通す。CI も同じ順で、段を分けて走らせる。

| 段 | コマンド |
|---|---|
| 1 | `cargo fmt --check` |
| 2 | `architecture-lint check --rules architecture.json --cargo .` |
| 3 | `cargo clippy --workspace --all-targets`（警告で止めるのは `.cargo/config.toml` の `build.warnings = "deny"`） |
| 4 | crate ごとの単独ビルド（`cargo build -p <crate>`）と `cargo tree -e features` |
| 5 | 追跡の検査と test-suite の形の検査 |
| 6 | `cargo test --workspace --lib`（単体） |
| 7 | `cargo test --workspace --test '*' --exclude semantic-agent-schema-cli`（cli を除く結合。`--test` は glob を受ける。`'*'` で全部を選ぶのは原文の例に無い推論で、導入時に実測で確かめる） |
| 8 | `cargo test -p semantic-agent-schema-cli --test system`（システム。`harness = false`） |

加えて、`untested.json` の行が増えていないこと。欠陥を直す変更には、直す前に落ちる試験が1件あること。

## 追跡の検査

`crates/inbound/cli/tests/system/trace.rs` に置く。`test-suite/trace.json`（不変条件 ・ 使う場面 ・ コマンドの組 ・ ポートの id と名前の一覧）を読み、ソースを syn（dev-dependency）で解析し、パスの一覧を grep する。ddd-advisor は xtask での検査を勧めたが、ツールを増やさないために qa-advisor の勧めた system テストの1モジュールにまとめた。

## 参照

ドメインモデルの設計の記録（集約 ・ 値オブジェクト ・ 不変条件の文）は、このリポジトリの外（ボードの model.json）に経緯として残す。実装が始まったあとの正本はコードと試験である。

| 場所 | 中身 |
|---|---|
| `test-suite/trace.json` | 不変条件 ・ 使う場面 ・ ポートの id と名前の一覧。追跡の検査が読む |
| `architecture.json` | architecture-lint の規則（許す辺） |
| `schemas/v1/` | Abstract の13ファイルと views/（既定のビュー） |
| `test-suite/` | システムテストの場面と untested.json |
