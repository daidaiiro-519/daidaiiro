# 言語をまたいだ検証 ── 2026-09-24

同じ抽象（概念の層 ・ 識別子 ・ 依存の向き ・ 対象）を、4つの言語で実際に動かした記録である。

| 言語 | 層の識別子の形 | 依存の向きを検査する相手 | 版 |
|---|---|---|---|
| Go | モジュールパス（`example/app/internal/core`） | `go build ./...` ── **言語が強制する** | go1.27.1 |
| Rust | crate 名（`core_layer`） | `cargo check --workspace` ── **Cargo が強制する** | cargo 1.98.1 |
| Python | dotted path（`app.core`） | `lint-imports`（import-linter の layers 契約） | import-linter |
| TypeScript ・ JavaScript | ディレクトリ（`src/core`） | `dependency-cruiser` の forbidden 規則 | 18.4.0 |

各ディレクトリに、違反を入れる前後の2つの形を置いてある（`*.ok` と `*.violation`）。

    cd <言語> && python3 <skill>/scripts/cli.py check . .coding/rules.json

**Go の例だけ、`check.target` の実装が要る**（論点7 の要求事項）── 規則は `other/` で
`go build` を実行する形なので、根で走らせると `go.mod` が見つからずに落ちる。

鍵は ASCII である（`rules` ・ `order` ・ `layers` ・ `rule` ・ `source` ・ `scope` ・
`check` ・ `tool` ・ `target`）。
