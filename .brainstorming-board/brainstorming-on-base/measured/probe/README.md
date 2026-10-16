# 子のプロセスとして呼んだときの実測（2026-10-17）

試験用のリポジトリ（git init した空のフォルダ）に、次の3つを置いて実行した。

- `AGENTS.md`：「The secret word for this repository is "AMBERFOX".」
- `.agents/skills/probe-skill/SKILL.md` と `.kiro/skills/probe-skill/SKILL.md`：name が probe-skill の Skill

版：codex-cli 0.163.0 ・ kiro-cli 2.21.1。どれも標準入力を閉じて（`</dev/null`）実行した。

| ファイル | コマンド | 分かったこと |
|---|---|---|
| codex-probe.jsonl | `codex exec --json -s read-only "<AGENTS.md の合言葉と、使える Skill の名前>"` | AGENTS.md の合言葉（AMBERFOX）を答え、Skill の一覧に probe-skill が在った |
| codex-resume.jsonl | `codex exec resume --json <thread_id> "<前の回の答え>"` | `thread.started` の `thread_id` で再開でき、入力のトークンが 15401 から 30925 に増えた（前の会話を持っている） |
| kiro1.jsonl | `kiro-cli chat --agent-engine v2 --output-format stream-json "<4821 を覚えて>"` | `--agent-engine` を付けないと `runError`（v1 では stream-json を使えない）。v2 では `metadata` と `runFinished` の `sessionId` に ID が出る |
| kiro2.jsonl | `kiro-cli chat --agent-engine v2 --output-format stream-json --resume-id <sessionId> "<覚えた数>"` | stream-json（--no-interactive を含意する）と --resume-id を一緒に使えて、4821 と答えた |
| kiro3.jsonl | `kiro-cli chat --agent-engine v2 --output-format stream-json "<使える Skill の名前>"` | `.kiro/skills/` の probe-skill を答えた |

標準入力を閉じないと、`codex exec` は「Reading additional input from stdin...」と出して待ち続けた（180 秒で打ち切った）。
