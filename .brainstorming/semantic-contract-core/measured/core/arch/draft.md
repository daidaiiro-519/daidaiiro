# 実装のアーキテクチャの案（下書き。レビューに渡す）

前提：核（model/model.json の業務領域11 ・ 集約9 ・ ドメインサービス ・ ポート）と、利用者の指示（premises/user-instructions.md の末尾）。Rust の原典は sources/rust/。

## 1. なぜポートとアダプターか（核から）

| 核の事実 | 構成への帰結 |
|---|---|
| 変更の仕組み（Git ・ スナップショット ・ ほか）、AI ツール（Claude Code ・ Codex ・ Copilot ・ Gemini ・ Kiro）、保存（ファイル ・ SQLite）を差し替えられる（利用者「アダプターが変わったとしても機能するように」） | 業務の規則はそれらを知らない。業務の側が求める口（ポート）を定義し、外の技術の側がそれを満たす（依存を内向きにする） |
| 同じ入力から同じ結果を返す（Q0 #14） | 業務の規則は入出力をしない。時刻もポートか値で受け取る |
| 入口が複数ある（CLI ・ MCP ・ CI ・ 人の入口） | 駆動する側のアダプターも業務の規則の外に置く |

## 2. 使う Rust の性質（原典で照合する）

| 性質 | 原文 | 構成への使い方 |
|---|---|---|
| crate はコンパイラが一度に扱う最小の単位 | ch07-01:5 | 依存の向きを守る単位を crate にする |
| ワークスペースの crate どうしの依存は明示する | ch14-03:150-151 | 許す辺だけを Cargo.toml に書く。書いていない辺はビルドが通らない |
| 依存の図は循環を許さない | cargo resolver:568 | 向きの逆流はビルドで止まる |
| 中のものは既定で非公開 | ch07-03:285 | crate の中の実装の詳細を隠す |
| orphan rule | reference implementations:155 | ポートの trait を業務の crate に置き、アダプターの crate が自分の型に実装する（許される） |
| 静的 ・ 動的ディスパッチ | ch18-02:234-244、API Guidelines flexibility | 実装が1つに決まるポートはジェネリクス、利用者の環境で替わるポート（AI ツール）は dyn |

## 3. crate の切り方の案

| 案 | 切り方 | 依存の向きを何が守るか |
|---|---|---|
| A | 業務の crate 1つ（中は業務領域ごとのモジュール）＋ アダプターの crate 1つ | 業務 ← アダプターだけはビルドが守る。業務領域どうしの向きはレビュー頼み |
| B（下書きの推し） | 業務領域ごとの crate ＋ 外の技術ごとのアダプターの crate ＋ 結合の crate ＋ 入口の crate | 業務領域どうし ・ 業務とアダプター ・ アダプターどうしの全部の辺を Cargo.toml が持ち、ビルドが守る |
| C | B をさらに domain と application に分ける | 同上。ただし crate が倍になる |

## 4. 案 B の crate（名前は業務領域の語）

業務の側（contexts/）：
- kernel：5つの値 ・ URN ・ id ・ JSON Pointer など、全部の領域が使う型だけ
- declaration（意味の宣言）：Abstract ・ Concrete ・ glossary、作成する ・ Concrete を照らす。Abstract のスキーマのファイルを持つ
- validation（意味の検証）：Resolver ・ Validator・ 任せる先の呼び出し（judge のポート）
- change（変更の確かめと例外の判断）：変更の記録 ・ 判定の記録 ・ 状態を導く ・ 変更を見せる ・ 例外をまとめる（changes のポート）
- session（目的ごとのセッションの管理）：セッション ・ 再開か新規か（sessions_store ・ ai_session ・ settings のポート）
- search（意味で探す）・ render（描画：ビュー ・ デザインシステム）・ instance（作成と取り込み）・ migration（移す）・ publish（公開された言葉の版）

許す辺（業務の側）：kernel ← declaration ← validation ← change、validation ← search、declaration ← render、change → render（描画の差分）、session は kernel だけ。領域をまたぐ使う場面（変更を記録する＝ session の目的を写す、承認して確定する）は workflow の crate が持つ（領域どうしを直接つながない）

外の技術の側（adapters/）：changes-git ・ changes-snapshot ・ store-sqlite ・ files ・ ai-claude-code ・ ai-codex ・ ai-copilot ・ ai-gemini ・ ai-kiro ・ settings-file。各々は自分が満たすポートの crate と kernel にだけ依存する

入口と結合（entry/）：compose（アダプターを workflow に渡す1か所）・ cli ・ mcp（Q4 のあと）

## 5. 決定的にするための検査
- 許す辺の表を1つ持ち、`cargo metadata` の依存の図と突き合わせる試験を置く（表に無い辺が0件）
- 業務の側の crate は入出力の API を禁じる（clippy の disallowed）
- 受け入れの試験（場面の JSON）は compose の crate の結合テストで、全アダプターを回す

## 6. 未決
- workflow の crate を置くか（領域ごとの application に寄せるか）
- 判定の記録の集約を change に置くか validation に置くか（核では業務領域「変更の確かめと例外の判断」）
- AI ツールのアダプターを1つの crate にするか、ツールごとにするか
