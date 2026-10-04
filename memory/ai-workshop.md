---
name: ai-workshop
description: 下期AI活用ワークショップ（docs/workshops/ai-workshop/）の現在地。企画書とオリエンテーションは承認 ・ push 済み、残りは動画 ・ 教材1の照合 ・ 教材2と3
metadata:
  node_type: memory
  type: project
---

2026-10-02 に、git の履歴 ・ ACDR ・ 作業ツリーと照らして確かめた現在地。

**企画書とオリエンテーション**：ACDR 0032 ・ 0034 ・ 0037 ・ 0078 ・ 0079 ・ 0080 ・ 0081（アンケートと配色）がすべて accepted で、push 済み（全枚の HTML と PDF の出力 9e734a4c まで）。関係するボード workshop-purpose ・ workshop-open-items ・ survey-redesign は全論点が決着。アンケートの設問の正本は `proposal/src/survey.json`、Forms のプロンプトは `proposal/out/forms-copilot-prompts.md`。

**次にすること**
- （済）オリエンテーションの音声と動画：2026-10-04 に作り直し、字幕を焼き込んだ（80ea3627 まで push 済み、22分30秒、1280×816、字幕は `out/videos/00-start.vtt` と同じ）。読みの辞書 `src/narration/lexicon.pls`。著者名は読み上げない
- 教材1のスライドと原稿の照合の続き：`material-1/src/review-log.json` は教材1の L1-S3 まで（2026-09-29）。手順は slide-deck の review と `material-1/README.md`
- 教材2（ドメイン駆動設計編、試作 `material-2/`）と教材3（道具編）：並びと題材はボード material-next で決着済み

**未解決**
- ACDR の番号の重複：0032 ・ 0036 ・ 0037 ・ 0081 が、ワークショップの記録と別の記録で2件ずつある。付け替えるかは利用者の判断待ち
- 前の記録から引き継いだ未確認の事項（この日には確かめていない）：Forms のセクションの表示、回答を本人へ送る方法、判定基準の文書、開始時期の部長の了承
- ご意見の画像 IMG_0596〜0599 と feedback-reply.md は、公開リポジトリのためコミットしていない
