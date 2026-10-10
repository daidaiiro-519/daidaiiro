---
name: skills-creator-rebuild-board
description: skills-creator を新しい契約でゼロから作り直すボードの現在地
metadata:
  type: project
---

ボード `.brainstorming-board/skills-creator-rebuild/`（Artifact https://claude.ai/artifact/57MCZ3Lk4s56HYUcdsuEh4）。

- 現在地（2026-10-11）：Q1〜Q4 すべて決着（ボードは完了）
- Q3 の試作の check と試験データ22件は、セッションの作業領域にだけある（コミットしていない）。check を作るときは答えの項目 id から作り直す
- 次にすること：ACDR 1件で新しい skills-creator を作る（型 v1 のスキーマ2つ ・ src/skills-creator/ の実行ファイル ・ 試験データ ・ 移す試験 ・ 古いものを src/skills-creator-v3/ へ凍結して置き換え）。そのあとブレストボードを移す。そのあと agents creator（「Agent」が何を指すかを利用者に聞く）
