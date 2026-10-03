---
name: skills-creator-contract-board
description: skills-creator を契約中心に作り直す作業。ACDR 0096〜0100 まで承認・コミット済み（未 push）
metadata:
  node_type: memory
  type: project
---

2026-10-03 の現在地。

**ボード**：`.brainstorming-board/skills-creator-contract/`（acc3fe50）。Artifact https://claude.ai/artifact/CApismLvSiMne1jqzNzawz 。ローカルは 8752 で配っていた。

**決まったこと**：契約 ＝ ディレクトリ構成 ・ CLI のインターフェース（tool.json に build ・ test ・ format を足して contract 3）・ テストケース（1件1ファイル、リファレンス実装のテスト1件ごとに同名、比較除外は JSON Pointer、スキーマ検査のエラー文は比較しない）。実装は Rust のリファレンス実装1組（sample/rust/）だけを持ち、ほかの言語は AI が移植してテストケースに合格させる。共通ツールは各 Skill に複製する。実装上の前提は contract/assumptions.json。

**試作**：trial/ にテストケース12件と run.py。Rust ・ 複製3つ ・ AI が Rust だけから移植した Go（trial/go-skill、2,442行）で全件合格、古い複製（platform-advisor）で8件不合格。

**ACDR**：0096（e291e2bb）・ 0097（49ba61bf）・ 0098（aade6879、no-more-spaghetti を除く13件を版3）を 2026-10-03 に承認・コミット。未 push。
**0099**（承認 ・ コミット済み 6dc358d4、未 push）https://claude.ai/artifact/YKuCVyqUJ9daoXjSX9qwJv ── 挙げた15の Skill をすべて契約に揃える。ACDR 0043（すべての Skill が references の4つの道具を持つ）の段4〜5 で残っていた session-memory と skill-router に道具を持たせ、session-memory の雛形を templates.json にした。check と構成の契約から、道具を持たない Skill の免除を外した。platform と ux の道具を置き直し、全件に CLI のテストを置き、build に --force。15件とも test と check（テストケース39件）に合格、verify も合格。

**0100**（承認 ・ コミット済み 9276a705、未 push）https://claude.ai/artifact/CnyY51AXC7mbLcLEDjHtNQ ── acdr の1枚の下半分を直す（「面」「まとまり」をやめて「ファイル」「変更箇所」、ファイルの種類によらず同じ帯と一覧、白い枠、削除だけの変更箇所にも印）。廃語に「この面」を足した。0099 はこの見た目で組み直した。

**次**：push を利用者に聞く。platform-advisor と ux-advisor は、それぞれブレストボードを立てて原典の選び直しから判断基準を作り直す。qa-advisor は利用者の返事待ちが3件（シラバスに無い概念を足すか ・ 配線表の qa→ddd の2行 ・ 回答にゲート1をかける手順を SKILL.md に入れるか）。no-more-spaghetti の廃止は別のボードの作業。doc-writing-skills の format の不合格は、ほかのセッションの編集中のテストが原因。

**保留**：brainstorming-board の固定の文言に残る廃語（代償 ・ 要る）と和語の述部を直す。
