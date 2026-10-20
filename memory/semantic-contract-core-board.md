---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-20、第95回）

- 決着：Q0 ・ Q1 ・ Q2 ・ Q5 ・ Q6 ・ Q7 ・ Q8 ・ Q9 ・ Q10。回答待ち：Q3（書き直しは 8a601f81 でコミット済み、反証の結果 r102/ddd.json は未提示）・ Q4。
- まとめのページ（Artifact RJgEw765n5vNrLinAbKUod）を基準にボード全体を照らし、食い違い49件を直した（bb2824ee）。一覧は Artifact 8T4pVpD5C8uFEFxgEc5oGR。
- 呼び方の決定：x- は「Core の拡張項目」「Concrete の拡張項目」。「語彙」は glossary だけを指す。validator は Q5 の語（機械 ・ AI ・ 人）。
- ボードの Artifact は CdRCEdAdiSnUrBSc3svC5j（site/semantic-contract-core.html をそのまま出す）。

## 次にすること（この順）

1. Q0 の検証の画面（q0-screen.html）が幅 358px で語の途中で折り返すのを直す
2. Q0 の Kiro の旗（--trust-tools）を fact-check で原典と照らして直す
3. Q3：反証の結果を採る ・ 採らないに分けて見せ、食い違い2件（子のセッション ・ アダプター）と一緒に直して開く
4. Q4：食い違い4件と Q0 のインターフェース（HTTP ・ WASM）
5. 実行できるモデル measured/model/ を案に合わせて作り直す
