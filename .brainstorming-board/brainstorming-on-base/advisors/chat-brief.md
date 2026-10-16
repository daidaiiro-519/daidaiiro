# 相談の材料：brainstorming の画面に AI のチャットを内蔵する（論点4）

## 前提（決まっていること）
- brainstorming は、ブレストボード（決めることの論点 ・ 答え ・ 完成イメージ）を作り、利用者が画面で承認 ・ 差し戻し ・ 書き足しをする Skill。データは `.brainstorming/<ボード>/`。正本はボードの JSON インスタンス（論点 ・ 回の記録 rounds/<n> ・ 回答）。
- serve（Rust の実行ファイル）が 127.0.0.1 で画面を配る。利用者の手元の1台で、1人で使う。
- 画面に「AI に聞く」のチャットを置く。方式は A＝serve が AI のコーディング用 CLI を子のプロセスとして呼び、JSON Lines の出来事の流れを受け取って画面へ中継する。対象は Claude Code（`claude -p --output-format stream-json`）・ Codex（`codex exec --json`）・ Kiro（`kiro-cli chat --no-interactive --output-format stream-json`）。利用者がログイン済みの CLI をそのまま使う。
- 出来事の名前は CLI ごとに違うので、serve の側で共通の形に直す。
- 権限を外す旗（`--dangerously-skip-permissions` ・ `--dangerously-bypass-approvals-and-sandbox` ・ `--trust-all-tools`）は使わない。

## 原文で確かめた事実（2026-10-17 取得）
- Claude Code：`--session-id` で呼ぶ側が UUID を決められる。`--resume <ID>` で再開。`--bare` を付けなければ作業フォルダの skills ・ CLAUDE.md を対話と同じく読み込む。`--allowedTools` で聞かずに使うツールを挙げ、`--permission-mode dontAsk` で聞くはずの呼び出しを拒否する。
- Codex：`codex exec` は既定で読み取り専用の sandbox。`--sandbox workspace-write` で書き換えを許す。`thread.started` に `thread_id`。`codex exec resume <SESSION_ID>`。`-C` で作業の根。
- Kiro：`--resume-id <ID>`。`--trust-tools` で信頼するツールを限定。`permissions.yaml` の規則（deny ・ ask ・ allow）。
- 未確認：Codex の exec がリポジトリの Skill と AGENTS.md を自動で読むか ／ Kiro で --no-interactive と --resume-id を一緒に使えるか ／ Kiro の stream-json のどこにセッションの ID が出るか（手元で実測して埋める）。

## いまの見解（反証してほしい）
1. 文脈の引き継ぎ：会話の続きは各 CLI の再開の機能（セッションの ID）を使う。ボードの中身を写した context.json は作らない（AI がボードを直した時点で古くなるため）。serve が持つのは `.brainstorming/<ボード>/chat/<AI>.json`（セッションの ID と、やりとりの記録。追記だけ）。毎回の依頼の頭に「ボードのフォルダ ・ 回の番号 ・ 論点の id」だけを付け、中身は AI が Skill の入口（inspect ・ get）でその時点のものを読む。再開できないとき（別のマシン ・ 期限切れ）は新しいセッションを始め、やりとりの記録とボードを読ませる。
2. Skill の使い方を知らせる：各 CLI の Skill の自動の発見に頼りきらない。新しいセッションの最初の依頼に「`.claude/skills/brainstorming/SKILL.md` を読み、その手順で `scripts/brainstorming` を使う。対象は … 論点は …」を必ず入れる。子のプロセスの作業フォルダはリポジトリの根。
3. 権限：Skill の入口の実行と、ボードのフォルダの書き換えだけを許す。AI が「検査した」と言っても信じず、作業が終わったら serve が自分で check を走らせ、その結果を画面の「検査」の行に出す。
4. serve の境界：127.0.0.1 だけで待ち受け、起動ごとに作る合言葉（トークン）を画面の URL に付け、合言葉の無い要求は断る。子のプロセスは1つのボードにつき同時に1つ。

## 聞きたいこと
見解1〜4を採ってよいか。セキュリティの境界（ローカルのサーバーが AI の CLI を起動し、その CLI がファイルを書き換える）として足りないところ、運用（子のプロセスの止め方 ・ 時間切れ ・ 失敗したときの戻し方）で足りないところはどこか。
