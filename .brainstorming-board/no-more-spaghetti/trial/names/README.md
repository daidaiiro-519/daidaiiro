# 層を指す名前の解決の試作（論点13）

no-more-spaghetti の `inward` に、参照の名前空間と置き場所の名前空間をつなぐ対応表を足した試作である。
正本の Skill には適用していない。

| 置いてあるもの | 中身 |
|---|---|
| `names.patch` | 正本との差分。`parts/src/inward/names.rs` を新設し、`syntax.rs` ・ `mod.rs` ・ `declare/src/lib.rs` ・ `parts/Cargo.toml`（`toml` を追加）を変更する |
| `fixtures/` | 9言語の見本。各言語で、内側の core ・ 外側の adapter ・ 層を宣言しない stray を持つ |
| `repositories.txt` | 測った実在のリポジトリと、その commit |
| `measure.py` | 実在のリポジトリを測った手順（経路は作業用の場所のまま） |

## 対応表の出どころ

| 言語 | 読む設定 |
|---|---|
| Rust | 各 `Cargo.toml` の `[package] name`、`[lib] name` ・ `path` |
| Go | 各 `go.mod` の `module` |
| TypeScript | 各 `package.json` の `name`、各 tsconfig の `paths` ・ `baseUrl` ・ `extends`（適用範囲はその tsconfig の下）、package の source root（`rootDir`、無ければ `src`） |
| Python | `pyproject.toml` の setuptools ・ hatch ・ poetry の書き方、無ければ `src` |
| Ruby | gemspec の `require_paths`、無ければ `lib` |
| C++ | `CMakeLists.txt` の `include_directories` ・ `target_include_directories`（生成式と変数を展開する）、`compile_commands.json` の `-I` |
| Java ・ Kotlin ・ C# ・ PHP | 不要（ソースが自分の名前を宣言する） |

**読めなかった設定は、層に属す点がその下に在るときだけ、判定できなかった範囲として報告する。**

## 実測（2026-09-28）

### 9言語の見本

| 言語 | いま：逆の並び | いま：stray を宣言しない | 試作：逆の並び | 試作：stray を宣言しない |
|---|---|---|---|---|
| TypeScript ・ Python ・ Ruby ・ C++ ・ Go | 合格（検出しない） | 合格（検出しない） | 違反を検出 | 未解決を報告 |
| Java ・ Kotlin ・ C# ・ PHP | 違反を検出 | 合格（検出しない） | 違反を検出 | 未解決を報告 |

### 実在のリポジトリ

層は、各リポジトリの package ・ crate ・ module の単位で宣言した。

| リポジトリ | 言語 | 辺 | 層どうしの辺（対応表なし → あり） | 書き換えた参照のうち、実在しない先 | 判定できなかった範囲 |
|---|---|---|---|---|---|
| ripgrep | Rust | 404 | 73 → 152 | 0 / 80 | 0 |
| opentelemetry-go（go.mod 29件） | Go | 5,875 | 14 → 2,380 | 0 / 2,486 | 0 |
| trpc（monorepo） | TypeScript | 3,555 | 1,257 → 1,985 | 2 / 938（生成されるファイルで、リポジトリに無い） | 8（根の `vitest.shared.ts` がどの層にも属さない） |
| black（src の配置） | Python | 710 | 9 → 118 | 0 / 144 | 0 |
| rubocop | Ruby | 362 | 199 → 256 | 0 / 68 | 0 |
| fmt | C++ | 168 | 22 → 37 | 0 / 100 | 0 |
| gson | Java | 2,674 | 1,058（対応表は不要） | ─ | 1（`com.example` がどの層にも属さない）＋ 3（`module-info.java` は名前を宣言しない） |

既存の11のまとまり（Rust の Skill 10件と Go の見本）も、置き場所の名前だけで、正しい並びは合格し、逆の並びは検出した。
既存の試験48件は成功し、clippy の警告は0件だった。

## 実在のリポジトリで判明し、直した不具合

| 不具合 | 直し方 |
|---|---|
| 外の部品名 `vitest` が、根のファイル `vitest.config.ts` に一致した | 経路で書く言語は、区切りを `/` だけにした |
| 読み込みの文の拡張子（`vitest.shared.ts`）で、点と一致しなかった | 経路で書く言語は、両側の拡張子を外して比べた |
| package の subpath が、組み立ての出力（dist）を指した | package の source root の下も探すようにした（実在しない先が494件から2件へ） |
| 例ごとに別の包みを指す tsconfig の `paths` を、全体へ適用した | `paths` の適用範囲を、その tsconfig の下に限定した |
| 層の外の、生成物を指す tsconfig を読めないと報告した | 層に属す点がその設定の下に無ければ、報告しない |
| `require "rubocop/#{feature}/version"` を、静的な経路として扱った | 埋め込みを含む文字列は、図に現れない読み込みとして別に出す |
