# 言語の組：C#

## 目的

道具の契約（`references/tool-contract.md`）を、**C# で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を保持する（ACDR 0064）。

**この組は契約の版2 を満たし、作業型と助言型を保持する**（ACDR 0069）。references の実装は、Rust の組の `refs.rs` と同じ出力を返す ── 同じ references が、言語によって違って見えてはならない。
作った Skill の配布（`dist`）には対応しない ── 利用者の環境に .NET SDK（10）を必要とする。

---

## 置き場所

```
tool/                      道具のソース
  Directory.Build.props    全てのプロジェクトに共通の設定（net10.0 ・ InvariantGlobalization ほか）
  Skill/                   契約の実体と道具の一覧（ライブラリ）
    Contract.cs            契約の実体（どの Skill も同じファイル）
    Refs.cs                references の実装（どの Skill も同じファイル）
    RefsTools.cs           references の4つの道具（どの Skill も同じファイル）
    Tools.cs               道具の一覧（能力の正本）
    Hello.cs               見本の業務ロジック
  Cli/                     プレゼンテーション層 ── シェルから呼ぶ唯一の経路
  Mcp/                     プレゼンテーション層 ── MCP の面（ModelContextProtocol.Core 2.x）
  Tests/                   事例（xUnit）
  bin/cli/ ・ bin/mcp/     組み立てた dll（git で追跡しない）
tool.json                  CLI の起動のコマンドと、外部の道具
mcp.json                   MCP の起動のコマンド（ホストの形式）
.gitignore                 bin/ と obj/ を追跡しない
```

**起動のコマンドは `dotnet <tool/bin/cli/cli.dll の経路>` である。** dll は OS に依存しないので、
Windows でも同じ登録で起動する。経路は `${CLAUDE_PROJECT_DIR:-.}` から書く。
**Skill のフォルダは、実行中の dll の位置から上へたどり、`SKILL.md` の在るフォルダである**（`Given.SkillRoot`）。

**文化に依存する処理を使わない**（`InvariantGlobalization`）── ICU の無い環境でも起動する。

**中の構成は指定しない**（ACDR 0059）。雛形は CLI と MCP が同じ一覧を参照するために `Skill/` を分けるだけで、層に分けない。

組み立ては、Skill のフォルダで次を実行する。

```
dotnet build tool/Cli -c Release -o tool/bin/cli
dotnet build tool/Mcp -c Release -o tool/bin/mcp
```

---

## 契約の実装

| 契約の規定 | C# の組での実装 |
|---|---|
| 道具の一覧 | `Tools.cs` の `Tools.All()`。MCP の面で、道具をその場で宣言する属性（`McpServerTool`）を使わない ── `McpServer.Create` と `McpServerHandlers` の `ListToolsHandler` ・ `CallToolHandler` を、同じ一覧から組む |
| 動詞なしの `--json` | CLI が `Contract.Catalog()` の結果を返す（`Contract.cs`） |
| 一覧に無い旗 | CLI が終了コード 2 で断る（`Cli/Program.cs` の `ReadArgs`） |
| 外部の道具 | `given.External("名前")` で `tool.json` から読み、引数として渡す。**`Process.Start("…")` に名前を直書きしない** |
| 標準出力 | MCP の面は標準出力へ書かない。標準出力は JSON-RPC の通信路である |
| 誤りの返し方 | `ok` が偽なら `IsError` を立てる。無い道具と足りない引数は、`McpProtocolException(…, McpErrorCode.InvalidParams)` で返す（Rust の組と同じ分け方） |
| references（版2） | `Refs.cs` が取り出す ・ 検査する ・ 描画する ・ 取り込むを持ち、`RefsTools.cs` が4つの道具（get ・ validate ・ view ・ import）として一覧に足す。JSON Schema の検査は `JsonSchema.Net`（Draft 2020-12。未知の欄 `x-view` を許す方言）が行う |

---

## 雛形

`references/profiles/csharp/` に置き、組の定義 `references/profiles/csharp.profile.json` が、何をどこへ置くかを保持する。
`skills-creator scaffold <名前> --type work --language csharp` が、共通の一式と作業型の一式を合わせて置く。

| 置き場所 | 中身 | 置く先 |
|---|---|---|
| `common/` | 全ての型に置く共通の一式 ── `Directory.Build.props` ・ 3つのプロジェクトの定義 ・ `Contract.cs` ・ CLI と MCP の `Program.cs` ・ `tool.json` ・ `mcp.json` ・ `.gitignore` ・ `Refs.cs` ・ `RefsTools.cs` ・ `document.schema.json` | `tool/` と Skill のフォルダ |
| `types/work/` | 作業型の一式 ── 道具の一覧 `Tools.cs`、見本の道具 `Hello`、その事例 | `tool/Skill/` ・ `tool/Tests/` |
| `types/advisor/` | 助言型の一式 ── 道具の一覧 `Tools.cs`（references の4つだけ）、事例5件 `ReferencesTests.cs`、回答の例 | `tool/Skill/` ・ `tool/Tests/` |

**生んだ直後に組み立てられる。** 見本の道具 `Hello` を書き換えて、この Skill の道具にする。助言型は、`references/types/advisor/procedure.md` の手順で判断基準を作る。

---

## 2段目の検査

`skills-creator check` は、`tool/Directory.Build.props` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 外部の道具の名前を直書きしている | `Process.Start(` ・ `new ProcessStartInfo(` ・ `FileName = ` の直後に、名前が文字列で書いてある。`tool.json` に宣言し、注入する |

**雛形の構成の検査（`--layout 1`）は保持しない。** 構成を指定しないので、照らす先が無い。
