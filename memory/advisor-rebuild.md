---
name: advisor-rebuild
description: 助言型の Skill（advisor）を原典の頁と照合して作り直す作業の現在地。usecase と ddd は完了 ・ 未 push、qa は作業中、次は platform → ux
metadata:
  node_type: memory
  type: project
---

2026-10-02 に、git の履歴 ・ ACDR ・ 作業ツリーと照らして確かめた現在地。

**完了（main にコミット済み、未 push）**：ACDR 0084（雛形の Rust を整形済みに）・ 0086（usecase-advisor の判断基準28件）・ 0087 ・ 0088（試しの相談で見つかった欠陥）・ 0089（ddd-advisor を『ドメイン駆動設計をはじめよう』の頁と照合して作り直し）。origin/main より7件先にある（この記録のコミットを含む）。push は利用者に確認してから。

**作業中**：qa-advisor。原典を JSTQB の Foundation Level シラバスに替えて作り直している（`references/criteria.json` ほかが未コミットで変更中、2026-10-02 時点。ACDR は未作成）。別のセッションが進めている可能性があるので、触る前に作業ツリーを確かめる。

**次**：qa → platform → ux。手順は ddd と同じ（原典のノート → 判断基準 → 図 → 雛形から生成 → accept → 試しの相談3件 → ACDR）。手本は `.claude/skills/ddd-advisor/references/archive/BRIEF-criteria.md`（git の外）。

**未決 ・ 未着手**
- 回答に文書の判定（ゲート1）を当てる手順を SKILL.md に入れるか（利用者の判断待ち）
- 試しの相談で見た道具の欠陥：get の出力が大きい ・ 図の role を描かない ・ Go の view --json が HTML を escape する
- ux-advisor の validate の欠陥（2026-10-02 に見つかった）：CLI の `validate --kind answer --file` がファイルを検査しない ・ MCP が指摘のあるときも ok を返す
- 確認用の頁はローカル 8749（/ddd/ ・ /review/ ・ /trial/）、ACDR は 8743
