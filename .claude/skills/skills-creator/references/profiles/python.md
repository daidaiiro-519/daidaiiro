# 言語の組：Python

## 目的

道具の契約（`references/tool-contract.md`）を、**Python で満たすときの実装の形**を規定する。
契約が規定するのは呼ぶ側から観察できるものだけで、この文書はそれを満たす置き場所 ・ 雛形 ・
組み立て ・ ソースの検査を持つ（ACDR 0062）。

**この組は契約の版1 を満たし、作業型だけを持つ。** 助言型（版2）は Rust の組で生む。
作った Skill の配布（`dist`）には対応しない ── 利用者の環境に Python と uv を必要とする。

---

## 置き場所

```
tool/                道具のソース
  pyproject.toml     依存（MCP の公式 SDK の v2）。組み立ての仕組みを置かない ── uv は依存だけを揃える
  cli.py             プレゼンテーション層 ── シェルから呼ぶ唯一の経路
  mcp_server.py      プレゼンテーション層 ── MCP の面
  <パッケージ名>/    Skill の名前の - を _ にしたもの
    contract.py      契約の実体（どの Skill も同じファイル）
    tools.py         道具の一覧（能力の正本）
    hello.py         見本の業務ロジック
  tests/             事例（unittest）
tool.json            CLI の起動のコマンドと、外部の道具
mcp.json             MCP の起動のコマンド（ホストの形式）
.gitignore           tool/.venv/ ・ __pycache__/ ・ tool/uv.lock を追跡しない
```

**実行ファイルを置かない。** 起動のコマンドは `uv run --project <tool のフォルダ> python <入口>` である。
経路は `${CLAUDE_PROJECT_DIR:-.}` から書く ── 登録の中で、作業場所に依存しない。
**Skill のフォルダは、`contract.py` の位置から上へたどり、`SKILL.md` の在るフォルダである**（`Given.skill_root`）。

**中の構成は指定しない**（ACDR 0059）。雛形はパッケージを1つ置くだけで、層に分けない ──
分けるかどうか、どう分けるかは Skill を作る人が決める。

組み立ては、Skill のフォルダで次を実行する。

```
uv sync --quiet --project tool
```

---

## 契約の実装

| 契約の規定 | Python の組での実装 |
|---|---|
| 道具の一覧 | `tools.py` の `tools()`。MCP の面で、道具をその場で宣言するデコレータを使わない ── 低水準の `Server` の `on_list_tools` ・ `on_call_tool` を、同じ一覧から組む |
| 動詞なしの `--json` | CLI が `catalog()` の結果を返す（`contract.py`） |
| 一覧に無い旗 | CLI が終了コード 2 で断る（`cli.py` の `read_args`） |
| 外部の道具 | `given.external("名前")` で `tool.json` から読み、引数として渡す。**`subprocess.run(["…"])` に名前を直書きしない** |
| 標準出力 | stdio で配るあいだ、SDK が標準出力を標準エラーへ向け直す |
| 誤りの返し方 | `ok` が偽なら `CallToolResult` の `is_error` を立てる。無い道具と足りない引数は、`MCPError(INVALID_PARAMS, …)` で返す（Rust の組と同じ分け方） |

---

## 雛形

`references/profiles/python/` に置き、組の定義 `references/profiles/python.profile.json` が、何をどこへ置くかを持つ。
`skills-creator scaffold <名前> --type work --language python` が、共通の一式と作業型の一式を合わせて置く。

| 置き場所 | 中身 | 置く先 |
|---|---|---|
| `common/` | 全ての型に置く共通の一式 ── `pyproject.toml` ・ `contract.py` ・ `cli.py` ・ `mcp_server.py` ・ `tool.json` ・ `mcp.json` ・ `.gitignore` | `tool/` と Skill のフォルダ |
| `types/work/` | 作業型の一式 ── 道具の一覧 `tools.py`、見本の道具 `hello`、その事例 | `tool/<パッケージ名>/` ・ `tool/tests/` |

**生んだ直後に動く。** 見本の道具 `hello` を書き換えて、この Skill の道具にする。

---

## 2段目の検査

`skills-creator check` は、`tool/pyproject.toml` が在る Skill にこの組の検査を適用する。

| 検出 | 何が起きているか |
|---|---|
| 外部の道具の名前を直書きしている | `subprocess.run(` ・ `subprocess.Popen(` ・ `os.system(` などの直後に、名前が文字列で書いてある。`tool.json` に宣言し、注入する |

**雛形の構成の検査（`--layout 1`）は保持しない。** 構成を指定しないので、照らす先が無い。
