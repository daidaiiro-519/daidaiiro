# 言語の組：TypeScript

## 目的

道具の契約（`references/tool-contract.md`）を、**TypeScript で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を保持する（ACDR 0063）。

**この組は契約の版2 を満たし、作業型と助言型を保持する**（ACDR 0069）。references の実装は、Rust の組の `refs.rs` と同じ出力を返す ── 同じ references が、言語によって違って見えてはならない。
作った Skill の配布（`dist`）には対応しない ── 利用者の環境に Node.js（22.18 以上）を必要とする。

---

## 置き場所

```
tool/              道具のソース
  package.json     依存（MCP の公式 SDK の v2 ・ @modelcontextprotocol/server）と、型の検査の道具
  tsconfig.json    型の検査と編集の補助だけに使う。出力を置かない
  src/
    contract.ts    契約の実体（どの Skill も同じファイル）
    refs.ts        references の実装（どの Skill も同じファイル）── 取り出す ・ 検査する ・ 描画する ・ 取り込む
    refs_tools.ts  references の4つの道具（どの Skill も同じファイル）
    cli.ts         プレゼンテーション層 ── シェルから呼ぶ唯一の経路
    mcp.ts         プレゼンテーション層 ── MCP の面
    tools.ts       道具の一覧（能力の正本）
    hello.ts       見本の業務ロジック
  tests/           事例（node:test）
tool.json          CLI の起動のコマンドと、外部の道具
mcp.json           MCP の起動のコマンド（ホストの形式）
.gitignore         tool/node_modules/ を追跡しない
```

**実行ファイルも、変換した JavaScript も置かない。** Node.js は型の注記を外して `.ts` を直接実行する。
起動のコマンドは `node <tool/src/cli.ts の経路>` である。経路は `${CLAUDE_PROJECT_DIR:-.}` から書く。
**型の注記を外すだけで実行できる構文に限る**（`tsconfig.json` の `erasableSyntaxOnly`）── enum と名前空間は使わない。
**Skill のフォルダは、`contract.ts` の位置から上へたどり、`SKILL.md` の在るフォルダである**（`Given.skillRoot`）。

**中の構成は指定しない**（ACDR 0059）。雛形は `src/` にファイルを並べるだけで、層に分けない。

組み立てと型の検査は、Skill のフォルダで次を実行する。

```
npm install --silent --prefix tool
npm run --silent --prefix tool check
```

---

## 契約の実装

| 契約の規定 | TypeScript の組での実装 |
|---|---|
| 道具の一覧 | `tools.ts` の `tools()`。MCP の面で、道具をその場で宣言する `registerTool` を使わない ── 低水準の `Server` の `tools/list` ・ `tools/call` を、同じ一覧から組む |
| 動詞なしの `--json` | CLI が `catalog()` の結果を返す（`contract.ts`） |
| 一覧に無い旗 | CLI が終了コード 2 で断る（`cli.ts` の `readArgs`） |
| 外部の道具 | `given.external("名前")` で `tool.json` から読み、引数として渡す。**`spawnSync("…")` に名前を直書きしない** |
| 標準出力 | MCP の面は標準出力へ書かない。標準出力は JSON-RPC の通信路である |
| 誤りの返し方 | `ok` が偽なら `isError` を立てる。無い道具と足りない引数は、`ProtocolError(ProtocolErrorCode.InvalidParams, …)` で返す（Rust の組と同じ分け方） |
| references（版2） | `refs.ts` が取り出す ・ 検査する ・ 描画する ・ 取り込むを持ち、`refs_tools.ts` が4つの道具（get ・ validate ・ view ・ import）として一覧に足す。JSON Schema の検査は `ajv`（Draft 2020-12）が行う |

---

## 雛形

`references/profiles/typescript/` に置き、組の定義 `references/profiles/typescript.profile.json` が、何をどこへ置くかを保持する。
`skills-creator scaffold <名前> --type work --language typescript` が、共通の一式と作業型の一式を合わせて置く。

| 置き場所 | 中身 | 置く先 |
|---|---|---|
| `common/` | 全ての型に置く共通の一式 ── `package.json` ・ `tsconfig.json` ・ `contract.ts` ・ `cli.ts` ・ `mcp.ts` ・ `tool.json` ・ `mcp.json` ・ `.gitignore` ・ `refs.ts` ・ `refs_tools.ts` ・ `document.schema.json` | `tool/` と Skill のフォルダ |
| `types/work/` | 作業型の一式 ── 道具の一覧 `tools.ts`、見本の道具 `hello`、その事例 | `tool/src/` ・ `tool/tests/` |
| `types/advisor/` | 助言型の一式 ── 道具の一覧 `tools.ts`（references の4つだけ）、事例5件 `references.test.ts`、回答の例 | `tool/src/` ・ `tool/tests/` |

**生んだ直後に動く。** 見本の道具 `hello` を書き換えて、この Skill の道具にする。助言型は、`references/types/advisor/procedure.md` の手順で判断基準を作る。

---

## 2段目の検査

`skills-creator check` は、`tool/package.json` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 外部の道具の名前を直書きしている | `spawn(` ・ `spawnSync(` ・ `execSync(` ・ `execFile(` ・ `execFileSync(` の直後に、名前が文字列で書いてある。`tool.json` に宣言し、注入する |

**雛形の構成の検査（`--layout 1`）は保持しない。** 構成を指定しないので、照らす先が無い。
