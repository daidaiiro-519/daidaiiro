---
name: advisor-rebuild
description: 助言型の Skill（advisor）を原典の頁と照合して作り直す作業の現在地。usecase ・ ddd ・ qa は完了 ・ 未 push。platform ・ ux は、skills-creator を契約中心に作り直すボードの決着後
metadata:
  node_type: memory
  type: project
---

2026-10-02 夕方の現在地。

**完了（main にコミット済み、未 push）**：ACDR 0084 ・ 0086 ・ 0087（usecase）・ 0088 ・ 0089（ddd、dddc0db2）・ 0090（d68f82b9、雛形の欠陥 ── 受け入れ検査が原典の名前を複製と数えない ・ スキーマの差し込み場所の検出 ・ validate が根拠の id と引用を判断基準に照らす ・ get --omit ・ 図の辺の名前を矢印の上に ・ 旗の位置 ・ Go と C# の JSON の書き出し。5言語の突き合わせは各582件で不一致0）・ 0091（249d44a1、qa-advisor を JSTQB FL シラバス日本語版 V4.0.J02 から41件で作り直し、目的を「テストを組み立て、十分かを判定し、継続して運用するための判断」に置いた）。push は利用者に確認してから。

**次**：利用者の指摘「言語ごとに雛形を置く方針が筋が悪い。どの言語でも満たす契約 ・ ディレクトリ構成 ・ 擬似言語の雛形にすべき（Rust でも Go でも書きたい）」を受け、skills-creator を契約中心に作り直すブレストボードを立てる（利用者の了承済み）。論点の候補：契約に何を置くか ／ 擬似言語の雛形と正解の事例の形 ／ 既存11の Skill と5言語の雛形の移し方 ／ references の道具を各 Skill に複製するか共通の1つにするか。platform ・ ux の作り直しはその後。

**利用者の返事待ち**
- qa にテストの臭い（Meszaros、条件を満たす）とミューテーションテスト（名付けた論文を取得できたら）を1件ずつ足すか
- 配線表の2行（「qa-advisor（弱いテストの原因が…疑いのとき）」「qa-advisor（分類が入力に無いとき）」→ ddd-advisor）を残すか消すか。新しい qa の手順に該当する場面が無い
- 回答にゲート1を当てる手順を SKILL.md に入れるか

**未着手の欠陥**：図の role を描かない ・ ux-advisor の validate（CLI の `validate --kind answer --file` がファイルを検査しない ・ MCP が指摘のあるときも ok を返す）── ux は作り直しで扱う。usecase の以前の試しの回答3件（scratchpad）は、新しい validate で旧い引用と id の誤りが見つかる。

**置き場所**：qa の原典 ・ ノート ・ 判断基準の下書き ・ 指示書は `.claude/skills/qa-advisor/references/archive/`（git の外）。確認用の頁はローカル 8749（/qa/ ・ /ddd/）、ACDR は 8743。
