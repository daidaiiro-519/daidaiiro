---
name: skills-creator-rebuild-board
description: skills-creator を新しい契約で作り直す作業の現在地
metadata:
  type: project
---

ボード `.brainstorming-board/skills-creator-rebuild/`（Artifact https://claude.ai/artifact/57MCZ3Lk4s56HYUcdsuEh4）は Q1〜Q4 とも決着。

- 現在地（2026-10-11）：ACDR 0166（エンジンが名前を assets/ から引く）と ACDR 0167（新しい skills-creator。古いものは src/skills-creator-v3/ に凍結）を承認 ・ コミット済み。push はしていない
- 次にすること：Q4 で決めた「移したコミットを戻すと基準に戻る」リハーサルを1度行い、そのあとブレストボードを新しい型へ移す
- 移す試験は `cd src/skills-creator && cargo test`。版3 の Skill を移したら tests/baseline-v3/<Skill>.json を消す
- 配布の論点へ送ったもの：tools.json（OS ごとの SHA-256）・ accept と dist ・ scripts/ へ入れる段
- ブレストボードを移す作業はボード brainstorming-board-migration（2026-10-11、1回目。Q1 見た目と操作が承認待ち、Q2 固有のツール ・ Q3 既にあるボードは待ち）。試作は作業用の領域だけにある
