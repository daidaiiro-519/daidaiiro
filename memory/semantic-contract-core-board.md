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
  - TLA+ ・ Z3 から採るのは「決まりは状態で書く」。Abstract の根の write に read-perspectives を足す案。
- シミュレーション第22版（Jhj2PvMe53KNCHVGHXWR5V、元は scratchpad/sim20/build.py）。JSON Schema の読み方 BH7hvc6RCqEzvLT7ru86Fh、能力の一覧 XSP4EXyAP9bKRBr6DyGrKv。
- 差分第7版（UoM9gCcp6BAxTdvdpwNBNa）は、上の決定より前のもの。そのまま当てない。

## 次にすること（この順）

1. 上流から開き直して直す：Q0（#13 の書き方）→ Q1 ・ Q7（x-delegates の中身と kind、by との対応、呼ばれた子は呼ばない）→ Q11（親のセッションの立て方、ID のでき方）→ Q3（層と部品、見出し、描いて起動する手順、返る値の形だけ）。論点ごとに差分をアーティファクトで出して承認を受ける。
2. 未決：URN の持ち主 core の名前、Concrete の URN の「種類」の名前、Q1 #7 ・ #22 の例外。
3. そのあと Q4 ・ 実行できるモデルの作り直し。
