# 配布

## 目的

作った Skill を、**他の人の環境へ配るときだけ**使う。**配布は契約の外の任意の機能である**（ACDR 0036）
── skills-creator で Skill を作る人の多くは、自分のリポジトリで使うために作る。

配布の仕組み（配布物 ・ 導入スクリプト ・ 組み立ての定義）は、**全言語で1つを共有する**。
言語の組からは、組み立てのコマンドだけを借りる。**入口が `bin/` に在ることを前提にする。**

---

## 規定

**利用者の環境に Rust が無くても動く形で配る。** 利用者はどの実行ファイルかを選ばない（ACDR 0029 ・ 0031）。

| 項目 | 規定 |
|---|---|
| 配布物 | Skill ごと × 環境ごと。中身は Skill のフォルダから `tool/` ・ `mcp.json` を除いたもの（SKILL.md ・ references/ ・ bin/ ・ tool.json など） |
| 形式 | Linux ・ macOS は tar.gz、Windows は zip。道具を保持しない Skill は、環境に依存しない tar.gz と zip を1つずつ |
| 名前 | `<Skill の名前>-<環境>.tar.gz` ・ `.zip`（環境は x86_64-unknown-linux-musl ・ x86_64-pc-windows-msvc ・ universal2-apple-darwin）。環境に依存しないものは `-any` |
| 導入 | 配布元のリポジトリの `install.sh`（macOS ・ Linux ・ WSL）と `install.ps1`（Windows）。OS と CPU を判別し、SHA-256 を照合してから展開し、MCP を `claude mcp add --scope project` で登録する |
| 組み立て | 配布元のリポジトリの `.github/workflows/release.yml`。各 OS のランナーで組み立て、組み立てた場所と展開した先の2か所で起動を試験してから公開する |

導入スクリプトと組み立ての定義は、**配布元のリポジトリに1つずつ置く**。雛形は `references/distribution/` に在り、置く操作は次である。

```
skills-creator dist --repo <所有者/リポジトリ> [--path <配布元のリポジトリ>]
```
