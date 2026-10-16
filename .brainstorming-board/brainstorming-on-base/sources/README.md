# AI のコーディング用 CLI を子のプロセスとして呼ぶための事実（原文で照合）

- 取得した日：2026-10-17
- 原文の置き場所：このフォルダ（各ファイルの横に `<名前>.meta.json`。原文そのものはリポジトリで追跡せず、meta.json の URL から取り直す）
- 手元の help の実測：ボードの `measured/`（claude 2.1.301 ・ codex-cli 0.163.0 ・ kiro-cli 2.21.1。2026-10-17 に `--help` だけを実行）
- どう読んだか：`fact-check fetch` で原文（`.md`）を取得し、引用は `fact-check verify --as quote` で1件ずつ照合した。**原典の引用 80 件を照合し、80 件とも原文に在った**（読めた範囲に無い 0 ／ 読めなかった 0）。実測の行番号は `measured/*.txt` の行番号である
- 印：【原典】＝ verify で照合した引用 ／【実測】＝ 手元の `--help` の出力 ／【推測】＝ 引用を添えられない、こちらの推論

ファイル名の略記：
- CC-headless ＝ `code.claude.com_docs_en_headless.md`
- CC-cli ＝ `code.claude.com_docs_en_cli-reference.md`
- CC-auth ＝ `code.claude.com_docs_en_authentication.md`
- CC-sdk ＝ `code.claude.com_docs_en_agent-sdk_typescript.md`
- CX-ni ＝ `developers.openai.com_codex_noninteractive.md`
- CX-ref ＝ `developers.openai.com_codex_cli_reference.md`
- CX-sec ＝ `developers.openai.com_codex_agent-approvals-security.md`
- CX-auth ＝ `developers.openai.com_codex_auth.md`
- KR-hl ＝ `kiro.dev_docs_cli_headless.md`
- KR-cmd ＝ `kiro.dev_docs_reference_cli-commands.md`
- KR-auth ＝ `kiro.dev_docs_getting-started_authentication.md`
- KR-acp ＝ `kiro.dev_docs_cli_acp.md`
- KR-perm ＝ `kiro.dev_docs_permissions.md`

---

## Claude Code

### 1. 1回の依頼を渡して終わる呼び方

| 主張 | 引用（原文そのまま） | 出どころ | 照合の結果 |
|---|---|---|---|
| `-p` に依頼を渡すと、対話しないモードで動く | To run Claude Code in non-interactive mode, pass `-p` with your prompt and the [CLI options](/docs/en/cli-reference) you need: | CC-headless:11 | 【原典】ok |
| 実測でも `-p, --print` が在る | `-p, --print                           Print response and exit (useful for` | measured/claude--help.txt:170 | 【実測】 |
| `--bare` を付けないと、作業フォルダや `~/.claude` の設定を対話のときと同じく読み込む | Without it, `claude -p` discovers and loads them as an interactive session would, including anything configured in the working directory or `~/.claude`. | CC-headless:37 | 【原典】ok |

### 2. 機械が読める形の出力と、流れに含まれる出来事

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--output-format` は `text` ・ `json` ・ `stream-json` を取る | \| `--output-format` \| Specify output format for print mode (options: `text`, `json`, `stream-json`) \| | CC-cli:112 | 【原典】ok |
| `json` は結果 ・ セッションの ID ・ メタデータを持つ | * `json`: structured JSON with result, session ID, and metadata | CC-headless:150 | 【原典】ok |
| `stream-json` は改行区切りの JSON の流れ | * `stream-json`: newline-delimited JSON for real-time streaming | CC-headless:151 | 【原典】ok |
| 生成中の断片は `--include-partial-messages` で得る。`--print` と `stream-json` が条件 | \| `--include-partial-messages` \| Include partial streaming events in output. Requires `--print` and `--output-format stream-json` \| | CC-cli:101 | 【原典】ok |
| 入力も `stream-json` にできる | \| `--input-format` \| Specify input format for print mode (options: `text`, `stream-json`) \| | CC-cli:102 | 【原典】ok |
| 流れの最後の行は `result` | The last line of the stream is a `result` message with the final response text, cost, and session metadata. | CC-headless:194 | 【原典】ok |
| 最初は `system/init`（モデル ・ ツール ・ MCP ・ プラグイン） | The `system/init` event reports session metadata including the model, tools, MCP servers, and loaded plugins. | CC-headless:255 | 【原典】ok |
| `assistant` ・ `user` のメッセージが流れる（サブエージェントのものも同じ型で、`parent_tool_use_id` で区別） | Messages from [subagents](/docs/en/sub-agents) and from skills that [run in a subagent](/docs/en/skills#run-skills-in-a-subagent) appear in the stream as `assistant` and `user` messages. | CC-headless:209 | 【原典】ok |
| 文字の断片は `stream_event` の `text_delta` | jq -rj 'select(.type == "stream_event" and .event.delta.type? == "text_delta") \| .event.delta.text' | CC-headless:202 | 【原典】ok |
| 再試行のとき `system/api_retry` | When an API request fails with a retryable error, Claude Code emits a `system/api_retry` event before retrying. | CC-headless:238 | 【原典】ok |
| 拒否は `permission_denied`、最後の結果に `permission_denials` | With `--output-format stream-json`, denials appear as `permission_denied` system messages, and the final result message lists them in `permission_denials`. | CC-headless:333 | 【原典】ok |
| 出来事の型の全体は、Agent SDK の `SDKMessage` の和型（`SDKAssistantMessage` から `SDKConversationResetMessage` まで約35種） | type SDKMessage = | CC-sdk:1398（1399〜1434 が列挙） | 【原典】ok。**CLI の `stream-json` がこの和型と1対1で同じかは、CLI 側の原文に書かれていない →【推測】** |

### 3. 人に聞かずに許可を決める方法

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--allowedTools` に挙げたツールは聞かずに使う | Use `--allowedTools` to let Claude use certain tools without prompting. | CC-headless:300 | 【原典】ok |
| 許可のモードは7つの値を取る | Accepts `default`, `acceptEdits`, `plan`, `auto`, `dontAsk`, `bypassPermissions`, or `manual` as an alias for `default`. | CC-cli:113 | 【原典】ok |
| `dontAsk` は、聞くはずの呼び出しをすべて拒否する | * **`dontAsk`**: Claude Code denies every call that would otherwise prompt, which is useful for locked-down CI runs. | CC-headless:310 | 【原典】ok |
| `acceptEdits` はファイルの書き込みを聞かずに行う | * **`acceptEdits`**: Claude writes files without prompting, and Claude Code auto-approves common filesystem commands such as `mkdir`, `touch`, `mv`, and `cp`. | CC-headless:311 | 【原典】ok |
| `--dangerously-skip-permissions` は `bypassPermissions` と同じ | \| `--dangerously-skip-permissions` \| Skip permission prompts. Equivalent to `--permission-mode bypassPermissions`. | CC-cli:82 | 【原典】ok |
| 答える人が居ないときは `--permission-prompts none` | Pass `--permission-prompts none` when nobody is available to answer permission prompts, for example in a scheduled job. | CC-headless:321 | 【原典】ok |
| 相手（host）の無い `-p` では、聞くはずの要求はどのみち拒否される | In a `-p` run with no host, these requests are denied either way, and the flag also tells Claude not to retry them. | CC-headless:323 | 【原典】ok |
| 実測の `--permission-mode` の選択肢には `default` が無く `manual` が在る | `(choices: "acceptEdits", "auto",` / `"bypassPermissions", "manual",` / `"dontAsk", "plan")` | measured/claude--help.txt:151-153 | 【実測】 |
| sandbox を CLI の旗で指定する方法 | ── | ── | **読めた範囲（headless ・ cli-reference）には無い**。`permissions.md` は取得したが sandbox を照合していない |

### 4. 会話を続ける

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--continue` で直近、`--resume` にセッションの ID で特定の会話を続ける | Use `--continue` to continue the most recent conversation, or `--resume` with a session ID to continue a specific conversation. | CC-headless:376 | 【原典】ok |
| ID は `json` 出力の `session_id` から取る | session_id=$(claude -p "Start a review" --output-format json \| jq -r '.session_id') | CC-headless:390 | 【原典】ok |
| ID で再開するなら、別のフォルダから実行してよい | You can run the two commands from different directories: Claude Code [finds the session by its ID](/docs/en/sessions#where-the-session-picker-looks) in any project on this machine. | CC-headless:394 | 【原典】ok |
| 最初の呼び出しで ID を呼ぶ側が決められる（UUID） | \| `--session-id` \| Use a specific session ID for the conversation (must be a valid UUID) \| | CC-cli:128 | 【原典】ok |
| 再開時に新しい ID へ分岐できる | \| `--fork-session` \| When resuming, create a new session ID instead of reusing the original (use with `--resume` or `--continue`) \| | CC-cli:94 | 【原典】ok |

### 5. 作業するフォルダとモデル

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| 作業するフォルダを指定する専用の旗（`--cd` のようなもの）は、読めた範囲に無い。直近の会話は「現在のフォルダ」で探す | \| `--continue`, `-c` \| Load the most recent conversation in the current directory | CC-cli:80 | 【原典】ok。**「子のプロセスの cwd が作業フォルダになる」は【推測】**（cwd を渡せと書いた行は無い） |
| フォルダを足すのは `--add-dir` | \| `--add-dir` \| Add additional working directories for Claude to read and edit files. | CC-cli:62 | 【原典】ok |
| モデルは `--model`（別名か正式名） | \| `--model` \| Sets the model for the current session with a [model alias](/docs/en/model-config#model-aliases) such as `sonnet`, `opus`, `haiku`, or `fable`, or a model's full name. | CC-cli:108 | 【原典】ok |

### 6. 認証

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `/login` のサブスクリプションの資格情報が Pro ・ Max などの既定 | 7. Subscription OAuth credentials from `/login`. This is the default for Claude Pro, Max, Team, and Enterprise users. | CC-auth:239 | 【原典】ok |
| `-p` では、`ANTHROPIC_API_KEY` が在れば常にそれを使う（ログインより優先） | In non-interactive mode (`-p`), the key is always used when present. | CC-auth:235 | 【原典】ok |
| 長期の OAuth トークンを環境変数で渡せる | 5. `CLAUDE_CODE_OAUTH_TOKEN` environment variable. | CC-auth:237 | 【原典】ok |
| `--bare` ではログインの資格情報を読まない（API キーが要る） | In bare mode, Claude Code never reads OAuth credentials or the system keychain. | CC-headless:49 | 【原典】ok |
| したがって、`--bare` を付けず環境変数も無ければ、利用者のログイン済みの状態をそのまま使う | ── | CC-auth:235,239 と CC-headless:49 を繋いだ | 【推測】（優先順位の一覧から導いた。そう書いた1行は無い） |

---

## OpenAI Codex CLI

### 1. 1回の依頼を渡して終わる呼び方

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| 対話しないモードは `codex exec` | You invoke it with `codex exec`. | CX-ni:6 | 【原典】ok |
| 短い形は `codex e` | Use `codex exec` (or the short form `codex e`) for scripted or CI-style runs that should finish without human interaction. | CX-ref:218 | 【原典】ok |
| 既定では進み具合を stderr、最後のメッセージだけを stdout | While `codex exec` runs, Codex streams progress to `stderr` and prints only the final agent message to `stdout`. | CX-ni:27 | 【原典】ok |
| Git のリポジトリの中が条件（外すのは `--skip-git-repo-check`） | Codex requires commands to run inside a Git repository to prevent destructive changes. | CX-ni:188 | 【原典】ok |
| 実測：依頼を渡さないか `-` なら stdin から読む | `Initial instructions for the agent. If not provided as an argument (or if `-` is used),` | measured/codex-exec--help.txt:14 | 【実測】 |

### 2. 機械が読める形の出力と、出来事

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--json` で stdout が JSON Lines になり、出来事は `thread.started` ・ `turn.started` ・ `turn.completed` ・ `turn.failed` ・ `item.*` ・ `error` | When you enable `--json`, `stdout` becomes a JSON Lines (JSONL) stream so you can capture every event Codex emits while it's running. Event types include `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`. | CX-ni:76 | 【原典】ok |
| item の種類（agent messages ・ reasoning ・ command executions ・ file changes ・ MCP tool calls ・ web searches ・ plan updates）。識別子としては例の `command_execution` ・ `agent_message` だけが原文に在る | Item types include agent messages, reasoning, command executions, file changes, MCP tool calls, web searches, and plan updates. | CX-ni:78（例は 85-86） | 【原典】ok。**残りの item の型の識別子（例：file change の型名）は読めた範囲に無い** |
| `thread.started` が `thread_id` を持つ | {"type":"thread.started","thread_id":"0199a213-81c0-7800-8aa1-bbab2a035a53"} | CX-ni:83 | 【原典】ok |
| 最後の応答を JSON Schema に合わせるのは `--output-schema` | If you need structured data for downstream steps, use `--output-schema` to request a final response that conforms to a JSON Schema. | CX-ni:94 | 【原典】ok |

### 3. 人に聞かずに許可を決める方法

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `codex exec` の既定は読み取り専用の sandbox | By default, `codex exec` runs in a read-only sandbox. | CX-ni:53 | 【原典】ok |
| 書き換えを許すのは `--sandbox workspace-write` | - Allow edits: `codex exec --sandbox workspace-write "<task>"` | CX-ni:55 | 【原典】ok |
| 対話しない実行には `--sandbox workspace-write` を勧める | For non-interactive runs, use `codex exec --sandbox workspace-write`. | CX-sec:369 | 【原典】ok |
| `--full-auto` はもう使えない | `--full-auto` is no longer supported. | CX-ni:60 | 【原典】ok |
| 承認の問い合わせを止めるのは `--ask-for-approval never`（`-a never`） | You can disable approval prompts with `--ask-for-approval never` or `-a never` (shorthand). | CX-sec:302 | 【原典】ok。**ただし実測の `codex exec --help` に `-a` ・ `--ask-for-approval` は無い**（【実測】measured/codex-exec--help.txt 全体）。exec へ伝わるかは原文に無い →【推測】で扱う |
| sandbox も承認も外すのは `--dangerously-bypass-approvals-and-sandbox`（別名 `--yolo`） | \| Dangerous full access          \| `--dangerously-bypass-approvals-and-sandbox` (alias: `--yolo`) | CX-sec:367 | 【原典】ok |
| 実測：exec の sandbox の値 | `[possible values: read-only, workspace-write, danger-full-access]` | measured/codex-exec--help.txt:56 | 【実測】 |
| 実測：`--approve-for-me` は承認の要求を自動の審査へ回す | `Route approval requests through automatic review using the workspace-write sandbox` | measured/codex-exec--help.txt:59 | 【実測】 |

### 4. 会話を続ける

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| 直近を続けるのは `codex exec resume --last` | codex exec resume --last "fix the race conditions you found" | CX-ni:181 | 【原典】ok |
| ID で続けるのは `codex exec resume <SESSION_ID>` | You can also target a specific session ID with `codex exec resume <SESSION_ID>`. | CX-ni:184 | 【原典】ok |
| `--last` は現在の作業フォルダの直近を選び、`--all` で全体 | Use `--last` to pick the most recent session from the current working directory, or add `--all` to search across all sessions: | CX-ref:222 | 【原典】ok |
| ID は `thread.started` の `thread_id` から取れる | ── | CX-ni:83 と CX-ni:184 を繋いだ | 【推測】（`thread_id` と `SESSION_ID` が同じ値だと書いた行は無い） |
| 実測：SESSION_ID は UUID かスレッドの名前 | `Conversation/session id (UUID) or thread name. UUIDs take precedence if it parses. If` | measured/codex-exec-resume--help.txt:7 | 【実測】 |

### 5. 作業するフォルダとモデル

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| 作業の根は `-C, --cd <DIR>` | `-C, --cd <DIR>` / `Tell the agent to use the specified directory as its working root` | measured/codex-exec--help.txt:69-70 | 【実測】（原典の旗の一覧は読めなかった。下記） |
| モデルは `-m, --model <MODEL>` | `-m, --model <MODEL>` / `Model the agent should use` | measured/codex-exec--help.txt:40-41 | 【実測】 |
| 書けるフォルダを足すのは `--add-dir` | When you need to grant Codex write access to more directories, prefer `--add-dir` rather than forcing `--sandbox danger-full-access`. | CX-ref:365 | 【原典】ok |

### 6. 認証

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `codex exec` は既定で保存済みの CLI の認証を使う | `codex exec` reuses saved CLI authentication by default. | CX-ni:133 | 【原典】ok |
| ChatGPT でのサインイン（サブスクリプション）と API キーの2通り | - Sign in with ChatGPT for subscription access / - Sign in with an API key for usage-based access | CX-auth:19-20 | 【原典】ok |
| ログインは `~/.codex/auth.json` か OS の資格情報ストア | Codex caches login details locally in a plaintext file at `~/.codex/auth.json` or in your OS-specific credential store. | CX-auth:244 | 【原典】ok |
| 1回だけ別の API キーを使うなら `CODEX_API_KEY` | CODEX_API_KEY=<api-key> codex exec --json "triage open bug reports" | CX-ni:148 | 【原典】ok |

---

## Kiro CLI

### 1. 1回の依頼を渡して終わる呼び方

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--no-interactive` に最初の指示を渡す | Pass `--no-interactive` with your initial instruction. | KR-hl:23 | 【原典】ok |
| 呼び方の例 | kiro-cli chat --no-interactive "your prompt here" | KR-hl:27 | 【原典】ok |
| 実測：`stream-json` は `--no-interactive` を含意する | `Response stream format: text (default), or stream-json for JSON Lines on stdout.` / `stream-json implies --no-interactive` | measured/kiro-cli-chat--help.txt:63-64 | 【実測】 |

### 2. 機械が読める形の出力と、出来事

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--output-format stream-json` で、実行の出来事が stdout に JSON Lines で出る | Pass `--output-format stream-json` to receive run events as [JSON Lines](https://jsonlines.org/) on stdout. | KR-hl:140 | 【原典】ok |
| V2 か V3 のエンジンが条件 | `--output-format stream-json` requires V2 or V3 (`--agent-engine v2` or `--agent-engine v3`). | KR-hl:146 | 【原典】ok |
| V3 で中断したら最後に中断の記録が出る | In V3, an interrupted non-interactive run writes a final interruption record to `stream-json`. | KR-hl:150 | 【原典】ok |
| 実測：中身は「その実行の ACP の出来事」 | `- stream-json: The run's ACP events as JSON Lines on stdout, one self-describing event per` | measured/kiro-cli-chat--help.txt:68 | 【実測】 |
| ACP のセッションの更新は `AgentMessageChunk` ・ `ToolCall` ・ `ToolCallUpdate` ・ `TurnEnd`（`TurnEnd` は v2 だけ） | The agent sends these session update types via `session/notification`: / \| `AgentMessageChunk` \| Streaming text/content from the agent \| ほか | KR-acp:115,119-121 | 【原典】ok（エディタ向け ACP の頁）。**この4つが `stream-json` の行にそのまま出るかは原文に無い →【推測】**（help の「ACP events」と ACP の頁を繋いだだけ） |

### 3. 人に聞かずに許可を決める方法

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| 承認する人が居ないので、`--trust-all-tools` か `--trust-tools` を使う | Since there's no user to approve tool calls, V3 can use `--trust-all-tools` for session-wide approval or `--trust-tools` with exact internal tool IDs. | KR-hl:33 | 【原典】ok |
| 使い回す方針は `permissions.yaml` の規則で書く | Use scoped `permissions.yaml` rules for reusable capability policies. | KR-hl:33 | 【原典】ok |
| 規則の効果は `deny` ・ `ask` ・ `allow` | \| **Effects** \| `deny` (block always), `ask` (prompt you), `allow` (proceed silently) \| | KR-perm:27 | 【原典】ok |
| sandbox が起きないなら止めるのが `--sandbox` | Add `--sandbox` when your script must stop instead of continuing without all requested protection: | KR-hl:77 | 【原典】ok |
| 実測：手元の 2.21.1 は `--trust-tools` の例が `fs_read,fs_write`（2.x の名前）で、`--sandbox` は help に無い | `Trust only this set of tools. Example: trust some tools: '--trust-tools=fs_read,fs_write',` | measured/kiro-cli-chat--help.txt:36 | 【実測】（原典は 2.31 以降の記述を含み、手元は古い） |

### 4. 会話を続ける

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| `--resume` はこのフォルダの直前の会話 | \| `--resume` / `-r` \| Resume the previous conversation from this directory \| | KR-cmd:104 | 【原典】ok |
| `--resume-id` は ID で特定 | \| `--resume-id <ID>` \| Resume a specific session by its ID \| | KR-cmd:106 | 【原典】ok |
| モデルを渡さず再開したら元のモデルのまま | Without `--model`, a resumed session keeps its original agent and model. | KR-hl:132 | 【原典】ok |
| `--no-interactive` と `--resume-id` を一緒に使えるか | ── | ── | **読めた範囲には無い**【推測】 |
| `stream-json` の中でセッションの ID がどの項目に出るか | ── | ── | **読めた範囲には無い** |

### 5. 作業するフォルダとモデル

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| セッションはフォルダごとに保存される | - Sessions are stored per directory, so each project has its own set of sessions | KR-cmd:854 | 【原典】ok。**作業フォルダを指定する旗は読めた範囲に無い**。子のプロセスの cwd になるのは【推測】 |
| モデルは `--model <id>` で固定 | Pass `--model <id>` to pin the model for a run. | KR-hl:132 | 【原典】ok |

### 6. 認証

| 主張 | 引用 | 出どころ | 照合の結果 |
|---|---|---|---|
| ヘッドレスのモードは `KIRO_API_KEY` を要る、と headless の頁は書く | Headless mode requires an API key set as the `KIRO_API_KEY` environment variable. | KR-hl:13 | 【原典】ok |
| 一方、認証の頁の優先順位は、ブラウザでのログインが1番、`KIRO_API_KEY` が2番 | 1. Active browser session (from `kiro-cli login`) / 2. `KIRO_API_KEY` environment variable | KR-auth:107-108 | 【原典】ok |
| 上の2つから、ログイン済みの手元なら API キー無しで `--no-interactive` が動く | ── | KR-hl:13 と KR-auth:107-108 を繋いだ | 【推測】。**2つの頁の記述が食い違って読める**。手元での実行は今回していない |
| API キーは Pro ・ Pro+ ・ Pro Max ・ Power の契約者だけ | API key authentication is only available for Kiro Pro, Pro+, Pro Max, and Power subscribers. | KR-hl:15 | 【原典】ok |

---

## 読めなかった範囲 ・ 原文に無かったもの

| 対象 | 状態 |
|---|---|
| Codex の旗の一覧（`codex exec` の全オプション表、グローバルの旗の表） | `.md` の中は `<ConfigTable client:load options={execOptions} />`（CX-ref:220）などの部品の呼び出しだけで、表の中身が無い。**旗の綴りは `codex exec --help` の実測で補った** |
| `https://kiro.dev/docs/cli/reference/cli-commands` ・ `https://kiro.dev/docs/cli/authentication` | 「Moved」の6行だけ。移転先（`/docs/reference/cli-commands.md` ・ `/docs/getting-started/authentication.md`）を取得し直した |
| Kiro の `stream-json` の1行ごとの型（項目名 ・ 例） | 原文に無い |
| Kiro の作業フォルダの旗 ・ Claude Code の作業フォルダの旗 | 読めた範囲には無い |
| Claude Code の sandbox の旗 | headless ・ cli-reference には無い。`permissions.md` は取得したが照合していない |
| Codex の item の型の識別子（`command_execution` ・ `agent_message` 以外） | 読めた範囲には無い |
| 版の差 | 手元の kiro-cli 2.21.1 は、原典が書く 2.27.1 ・ 2.31 の機能（`--sandbox` など）より古い |
