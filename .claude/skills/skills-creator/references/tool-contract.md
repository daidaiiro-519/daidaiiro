# 道具の契約：tool-contract

## 目的

Skill が道具（スクリプト）を伴うとき、**呼ぶ側に形を推測させない**ために使用する。

道具ごとに入口の形が違うと、呼ぶ側は呼ぶたびに本文を読み直す。
**契約は、呼び方を1つに固定する。**

---

## 原則

**能力は1つ、呼び出し方は複数である。**

能力を宣言に1度だけ書き、CLI と MCP は**その宣言から組む**。
能力を2か所に書くと、片方だけが古くなる。

**呼び出し方が欠けても、能力は欠けない。**
MCP の実装が無い環境では、MCP の面は立たず、CLI だけが動く。

---

## 規定するもの

| 規定 | 中身 |
|---|---|
| **生成物の型** | **成果物を出す Skill は、HTML の形を型のファイルへ置く**（下の表）── 形をコードの中の文字列に散らすと、成果物ごとに違う形が出る |
| **置き場所** | **役割ごとに階層を分ける**（下の表）── 中身を開かずに、どれが入口かを判定できる形にする |
| 宣言 | `scripts/tools.py` の `TOOLS`。名前 ・ 引数 ・ 説明 ・ 実体 ・ 人が読む形 |
| 入口 | `scripts/cli.py` が唯一。`python3 scripts/cli.py <動詞> [対象…] [--json]` |
| MCP の面 | `scripts/mcp_server.py`。**同じ宣言から組む** |
| 登録 | `mcp.json` の断片。**ホストごとの差は、ここだけが吸収する** |
| 戻り値 | `{"ok": 真偽, "findings": [検出], "data": {本体}}` |
| 終了コード | `0` 正常 ／ `1` 検出あり ／ `2` 誤用 |
| 印字 | **道具は印字しない** ── 印字と終了コードは入口が持つ |
| **標準出力** | **MCP の面では、道具が標準出力へ1バイトも書かない** ── 原典が禁じている（下の表）。書くなら標準エラーである |
| **面のファイル名** | **`mcp.py` と名付けない** ── 実行すると自分の在る場所が探索の先頭に入り、`import mcp` がこのファイル自身を指す。実装が在っても「無い」と報告する |

### 標準入出力の規約 ── 原典

`modelcontextprotocol.io/specification/2025-06-18/basic/transports`（2026-09-24 取得 ・ 297行）から引用する。

| 行 | 原文 |
|---|---|
| 27 | `The server reads JSON-RPC messages from its standard input (stdin) and sends messages to its standard output (stdout).` |
| 31 | `The server **MAY** write UTF-8 strings to its standard error (stderr) for logging purposes.` |
| 33 | `The server **MUST NOT** write anything to its stdout that is not a valid MCP message.` |

**だから、道具の中で `print()` を使うと MCP の面が壊れる** ── 子プロセスの標準出力も同じである（`capture_output` か `stdout=` で受ける）。

### 誤りの返し方 ── 原典

原典は2つを分けている（`specification/2025-06-18/server/tools:389`、2026-09-24 取得）。

| 種類 | 原文 | この契約での対応 |
|---|---|---|
| Protocol Errors | `Standard JSON-RPC errors for issues like: Unknown tools, Invalid arguments` | 引数の不足 ・ 知らない動詞。SDK が返す |
| Tool Execution Errors | `Reported in tool results with isError: true` | **`ok` が偽のとき**（読めない ・ 誤用） |

**`findings` は誤りではない。** 検出が在っても `isError` は偽のままにする ── 違反の検出は、道具が正しく動作した結果である。

**予期した失敗は `ToolError` で投げる。** SDK の原典が述べている ── その型なら `is_error=True` とこちらの文言が届き、それ以外の例外は `Error executing tool <名前>` しか届かない。

> the call returns `is_error=True` with your message in `content` for the model to read … Any other exception bar `MCPError` (a protocol error) is treated as a crash: the model sees only `Error executing tool <name>`

出典 ── python-sdk v2.2.0 の `src/mcp/server/mcpserver/exceptions.py:47`（2026-09-24 取得 ・ 77行 ・ sha256 `14564c0dedfe79db…`）。導入した版と原文が一致することを確認した。

**型は緩く受ける。** 呼ぶ側は JSON の値を渡すので、数を文字列で包むことを強制しない（実測: `timeout` に 30 を渡すと、検証が不合格になっていた）。まとめて受ける引数は配列も受ける。

### 子プロセスを起こすときの規律

道具が外のコマンドを実行するなら、3つを必ず指定する。**どれも実測で欠陥が出た項目である**（2026-09-24、9つの Skill で6か所）。

| 指定 | 欠けると何が起きるか |
|---|---|
| `stdin=subprocess.DEVNULL` | 入力を待つ道具が、制限時間まで停止する（実測 5秒の設定で5.1秒） |
| `timeout=` | 停止した子プロセスを、永久に待つ |
| `capture_output=True` か `stdout=` | **子プロセスの出力が、こちらの標準出力へ流れる** ── MCP の面では JSON-RPC の流れへ混入する |

**出力をまとめて受け取らない。** ファイルへ流し、上限までしか読む ── 受け取ると、道具が出した量がそのままこちらの記憶に載る（実測: 100MB の出力で最大常駐が 306MB、ファイルへ流すと 24KB）。

Python の SDK は 2.x で `FastMCP` が `MCPServer` へ改称された。原典（`python-sdk` の README、2026-09-24 取得 ・ 134行）の55行が `from mcp.server import MCPServer` を示す。**1.x も動く形にする** ── `ImportError` のときは `mcp.server.fastmcp.FastMCP` を採用する。

## 規定しないもの

| 規定しない | 理由 |
|---|---|
| 道具の中身 | 契約は呼び方を固定する。何をするかは Skill が決める |
| 検出の意味 | `findings` は検出であって、誤りではない。意味づけは呼ぶ側が持つ |
| MCP の実装 | 面の組み方はホストの側の都合である |

---

## 置き場所

```
scripts/
  cli.py        唯一の入口
  mcp_server.py MCP の面
  tools.py      能力の宣言（正本）
  contract.py   契約の実体
  lib/          部品 ── 読み込まれるもの。__init__.py を持つ
  tests/        検証
```

| 階層 | 置くもの | 入口を保持するか |
|---|---|---|
| `scripts/` 直下 | 上の4つ**だけ** | `cli.py` のみ |
| `scripts/lib/` | 組み立て ・ 描画 ・ トークン ・ 検査など | **保持しない** |
| `scripts/tests/` | `test_*.py` | ── |

**平らに並べない。** 入口 ・ 部品 ・ 検証が同じ階層に同居すると、
**どれを起動してよいかが、中身を開くまで判定できない。**

**部品は、置き場所を階層の数で数えない。** `parent.parent` のような数え方は、
部品を動かすたびに狂う ── 上へ辿って目印（`references/`）を探す。

---

## 生成物を持つ Skill は、3つで組む

**入力の契約 ・ 出力の型 ・ 組み立ての3つに分ける。** どれか1つに形が混ざると、
同じ Skill が出す成果物どうしで形が違ってくる（実際に、節の1つだけ別の入れ物で組まれていた）。

| | 何を持つか | 置き場所 |
|---|---|---|
| **入力の契約** | 何を書けるか。JSON Schema | `references/<名前>.schema.json` |
| **出力の型** | どう並ぶか。差し込む場所（`{{名前}}`）を持つ | `references/<名前>.template.html` |
| **組み立て** | 値を差し込むだけ。**構造を作る文字列を保持しない** | `scripts/lib/` |

```
JSON Schema  →  実体の JSON（値を埋める）  →  型（HTML）へ差し込む  →  生成物
```

**差し込む場所の過不足は、その場で例外にする** ── 埋め忘れも、余分な値も、出てから気づく形にしない。
`scripts/lib/template.py` が読み、`part("<部品>", <差し込む場所>=…)` で組む。

**検証で、組み立て側に構造を作る文字列が残っていないことを検査する** ── 規定だけでは戻る。

---

## 道具と部品の区別

| | 入口を持つか | どう呼ぶか | どこに置くか |
|---|---|---|---|
| **道具** | 持つ（宣言に載る） | `cli.py <動詞>` ／ MCP | `scripts/tools.py` が宣言する |
| **部品** | **持たない** | 他のコードから `import` する | `scripts/lib/` |

**部品に入口を付けない。** 付けると、同じ能力に呼び方が2つできる。

---

## 検査

```
python3 scripts/cli.py check <Skill のフォルダ>
```

見るのは2つ ── **入口が1つであること**と、**置き場所が役割と一致すること**である。

| 検出 | 何が起きているか |
|---|---|
| 入口の部品が無い | 契約の一式が完備していない |
| 部品が入口の側に在る | `scripts/` 直下に部品が置かれている ── `lib/` へ移す |
| 検証が入口の側に在る | `tests/` へ移す |
| 部品が入口を保持している | `lib/` の中に `__main__` か `sys.argv` が在る ── 呼び方が2つになる |
| 検証が `tests/` の外に在る | 検証の置き場所が散る |

**見つけるが、直さない。**
