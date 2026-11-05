---
name: sas-light-sessions-opinion
description: SAS のセッション管理を軽くする案への1つの意見（Agents Workspace のセッションから SAS のセッションへ渡す。未承認）。2026-10-31
metadata:
  type: project
---

**2026-11-01 の結果：** SAS のセッションがこの意見を受けて最終の形にした（未適用）。ページ https://claude.ai/artifact/XQmeBmywzzsZy3AUgcSK4B 、本物の ID で動かした試作 https://claude.ai/artifact/VYeEarjweXihDbJTNxnGhW 。SAS は子の状態を管理せず、薄い sessions（id＝ツールのセッション ID ・ role caller/writer/checker ・ parent_id ・ tool ・ model ・ launch ・ first_seen、追記だけ）に親も子も入れる。記録は changes と validations に分け、断った操作は持たない。親が返った子の ID を --continue で渡す。書いた子では判定しない。caller_ref ・ --caller は外れた。DB は <git-common-dir>/sas/sas.db。文脈は1つに戻す。下の意見との違い：セッションの表は無くさず薄く残した、やり直しは毎回起動ではなく続ける子（2回目から約 $0.009 対 毎回 $0.198）。

**扱い（当時）：** Agents Workspace の話から出た、SAS を作り直しているセッションへの1つの意見。利用者はまだ承認していない。採るかどうかは SAS の側で、利用者の言葉と照らして決める。材料は `memory/sas-light-sessions/`（context.md ・ ddd-answer.json ・ platform-answer.json ・ review.json）。

## 結論
**SAS はセッションを管理しない。** セッション ID は操作1件ごとに値として記録するだけにし、開く ・ 閉じる ・ 続ける ・ 再開するの決まりと、そのための3つの表（sessions ・ launched_sessions ・ session_turns）をやめる。続けるか再開するかは呼び出し元が決める。

## 根拠（原典の判断基準。引用は memory/sas-light-sessions/ の answer.json）
| 理由 | 判断基準 |
|---|---|
| セッションは SAS の補完の業務領域なのに、寿命 ・ 再開 ・ 8つの終わり方のドメインモデルを持っている。これは要件を複雑にしている兆候で、対応は要件を単純にすること | ddd：subdomain「業務担当者は補完と考えるが、実装にはドメインモデル…を必要とする ・ 可能性：…要件を複雑にしている（よくある） ・ 対応：要件を単純にする」 |
| 呼び出し元のセッションと SAS が立てたセッションは、「セッション」の語で同じ物を2つのモデルで持っていた（文脈を分けた根拠）。SAS がモデルとして持たなければ、競合そのものが消える | ddd：bounded-context「競合するモデルは、同じ区切られた文脈の中に実装しない」・ ubiquitous-language「1つの用語に1つの意味」 |
| 目的やセッションの続き具合は、SAS と結果的に整合すればよい情報。境界の外に置き、ID で参照する | ddd：aggregate「結果的に整合していればよい情報か ・ 「はい」なら：境界の外側…に置き、ID で参照する」 |
| SAS が使う側に出すのは、ID と schemas/v1 の CLI の出力だけにする。内部の DB の形を変えても使う側に響かない | ddd：open-host-service「外部に公開するインターフェースを、内部のモデルから分離する」 |
| やり直し ・ 時間切れ ・ 二重実行の防止は1回の CLI の中で閉じる。request_id で1回だけ処理し、回数の上限とタイムアウトを持てば、表が無くても保てる | platform：rel-04「処理の前に一意の識別子が処理ずみかを確かめ、処理ずみならやり直さずに前の結果を返す」・ rel-05「再試行の回数に上限を設ける」「クライアントタイムアウトを設定する」 |
| SAS の DB はコミットせず、作り直せるデータと決める。消えても主な流れが止まらない作りにする | platform：rel-09「ほかのソースから作り直せるようにする」 |

## Concrete を考えたとき
| 場面 | セッションを管理する場合 | 管理しない場合 |
|---|---|---|
| 使う側の Concrete が独自のセッションを持つ（Agents Workspace の目的ごとのセッション、レビューのセッションなど） | Concrete ごとに欲しいセッションの意味が違う。SAS の4つの型に押し込むか、SAS と使う側で二重に持つことになる | 使う側が自分のモデルで持ち、SAS の記録とは ID（tool_ref ・ launched_ref ・ caller_ref）でつなぐ |
| 残すべき結びつき（目的とセッションなど） | SAS の DB に頼るが、コミットしないので消えうる | 使う側の Concrete のインスタンスに ID を値で書き、Git に残す |
| Concrete の rule を意味で検証する | 呼び出し元のセッションの状態に左右されうる | validate は毎回新しく起動するので、どの Concrete でも同じ条件で判定される |
| 人だけが決める欄（x-decided-by: human） | 変わらない | 変わらない。書く前に断る守りは SAS に残り、どの Concrete にも効く |
| 新しい Concrete を作る人が知ること | SAS のセッションの決まり（型 ・ 再開 ・ 終わり方） | `--session` と `--caller` を渡すことと、出力に返る ID だけ |

## 出てきた経緯（利用者の言葉）
- 「そもそもこのDBってリポジトリ内に持つ必要ってないと思うんだけどね。だってコミットしないし。」
- 「SASでセッション管理は少し重いのかな。。なくても動くには動くのか？？せめてＳＡＳを動かすセッションＩＤさえわかってれば別に-pでおくれるもんな。。」
- 「軽くした形でアドバイザーと反証に回して。そのうえで別のセッションに一つの意見として渡せるようにメモリに記録」

## 意見（ddd ・ platform のアドバイザーとも「直して採る」、fork の反証12件を反映した形）
| 項目 | 意見 |
|---|---|
| 表 | operations ・ field_changes ・ rule_results の3つ。sessions ・ launched_sessions ・ session_turns は外す |
| operations の列 | 今の列に actor（記録。守りではない）・ adapter ・ tool_ref ・ caller_ref ・ launched_ref（SAS が起動したツールのセッション ID）・ attempts を足す。seq ・ worktree ・ tokens は持たない |
| CLI の旗 | `--session`（誰が呼んだか）・ `--caller`（使う側の参照、操作1件ごと）・ `--request-id` ・ `--intent` ・ `--if-match`。`--continue <launched_ref>` と `--fork` は呼び出し元が明示したときだけの任意の旗（能力が無ければ unsupported_capability、validate では invalid_input）。session-close ・ session-status と session-spec の lifetime ・ context ・ new は外す |
| 人だけが決める欄 | 集約 Instance が書く前に断る（今の実装どおり）。環境変数（CLAUDE_CODE_SESSION_ID）は AI と見分ける補助で、守りの根拠にしない |
| validate | 毎回新しく起動し、続けない |
| 形の誤り ・ 時間切れのやり直し | ツールの再開に頼らない。新しく起動し、前の答えと誤りをプロンプトに入れる。上限は直し2回 ・ 時間切れ2回。これでどのツールでも動く |
| SAS を通ったかの判定 | （target_path, after_hash）の組が行にあるか。取り込みの前の値は、そのワークツリーの Git の HEAD の版（無ければ before_unavailable）。0003 の trigger operations_adopt_follows は外す |
| 二重実行 ・ 同時送信 | request_id と launched_ref ごとのファイルのロックを起動の前に取る。取れなければ busy。落ちたあとの再起動は「少なくとも1回」 |
| DB の置き場所 | `<git-common-dir>/sas/sas.db`（Git が無ければ `.sas/sas.db`、db_path で上書き可）。作業ツリーの外 ・ コミットされない ・ ワークツリーで共通 ・ リポジトリと一緒に動き一緒に消える。WAL ・ busy_timeout ・ BEGIN IMMEDIATE（照合待ち）、0700/0600。消えたら空で作り直し「記録が無い」と出す |
| 使う側との関係 | SAS は公開された言葉（ID と schemas/v1 の CLI の出力）だけを出し、使う側は従う側。operation-result に launched_ref と caller_ref を返す |
| 文脈 | 文脈「立てたセッション」は外して「スキーマとインスタンス」1つに戻し、業務領域「意味の操作」はその中のモジュールにする。切り出した根拠（セッションを2つのモデルで持つ）が無くなるため。**ただし 9519cffa で承認済みの決定を変えるので、変更前後を見せて承認を求める。** 別の言葉のずれがあるなら2つのままでよい |
| 入れる時期 | 承認されたら、ステップ3 で sessions を外す移行（actor ・ tool_ref を operations へ写す、seq と閉じたセッションの trigger を外す）を一緒に入れる |

## 利用者の言葉との関係
- 「リポジトリ内に持つ必要はない」：`<git-common-dir>` は作業ツリーの外でコミットされないが、リポジトリのフォルダ（.git）の中ではある。理由を添えて確かめる。
- 「セッション ID さえわかれば -p で送れる」：呼び出し元のセッションへ -p で送るのは使う側（Agents Workspace）の仕事として当てはまる。SAS は呼ばれている最中のセッションには送らない。
- 「無くても動くか」：動く。失うのは session_turns のプロンプトの指紋 ・ 答えの原文 ・ 再開の判断の3つで、前の2つはツール自身の記録を launched_ref で辿れる。

## 残っている確かめごと
- fact-check：SQLite の WAL ・ busy_timeout ・ BEGIN IMMEDIATE の振る舞い、同じ Claude Code のセッションへの同時の `-p --resume`、Claude Code と Codex 以外で ID を指定して送れるか。
- 人の入口の確かさ（TTY ・ 確認の入力 ・ 使う側の UI からの呼び方）は、この意見の外の論点。Agents Workspace の Q1 にも関わる（[[agents-workspace-board]]）。

関連：[[sas-foundation-schemas]]
