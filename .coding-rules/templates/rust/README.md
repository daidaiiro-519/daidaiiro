# Rust の規則ファイル ── 別のリポジトリへ渡す形

この `rules.json` を、対象のリポジトリの `.coding-rules/applied/<成果物>.json` として置き、索引へ1行足す。

**この場所（`templates/`）では実行しない** ── 索引にも載せない。

## 置いたあとに直す2か所

| 欄 | 何を書くか |
|---|---|
| `order` | 層の名前を、内から外の順で書く |
| `layers` | 層の名前と、その crate 名の対応 |

**規則の側は直さない** ── 道具は Rust の標準の道具で、成果物に依存しない。

## 実行

```
python3 <no-more-spaghetti>/scripts/cli.py plan  <リポジトリ> <成果物の名前>
python3 <no-more-spaghetti>/scripts/cli.py check <リポジトリ> <成果物の名前>
```

## 規則4件と、実測した終了コード

| 規則 | 道具 | 違反あり | 違反なし |
|---|---|---|---|
| 整形されている | `cargo fmt --all -- --check` | 1 | 0 |
| 静的解析を通る | `cargo clippy --all-targets -- -Dwarnings` | 101 | 0 |
| 試験が通る | `cargo test --all-targets` | 101 | 0 |
| 既知の脆弱性が無い | `cargo audit` | 1 | 0 |

実測は 2026-09-26、cargo 1.98.1 ・ rustfmt 1.9.0 ・ clippy 0.1.98 ・ cargo-audit 0.22.2 である。

## 依存の向きの規則を立てていない理由

**Rust は crate の依存宣言をコンパイラが強制する** ── 宣言していない crate を参照すると
`cargo check` が停止する。規則を立てる条件①（無ければ外れるか）を満たさない。

層を crate ではなく `mod` で切る場合は、可視性（`pub(crate)`）が同じ役目を果たす。

## 道具が無い環境

判定は「実行しない」になり、**合格には寄らない**。`cargo audit` は別途
`cargo install cargo-audit` が要る。
