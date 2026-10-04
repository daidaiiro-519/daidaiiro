---
name: schema-driven-build-board
description: ブレストボード「schema-driven を Rust で作る ── アーキテクチャと技術スタック」の現在地
metadata:
  type: project
---

`.brainstorming-board/schema-driven-build/`（Artifact 未公開。2026-10-05 は公開の日次上限に達した。次の公開で出す）。schema-driven-base の次にすること3。

現在地（2026-10-05、1回目、90e2fc5c、未 push）：
- 論点1 アーキテクチャ A：外側をポートとアダプタ、中の実装の型は業務領域ごと。crate 9つ（sd_domain ・ sd_published ・ sd_instances ・ sd_render ・ sd_transcribe ・ sd_fs ・ sd_jmespath ・ sd_cli ・ sd_mcp）。向きは Cargo の依存と clippy（std::fs を禁じる）で止める。no-more-spaghetti は廃止、.coding-rules は持ち主なし
- 論点2 技術スタック A：最新の版（serde_json 1.0.151 ・ jsonschema 0.58.5 ・ jmespath 0.5.0 ・ json-patch 4.2.0 ・ sha2 0.11.0 ・ rmcp 3.5.0 ・ tokio 1.53.2）、ネットワークの機能を外す、版は正本の Cargo.lock。原文は sources/（fact-check で14件照合）
- 論点3 どこに書くか：待ち（宣言に書かない。Cargo と ACDR。実装から文脈を指す場所は usecase-modeling-coding で）
- 要求：雛形の4層は手続き的な変換の Skill に当て、ドメインモデルの Skill はポートとアダプタ、を ACDR で起こす
- `.brainstorming-board/README.md` に init が1行足した（ほかのセッションの変更と同居しているのでコミットしていない）
- 公開待ち：このボード、schema-driven の宣言の頁（GPSBWs5TVjxN8wKNtXG4EL）
