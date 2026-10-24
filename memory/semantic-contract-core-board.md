---
name: semantic-contract-core-board
description: 基盤 Semantic Agent Schema（Semantic Abstract Schema と Semantic Concrete Schema）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-23）

- ボードの Artifact は CdRCEdAdiSnUrBSc3svC5j。Q3 だけが開いていて、ほかは決着済み（Q11 セッション、Q1 の ER 図まで c1f48c9c）。
- Q3（Agents 契約）の検討で、下流の Q3 から上流の決着済み（Q0 ・ Q1 ・ Q7 ・ Q11）を書き換えていたので、根幹 Q0 から整理し直した。整理のページ：LBLn1Jn6QcKXpq3LBkRWjA。シミュレーション第15版：Jhj2PvMe53KNCHVGHXWR5V。
- 利用者と決めたこと：
  - x-delegates は残す。基盤（Abstract）も Concrete も根に持つ「任せる先の一覧」で、中身は自分の agent の定義（ほかの Concrete の URN は書かない）。kind は judging ・ working ・ program。rule の by: ai → judging、by: machine → program（agent が無ければ基盤のエンジン）。
  - Q0 #13（外から足す検査のプログラム）は残す。文脈の技術スタックの枯れた CLI（Rust なら cargo）を Concrete が program として置く。止めるのは x-prompt の文に特定の言語の道具を書くことだけ。
  - 呼ぶのは親のセッション。呼ばれた子はほかの agent を呼ばず、決めきれない点は needs で親に返す。advisor の根拠は親がインスタンスの欄に書いてから検証し直す。
  - 判定させる AI にも、親が渡すなら引き継ぎを渡す（Q0 #4 のまま）。親のセッションは人 ・ harness が立てても基盤が立ててもよい。
  - x-prompt は、項目を見出しにし、read（means ・ shown ・ parse ・ when）と write を添えて描く。描き方の3つの型（目的のセッションが書く ・ 任せる先が書く ・ 任せる先が判定する）。ビューは任意。
  - セッションの呼び名：オーケストレーターのセッション ・ 目的のセッション（親はオーケストレーターのセッション）・ 任せる先のセッション。Q11 の不変条件①は任せる先のセッションにだけ当てる。
  - JSON Schema の能力はフルに使う。値だけで決まる決まりは if ・ then ・ dependentRequired ・ contains ・ oneOf ・ readOnly などで書き、x-prompt には書けないもの（記録をまたぐ ・ Git を読む ・ 意味の判定）だけを残す。
  - advisor の答えは親が本文に書き足してから再検証。AI の答えは rule ごとの5つの値と問い（外枠は protocol）。AI を呼ぶ書式は見出し7つを Abstract の $defs/agents に。
  - URN の持ち主は core をやめて base。Concrete の URN の種類はインスタンスの kind と同じ名詞（ACDR は urn:semantic-agent-schema:acdr:decision-record:v1）。
  - 手元の DB の案は measured/abstract-draft/db.sql（sessions と session_changes）。I46：同じ目的の目的のセッションは親が違っても1つまで、任せる先のセッションは目的 ・ 親 ・ 役割が同じで1つまで、オーケストレーターのセッションはリポジトリ（手元の DB）で開いているのは1つで、ほかのターミナルは同じ ID で再開する。オーケストレーターは入れ子にしない（決めた）。
  - TLA+ ・ Z3 から採るのは「決まりは状態で書く」。Abstract の根の write に read-perspectives を足す案。
- 案の正本は .brainstorming/semantic-contract-core/measured/abstract-draft/（README に組み直しと検査の手順）。シミュレーション Jhj2PvMe53KNCHVGHXWR5V、完成イメージ GRqKKpvcij9JmhC26VczG1。JSON Schema の読み方 BH7hvc6RCqEzvLT7ru86Fh、能力の一覧 XSP4EXyAP9bKRBr6DyGrKv。
- 差分第7版（UoM9gCcp6BAxTdvdpwNBNa）は、上の決定より前のもの。そのまま当てない。

## 現在地（2026-10-24 追記）

- 利用者の方針：ドメインモデル ・ スキーマ ・ ポートの契約 ・ DB ・ 期待する振る舞いを1つの「核」（measured/core/）にそろえて合意し、シミュレーションで期待する振る舞いを全部実行して通すことを核ができた印にする。地図 S1m24U6u8LhsU5QR1BJXch（core/map.html）。
- 変更の仕組み（Git）は「対象の変更の前後を受け取る」ポートのアダプターの1つにすぎない。基盤は Git と同じことをしない（利用者の指摘）。change_id は基盤が決める ULID。
- core/agreed.json（合意済み12件）・ core/answers.json（案と未決の9件の答えの下書き）。ddd ・ platform ・ qa ・ usecase の4アドバイザーと反証役に依頼中。結果は core/reviews/*.md。
- 2026-10-24 決着：①基盤はどのファイルも「最後に確定した姿 → いま」の差分を取る（誰が書いたかを問わない。意図に入らない未確定の変更は先に確定する）。②同じ目的のセッションは AI ツールに関係なく同時に開かない。新しく立てるときに古いほうを閉じる（採用）。
- 2026-10-24：費用の式は採らない（他の AI ツールを原典で調べ、トークン課金でしか成り立たない。LTNMGFUXwuXFw9Pr41EgMj）。案：Q11 #3 の形（有効時間内なら再開、切れた ・ 不明なら新規）＋履歴の割合が設定ファイルの値を超えたら新規。既定は置かない。
- 旧案（取り下げ）：再開か新規かを費用で比べ、分かれ目の値を Semantic Agent Schema の設定ファイル（Abstract の一員 settings、利用者 ・ プロジェクト ・ 手元の3段）に置く。完成イメージ 78cZGovzmkqKvpeCH2izXy の3 ・ 4節。整理のページ UcziDsYkZyfS8uhV187QLa。
- 2026-10-24：核の第1段を書いた（measured/core/。model ・ schemas ・ ports.json ・ adapters/sqlite.sql ・ ref ・ behaviors ・ run.py）。変更の記録 ・ 判定 ・ セッション ・ 設定の範囲で58場面 ・ 72回が合格、わざと壊した5か所を検出。対応表90行のうち67行は未確認（検証のエンジン ・ 描画など）。結果のページ core/result.html。合意待ち。
- 次：指摘を統合して答えを直す → v2 に全部を適用して核の5つを書く → 参照の実装でシミュレーション → 合否と核を1ページで出して合意 → ACDR で正本へ。

## 次にすること（この順）

1. 上流から開き直して直す：Q0（#13 の書き方）→ Q1 ・ Q7（x-delegates の中身と kind、by との対応、呼ばれた子は呼ばない）→ Q11（親のセッションの立て方、ID のでき方）→ Q3（層と部品、見出し、描いて起動する手順、返る値の形だけ）。論点ごとに差分をアーティファクトで出して承認を受ける。
2. 未決：Q1 #7 ・ #22 の例外の書き方。
3. そのあと Q4 ・ 実行できるモデルの作り直し。
