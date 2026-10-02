---
name: usecase-modeling-coding-board
description: ブレストボード「ユースケース抽象設計モデリングとユースケース抽象コーディング」の現在地。9月の usecase-schema-driven のボードと合わせて考える
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/usecase-modeling-coding/`（Artifact https://claude.ai/artifact/SfY1RnNxrmAoPS2N3CDDGm 、ローカルの serve は 8751）。no-more-spaghetti（名前は廃止）に代わる一式（モデリングの Skill ・ コーディングの Skill ・ 両者を呼び出すエージェント、OSS は AI ツールに依存しない本体と、変換機が出すツールごとの形で配る）を決めるボード。

**9月の usecase-schema-driven のボード（`.brainstorming-board/usecase-schema-driven/`） と合わせて考える**（利用者の依頼 2026-10-02）。宣言6種類 ・ 射影 ・ 引き算などの9月の決定が土台で、データの持ち方（スキーマ）・ 射影 ・ 逆参照 ・ 差分 ・ ドリフトの引き算はスキーマ駆動の道具が担当する。2つの Skill はその道具を使う側である。

コーディングの Skill は、スキーマ駆動の道具を参照しない（利用者の指摘 2026-10-02）。規則とテストの書き方はプロダクトのアーキテクチャと技術スタックから作り、シナリオはエージェントが入力として渡す。シナリオの管理と引き算は、道具のユースケース駆動の concrete が担当する。
シナリオという語はユースケースにだけ使う。集約は不変条件と事前 ・ 事後条件を、ドメインサービスは事前 ・ 事後条件を持ち、テストはその条件と結ぶ（2026-10-02、宣言の形も合わせて直す前提。9月の試作の ops[].nodes を外す改訂は、このボードの決着後に ACDR で提示する）。
Q7 は12回目で、テストを10の種類（宣言から7つ、実装の定義から3つ）にした。宣言に値オブジェクトの種類と、保証 ・ 業務イベント ・ サーガ ・ 公開された言葉の欄を足す（9月の宣言の6種類の改訂を伴う）。

現在地（2026-10-02、14回目）：
- Q1 ・ Q2 ・ Q3 ・ Q4 は決着。Q6 ・ Q7 は承認済みで、ctxtrace での外の作業で試した時点で決着にする
- Q5（OSS の配り方：本体は AI ツールに依存しない形〈Agent Skills の仕様の Skill ・ OS ごとの実行ファイル ・ エージェントの中立の宣言〉、ツールごとの形〈Claude Code のプラグイン ・ Codex ・ Cursor ・ Kiro ・ Copilot〉は変換機が出す）は回答待ち。名前は全論点の決着後に既存の語から決める

持ち越し：ACDR 0085（no-more-spaghetti の inward の不備の修正、未コミット）は承認待ち。brainstorming-board の定型文言（代償 ・ 要る ・ 持たない）と ux-advisor の validate の不備（CLI が検査しない ・ MCP が常に ok）は、別の ACDR で直す提案のみ。ctxtrace の是正（app 層の新設ほか）と規則ファイル（ctxtrace/.coding-rules/、未コミット）は、このボードの決着後に当てる。
