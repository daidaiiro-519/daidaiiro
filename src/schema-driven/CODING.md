# schema-driven のコーディング規則

このファイルを置いたフォルダ（`src/schema-driven/`）より下のコードは、この規則に従う。下のフォルダに別の CODING.md があれば、近いほうに従う。

規則の1件は、**なぜ（守らないと何が起きるか）** と **検査（機械で見る方法）** を持つ。どちらも書けない規則は置かない。

---

## アーキテクチャ

ポートとアダプタ（ヘキサゴナル）で作る。中心は domain と application で、外側との境界は port の trait である。

| 層 | 持つもの | 依存してよい先 |
|---|---|---|
| domain | 業務の規則（スキーマの契約 ・ 検証 ・ 参照 ・ 導出値 ・ 公開の記録 ・ デザインシステムの検査） | 無し |
| application | ユースケース（作る ・ 取る ・ 更新する ・ 消す ・ 検査する ・ 承認する ・ 描画する） | domain ・ port |
| port | 出ていく側の trait。application が定義し、adapters が実装する | domain |
| inbound adapter | CLI と MCP の受け口と、ツールの一覧と振り分け | application |
| outbound adapter | port の実装（ファイル ・ JMESPath ・ 部品のカタログ） | port ・ domain |
| 起動時の結合 | outbound adapter を application へ渡し、受け口に渡す | adapters の中のすべて |

| 規則 | なぜ | 検査 |
|---|---|---|
| port と呼ぶのは、application が定義して adapters が実装する、出ていく側の trait だけにする。port のメソッドが引数として受け取るコールバック（JMESPath に足す関数 ・ 描画の部品）は、core が実装して ports.rs に並べ、doc コメントに「port ではない」と書く | core が自分で実装する trait まで port と呼ぶと、どれが外との境界なのかがコードから読めなくなる | core の中に、port の trait への `impl … for` が0件（`src/tests.rs` のテストの代役を除く） |
| application が自分で実装するものは trait にせず、型と関数として公開する。差し替える実装が2つ以上あるときだけ trait にする | 実装が1つしかない trait は、読む人に「差し替えがある」と思わせ、呼び出しを追う手間だけが増える | core の中で定義と実装が core にある trait が0件（ports.rs のコールバックの2つを除く） |
| inbound adapter は application だけを呼ぶ。outbound adapter を直に呼ばない | 受け口が出ていく側を直に呼ぶと、業務の規則が application を通らなくなる | `adapters/src/inbound/` に `outbound::` が0件 |
| 起動時の結合は、adapters の1か所（`compose`）に置く。CLI と MCP の実行ファイルは、それを呼ぶ | 結合が2か所にあると、片方だけ直したときに CLI と MCP で振る舞いが食い違う | 結合を書いた関数が1つだけ |
| domain と application は、ファイル ・ 通信 ・ 時刻に直接触らない | 中心が入出力に触ると、入出力の代役を渡して単体テストで確かめられなくなる | `crates/core/clippy.toml` が `std::fs` ・ `std::net` ・ `std::process` ・ `std::time::SystemTime` を禁じ、clippy が0件 |

---

## crate とフォルダ

The Rust Programming Language 11.3 は「Rust projects that provide a binary have a straightforward src/main.rs file that calls logic that lives in the src/lib.rs file」と書く。The Cargo Book（Workspaces）は、メンバーを `crates/` の平らなフォルダに置くことを勧める。

```
src/schema-driven/
  Cargo.toml          ワークスペース（メンバーは crates/*）
  Cargo.lock          依存の版の1か所
  CODING.md           この規則
  crates/
    core/             domain ・ application ・ port（ライブラリ）
      schemas/        基盤のスキーマ。build.rs が取り込み、名前（$id）で引く
      build.rs
      clippy.toml
      src/
        lib.rs
        embedded.rs   取り込んだスキーマを名前で引く
        domain/
        application/
        ports.rs      出ていく側の port と、port が受け取るコールバック
        tests.rs      単体テストが共有する代役（port をメモリ上で実装したもの）
    adapters/         inbound ・ outbound ・ 起動時の結合（ライブラリ）と実行ファイル
      catalog/        部品のカタログ。build.rs が取り込み、ファイル名で引く
      build.rs
      src/
        lib.rs
        compose.rs
        tests.rs      単体テストが共有する補助（作業用のフォルダ）
        inbound/      tools.rs ・ cli.rs ・ mcp.rs
        outbound/     fs.rs ・ jmespath.rs ・ catalog.rs
        bin/          schema-driven.rs ・ schema-driven-mcp.rs
      tests/          結合テストと、少数の端から端までのテスト
        support/      テストの補助（作業用のフォルダ ・ 値の表を回す関数）
```

| 規則 | なぜ | 検査 |
|---|---|---|
| crate は core と adapters の2つにする。層ごとに crate を分けない | 層ごとに crate を分けても、守りたい依存の向きは core と adapters の境界で Cargo が強制できる。crate が増えると、同じ設定と依存の宣言を何か所にも書くことになる | ワークスペースのメンバーが2つ |
| 実行ファイル（`src/bin/`）は、`compose` と受け口を呼ぶだけにする。`impl` も分岐も書かない | 実行ファイルに処理を書くと、結合テストから呼べず（The Book 11.3 :237）、CLI と MCP で同じ処理を2回書くことになる | `src/bin/` に `impl` ・ `match` ・ `if` が0件 |
| 取り込むファイルは、それを使う層の crate が持つ。スキーマは core、カタログは adapters。build.rs がフォルダから一覧を作り、コードはファイルの場所を書かない | ファイルの場所を相対パスで書くと、ソースのファイルを動かしただけで壊れる。使わない層の crate が持つと、core が表示の部品まで抱えることになる | `include_str!` と `include_bytes!` が build.rs の出力の外に0件。`../` を含む文字列のパスが0件 |
| core は、adapters ・ rmcp ・ tokio ・ jmespath に依存しない | core が外の技術に依存すると、その技術を替えるときに業務の規則まで直すことになる | `cargo tree -p schema-driven-core` にそれらが現れない |

---

## trait とジェネリクス

Rust API Guidelines（flexibility）は「When designing a trait, decide early on whether the trait will be used as an object or as a bound on generics.」と書く。The Rust Programming Language 18.2 は、型の集まりが同じならジェネリクスと trait の境界を、利用者が型を足せるようにしたいなら trait object を勧める。

| 規則 | なぜ | 検査 |
|---|---|---|
| port の trait は、定義するときに、ジェネリクスの境界として使うか dyn で使うかを決め、doc コメントの1行目に書く | 後から dyn とジェネリクスを混ぜて使うと、同じ trait の呼び方が場所によって変わり、読み手が毎回確かめることになる | port の trait の doc コメントに「ジェネリクスで使う」か「dyn で使う」がある |
| 1つの実行ファイルの中で実装が1つに決まる port（ファイル ・ スキーマの読み込み ・ JMESPath）は、ジェネリクスの境界で使う | 実装が1つに決まるのに dyn で渡すと、実行時に呼び先を探す負荷がかかり、型の上でも実装が1つだと読めない（The Book 18.2 :119） | 該当する port が `dyn` で使われていない |
| 外から実装を足す port（部品のカタログ）と、port のメソッドが受け取るコールバック（JMESPath に足す関数 ・ 描画の部品）だけを dyn で使う | 実装を足していくものは、型が実行時に決まるので dyn が合う（The Book 18.2 :15） | `dyn` の使用が、Design ・ Functions ・ Renderer の3つの trait だけ |
| 値の制約は型で表す（newtype）。bool や Option で意味を運ばない（API Guidelines の C-NEWTYPE ・ C-CUSTOM-TYPE） | bool や Option で意味を運ぶと、呼ぶ側が値の意味を取り違えてもコンパイラが止めない | レビューで見る（機械の検査は無い） |

最後の規則は機械で見られないので、レビューでだけ見る。この1件だけを例外にする。

---

## テスト

The Rust Programming Language 11.3 は、単体テストを「in the src directory in each file with the code that they're testing」に置き、結合テストを `tests` に置いて「using only the public interface」で使う、と書く。

| 層 | テストの型 | 置き場所 |
|---|---|---|
| domain | 単体テスト。入力と、前もって計算した期待値の表で確かめる | 同じファイルの `#[cfg(test)] mod tests` |
| application | 単体テスト。port をメモリ上で実装した代役を渡して確かめる | 同じファイルの `#[cfg(test)] mod tests` |
| outbound adapter | 本物の入出力を使う単体テスト（作業用のフォルダ） | 同じファイルの `#[cfg(test)] mod tests` |
| inbound adapter | CLI と MCP の変換（引数 ↔ 呼び出し、結果 ↔ 出力）の単体テスト。同じ表で両方を確かめる | 同じファイルの `#[cfg(test)] mod tests` |
| 共通ツールの約束 | 公開 API の振り分け（`Toolbox::dispatch`）を、値の表で1回ずつ確かめる結合テスト | `crates/adapters/tests/` |
| 実行ファイル | 起動して確かめる端から端までのテスト。CLI と MCP で代表の数件だけ | `crates/adapters/tests/` |

| 規則 | なぜ | 検査 |
|---|---|---|
| テストの本体に、分岐 ・ ループ ・ 確かめる対象と同じ計算を書かない。期待値は前もって計算した値を並べる | テストの中に処理があると、そのテスト自体の正しさを別に確かめることになる | `#[test]` の関数の本体に `if` ・ `for` ・ `while` ・ `match` が0件 |
| テストの中で、ビルド（`cargo build`）・ 外部コマンド（`sha256sum` ・ `certutil`）を起動しない。実行ファイルの場所は `CARGO_BIN_EXE_<名前>` を使う | テストの結果が、ビルドのやり方や OS ごとのコマンドの有無に左右される | `tests/` と `mod tests` に `env!("CARGO")` ・ `sha256sum` ・ `certutil` が0件 |
| プロトコルのクライアントを手で書かない。MCP を起動して確かめるときは、使っている MCP のライブラリのクライアントを使う | 手で書いたクライアントは、製品の MCP と同じ間違いをしても見つけられない。テストの中にプロトコルの実装が1つ増える | `tests/` に JSON-RPC の文字列（`"jsonrpc"`）が0件 |
| テストの補助は、結合テストは `tests/support/`、単体テストは crate の `src/tests.rs` の、それぞれ1か所に置く。同じ補助を2か所に書かない | 同じ補助が2か所にあると、片方だけ直したときにテストどうしで前提が食い違う | 補助の型（作業用のフォルダなど）の定義が、`tests/support/` と `src/tests.rs` の中だけにある |
| 製品のコードに、テストのためだけの関数 ・ 分岐 ・ pub を足さない。`#[cfg(test)]` は `mod tests` にだけ付ける | テストのためのコードが配る実行ファイルに入り、テストの都合で製品の公開 API が広がる | `#[cfg(test)]` が `mod tests` の外に0件 |
| 結合テストから core の domain と application を直に呼ばない。結合テストは公開 API（振り分け）だけを使う | 結合テストが中の型に結びつくと、中を直すたびに結合テストまで壊れる。中の確かめは単体テストが持つ | `crates/adapters/tests/` に `schema_driven_core::domain` ・ `schema_driven_core::application` が0件 |

---

## 検査の順

変更をコミットする前に、次の順で実行し、すべて0件で通す。静的な検査を、動くテストより前に置く。

| 順 | 何を | コマンド |
|---|---|---|
| 1 | 書式 | `cargo fmt --all --check` |
| 2 | lint（禁じた API を含む） | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| 3 | 依存の向き | `cargo tree -p schema-driven-core` に adapters ・ rmcp ・ tokio ・ jmespath が無い |
| 4 | この規則の grep の検査 | 各規則の「検査」の欄 |
| 5 | テスト | `cargo test --workspace --locked` |

---

## 出典

| 原典 | 使った箇所 |
|---|---|
| The Rust Programming Language 11.3 Test Organization（rust-lang/book、2026-10-12 取得） | :6–7 単体と結合の違い、:19–20 単体テストの置き場所、:93 tests、:199–218 tests/common、:237–243 main.rs は薄く |
| The Rust Programming Language 18.2 Trait Objects（同） | :11 ・ :15 enum と trait object、:119 同じ型ならジェネリクス、:240–249 動的ディスパッチの実行時の負荷 |
| The Cargo Book：Package Layout ・ Cargo Targets ・ Workspaces ・ Build Scripts（2026-10-12 取得） | 標準の配置、`CARGO_BIN_EXE_<name>`、`crates/` の平らな置き方、build.rs と OUT_DIR |
| Rust API Guidelines（rust-lang/api-guidelines、2026-10-12 取得） | flexibility.md :163（trait を object か境界かで先に決める）、C-GENERIC ・ C-OBJECT ・ C-NEWTYPE ・ C-CUSTOM-TYPE |
| ddd-advisor の回答（2026-10-12） | ports-and-adapters ・ layered-architecture：port の定義、起動時の結合、受け口の置き場所 |
| qa-advisor の回答（2026-10-12） | test-levels ・ test-pyramid ・ conditional-test-logic ・ test-logic-in-production ・ hard-to-test-code：テストの型と置き場所、テストに書かないもの |
