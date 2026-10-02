---
name: schema-driven-base-board
description: ブレストボード「スキーマ駆動の道具（抽象の基盤）」の現在地
metadata:
  node_type: memory
  type: project
---

`.brainstorming-board/schema-driven-base/`（Artifact https://claude.ai/artifact/2SFURsbhMHgDHg9Vj5oLgG ）。スキーマ駆動の道具を、どの concrete にも使える抽象の基盤として形を決める。ユースケース駆動はその concrete の1つで、ボード usecase-modeling-coding と ACDR 0092 の決定は concrete の側の決定として扱う。

現在地（2026-10-03、3回目）：
- 論点1を B″ に組み直し、回答待ち。抽象の基盤の契約は7つ ── 操作の4つ（作成 ・ 取得 ・ 更新 ・ 削除）と形の3つ（検証 ・ 描画 ・ 案内）。取得は JMESPath、更新と削除は JSON Patch（RFC 6902、位置は JSON Pointer）。更新と削除は、適用後の実体が検証を通過したときだけ書き込む
- ユースケース駆動の concrete は同じ7つを特化した表現で持ち、ドリフト検知（テスト仕様とテスト実装）の契約を足す
- 1回目（kind ・ id ・ name ・ body と JSONPath）と2回目（形の3つだけ）は前の答えとしてボードに残る
- 論点2（concrete の契約）・ 3（描画の持ち方）・ 4（配り方と名前）は待ち
- Rust の crate（jmespath 0.5.0 ・ json-patch 4.2.0）は、仕様への準拠の範囲をまだ照合していない

検証と描画は既存の Skill の references の契約の版2（get ・ validate ・ view ・ x-view）、案内は no-more-spaghetti の schema-meta.schema.json（x-prompt の read ・ write、x-generates）に実装がある。
