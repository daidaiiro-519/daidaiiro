# 配布の形の試作（Linux の1環境、2026-09-29）

ACDR 0029 の「承認後に実施すること」の試作である。題材は acdr で、正本には適用していない。

| 置いてあるもの | 中身 |
|---|---|
| `acdr-skill-root.patch` | acdr の宣言（declare/src/lib.rs）の差分。build 時の絶対パスをやめ、`--skill_root` → 実行ファイルの1つ上の順で Skill のフォルダを求める |
| `install.sh` | 導入スクリプトの試作。OS と CPU を判別し、SHA-256 を照合してから展開し、MCP を登録する |

## 手順

1. acdr を複製し、`rs/` を `tool/` へ改名した
2. 宣言の差分を適用し、`cargo install --path tool/cli --root . --target-dir tool/target` で `bin/` へ出力した（mcp も同じ）
3. `SKILL.md` ・ `LICENSE` ・ `references/` ・ `bin/` だけを tar.gz にまとめ、SHA-256 の一覧を作った
4. 別の git リポジトリで、配布元を `file://` にして `install.sh` を実行した

## 結果

| 確認したこと | 結果 |
|---|---|
| リポジトリの直下から、`--skill_root` 無しで実行 | 動く（new ・ validate ・ tokens） |
| 別の場所（/tmp）から実行 | 動く |
| references/ が無い場所に置いた実行ファイル | 終了コード2で停止する。ただし文言の経路が二重になる（「…/references に references/ が無い」）── 本実装で直す |
| `--skill_root` を渡した場合 | 渡した場所を優先する |
| MCP 経由の呼び出し | 動く |
| 導入スクリプト | 判別 ・ 取得 ・ SHA-256 の照合 ・ 展開 ・ MCP の登録まで通る |

## 試作で判明したこと

| 判明したこと | 扱い |
|---|---|
| この環境には `zip` コマンドが無い。Linux（WSL の最小構成を含む）では、`zip` ・ `unzip` が標準で入っていないことがある | Linux と macOS 向けは tar.gz、Windows 向けは zip にする。ACDR 0029 の「配布用の zip」を改める必要がある |
| `cargo install --root` は、Skill のフォルダの直下に管理用のファイル（`.crates.toml` ・ `.crates2.json`）を作る | 配布物から除外し、開発時は `.gitignore` に入れる。または `cargo build` のあとに `bin/` へ複製する手順にする |
| MCP の登録は、`claude mcp add --scope project <名前> -- <経路>` で書ける。`jq` や `python3` が無くても、JSON を壊さずに追記できる | 導入スクリプトはこの方法を採る。`claude` が PATH に無いときは、登録を省いて案内を出す |
| `.mcp.json` には `${CLAUDE_PROJECT_DIR:-.}` がそのまま書かれる | 導入したリポジトリの場所に依存しない |

## 原文での照合（2026-09-29）

| 点 | 原文 | 確定したこと | 残る推測と、その扱い |
|---|---|---|---|
| Windows で拡張子の無い経路から .exe を起動できるか | Microsoft の CreateProcessW の文書 ・ libuv の src/win/process.c（search_path） | CreateProcess は「If the file name does not contain an extension, .exe is appended.」。libuv は経路を含む名前でも .com、次に .exe を補って探す | Claude Code が MCP サーバーを libuv で起動しているかは未確認である。Windows 向けの設定には .exe を明記する |
| macOS の隔離の属性 | Apple の Gatekeeper の文書 ・ LSFileQuarantineEnabled の文書 | 隔離の属性は、ファイルを作ったアプリの設定で決まる（「whether the files this app creates are quarantined by default」）。Gatekeeper は、ダウンロードしたソフトウェアを初めて開くときに承認を求める | 端末の curl で取得したファイルに属性が付かないことは、原文に記述が無い。導入スクリプトは SHA-256 の照合のあとに必ず属性を外す |

## まだ確認していないこと

- musl で静的に組むこと（この環境に musl の target が無い）
- macOS と Windows での動作、install.ps1
- Claude Code が Windows で MCP サーバーを起動する方法（libuv かどうか）
