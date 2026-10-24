# Codex CLI（codex）：再開か新規かの契約の入力

- 原文の置き場所：`.brainstorming/semantic-contract-core/sources/ai-tools/codex/`（2026-10-24 取得、すべて text/markdown の原文）
- 読んだリポジトリ：openai/codex commit `970111f6cb62cc02fe36a82ea7d71b37a2889c97`（2026-10-23）
- 照合：下の引用はすべて `fact-check verify` で ok（文書は 32 件、リポジトリは 10 件。読めた範囲に無い 0）
- 略号：REF＝developers.openai.com/codex/cli/reference ／ NI＝/codex/noninteractive ／ CP＝/codex/pricing ／ AUTH＝/codex/auth ／ PC＝/api/docs/guides/prompt-caching ／ AP＝/api/docs/pricing

## 表

| 契約の入力 | 答え | 原文の引用 | URL と行 | 照合 |
|---|---|---|---|---|
| 再開（対話） | できる。`codex resume <SESSION_ID>`、直近は `--last`（作業ディレクトリ内）、`--all` で全体 | "Continue an interactive session by ID or resume the most recent chat." | REF:317 | ok |
| 再開（非対話） | できる。`codex exec resume <SESSION_ID>` ／ `codex exec resume --last "<prompt>"` | "You can also target a specific session ID with `codex exec resume <SESSION_ID>`." | NI:184 | ok |
| 新しく立てて引き継ぐ | 保存済みのセッションを複製して新しいセッションにできる。`codex exec fork <SESSION_ID> [PROMPT]` | `codex exec fork <SESSION_ID> [PROMPT]`（identifier） | REF:226 | ok |
| 料金の倍率（API キー） | GPT-5.6 以降：書く 1.25×（30m）／ 2.5×（12h）、読む 0.1×（GPT-6.1 Sol は 0.05×）。GPT-5.5 以前：書く追加料金なし、読むはモデルごと | "For GPT-5.6 and later, cache writes cost 1.25× the standard, uncached input-token rate for the default 30-minute lifetime." ／ "Subsequent reads cost 0.1× the ordinary input rate on most of these models and 0.05× on [GPT-6.1 Sol]…" ／ 料金表 "\| gpt-6.1-sol \| $2.00 \| $10.00 \| $0.10 \| $2.50 \| $5.00 …"（入力 $2.00、キャッシュ入力 $0.10、30m 書き込み $2.50） | PC:83、PC:310、AP:39 | ok |
| 料金の倍率（ChatGPT のプラン） | 書く倍率は無い（1×）。読むはクレジット表で入力の 0.1×（GPT-6.1 Sol は 50→2.5 で 0.05×）。ただし定額の枠内では倍率は請求額に直結しない | "Codex credit billing has no separate cache-write charge. API-key usage follows" | CP:392、CP:424-432 | ok |
| 有効時間 | GPT-5.6 以降：最小 30 分（既定）または 12 時間、読むたびに 30 分延長。上限は保証なし。GPT-5.5 以前：目安のみ（保証なし） | "\| Cache lifetime \| Minimum lifetime of 30 minutes or 12 hours; … \| Typically around 30 minutes, up to 24 hours \| Typically 5 to 10 minutes inactive for `in_memory`, or up to 24 hours for `24h` \|" ／ "A 30-minute read refreshes coverage to at least 30 minutes after that read." ／ "OpenAI may retain entries longer than the requested minimum; `"12h"` does not set a deletion deadline" | PC:313、PC:222、PC:161 | ok |
| 有効時間（Codex が選ぶ値） | Codex CLI はリクエストに `prompt_cache_key` を付けるが、`prompt_cache_options`（ttl）の項目を持たない → 既定の 30m が当たる（**推測**：リクエストの構造体に項目が無いことからの推論） | `pub prompt_cache_key: Option<String>,` | codex-rs/codex-api/src/common.rs:297 | ok |
| 当たる保証 | 無い | "Keys influence routing; they do not pin requests to a machine or guarantee a cache hit." | PC:288 | ok |
| 履歴の量 | 取れる。① `codex exec --json` の `turn.completed` の `usage`（`input_tokens` ・ `cached_input_tokens` ・ `output_tokens` ・ `reasoning_output_tokens`、リポジトリでは `cache_write_input_tokens` も出す）② 対話では `/status` ③ ロールアウトファイル `~/.codex/sessions/…/rollout-*.jsonl` の `TokenCount` イベント（`total_token_usage` ・ `last_token_usage` ・ `model_context_window`） | `{"type":"turn.completed","usage":{"input_tokens":24763,"cached_input_tokens":24448,"output_tokens":122,"reasoning_output_tokens":0}}` ／ "Display session configuration and token usage." ／ `pub total_token_usage: TokenUsage,` ／ `pub last_token_usage: TokenUsage,` ／ `pub model_context_window: Option<i64>,` | NI:87、REF:452、codex-rs/protocol/src/protocol.rs:2429-2433 | ok |
| 履歴の量（注意） | `turn.completed` の `usage` は、型の説明は「そのターン」だが、実装はスレッドの累計（`usage.total`）を入れている。いまの入力の大きさには `last_token_usage`（ロールアウト）を使う必要がある | `cached_input_tokens: usage.total.cached_input_tokens,` ／ `cache_write_input_tokens: usage.total.cache_write_input_tokens,` | codex-rs/exec/src/event_processor_with_jsonl_output.rs:124-125 | ok |
| ターン数 | 公式文書に「ターン数を返す」記述は原典に無い。数えることはできる：`--json` の `turn.started` ／ `turn.completed` の行数、またはロールアウトに残る `TurnStarted` ・ `TurnComplete` の数 | "Event types include `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`." ／ `EventMsg::TokenCount(_)` `\| EventMsg::TurnStarted(_)` `\| EventMsg::TurnComplete(_)`（永続化するイベント） | NI:76、codex-rs/rollout/src/policy.rs:133-138 | ok |
| ロールアウトの場所 | `$CODEX_HOME/sessions`（`--ephemeral` なら書かない＝再開できない） | `pub const SESSIONS_SUBDIR: &str = "sessions";` ／ "Use `--ephemeral` when you don't want to persist session rollout files to disk:" | codex-rs/rollout/src/lib.rs:86、NI:33 | ok |
| 料金の形 | 2通り。ChatGPT サインイン＝定額の枠（5時間ごと・週ごとの目安、固定値ではない）＋超過分はクレジット。API キー＝API 料金のトークン課金 | "- Sign in with ChatGPT for subscription access" ／ "- Sign in with an API key for usage-based access" ／ "- Pay for Codex usage based on [API pricing](…)" ／ "The estimates below show local messages per five-hour period for Plus and" ／ "These estimates are not fixed message limits; check your" ／ "API token prices are separate from subscription usage; don't use them to" | AUTH:19-20、CP:109、CP:227、CP:230、CP:15 | ok |
| 残りの枠 | ChatGPT のプランでは `/status` で残りの枠を見られる（対話のみ。非対話で取る方法は原典に無い） | "remaining limits during an active Codex CLI session, you can use `/status`." | CP:369 | ok |

## 所見

| 項目 | 所見 |
|---|---|
| 契約の形で表せるか | API キーで使う場合は表せる。再開（`exec resume`）・新規へ複製（`exec fork`）・書く倍率 ・ 読む倍率 ・ 最小の有効時間 ・ 入力トークン数 ・ ターン数（数える）がすべて取れる |
| 倍率はモデルで変わる | 同じ Codex でも GPT-5.6 以降（書く 1.25×、読む 0.1× か 0.05×、最小 30 分）と GPT-5.5 以前（書く追加なし、有効時間は目安）で値の形が違う。アダプターはモデル名から倍率を引く表を持つ必要がある |
| ChatGPT のプランでは足りない | 書く倍率は無く（クレジットに書き込み料金なし）、定額の枠内では請求額が変わらない。比べる量は「枠の消費」になるが、枠の大きさは「固定値ではない」と原典が書き、非対話で残りを取る方法は原典に無い。契約に「料金の形＝定額の枠」と「枠の消費量を費用とみなすか」の値が要る |
| 有効時間の扱い | 保証されるのは最小値（30 分）だけで、当たる保証は無い。契約の有効時間は「最小値」として持ち、上限は持たない形にする必要がある |
| 履歴の量の取り方 | `turn.completed.usage` は累計なので、再開時の入力の大きさには使えない。ロールアウトの `last_token_usage.input_tokens` を読むのが正しい（ファイル形式は公式文書に無く、リポジトリの実装に依る＝版で変わりうる） |
| 足りないもの | ① ターン数を直接返す公式の値 ② 非対話で枠の残りを取る方法 ③ ロールアウトの形式の公式の文書。どれも原典に無い |
