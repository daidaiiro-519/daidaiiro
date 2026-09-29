# 言語の組：Rust

## 目的

道具の契約（`references/tool-contract.md`）を、**Rust で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を持つ（ACDR 0036）。

---

## 置き場所

```
tool/           道具のソース
  Cargo.toml    workspace ── 4つの crate を並べる
  parts/        部品 ── 実体。読み込まれるもの
    tests/      事例
  declare/      能力の宣言（正本）と契約の実体（contract.rs）
  cli/          唯一の入口
  mcp/          MCP の面
bin/            組み立てた実行ファイル（git で追跡しない）
  <名前>        CLI（Windows は <名前>.exe）
  <名前>-mcp    MCP サーバー（Windows は <名前>-mcp.exe）
tool.json       CLI の起動のコマンドと、外部の道具
mcp.json        MCP の起動のコマンド（ホストの形式）
.gitignore      bin/ と tool/target/ を追跡しない
```

**道具のソースの置き場所の名前は、中身の役割で付ける。** 以前の `rs/` は言語の名前で、役割を示さなかった。

| crate | 層 | 参照してよい先 |
|---|---|---|
| `parts` | 部品 | **無し** |
| `declare` | 宣言 | `parts` |
| `cli` ・ `mcp` | 入口 | `declare` |

**層の境界を crate の境界に置く。** 1つの crate の中の module では、内側が外側を参照してもコンパイラが通す
── crate に分けて初めて、**宣言に無い依存が解決しなくなる**。許可辺は各 `Cargo.toml` が宣言する。

**入口は `bin/` に置く。Skill のフォルダは、実行ファイルの1つ上（`bin/` の親）である。**
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
| 宣言 | `tool/declare/` の `tools()`。MCP の面で `#[tool]` マクロを使い、道具をその場で宣言しない |
| 動詞なしの `--json` | CLI が `catalog()` の結果を返す（`contract.rs`） |
| 外部の道具 | 宣言の層が `given.external("名前")` で `tool.json` から読み、部品の関数へ引数として渡す。**部品は `Command::new("…")` に名前を直書きしない** |
| 標準出力 | 道具の中で `println!` を使わない。子プロセスの出力は `Stdio::piped()` かファイルで受ける |
| 子プロセスの規律 | `.stdin(Stdio::null())` ・ `try_wait` で待ち、制限時間を過ぎたら `kill` ・ `.stdout(…)` と `.stderr(…)` を指定する |
| 誤りの返し方 | `ok` が偽なら `CallToolResult` の `is_error` を立てる（`mcp.main.rs.tmpl`） |

---

## 雛形

`references/profiles/rust/` に置く。`skills-creator scaffold` がこの一式を置く。
**`contract.rs` ・ `cli` ・ `mcp` は Skill をまたいで同一である**ので、正本をここに置く。

| 雛形 | 置く先 |
|---|---|
| `workspace.Cargo.toml.tmpl` | `tool/Cargo.toml` |
| `parts.Cargo.toml.tmpl` ・ `parts.lib.rs.tmpl` ・ `parts.tests.rs.tmpl` | `tool/parts/` |
| `declare.Cargo.toml.tmpl` ・ `declare.lib.rs.tmpl` ・ `contract.rs.tmpl` | `tool/declare/` |
| `cli.Cargo.toml.tmpl` ・ `cli.main.rs.tmpl` | `tool/cli/` |
| `mcp.Cargo.toml.tmpl` ・ `mcp.main.rs.tmpl` | `tool/mcp/` |
| `tool.json.tmpl` ・ `mcp.json.tmpl` ・ `gitignore.tmpl` | Skill のフォルダ |

---

## 2段目の検査

`skills-creator check` は、`tool/Cargo.toml` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 層が crate に分かれていない | `tool/Cargo.toml` が無い |
| 層の crate が無い ／ 入口の crate が無い | 契約の一式が完備していない |
| どの層か決まらない | crate の名前が層の名前で終わっていない |
| 許可していない辺を宣言している | 内側の crate が外側を依存に宣言している |
| 事例が無い | `tool/parts/tests/` が無い |
| Python が残っている | `scripts/` が在る ── 道具は `tool/` が持つ |
| rs/ が残っている | 道具のソースを `tool/` へ移していない |
| bin/ を git の追跡から外していない | `.gitignore` に `bin/` が無い |
| 部品が外部の道具の名前を直書きしている | `Command::new("…")` に名前が書いてある。`tool.json` に宣言し、注入する |

依存が宣言どおりに守られているかは、コンパイラが判定する。**見つけるが、直さない。**
