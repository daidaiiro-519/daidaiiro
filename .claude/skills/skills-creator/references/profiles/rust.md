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
  business_logic/  業務ロジック層 ── 処理の実体。サービス層から読み込まれる
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

| crate | 層 | 依存してよい先 |
|---|---|---|
| `business_logic` | 業務ロジック層 | **無し**（外の crate は、references の実装が使う serde_json と jsonschema だけ） |
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
| 外部の道具 | サービス層が `given.external("名前")` で `tool.json` から読み、業務ロジック層の関数へ引数として渡す。**業務ロジック層は `Command::new("…")` に名前を直書きしない** |
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
| `business_logic.Cargo.toml.tmpl` ・ `business_logic.lib.rs.tmpl` ・ `business_logic.tests.rs.tmpl` | `tool/business_logic/` |
| `service.Cargo.toml.tmpl` ・ `service.lib.rs.tmpl` ・ `contract.rs.tmpl` | `tool/service/` |
| `cli.Cargo.toml.tmpl` ・ `cli.main.rs.tmpl` | `tool/cli/` |
| `mcp.Cargo.toml.tmpl` ・ `mcp.main.rs.tmpl` | `tool/mcp/` |
| `tool.json.tmpl` ・ `mcp.json.tmpl` ・ `gitignore.tmpl` | Skill のフォルダ |
| `refs.rs.tmpl` | `tool/business_logic/src/refs.rs` ── references の実装。**どの Skill も同じファイルを複製する**（契約の版2） |
| `service.refs.rs.tmpl` | `tool/service/src/refs.rs` ── get ・ validate ・ view ・ import を道具の一覧に載せる |
| `document.schema.json.tmpl` | `references/document.schema.json` ── 原典の複製の形 |

---

## 2段目の検査

`skills-creator check` は、`tool/Cargo.toml` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 層が crate に分かれていない | `tool/Cargo.toml` が無い |
| 層の crate が無い ／ プレゼンテーション層の crate が無い | 契約の一式が完備していない |
| どの層か決まらない | crate の名前が層の名前で終わっていない |
| 依存の向きに違反している | 下の層の crate が上の層を依存に宣言している。または、プレゼンテーション層が業務ロジック層を依存に宣言している |
| 事例が無い | `tool/business_logic/tests/` が無い |
| Python が残っている | `scripts/` が在る ── 道具は `tool/` が持つ |
| rs/ が残っている | 道具のソースを `tool/` へ移していない |
| parts/ か declare/ が残っている | crate のフォルダを層の正式名（`business_logic/` ・ `service/`）へ改めていない |
| bin/ を git の追跡から外していない | `.gitignore` に `bin/` が無い |
| 外部の道具の名前を直書きしている | `Command::new("…")` に名前が書いてある。`tool.json` に宣言し、注入する |
| references の実装が無い（版2） | `tool/business_logic/src/refs.rs` か `tool/service/src/refs.rs` が無い |
| references の実装が雛形と違う（版2） | `tool/business_logic/src/refs.rs` が `refs.rs.tmpl` と一致しない。雛形から複製し直す |

依存が `Cargo.toml` どおりに守られているかは、コンパイラが判定する。**見つけるが、直さない。**
