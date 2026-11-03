---
name: agents-skills-creator
description: 作りたい作業を思いついたとき、使われて外れた具体で直したいとき、配りたいときに使う。作るか直すか、1つの Skill で終わるか複数の Skill を束ねる Agent かを見分け、目的の仮説 ・ 完成の像 ・ 構造の像を見せて作り、直し、確かめ、配る。配布の取り下げは影響を示し、人が打つ。作った Skill の作業そのもの ・ 承認 ・ コミットはしない。
compatibility: sas（Semantic Agent Schema）が PATH にあること
---

## 目的

AI ツールの会話の中で、作業を Skill か Agent に任せたい人が、作りたい作業を思いついたとき、使われて外れた具体を直したいとき、配りたいときに、作りたい作業を言うだけで、使える Agent Skill か Agent が手元にある状態を得る。

## 役割

- 受け持つ：Skill か Agent を作る
- 受け持つ：Skill か Agent を直す
- 受け持つ：Skill か Agent が使える状態かを確かめる
- 受け持つ：Skill か Agent を配る
- 受け持つ：Skill か Agent の配布の取り下げの影響を示し、人が打つコマンドを返す
- 受け持たない：作った Skill ・ Agent の作業そのものをする
- 受け持たない：承認を書く（承認は人が書く）
- 受け持たない：コミットする

## 事前条件

- 作りたい作業か、使われて外れた具体を言える
- Git のリポジトリの中で使う

## 入力と出力

- 入力：作りたい作業（文（最近それを手でした場面があればなおよい））
- 入力：使われて外れた具体（文（頼んだこと ・ 出たもの ・ 期待したもの））
- 出力：Skill か Agent のフォルダ（skill.json ・ SKILL.md（Agent なら agent.json ・ AGENT.md ・ codex.toml） ・ skills/<name>/ か agents/<name>/）
- 出力：配布先の置き場所（Skill はフォルダへのリンク、Agent は定義の写し ・ .claude/skills ・ .agents/skills ・ .claude/agents ・ .codex/agents（リポジトリか HOME の下））

## 参照する資料

- 作る流れの手引き（references/create-flow.md）：見分け方 ・ 最初の返事で見せる4つ ・ 聞く所の見分け方 ・ 書かせて確かめる順 ・ 承認と配布 ・ 直し方。形：Markdown。手順 1 ・ 2 ・ 3 ・ 4 ・ 5 ・ 6 ・ 7 ・ 8 ・ 9 で読む

## 道具

- skills create（scripts/skills）：Skill か Agent を作る。受け取る：作りたい作業。返す：Skill か Agent のフォルダ
- skills update（scripts/skills）：Skill か Agent を直す。受け取る：使われて外れた具体。返す：Skill か Agent のフォルダ
- skills write-skill-md（scripts/skills）：Skill か Agent を作る
- skills validate（scripts/skills）：Skill か Agent が使える状態かを確かめる
- skills distribute（scripts/skills）：Skill か Agent を配る。返す：配布先の置き場所
- skills withdraw（scripts/skills）：Skill か Agent の配布の取り下げの影響を示し、人が打つコマンドを返す
- agents create（scripts/agents）：Skill か Agent を作る。受け取る：作りたい作業。返す：Skill か Agent のフォルダ
- agents update（scripts/agents）：Skill か Agent を直す。受け取る：使われて外れた具体。返す：Skill か Agent のフォルダ
- agents write-agent-md（scripts/agents）：Skill か Agent を作る
- agents validate（scripts/agents）：Skill か Agent が使える状態かを確かめる
- agents distribute（scripts/agents）：Skill か Agent を配る。返す：配布先の置き場所
- agents withdraw（scripts/agents）：Skill か Agent の配布の取り下げの影響を示し、人が打つコマンドを返す

## 外の道具

- sas（>=0.1.0, <0.2.0）：Skill ・ Agent の項目を書かせ、確かめさせ、記録を読む

## 資源

- Skill の Concrete（assets/skill.schema.json）：Agent Skill の項目の形（urn:agents-skills-creator:skill:v1）。形：JSON（SAS の Concrete）（schema：urn:semantic-agent-schema:published-language:abstract:v1）
- Agent の Concrete（assets/agent.schema.json）：Agent の項目の形（urn:agents-skills-creator:agent:v1）。形：JSON（SAS の Concrete）（schema：urn:semantic-agent-schema:published-language:abstract:v1）

## 手順

1. 作るか直すか、1つの Skill で終わるか複数の Skill を束ねる Agent かを見分ける（使う道具：skills withdraw ・ agents withdraw）（読む資料：作る流れの手引き）（完了条件：見分けた結果と理由を作る人に示した）
   - 1a 束ねたい Skill が足りない → 足りない Skill を先に Skill として作る（ステップ1へ戻る）
   - 1b 使わなくしたい → withdraw で取り下げの影響と人が打つコマンドを示し、人が打つ（止める）
2. 書く手順を --semantic で受け取り、返る flow に従って、目的の仮説 ・ 完成の像 ・ 構造の像と推奨付きの問いを見せる（使う道具：skills create ・ skills update ・ agents create ・ agents update）（読む資料：作る流れの手引き）（完了条件：作る人が目的の仮説を承認し、問いに答えた）
3. SAS に項目を書かせる（使う道具：skills create ・ skills update ・ agents create ・ agents update）（読む資料：作る流れの手引き）（完了条件：sas の結果が created か updated である）
4. SKILL.md（Agent なら AGENT.md と codex.toml）を作る（使う道具：skills write-skill-md ・ agents write-agent-md）（読む資料：作る流れの手引き）（完了条件：write-skill-md（Agent なら write-agent-md）の結果に、書いたファイルの経路がある）
5. validate で確かめる（使う道具：skills validate ・ agents validate）（読む資料：作る流れの手引き）（完了条件：validate の終了コードが 0 である）
6. 意味の rule を別の会話で判定させ、判定を SAS に渡す（読む資料：作る流れの手引き）（完了条件：validate の承認の材料の approvable が true である）
7. 別の会話で例を動かし、実際の答えを完成の像と並べて見せる（読む資料：作る流れの手引き）（完了条件：どの例にも、実際の答えが期待と並んでいる）
   - 7a 例の実際の答えが期待と違う → 外れた具体として直す層を選んでもらう（ステップ2へ戻る）
8. 承認のコマンドを作る人に示し、人に承認を打ってもらう（読む資料：作る流れの手引き）（完了条件：skill.json（Agent なら agent.json）に approved がある）
9. 作る人が選んだ配布先に置く（使う道具：skills distribute ・ agents distribute）（読む資料：作る流れの手引き）（完了条件：distribute の placed-readable が verified である）

## 守ること

- 目的は AI が仮説として示し、作る人が承認して決める（問題の輪郭を言葉にするのは人の領域で、AI が決めると作る人の作業とずれる）
- 配布の取り下げは、影響を示したあと人が打つ（影響を見ずに取り下げると、その Skill ・ Agent を使っている人やツールが黙って使えなくなる）

## 最低保証

- 作る人が承認するまで、配布先は変わらない
- 既にある Skill ・ Agent は、直すと言われた所のほかは変わらない
- どこで止まっても、作るフォルダと配布先の外は変わらない

## 完了条件

- C1 作ったものが validate を通る（確かめ方：skills validate か agents validate の終了コードが 0）
- C2 作る人が完成の像を見て承認した（確かめ方：skill.json か agent.json に approved がある）

## 例

- E1（人が直接）：Claude Code のサブエージェントの頁の URL と、項目名3つを照合したいという作業の説明がある。「外の原文と、書き写す項目名を照合する Skill が欲しい」
  → 1つの Skill で終わると見分け、目的の仮説 ・ 完成の像（URL と項目名を渡すと、ある ・ 無いが返る）・ 構造の像を見せ、承認のあと fact-check のフォルダに skill.json と SKILL.md ができ、validate が通る
- E2（人が直接）：6つのアドバイザーの Skill が skills/ にある。「設計で迷ったとき、アドバイザーたちに同時に聞いてまとめる Agent が欲しい」
  → 複数の Skill を束ねる Agent だと見分け、呼ぶ Skill と受け渡しの像を見せ、承認のあと consult-advisors のフォルダに agent.json と AGENT.md ができ、agents validate が通る
- E3（人が直接）：fact-check が、全角の空白を含む項目名に「無い」と返した。「この外れた具体で fact-check を直して」
  → 外れた具体が examples に足され、直す層の案が推奨付きで出て、承認のあと「層：examples」の理由で直され、validate が通る
- E4（人が直接）：fact-check が .claude/skills に配られている。「fact-check を使わなくしたい」
  → 配布先の置き場所（.claude/skills/fact-check）と、人が打つ withdraw のコマンドが示され、AI は外さない
