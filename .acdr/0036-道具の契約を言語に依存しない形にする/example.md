# 完成イメージ ── acdr の場合

## 契約 ・ 言語の組 ・ 配布の分け方

| 置く先 | 中身 |
|---|---|
| 契約（必須 ・ 言語に依存しない） | CLI と MCP の2つの入口と、その起動のコマンド ・ CLI の規約（動詞 ・ `--json` ・ `{ok, findings, data}` ・ 終了コード 0/1/2）・ 1つの宣言から CLI と MCP を組む ・ Skill のフォルダは、渡された値か入口自身の位置から求める ・ 外部の道具は `tool.json` に宣言して注入する ・ 層と許可辺 |
| 言語の組（Rust が1つ目） | `tool/` の中の構成 ・ 入口の置き場所（`bin/`）・ 雛形 ・ 組み立てのコマンド ・ ソースの検査 |
| 配布（任意） | `dist` を実行した人だけが使う。入口が `bin/` に在ることを前提にし、導入スクリプトと CI の枠は全言語で共有する |

## Skill のフォルダ

tool.json を新しく置き、CLI の起動のコマンドと外部の道具を宣言する。

```
acdr/
  SKILL.md
  mcp.json     MCP の起動のコマンド（変わらない）
  tool.json    CLI の起動のコマンドと、外部の道具（新しく置く）
  bin/         入口（Rust の組が決める置き場所）
  tool/        Rust のソース（EXTERNAL の定数は無くなる）
```

## tool.json

```json
{
  "cli": {"command": "${CLAUDE_PROJECT_DIR:-.}/.claude/skills/acdr/bin/acdr", "args": []},
  "external": [
    {"name": "git",
     "command": "git",
     "reason": "git のリポジトリの版から、変更前の中身を取得する。git の差分を記録することが目的である"}
  ]
}
```

利用者が別の git を使うときは、`"command"` の1行だけを書き換える。

## 部品は、外部の道具を引数で受け取る

部品は外部の道具の名前を直書きせず、宣言の層から渡されたコマンドを起動する。

```rust
pub fn before_of(git: &Path, root: &Path, rev: &str, rel: &str) -> Option<String> {
    let done = Command::new(git)   // 渡されたコマンドを起動する
    …
}

// 宣言の層（tool/declare）が tool.json から読んで渡す
let git = or_misuse!(given.external("git"));
panes::before_of(&git, …)
```

## 動詞なしの `--json` は、宣言を返す

```
$ acdr --json
{"ok": true, "findings": [], "data": {"tools": [
  {"name": "new",      "args": ["record", "title", "skill_root"]},
  {"name": "validate", "args": ["record", "skill_root"]},
  {"name": "render",   "args": ["record", "check", "force", "skill_root"]},
  {"name": "tokens",   "args": ["check", "skill_root"]}
]}}
```

## check は2段で検査する

check は、入口を起動する1段目と、言語の組が持つ2段目の2段で検査する。

```
$ skills-creator check .claude/skills/acdr
1段目（振る舞い）
  OK  tool.json と mcp.json が在り、経路が絶対パスでない
  OK  宣言を JSON で返す（道具 4件）
  OK  宣言に無い旗を、終了コード 2 で断る
  OK  MCP の tools/list が CLI の宣言と一致する（4件）
  OK  別の作業場所から起動できる
2段目（ソース ── Rust の組）
  OK  層が crate に分かれ、許可辺が宣言どおりである
  OK  部品が外部の道具の名前を直書きしていない
```

言語の組が無い Skill（たとえば Python で書いた Skill）では、2段目は「実行しない」と出し、合格とは扱わない。
