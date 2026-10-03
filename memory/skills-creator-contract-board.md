---
name: skills-creator-contract-board
description: skills-creator を契約中心（ディレクトリ構成 ・ CLI のインターフェース ・ テストケース ＋ Rust のリファレンス実装1組）に作り直すボード。5論点とも決着、次は ACDR A ・ B ・ C
metadata:
  node_type: memory
  type: project
---

2026-10-03 の現在地。

**ボード**：`.brainstorming-board/skills-creator-contract/`（acc3fe50）。Artifact https://claude.ai/artifact/CApismLvSiMne1jqzNzawz 。ローカルは 8752 で配っていた。

**決まったこと**：契約 ＝ ディレクトリ構成 ・ CLI のインターフェース（tool.json に build ・ test ・ format を足して contract 3）・ テストケース（1件1ファイル、リファレンス実装のテスト1件ごとに同名、比較除外は JSON Pointer、スキーマ検査のエラー文は比較しない）。実装は Rust のリファレンス実装1組（sample/rust/）だけを持ち、ほかの言語は AI が移植してテストケースに合格させる。共通ツールは各 Skill に複製する。実装上の前提は contract/assumptions.json。

**試作**：trial/ にテストケース12件と run.py。Rust ・ 複製3つ ・ AI が Rust だけから移植した Go（trial/go-skill、2,442行）で全件合格、古い複製（platform-advisor）で8件不合格。

**次**：ACDR A（契約とテストケース31件を追加、check にテストケースの実行）→ B（テンプレートを Rust 1組に限定、4言語と profile を削除、verify の言語間比較を置き換え）→ C（11の Skill の tool.json を contract 3、文書を書き直す）→ platform ・ ux の作り直し。

**保留**：brainstorming-board の固定の文言に残る廃語（代償 ・ 要る）と和語の述部を直す。
