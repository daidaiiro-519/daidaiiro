# GitHub Copilot CLI（copilot）：再開か新規かの契約の入力を、原典で確かめた結果

取得日は 2026-10-24。原文は `.brainstorming/semantic-contract-core/sources/ai-tools/copilot/` に保存した。docs.github.com は `https://docs.github.com/api/article/body?pathname=<パス>` で Markdown の原文を取得した。github/copilot-cli は main を取得した（取得時の main の先頭 commit は `7ea98dfc33c68aaf6443ea8fc054e20ff7fcd120`、2026-10-22。GitHub API で確認）。

## 原典

| 記号 | 文書 | パス |
|---|---|---|
| CMD | GitHub Copilot CLI command reference | `/en/copilot/reference/cli-command-reference` |
| DIR | GitHub Copilot CLI configuration directory | `/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference` |
| PRICE | Models and pricing for GitHub Copilot | `/en/copilot/reference/copilot-billing/models-and-pricing` |
| OPT | Optimizing your AI usage | `/en/copilot/tutorials/optimize-ai-usage` |
| CHG | What changed with Copilot billing (legacy) | `/en/copilot/reference/copilot-billing/request-based-billing-legacy/what-changed-with-billing` |
| REQ | Requests in GitHub Copilot (legacy) | `/en/copilot/concepts/billing/copilot-requests` |
| MUL | Model multipliers for annual plans (legacy) | `/en/copilot/reference/copilot-billing/request-based-billing-legacy/model-multipliers-for-annual-plans` |
| LOG | github/copilot-cli changelog.md | `https://raw.githubusercontent.com/github/copilot-cli/main/changelog.md`（commit 7ea98dfc） |

## 表

| 契約の入力 | 答え | 原文の引用 | URL と行 | 照合の結果 |
|---|---|---|---|---|
| 再開 | できる。`--resume=<ID・ID の接頭辞・名前>`、完全一致の `--session-id <ID>`、直近を開く `--continue`。対話中は `/resume [SESSION-ID]` | "Optionally specify a session ID, ID prefix, or session name." / "If the ID matches an existing session or task, that session or task is resumed." / "Resume the most recent session in the current working directory, falling back to the globally most recent session." | CMD:718 ・ 722 ・ 676 | quote ok ×3、identifier `--resume` `--session-id` `--continue` ok |
| 再開（非対話） | できる。`-p` は終了時に再開用の ID を出す。非対話では選択画面を出せないので、ID を明示する | "Execute a prompt programmatically (exits after completion). The exit summary includes a `copilot --resume=SESSION-ID` hint for continuing the session." / "pass an explicit `--resume=SESSION-ID` or use `--continue`." | CMD:712 ・ 718 | quote ok ×2 |
| 料金の形 | 2026-06-01 からトークン課金（GitHub AI Credits、1 credit = $0.01）。premium requests はその日以前の課金で、年額の Pro ・ Pro+ を続ける人だけに残る | "As of June 1, 2026, GitHub replaced request-based billing with usage-based billing, where the cost of an interaction depends on two things: the **model** and the **number of tokens consumed**." / "This article only applies to Copilot Pro and Copilot Pro+ subscribers on an existing annual plan who remained on legacy premium request-based billing after June 1, 2026." | CHG:13 ・ REQ:5 | quote ok ×2 |
| 料金の倍率（トークン課金） | キャッシュは在る。モデルごとに Input ・ Cached input ・ Cache write の単価表が在り、倍率は表から割り算で出す。例：Claude Sonnet 4.6 は書く 3.75/3.00＝1.25 倍、読む 0.30/3.00＝0.1 倍。Claude Opus 5.5 は書く 1.25 倍、読む 0.05 倍。GPT-5.3-Codex など古い OpenAI のモデルは書く費用が無い。Google ・ xAI ・ Microsoft ・ Moonshot の表には Cache write の列が無い | "Anthropic models include a cache write cost in addition to cached input." / "Earlier OpenAI models have no cache write cost." / "\| Claude Sonnet 4.6 ... \|  $3.00 \|        $0.30 \|       $3.75 \| $15.00 \|" / "cached tokens are typically billed at 10% of the normal input price." | PRICE:70 ・ 28 ・ 83 ・ 89、OPT:105 | quote ok ×5 |
| 料金の倍率（premium requests） | トークン量に依らず、プロンプト1回が 1 premium request × モデルの倍率。ツール呼び出しは数えない。キャッシュの倍率という考えは無い | "Each prompt to Copilot CLI uses **one premium request** with the default model. For other models, this is multiplied by the model's rate." / "For agentic features, only the prompts you send count as premium requests; actions Copilot takes autonomously to complete your task, such as tool calls, do not." | REQ:27 ・ 9 | quote ok ×2 |
| 有効時間 | 無操作で切れる。OpenAI のモデルは 24 時間、ほかの多くは 1 時間。手引きの文書（tutorial）の記述で、モデルごとの一覧や保証は原典に無い | "Caches expire after a period of inactivity (24 hours for OpenAI models and 1 hour for most others)." | OPT:110 | quote ok |
| 履歴の量 | 取れる。対話では `/usage` ・ `/context`。非対話では OTel のファイル出力（`COPILOT_OTEL_FILE_EXPORTER_PATH`）の `chat` span ごとに `gen_ai.usage.input_tokens` ・ `gen_ai.usage.cache_read.input_tokens` ・ `gen_ai.usage.cache_creation.input_tokens` | "Display session usage metrics and statistics, including per-model token totals." / "Show the context window token usage and visualization." / "Write all signals to this file as JSON-lines. Setting this automatically enables OTel." / "Input tokens this turn" / "Cached tokens read" / "Cached input tokens created" | CMD:614 ・ 542 ・ 1748 ・ 1836 ・ 1835 ・ 1807 | quote ok ×6、identifier ok ×6 |
| 履歴の量（ファイル） | セッションごとに `events.jsonl` が在り、終了時に使用量が書かれる。ただし events.jsonl の項目の形は原典に無い。changelog に `--usage-output-file` が在るが、CMD には無い | "Each session directory stores an event log (`events.jsonl`) ... This data enables session resume" / "- Session usage metrics (requests, tokens, code changes) are now persisted to events.jsonl after each session ends" / "- Add per-agent usage metrics to --usage-output-file JSON output" | DIR:320、LOG:2553 ・ 547 | quote ok ×3。`--usage-output-file` は CMD で identifier 照合が不一致（CMD に無い） |
| ターン数 | 一部だけ取れる。OTel の `github.copilot.turn_count` はモデルへの往復の回数で、利用者のプロンプトの回数ではない。利用者のプロンプトの回数を返す項目は原典に無い | "Number of LLM round-trips" | CMD:1808 | quote ok、identifier ok |
| 請求の単位（参考） | OTel が1回ごとの課金の値を返す。`github.copilot.cost` はモデルの倍率（金額ではない）、`github.copilot.nano_aiu` は消費した AI units | "Per-request model multiplier used for billing. **Not a monetary value.**" / "AI units consumed this turn, in nano AI units (1 AIU = 1,000,000,000 nano AIU)" | CMD:1809 ・ 1838 ・ 1839 | quote ok ×2、identifier ok ×2 |

照合の合計は quote 33 件（所見の `/usage` の引用を含む） ・ identifier 14 件が一致、identifier 1 件（`--usage-output-file` を CMD で）が不一致。

## 所見

| 問い | 答え |
|---|---|
| この契約の形で表せるか | トークン課金の利用者は表せる。再開 ・ 料金の倍率 ・ 有効時間 ・ 履歴の量の4つは原典で埋まる |
| 足りないもの1：倍率はモデルごと | 倍率はツールで1つに決まらない。アダプターは、使っているモデルを `PRICE` の単価表に引いて、書く倍率 ＝ Cache write ÷ Input、読む倍率 ＝ Cached input ÷ Input を返す。Cache write の列が無いモデルは「書く費用の記載が無い」と返す（1.0 として扱うのは推測） |
| 足りないもの2：有効時間は保証が無い | 「24 時間 ・ 1 時間」は tutorial の記述で、"most others" のどれが外れるかは原典に無い。契約には値と「保証が無い」の印を両方持たせる |
| 足りないもの3：ターン数 | 利用者のプロンプトの回数は原典に無い。取れるのはモデルへの往復の回数（`turn_count`）で、意味が違う。契約がターン数を必要とするなら、OTel の `invoke_agent` span（利用者のメッセージ1件につき1つ）の数を数える（推測：span の数え方を原典は契約として書いていない） |
| 足りないもの4：非対話で使用量を読むには OTel を必要とする | `-p` の JSONL 出力に使用量の項目が在るとは原典に無い。確実に取れるのは OTel のファイル出力 |
| premium requests の利用者で「料金の倍率」をどう表すか | 1プロンプトの費用 ＝ 1 × モデルの倍率で、入力トークンの量に依らない。キャッシュに書く ・ 読むの区別も無い。よって「書く倍率 ・ 読む倍率」は「該当しない」と返し、代わりに「1プロンプトあたりの倍率」（`MUL` の表、OTel の `github.copilot.cost`）を返す。この場合、履歴が長くても再開の費用は増えず、有効時間も費用に効かない。再開か新規かの分かれ目は費用ではなく、文脈の上限と品質で決まる。新しく立てて引き継ぐと、引き継ぎのプロンプトの分だけ premium request が増える（推測：引き継ぎを1プロンプトで行う前提） |
| 契約に足すもの | 「料金の形」を値の種類（トークン課金 ／ リクエスト課金）として先に持ち、種類ごとに倍率の欄の意味を分ける。同じ copilot でも利用者の契約で形が変わるので、アダプターはツール名だけでは決められない。どちらの課金かを CLI から取る方法は原典に無い（`/usage` は "For token-based-billing accounts" で表示が変わるとだけ書かれている） |
