---
name: usecase-modeling-coding-board
description: ブレストボード「ユースケース抽象設計モデリングとユースケース抽象コーディング」の現在地。9月の usecase-schema-driven のボードと合わせて考える
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/usecase-modeling-coding/`（Artifact https://claude.ai/artifact/SfY1RnNxrmAoPS2N3CDDGm 、ローカルの serve は 8751）。no-more-spaghetti（名前は廃止）に代わる一式（モデリングの Skill ・ コーディングの Skill ・ 両者を呼び出すエージェント、OSS はプラグインで配る）を決めるボード。

**9月の usecase-schema-driven のボード（`.brainstorming-board/usecase-schema-driven/`） と合わせて考える**（利用者の依頼 2026-10-02）。宣言6種類 ・ 射影 ・ 引き算などの9月の決定が土台で、データの持ち方（スキーマ）・ 射影 ・ 逆参照 ・ 差分 ・ ドリフトの引き算はスキーマ駆動の道具が担当する。2つの Skill はその道具を使う側である。

現在地（2026-10-02、7回目）：
- Q1（2つの Skill の境界。ポートとアダプターを外枠に固定）は決着
- Q2（ドメインの宣言の7つの欄）・ Q3（実装の規則の主体＝プロダクト）・ Q4（専用エージェント）・ Q6（目的の連鎖として見せる）は回答待ち
- Q5（OSS の配り方）は Q4 の決着後に開く。名前は Q1〜6 の決着後に既存の語から決める

持ち越し：ACDR 0085（no-more-spaghetti の inward の不備の修正、未コミット）は承認待ち。brainstorming-board の定型文言（代償 ・ 要る ・ 持たない）と ux-advisor の validate の不備（CLI が検査しない ・ MCP が常に ok）は、別の ACDR で直す提案のみ。ctxtrace の是正（app 層の新設ほか）と規則ファイル（ctxtrace/.coding-rules/、未コミット）は、このボードの決着後に当てる。
