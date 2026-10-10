# skills-creator の道具のソース（提供者向け）

**この文書と `tool/` の下は、配布物に入らない**（`release.yml` が梱包から外す）。
利用者に届くのは、組み立てた `bin/` と `references/` ・ SKILL.md ・ tool.json だけである。

---

## ソースの地図

| 置き場所 | 中身 |
|---|---|
| `business_logic/src/scaffold.rs` | 受け取った一式を置く。**何を置くかは型とリファレンス実装の定義が決める** |
| `business_logic/src/sample.rs` | リファレンス実装（`references/sample/rust/sample.json`）・ 枠（`references/types/skeleton.json`）・ 型（`references/types/<型>/type.json`）を読み、置く一式を決める。Rust 以外の言語には枠と型の一式だけを置く |
| `business_logic/src/behavior.rs` | check の1段目。**実行ファイルを起動して振る舞いを確認する** ── 道具の一覧の JSON ・ 旗の拒否 ・ Skill のフォルダの求め方 ・ MCP の `tools/list` との一致 |
| `business_logic/src/check.rs` | check の2段目と文書の検査。外部の道具の直書き ・ 雛形の構成（`--layout 1`）・ 節の構成 ・ 未記入の差し込み場所 |
| `business_logic/src/sections.rs` | 節の構成を、対応する雛形と照合する。**節の名前は雛形が保持する** |
| `business_logic/src/accept.rs` | 助言型の受け入れの検査（機械の7件）。回答の例は `tool/` の下の `answer*.json` を探し、試験のコマンドは `tool.json` の `test` から読む ── 言語に依存しない |
| `business_logic/src/cases.rs` | 契約のテストケース（`references/contract/cases/`）を、検証する Skill の tool.json の実行コマンドで実行し、期待値と JSON の値として比較する。リファレンス実装のテストとテストケースの対応も照合する |
| `business_logic/src/refs.rs` | この Skill 自身の references の実装（Rust の組の雛形 `refs.rs.tmpl` と同じ処理） |
| `business_logic/tests/` | 事例。生んだものが契約を満たすこと ・ 検査の検出 ・ references の実装 ・ テストケースの実行を固定してある |
| `business_logic/src/provider.rs` | 提供者の検証（verify）。リファレンス実装から2型を生み、`tool.json` の組み立て ・ テスト ・ フォーマットと check（テストケースを含む）・ accept を行う。テストとテストケースの対応と、ほかの言語の枠も確かめる。**シェルを経由しない** |
| `business_logic/src/view.rs` | 見た目の複製の検査（複製と正本の差 ・ 色の直値 ・ 定まらない変数 ・ 文字と地の比） |
| `service/src/lib.rs` | 道具の一覧（能力の正本）── 利用者の `scaffold` ・ `check` ・ `accept` ・ `dist` と、提供者の `verify`（`provider_tools`） |
| `cli/` ・ `mcp/` | 2つのプレゼンテーション層。`cli/examples/provider.rs` は提供者の道具の入口で、**配布する実行ファイルに入れない** |

---

## リファレンス実装を直したとき

共通ツール ・ CLI ・ MCP の実装は、Rust のリファレンス実装（`references/sample/rust/`）が1組だけ持つ（ACDR 0097）。
**振る舞いは契約のテストケース（`references/contract/cases/`）が決める** ── リファレンス実装のテストを足したら、同じ名前のテストケースも足す。

直したら、公開の前に次を実行する。

```
cargo install --force --path tool/cli --root . --target-dir tool/target
cargo install --force --path tool/mcp --root . --target-dir tool/target
cargo run -q --release --manifest-path tool/Cargo.toml -p sc_cli --example provider -- verify --skill_root "$PWD" [--work <作業場所>] [--json]
```

作業場所（既定は `tool/target/verify`）の下に新しい置き場所を作り、利用者と同じ道具（`bin/skills-creator` の scaffold）でリファレンス実装から作業型と助言型を生む。
生んだ Skill の `tool.json` が持つ組み立て ・ テスト ・ フォーマットのコマンドを実行し、check（テストケースを含む）と accept を当てる。
リファレンス実装のテストとテストケースの対応と、ほかの言語の枠が置けることも確かめる。**シェルを経由しない** ── コマンドは語の並びとして起動し、検出は JSON で受け取る。**終了コードが0なら、公開してよい。**

既存の Rust の Skill の references の実装（`tool/business_logic/src/refs.rs`）は、リファレンス実装の複製である。直したら複製し直し、各 Skill で `skills-creator check <フォルダ> --cases 1` を実行する。

必要な処理系は cargo である。
