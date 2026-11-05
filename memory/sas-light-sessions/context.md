# SAS のセッション管理を軽くする案：相談の前提（2026-10-31）

## 利用者の言葉（原文のまま）

- 「そもそもこのDBってリポジトリ内に持つ必要ってないと思うんだけどね。だってコミットしないし。」
- 「今回みたいにSASのDBのセッション情報をAgentsWorkspaceでも利用するような結合度の高い運用が（回るか）って話です。」
- 「ふむ、、SASでセッション管理は少し重いのかな。。なくても動くには動くのか？？せめてＳＡＳを動かすセッションＩＤさえわかってれば別に-pでおくれるもんな。。」
- それ以前に：「（SAS のセッションは）単発だけではない認識。基盤が基本的にはその親sessionを前提としている。セッションを立てるというのは別のコンテキスト内で操作をさせたいとき。基本的には-pで呼ぶにしても同じセッションIDを与えてそのsessionに送るはず。」
- 「軽くした形でアドバイザーと反証に回して」

## 決まっていること（守る）

- Semantic Agent Schema（SAS）は事業領域であって、区切られた文脈ではない（2026-10-30）。業務領域は見つけるもの、文脈は設計するもの、対応は1対1に限らない。
- SAS の文脈は2つ（コミット 9519cffa）：「スキーマとインスタンス」（schemas-and-instances/：スキーマへの適合 ・ 操作の記録 ・ 描画）と「立てたセッション」（launched-sessions/：意味の操作。ステップ6 で作る予定）。後者を切り出した根拠は「セッション」という語で同じ物を2つのモデルで持っていたこと。
- SAS の範囲（2026-10-31）：目的 ・ 目的のセッション ・ 変更の記録 ・ 承認とコミットは SAS の外（使う側の Concrete）。SAS は型（Abstract → Concrete → インスタンス）の作成 ・ 更新 ・ 削除 ・ 検証 ・ 描画 ・ 検索を担う。書くのは SAS だけ（エージェントには JSON を返させ、機械の検証と指紋の確認を通ったら SAS が書く）。
- SAS の DB はコミットしない。役目は動いているセッションを回すことで、記録は分析程度。DB のモデルは SAS の中だけ、DDL は公開しない。使う側に公開するのは ID と CLI の出力の形（schemas/v1 のスキーマ）。
- 人だけが決める欄（`x-decided-by: human`）は、人の入口（`--session` を付けない呼び出し）からしか書けない（ステップ1 で実装済み）。
- ローカル PC で動く。PR ・ コミットは SAS が担わない。
- 進め方：上からの設計は仮説。ユースケースから探索して育てる。ステップごとにシミュレーターで承認を受けてコミット。いまステップ2 まで済み、次はステップ3（SAS を通らない変更の取り込み）。セッションを立てるのはステップ6。

## いまの設計と実装（読む場所）

- 設計：`semantic-agent-schema/docs/design/schema-db/design.json` の `4_db`（DDL 全文が `ddl` にある）・ `3_schemas`（operation-request の session-spec、session スキーマ）・ `6_flows`。計画：`semantic-agent-schema/docs/design/plan/plan.json`（ステップ6）。文脈の地図：`semantic-agent-schema/docs/design/arch/arch-v15.json` の context_map。
- 実装（ステップ2 まで）：`semantic-agent-schema/schemas-and-instances/adapters/outbound/sqlite/migrations/0001〜0004*.sql`、`.../adapters/inbound/cli/src/main.rs`・`caller.rs`。
- 設計の表：
  - `sessions`（呼び出し元のセッション）：id ・ actor（ai_agent ・ human）・ adapter ・ tool_ref ・ caller_ref ・ started_at ・ last_used_at ・ ended_at ・ ended_as。開いている人のセッションは1つだけ（部分一意索引）、開いている間は caller_ref を変えられない。
  - `launched_sessions`（SAS が立てたセッション）：caller_session_id（親）・ operation_kind ・ lifetime（single ・ continuing）・ context（none ・ inherit）・ inherited_from ・ continued_from ・ capabilities ・ model ・ brief ・ ended_as（8値）。validate は single かつ none に固定（判定を独立させるため）。
  - `session_turns`：回ごとのプロンプトの経路とハッシュ ・ 再開の判断（5値）・ 結果 ・ トークン。形の誤りの直しは同じセッションで2回まで、時間切れは新しいセッションで2回まで。
  - `operations`：session_id（呼び出し元のセッション）・ launched_session_id ・ turn_seq ・ request_id ・ kind ・ source（sas ・ outside）・ target ・ before_hash ・ after_hash ・ intent など。`field_changes`・`rule_results` が操作にぶら下がる。
- 実装の `sessions` は id ・ actor（human ・ tool）・ tool_ref ・ started_at ・ ended_at だけ。人のセッションは開いているものを使い回す。
- 照合済みの事実（design.json の fact_check）：非対話で fork できるのは Claude Code と Codex だけ。セッションを閉じる ・ 状態を読む操作はどのツールにも無い。Claude Code はセッション ID を環境変数 CLAUDE_CODE_SESSION_ID で取れる。

## 検討してほしい案：軽くした形

```
今の設計                                     軽くした形
sessions（呼び出し元）                        （無し）
launched_sessions（再開の決まり ・ 8つの終わり方）（無し）
session_turns（回ごとの記録）                   （無し）
operations ── session_id                    operations ── actor ・ tool_ref ・ caller_ref
                                                          launched_ref（SAS が -p で起動したときのツールのセッション ID）
```

- SAS が持つのは「どの操作を、誰が、どのセッションで行ったか」の記録だけ。
- 続ける ・ 引き継ぐ ・ 再開は呼び出し元が決める。呼び出し元は自分のセッション ID（または前に SAS が返した launched_ref）を渡す。
- 形の誤りの直しは、SAS が同じ ID に `-p` で送り直す（表は要らない）。
- 人だけが決める欄は、操作の行の actor で守る。
- DB はリポジトリの外（例 `~/.local/share/sas/<リポジトリの識別>/sas.db`、識別は `git rev-parse --git-common-dir` の絶対パス）に置き、ワークツリーをまたいで1つにする。
- caller_ref を操作1件ごとに持つので、人が目的 A → B を続けて承認しても参照が混ざらない。

## 使う側の例：Agents Workspace

目的ごとにワークツリーを切り、AI のセッション（Claude Code など）を起動して工程を進め、工程の差分を人が承認してマージする入れ物。目的の表は Agents Workspace が持つ（セッション ID は値で）。SAS とのつなぎは、`--session`（ツールのセッション ID）と `--caller purpose/<id>`。読むのは CLI が返す公開のスキーマの出力だけ。SAS の情報が無くても、承認は Git の差分で回る。

## 書き方

日本語のふだんの言葉。比喩 ・ 新しく作った語 ・ 漢語の述部を使わない。表で短く、1欄に結論1文。識別子は中身の言葉と一緒に書く。
