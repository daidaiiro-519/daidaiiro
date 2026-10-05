---
name: schema-driven-build-board
description: ブレストボード「schema-driven を Rust で作る ── アーキテクチャと技術スタック」の現在地
metadata:
  type: project
---

`.brainstorming-board/schema-driven-build/`（Artifact https://claude.ai/artifact/DZ86giZdRp2vvvx9MYwz4a ）。schema-driven-base の次にすること3。

現在地（2026-10-05、2回目、未 push）：論点1〜3 決着（739a19db）。ACDR 0121（ヘキサゴナルで組む）・ 0122（crate と版）を承認待ち（作業ツリーにあり未コミット）。番号 0119 ・ 0120 はほかのセッションが使っている（0119 の index.html をこちらの glob で一度組み直してしまった）。ACDR 0121 ・ 0122 ・ 0123（no-more-spaghetti の廃止）承認 ・ コミット済み。schema-driven の骨組みを新しく起こした（5ade7814：.claude/skills/schema-driven/、core ・ adapters ・ cli ・ mcp、Cargo.lock は ACDR 0122 の版、clippy が core の std::fs を止めることを確かめた）。scaffold は使わない（refs.rs は schema-driven の契約の先行の試しなので、schema-driven は複製を持たない側）。qa-advisor の指摘（突き合わせの道具 ・ 回ごとの完了の条件の累積 ・ system は CLI から ・ 試作の出力を凍結 ・ UC-4 を2回目へ ・ UC-0）は1回目の前に片付ける。集約のレベルが BC-1 の最初のサブドメインから出る欠陥（concrete7 の impl_of）は後で ACDR。次は1回目 インスタンスの読み書き
- 論点1 アーキテクチャ A（組み直し2回目）：ヘキサゴナル。core（schema-driven-core：domain ・ application ・ ports）と adapters（schema-driven-adapters：inbound ・ outbound）と cli ・ mcp。業務の分け方は core の中の名前に出す。向きは Cargo の依存と clippy（core で std::fs を禁じる）。sd_ は slide-deck の接頭辞なので使わない。no-more-spaghetti は廃止、.coding-rules は持ち主なし。要求：skills-creator 自身もヘキサゴナルか確かめる
- 論点2 技術スタック A：最新の版（serde_json 1.0.151 ・ jsonschema 0.58.5 ・ jmespath 0.5.0 ・ json-patch 4.2.0 ・ sha2 0.11.0 ・ rmcp 3.5.0 ・ tokio 1.53.2）、ネットワークの機能を外す、版は正本の Cargo.lock。原文は sources/（fact-check で14件照合）
- 論点3 どこに書くか：待ち（宣言に書かない。Cargo と ACDR。実装から文脈を指す場所は usecase-modeling-coding で）
- 要求：雛形の4層は手続き的な変換の Skill に当て、ドメインモデルの Skill はポートとアダプタ、を ACDR で起こす
- `.brainstorming-board/README.md` に init が1行足した（ほかのセッションの変更と同居しているのでコミットしていない）
- ACDR 0124（述部を和語にも置き換えない）承認 ・ コミット済み
