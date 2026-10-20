---
name: semantic-contract-core-board
description: 基盤 Semantic Agent Schema（旧称 Semantic Schema Core。Semantic Abstract Schema と Semantic Concrete Schema）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-21）

- 土台を直した：Q0 の目的（構造の機械的な検証を意味の領域まで広げる。承認は2つ。子のセッションは親の文脈を引き継がない）・ Q8（U27 親のセッション、U16 本人の承認、U2 注意を x-prompt に上げる）・ Q9（中核に「目的ごとのセッションの管理」。オーケストレーター＝AGENTS.md ・ CLAUDE.md のような定義）。コミット e11197a1 ・ e9511b75 ・ 0663d847。
- 名前は Semantic Agent Schema、URN は urn:semantic-agent-schema:…（122987dc）。今の実装（schema-driven）との比較はボードから外した。
- ボードの Artifact は CdRCEdAdiSnUrBSc3svC5j。差分は毎回アーティファクトで出して承認を受けてから入れる。

## 次にすること（この順）

1. 論点「目的ごとのセッション」を新しく立てる：セッションの集約の形（目的 ・ 意図 ・ 親子 ・ 再開の ID ・ 状態）、再開か新規かをデータで決める規則、手元だけに置くか、境界の表のセッションの行と I34 ・ I35 ・ I38 の直し
2. 論点「信用の根拠」（人がレビューして承認した x-prompt）を立てるか、上の論点と Q0 で足りるかを決める
3. Q1 に反映（集約9つ目、境界の表、不変条件）
4. Q3：Agents 契約を Abstract の一員に。判定させる AI と作業させる AI、一般「AI と外のツールによる判定」の改名と範囲、Q3 の反証の直し（子のセッションの行は残す）
5. Q4 ・ 実行できるモデルの作り直し
