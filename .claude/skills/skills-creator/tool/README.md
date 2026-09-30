# skills-creator の道具のソース（提供者向け）

**この文書と `tool/` の下は、配布物に入らない**（`release.yml` が梱包から外す）。
利用者に届くのは、組み立てた `bin/` と `references/` ・ SKILL.md ・ tool.json だけである。

---

## ソースの地図

| 置き場所 | 中身 |
|---|---|
| `business_logic/src/scaffold.rs` | 受け取った一式を置く。**何を置くかは型と言語の組の定義が決める** |
| `business_logic/src/profile.rs` | 言語の組（`references/profiles/<言語>.profile.json`）と型（`references/types/<型>/type.json`）を読み、置く一式を決める |
| `business_logic/src/behavior.rs` | check の1段目。**実行ファイルを起動して振る舞いを確認する** ── 道具の一覧の JSON ・ 旗の拒否 ・ Skill のフォルダの求め方 ・ MCP の `tools/list` との一致 |
| `business_logic/src/check.rs` | check の2段目と文書の検査。外部の道具の直書き ・ 雛形の構成（`--layout 1`）・ 節の構成 ・ 未記入の差し込み場所 |
| `business_logic/src/sections.rs` | 節の構成を、対応する雛形と照合する。**節の名前は雛形が保持する** |
| `business_logic/src/accept.rs` | 助言型の受け入れの検査（機械の7件）。回答の例の置き場所とソースの拡張子は、言語の組の定義から読む |
| `business_logic/src/conform.rs` | 言語の組の references の実装の出力の突き合わせ（提供者だけが使う） |
| `business_logic/src/refs.rs` | この Skill 自身の references の実装（Rust の組の雛形 `refs.rs.tmpl` と同じ処理） |
| `business_logic/tests/` | 事例。生んだものが契約を満たすこと ・ 検査の検出 ・ references の実装 ・ 突き合わせの判定を固定してある |
| `business_logic/examples/conform.rs` | 突き合わせの入口。**配布する実行ファイルに入れない** |
| `service/src/lib.rs` | 道具の一覧（能力の正本）── `scaffold` ・ `check` ・ `accept` ・ `dist` |
| `cli/` ・ `mcp/` | 2つのプレゼンテーション層 |
| `verify-profiles.sh` | 言語の組の雛形の確認（下の節） |

---

## 言語の組の雛形を直したとき

5つの言語の組は、references の実装（`refs.rs.tmpl` ・ `refs.py.tmpl` ・ `refs.ts.tmpl` ・ `Refs.cs.tmpl` ・ `refs.go.tmpl`）を
それぞれ保持する。**同じ references が、言語によって違って見えてはならない**（ACDR 0069）。

雛形を直したら、公開の前に次を実行する。

```
cargo install --path tool/cli --root . --target-dir tool/target
tool/verify-profiles.sh
```

配布物と同じ形を空の作業場所へ置き、利用者と同じ手順で5言語 × 2型を生んで、組み立て ・ 試験 ・ check ・ accept を実行する。
最後に、Rust の助言型を基準に、他の4言語の出力をリポジトリの references で突き合わせる。**終了コードが0なら、公開してよい。**

突き合わせだけを実行するときは次である。**基準が失敗した事例は、一致と数えない** ── 両方が同じ誤りを返すと、比べていないのに一致に見える。

```
cargo run -q --manifest-path tool/Cargo.toml -p sc_business_logic --example conform -- <基準の Skill> <比べる Skill> [<置き場所>]
```

必要な処理系は cargo ・ uv ・ node（22.18 以上）・ dotnet（10）・ go である。
