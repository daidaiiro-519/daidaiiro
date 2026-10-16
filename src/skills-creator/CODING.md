# skills-creator のコーディング規則

このファイルを置いたフォルダ（`src/skills-creator/`）より下のコードは、この規則に従う。下のフォルダに別の CODING.md があれば、近いほうに従う。

基盤（`src/schema-driven/CODING.md`）と同じ設計標準に従い、skills-creator に固有のこと（port の一覧 ・ 外のプログラムの呼び方 ・ 試験の置き場所）だけを変える。

規則の1件は、**なぜ（守らないと何が起きるか）** と **検査（機械で見る方法）** を持つ。どちらも書けない規則は置かない。

---

## アーキテクチャ

ポートとアダプタ（ヘキサゴナル）で作る。中心は domain と application で、外側との境界は port の trait である。

| 層 | 持つもの | 依存してよい先 |
|---|---|---|
| domain | 業務の規則（SKILL.md の読み取り ・ check の項目と結果 ・ 公開された基盤の配布物の名前と SHA-256 の照合） | 無し |
| application | ユースケース（scaffold ・ engine ・ check） | domain ・ port |
| port | 出ていく側の trait（Files ・ Programs ・ Releases）。application が定義し、adapters が実装する | domain |
| inbound adapter | CLI の受け口と、操作の一覧と振り分け | application |
| outbound adapter | port の実装（ファイル ・ プログラムの起動 ・ gh による取得と照合 ・ 配布物の展開） | port ・ domain |
| 起動時の結合 | outbound adapter を application へ渡し、受け口に渡す | adapters の中のすべて |

| 規則 | なぜ | 検査 |
|---|---|---|
| port と呼ぶのは、application が定義して adapters が実装する、出ていく側の trait だけにする | core が自分で実装する trait まで port と呼ぶと、どれが外との境界なのかがコードから読めなくなる | core の中に、port の trait への `impl … for` が0件（`src/tests.rs` のテストの代役を除く） |
| application が自分で実装するものは trait にせず、型と関数として公開する | 実装が1つしかない trait は、読む人に「差し替えがある」と思わせ、呼び出しを追う手間だけが増える | core の中で定義と実装が core にある trait が0件 |
| inbound adapter は application だけを呼ぶ。outbound adapter を直に呼ばない | 受け口が出ていく側を直に呼ぶと、業務の規則が application を通らなくなる | `adapters/src/inbound/` に `outbound::` が0件 |
| 起動時の結合は、adapters の1か所（`compose`）に置く | 結合が2か所にあると、片方だけ直したときに振る舞いが食い違う | 結合を書いた関数が1つだけ |
| domain と application は、ファイル ・ 通信 ・ プロセス ・ 時刻に直接触らない | 中心が入出力に触ると、入出力の代役を渡して単体テストで確かめられなくなる | `crates/core/clippy.toml` が `std::fs` ・ `std::net` ・ `std::process` ・ `std::time::SystemTime` を禁じ、clippy が0件 |
| 外のプログラム（基盤のエンジン ・ gh ・ tar ・ 入口）は、outbound adapter からだけ起動する。core はその名前と引数を Programs と Releases に渡す | 起動のしかた（OS ごとの違い ・ 見つからないとき）が core に混ざると、代役で確かめられなくなる | `crates/` の中で、`std::process::Command` が `adapters/src/outbound/` と `adapters/tests/`（端から端までの試験）の外に0件 |
| 証明の照合を飛ばす経路を作らない。gh が無いか古いときは止める | 照合を飛ばす旗を置くと、それを付けた配布物が来歴を確かめないまま入る | `--no-verify` ・ `skip` を含む旗が0件 |

---

## crate とフォルダ

```
src/skills-creator/
  Cargo.toml          ワークスペース（メンバーは crates/* と xtask）。skills-creator の版の1か所（[workspace.package] の version）
  Cargo.lock          依存の版の1か所
  CODING.md           この規則
  .cargo/config.toml  cargo xtask の alias と、Windows の実行ファイルの設定
  crates/
    core/             domain ・ application ・ port（ライブラリ）
      clippy.toml
      src/
        lib.rs
        domain/       skill_md.rs ・ report.rs ・ release.rs
        application/  scaffold.rs ・ engine.rs ・ check.rs
        ports.rs      出ていく側の port（Files ・ Programs ・ Releases）
        tests.rs      単体テストが共有する代役（port をメモリ上で実装したもの）
    adapters/         inbound ・ outbound ・ 起動時の結合（ライブラリ）と実行ファイル
      src/
        lib.rs
        compose.rs
        inbound/      cli.rs
        outbound/     fs.rs ・ process.rs ・ release.rs（gh で取得と照合、tar で展開）
        bin/          skills-creator.rs
      tests/          結合テストと、少数の端から端までのテスト
        support/      テストの補助
  xtask/              ツール（製品に入らない）。skills-creator の Skill のフォルダを作り（skill）、確かめる（verify）
```

雛形（SKILL.md ・ 入口 ・ 入口の .cmd）と型のスキーマ（`skills-creator/skill@v1`）は、ソースではなく Skill のフォルダの `assets/` が持ち、実行ファイルは読むだけである。

| 規則 | なぜ | 検査 |
|---|---|---|
| 製品の crate は core と adapters の2つにする | 層ごとに crate を分けても、守りたい依存の向きは core と adapters の境界で Cargo が強制できる | `crates/` の下が core と adapters の2つ |
| 製品に入らない作業（Skill のフォルダを作る ・ 確かめる）は xtask に置く。xtask は skills-creator をビルドして置き、基盤のエンジンは skills-creator の engine で入れる | エンジンを入れる手順を xtask にも書くと、手順が2か所に分かれる | xtask に `gh` ・ `attestation` の文字列が0件 |
| skills-creator の版は、Cargo.toml の [workspace.package] の version の1か所に書く | 版を2か所に書くと、公開した版と実行ファイルが答える版が食い違っても気づけない | `version = "` で始まる行が、Cargo.toml の [workspace.package] の1行だけ（依存の宣言を除く） |
| 実行ファイル（`src/bin/`）は、`compose` と受け口を呼ぶだけにする | 実行ファイルに処理を書くと、結合テストから呼べない | `src/bin/` に `impl` ・ `match` ・ `if` が0件 |
| ファイルの場所を相対パスの文字列で書かない。Skill のフォルダは実行ファイルの場所から決める | ソースのファイルを動かしただけで壊れる。作業ディレクトリから決めると、別の場所から呼ばれたときに別のフォルダを読む | `../` を含む文字列のパスが0件。`std::env::current_dir` が0件 |
| core は adapters と、外のプログラムを起動する crate に依存しない | core が外の技術に依存すると、その技術を替えるときに業務の規則まで直すことになる | `cargo tree -p skills-creator-core` に adapters が現れない |

---

## trait とジェネリクス

| 規則 | なぜ | 検査 |
|---|---|---|
| port の trait は、ジェネリクスの境界として使うか dyn で使うかを、doc コメントの1行目に書く | 後から dyn とジェネリクスを混ぜると、同じ trait の呼び方が場所によって変わる | port の trait の doc コメントに「ジェネリクスで使う」か「dyn で使う」がある |
| Files ・ Programs ・ Releases は、1つの実行ファイルの中で実装が1つに決まるので、ジェネリクスの境界で使う | 実装が1つに決まるのに dyn で渡すと、型の上でも実装が1つだと読めない | `dyn` の使用が0件 |
| 値の制約は型で表す（newtype）。bool や Option で意味を運ばない | 呼ぶ側が値の意味を取り違えてもコンパイラが止めない | レビューで見る（機械の検査は無い） |

最後の規則は機械で見られないので、レビューでだけ見る。この1件だけを例外にする。

---

## テスト

| 層 | テストの型 | 置き場所 |
|---|---|---|
| domain | 単体テスト。入力と、前もって計算した期待値の表で確かめる | 同じファイルの `#[cfg(test)] mod tests` |
| application | 単体テスト。port をメモリ上で実装した代役を渡して、正しい場合と壊した場合の両方を確かめる | 同じファイルの `#[cfg(test)] mod tests` |
| outbound adapter | 本物の入出力を使う単体テスト（作業用のフォルダ）。gh とネットワークを使うものは置かない | 同じファイルの `#[cfg(test)] mod tests` |
| inbound adapter | CLI の変換（引数 ↔ 呼び出し、結果 ↔ 出力）の単体テスト | 同じファイルの `#[cfg(test)] mod tests` |
| 実行ファイル | 起動して確かめる端から端までのテスト。代表の数件だけで、基盤のエンジンと gh を使わないもの | `crates/adapters/tests/` |
| 基盤のエンジンと gh を使うもの | 公開された基盤の取得と照合 ・ scaffold した Skill の check ・ 入口の起動（Windows の .cmd を含む） | ワークフローの verify と receive（`cargo xtask verify`） |
| xtask | 引数の読み方の単体テスト | 同じファイルの `#[cfg(test)] mod tests` |

| 規則 | なぜ | 検査 |
|---|---|---|
| テストの本体に、分岐 ・ ループ ・ 確かめる対象と同じ計算を書かない | テストの中に処理があると、そのテスト自体の正しさを別に確かめることになる | `#[test]` の関数の本体に `if` ・ `for` ・ `while` ・ `match` が0件 |
| テストの中で、ビルド ・ 外部コマンド（`gh` ・ `sha256sum` ・ `certutil`）・ ネットワークを使わない | テストの結果が、ビルドのやり方や環境の道具の有無で変わる | `tests/` と `mod tests` に `env!("CARGO")` ・ `"gh"` ・ `sha256sum` ・ `certutil` が0件 |
| 項目ごとに、合格する例と不合格になる例の両方を置く | 合格する例だけでは、何でも合格にする check でも通る | check の項目の id ごとに、`Pass` と `Fail` を期待する試験が1件以上ある |
| テストの補助は、結合テストは `tests/support/`、単体テストは crate の `src/tests.rs` の1か所に置く | 同じ補助が2か所にあると、片方だけ直したときに前提が食い違う | 補助の型の定義が、`tests/support/` と `src/tests.rs` の中だけにある |
| 製品のコードに、テストのためだけの関数 ・ 分岐 ・ pub を足さない。`#[cfg(test)]` は `mod tests` にだけ付ける | テストのためのコードが配る実行ファイルに入る | `#[cfg(test)]` が `mod tests` の外に0件 |

---

## 検査の順

変更をコミットする前に、次の順で実行し、すべて0件で通す。

| 順 | 何を | コマンド |
|---|---|---|
| 1 | 書式 | `cargo fmt --all --check` |
| 2 | lint（禁じた API を含む） | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| 3 | 依存の向き | `cargo tree -p skills-creator-core` に adapters が無い |
| 4 | この規則の grep の検査 | 各規則の「検査」の欄 |
| 5 | テスト | `cargo test --workspace --locked` |
| 6 | Skill のフォルダ | `cargo xtask skill --out <Skill のフォルダ> --engine <基盤の版>` のあと `cargo xtask verify --dir <Skill のフォルダ>`（gh とネットワークを使う） |

---

## 出典

基盤の CODING.md の出典（The Rust Programming Language 11.3 ・ 18.2、The Cargo Book、Rust API Guidelines、cargo-xtask の README）をそのまま使う。加えて次を使う。

| 原典 | 使った箇所 |
|---|---|
| ボード skills-creator-on-base（2026-10-13 決着） | 論点1（scaffold ・ engine ・ check と項目）、論点3（照合を飛ばさない ・ 失敗したら元のまま）、論点4（基盤と同じ形 ・ xtask はビルドだけ） |
| qa-advisor の回答（2026-10-13） | manual-intervention ・ acceptance-criteria：既定を判定不能にし、合格と不合格の両方を確かめる |
| platform-advisor の回答（2026-10-13） | sec-06 ・ ops-05 ・ ops-06：証明の照合、設計標準の共有、公開の手順の試験 |
