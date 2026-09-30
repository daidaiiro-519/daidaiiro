# 言語の組：Go

## 目的

道具の契約（`references/tool-contract.md`）を、**Go で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を保持する（ACDR 0065）。

**この組は契約の版1 を満たし、作業型だけを保持する。** 助言型（版2）は Rust の組で生む。
作った Skill の配布（`dist`）には対応しない。組み立てには Go（1.25 以上）を必要とする。

---

## 置き場所

```
tool/                道具のソース
  go.mod             モジュール（名前は Skill の名前の - を _ にしたもの）と、MCP の公式 SDK（v1.8.0）
  contract/          契約の実体（どの Skill も同じファイル）
  tools/             道具の一覧（能力の正本）と、見本の業務ロジック・事例
  cmd/cli/           プレゼンテーション層 ── シェルから呼ぶ唯一の経路
  cmd/mcp/           プレゼンテーション層 ── MCP の面
bin/                 組み立てた実行ファイル（git で追跡しない）
  <名前>             CLI（Windows は <名前>.exe）
  <名前>-mcp         MCP サーバー（Windows は <名前>-mcp.exe）
tool.json            CLI の起動のコマンドと、外部の道具
mcp.json             MCP の起動のコマンド（ホストの形式）
.gitignore           bin/ を追跡しない
```

**実行ファイルの置き場所は Rust の組と同じ `bin/` である。** 起動に処理系を必要としない。
**Skill のフォルダは、実行ファイルの位置から上へたどり、`SKILL.md` の在るフォルダである**（`Given.SkillRoot`）。

**中の構成は指定しない**（ACDR 0059）。雛形は CLI と MCP が同じ一覧を参照するためにパッケージを分けるだけで、層に分けない。

組み立ては、Skill のフォルダで次を実行する。`go mod tidy` が `go.sum` を作る ── `go.sum` は git で追跡する。

```
go mod tidy -C tool
go build -C tool -o ../bin/<名前> ./cmd/cli
go build -C tool -o ../bin/<名前>-mcp ./cmd/mcp
```

---

## 契約の実装

| 契約の規定 | Go の組での実装 |
|---|---|
| 道具の一覧 | `tools` の `All()`。MCP の面で、引数の型から入力の形を導く汎用の `mcp.AddTool` を使わない ── `Server.AddTool` に、同じ一覧から組んだ入力の形と処理を渡す |
| 動詞なしの `--json` | CLI が `contract.Catalog()` の結果を返す |
| 一覧に無い旗 | CLI が終了コード 2 で断る（`cmd/cli` の `readArgs`） |
| 外部の道具 | `given.External("名前")` で `tool.json` から読み、引数として渡す。**`exec.Command("…")` に名前を直書きしない** |
| 標準出力 | MCP の面は標準出力へ書かない。標準出力は JSON-RPC の通信路である |
| 誤りの返し方 | `ok` が偽なら `IsError` を立てる。足りない引数は、`jsonrpc.Error{Code: jsonrpc.CodeInvalidParams}` で返す（Rust の組と同じ分け方） |

---

## 雛形

`references/profiles/go/` に置き、組の定義 `references/profiles/go.profile.json` が、何をどこへ置くかを保持する。
`skills-creator scaffold <名前> --type work --language go` が、共通の一式と作業型の一式を合わせて置く。

| 置き場所 | 中身 | 置く先 |
|---|---|---|
| `common/` | 全ての型に置く共通の一式 ── `go.mod` ・ `contract.go` ・ CLI と MCP の `main.go` ・ `tool.json` ・ `mcp.json` ・ `.gitignore` | `tool/` と Skill のフォルダ |
| `types/work/` | 作業型の一式 ── 道具の一覧 `tools.go`、見本の道具 `hello`、その事例 | `tool/tools/` |

**生んだ直後に組み立てられる。** 見本の道具 `hello` を書き換えて、この Skill の道具にする。

---

## 2段目の検査

`skills-creator check` は、`tool/go.mod` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 外部の道具の名前を直書きしている | `exec.Command(` の直後に、名前が文字列で書いてある。`tool.json` に宣言し、注入する |

**雛形の構成の検査（`--layout 1`）は保持しない。** 構成を指定しないので、照らす先が無い。
