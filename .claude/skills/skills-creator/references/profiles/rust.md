# 言語の組：Rust

## 目的

道具の契約（`references/tool-contract.md`）を、**Rust で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を持つ（ACDR 0036）。

---

## 置き場所

```
tool/           道具のソース
  Cargo.toml    workspace ── 5つの crate を並べる
  data_access/  データアクセス層 ── ファイル ・ 外部の道具 ・ 通信の入出力（files.rs ・ process.rs は雛形の複製）
  business_logic/  業務ロジック層 ── 判定と変換の実体。入出力を禁じる
    tests/      事例
  service/      サービス層 ── 道具の一覧（能力の正本）と契約の実体（contract.rs）
  cli/          プレゼンテーション層 ── シェルから呼ぶ唯一の経路
  mcp/          プレゼンテーション層 ── MCP の面
bin/            組み立てた実行ファイル（git で追跡しない）
  <名前>        CLI（Windows は <名前>.exe）
  <名前>-mcp    MCP サーバー（Windows は <名前>-mcp.exe）
tool.json       CLI の起動のコマンドと、外部の道具
mcp.json        MCP の起動のコマンド（ホストの形式）
.gitignore      bin/ と tool/target/ を追跡しない
```

**道具のソースの置き場所の名前は、中身の役割で付ける。** 以前の `rs/` は言語の名前で、役割を示さなかった。
crate のフォルダの名前は、レイヤードアーキテクチャの層の正式名に合わせる（以前の `parts/` ・ `declare/` は、層の名前と一致しなかった）。

以下は雛形が採る構成である（契約ではない。`references/tool-contract.md` の「雛形が採る構成」）。

| crate | 層 | 依存してよい先 |
|---|---|---|
| `data_access` | データアクセス層 | **無し** |
| `business_logic` | 業務ロジック層 | `data_access`（外の crate は、references の実装が使う serde_json と jsonschema だけ） |
| `service` | サービス層 | `business_logic` |
| `cli` ・ `mcp` | プレゼンテーション層 | `service` |

**層の境界を crate の境界に置く。** 1つの crate の中の module では、内側が外側を参照してもコンパイラが通す
── crate に分けて初めて、**`Cargo.toml` に書いていない依存が解決しなくなる**。依存の向きは各 `Cargo.toml` が宣言する。

**実行ファイルは `bin/` に置く。Skill のフォルダは、実行ファイルの1つ上（`bin/` の親）である。**
求める処理は `contract.rs`（`Given::skill_root`）が持ち、すべての Skill が同じ実装を使う。
**build のときの絶対パスを埋め込まない** ── 組み立てた機械のパスが残り、別の機械では存在しない場所を指す（ACDR 0029）。

**開発時も `bin/` へ置く。** Skill のフォルダで次を実行する。

```
cargo install --path tool/cli --root . --target-dir tool/target
cargo install --path tool/mcp --root . --target-dir tool/target
```

---

## 契約の実装

| 契約の規定 | Rust の組での実装 |
|---|---|
| 道具の一覧 | `tool/service/` の `tools()`。MCP の面で `#[tool]` マクロを使い、道具をその場で宣言しない |
| 動詞なしの `--json` | CLI が `catalog()` の結果を返す（`contract.rs`） |
| 外部の道具 | サービス層が `given.external("名前")` で `tool.json` から読み、業務ロジック層の関数へ引数として渡す。起動するのはデータアクセス層の `process::run` ・ `process::Session` である。**どの層も `Command::new("…")` に名前を直書きしない** |
| 入出力 | 業務ロジック層とサービス層は `std::fs` ・ `std::process` ・ `std::net` を直接呼ばない。業務ロジック層は、`lib.rs` に置いた公開しない別名 `use <接頭辞>_data_access as data_access;` を経由して `data_access::files` ・ `data_access::process` を呼ぶ ── 別名を公開すると、上の層がこの crate を経由してデータアクセス層へ届く |
| 標準出力 | 道具の中で `println!` を使わない。子プロセスの出力は `Stdio::piped()` かファイルで受ける |
| 子プロセスの規律 | データアクセス層の `process.rs` が1回だけ実装する ── `.stdin(Stdio::null())` ・ `try_wait` で待ち、制限時間を過ぎたら `kill` ・ 標準出力と標準エラーを上限（`MAX_OUTPUT`）までだけ受け取る |
| 誤りの返し方 | `ok` が偽なら `CallToolResult` の `is_error` を立てる（`mcp.main.rs.tmpl`） |

---

## 雛形

`references/profiles/rust/` に置き、組の定義 `references/profiles/rust.profile.json` が、何をどこへ置くかを持つ（ACDR 0060）。
`skills-creator scaffold <名前> --type <型> --language rust` が、共通の一式と型ごとの一式を合わせて置く。
**`contract.rs` ・ `files.rs` ・ `process.rs` ・ `refs.rs` ・ `cli` ・ `mcp` は Skill をまたいで同一である**ので、正本をここに置く。

| 置き場所 | 中身 | 置く先 |
|---|---|---|
| `common/` | 全ての型に置く共通の一式 ── workspace ・ データアクセス層（`files.rs` ・ `process.rs`）・ 業務ロジック層とサービス層の Cargo.toml ・ `contract.rs` ・ references の実装（`refs.rs` ・ `service.refs.rs`）・ CLI ・ MCP ・ `tool.json` ・ `mcp.json` ・ `.gitignore` ・ `document.schema.json` | `tool/` と Skill のフォルダ |
| `types/work/` | 作業型の一式 ── 業務ロジック層とサービス層の `lib.rs`、見本の道具 `hello`、その事例 | `tool/business_logic/` ・ `tool/service/` |
| `types/<型>/` | その型の道具のコード。**ここに無い型は、この組では生めない** | 型の定義による |

**生んだ直後に組み立てられる。** 雛形は、コードの識別子の位置に差し込み場所を置かない。見本の道具 `hello` を書き換えて、この Skill の道具にする。

---

## 2段目の検査

`skills-creator check` は、`tool/Cargo.toml` が在る Skill にこの組の検査を適用する。

**既定では、契約に関わるものだけを検査する**（ACDR 0059）。

| 検出 | 何が起きているか |
|---|---|
| 外部の道具の名前を直書きしている | `Command::new("…")` に名前が書いてある。`tool.json` に宣言し、注入する |

**`--layout 1` を渡すと、雛形の構成も検査する。** 雛形の構成を保つと決めたリポジトリが、自分で選んで有効にする。

| 検出 | 何が起きているか |
|---|---|
| 層が crate に分かれていない | `tool/Cargo.toml` が無い |
| 層の crate が無い ／ プレゼンテーション層の crate が無い | 雛形の一式が完備していない |
| どの層か決まらない | crate の名前が層の名前で終わっていない |
| 依存の向きに違反している | 下の層の crate が上の層を依存に宣言している。または、層を飛ばして依存を宣言している |
| 事例が無い | `tool/business_logic/tests/` が無い |
| Python が残っている | `scripts/` が在る ── 道具は `tool/` が持つ |
| rs/ が残っている | 道具のソースを `tool/` へ移していない |
| parts/ か declare/ が残っている | crate のフォルダを層の正式名（`business_logic/` ・ `service/`）へ改めていない |
| bin/ を git の追跡から外していない | `.gitignore` に `bin/` が無い |
| 入出力を禁じた層が入出力を直接扱っている | 業務ロジック層かサービス層（`contract.rs` を除く）の行に、`std::fs` ・ `std::process` ・ `std::net` ・ `Command::new` ・ `.is_file()` などが在る。行ごとに出す。文字列の中と注記の行は数えない |
| references の実装が無い（版2） | `tool/business_logic/src/refs.rs` か `tool/service/src/refs.rs` が無い |
| references の実装が雛形と違う（版2） | `tool/business_logic/src/refs.rs` が `refs.rs.tmpl` と一致しない。雛形から複製し直す |

依存が `Cargo.toml` どおりに守られているかは、コンパイラが判定する。**見つけるが、直さない。**
