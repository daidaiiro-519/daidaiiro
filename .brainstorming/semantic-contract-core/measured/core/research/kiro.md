# Kiro（kiro）：再開か新規かの契約の入力を、原典で確かめた結果

- 調べた日：2026-10-24
- 原典：kiro.dev の公式文書。各ページの Markdown 版（`https://kiro.dev/docs/<ページ>.md`）を fact-check で取得した
- 保存先：`.brainstorming/semantic-contract-core/sources/ai-tools/kiro/`
- 対象：Kiro CLI（`kiro-cli`）。Kiro の課金は IDE ・ CLI に共通のクレジット制である

## 表

| 契約の入力 | 答え | 原文の引用 | URL と行 | 照合の結果 |
|---|---|---|---|---|
| 再開 | できる。ID で再開するフラグは `--resume-id`。最新の再開は `--resume`（`-r`） | `kiro-cli chat --resume-id <SESSION_ID>` | https://kiro.dev/docs/cli/chat/session-management.md 32行 | ok |
| 再開（ID の形） | セッション ID は UUID。会話のターンごとに自動で保存される | `**Session ID**: UUID for each session` ／ `Kiro CLI automatically saves all chat sessions on every conversation turn.` | 同上 16行 ・ 9行 | ok |
| 再開（ID の取得） | 終了時に ID と再開のコマンドを表示する | `` When you exit a session, Kiro CLI prints the session ID along with the command to resume it (`kiro-cli chat --resume-id <ID>`). `` | 同上 67行 | ok |
| 再開（非対話） | できる。`--no-interactive` と `--resume-id` を併用する例が公式にある | `kiro-cli chat --v3 --no-interactive --resume-id <SESSION_ID> "Continue the task"` ／ `` \| `--resume-id <ID>` \| Resume a local or cloud session by ID \| `` | https://kiro.dev/docs/cli/headless.md 199行 ・ 216行 | ok |
| 料金の倍率（キャッシュ） | **原典に無い**。プロンプトキャッシュの有無 ・ 書くときの倍率 ・ 読むときの倍率のどれも、調べた文書に記述が無い。`cach` で検索して出るのは MCP の OAuth トークンと `__pycache__` だけである | （該当なし） | billing.md ・ models.md ・ add-on-credits.md ・ cli/chat/context.md ・ compaction.md ・ reference/slash-commands.md ・ reference/cli-commands.md ・ reference/settings.md ・ cli/headless.md を検索した | 原典に無い |
| 料金の倍率（モデル） | キャッシュの倍率の代わりに、モデルごとの「クレジットの倍率」がある。Auto を 1.0x とする相対値 | `Cost is relative to Auto (1.0x baseline).` | https://kiro.dev/docs/models.md 50行 | ok |
| 料金の倍率（入力の長さ） | 一部のモデルは入力の長さで倍率が2段になる | `GPT-5.6 requests up to 272K tokens use the listed rate; requests over 272K tokens are billed at double that rate (Sol 8.8x, Terra 4.4x, Luna 1.2x).` | 同上 52行 | ok |
| 料金の倍率（同じ倍率でも消費は違う） | 消費量は、生成したトークン数 ・ 思考の深さ ・ トークナイザで変わる | `Actual consumption depends on factors like how many tokens the model generates, internal thinking depth, and tokenizer differences.` | 同上 54行 | ok |
| 有効時間 | **原典に無い**。キャッシュの記述が無いので、TTL も無い | （該当なし） | 上と同じ範囲を検索した | 原典に無い |
| 履歴の量 | 割合（コンテキストウィンドウの何 % か）は `/context show` で取れる。トークン数そのものを出す公式のコマンド ・ ファイルは**原典に無い** | `Current context window (5.9% used)` | https://kiro.dev/docs/cli/chat/context.md 166行 | ok |
| 履歴の量（値が決まる時点） | 割合は次のプロンプトを送ったときにバックエンドが決める | `The accurate usage is only calculated when you send your next message, because the backend determines actual token counts during request processing.` | 同上 157行 | ok |
| 履歴の量（隠れたコマンド） | `/stats` はリクエストごとの入力 ・ 出力トークン数を出し、JSON へ保存できる。ただしこの節は MDX のコメント `{/* … */}` の中にあり、公開ページには表示されない。直近100件までで、セッションが終わると消える | `Displays an in-memory log of recent API requests with their request IDs, total duration, time-to-first-chunk (TTFC), input and output token counts, and error status.` ／ `Stats are kept in an in-memory ring buffer of the last 100 requests and reset when the session ends.` | https://kiro.dev/docs/reference/slash-commands.md 734行 ・ 739行（コメントの範囲は 716〜741行） | ok |
| 履歴の量（非対話の出力） | `--output-format stream-json` で JSON Lines を出す。各イベントの項目（トークン数を含むか）は**原典に無い** | `` Pass `--output-format stream-json` to receive run events as [JSON Lines](https://jsonlines.org/) on stdout. `` | https://kiro.dev/docs/cli/headless.md 150行 | ok |
| ターン数 | **原典に無い**。セッション一覧を「メッセージ数」で並べ替えられる記述だけがある | `` Sort by `last used`, `session name`, or `messages` `` | https://kiro.dev/docs/cli/chat/session-management.md 93行 | ok |
| 料金の形 | 定額の月額プラン＋クレジット。クレジットはリクエストごとに端数で消費する。トークン課金ではない | `Credits are consumed fractionally based on each request. Simple edits and shorter prompts will use fewer credits than complex, lengthy tasks.` ／ `` \| Pro \| 1,000 \| Available to purchase \| `` | https://kiro.dev/docs/billing.md 13行 ・ 20行 | ok |
| 料金の形（超過分） | 有料プランは追加クレジットを 1クレジット $0.04 で買える | `They're available on all paid tiers (Pro, Pro+, Pro Max, and Power) at $0.04 per credit, billed immediately when you purchase them.` | https://kiro.dev/docs/billing/add-on-credits.md 9行 | ok |
| 料金の形（残高） | `/usage` が使用量と残りクレジットを出す。非対話では内訳を出す | ``View your current usage statistics and remaining credits. In non-interactive runs, `/usage` prints the full usage breakdown instead of a one-line summary.`` | https://kiro.dev/docs/reference/slash-commands.md 714行 | ok |

識別子の照合（`--as identifier`、https://kiro.dev/docs/reference/cli-commands.md）：`--resume-id`（106行）・ `--no-interactive`（103行）・ `--output-format`（117行）・ `--resume`（104行）・ `--list-sessions`（109行）はすべて ok。

## 所見

| 契約の入力 | この契約の形で表せるか | 足りないもの |
|---|---|---|
| 再開 | 表せる。`--resume-id <UUID>` を非対話でも使える | なし |
| 料金の倍率 | **キャッシュの倍率としては表せない**。Kiro はキャッシュを利用者に見せず、利用者が払うのはクレジットである | 下の「クレジット制での料金の倍率」を参照 |
| 有効時間 | 表せない。値が公開されていない | 契約側で「保証が無い」を表す値が必要である |
| 履歴の量 | 割合（%）なら表せる。トークン数は公式には取れない | 契約の入力を「トークン数」に固定すると Kiro は返せない。「コンテキストウィンドウに対する割合」も受けられる形が必要である |
| ターン数 | 表せない。公式の取り方が無い | アダプター側で数える（stream-json のイベントを数える等。推測：イベントの項目が原典に無いので、数え方は実測で確かめる必要がある） |
| 料金の形 | 表せる。「定額＋クレジット（リクエスト単位で端数消費）」 | 契約の値に「クレジット制」を区別する種類が必要である |

### クレジット制での料金の倍率

- Kiro の利用者が払う額は、クレジットの消費量で決まる。消費量は「モデルの倍率 × 生成したトークン数 ・ 思考の深さ ・ トークナイザ」で変わる（models.md 50〜54行）。キャッシュに当たったかどうかがクレジットに反映されるかは**原典に無い**。
- そのため、契約の「キャッシュに書くときの倍率 ・ 読むときの倍率」は、Kiro では「不明」としか返せない。再開すれば入力が安くなる、という前提を Kiro では置けない。
- 推測：契約で比べられる量は「再開したときの1ターンのクレジット消費」と「新規で引き継いだときの1ターンのクレジット消費」の実測値である。`/usage` の非対話の出力を、ターンの前後で読んで差を取れば測れる見込みがある。ただし `/usage` の出力の形式は原典に無いので、実測で確かめる。
- 定額の範囲内（月のクレジットを使い切る前）では、追加の支払いは発生しない。超過後は 1クレジット $0.04（add-on-credits.md 9行）なので、超過後に限り「クレジット × $0.04」で金額に直せる。
- 履歴の量の上限は、自動の compaction（要約）で扱われる。compaction で新しいセッションが作られる場合がある（cli/chat/context.md 155行）。再開か新規かの判断は、Kiro 自身の compaction と重なる。
