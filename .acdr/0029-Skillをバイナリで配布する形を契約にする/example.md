# 変更前と変更後を、具体の形で比べる

ここでは、道具を持つ Skill の1つ（acdr）を例として示す。
**同じ形が、道具の契約に従うすべての Skill に適用される。**

## 例1：Skill のフォルダの構成

### 変更前

| 置き場所 | 中身 | 配布物に入るか |
|---|---|---|
| `SKILL.md` ・ `references/` | 手順と文書 | 入る |
| `rs/` | 道具のソース（Rust の workspace） | 入る（ソースとして） |
| `rs/target/release/acdr` ・ `acdr-mcp` | 組み立てた実行ファイル | **入らない**（`.gitignore` で除外） |
| `mcp.json` | MCP の登録。`/home/daidaiiro/…/rs/target/release/acdr-mcp` を指す | 入る（開発機の絶対パスのまま） |

### 変更後

| 置き場所 | 中身 | 配布用の zip に入るか |
|---|---|---|
| `SKILL.md` ・ `references/` | 手順と文書 | 入る |
| `tool/` | 道具のソース（いまの `rs/`） | **入らない** |
| `bin/acdr` ・ `bin/acdr-mcp` | 組み立てた実行ファイル。Windows では `.exe` が付く | **入る** |
| MCP の設定 | 導入スクリプトが、利用者の環境に合わせて書く | ── |

## 例2：利用者の操作

### 変更前

1. `skills.zip` を取得して、`.claude/skills/` に展開する
2. Rust を導入する
3. Skill ごとに `cargo build --release` を実行する
4. `mcp.json` の絶対パスを、自分の環境の経路に書き換える

### 変更後

| 利用者の環境 | 実行する1行 |
|---|---|
| macOS ・ Linux ・ WSL | `curl -fsSL https://…/install.sh \| bash` |
| Windows（PowerShell） | `irm https://…/install.ps1 \| iex` |

導入スクリプトが OS と CPU を判別し、合う配布用の zip を取得して展開する。
Skill の名前を並べると、その Skill だけを導入する。更新も同じ1行である。

## 例3：道具の契約の文言

### 変更前（`skills-creator/references/tool-contract.md` の132行目）

> **部品は、置き場所を階層の数で数えない。** 実行ファイルからの相対で数えると、置き場所を動かすたびに狂う
> ── Skill の置き場所は `--skill_root` で受け取る。

### 変更後（案）

> **実行ファイルは `bin/` に配置し、Skill のフォルダは `bin/` の1つ上とする。** 置き場所は契約が決めるので、
> 配布しても変わらない。`--skill_root` を渡したときは、それを優先する。

## 例4：OS によって無い外部コマンド

acdr は、記録の日付を外部コマンド `date +%Y-%m-%d` で取得している（`record.rs` の506行目）。

| 環境 | 変更前 | 変更後 |
|---|---|---|
| Linux ・ macOS | 動く | 動く |
| Windows ネイティブ | `date` は実行ファイルとして存在しないので、日付が空になる（platform-advisor の推測。未測定） | Rust の中で日付を計算するので、同じ結果になる |
