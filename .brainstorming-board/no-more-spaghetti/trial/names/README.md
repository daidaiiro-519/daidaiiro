# 層を指す名前の解決の試作（論点13）

no-more-spaghetti の `inward` に、参照の名前空間と置き場所の名前空間をつなぐ対応表を足した試作である。
正本の Skill には適用していない。

| 置いてあるもの | 中身 |
|---|---|
| `names.patch` | 正本との差分。`parts/src/inward/names.rs` を新設し、`syntax.rs` ・ `mod.rs` ・ `declare/src/lib.rs` を変更する |
| `fixtures/` | 9言語の見本。各言語で、内側の core ・ 外側の adapter ・ 層を宣言しない stray を持つ |

## 再現の手順

1. 正本の `rs/` を作業用の場所へ複製し、`names.patch` を適用して build する
2. 各言語の見本へ `inward` を実行する。層は置き場所の名前だけで渡す

## 実測（2026-09-28）

| 言語 | いま：逆の並び | いま：stray を宣言しない | 試作：逆の並び | 試作：stray を宣言しない |
|---|---|---|---|---|
| TypeScript ・ Python ・ Ruby ・ C++ ・ Go | 合格（検出しない） | 合格（検出しない） | 違反を検出 | 未解決を報告 |
| Java ・ Kotlin ・ C# ・ PHP | 違反を検出 | 合格（検出しない） | 違反を検出 | 未解決を報告 |

既存の11のまとまり（Rust の Skill 10件と Go の見本）も、置き場所の名前だけで、正しい並びは合格し、逆の並びは検出した。
既存の試験48件は成功し、clippy の警告は0件だった。

## この試作の限界

- 設定のファイルを文字列として読むだけで、解析はしない（TOML の1行 ・ CMake の `include_directories` ・ コメントの無い `tsconfig.json`）
- Python の探索路は、`pyproject.toml` の `where` か、`src` の在ることから決める
- Ruby の load path は、`lib` の在ることから決める
