---
name: advisor-rebuild
description: 助言型の Skill（advisor）を原典の頁と照合して作り直す作業の現在地。usecase ・ ddd は完了、qa も完了（テストの臭い15件を足した）。platform ・ ux はブレストボードを立てて作り直す
metadata:
  node_type: memory
  type: project
---

2026-10-02 夕方の現在地。

**完了（main にコミット済み、未 push）**：ACDR 0084 ・ 0086 ・ 0087（usecase）・ 0088 ・ 0089（ddd、dddc0db2）・ 0090（d68f82b9、雛形の欠陥 ── 受け入れ検査が原典の名前を複製と数えない ・ スキーマの差し込み場所の検出 ・ validate が根拠の id と引用を判断基準に照らす ・ get --omit ・ 図の辺の名前を矢印の上に ・ 旗の位置 ・ Go と C# の JSON の書き出し。5言語の突き合わせは各582件で不一致0）・ 0091（249d44a1、qa-advisor を JSTQB FL シラバス日本語版 V4.0.J02 から41件で作り直し、目的を「テストを組み立て、十分かを判定し、継続して運用するための判断」に置いた）。push は利用者に確認してから。

**次**：利用者の指摘「言語ごとに雛形を置く方針が筋が悪い。どの言語でも満たす契約 ・ ディレクトリ構成 ・ 擬似言語の雛形にすべき（Rust でも Go でも書きたい）」を受け、skills-creator を契約中心に作り直すブレストボードを立てる（利用者の了承済み）。論点の候補：契約に何を置くか ／ 擬似言語の雛形と正解の事例の形 ／ 既存11の Skill と5言語の雛形の移し方 ／ references の道具を各 Skill に複製するか共通の1つにするか。platform ・ ux の作り直しはその後。

**qa にテストの臭いを足す作業**：ACDR 0102 で完了（判断基準56件、受け入れの検査7件に合格、試しの相談 archive/trial/judgment-erratic.json）。配線表に「テストを書いたら必ず qa」の行は足さない
- **引き継ぎ**：テストを書くときに臭いを作らない規則（ガードレール）は、モデリング ・ コーディングの Skill が持つ（利用者の判断）。no-more-spaghetti に代わる Skill を設計するボード（usecase-modeling-coding）で、ガードレールに入れ、困ったときは qa-advisor の判断基準を引く形にする
- 決着済み：配線表の qa→ddd の2行は消した（ACDR 0101）。回答のゲート1は SKILL.md に入れない（advisor は他の Skill の名前を書けない。CLAUDE.md の規則で Orchestrator が当てる）。ミューテーションテストは原文を取得できないので足さない

**次**：platform ・ ux はそれぞれブレストボードを立てて、原典の選び直しから判断基準を作り直す（契約は ACDR 0099 で揃った）。

**未着手の欠陥**：図の role を描かない ・ ux-advisor の validate（CLI の `validate --kind answer --file` がファイルを検査しない ・ MCP が指摘のあるときも ok を返す）── ux は作り直しで扱う。usecase の以前の試しの回答3件（scratchpad）は、新しい validate で旧い引用と id の誤りが見つかる。

**置き場所**：qa の原典 ・ ノート ・ 判断基準の下書き ・ 指示書は `.claude/skills/qa-advisor/references/archive/`（git の外）。確認用の頁はローカル 8749（/qa/ ・ /ddd/）、ACDR は 8743。
