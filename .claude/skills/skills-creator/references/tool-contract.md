# 道具の契約：tool-contract

## 目的

Skill が道具（実行ファイル）を伴うとき、**呼ぶ側に形を推測させない**ために使用する。

道具ごとに入口の形が違うと、呼ぶ側は呼ぶたびに本文を読み直す。
**契約は、呼び方を1つに固定する。**

---

## 原則

**能力は1つ、呼び出し方は複数である。**

能力を宣言に1度だけ書き、CLI と MCP は**その宣言から組む**。
能力を2か所に書くと、片方だけが古くなる。

**呼び出し方が欠けても、能力は欠けない。**
MCP の実装が無い環境では、MCP サーバーは立たず、CLI だけが動く。

---

## 規定するもの

| 規定 | 中身 |
|---|---|
| **生成物の型** | **成果物を出す Skill は、HTML の形を型のファイルへ置く**（下の表）── 形をコードの中の文字列に散らすと、成果物ごとに違う形が出る |
| **置き場所** | **役割ごとに階層を分ける**（下の表）── 中身を開かずに、どれが入口かを判定できる形にする |
| 宣言 | `rs/declare/` の `tools()`。名前 ・ 引数 ・ 説明 ・ 実体 ・ 人が読む形 |
| 入口 | `rs/cli/` が唯一。`<Skill の名前> <動詞> [対象…] [--json]` |
| MCP サーバー | `rs/mcp/`。**同じ宣言から組む** ── `#[tool]` マクロで道具をその場で宣言しない |
| 登録 | `mcp.json` の断片。**ホストごとの差は、ここだけが吸収する** |
| 戻り値 | `{"ok": 真偽, "findings": [検出], "data": {本体}}` |
| 終了コード | `0` 正常 ／ `1` 検出あり ／ `2` 誤用 |
| 印字 | **道具は印字しない** ── 印字と終了コードは入口が持つ |
| **標準出力** | **MCP サーバーでは、道具が標準出力へ1バイトも書かない** ── 原典が禁じている（下の表）。書くなら標準エラーである |

### 標準入出力の規約 ── 原典

`modelcontextprotocol.io/specification/2025-06-18/basic/transports`（2026-09-24 取得 ・ 297行）から引用する。

| 行 | 原文 |
|---|---|
| 27 | `The server reads JSON-RPC messages from its standard input (stdin) and sends messages to its standard output (stdout).` |
| 31 | `The server **MAY** write UTF-8 strings to its standard error (stderr) for logging purposes.` |
| 33 | `The server **MUST NOT** write anything to its stdout that is not a valid MCP message.` |

**だから、道具の中で `println!` を使うと MCP サーバーが壊れる** ── 子プロセスの標準出力も同じである（`Stdio::piped()` かファイルで受ける）。

### 誤りの返し方 ── 原典

原典は2つを分けている（`specification/2025-06-18/server/tools:389`、2026-09-24 取得）。

| 種類 | 原文 | この契約での対応 |
|---|---|---|
| Protocol Errors | `Standard JSON-RPC errors for issues like: Unknown tools, Invalid arguments` | 引数の不足 ・ 知らない動詞。SDK が返す |
| Tool Execution Errors | `Reported in tool results with isError: true` | **`ok` が偽のとき**（読めない ・ 誤用） |

**`findings` は誤りではない。** 検出が在っても `isError` は偽のままにする ── 違反の検出は、道具が正しく動作した結果である。

**予期した失敗は、道具の結果として返す。** `ok` が偽なら `CallToolResult` の `is_error` を立て、こちらの文言を `content` に載せる ── 異常終了（panic）させると、呼ぶ側には理由が届かない。雛形の `mcp.main.rs.tmpl` がこの形を持つ。

**型は緩く受ける。** 呼ぶ側は JSON の値を渡すので、数を文字列で包むことを強制しない（実測: `timeout` に 30 を渡すと、検証が不合格になっていた）。まとめて受ける引数は配列も受ける。

### 戻り値は、構造としても乗せる

原典が両方を求めている（`server/tools:296・298`、2026-09-24 取得）。

> **Structured** content is returned as a JSON object in the `structuredContent` field of a result.

> For backwards compatibility, a tool that returns structured content SHOULD also return the serialized JSON in a TextContent block.

`{ok, findings, data}` は JSON なので、**そのまま `structuredContent` に乗る**。雛形の MCP の面は、同じ値を `structured_content` と、文字列にした `content` の両方へ載せる。

### 経路 ・ 大きい出力 ・ 保持

道具が外のものに触れるなら、3つを遵守する。**どれも公式の参照実装が採っている形である**（2026-09-24 に原文を取得）。

| 規定 | 原典 |
|---|---|
| **経路は、解決してから根の中かを確認する** ── `../` や絶対のパスで外へ出るのを止める | Git の `server.py:136` ── `Defense in depth: validate each path resolves within the repository working tree to prevent path traversal` |
| **大きい出力は上限で切り、続きの取り方を出力に書く** ── 読む側は MCP しか持たないことがある | Fetch の `server.py:254` ── `Content truncated. Call the fetch tool with a start_index of {next_start} to get more content.` |
| **範囲は明示し、空なら動作しない** | Filesystem の `README.md:31` ── `the server will throw an error during initialization` |

**保持したものは、識別子で照合してから返す。** 合わなければ明示して断る ── 別の実行の中身を返すと、読む側はそれを今回の結果として受け取る。**古いものを黙って返さないのは、「検査していないのに合格と出る」と同じ型の欠陥である。**

### 子プロセスを起こすときの規律

道具が外のコマンドを実行するなら、3つを必ず指定する。**どれも実測で欠陥が出た項目である**（2026-09-24、9つの Skill で6か所）。

| 指定 | 欠けると何が起きるか |
|---|---|
| `.stdin(Stdio::null())` | 入力を待つ道具が、制限時間まで停止する（実測 5秒の設定で5.1秒） |
| 制限時間（`try_wait` で待ち、過ぎたら `kill`） | 停止した子プロセスを、永久に待つ |
| `.stdout(…)` ・ `.stderr(…)` の指定 | **子プロセスの出力が、こちらの標準出力へ流れる** ── MCP サーバーでは JSON-RPC の流れへ混入する |

**出力をまとめて受け取らない。** ファイルへ流し、上限までしか読む ── 受け取ると、道具が出した量がそのままこちらの記憶に載る（実測: 100MB の出力で最大常駐が 306MB、ファイルへ流すと 24KB）。

## 規定しないもの

| 規定しない | 理由 |
|---|---|
| 道具の中身 | 契約は呼び方を固定する。何をするかは Skill が決める |
| 検出の意味 | `findings` は検出であって、誤りではない。意味づけは呼ぶ側が持つ |
| MCP の実装 | サーバーの組み方はホストの側の都合である |

---

## 置き場所

```
rs/
  Cargo.toml    workspace ── 4つの crate を並べる
  parts/        部品 ── 実体。読み込まれるもの
    tests/      事例
  declare/      能力の宣言（正本）と契約の実体（contract.rs）
  cli/          唯一の入口
  mcp/          MCP の面
mcp.json        登録
```

| crate | 置くもの | 参照してよい先 | 入口を保持するか |
|---|---|---|---|
| `parts` | 組み立て ・ 描画 ・ トークン ・ 検査など | **無し** | **保持しない** |
| `declare` | 名前 ・ 引数 ・ 説明 ・ 実体の対応 | `parts` | 保持しない |
| `cli` ・ `mcp` | 入口 | `declare` | `cli` は唯一の入口 |

**層の境界を crate の境界に置く。** 1つの crate の中の module では、内側が外側を参照してもコンパイラが通す
── crate に分けて初めて、**宣言に無い依存が解決しなくなる**。許可辺は各 `Cargo.toml` が宣言する。

**部品は、置き場所を階層の数で数えない。** 実行ファイルからの相対で数えると、置き場所を動かすたびに狂う
── Skill の置き場所は `--skill_root` で受け取る。

---

## 生成物を持つ Skill は、3つで組む

**入力の契約 ・ 出力の型 ・ 組み立ての3つに分ける。** どれか1つに形が混ざると、
同じ Skill が出す成果物どうしで形が違ってくる（実際に、節の1つだけ別の入れ物で組まれていた）。

| | 何を持つか | 置き場所 |
|---|---|---|
| **入力の契約** | 何を書けるか。JSON Schema | `references/<名前>.schema.json` |
| **出力の型** | どう並ぶか。差し込む場所（`{{名前}}`）を持つ | `references/<名前>.template.html` |
| **組み立て** | 値を差し込むだけ。**構造を作る文字列を保持しない** | `rs/parts/` |

```
JSON Schema  →  実体の JSON（値を埋める）  →  型（HTML）へ差し込む  →  生成物
```

**差し込む場所の過不足は、その場で例外にする** ── 埋め忘れも、余分な値も、出てから気づく形にしない。
型を読んで部品を組むのは `rs/parts/` の組み立てである。

**検証で、組み立て側に構造を作る文字列が残っていないことを検査する** ── 規定だけでは戻る。

---

## 道具と部品の区別

| | 入口を持つか | どう呼ぶか | どこに置くか |
|---|---|---|---|
| **道具** | 持つ（宣言に載る） | `<Skill の名前> <動詞>` ／ MCP | `rs/declare/` が宣言する |
| **部品** | **持たない** | 他の crate から `use` する | `rs/parts/` |

**部品に入口を付けない。** 付けると、同じ能力に呼び方が2つできる。

---

## 検査

```
skills-creator check <Skill のフォルダ>
```

見るのは2つ ── **層が crate に分かれていること**と、**許可辺が各 `Cargo.toml` の宣言どおりであること**である。
依存が宣言どおりに守られているかは、コンパイラが判定する。

| 検出 | 何が起きているか |
|---|---|
| 層が crate に分かれていない | `rs/Cargo.toml` が無い |
| 層の crate が無い ／ 入口の crate が無い | 契約の一式が完備していない |
| どの層か決まらない | crate の名前が層の名前で終わっていない |
| 許可していない辺を宣言している | 内側の crate が外側を依存に宣言している |
| 事例が無い | `rs/parts/tests/` が無い |
| Python が残っている | `scripts/` が在る ── 道具は `rs/` が持つ |

**見つけるが、直さない。**
