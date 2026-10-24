# Gemini CLI（gemini）：再開か新規かの契約の入力を、原典で確かめた結果

- 調べた日：2026-10-24
- 原典
  - キャッシュと料金：Google の公式文書（ai.google.dev）。Markdown 版（`.md.txt`）を fact-check で取得した
  - セッションの再開：google-gemini/gemini-cli リポジトリの docs と packages。**読んだ commit は `cd1b45c2db0a5ea7b7fc3b67c1ba5879601d6005`（2026-10-23T20:43:47Z の main）**。raw.githubusercontent.com から commit を固定して取得した
- 保存先：`.brainstorming/semantic-contract-core/sources/ai-tools/gemini/`
- 表の「URL と行」で `<repo>` は `https://raw.githubusercontent.com/google-gemini/gemini-cli/cd1b45c2db0a5ea7b7fc3b67c1ba5879601d6005` を指す

## 表

| 契約の入力 | 答え | 原文の引用 | URL と行 | 照合の結果 |
|---|---|---|---|---|
| 再開 | できる。`--resume`（`-r`）に UUID ・ 一覧の番号 ・ `"latest"` を渡す | `gemini --resume a1b2c3d4-e5f6-7890-abcd-ef1234567890` | `<repo>/docs/cli/session-management.md` 52行 | ok |
| 再開（非対話） | できる。`-r` に ID とプロンプトを続けて渡す。`-p`（`--prompt`）は非対話を強制する | `` \| `gemini -r "<session-id>" "query"` \| Resume session by ID               \| `gemini -r "abc123" "Finish this PR"`                        \| `` ／ `Prompt text. Appended to stdin input if provided. Forces non-interactive mode.` | `<repo>/docs/cli/cli-reference.md` 17行 ・ 52行 | ok |
| 再開（ID の取得） | stream-json の `init` イベントがセッション ID を持つ | `` `init`: Session metadata (session ID, model). `` | `<repo>/docs/cli/headless.md` 29行 | ok |
| 料金の倍率（キャッシュの種類） | Gemini API は2種類。implicit は自動で、割引の保証が無い。explicit は手動で作り、割引の保証がある。explicit は Gemini 4 Argon 以降では使えない | `Implicit caching (automatically enabled on Gemini 2.5 and newer models, no cost saving guarantee)` ／ `Explicit caching (can be manually enabled on Gemini 3 and 2.5 models, cost saving guarantee; not supported for Gemini 4 Argon and onward)` | https://ai.google.dev/gemini-api/docs/generate-content/caching.md.txt 4行 ・ 5行 | ok |
| 料金の倍率（読むとき：Gemini 4 Argon） | 入力 $2.00 に対しキャッシュ $0.10 ＝ **0.05 倍**（12月以降も $4.00 ／ $0.20 で 0.05 倍）。implicit のみ | `` \| Input price \| Not available \| $2.00 through November 30, 2026. $4.00 starting December 1, 2026. \| `` ／ `` \| Context caching price \| Not available \| $0.10 through November 30, 2026. $0.20 starting December 1, 2026. Implicit caching only (explicit caching is not supported). \| `` | https://ai.google.dev/gemini-api/docs/pricing.md.txt 56行 ・ 58行 | ok（倍率は引用の値からの計算） |
| 料金の倍率（読むとき：Gemini 3.1 Pro Preview） | 入力 $2.00 に対しキャッシュ $0.20 ＝ **0.1 倍**（200k 超は $4.00 ／ $0.40 で 0.1 倍）。explicit は保存料 $4.50 ／ 100万トークン ・ 1時間 が別にかかる | `` \| Input price \| Not available \| $2.00, prompts \<= 200k tokens $4.00, prompts \> 200k tokens \| `` ／ `` \| Context caching price \| Not available \| $0.20, prompts \<= 200k tokens $0.40, prompts \> 200k $4.50 / 1,000,000 tokens per hour (storage price) \| `` | 同上 484行 ・ 486行 | ok（倍率は引用の値からの計算） |
| 料金の倍率（読むとき：Gemini 3.8 Flash） | 入力 $0.75 に対しキャッシュ $0.075 ＝ **0.1 倍** | `` \| Input price \| Free of charge \| $0.75 through December 31, 2026. $1.50 starting January 1, 2027. \| `` | 同上 121行（キャッシュは 123行） | ok |
| 料金の倍率（書くとき） | **原典に無い**。implicit も explicit も、キャッシュに書くときの割増の記述が無い。explicit の課金要素は「キャッシュしたトークン数（後のプロンプトで割引）」「保存の時間（TTL）」「その他」の3つ | `**Cache token count:** The number of input tokens cached, billed at a reduced rate when included in subsequent prompts.` | https://ai.google.dev/gemini-api/docs/generate-content/caching.md.txt 616行 | ok |
| 料金の倍率（Gemini CLI での適用） | Gemini CLI がキャッシュを使うのは API キー（Gemini API ・ Vertex AI）で認証したときだけ。Google アカウントの OAuth（Code Assist API）では使えない | `Gemini CLI automatically optimizes API costs through token caching when using` ／ `API key authentication (Gemini API key or Vertex AI).` ／ `OAuth users (Google Personal/Enterprise accounts) - the Code Assist API does` ／ `not support cached content creation at this time` | `<repo>/docs/cli/token-caching.md` 3〜4行 ・ 15〜16行 | ok |
| 料金の倍率（Gemini CLI の実装） | commit `cd1b45c` の packages に `caches.create` ・ `createCachedContent` の呼び出しは無い（grep の結果）。推測：Gemini CLI は explicit を作らず、implicit に頼っている | （コードの検索の結果。引用なし） | `<repo>/packages/`（git clone して grep） | 検索した |
| 有効時間（implicit） | **原典に無い**。TTL の値は書かれていない。「似た接頭辞のリクエストを短い時間のうちに送る」とだけある | `Try to send requests with similar prefix in a short amount of time` | https://ai.google.dev/gemini-api/docs/generate-content/caching.md.txt 27行 | ok |
| 有効時間（explicit） | 既定は1時間。上限も下限も無い。利用者が決める | `If not set, the TTL defaults to 1 hour.` ／ `There are no minimum or maximum bounds on the TTL.` | 同上 47行 ・ 617行 | ok |
| 有効時間（Gemini 4 の複数ターン） | 前のリクエスト全体がキャッシュされるのは、新しいリクエストが 2,050 トークン以上を足したときだけ | `Gemini caches the full previous request only if the new request adds at least 2,050 tokens.` | 同上 30行 | ok |
| 履歴の量 | 取れる。セッションのファイルにメッセージごとのトークン数（input ・ output ・ cached など）が残る。保存先は `~/.gemini/tmp/<project_hash>/chats/` | `Token usage statistics (input, output, cached, etc.).` ／ ``**Location:** Sessions are stored in `~/.gemini/tmp/<project_hash>/chats/`,`` | `<repo>/docs/cli/session-management.md` 16行 ・ 18行 | ok |
| 履歴の量（ファイルの項目名） | `TokensSummary` の `input` は `promptTokenCount`、`cached` は `cachedContentTokenCount` | `input: number; // promptTokenCount` ／ `cached: number; // cachedContentTokenCount` | `<repo>/packages/core/src/services/chatRecordingTypes.ts` 20行 ・ 22行 | ok |
| 履歴の量（非対話の出力） | `--output-format json` の `stats`、`stream-json` の `result` がトークン数を持つ。`StreamStats` は `input_tokens` の内訳として `cached` と `input` を持つ | `` `stats`: (object) Token usage and API latency metrics. `` ／ `` `result`: Final outcome with aggregated statistics and per-model token usage `` ／ `// Breakdown of input_tokens` | `<repo>/docs/cli/headless.md` 21行 ・ 34行 ／ `<repo>/packages/core/src/output/types.ts` 93行 | ok |
| ターン数 | 上限の設定 `model.maxSessionTurns` がある。非対話で上限に達すると終了コード 53 で終わる。ターン数そのものを出力する項目は**原典に無い**。実装の数は `sessionTurnCount` でメモリ上にだけある | ``**`maxSessionTurns`**: (number) The maximum number of turns (user and model`` ／ ``**Non-interactive mode:** The CLI exits with an error.`` ／ `` `53`: Turn limit exceeded. `` ／ `private sessionTurnCount = 0;` | `<repo>/docs/cli/session-management.md` 201行 ・ 208行 ／ `<repo>/docs/cli/headless.md` 45行 ／ `<repo>/packages/core/src/core/client.ts` 95行 | ok |
| 料金の形 | 認証の方法で変わる。Google アカウントは定額（または無料）でリクエスト数の上限、API キーはトークン課金 | `` \| **Google account**    \| Gemini Code Assist (Individual) \| 1,000 requests                    \| `` ／ `Quotas and pricing are based on a fixed price subscription.` ／ `It’s important to highlight that when using an API key, you pay per token/call.` | `<repo>/docs/resources/quota-and-pricing.md` 19行 ・ 102行 ・ 175行 | ok |

識別子の照合（`--as identifier`）：`<repo>/docs/cli/cli-reference.md` で `--resume`（65行）・ `--prompt`（52行）・ `--output-format`（70行）・ `--list-sessions`（66行）、`<repo>/docs/cli/session-management.md` で `maxSessionTurns`（196行）・ `sessionRetention`（169行）はすべて ok。

## 所見

| 契約の入力 | この契約の形で表せるか | 足りないもの |
|---|---|---|
| 再開 | 表せる。`-r <UUID>` を `-p` と併用できる | なし |
| 料金の倍率 | **API キーで認証したときだけ表せる**。読むときの倍率はモデルごとに 0.05 倍（Gemini 4 Argon）または 0.1 倍（調べたほかのモデル）。書くときの倍率は原典に無いので「割増なし」とは書けず「不明」になる | 倍率を「認証の方法 × モデル」で引く表が必要である。OAuth のときは「キャッシュなし」を返す値が必要である |
| 有効時間 | implicit は表せない（値が無い）。explicit は表せる（既定1時間、利用者が決める）が、Gemini CLI は explicit を作らない（推測、上の grep の結果による） | 契約に「保証が無い」を表す値が必要である。implicit の割引は保証されない点も同じ値で表す |
| 履歴の量 | 表せる。セッションのファイルの最後の `gemini` メッセージの `tokens`、または非対話の `stats` ・ `result` から取れる | なし |
| ターン数 | 直接は表せない。上限の設定と超過の終了コードだけが公式にある | アダプター側で、セッションのファイルの `user` の種類のメッセージを数える（推測：この数え方が CLI の `sessionTurnCount` と一致するかは実測で確かめる） |
| 料金の形 | 表せるが、値が2種類に分かれる | 契約の値に「リクエスト数の上限つき定額」と「トークン課金」の区別が必要である |

### 定額（Google アカウント）の場合の料金の倍率

- Google アカウントで認証すると、課金は「1日あたりのリクエスト数の上限」で決まる（quota-and-pricing.md 19〜28行）。このときキャッシュは作られない（token-caching.md 15〜16行）。
- このとき、契約の「キャッシュの倍率」は意味を持たない。1回のリクエストの費用は入力の長さに依らず「上限に対する1回分」である。
- 推測：再開か新規かの比べ方は、トークンの金額ではなく「残りのリクエスト数」と「コンテキストウィンドウに対する履歴の量（応答の質と上限への接近）」になる。再開しても新規にしても、1ターンが消費するリクエスト数は同じ見込みである（ツールの呼び出しでリクエストが増える点は両方に共通）。
- API キー（トークン課金）のときは、契約の形がそのまま当てはまる。ただし implicit は割引の保証が無いので、倍率は「当たった場合の値」であり、当たる確率は実測（`stats` の `cached` ÷ `input_tokens`）で求める。
