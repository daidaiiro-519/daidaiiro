# architecture-lint

許していない依存の辺を、規則で検出して止める CLI。アーキテクチャに寄らない。

Cargo は「Cargo.toml に書いていない crate を使う」と「通常の依存の循環」を止める。ただし、その辺を Cargo.toml に書いてよいかは知らない。architecture-lint は、規則（component と allow）に無い辺を見つけ、終了コード 1 で CI を止める。

決めた経緯はブレストボード `.brainstorming/dependency-rules.json` と ACDR 0199 と 0200（名前）にある。

## 入れ方

```
cargo install --path architecture-lint
```

## 使い方

```
architecture-lint check --rules <規則.json> (--cargo <workspace の根> | --graph <ファイル|->) [--json]
architecture-lint graph --cargo <workspace の根>
architecture-lint --help
architecture-lint --version
```

| 操作 | すること |
|---|---|
| `check --cargo` | Rust の workspace を読み、規則で検査する |
| `check --graph` | ほかのプログラムが作ったグラフの JSON（`-` は標準入力）を、規則で検査する |
| `graph --cargo` | Rust の workspace のグラフの JSON だけを出す |
| `--json` | 結果を JSON で出す（形は `result.schema.json`） |

## 終了コード

| 値 | 意味 |
|---|---|
| 0 | 違反なし |
| 1 | 違反あり |
| 2 | 規則かグラフが読めない、`cargo metadata` が失敗した、使い方の誤り。理由は標準エラーに出る |

## 規則の書き方

形は `rules.schema.json` が決める。例は `examples/semantic-agent-schema.json`。

```json
{
  "version": 1,
  "components": [
    {"name": "domain", "nodes": ["myapp-domain-*"]},
    {"name": "app", "nodes": ["myapp-app"]},
    {"name": "tempfile", "nodes": ["tempfile"]}
  ],
  "allow": [
    {"from": "app", "to": ["domain"]},
    {"from": "domain", "to": ["tempfile"], "kinds": ["dev"], "description": "テストの一時ディレクトリだけ"}
  ]
}
```

| 欄 | 決まり |
|---|---|
| `components` | node の名前の集まり。`nodes` は完全一致か、末尾の `*` で前方一致。1つの node が2つ以上の component に当たると規則の誤り（終了コード 2） |
| `allow` | 許す辺。`from` の component から `to` の component へ。書いていない辺はすべて禁止で、同じ component の中の辺も書かなければ禁止 |
| `kinds` | 許す辺の種類。省けばすべての種類を許す |
| `description` | なぜ許すか。任意 |

違反は2種類ある。

| 違反 | 意味 |
|---|---|
| `not-in-allowed` | allow に無い辺（種類が kinds に無い辺を含む） |
| `uncovered` | どの component にも入らない node。外の依存も、規則に書かなければこれになる |

## グラフの読み取り

判定の中心は、node と edge のグラフ（形は `graph.schema.json`）と規則だけを受け取る。言語ごとの読み取りがグラフを作る。

| 読み取り | node の名前 | kind の値 |
|---|---|---|
| Rust（`--cargo`） | package の name。依存では `dependencies` の name で、rename の名前は使わない | `normal` ・ `dev` ・ `build` |

Rust の読み取りは `cargo metadata --no-deps --format-version 1` を起動し、workspace の member が Cargo.toml に書いた依存を辺にする。optional の依存も feature に依らず入る。target（プラットフォームの条件）は見ない。外の node は、member の直接の依存だけである。

ほかの言語は、`graph.schema.json` の形のグラフを出すプログラムを書き、`check --graph` に渡す。

## 版

`rules.schema.json` ・ `graph.schema.json` ・ `result.schema.json` は `version` を持つ。欄を足すだけなら同じ版、意味を変えるなら版を上げる。読めない版は終了コード 2 になる。

## 中の構成

| モジュール | 中身 |
|---|---|
| `src/core.rs` | 判定の中心。入出力をしない。言語の語を持たない |
| `src/graph.rs` | 公開する形の型（規則 ・ グラフ ・ 結果の JSON）。core の型と相互に直す |
| `src/cargo.rs` | Rust の読み取り |
| `src/main.rs` | CLI。引数 ・ 出力 ・ 終了コード |

試験は `cargo test`。`tests/check.rs` が小さな workspace を一時ディレクトリに作り、実行ファイルを起動して確かめる。
